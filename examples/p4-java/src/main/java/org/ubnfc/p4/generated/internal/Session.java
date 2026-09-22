package org.ubnfc.p4.generated.internal;

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
import org.ubnfc.p4.generated.api.Capture;
import org.ubnfc.p4.generated.api.Diagnostic;
import org.ubnfc.p4.generated.api.MappingException;
import org.ubnfc.p4.generated.api.ParseOptions;
import org.ubnfc.p4.generated.api.ParseResult;
import org.ubnfc.p4.generated.api.Span;
import org.ubnfc.p4.generated.api.TokenScanner;
import org.ubnfc.p4.generated.rt.Diagnostics;
import org.ubnfc.p4.generated.rt.Input;
import org.ubnfc.p4.generated.rt.MemoTable;
import org.ubnfc.p4.generated.rt.ScopeStore;
import org.ubnfc.p4.generated.rt.Severity;
import org.ubnfc.p4.generated.internal.Recipe.Value;

/** Per-parse state; transactions preserve syntax diagnostics but undo semantic effects. */
public final class Session {
    @FunctionalInterface public interface Call { Match parse(Session session, Frame frame); }
    /** Expected-hint label with a process-wide stable id; generated code holds these as static constants so failures never hash strings. */
    public static final class Label {
        private static final java.util.concurrent.ConcurrentHashMap<String, Label> REGISTRY = new java.util.concurrent.ConcurrentHashMap<>();
        private static volatile String[] texts = new String[256];
        private static int count;
        public final String text; public final int id;
        /** A group label stands for several expected hints registered at one position (trivia delimiter sets); expanded on output. */
        final Label[] members;
        private Label(String text, int id, Label[] members) { this.text = text; this.id = id; this.members = members; }
        public static Label of(String text) {
            Label label = REGISTRY.get(text);
            if (label != null) return label;
            synchronized (Label.class) {
                label = REGISTRY.get(text);
                if (label == null) {
                    if (count == texts.length) texts = Arrays.copyOf(texts, count * 2);
                    texts[count] = text; label = new Label(text, count++, null); REGISTRY.put(text, label);
                }
                return label;
            }
        }
        public static String text(int id) { return texts[id]; }
        static Label group(Label[] members) {
            var names = new StringBuilder("\u0000group");
            for (Label member : members) names.append('\u0001').append(member.text);
            String key = names.toString();
            Label label = REGISTRY.get(key);
            if (label != null) return label;
            synchronized (Label.class) {
                label = REGISTRY.get(key);
                if (label == null) {
                    if (count == texts.length) texts = Arrays.copyOf(texts, count * 2);
                    texts[count] = key; label = new Label(key, count++, members.clone()); REGISTRY.put(key, label);
                }
                return label;
            }
        }
        static int count() { return count; }
        /** Expand ids into sorted distinct hint texts (group members flattened). */
        static List<String> texts(int[] ids) {
            var result = new java.util.TreeSet<String>();
            for (int id : ids) {
                Label label = REGISTRY.get(texts[id]);
                if (label.members == null) result.add(label.text); else for (Label member : label.members) result.add(member.text);
            }
            return List.copyOf(result);
        }
    }
    private static final Label L_SPACE = Label.of("' '"), L_BLOCK_END = Label.of("'*/'"), L_WILDCARD_STRING = Label.of("WildCardStringParser"),
        L_SIGN = Label.of("SignParser"), L_DIGIT = Label.of("DigitParser"), L_ONE_OR_MORE = Label.of("OneOrMore"), L_DOT = Label.of("'.'"),
        L_CHAIN = Label.of("Chain"), L_NUMBER = Label.of("NumberParser"), L_E = Label.of("EParser"), L_EXPONENT = Label.of("ExponentParser"),
        L_ALPHABET_UNDERSCORE = Label.of("AlphabetUnderScoreParser"), L_IDENTIFIER = Label.of("IdentifierParser"), L_ALNUM_UNDERSCORE = Label.of("AlphabetNumericUnderScoreParser"),
        L_ANY = Label.of("ANY"), L_EMPTY = Label.of("EMPTY"), L_WILDCARD_CHAR = Label.of("WildCardCharacterParser"), L_EOS = Label.of("EndOfSourceParser"), L_REPEAT = Label.of("Repeat");
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
    /** Capture occurrence; completion order is stamped by the completing rule (was an identity map). */
    public static final class Occurrence {
        private final String site, name; private final Value value; long completed = -1;
        public Occurrence(String site, String name, Value value) { this.site = site; this.name = name; this.value = value; }
        public String site() { return site; } public String name() { return name; } public Value value() { return value; }
    }
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
        // Lists are built only inside Session and never mutated after construction; no defensive copies.
        public static Match empty(int start, int end) { return new Match(List.of(), List.of(), List.of(), List.of(), start, end); }
    }
    /** Trivia delimiter with labels and an ASCII bitmap precomputed once (generated code builds these as static constants). */
    public static final class Delimiter {
        private final String kind, open, close; private final int[] characters;
        final boolean isCharacters, isLineComment; final Label openLabel, closeLabel; final long asciiLow, asciiHigh; final boolean asciiOnly; final char first;
        public Delimiter(String kind, String open, String close, int[] characters) {
            this.kind = kind; this.open = open; this.close = close; this.characters = characters;
            isCharacters = kind.equals("characters"); isLineComment = kind.equals("lineComment");
            openLabel = open == null ? null : Label.of("'" + open + "'"); closeLabel = close == null ? null : Label.of("'" + close + "'");
            long low = 0, high = 0; boolean ascii = true;
            for (int ch : characters) { if (ch < 64) low |= 1L << ch; else if (ch < 128) high |= 1L << (ch - 64); else ascii = false; }
            asciiLow = low; asciiHigh = high; asciiOnly = ascii; first = open == null || open.isEmpty() ? 0 : open.charAt(0);
        }
        public String kind() { return kind; } public String open() { return open; } public String close() { return close; } public int[] characters() { return characters; }
        boolean contains(int ch) {
            if (ch < 64) return ch >= 0 && (asciiLow & (1L << ch)) != 0;
            if (ch < 128) return (asciiHigh & (1L << (ch - 64))) != 0;
            return !asciiOnly && Arrays.binarySearch(characters, ch) >= 0;
        }
    }
    /** A delimiter set with the failure hint groups of each failing prefix precomputed (one registered id per trivia round). */
    public static final class Delimiters {
        final Delimiter[] items; final Label[] prefixGroups;
        public Delimiters(Delimiter... items) {
            this.items = items; prefixGroups = new Label[items.length];
            var members = new ArrayList<Label>();
            for (int i = 0; i < items.length; i++) {
                if (items[i].isCharacters) members.add(L_SPACE);
                else if (items[i].isLineComment) members.add(items[i].openLabel);
                else { members.add(items[i].openLabel); members.add(items[i].closeLabel); }
                prefixGroups[i] = members.size() == 1 ? members.getFirst() : Label.group(members.toArray(new Label[0]));
            }
        }
    }
    /** Trivia insertion plan of one sequence/repeat: null means no trivia at that boundary. */
    public static final class Trivia {
        public static final Trivia NONE = new Trivia(null, null, null, null, null);
        final Delimiters entry, helperEntry, helperExit, exit; final Delimiters[] afterChild;
        public Trivia(Delimiters entry, Delimiters helperEntry, Delimiters[] afterChild, Delimiters helperExit, Delimiters exit) {
            this.entry = entry; this.helperEntry = helperEntry; this.afterChild = afterChild; this.helperExit = helperExit; this.exit = exit;
        }
    }
    public record Field(String name, String[] sites, String conversion, String cardinality) {}
    public record Effect(String when, String action, String site, String name, String mode) {}
    private record Memo(Match match, boolean reset, List<TokenScanner.Effect> effects, Diagnostics.Snapshot diagnostics) {}
    public final String source;
    public final Input input;
    public final ParseOptions options;
    public final ScopeStore scope = new ScopeStore();
    private Diagnostics diagnostics;
    private final MemoTable memo;
    private final ArrayList<Memo> payloads = new ArrayList<>();
    private final ArrayList<TokenScanner.Effect> effects = new ArrayList<>();
    private final String[] ruleNames; private final Label[] ruleParserLabels;
    private int depth;
    private int[] activeRules = new int[64]; private int activeDepth;
    private final int[] activeCounts;
    private int reached;
    private ParseResult<?> syntax;
    private long nextCaptureCompletion;
    public final IdentityHashMap<Object, Span> spans = new IdentityHashMap<>();
    private final IdentityHashMap<Recipe, Object> built = new IdentityHashMap<>();
    public Session(CharSequence source, ParseOptions options, String[] ruleNames) {
        this.source = source.toString(); this.input = new Input(this.source); this.options = options; this.ruleNames = ruleNames.clone();
        this.activeCounts = new int[ruleNames.length];
        diagnostics = new Diagnostics(Label.count(), Math.max(256, Math.min(1 << 20, input.length() * 64)));
        ruleParserLabels = new Label[ruleNames.length];
        // 観測: P4 で 1 文字あたり約 5 entry。成長の再ハッシュを避けつつ上限は固定する。
        memo = new MemoTable(Math.max(1024, Math.min(1 << 20, input.length() * 8)));
    }
    private static final class DepthLimit extends RuntimeException {
        private static final long serialVersionUID = 1L;
        private DepthLimit() { super(null, null, false, false); }
    }
    /** Abort the entire parse: an outer optional/choice must not swallow a resource limit. */
    public Match parse(Call entry, Frame f) {
        long start = mark(); int c0 = f.c, m0 = f.m; boolean reset0 = f.reset;
        try { return entry.parse(this, f); }
        catch (DepthLimit | StackOverflowError limit) {
            restore(start); f.c = c0; f.m = m0; f.reset = reset0;
            diagnostics = new Diagnostics(Label.count(), 256);
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
    private void fail(int at, String label) { fail(at, Label.of(label).id, at, at); }
    private void fail(int at, Label label) { fail(at, label.id, at, at); }
    private void fail(Frame frame, Label label) { fail(Math.max(frame.c, frame.m), label.id, frame.c, frame.m); }
    private void fail(int at, int id, int consumed, int matched) {
        diagnostics.failAt(at, id, consumed, matched); if (at > reached) reached = at;
    }
    private long mark() { return ((long) scope.checkpoint() << 32) | effects.size(); }
    private void restore(long mark) {
        scope.restore((int) (mark >>> 32)); int size = (int) mark;
        if (effects.size() > size) effects.subList(size, effects.size()).clear();
    }
    private List<TokenScanner.Effect> effectsSince(long mark) {
        int size = (int) mark; return effects.size() == size ? List.of() : List.copyOf(effects.subList(size, effects.size()));
    }
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
                      String recipe, String fallback, String recipeKind, Field[] fields, Effect[] scopeEffects, boolean skip, Label[] recoveryHints, boolean rightAssociative, Label[] failureHints) {
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
        boolean recursive = activeCounts[id]++ > 0;
        if (activeDepth == activeRules.length) activeRules = Arrays.copyOf(activeRules, activeDepth * 2);
        activeRules[activeDepth++] = id;
        long mark = mark(); diagnostics.enterRule(id); progress(f);
        if (scopeEffects.length > 0) applyEffects(scopeEffects, "entry", Match.empty(c, c));
        Match result = body.parse(this, f);
        if (result != null) {
            if (scopeEffects.length > 0) {
                applyEffects(scopeEffects, "successBeforeLeave", result);
                applyEffects(scopeEffects, "successAfterLeave", result);
                applyEffects(scopeEffects, "afterDeclarations", result);
            }
            List<Recipe> nodes = result.nodes();
            boolean skipped = skip || "skip".equals(recipeKind);
            if (skipped) nodes = List.of();
            else if (recipe != null && !"transparent".equals(recipeKind) && !"text".equals(recipeKind)) {
                var names = new String[fields.length]; List<Value>[] lists = Recipe.lists(fields.length);
                for (int fi = 0; fi < fields.length; fi++) {
                    Field field = fields[fi];
                    var captures = new ArrayList<Value>();
                    boolean nodeLike = "node".equals(field.conversion()) || "mixed".equals(field.conversion());
                    boolean list = field.cardinality().equals("list"), scalar = field.cardinality().equals("scalar");
                    List<Occurrence> local = result.local();
                    for (int k = 0, n = local.size(); k < n; k++) { Occurrence occurrence = local.get(k); if (!contains(field.sites(), occurrence.site())) continue;
                        Value v = occurrence.value();
                        if (nodeLike && !scalar && v.elements() != null)
                            captures.addAll(v.elements());
                        else if (nodeLike && list && !v.nodes().isEmpty())
                            for (Recipe node : v.nodes()) captures.add(new Value(node.start(), node.end(), List.of(node)));
                        else if (!(nodeLike && list && v.start() == v.end() && v.nodes().isEmpty())) captures.add(v);
                    }
                    names[fi] = field.name(); lists[fi] = List.copyOf(captures);
                }
                int nodeEnd = f.c;
                if (rightAssociative && recursive) {
                    int max = Integer.MIN_VALUE;
                    for (List<Value> list : lists) for (int k = 0; k < list.size(); k++) max = Math.max(max, list.get(k).end());
                    if (max != Integer.MIN_VALUE) nodeEnd = max;
                }
                nodes = List.of(new Recipe(recipe, c, nodeEnd, names, lists, fallback));
            }
            List<Occurrence> completed = result.local();
            for (int k = 0, n = completed.size(); k < n; k++) completed.get(k).completed = nextCaptureCompletion++;
            List<Value> values = result.values();
            if (skipped || nodes != result.nodes()) {
                values = nodes.isEmpty() ? List.of() : List.of(new Value(nodes.getFirst().start(), nodes.getFirst().end(), nodes));
            }
            result = new Match(nodes, List.of(), result.captures(), result.traces(), c, f.c,
                result.mappingFailure(), result.recoveries(), List.of(), result.text(), values);
        } else {
            int localFarthest = diagnostics.localFarthest();
            if (localFarthest == Math.max(c, m)) for (Label hint : recoveryHints) fail(localFarthest, hint);
            for (Label hint : failureHints) fail(Math.max(c, m), hint);
            restore(mark); f.c = c; f.m = m;
        }
        depth--; activeCounts[id]--; activeDepth--;
        boolean store = cache && (result != null ? successMemo : failureMemo);
        var diagnostic = diagnostics.leaveRule(store);
        if (store) {
            int payload = payloads.size();
            payloads.add(new Memo(result, f.reset, effectsSince(mark), diagnostic));
            memo.put(memoId, c, m, mode, state, result != null, f.c, f.m, payload);
        }
        return result;
    }
    private static boolean contains(String[] sites, String site) {
        for (String candidate : sites) if (candidate == site || candidate.equals(site)) return true;
        return false;
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
        long mark = mark(); diagnostics.enterExpression(); progress(f);
        Match result = body.parse(this, f);
        boolean store = result != null ? successMemo : failureMemo;
        var diagnostic = diagnostics.leaveRule(store);
        if (store) {
            int payload = payloads.size();
            payloads.add(new Memo(result, f.reset, effectsSince(mark), diagnostic));
            memo.put(id, c, m, mode, state, result != null, f.c, f.m, payload);
        }
        return result;
    }
    private void applyEffects(Effect[] definitions, String when, Match result) {
        HashMap<String, String> declared = null;
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
                    String target = declared == null ? null : declared.get(key);
                    for (Occurrence own : result.local()) if (own.name().equals(key)) {
                        target = source.substring(own.value().start(), own.value().end()).trim(); break;
                    }
                    if (definition.action().equals("declare")) { if (declared == null) declared = new HashMap<>(); declared.put(key, name); }
                    else if (definition.action().equals("use") && target != null && !name.equals(target))
                        effect(new TokenScanner.Effect("error", "back-reference mismatch: expected '" + target + "' but got '" + name + "'",
                            input.cpIndex(trimmedStart), input.cpLength(trimmedStart, trimmedEnd)));
                } else effect(new TokenScanner.Effect(definition.action(), name, input.cpIndex(trimmedStart), input.cpLength(trimmedStart, trimmedEnd), definition.mode()));
            }
        }
    }
    public Match capture(Frame f, Call call, String[] sites, String[] names, String[] spanRules, boolean[] skips,
                         String rule, String expr, boolean quantified, boolean terminal, String semanticKind, boolean semanticMany, Label[] failureHints) {
        int c = f.c, m = f.m; Match result = call.parse(this, f);
        progress(f);
        if (result == null) {
            for (Label hint : failureHints) fail(Math.max(c, m), hint);
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
                for (int i = 0; i < sites.length; i++) value = captured(value, item.start(), item.end(), sites[i], names[i], skips[i]);
                items.add(value);
            }
            Match joined = combine(items, result.start(), result.end());
            return new Match(result.nodes(), joined.local(), joined.captures(), result.traces(), result.start(), result.end(),
                result.mappingFailure(), result.recoveries(), items, result.text(), result.values());
        }
        Match captured = result;
        for (int i = 0; i < sites.length; i++) {
            int start = spanRules[i].equals("matchedExtent") ? m : spanRules[i].equals("selectedChildExtent") ? result.start() : c;
            int end = spanRules[i].equals("matchedExtent") ? f.m : spanRules[i].equals("selectedChildExtent") ? result.end() : f.c;
            captured = captured(captured, start, Math.max(start, end), sites[i], names[i], skips[i]);
        }
        if (!options.lexical()) return captured;
        List<Trace> traces = List.of(new Trace(rule, expr, c, f.c, result.traces(), terminal ? source.substring(result.start(), result.end()) : null));
        return new Match(captured.nodes(), captured.local(), captured.captures(), traces, result.start(), result.end(),
            result.mappingFailure(), result.recoveries(), result.items(), result.text(), result.values());
    }
    private Match captured(Match result, int start, int end, String site, String name, boolean skip) {
        if (skip) return result;
        var occurrence = new Occurrence(site, name, new Value(start, end, result.nodes(), result.text(), result.values()));
        return new Match(result.nodes(), appended(result.local(), occurrence), appended(result.captures(), occurrence), result.traces(), result.start(), result.end(),
            result.mappingFailure(), result.recoveries(), result.items(), result.text(), result.values());
    }
    private static List<Occurrence> appended(List<Occurrence> list, Occurrence occurrence) {
        if (list.isEmpty()) return List.of(occurrence);
        var copy = new Occurrence[list.size() + 1]; list.toArray(copy); copy[list.size()] = occurrence; return List.of(copy);
    }
    private Match project(Match result, String kind, boolean many) {
        if (kind.equals("text")) return result;
        List<Value> values = result.values();
        if (values == null) values = List.of(new Value(result.start(), result.end(), result.nodes(), result.text()));
        if (kind.equals("mixed") && !many && !values.isEmpty() && allLeaves(values))
            values = List.of(new Value(result.start(), result.end(), List.of(), result.text()));
        if (values == result.values()) return result;
        return new Match(result.nodes(), result.local(), result.captures(), result.traces(), result.start(), result.end(),
            result.mappingFailure(), result.recoveries(), result.items(), result.text(), values);
    }
    private static boolean allLeaves(List<Value> values) {
        for (int k = 0, n = values.size(); k < n; k++) if (!values.get(k).nodes().isEmpty()) return false;
        return true;
    }
    private static Match combine(List<Match> items, int start, int end) {
        if (items.size() == 1) {
            Match only = items.getFirst();
            return new Match(only.nodes(), only.local(), only.captures(), only.traces(), start, end, only.mappingFailure(), only.recoveries(), items, only.text(), only.values());
        }
        List<Recipe> nodes = List.of(); List<Occurrence> local = List.of(), captures = List.of(); List<Trace> traces = List.of();
        List<RecoveryEvent> recoveries = List.of(); List<Value> values = null; boolean failure = false;
        for (int k = 0, n = items.size(); k < n; k++) {
            Match item = items.get(k);
            if (!item.nodes().isEmpty()) nodes = concat(nodes, item.nodes());
            if (!item.local().isEmpty()) local = concat(local, item.local());
            if (!item.captures().isEmpty()) captures = concat(captures, item.captures());
            if (!item.traces().isEmpty()) traces = concat(traces, item.traces());
            if (!item.recoveries().isEmpty()) recoveries = concat(recoveries, item.recoveries());
            if (item.values() != null) values = values == null ? item.values() : concat(values, item.values());
            failure |= item.mappingFailure();
        }
        return new Match(nodes, local, captures, traces, start, end, failure, recoveries, items, null, values);
    }
    /** Immutable concatenation; both inputs are already immutable lists. */
    private static <T> List<T> concat(List<T> head, List<T> tail) {
        if (head.isEmpty()) return tail;
        if (tail.isEmpty()) return head;
        var merged = new ArrayList<T>(head.size() + tail.size()); merged.addAll(head); merged.addAll(tail); return merged;
    }
    // Transactions run on the caller's frame in place: begin saves (c, m, reset) and applies
    // "m := c when reset"; failure restores c/m; either outcome restores reset (a commit never
    // copied the child's reset flag). Observable cursor behaviour equals the frame-copy version.
    public Match sequence(Frame f, Call[] children, Trivia trivia) {
        long mark = mark(); int c0 = f.c, m0 = f.m; boolean reset0 = f.reset;
        if (reset0) f.m = f.c;
        int start = f.c;
        if (trivia.entry != null) trivia(f, trivia.entry); if (trivia.helperEntry != null) trivia(f, trivia.helperEntry);
        var matches = new Match[children.length];
        for (int i = 0; i < children.length; i++) {
            Match result = children[i].parse(this, f);
            if (result == null) { restore(mark); f.c = c0; f.m = m0; f.reset = reset0; return null; }
            matches[i] = result; if (trivia.afterChild != null && trivia.afterChild[i] != null) trivia(f, trivia.afterChild[i]);
        }
        if (trivia.helperExit != null) trivia(f, trivia.helperExit); if (trivia.exit != null) trivia(f, trivia.exit);
        f.reset = reset0; return combine(Arrays.asList(matches), start, f.c);
    }
    public Match choice(Frame f, Call[] alternatives, boolean longest, boolean[] eligible, boolean fallback) {
        Match best = null; int bestC = 0, bestM = 0; List<TokenScanner.Effect> bestEffects = List.of();
        int c0 = f.c, m0 = f.m; boolean reset0 = f.reset;
        for (int pass = 0; pass < (fallback ? 2 : 1); pass++) {
            for (int i = 0; i < alternatives.length; i++) {
                if (pass == 0 && eligible != null && !eligible[i]) continue;
                long mark = mark(); f.c = c0; f.m = reset0 ? c0 : m0; f.reset = reset0;
                Match result = alternatives[i].parse(this, f);
                if (result != null && !longest) { f.reset = reset0; return result; }
                if (result != null && (best == null || f.c > bestC)) {
                    best = result; bestC = f.c; bestM = f.m; bestEffects = effectsSince(mark);
                }
                restore(mark);
            }
            if (best != null) break;
        }
        f.reset = reset0;
        if (best != null) { f.c = bestC; f.m = bestM; replay(bestEffects); } else { f.c = c0; f.m = m0; }
        return best;
    }
    public Match optional(Frame parent, Call child) {
        return repeat(parent, child, 0, 1, null, Trivia.NONE);
    }
    public Match repeat(Frame parent, Call child, int min, int max, Call terminator, Trivia trivia) {
        return repeat(parent, child, min, max, terminator, trivia, L_REPEAT);
    }
    public Match repeat(Frame parent, Call child, int min, int max, Call terminator, Trivia trivia, String label) {
        return repeat(parent, child, min, max, terminator, trivia, Label.of(label));
    }
    public Match repeat(Frame f, Call child, int min, int max, Call terminator, Trivia trivia, Label label) {
        long whole = mark(); int c0 = f.c, m0 = f.m; boolean reset0 = f.reset, matched0 = f.matched;
        if (reset0) f.m = f.c;
        int start = f.c, count = 0;
        if (trivia.entry != null) trivia(f, trivia.entry); if (trivia.helperEntry != null) trivia(f, trivia.helperEntry);
        var matches = new ArrayList<Match>();
        while (true) {
            int before = f.position();
            if (terminator != null) {
                // The terminator runs in consumed mode on a begun frame; its cursors commit only when the loop itself is in consumed mode.
                long mark = mark(); int c1 = f.c, m1 = f.m; boolean reset1 = f.reset;
                if (reset1) f.m = f.c;
                f.matched = false;
                Match done = terminator.parse(this, f);
                f.matched = matched0; f.reset = reset1;
                if (done != null) {
                    if (!matched0) matches.add(done);
                    else { restore(mark); f.c = c1; f.m = m1; }
                    break;
                }
                restore(mark); f.c = c1; f.m = m1;
            }
            long mark = mark(); Match result = child.parse(this, f);
            if (result == null) { restore(mark); break; }
            matches.add(result); count++;
            if (before == f.position() || count >= max) break;
        }
        if (count < min || count > max) {
            int at = Math.max(c0, m0), fc = f.c, fm = f.m;
            f.c = c0; f.m = m0; f.reset = reset0;
            fail(at, label.id, c0, m0);
            if (label.text.startsWith("'") && activeDepth > 0) {
                int rule = activeRules[activeDepth - 1];
                if (ruleParserLabels[rule] == null) ruleParserLabels[rule] = Label.of(ruleNames[rule] + "Parser");
                fail(at, ruleParserLabels[rule]);
            }
            diagnostics.failureCursor(at, fc, fm);
            restore(whole); return null;
        }
        if (trivia.helperExit != null) trivia(f, trivia.helperExit); if (trivia.exit != null) trivia(f, trivia.exit);
        f.reset = reset0; Match combined = combine(matches, start, f.c);
        return new Match(combined.nodes(), combined.local(), combined.captures(), combined.traces(), start, f.c, combined.mappingFailure(), combined.recoveries(), matches, combined.text(),
            matches.isEmpty() ? List.of() : combined.values());
    }
    public Match separated(Frame f, Call element, Call separator, int min, Trivia trivia) {
        long whole = mark(); int c0 = f.c, m0 = f.m; boolean reset0 = f.reset;
        if (reset0) f.m = f.c;
        int start = f.c;
        if (trivia.entry != null) trivia(f, trivia.entry); if (trivia.helperEntry != null) trivia(f, trivia.helperEntry);
        var matches = new ArrayList<Match>(); var elements = new ArrayList<Match>(); long first = mark();
        Match initial = element.parse(this, f);
        if (initial == null) {
            restore(first);
            if (min > 0) { restore(whole); f.c = c0; f.m = m0; f.reset = reset0; return null; }
            f.c = c0; f.m = reset0 ? c0 : m0; f.reset = reset0;
        } else {
            matches.add(initial); elements.add(initial);
            while (true) {
                long mark = mark(); int c1 = f.c, m1 = f.m; boolean reset1 = f.reset;
                if (reset1) f.m = f.c;
                int before = f.position();
                Match sep = separator.parse(this, f); Match next = sep == null ? null : element.parse(this, f);
                f.reset = reset1;
                if (next == null) { restore(mark); f.c = c1; f.m = m1; break; }
                matches.add(sep); matches.add(next); elements.add(next);
                if (before == f.position()) break;
            }
        }
        if (initial != null && (matches.size() + 1) / 2 < min) { restore(whole); f.c = c0; f.m = m0; f.reset = reset0; return null; }
        if (trivia.helperExit != null) trivia(f, trivia.helperExit); if (trivia.exit != null) trivia(f, trivia.exit);
        f.reset = reset0; Match result = combine(matches, start, f.c);
        return new Match(result.nodes(), result.local(), result.captures(), result.traces(), start, f.c,
            result.mappingFailure(), result.recoveries(), elements, result.text(), elements.isEmpty() ? List.of() : result.values());
    }
    public Match look(Frame parent, Call child, boolean negative, String label) { return look(parent, child, negative, Label.of(label)); }
    public Match look(Frame f, Call child, boolean negative, Label label) {
        long mark = mark(); int c0 = f.c, m0 = f.m; boolean reset0 = f.reset, matched0 = f.matched;
        if (reset0) f.m = f.c;
        f.matched = true;
        var diagnosticMark = negative ? diagnostics.checkpoint() : null;
        int priorReached = reached;
        Match result;
        try { result = child.parse(this, f); }
        finally { restore(mark); f.matched = matched0; f.reset = reset0; }
        if (negative) {
            f.c = c0; f.m = m0;
            if (result != null) { fail(f, label); return null; }
            diagnostics.restore(diagnosticMark); reached = priorReached;
            f.m = reset0 ? c0 : m0; return Match.empty(c0, c0);
        }
        if (result == null) { f.c = c0; f.m = m0; return null; }
        return Match.empty(f.c, f.c);
    }
    public Match literal(Frame f, String word, boolean sensitive, String label) { return literal(f, word, sensitive, Label.of(label)); }
    public Match literal(Frame f, String word, boolean sensitive, Label label) {
        int at = f.position();
        boolean ok = !word.isEmpty() && (sensitive ? input.startsWith(at, word) : input.startsWithIgnoreCase(at, word));
        if (!ok) { fail(f, label); f.advance(0); return null; }
        f.advance(word.length()); return Match.empty(at, at + word.length());
    }
    public Match any(Frame f, String label) { return any(f, Label.of(label)); }
    public Match any(Frame f, Label label) {
        int at = f.position(); if (cp(at) < 0) { fail(f, label); f.advance(0); return null; }
        int length = Character.charCount(cp(at)); f.advance(length); return Match.empty(at, at + length);
    }
    public Match empty(Frame f) { return optional(f, (s, p) -> s.look(p, (ss, pp) -> ss.any(pp, L_ANY), false, L_EMPTY)); }
    public Match eof(Frame f) { return repeat(f, (s, p) -> s.any(p, L_WILDCARD_CHAR), 0, 0, null, Trivia.NONE, L_EOS); }
    public Match error(Frame f, String message) { fail(Math.max(f.c, f.m), message); return null; }
    public Match error(Frame f, Label message) { fail(Math.max(f.c, f.m), message); return null; }
    private boolean digit(int p) { return cp(p) >= '0' && cp(p) <= '9'; }
    public Match number(Frame f) {
        int at = f.position(), p = at;
        if (cp(p) == '+' || cp(p) == '-') p++; else fail(p, L_SIGN);
        int digits = p; while (digit(p)) p++;
        fail(p, L_DIGIT);
        boolean hasDigits = p > digits;
        if (!hasDigits) fail(p, L_ONE_OR_MORE);
        if (cp(p) == '.') {
            p++; int fraction = p; while (digit(p)) p++;
            fail(p, L_DIGIT);
            if (p == fraction) fail(p, L_ONE_OR_MORE);
            if (!hasDigits && p == fraction) { fail(at, L_CHAIN); fail(at, L_DOT); fail(at, L_NUMBER); f.advance(0); return null; }
        } else {
            fail(p, L_DOT);
            if (!hasDigits) { fail(digits, L_CHAIN); fail(at, L_NUMBER); f.advance(0); return null; }
        }
        int mantissa = p;
        if (cp(p) == 'e' || cp(p) == 'E') {
            p++; if (cp(p) == '+' || cp(p) == '-') p++; else fail(p, L_SIGN);
            int exponent = p; while (digit(p)) p++;
            fail(p, L_DIGIT);
            if (p == exponent) { fail(p, L_ONE_OR_MORE); p = mantissa; }
        } else { fail(p, L_E); fail(p, L_EXPONENT); }
        f.advance(p - at); return Match.empty(at, p);
    }
    public Match builtin(Frame f, String kind, int[] characters, String label) { return builtin(f, kind, characters, Label.of(label)); }
    public Match builtin(Frame f, String kind, int[] characters, Label label) {
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
            if (!alphabet(cp(p)) && cp(p) != '_') { fail(f, L_ALPHABET_UNDERSCORE); return error(f, L_IDENTIFIER); }
            p++; while (alphabet(cp(p)) || digit(p) || cp(p) == '_') p++;
            fail(p, L_ALNUM_UNDERSCORE);
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
    public Match range(Frame f, int min, int max, String excluded, String label) { return range(f, min, max, excluded, Label.of(label)); }
    public Match range(Frame f, int min, int max, String excluded, Label label) {
        int at = f.position(), ch = cp(at);
        if (ch < 0 || ch < min || ch > max || excluded != null && excluded.codePoints().anyMatch(v -> v == ch)) { Match failed = error(f, label); f.advance(0); return failed; }
        f.advance(Character.charCount(ch)); return Match.empty(at, f.position());
    }
    public Match dfa(Frame f, int state, int[] accepting, int[][] transitions, String label) { return dfa(f, state, accepting, transitions, Label.of(label)); }
    public Match dfa(Frame f, int state, int[] accepting, int[][] transitions, Label label) {
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
    public Match regex(Frame f, Pattern pattern, String label) { return regex(f, pattern, Label.of(label)); }
    public Match regex(Frame f, Pattern pattern, Label label) {
        int at = f.position(); var matcher = pattern.matcher(source).region(at, source.length());
        if (!matcher.lookingAt() || matcher.end() == at) return error(f, label);
        input.cpIndex(matcher.end()); f.advance(matcher.end() - at); return Match.empty(at, matcher.end());
    }
    public Match until(Frame f, String terminator) {
        Label label = Label.of("'" + terminator + "'");
        return repeat(f, (s, p) -> s.any(p, L_ANY), 0, Integer.MAX_VALUE,
            terminator.isEmpty() ? null : (s, p) -> s.look(p, (ss, pp) -> ss.literal(pp, terminator, true, label), false, label), Trivia.NONE);
    }
    private TokenScanner[] scannerCache = new TokenScanner[0];
    public Match external(Frame f, String id) { return external(f, id, -1); }
    public Match external(Frame f, String id, int index) {
        TokenScanner scanner = index >= 0 && index < scannerCache.length ? scannerCache[index] : null;
        if (scanner == null) {
            scanner = options.scanners().get(id);
            if (scanner == null) throw new IllegalArgumentException("Missing TokenScanner: " + id);
            if (index >= 0) { if (index >= scannerCache.length) scannerCache = Arrays.copyOf(scannerCache, index + 1); scannerCache[index] = scanner; }
        }
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
    /**
     * Each failing round registers the hints of its failing delimiter prefix as one group id
     * (the same set as registering ' ', '//', '/*', '*\/' ... individually at that position). An
     * unterminated block comment keeps the individual registration path. A bare @comment delimiter
     * (CPPComment) registers its own label again, which is a set no-op.
     */
    private void trivia(Frame f, Delimiters set) {
        Delimiter[] delimiters = set.items;
        boolean again = true;
        while (again) {
            again = false;
            int p = f.position(), at = Math.max(f.c, f.m), failed = 0;
            for (var delimiter : delimiters) {
                int next = p, terminatorEnd = -1;
                if (delimiter.isCharacters) {
                    int ch = cp(p);
                    if (ch >= 0 && delimiter.contains(ch)) next += Character.charCount(ch);
                } else if (p < input.length() && input.charAt(p) == delimiter.first && input.startsWith(p, delimiter.open)) {
                    if (delimiter.isLineComment) {
                        next += delimiter.open.length();
                        while (cp(next) >= 0 && cp(next) != '\r' && cp(next) != '\n') next += Character.charCount(cp(next));
                        terminatorEnd = cp(next) == '\r' && cp(next + 1) == '\n' ? next + 2 : cp(next) >= 0 ? next + 1 : next;
                    } else {
                        int end = source.indexOf(delimiter.close, p + delimiter.open.length());
                        if (end < 0) {
                            // Started but unterminated: hints go to the end of input, not to this position.
                            if (failed > 0) fail(at, set.prefixGroups[failed - 1]);
                            failed = -1;
                            fail(input.length(), L_BLOCK_END); fail(input.length(), L_WILDCARD_STRING);
                        } else next = end + delimiter.close.length();
                    }
                }
                if (next > p) {
                    if (failed > 0) fail(at, set.prefixGroups[failed - 1]);
                    f.advance(next - p);
                    // CPPComment's MatchOnly(LineTerminator) commits M, leaving C before the newline.
                    if (!f.matched && terminatorEnd >= 0) f.m = terminatorEnd;
                    progress(f);
                    again = true; failed = -1; break;
                }
                if (failed >= 0) failed++;
            }
            if (failed > 0) fail(at, set.prefixGroups[failed - 1]);
        }
    }
    public Match recover(Frame f, Call child, String rule, String mode, String[] patterns, boolean consumeSync,
                         String noSync, String projection, String recipe) {
        long mark = mark(); int c0 = f.c, m0 = f.m; boolean reset0 = f.reset;
        diagnostics.enterExpression();
        Match result = child.parse(this, f);
        var failure = diagnostics.leaveRule();
        f.reset = reset0;
        if (result != null) return result;
        f.c = c0; f.m = m0;
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
        var hints = Label.texts(failure.expected());
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
                var value = new Capture(occurrence, capture.site(), capture.name(), span(capture.value().start(), capture.value().end()), occurrence++, capture.completed);
                captures.add(value);
                String context = catalogs.get(capture.site()); if (context != null) catalog.computeIfAbsent(context, key -> new ArrayList<>()).add(value);
            }
            long[] counter = {0, 0}; for (var trace : match.traces()) trace(trace, -1, counter, lexical);
        }
        catalog.replaceAll((key, values) -> List.copyOf(values));
        var farthestExpected = Label.texts(diagnostics.expected());
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
        List<Value> values = recipe.field(field);
        if (values.isEmpty()) throw mapping(recipe.start(), recipe.end(), "Missing scalar capture: " + field, null);
        T value = conversion.apply(values.getFirst());
        if (value == null && !values.getFirst().nodes().isEmpty()) throw new RecoveredNodeMissing();
        return value;
    }
    public <T> Optional<T> optional(Recipe recipe, String field, Function<Value, T> conversion) {
        List<Value> values = recipe.field(field);
        return values.isEmpty() ? Optional.empty() : Optional.ofNullable(conversion.apply(values.getFirst()));
    }
    public <T> List<T> list(Recipe recipe, String field, Function<Value, T> conversion) {
        List<Value> values = recipe.field(field);
        if (values.isEmpty()) return List.of();
        var result = new ArrayList<T>(values.size());
        for (int k = 0, n = values.size(); k < n; k++) { T converted = conversion.apply(values.get(k)); if (converted != null) result.add(converted); }
        return List.copyOf(result);
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
