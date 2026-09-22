package generated.internal;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.IdentityHashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;
import java.util.regex.Pattern;
import generated.api.Capture;
import generated.api.Diagnostic;
import generated.api.MappingException;
import generated.api.ParseOptions;
import generated.api.ParseResult;
import generated.api.Span;
import generated.api.TokenScanner;
import generated.rt.Diagnostics;
import generated.rt.Input;
import generated.rt.MemoTable;
import generated.rt.ScopeStore;
import generated.rt.Severity;
import generated.internal.Recipe.Value;

/** Per-parse state; transactions preserve syntax diagnostics but undo semantic effects. */
public final class Session {
    @FunctionalInterface public interface Call { Match parse(Session session, Frame frame); }
    public static final class Frame {
        public int c, m;
        public boolean matched, invert, reset = true;
        public int position() { return matched ? m : c; }
        public Frame copy(boolean begin) {
            var f = new Frame(); f.c = c; f.m = begin && reset ? c : m;
            f.matched = matched; f.invert = invert; f.reset = reset; return f;
        }
        public void commit(Frame child) { c = child.c; m = child.m; }
        public void advance(int length) { if (matched) m += length; else { c += length; m = c; } }
    }
    public record Occurrence(String site, String name, Value value) {}
    public record Trace(String rule, String expr, int start, int end, List<Trace> children) {}
    public record Match(List<Recipe> nodes, List<Occurrence> local, List<Occurrence> captures,
                        List<Trace> traces, int start, int end, boolean mappingFailure) {
        public Match(List<Recipe> nodes, List<Occurrence> local, List<Occurrence> captures,
                     List<Trace> traces, int start, int end) { this(nodes, local, captures, traces, start, end, false); }
        public Match { nodes = List.copyOf(nodes); local = List.copyOf(local); captures = List.copyOf(captures); traces = List.copyOf(traces); }
        public static Match empty(int start, int end) { return new Match(List.of(), List.of(), List.of(), List.of(), start, end); }
    }
    public record Delimiter(String kind, String open, String close, int[] characters) {}
    public record Boundary(String at, int child, Delimiter[] delimiters) {}
    public record Field(String name, String[] sites, String conversion, String cardinality) {}
    public record Effect(String when, String action, String site, String name, String mode) {}
    private record Mark(int scope, int effects) {}
    private record Memo(Match match, boolean reset, List<TokenScanner.Effect> effects, Diagnostics.Snapshot diagnostics) {}
    public final String source;
    public final Input input;
    public final ParseOptions options;
    public final ScopeStore scope = new ScopeStore();
    private final Diagnostics diagnostics = new Diagnostics();
    private final MemoTable memo = new MemoTable();
    private final ArrayList<Memo> payloads = new ArrayList<>();
    private final ArrayList<TokenScanner.Effect> effects = new ArrayList<>();
    private final ArrayList<String> expected = new ArrayList<>();
    private final Map<String, Integer> expectedIds = new LinkedHashMap<>();
    private final String[] ruleNames;
    private int suppression;
    private int reached;
    private ParseResult<?> syntax;
    public final IdentityHashMap<Object, Span> spans = new IdentityHashMap<>();
    private final IdentityHashMap<Recipe, Object> built = new IdentityHashMap<>();
    public Session(CharSequence source, ParseOptions options, String[] ruleNames) {
        this.source = source.toString(); this.input = new Input(this.source); this.options = options; this.ruleNames = ruleNames.clone();
    }
    public int cp(int at) { return at < input.length() ? input.codePointAt(at) : -1; }
    public Span span(int start, int end) { return new Span(input.cpIndex(start), input.cpIndex(end)); }
    private void fail(int at, String label) {
        int id = expectedIds.computeIfAbsent(label, key -> { expected.add(key); return expected.size() - 1; });
        diagnostics.failAt(at, id); reached = Math.max(reached, at);
    }
    private Mark mark() { return new Mark(scope.checkpoint(), effects.size()); }
    private void restore(Mark mark) { scope.restore(mark.scope()); effects.subList(mark.effects(), effects.size()).clear(); }
    private void replay(List<TokenScanner.Effect> replay) { for (var effect : replay) effect(effect); }
    private void effect(TokenScanner.Effect effect) {
        effects.add(effect);
        switch (effect.action()) {
            case "enter" -> scope.enter(); case "leave" -> scope.leave(); case "clear" -> scope.clearDiagnostics();
            case "declare" -> scope.declare(effect.name(), effect.offsetCp());
            case "use" -> {
                scope.addReference(effect.name(), effect.offsetCp(), effect.lengthCp());
                if (!scope.isDeclared(effect.name())) scope.addDiagnostic("Undefined reference: " + effect.name(), effect.offsetCp(), effect.lengthCp(), Severity.WARNING);
            }
            case "error" -> scope.addDiagnostic(effect.name(), effect.offsetCp(), effect.lengthCp(), Severity.ERROR);
            default -> throw new IllegalArgumentException("Unsupported scanner effect: " + effect.action());
        }
    }
    public Match rule(Frame f, int id, boolean successMemo, boolean failureMemo, Call body,
                      String recipe, String fallback, String recipeKind, Field[] fields, Effect[] scopeEffects, boolean skip) {
        int c = f.c, m = f.m, state = scope.stateVersion();
        int mode = (f.matched ? 1 : 0) | (f.invert ? 2 : 0) | (f.reset ? 4 : 0);
        boolean cache = options.memo() && suppression == 0 && (successMemo || failureMemo);
        if (cache) {
            int slot = memo.get(id, c, m, mode, state);
            if (slot >= 0) {
                Memo value = payloads.get(memo.payload(slot)); diagnostics.merge(value.diagnostics()); replay(value.effects());
                f.c = memo.consumedEnd(slot); f.m = memo.matchedEnd(slot); f.reset = value.reset();
                return value.match();
            }
        }
        var mark = mark(); diagnostics.enterRule(id);
        applyEffects(scopeEffects, "entry", null);
        Match result = body.parse(this, f);
        if (result != null) {
            applyEffects(scopeEffects, "successBeforeLeave", result);
            applyEffects(scopeEffects, "successAfterLeave", result);
            applyEffects(scopeEffects, "afterDeclarations", result);
            List<Recipe> nodes = result.nodes();
            if (skip || "skip".equals(recipeKind)) nodes = List.of();
            else if (recipe != null && !"transparent".equals(recipeKind) && !"text".equals(recipeKind)) {
                var values = new LinkedHashMap<String, List<Value>>();
                for (var field : fields) {
                    var captures = new ArrayList<Value>();
                    for (var occurrence : result.local()) if (Arrays.asList(field.sites()).contains(occurrence.site())) {
                        Value v = occurrence.value();
                        if (field.conversion().equals("node") && field.cardinality().equals("list") && !v.nodes().isEmpty())
                            for (Recipe node : v.nodes()) captures.add(new Value(node.start(), node.end(), List.of(node)));
                        else if (!(field.conversion().equals("node") && field.cardinality().equals("list") && v.start() == v.end() && v.nodes().isEmpty())) captures.add(v);
                    }
                    values.put(field.name(), List.copyOf(captures));
                }
                nodes = List.of(new Recipe(recipe, c, f.c, values, fallback));
            }
            result = new Match(nodes, List.of(), result.captures(), result.traces(), c, f.c, result.mappingFailure());
        } else { restore(mark); f.c = c; f.m = m; }
        var diagnostic = diagnostics.leaveRule();
        if (cache && (result != null ? successMemo : failureMemo)) {
            int payload = payloads.size();
            payloads.add(new Memo(result, f.reset, List.copyOf(effects.subList(mark.effects(), effects.size())), diagnostic));
            memo.put(id, c, m, mode, state, result != null, f.c, f.m, payload);
        }
        return result;
    }
    private void applyEffects(Effect[] definitions, String when, Match result) {
        var declared = new HashMap<String, String>();
        for (Effect definition : definitions) {
            if (!definition.when().equals(when)) continue;
            if (definition.site() == null) {
                effect(new TokenScanner.Effect(definition.action(), definition.name(), 0, 0)); continue;
            }
            if (result == null) continue;
            for (Occurrence capture : result.local()) if (capture.site().equals(definition.site())) {
                Value v = capture.value();
                int trimmedStart = v.start(), trimmedEnd = v.end();
                while (trimmedStart < trimmedEnd && source.charAt(trimmedStart) <= 0x20) trimmedStart++;
                while (trimmedEnd > trimmedStart && source.charAt(trimmedEnd - 1) <= 0x20) trimmedEnd--;
                String name = source.substring(trimmedStart, trimmedEnd);
                if (name.isEmpty()) continue;
                if (definition.mode().equals("sameRuleText")) {
                    String key = definition.name() == null ? capture.name() : definition.name();
                    // Declarations can belong to an earlier phase; inspect the selected local captures.
                    String target = declared.get(key);
                    for (Effect declaration : definitions) if (declaration.action().equals("declare") && java.util.Objects.equals(key, declaration.name()))
                        for (Occurrence own : result.local()) if (own.site().equals(declaration.site())) target = source.substring(own.value().start(), own.value().end()).trim();
                    if (definition.action().equals("declare")) declared.put(key, name);
                    else if (definition.action().equals("use") && !name.equals(target)) effect(new TokenScanner.Effect("error", "Reference mismatch: " + name, input.cpIndex(trimmedStart), input.cpLength(trimmedStart, trimmedEnd)));
                } else effect(new TokenScanner.Effect(definition.action(), name, input.cpIndex(trimmedStart), input.cpLength(trimmedStart, trimmedEnd)));
            }
        }
    }
    public Match capture(Frame f, Call call, String[] sites, String[] names, String[] spanRules, boolean[] skips,
                         String rule, String expr) {
        int c = f.c, m = f.m; Match result = call.parse(this, f);
        reached = Math.max(reached, Math.max(f.c, f.m));
        if (result == null) return null;
        var locals = new ArrayList<>(result.local()); var captures = new ArrayList<>(result.captures());
        for (int i = 0; i < sites.length; i++) if (!skips[i]) {
            int start = spanRules[i].equals("matchedExtent") ? m : spanRules[i].equals("selectedChildExtent") ? result.start() : c;
            int end = spanRules[i].equals("matchedExtent") ? f.m : spanRules[i].equals("selectedChildExtent") ? result.end() : f.c;
            var occurrence = new Occurrence(sites[i], names[i], new Value(start, Math.max(start, end), result.nodes()));
            locals.add(occurrence); captures.add(occurrence);
        }
        List<Trace> traces = options.lexical() ? List.of(new Trace(rule, expr, c, f.c, result.traces())) : List.of();
        return new Match(result.nodes(), locals, captures, traces, result.start(), result.end(), result.mappingFailure());
    }
    private static Match combine(List<Match> items, int start, int end) {
        var nodes = new ArrayList<Recipe>(); var local = new ArrayList<Occurrence>();
        var captures = new ArrayList<Occurrence>(); var traces = new ArrayList<Trace>();
        for (Match item : items) { nodes.addAll(item.nodes()); local.addAll(item.local()); captures.addAll(item.captures()); traces.addAll(item.traces()); }
        return new Match(nodes, local, captures, traces, start, end, items.stream().anyMatch(Match::mappingFailure));
    }
    public Match sequence(Frame parent, Call[] children, Boundary[] boundaries) {
        var mark = mark(); Frame f = parent.copy(true); int start = f.c;
        boundary(f, boundaries, "entry", -1); boundary(f, boundaries, "helperEntry", -1);
        var matches = new ArrayList<Match>();
        for (int i = 0; i < children.length; i++) {
            Match result = children[i].parse(this, f);
            if (result == null) { restore(mark); return null; }
            matches.add(result); boundary(f, boundaries, "afterChild", i);
        }
        boundary(f, boundaries, "helperExit", -1); boundary(f, boundaries, "exit", -1);
        parent.commit(f); return combine(matches, start, f.c);
    }
    public Match choice(Frame parent, Call[] alternatives, boolean longest, boolean[] eligible, boolean fallback) {
        Match best = null; Frame bestFrame = null; List<TokenScanner.Effect> bestEffects = List.of();
        for (int pass = 0; pass < (fallback ? 2 : 1); pass++) {
            for (int i = 0; i < alternatives.length; i++) {
                if (pass == 0 && eligible != null && !eligible[i]) continue;
                var mark = mark(); Frame trial = parent.copy(true);
                Match result = alternatives[i].parse(this, trial);
                if (result != null && !longest) { parent.commit(trial); return result; }
                if (result != null && (best == null || trial.c > bestFrame.c)) {
                    best = result; bestFrame = trial; bestEffects = List.copyOf(effects.subList(mark.effects(), effects.size()));
                }
                restore(mark);
            }
            if (best != null) break;
        }
        if (best != null) { parent.commit(bestFrame); replay(bestEffects); }
        return best;
    }
    public Match optional(Frame parent, Call child) {
        var mark = mark(); Frame trial = parent.copy(true); Match result = child.parse(this, trial);
        if (result == null) { restore(mark); return Match.empty(parent.c, parent.c); }
        parent.commit(trial); return result;
    }
    public Match repeat(Frame parent, Call child, int min, int max, Call terminator, Boundary[] boundaries) {
        var whole = mark(); Frame f = parent.copy(true); int start = f.c, count = 0;
        boundary(f, boundaries, "entry", -1); boundary(f, boundaries, "helperEntry", -1);
        var matches = new ArrayList<Match>();
        while (true) {
            int before = f.position();
            if (terminator != null) {
                var mark = mark(); Frame end = f.copy(true); end.matched = false;
                Match done = terminator.parse(this, end);
                if (done != null) {
                    if (!f.matched) { f.commit(end); matches.add(done); }
                    else restore(mark);
                    break;
                }
                restore(mark);
            }
            var mark = mark(); Frame trial = f.copy(true); Match result = child.parse(this, trial);
            if (result == null) { restore(mark); break; }
            f.commit(trial); matches.add(result); count++;
            if (before == f.position() || count >= max) break;
        }
        if (count < min || count > max) { restore(whole); return null; }
        boundary(f, boundaries, "helperExit", -1); boundary(f, boundaries, "exit", -1);
        parent.commit(f); return combine(matches, start, f.c);
    }
    public Match separated(Frame parent, Call element, Call separator, int min, Boundary[] boundaries) {
        var whole = mark(); Frame f = parent.copy(true); int start = f.c;
        boundary(f, boundaries, "entry", -1); boundary(f, boundaries, "helperEntry", -1);
        var matches = new ArrayList<Match>(); var first = mark();
        Match initial = element.parse(this, f);
        if (initial == null) { restore(first); if (min > 0) { restore(whole); return null; } f = parent.copy(true); }
        else {
            matches.add(initial);
            while (true) {
                var mark = mark(); Frame trial = f.copy(true); int before = trial.position();
                Match sep = separator.parse(this, trial); Match next = sep == null ? null : element.parse(this, trial);
                if (next == null) { restore(mark); break; }
                f.commit(trial); matches.add(sep); matches.add(next);
                if (before == f.position()) break;
            }
        }
        if (initial != null && (matches.size() + 1) / 2 < min) { restore(whole); return null; }
        boundary(f, boundaries, "helperExit", -1); boundary(f, boundaries, "exit", -1);
        parent.commit(f); return combine(matches, start, f.c);
    }
    public Match look(Frame parent, Call child, boolean negative, String label) {
        var mark = mark(); Frame trial = parent.copy(true); trial.matched = true;
        trial.invert = negative;
        if (negative) { suppression++; diagnostics.suppressBegin(); }
        Match result;
        try { result = child.parse(this, trial); }
        finally { if (negative) { diagnostics.suppressEnd(); suppression--; } restore(mark); }
        if (negative) {
            if (result != null) { fail(Math.max(parent.c, parent.m), label); return null; }
            parent.m = parent.reset ? parent.c : parent.m; return Match.empty(parent.c, parent.c);
        }
        if (result == null) return null;
        parent.commit(trial); return Match.empty(parent.c, parent.c);
    }
    public Match literal(Frame f, String word, boolean sensitive, String label) {
        int at = f.position();
        boolean ok = sensitive ? input.startsWith(at, word) : input.startsWithIgnoreCase(at, word);
        if (!ok) { fail(Math.max(f.c, f.m), label); f.advance(0); return null; }
        f.advance(word.length()); return Match.empty(at, at + word.length());
    }
    public Match any(Frame f, String label) {
        int at = f.position(); if (cp(at) < 0) { fail(Math.max(f.c, f.m), label); f.advance(0); return null; }
        int length = Character.charCount(cp(at)); f.advance(length); return Match.empty(at, at + length);
    }
    public Match empty(Frame f) { return optional(f, (s, p) -> s.look(p, (ss, pp) -> ss.any(pp, "ANY"), false, "EMPTY")); }
    public Match eof(Frame f) { return repeat(f, (s, p) -> s.any(p, "EOF"), 0, 0, null, new Boundary[0]); }
    public Match error(Frame f, String message) { fail(Math.max(f.c, f.m), message); return null; }
    private boolean digit(int p) { return cp(p) >= '0' && cp(p) <= '9'; }
    public Match number(Frame f) {
        int at = f.position(), p = at;
        if (cp(p) == '+' || cp(p) == '-') p++;
        int digits = p; while (digit(p)) p++;
        boolean hasDigits = p > digits;
        if (cp(p) == '.') {
            p++; int fraction = p; while (digit(p)) p++;
            if (!hasDigits && p == fraction) { fail(p, "NUMBER"); f.advance(0); return null; }
        } else if (!hasDigits) { fail(p, "NUMBER"); f.advance(0); return null; }
        int mantissa = p;
        if (cp(p) == 'e' || cp(p) == 'E') {
            p++; if (cp(p) == '+' || cp(p) == '-') p++;
            int exponent = p; while (digit(p)) p++;
            if (p == exponent) { fail(p, "NUMBER"); p = mantissa; }
        }
        f.advance(p - at); return Match.empty(at, p);
    }
    public Match builtin(Frame f, String kind, int[] characters) {
        if (kind.equals("Number")) return number(f);
        if (kind.equals("EndOfSource")) return eof(f);
        int start = f.position(), p = start;
        if (kind.equals("Quoted") || kind.equals("SingleQuoted")) {
            int quote = kind.equals("Quoted") ? '"' : '\'';
            if (cp(p) != quote) return error(f, kind);
            p++;
            while (cp(p) >= 0 && cp(p) != quote) {
                if (cp(p) == '\\') { p++; if (cp(p) < 0) { fail(p, kind); return null; } }
                p += Character.charCount(cp(p));
            }
            if (cp(p) < 0) { fail(p, kind); return null; } p++;
        } else if (kind.equals("Identifier")) {
            if (!alphabet(cp(p)) && cp(p) != '_') return error(f, "IDENTIFIER");
            p++; while (alphabet(cp(p)) || digit(p) || cp(p) == '_') p++;
        } else {
            int ch = cp(p);
            boolean ok = ch >= 0 && switch (kind) {
                case "Digit" -> digit(p); case "Alphabet" -> alphabet(ch);
                case "AlphabetUnderScore" -> alphabet(ch) || ch == '_';
                case "AlphabetNumericUnderScore" -> alphabet(ch) || digit(p) || ch == '_';
                case "Space" -> ch == 32 || ch >= 9 && ch <= 13;
                case "CodePointSet" -> Arrays.binarySearch(characters, ch) >= 0;
                default -> throw new IllegalArgumentException(kind);
            };
            if (!ok) return error(f, kind); p += Character.charCount(ch);
        }
        f.advance(p - start); return Match.empty(start, p);
    }
    private static boolean alphabet(int ch) { return ch >= 'A' && ch <= 'Z' || ch >= 'a' && ch <= 'z'; }
    public Match range(Frame f, int min, int max, String excluded, String label) {
        int at = f.position(), ch = cp(at);
        if (ch < 0 || ch < min || ch > max || excluded != null && excluded.codePoints().anyMatch(v -> v == ch)) return error(f, label);
        f.advance(Character.charCount(ch)); return Match.empty(at, f.position());
    }
    public Match regex(Frame f, Pattern pattern) {
        int at = f.position(); var matcher = pattern.matcher(source).region(at, source.length());
        if (!matcher.lookingAt() || matcher.end() == at) return error(f, pattern.pattern());
        input.cpIndex(matcher.end()); f.advance(matcher.end() - at); return Match.empty(at, matcher.end());
    }
    public Match until(Frame f, String terminator) {
        return repeat(f, (s, p) -> s.any(p, "ANY"), 0, Integer.MAX_VALUE,
            terminator.isEmpty() ? null : (s, p) -> s.look(p, (ss, pp) -> ss.literal(pp, terminator, true, terminator), false, terminator), new Boundary[0]);
    }
    public Match external(Frame f, String id) {
        TokenScanner scanner = options.scanners().get(id);
        if (scanner == null) throw new IllegalArgumentException("Missing TokenScanner: " + id);
        var result = scanner.scan(input, f.c, f.m);
        for (var diagnostic : result.diagnostics()) { input.cpIndex(diagnostic.offsetUtf16()); fail(diagnostic.offsetUtf16(), diagnostic.expected()); }
        if (!result.ok()) return null;
        input.cpIndex(result.consumedEnd()); input.cpIndex(result.matchedEnd()); input.cpIndex(result.valueStart()); input.cpIndex(result.valueEnd());
        if (result.consumedEnd() < f.c || result.matchedEnd() < result.consumedEnd() || result.valueEnd() < result.valueStart()) throw new IllegalArgumentException("Invalid TokenScanner cursors: " + id);
        if (f.matched) f.m = result.matchedEnd(); else { f.c = result.consumedEnd(); f.m = result.matchedEnd(); }
        replay(result.effects()); return Match.empty(result.valueStart(), result.valueEnd());
    }
    public void boundary(Frame f, Boundary[] boundaries, String at, int child) {
        for (var boundary : boundaries) if (boundary.at().equals(at) && boundary.child() == child) trivia(f, boundary.delimiters());
    }
    private void trivia(Frame f, Delimiter[] delimiters) {
        int start = f.position(), p = start;
        boolean again = true;
        while (again) {
            again = false;
            for (var delimiter : delimiters) {
                int next = p;
                if (delimiter.kind().equals("characters")) {
                    if (cp(p) >= 0 && Arrays.binarySearch(delimiter.characters(), cp(p)) >= 0) next += Character.charCount(cp(p));
                } else if (input.startsWith(p, delimiter.open())) {
                    if (delimiter.kind().equals("lineComment")) {
                        next += delimiter.open().length();
                        while (cp(next) >= 0 && cp(next) != '\r' && cp(next) != '\n') next += Character.charCount(cp(next));
                    } else {
                        int end = source.indexOf(delimiter.close(), p + delimiter.open().length());
                        if (end < 0) fail(input.length(), "'*/'");
                        else next = end + delimiter.close().length();
                    }
                }
                if (next > p) { p = next; again = true; break; }
            }
        }
        f.advance(p - start);
    }
    public Match recover(Frame f, Call child, String[] patterns, boolean consumeSync, String noSync, String projection, String recipe) {
        var mark = mark(); Frame trial = f.copy(false); Match result = child.parse(this, trial);
        if (result != null) { f.commit(trial); return result; }
        restore(mark); int start = f.c, end = -1, length = 0;
        for (String pattern : patterns) {
            int found = source.indexOf(pattern, start);
            if (found >= 0 && (end < 0 || found < end)) { end = found; length = pattern.length(); }
        }
        if (end < 0) {
            if (noSync.equals("fail")) return null;
            end = noSync.equals("consumeToEof") ? input.length() : start;
        } else if (consumeSync) end += length;
        f.advance(end - start);
        effect(new TokenScanner.Effect("error", "Recovered input", input.cpIndex(start), input.cpLength(start, end)));
        String id = projection.equals("explicitRecipe") ? recipe : "#recovery";
        return new Match(List.of(new Recipe(id, start, end, Map.of(), null)), List.of(), List.of(), List.of(), start, end, projection.equals("mappingFailure"));
    }
    public <N> ParseResult<N> result(Frame f, Match match, Map<String, String> catalogs) {
        boolean ok = match != null; String kind = ok ? "none" : "syntax"; int offset = -1;
        if (ok && options.requireFullInput() && f.c != input.length()) { ok = false; kind = "trailing_input"; offset = f.c; fail(f.c, "EOF"); }
        if (!ok && offset < 0) offset = diagnostics.farthest() < 0 ? reached : diagnostics.farthest();
        var captures = new ArrayList<Capture>(); var lexical = new ArrayList<ParseResult.Lexical>();
        var catalog = new LinkedHashMap<String, List<Capture>>();
        if (ok) {
            long occurrence = 0;
            for (var capture : match.captures()) {
                var value = new Capture(occurrence, capture.site(), capture.name(), span(capture.value().start(), capture.value().end()), occurrence++);
                captures.add(value);
                String context = catalogs.get(capture.site()); if (context != null) catalog.computeIfAbsent(context, key -> new ArrayList<>()).add(value);
            }
            long[] counter = {0, 0}; for (var trace : match.traces()) trace(trace, -1, counter, lexical);
        }
        catalog.replaceAll((key, values) -> List.copyOf(values));
        var diagnostic = new Diagnostic(kind, offset < 0 ? -1 : input.cpIndex(offset), diagnostics.farthest() < 0 ? -1 : input.cpIndex(diagnostics.farthest()),
            Arrays.stream(diagnostics.expected()).mapToObj(expected::get).sorted().toList(),
            diagnostics.deepestRule() < 0 ? null : ruleNames[diagnostics.deepestRule()],
            Arrays.stream(diagnostics.ruleStack()).mapToObj(i -> ruleNames[i]).toList());
        var result = new ParseResult<N>(ok, input.cpIndex(f.c), input.cpIndex(f.m), Optional.empty(), Map.of(), captures, lexical,
            new ParseResult.Scope(scope.allDeclarations(), scope.references(), scope.diagnostics()), diagnostic, catalog);
        syntax = result; return result;
    }
    private void trace(Trace trace, long parent, long[] counter, List<ParseResult.Lexical> out) {
        long id = counter[0]++;
        for (var child : trace.children()) trace(child, id, counter, out);
        out.add(new ParseResult.Lexical(id, parent, trace.rule(), trace.expr(), span(trace.start(), trace.end()), counter[1]++));
    }
    public String text(Value value) { return source.substring(value.start(), value.end()).strip(); }
    public int number(Value value) {
        try { return Integer.parseInt(text(value)); }
        catch (NumberFormatException error) { throw mapping(value.start(), value.end(), "NUMBER cannot be mapped to int", error); }
    }
    public Recipe node(Value value, Recipe owner) {
        if (!value.nodes().isEmpty()) return value.nodes().getFirst();
        if (owner.fallback() != null) return new Recipe("#leaf:" + owner.fallback(), value.start(), value.end(), Map.of(), null);
        throw mapping(value.start(), value.end(), "Missing mapped node", null);
    }
    public <T> T scalar(Recipe recipe, String field, Function<Value, T> conversion) {
        List<Value> values = recipe.fields().getOrDefault(field, List.of());
        if (values.isEmpty()) throw mapping(recipe.start(), recipe.end(), "Missing scalar capture: " + field, null);
        return conversion.apply(values.getFirst());
    }
    public <T> Optional<T> optional(Recipe recipe, String field, Function<Value, T> conversion) {
        List<Value> values = recipe.fields().getOrDefault(field, List.of());
        return values.isEmpty() ? Optional.empty() : Optional.ofNullable(conversion.apply(values.getFirst()));
    }
    public <T> List<T> list(Recipe recipe, String field, Function<Value, T> conversion) {
        return recipe.fields().getOrDefault(field, List.of()).stream().map(conversion).toList();
    }
    public MappingException mapping(int start, int end, String message, Throwable cause) { return new MappingException(syntax, span(start, end), message, cause); }
    public Object built(Recipe recipe) { return built.get(recipe); }
    public Object remember(Recipe recipe, Object object) { built.put(recipe, object); spans.put(object, span(recipe.start(), recipe.end())); return object; }
}
