package org.unlaxer.ubnfc.java.emit;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import org.unlaxer.ubnfc.java.ir.GrammarIr;
import org.unlaxer.ubnfc.java.ir.GrammarIr.*;
import org.unlaxer.ubnfc.java.emit.Prediction.Ranges;

/** Java の意味論から命令・遷移の再検証を行い、認識専用 VM の閉領域を選ぶ。 */
final class Ll1 {
    private final List<Ll1State> table;
    private final Map<String, Integer> roots = new LinkedHashMap<>();
    private final Set<Integer> emitted = new HashSet<>();
    private final Scanning scanning;
    private final String parser;
    private List<First> first;

    // depth は入力を進める前に入り得る規則数。選択で省く失敗候補の深さも含む。
    private record First(Ranges chars, boolean nullable, int depth) {
        First then(First next) { return new First(chars.union(next.chars), next.nullable, Math.max(depth, next.depth)); }
        First or(First next) { return new First(chars.union(next.chars), nullable || next.nullable, Math.max(depth, next.depth)); }
        boolean same(First next) { return depth == next.depth && nullable == next.nullable && chars.equalsRanges(next.chars); }
    }

    Ll1(GrammarIr ir, Map<String, Expr> expressions, Map<String, Rule> rules, Map<String, Token> tokens,
        Predicate<String> memoized, Scanning scanning, String parser) {
        this.table = ir.plan().ll1();
        this.scanning = scanning;
        this.parser = parser;
        if (table.isEmpty() || !ir.semantics().scope().effects().isEmpty() || !ir.semantics().recovery().isEmpty()) return;
        var ids = new HashSet<String>();
        for (Ll1State state : table) if (!ids.add(state.exprId())) return;
        var blocked = new HashSet<String>();
        ir.semantics().normalizations().forEach(n -> blocked.add(n.ruleId()));
        rules.values().stream().filter(r -> r.operator() != null).forEach(r -> blocked.add(r.symbolId()));
        var trivia = new HashSet<String>();
        ir.semantics().trivia().insertionPlan().forEach(t -> trivia.add(t.boundary().ownerExprId()));
        for (Ll1State state : table) {
            Expr expr = expressions.get(state.exprId());
            if (expr == null || blocked.contains(expr.ruleId()) || trivia.contains(expr.id())
                || !"inherit".equals(expr.execution().childMode())
                || !matches(state.operation(), expr.node(), rules, tokens)) return;
        }
        if (!verifyFirst()) return;
        var choices = new HashMap<String, ChoicePlan>();
        ir.plan().choices().forEach(plan -> choices.putIfAbsent(plan.exprId(), plan));
        var starts = new HashMap<String, MayStart>();
        ir.analysis().expressions().forEach(expr -> starts.put(expr.exprId(), expr.mayStart()));
        for (Ll1State state : table) {
            if (!state.operation().kind().equals("choice")) continue;
            ChoicePlan plan = choices.get(state.exprId());
            ExprNode node = expressions.get(state.exprId()).node();
            if (plan != null && !plan.fallback().equals("fullChoice")
                && (plan.strategy().equals("predict") || node.selection().equals("predictive"))) {
                for (var transition : state.operation().transitions()) {
                    MayStart start = starts.get(target(transition.target()));
                    if (start == null) return;
                    if (JavaEmitter.usesFirstFilter(start)) {
                        Ranges eligible = ranges(start.ranges());
                        if (eligible == null || !covers(eligible, first.get(transition.target()).chars())) return;
                    }
                }
            }
        }
        for (int i = 0; i < table.size(); i++) {
            Ll1State state = table.get(i);
            if (!Boolean.TRUE.equals(state.root())) continue;
            var region = closure(i);
            if (region.size() < 3 || region.stream().map(table::get).map(Ll1State::operation)
                .noneMatch(op -> op.kind().equals("choice") || op.kind().equals("repeat"))) continue;
            if (region.stream().anyMatch(row -> memoized.test(table.get(row).exprId()))) continue;
            roots.put(state.exprId(), i);
            emitted.addAll(region);
        }
    }

    private String target(Integer index) {
        return index != null && index >= 0 && index < table.size() ? table.get(index).exprId() : null;
    }

