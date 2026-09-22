package generated.internal;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.IdentityHashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
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
    public record Trace(String rule, String expr, int start, int end, List<Trace> children, String token) {}
    public record RecoveryEvent(String rule, String mode, int start, int end, String pattern, int syncStart,
                                int syncEnd, Diagnostic diagnostic, Recipe marker) {}
    /** values == null denotes plain text; an empty values list denotes an absent semantic value. */
    public record Match(List<Recipe> nodes, List<Occurrence> local, List<Occurrence> captures,
                        List<Trace> traces, int start, int end, boolean mappingFailure, List<RecoveryEvent> recoveries, List<Match> items, String text, List<Value> values) {
        public Match(List<Recipe> nodes, List<Occurrence> local, List<Occurrence> captures,
                     List<Trace> traces, int start, int end, boolean mappingFailure, List<RecoveryEvent> recoveries, List<Match> items, String text) {
            this(nodes, local, captures, traces, start, end, mappingFailure, recoveries, items, text, null);
        }
        public Match(List<Recipe> nodes, List<Occurrence> local, List<Occurrence> captures,
                     List<Trace> traces, int start, int end, boolean mappingFailure, List<RecoveryEvent> recoveries) {
            this(nodes, local, captures, traces, start, end, mappingFailure, recoveries, List.of(), null);
        }
        public Match(List<Recipe> nodes, List<Occurrence> local, List<Occurrence> captures,
                     List<Trace> traces, int start, int end, boolean mappingFailure) {
            this(nodes, local, captures, traces, start, end, mappingFailure, List.of());
        }
        public Match(List<Recipe> nodes, List<Occurrence> local, List<Occurrence> captures,
                     List<Trace> traces, int start, int end) { this(nodes, local, captures, traces, start, end, false); }
        public Match { nodes = List.copyOf(nodes); local = List.copyOf(local); captures = List.copyOf(captures); traces = List.copyOf(traces); recoveries = List.copyOf(recoveries); items = List.copyOf(items); values = values == null ? null : List.copyOf(values); }
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
    private Diagnostics diagnostics = new Diagnostics();
    private final MemoTable memo = new MemoTable();
    private final ArrayList<Memo> payloads = new ArrayList<>();
    private final ArrayList<TokenScanner.Effect> effects = new ArrayList<>();
    private final ArrayList<String> expected = new ArrayList<>();
    private final Map<String, Integer> expectedIds = new LinkedHashMap<>();
    private final String[] ruleNames;
    private int depth;
    private final ArrayList<Integer> activeRules = new ArrayList<>();
    private int reached;
    private ParseResult<?> syntax;
    private final IdentityHashMap<Occurrence, Long> completedCaptures = new IdentityHashMap<>();
    private long nextCaptureCompletion;
    public final IdentityHashMap<Object, Span> spans = new IdentityHashMap<>();
    private final IdentityHashMap<Recipe, Object> built = new IdentityHashMap<>();
    public Session(CharSequence source, ParseOptions options, String[] ruleNames) {
        this.source = source.toString(); this.input = new Input(this.source); this.options = options; this.ruleNames = ruleNames.clone();
    }
    private static final class DepthLimit extends RuntimeException {
        private static final long serialVersionUID = 1L;
        private DepthLimit() { super(null, null, false, false); }
    }
    /** Abort the entire parse: an outer optional/choice must not swallow a resource limit. */
    public Match parse(Call entry, Frame f) {
        var start = mark(); Frame initial = f.copy(false);
        try { return entry.parse(this, f); }
        catch (DepthLimit | StackOverflowError limit) {
            restore(start); f.commit(initial); f.reset = initial.reset;
            diagnostics = new Diagnostics();
            String reason = limit instanceof StackOverflowError ? "JVM stack limit; maxDepth=" : "maxDepth=";
            fail(reached, "maximum parse depth exceeded (" + reason + options.maxDepth() + ")");
            return null;
        }
    }
    public int cp(int at) { return at < input.length() ? input.codePointAt(at) : -1; }
    public Span span(int start, int end) { return new Span(input.cpIndex(start), input.cpIndex(end)); }
    private void progress(Frame f) {
        int at = Math.max(f.c, f.m); reached = Math.max(reached, at); diagnostics.reach(at);
    }
    private void fail(int at, String label) { fail(at, label, at, at); }
    private void fail(Frame frame, String label) { fail(Math.max(frame.c, frame.m), label, frame.c, frame.m); }
    private void fail(int at, String label, int consumed, int matched) {
        int id = expectedIds.computeIfAbsent(label, key -> { expected.add(key); return expected.size() - 1; });
        diagnostics.failAt(at, id, consumed, matched); reached = Math.max(reached, at);
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
                if (!scope.isDeclared(effect.name())) scope.addDiagnostic("未定義のシンボル: '" + effect.name() + "'", effect.offsetCp(), effect.lengthCp(), Severity.WARNING);
            }
            case "error" -> scope.addDiagnostic(effect.name(), effect.offsetCp(), effect.lengthCp(), Severity.ERROR);
            default -> throw new IllegalArgumentException("Unsupported scanner effect: " + effect.action());
        }
    }
    public Match rule(Frame f, int id, int memoId, boolean successMemo, boolean failureMemo, Call body,
                      String recipe, String fallback, String recipeKind, Field[] fields, Effect[] scopeEffects, boolean skip, String[] recoveryHints, boolean rightAssociative, String[] failureHints) {
        int c = f.c, m = f.m, state = scope.stateVersion();
        int mode = (f.matched ? 1 : 0) | (f.invert ? 2 : 0) | (f.reset ? 4 : 0);
        boolean cache = options.memo() && (successMemo || failureMemo);
        if (cache) {
            int slot = memo.get(memoId, c, m, mode, state);
            if (slot >= 0) {
                Memo value = payloads.get(memo.payload(slot)); diagnostics.merge(value.diagnostics()); replay(value.effects());
                f.c = memo.consumedEnd(slot); f.m = memo.matchedEnd(slot); f.reset = value.reset();
                return value.match();
            }
        }
        progress(f);
        if (depth >= options.maxDepth()) throw new DepthLimit();
        depth++;
        boolean recursive = activeRules.contains(id); activeRules.add(id);
        var mark = mark(); diagnostics.enterRule(id); progress(f);
        applyEffects(scopeEffects, "entry", Match.empty(c, c));
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
                        if (Set.of("node", "mixed").contains(field.conversion()) && !field.cardinality().equals("scalar") && v.elements() != null)
                            captures.addAll(v.elements());
                        else if (Set.of("node", "mixed").contains(field.conversion()) && field.cardinality().equals("list") && !v.nodes().isEmpty())
                            for (Recipe node : v.nodes()) captures.add(new Value(node.start(), node.end(), List.of(node)));
                        else if (!(Set.of("node", "mixed").contains(field.conversion()) && field.cardinality().equals("list") && v.start() == v.end() && v.nodes().isEmpty())) captures.add(v);
                    }
                    values.put(field.name(), List.copyOf(captures));
                }
                int nodeEnd = rightAssociative && recursive ? values.values().stream().flatMap(List::stream).mapToInt(Value::end).max().orElse(f.c) : f.c;
                nodes = List.of(new Recipe(recipe, c, nodeEnd, values, fallback));
            }
            for (var occurrence : result.local()) completedCaptures.put(occurrence, nextCaptureCompletion++);
            result = new Match(nodes, List.of(), result.captures(), result.traces(), c, f.c,
                result.mappingFailure(), result.recoveries(), List.of(), result.text(),
                (skip || "skip".equals(recipeKind) || nodes != result.nodes()) ? nodes.stream().map(n -> new Value(n.start(), n.end(), List.of(n))).toList() : result.values());
        } else {
            var local = diagnostics.snapshot();
            if (local.farthest() == Math.max(c, m)) for (String hint : recoveryHints) fail(local.farthest(), "'" + hint + "'");
            for (String hint : failureHints) fail(Math.max(c, m), hint);
            restore(mark); f.c = c; f.m = m;
        }
        depth--; activeRules.removeLast();
        var diagnostic = diagnostics.leaveRule();
        if (cache && (result != null ? successMemo : failureMemo)) {
            int payload = payloads.size();
            payloads.add(new Memo(result, f.reset, List.copyOf(effects.subList(mark.effects(), effects.size())), diagnostic));
            memo.put(memoId, c, m, mode, state, result != null, f.c, f.m, payload);
        }
        return result;
    }
    public Match expression(Frame f, int id, boolean successMemo, boolean failureMemo, Call body) {
        if (!options.memo() || !successMemo && !failureMemo) return body.parse(this, f);
        int c = f.c, m = f.m, state = scope.stateVersion();
        int mode = (f.matched ? 1 : 0) | (f.invert ? 2 : 0) | (f.reset ? 4 : 0);
        int slot = memo.get(id, c, m, mode, state);
        if (slot >= 0) {
            Memo value = payloads.get(memo.payload(slot)); diagnostics.merge(value.diagnostics()); replay(value.effects());
            f.c = memo.consumedEnd(slot); f.m = memo.matchedEnd(slot); f.reset = value.reset();
            return value.match();
        }
        var mark = mark(); diagnostics.enterExpression(); progress(f);
        Match result = body.parse(this, f);
        var diagnostic = diagnostics.leaveRule();
        if (result != null ? successMemo : failureMemo) {
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
                effect(new TokenScanner.Effect(definition.action(), definition.name(), result == null ? 0 : input.cpIndex(result.end()), 0, definition.mode())); continue;
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
                    for (Occurrence own : result.local()) if (own.name().equals(key)) {
                        target = source.substring(own.value().start(), own.value().end()).trim(); break;
                    }
                    if (definition.action().equals("declare")) declared.put(key, name);
                    else if (definition.action().equals("use") && target != null && !name.equals(target))
                        effect(new TokenScanner.Effect("error", "back-reference mismatch: expected '" + target + "' but got '" + name + "'",
                            input.cpIndex(trimmedStart), input.cpLength(trimmedStart, trimmedEnd)));
                } else effect(new TokenScanner.Effect(definition.action(), name, input.cpIndex(trimmedStart), input.cpLength(trimmedStart, trimmedEnd), definition.mode()));
            }
        }
    }
    public Match capture(Frame f, Call call, String[] sites, String[] names, String[] spanRules, boolean[] skips,
                         String rule, String expr, boolean quantified, boolean terminal, String semanticKind, boolean semanticMany, String[] failureHints) {
        int c = f.c, m = f.m; Match result = call.parse(this, f);
        progress(f);
        if (result == null) {
            for (String hint : failureHints) fail(Math.max(c, m), hint);
            return null;
        }
        result = project(result, semanticKind, semanticMany);
        if (quantified && sites.length > 0) {
            var items = new ArrayList<Match>();
            for (Match item : result.items()) {
                Match selected = item;
                if (selected.items().size() == 1) selected = selected.items().getFirst();
                String text = selected.text() == null ? source.substring(selected.start(), selected.end()).strip() : selected.text();
                var value = new Match(item.nodes(), item.local(), item.captures(), item.traces(), item.start(), item.end(),
                    item.mappingFailure(), item.recoveries(), item.items(), text, item.values());
                items.add(captured(value, item.start(), item.end(), sites, names, skips));
            }
            Match joined = combine(items, result.start(), result.end());
            return new Match(result.nodes(), joined.local(), joined.captures(), result.traces(), result.start(), result.end(),
                result.mappingFailure(), result.recoveries(), items, result.text(), result.values());
        }
        Match captured = result;
        for (int i = 0; i < sites.length; i++) {
            int start = spanRules[i].equals("matchedExtent") ? m : spanRules[i].equals("selectedChildExtent") ? result.start() : c;
            int end = spanRules[i].equals("matchedExtent") ? f.m : spanRules[i].equals("selectedChildExtent") ? result.end() : f.c;
            captured = captured(captured, start, Math.max(start, end), new String[]{sites[i]}, new String[]{names[i]}, new boolean[]{skips[i]});
        }
        List<Trace> traces = options.lexical() ? List.of(new Trace(rule, expr, c, f.c, result.traces(), terminal ? source.substring(result.start(), result.end()) : null)) : List.of();
        return new Match(captured.nodes(), captured.local(), captured.captures(), traces, result.start(), result.end(),
            result.mappingFailure(), result.recoveries(), result.items(), result.text(), result.values());
    }
    private Match captured(Match result, int start, int end, String[] sites, String[] names, boolean[] skips) {
        var locals = new ArrayList<>(result.local()); var captures = new ArrayList<>(result.captures());
        for (int i = 0; i < sites.length; i++) if (!skips[i]) {
            var occurrence = new Occurrence(sites[i], names[i], new Value(start, end, result.nodes(), result.text(), result.values()));
            locals.add(occurrence); captures.add(occurrence);
        }
        return new Match(result.nodes(), locals, captures, result.traces(), result.start(), result.end(),
            result.mappingFailure(), result.recoveries(), result.items(), result.text(), result.values());
    }
    private Match project(Match result, String kind, boolean many) {
        if (kind.equals("text")) return result;
        List<Value> values = result.values();
        if (values == null) values = List.of(new Value(result.start(), result.end(), result.nodes(), result.text()));
        if (kind.equals("mixed") && !many && !values.isEmpty() && values.stream().allMatch(v -> v.nodes().isEmpty()))
            values = List.of(new Value(result.start(), result.end(), List.of(), result.text()));
        return new Match(result.nodes(), result.local(), result.captures(), result.traces(), result.start(), result.end(),
            result.mappingFailure(), result.recoveries(), result.items(), result.text(), values);
    }
    private static Match combine(List<Match> items, int start, int end) {
        var nodes = new ArrayList<Recipe>(); var local = new ArrayList<Occurrence>();
        var captures = new ArrayList<Occurrence>(); var traces = new ArrayList<Trace>();
        for (Match item : items) { nodes.addAll(item.nodes()); local.addAll(item.local()); captures.addAll(item.captures()); traces.addAll(item.traces()); }
        return new Match(nodes, local, captures, traces, start, end, items.stream().anyMatch(Match::mappingFailure),
            items.stream().flatMap(m -> m.recoveries().stream()).toList(), List.copyOf(items), items.size() == 1 ? items.getFirst().text() : null,
            items.stream().anyMatch(i -> i.values() != null) ? items.stream().filter(i -> i.values() != null).flatMap(i -> i.values().stream()).toList() : null);
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
        return repeat(parent, child, 0, 1, null, new Boundary[0]);
    }
    public Match repeat(Frame parent, Call child, int min, int max, Call terminator, Boundary[] boundaries) {
        return repeat(parent, child, min, max, terminator, boundaries, "Repeat");
    }
    public Match repeat(Frame parent, Call child, int min, int max, Call terminator, Boundary[] boundaries, String label) {
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
            var mark = mark(); Match result = child.parse(this, f);
            if (result == null) { restore(mark); break; }
            matches.add(result); count++;
            if (before == f.position() || count >= max) break;
        }
        if (count < min || count > max) {
            fail(parent, label);
            if (label.startsWith("'") && !activeRules.isEmpty()) fail(Math.max(parent.c, parent.m), ruleNames[activeRules.getLast()] + "Parser");
            diagnostics.failureCursor(Math.max(parent.c, parent.m), f.c, f.m);
            restore(whole); return null;
        }
        boundary(f, boundaries, "helperExit", -1); boundary(f, boundaries, "exit", -1);
        parent.commit(f); Match combined = combine(matches, start, f.c);
        return new Match(combined.nodes(), combined.local(), combined.captures(), combined.traces(), start, f.c, combined.mappingFailure(), combined.recoveries(), matches, combined.text(),
            matches.isEmpty() ? List.of() : combined.values());
    }
    public Match separated(Frame parent, Call element, Call separator, int min, Boundary[] boundaries) {
        var whole = mark(); Frame f = parent.copy(true); int start = f.c;
        boundary(f, boundaries, "entry", -1); boundary(f, boundaries, "helperEntry", -1);
        var matches = new ArrayList<Match>(); var elements = new ArrayList<Match>(); var first = mark();
        Match initial = element.parse(this, f);
        if (initial == null) { restore(first); if (min > 0) { restore(whole); return null; } f = parent.copy(true); }
        else {
            matches.add(initial); elements.add(initial);
            while (true) {
                var mark = mark(); Frame trial = f.copy(true); int before = trial.position();
                Match sep = separator.parse(this, trial); Match next = sep == null ? null : element.parse(this, trial);
                if (next == null) { restore(mark); break; }
                f.commit(trial); matches.add(sep); matches.add(next); elements.add(next);
                if (before == f.position()) break;
            }
        }
        if (initial != null && (matches.size() + 1) / 2 < min) { restore(whole); return null; }
        boundary(f, boundaries, "helperExit", -1); boundary(f, boundaries, "exit", -1);
        parent.commit(f); Match result = combine(matches, start, f.c);
        return new Match(result.nodes(), result.local(), result.captures(), result.traces(), start, f.c,
            result.mappingFailure(), result.recoveries(), elements, result.text(), elements.isEmpty() ? List.of() : result.values());
    }
    public Match look(Frame parent, Call child, boolean negative, String label) {
        var mark = mark(); Frame trial = parent.copy(true); trial.matched = true;
        var diagnosticMark = negative ? diagnostics.checkpoint() : null;
        int priorReached = reached;
        Match result;
        try { result = child.parse(this, trial); }
        finally { restore(mark); }
        if (negative) {
            if (result != null) { fail(parent, label); return null; }
            diagnostics.restore(diagnosticMark); reached = priorReached;
            parent.m = parent.reset ? parent.c : parent.m; return Match.empty(parent.c, parent.c);
        }
        if (result == null) return null;
        parent.commit(trial); return Match.empty(parent.c, parent.c);
    }
    public Match literal(Frame f, String word, boolean sensitive, String label) {
        int at = f.position();
        boolean ok = !word.isEmpty() && (sensitive ? input.startsWith(at, word) : input.startsWithIgnoreCase(at, word));
        if (!ok) { fail(f, label); f.advance(0); return null; }
        f.advance(word.length()); return Match.empty(at, at + word.length());
    }
    public Match any(Frame f, String label) {
        int at = f.position(); if (cp(at) < 0) { fail(f, label); f.advance(0); return null; }
        int length = Character.charCount(cp(at)); f.advance(length); return Match.empty(at, at + length);
    }
    public Match empty(Frame f) { return optional(f, (s, p) -> s.look(p, (ss, pp) -> ss.any(pp, "ANY"), false, "EMPTY")); }
    public Match eof(Frame f) { return repeat(f, (s, p) -> s.any(p, "WildCardCharacterParser"), 0, 0, null, new Boundary[0], "EndOfSourceParser"); }
    public Match error(Frame f, String message) { fail(Math.max(f.c, f.m), message); return null; }
    private boolean digit(int p) { return cp(p) >= '0' && cp(p) <= '9'; }
    private void failures(int at, String... labels) { for (String label : labels) fail(at, label); }
    public Match number(Frame f) {
        int at = f.position(), p = at;
        if (cp(p) == '+' || cp(p) == '-') p++; else fail(p, "SignParser");
        int digits = p; while (digit(p)) p++;
        fail(p, "DigitParser");
        boolean hasDigits = p > digits;
        if (!hasDigits) fail(p, "OneOrMore");
        if (cp(p) == '.') {
            p++; int fraction = p; while (digit(p)) p++;
            fail(p, "DigitParser");
            if (p == fraction) fail(p, "OneOrMore");
            if (!hasDigits && p == fraction) { failures(at, "Chain", "'.'", "NumberParser"); f.advance(0); return null; }
        } else {
            fail(p, "'.'");
            if (!hasDigits) { fail(digits, "Chain"); fail(at, "NumberParser"); f.advance(0); return null; }
        }
        int mantissa = p;
        if (cp(p) == 'e' || cp(p) == 'E') {
            p++; if (cp(p) == '+' || cp(p) == '-') p++; else fail(p, "SignParser");
            int exponent = p; while (digit(p)) p++;
            fail(p, "DigitParser");
            if (p == exponent) { fail(p, "OneOrMore"); p = mantissa; }
        } else failures(p, "EParser", "ExponentParser");
        f.advance(p - at); return Match.empty(at, p);
    }
    public Match builtin(Frame f, String kind, int[] characters, String label) {
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
            if (!alphabet(cp(p)) && cp(p) != '_') { fail(f, "AlphabetUnderScoreParser"); return error(f, "IdentifierParser"); }
            p++; while (alphabet(cp(p)) || digit(p) || cp(p) == '_') p++;
            fail(p, "AlphabetNumericUnderScoreParser");
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
            if (!ok) return error(f, label); p += Character.charCount(ch);
        }
        f.advance(p - start);
        return new Match(List.of(), List.of(), List.of(), List.of(), start, p, false, List.of(), List.of(),
            kind.equals("Quoted") || kind.equals("SingleQuoted") ? source.substring(start + 1, p - 1) : null);
    }
    private static boolean alphabet(int ch) { return ch >= 'A' && ch <= 'Z' || ch >= 'a' && ch <= 'z'; }
    public Match range(Frame f, int min, int max, String excluded, String label) {
        int at = f.position(), ch = cp(at);
        if (ch < 0 || ch < min || ch > max || excluded != null && excluded.codePoints().anyMatch(v -> v == ch)) { Match failed = error(f, label); f.advance(0); return failed; }
        f.advance(Character.charCount(ch)); return Match.empty(at, f.position());
    }
    public Match dfa(Frame f, int state, int[] accepting, int[][] transitions, String label) {
        int start = f.position(), at = start, candidate = -1;
        while (at < input.length()) {
            int point = cp(at), next = -1;
            for (int[] transition : transitions) if (transition[0] == state && transition[1] <= point && point <= transition[2]) {
                next = transition[3]; break;
            }
            if (next < 0) break;
            state = next; at += Character.charCount(point);
            for (int accepted : accepting) if (state == accepted) { candidate = at; break; }
        }
        if (candidate <= start) return error(f, label);
        f.advance(candidate - start); return Match.empty(start, candidate);
    }
    public Match regex(Frame f, Pattern pattern, String label) {
        int at = f.position(); var matcher = pattern.matcher(source).region(at, source.length());
        if (!matcher.lookingAt() || matcher.end() == at) return error(f, label);
        input.cpIndex(matcher.end()); f.advance(matcher.end() - at); return Match.empty(at, matcher.end());
    }
    public Match until(Frame f, String terminator) {
        return repeat(f, (s, p) -> s.any(p, "ANY"), 0, Integer.MAX_VALUE,
            terminator.isEmpty() ? null : (s, p) -> s.look(p, (ss, pp) -> ss.literal(pp, terminator, true, "'" + terminator + "'"), false, "'" + terminator + "'"), new Boundary[0]);
    }
    public Match external(Frame f, String id) {
        TokenScanner scanner = options.scanners().get(id);
        if (scanner == null) throw new IllegalArgumentException("Missing TokenScanner: " + id);
        var result = scanner.scan(input, new TokenScanner.ScanState(f.c, f.m, f.matched ? TokenScanner.Mode.matchOnly : TokenScanner.Mode.consumed, f.invert, f.reset));
        for (var diagnostic : result.diagnostics()) { input.cpIndex(diagnostic.offsetUtf16()); fail(diagnostic.offsetUtf16(), diagnostic.expected()); }
        if (!result.ok()) return null;
        int valueStart = result.valueSpan() == null ? f.position() : result.valueSpan().start();
        int valueEnd = result.valueSpan() == null ? (f.matched ? result.matchedEnd() : result.consumedEnd()) : result.valueSpan().end();
        input.cpIndex(result.consumedEnd()); input.cpIndex(result.matchedEnd()); input.cpIndex(valueStart); input.cpIndex(valueEnd);
        if (result.consumedEnd() < f.c || result.matchedEnd() < result.consumedEnd() || valueEnd < valueStart
            || f.matched && result.consumedEnd() != f.c) throw new IllegalArgumentException("Invalid TokenScanner cursors: " + id);
        if (f.matched) f.m = result.matchedEnd(); else { f.c = result.consumedEnd(); f.m = result.matchedEnd(); }
        replay(result.effects()); return new Match(List.of(), List.of(), List.of(), List.of(), valueStart, valueEnd, false, List.of(), List.of(), source.substring(valueStart, valueEnd));
    }
    public void boundary(Frame f, Boundary[] boundaries, String at, int child) {
        for (var boundary : boundaries) if (boundary.at().equals(at) && boundary.child() == child) trivia(f, boundary.delimiters());
    }
    private void trivia(Frame f, Delimiter[] delimiters) {
        boolean again = true;
        while (again) {
            again = false;
            for (var delimiter : delimiters) {
                int p = f.position(), next = p, terminatorEnd = -1;
                if (delimiter.kind().equals("characters")) {
                    if (cp(p) >= 0 && Arrays.binarySearch(delimiter.characters(), cp(p)) >= 0) next += Character.charCount(cp(p));
                } else if (input.startsWith(p, delimiter.open())) {
                    if (delimiter.kind().equals("lineComment")) {
                        next += delimiter.open().length();
                        while (cp(next) >= 0 && cp(next) != '\r' && cp(next) != '\n') next += Character.charCount(cp(next));
                        terminatorEnd = cp(next) == '\r' && cp(next + 1) == '\n' ? next + 2 : cp(next) >= 0 ? next + 1 : next;
                    } else {
                        int end = source.indexOf(delimiter.close(), p + delimiter.open().length());
                        if (end < 0) failures(input.length(), "'*/'", "WildCardStringParser");
                        else next = end + delimiter.close().length();
                    }
                }
                if (next > p) {
                    f.advance(next - p);
                    // CPPComment's MatchOnly(LineTerminator) commits M, leaving C before the newline.
                    if (!f.matched && terminatorEnd >= 0) f.m = terminatorEnd;
                    progress(f);
                    again = true; break;
                }
                if (delimiter.kind().equals("characters")) fail(Math.max(f.c, f.m), "' '");
                else if (delimiter.kind().equals("lineComment")) fail(Math.max(f.c, f.m), "'" + delimiter.open() + "'");
                else if (!input.startsWith(p, delimiter.open())) failures(Math.max(f.c, f.m), "'" + delimiter.open() + "'", "'" + delimiter.close() + "'");
                // A bare @comment delimiter uses CPPComment directly. Composite whitespace
                // retains its own diagnostic profile, as in the Java-style delimiter scanner.
                if (delimiters.length == 1 && delimiter.kind().equals("lineComment")) fail(Math.max(f.c, f.m), "'" + delimiter.open() + "'");
            }
        }
    }
    public Match recover(Frame f, Call child, String rule, String mode, String[] patterns, boolean consumeSync,
                         String noSync, String projection, String recipe) {
        var mark = mark(); Frame trial = f.copy(false);
        diagnostics.enterExpression();
        Match result = child.parse(this, trial);
        var failure = diagnostics.leaveRule();
        if (result != null) { f.commit(trial); return result; }
        restore(mark); int start = f.position(), syncStart = -1, syncLength = 0;
        String selected = null;
        for (String pattern : patterns) {
            int found = source.indexOf(pattern, start);
            if (found >= 0 && (syncStart < 0 || found < syncStart)) { syncStart = found; syncLength = pattern.length(); selected = pattern; }
        }
        int end;
        if (mode.equals("SKIP")) {
            if (start >= input.length()) return null;
            end = patterns.length == 0 || syncStart == start ? start + Character.charCount(cp(start))
                : syncStart < 0 ? input.length() : syncStart;
        } else if (syncStart < 0) {
            if (noSync.equals("fail")) return null;
            end = noSync.equals("consumeToEof") ? input.length() : start;
        } else end = syncStart + (consumeSync ? syncLength : 0);
        if (end <= start) return null;
        f.advance(end - start);
        var hints = Arrays.stream(failure.expected()).mapToObj(expected::get).sorted().toList();
        var diagnostic = new Diagnostic("recovery", input.cpIndex(start), failure.farthest() < 0 ? input.cpIndex(start) : input.cpIndex(failure.farthest()),
            hints, failure.deepestRule() < 0 ? null : ruleNames[failure.deepestRule()],
            Arrays.stream(failure.ruleStack()).mapToObj(i -> ruleNames[i]).toList(), hints, "ERROR", input.cpLength(start, end), "recovery");
        String id = projection.equals("explicitRecipe") ? recipe : "#recovery";
        Recipe marker = new Recipe(id, start, end, Map.of(), null);
        var recovery = new RecoveryEvent(rule, mode, start, end, selected, syncStart, syncStart < 0 ? -1 : syncStart + syncLength, diagnostic, marker);
        return new Match(List.of(marker), List.of(), List.of(), List.of(), start, end,
            projection.equals("mappingFailure"), List.of(recovery));
    }
    public <N> ParseResult<N> result(Frame f, Match match, Map<String, String> catalogs) {
        boolean ok = match != null; String kind = ok ? "none" : "syntax"; int offset = -1;
        if (ok && options.requireFullInput() && f.c != input.length()) { ok = false; kind = "trailing_input"; offset = f.c; }
        if (!ok && offset < 0) offset = diagnostics.farthest() < 0 ? reached : diagnostics.farthest();
        var captures = new ArrayList<Capture>(); var lexical = new ArrayList<ParseResult.Lexical>();
        var catalog = new LinkedHashMap<String, List<Capture>>();
        if (ok || match != null && !match.recoveries().isEmpty()) {
            long occurrence = 0;
            for (var capture : match.captures()) {
                var value = new Capture(occurrence, capture.site(), capture.name(), span(capture.value().start(), capture.value().end()), occurrence++, completedCaptures.getOrDefault(capture, -1L));
                captures.add(value);
                String context = catalogs.get(capture.site()); if (context != null) catalog.computeIfAbsent(context, key -> new ArrayList<>()).add(value);
            }
            long[] counter = {0, 0}; for (var trace : match.traces()) trace(trace, -1, counter, lexical);
        }
        catalog.replaceAll((key, values) -> List.copyOf(values));
        var farthestExpected = Arrays.stream(diagnostics.expected()).mapToObj(expected::get).sorted().toList();
        var scopeEvents = new ArrayList<ParseResult.ScopeEvent>();
        for (var effect : effects) scopeEvents.add(new ParseResult.ScopeEvent(scopeEvents.size(), effect.action(), effect.mode(), effect.name(), effect.offsetCp(), effect.lengthCp()));
        var diagnostic = new Diagnostic(kind, offset < 0 ? -1 : input.cpIndex(offset), diagnostics.farthest() < 0 ? kind.equals("trailing_input") ? input.cpIndex(Math.max(diagnostics.reached(), Math.max(f.c, f.m))) : -1 : input.cpIndex(diagnostics.farthest()),
            kind.equals("trailing_input") ? List.of("end of input") : farthestExpected,
            diagnostics.deepestRule() < 0 ? null : ruleNames[diagnostics.deepestRule()],
            Arrays.stream(diagnostics.ruleStack()).mapToObj(i -> ruleNames[i]).toList(), farthestExpected);
        var recoveries = new ArrayList<ParseResult.Recovery>();
        if (match != null) for (RecoveryEvent recovery : match.recoveries()) {
            var occurrences = new ArrayList<Long>();
            for (int i = 0; i < match.captures().size(); i++)
                if (match.captures().get(i).value().nodes().stream().anyMatch(n -> n == recovery.marker())) occurrences.add((long) i);
            recoveries.add(new ParseResult.Recovery(recovery.rule(), recovery.mode(), span(recovery.start(), recovery.end()), recovery.pattern(),
                span(recovery.start(), recovery.mode().equals("SKIP") || recovery.syncStart() < 0 ? recovery.end() : Math.min(recovery.end(), recovery.syncStart())),
                recovery.syncStart() < 0 ? null : span(recovery.syncStart(), recovery.syncEnd()), occurrences, recovery.diagnostic()));
        }
        var result = new ParseResult<N>(ok, input.cpIndex(f.c), input.cpIndex(f.m), Optional.empty(), Map.of(), captures, lexical,
            new ParseResult.Scope(scope.allDeclarations(), scope.references(), scope.diagnostics(), scopeEvents, scope.depth(), resolved()), diagnostic, catalog, recoveries,
            new ParseResult.Failure(diagnostics.farthest() < 0 ? input.cpIndex(f.c) : input.cpIndex(diagnostics.farthest()),
                input.cpIndex(diagnostics.farthest() < 0 ? f.c : diagnostics.snapshot().consumed()), input.cpIndex(diagnostics.farthest() < 0 ? f.m : diagnostics.snapshot().matched()), farthestExpected));
        syntax = result; return result;
    }
    private Map<String, ScopeStore.Decl> resolved() {
        var resolved = new LinkedHashMap<String, ScopeStore.Decl>();
        for (var declaration : scope.allDeclarations()) {
            var visible = scope.resolve(declaration.name());
            if (visible != null) resolved.put(declaration.name(), visible);
        }
        return resolved;
    }
    public void rollback(Frame f) { scope.restore(0); effects.clear(); f.c = 0; f.m = 0; }
    private void trace(Trace trace, long parent, long[] counter, List<ParseResult.Lexical> out) {
        long id = counter[0]++;
        for (var child : trace.children()) trace(child, id, counter, out);
        out.add(new ParseResult.Lexical(id, parent, trace.rule(), trace.expr(), span(trace.start(), trace.end()), counter[1]++, trace.token()));
    }
    public String text(Value value) {
        String text = new String(value.text() == null ? source.substring(value.start(), value.end()).strip() : value.text());
        spans.put(text, span(value.start(), value.end())); return text;
    }
    public int number(Value value) {
        try { return Integer.parseInt(text(value)); }
        catch (NumberFormatException error) { throw mapping(value.start(), value.end(), "NUMBER cannot be mapped to int", error); }
    }
    public Recipe node(Value value, Recipe owner) {
        if (value.nodes().size() > 1) throw mapping(value.start(), value.end(), "Scalar capture has multiple mapped nodes", null);
        if (!value.nodes().isEmpty()) return value.nodes().getFirst();
        if (owner.fallback() != null) return new Recipe("#leaf:" + owner.fallback(), value.start(), value.end(), Map.of(), null);
        throw mapping(value.start(), value.end(), "Missing mapped node", null);
    }
    public <T> T scalar(Recipe recipe, String field, Function<Value, T> conversion) {
        List<Value> values = recipe.fields().getOrDefault(field, List.of());
        if (values.isEmpty()) throw mapping(recipe.start(), recipe.end(), "Missing scalar capture: " + field, null);
        T value = conversion.apply(values.getFirst());
        if (value == null && !values.getFirst().nodes().isEmpty()) throw new RecoveredNodeMissing();
        return value;
    }
    public <T> Optional<T> optional(Recipe recipe, String field, Function<Value, T> conversion) {
        List<Value> values = recipe.fields().getOrDefault(field, List.of());
        return values.isEmpty() ? Optional.empty() : Optional.ofNullable(conversion.apply(values.getFirst()));
    }
    public <T> List<T> list(Recipe recipe, String field, Function<Value, T> conversion) {
        return recipe.fields().getOrDefault(field, List.of()).stream().map(conversion).filter(java.util.Objects::nonNull).toList();
    }
    public MappingException mapping(int start, int end, String message, Throwable cause) { return new MappingException(syntax, span(start, end), message, cause); }
    /** A missing recovered scalar omits its parent; optional/list callers absorb that absence. */
    public static final class RecoveredNodeMissing extends RuntimeException {
        private static final long serialVersionUID = 1L;
        private RecoveredNodeMissing() { super(null, null, false, false); }
    }
    public void typeof(Recipe recipe, String ownName, Object own, String refName, Object referenced) {
        if (own != null && referenced != null && !own.getClass().equals(referenced.getClass()))
            {
            String message = "@typeof constraint violated: " + ownName + " must be same type as " + refName
                + ", expected " + referenced.getClass().getSimpleName() + " but got " + own.getClass().getSimpleName();
            throw mapping(recipe.start(), recipe.end(), message, new IllegalArgumentException(message));
        }
    }
    public boolean hasBuilt(Recipe recipe) { return built.containsKey(recipe); }
    public Object built(Recipe recipe) { return built.get(recipe); }
    public Object remember(Recipe recipe, Object object) { built.put(recipe, object); if (object != null) spans.put(object, span(recipe.start(), recipe.end())); return object; }
}