    private boolean matches(Ll1Operation op, ExprNode node, Map<String, Rule> rules, Map<String, Token> tokens) {
        return switch (node.kind()) {
            case "seq" -> op.kind().equals("sequence") && node.items().equals(op.targets().stream().map(this::target).toList());
            case "group" -> op.kind().equals("sequence") && op.targets().size() == 1
                && node.child().equals(target(op.targets().getFirst()));
            case "choice" -> !"longest".equals(node.selection()) && op.kind().equals("choice")
                && node.alternatives().equals(op.transitions().stream().map(t -> target(t.target())).toList());
            case "optional" -> op.kind().equals("optional") && node.child().equals(target(op.target()));
            case "repeat" -> Integer.valueOf(0).equals(node.max()) && Integer.valueOf(0).equals(node.min())
                ? op.kind().equals("empty")
                : op.kind().equals("repeat") && node.terminator() == null && !Integer.valueOf(0).equals(node.max())
                    && node.child().equals(target(op.target())) && node.min().equals(op.min()) && Objects.equals(node.max(), op.max());
            case "ruleRef" -> op.kind().equals("call") && rules.get(node.symbolId()).body().equals(target(op.target()));
            case "literal" -> op.kind().equals("literal") && Boolean.TRUE.equals(node.caseSensitive())
                && node.value().equals(op.value()) && !op.value().isEmpty()
                && op.value().codePoints().noneMatch(cp -> cp >= 0xd800 && cp <= 0xdfff)
                && Boolean.TRUE.equals(node.wordBoundary()) == Boolean.TRUE.equals(op.wordBoundary());
            case "eof" -> op.kind().equals("eof");
            case "any" -> matchesClass(op, scalars(0, Character.MAX_CODE_POINT));
            case "tokenRef" -> matchesToken(op, tokens.get(node.symbolId()).spec());
            default -> false;
        };
    }

    private static Ranges scalars(int lower, int upper) {
        return Ranges.of(Math.max(0, lower), Math.min(0xd7ff, upper))
            .union(Ranges.of(Math.max(0xe000, lower), Math.min(Character.MAX_CODE_POINT, upper)));
    }

    private static Ranges points(java.util.stream.IntStream points) {
        return points.mapToObj(cp -> scalars(cp, cp)).reduce(new Ranges(), Ranges::union);
    }

    private static boolean matchesToken(Ll1Operation op, TokenSpec spec) {
        Ranges alphabet = Ranges.of('A', 'Z').union(Ranges.of('a', 'z'));
        Ranges digits = Ranges.of('0', '9');
        return switch (spec.kind()) {
            case "eof" -> op.kind().equals("eof");
            case "any" -> matchesClass(op, scalars(0, Character.MAX_CODE_POINT));
            case "charRange" -> matchesClass(op, scalars(spec.min(), spec.max()));
            case "negation" -> {
                Ranges allowed = new Ranges();
                int start = 0;
                for (int cp : spec.excluded().codePoints().distinct().sorted().toArray()) {
                    allowed = allowed.union(scalars(start, cp - 1));
                    start = cp + 1;
                }
                yield matchesClass(op, allowed.union(scalars(start, Character.MAX_CODE_POINT)));
            }
            case "builtin" -> switch (spec.which()) {
                case "EndOfSource" -> op.kind().equals("eof");
                case "Digit" -> matchesClass(op, digits);
                case "Alphabet" -> matchesClass(op, alphabet);
                case "AlphabetUnderScore" -> matchesClass(op, alphabet.union(Ranges.single('_')));
                case "AlphabetNumericUnderScore" -> matchesClass(op, alphabet.union(digits).union(Ranges.single('_')));
                case "Space" -> matchesClass(op, Ranges.of(9, 13).union(Ranges.single(32)));
                case "CodePointSet" -> matchesClass(op, points(spec.codePoints().stream().mapToInt(Integer::intValue)));
                default -> false;
            };
            default -> false;
        };
    }

    private static boolean matchesClass(Ll1Operation op, Ranges expected) {
        if (!op.kind().equals("class")) return false;
        Ranges actual = ranges(op.ranges());
        return actual != null && expected.equalsRanges(actual);
    }

    /** IR が要求する昇順かつ重複のない Unicode scalar 区間。 */
    private static Ranges ranges(List<CharRange> input) {
        var result = new Ranges();
        int previous = -1;
        for (CharRange r : input) {
            if (r.start() <= previous || r.end() < r.start() || r.end() > Character.MAX_CODE_POINT
                || r.start() <= 0xdfff && r.end() >= 0xd800) return null;
            result = result.union(Ranges.of(r.start(), r.end()));
            previous = r.end();
        }
        return result;
    }

    private static List<Integer> targets(Ll1Operation op) {
        return switch (op.kind()) {
            case "sequence" -> op.targets();
            case "choice" -> op.transitions().stream().map(Ll1Transition::target).toList();
            case "call", "repeat", "optional" -> List.of(op.target());
            default -> List.of();
        };
    }

    private Set<Integer> closure(int root) {
        var seen = new HashSet<Integer>();
        var pending = new ArrayDeque<Integer>();
        pending.push(root);
        while (!pending.isEmpty()) {
            int index = pending.pop();
            if (seen.add(index)) pending.addAll(targets(table.get(index).operation()));
        }
        return seen;
    }

    private boolean verifyFirst() {
        First empty = new First(new Ranges(), false, 0);
        first = new ArrayList<>(Collections.nCopies(table.size(), empty));
        // 右再帰は先頭の非 nullable な項で止まる。無消費で再帰する経路の深さは 33 で飽和する。
        for (int round = 0; round <= table.size() + 33; round++) {
            boolean changed = false;
            for (int i = 0; i < table.size(); i++) {
                First next = first(table.get(i).operation());
                if (!next.same(first.get(i))) { first.set(i, next); changed = true; }
            }
            if (!changed) return table.stream().allMatch(state -> verifyControl(state.operation()));
        }
        return false;
    }

    private First first(Ll1Operation op) {
        return switch (op.kind()) {
            case "class" -> new First(ranges(op.ranges()), false, 0);
            case "literal" -> new First(Ranges.single(op.value().codePointAt(0)), false, 0);
            case "eof", "empty" -> new First(new Ranges(), true, 0);
            case "optional", "repeat", "call" -> {
                First child = first.get(op.target());
                yield new First(child.chars(), child.nullable() || !op.kind().equals("call")
                    && (op.kind().equals("optional") || op.min() == 0),
                    Math.min(33, child.depth() + (op.kind().equals("call") ? 1 : 0)));
            }
            case "choice" -> {
                First result = new First(new Ranges(), false, 0);
                for (var transition : op.transitions()) result = result.or(first.get(transition.target()));
                yield result;
            }
            case "sequence" -> {
                First result = new First(new Ranges(), true, 0);
                for (int target : op.targets()) {
                    if (!result.nullable()) break;
                    result = result.then(first.get(target));
                }
                yield result;
            }
            default -> throw new IllegalStateException(op.kind());
        };
    }

    private boolean verifyControl(Ll1Operation op) {
        if (op.kind().equals("repeat")) return !first.get(op.target()).nullable();
        if (!op.kind().equals("choice")) return true;
        var seen = new Ranges();
        for (var transition : op.transitions()) {
            var actual = first.get(transition.target());
            var declared = ranges(transition.ranges());
            if (actual.nullable() || declared == null || !covers(declared, actual.chars())) return false;
            for (int[] a : seen.items())
                for (int[] b : declared.items())
                    if (a[0] <= b[1] && b[0] <= a[1]) return false;
            seen = seen.union(declared);
        }
        return true;
    }

    private static boolean covers(Ranges upper, Ranges lower) {
        return upper.union(lower).equalsRanges(upper);
    }

    boolean enabled() { return !roots.isEmpty(); }

    String fastPath(String id) {
        Integer root = roots.get(id);
        if (root == null) return "";
        return "if (s.ll1 && !f.invert && f.c == f.m) { Match r = s.recognizeLl1(LL1, " + root
            + ", f);" + (scanning.resetsToFalse(id, true) ? " if (r != null) f.reset = false;" : "")
            + " if (r != null) return r; }\n        ";
    }

    String source() {
        if (!enabled()) return "";
        var out = new StringBuilder("    private static final Ll1Machine.Op[] LL1 = ll1Table();\n"
            + "    private static Ll1Machine.Op[] ll1Table() {\n"
            + "        var ops = new Ll1Machine.Op[" + table.size() + "];\n");
        var sorted = emitted.stream().sorted().toList();
        final int chunk = 128;
        for (int from = 0; from < sorted.size(); from += chunk)
            out.append("        ll1Table").append(from).append("(ops);\n");
        out.append("        return ops;\n    }\n");
        for (int from = 0; from < sorted.size(); from += chunk) {
            out.append("    private static void ll1Table").append(from).append("(Ll1Machine.Op[] ops) {\n");
            for (int index : sorted.subList(from, Math.min(from + chunk, sorted.size())))
                out.append("        ops[").append(index).append("] = ").append(instruction(index)).append(";\n");
            out.append("    }\n");
        }
        return out.toString();
    }

    private String instruction(int index) {
        Ll1State state = table.get(index);
        Ll1Operation op = state.operation();
        String scan = scanning.isRoot(state.exprId()) ? parser + "::" + scanning.scan(state.exprId()) : "null";
        String args = switch (op.kind()) {
            case "sequence" -> ", " + ints(op.targets());
            case "choice" -> ", " + first.get(index).depth() + ", " + op.transitions().stream()
                .map(t -> "new Ll1Machine.Edge(" + t.target() + ", " + rangesSource(t.ranges()) + ")")
                .collect(Collectors.joining(", "));
            case "optional", "call" -> ", " + op.target();
            case "repeat" -> ", " + op.target() + ", " + op.min() + ", " + (op.max() == null ? "Integer.MAX_VALUE" : op.max());
            case "literal" -> ", " + JavaEmitter.q(op.value()) + ", " + op.wordBoundary();
            case "class" -> ", " + rangesSource(op.ranges());
            case "eof", "empty" -> "";
            default -> throw new IllegalStateException(op.kind());
        };
        String kind = op.kind().equals("class") ? "characterClass" : op.kind();
        return "Ll1Machine." + kind + "(" + scan + args + ")";
    }

    private static String ints(List<Integer> values) {
        return values.stream().map(Object::toString).collect(Collectors.joining(",", "new int[]{", "}"));
    }

    private static String rangesSource(List<CharRange> ranges) {
        var values = new ArrayList<Integer>();
        for (CharRange range : ranges) { values.add(range.start()); values.add(range.end()); }
        return ints(values);
    }
}
