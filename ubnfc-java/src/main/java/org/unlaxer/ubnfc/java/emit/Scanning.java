package org.unlaxer.ubnfc.java.emit;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.function.BiFunction;
import java.util.stream.Collectors;
import org.unlaxer.ubnfc.java.ir.GrammarIr;
import org.unlaxer.ubnfc.java.ir.GrammarIr.*;
import org.unlaxer.ubnfc.java.ir.Validation;

/**
 * D-080（D-072 の Java 版）— {@code plan.scans} の領域を {@code Session} を通さない走査関数へ落とす。
 * Rust の {@code crates/ubnfc-rust/src/scan.rs} と 1 対 1 に対応する。
 *
 * <p>front が「領域は終端だけでできていて観測を 1 つも残さない」ことを証明している
 * （{@code crates/ubnfc-front/src/analysis/scan.rs}）。ここで生成するのは
 * {@code private static long scanN(char[] t, int p)} だけの純関数で、Frame・Match・memo・診断・
 * scope・深さ計数を一切触らない。返り値は {@code (終了位置 << 1) | 値なし}、失敗は {@code -1}。
 *
 * <p>各節点の判定は<b>Java の参照経路（{@code Session}）の意味そのもの</b>を写す（Rust の走査関数ではなく）:
 * 選択は宣言順、反復は「成功したら数えてから 0 進行なら止める」、単一 code point の終端は
 * {@code Session.range} / {@code builtin} / {@code any} と同じ式、literal は {@code Input.startsWith}。
 * 値なし bit は {@code Session.combine} / {@code repeat} の値列（null か空列か）を再現する。
 */
final class Scanning {
    private final GrammarIr ir;
    private final Map<String, Expr> expressions;
    private final Map<String, Rule> rules;
    private final Map<String, Token> tokens;
    private final Map<String, GrammarIr.Recipe> recipes;
    private final Map<String, Integer> exprIds;
    private final BiFunction<String, String, String> constant;
    /** 走査関数を持つ根（式 id）。 */
    final Set<String> roots = new LinkedHashSet<>();
    /** 走査関数を生成する全式（根と、その下の領域）。 */
    private final Set<String> members = new LinkedHashSet<>();
    /** 反復ごとの一括走査表。 */
    private final Map<String, ScanClass> classes = new LinkedHashMap<>();

    Scanning(GrammarIr ir, Map<String, Expr> expressions, Map<String, Rule> rules, Map<String, Token> tokens,
             Map<String, GrammarIr.Recipe> recipes, Map<String, Integer> exprIds, BiFunction<String, String, String> constant) {
        this.ir = ir; this.expressions = expressions; this.rules = rules; this.tokens = tokens; this.recipes = recipes;
        this.exprIds = exprIds; this.constant = constant;
        for (ScanPlan plan : ir.plan().scans()) {
            if (!expressions.containsKey(plan.exprId())) continue;
            if (Boolean.TRUE.equals(plan.root())) roots.add(plan.exprId());
            if (plan.class_() != null) classes.put(plan.exprId(), plan.class_());
        }
        // 生成できない節点・再帰・値を作る規則を含む根は落とす（front の証明より backend の
        // 実装範囲が狭い場合の安全側。Rust の generate と同じ作法）。
        roots.removeIf(root -> !supported(root));
        for (String root : roots) members.addAll(region(root));
    }

    boolean isRoot(String id) { return roots.contains(id); }

    /** 根から規則参照をたどって到達する式（根を含む）。 */
    private Set<String> region(String root) {
        var seen = new LinkedHashSet<String>();
        var pending = new ArrayList<String>(List.of(root));
        while (!pending.isEmpty()) {
            String id = pending.removeLast();
            if (!seen.add(id)) continue;
            pending.addAll(next(id));
        }
        return seen;
    }
    private List<String> next(String id) {
        ExprNode node = expressions.get(id).node();
        if (node.kind().equals("ruleRef")) return List.of(rules.get(node.symbolId()).body());
        return Validation.children(node);
    }
    private boolean supported(String root) {
        var region = region(root);
        for (String id : region) if (!local(id)) return false;
        return acyclic(root, new HashSet<>(), new HashSet<>());
    }
    /** 走査関数は互いを呼ぶので、領域に閉路があれば（front の条件 4 が崩れた IR）使わない。 */
    private boolean acyclic(String id, Set<String> path, Set<String> done) {
        if (done.contains(id)) return true;
        if (!path.add(id)) return false;
        for (String child : next(id)) if (!acyclic(child, path, done)) return false;
        path.remove(id); done.add(id); return true;
    }
    /** 子を見ない局所条件。生成できる節点か。 */
    private boolean local(String id) {
        Expr expr = expressions.get(id);
        ExprNode n = expr.node();
        if (!"inherit".equals(expr.execution().childMode())) return false;
        return switch (n.kind()) {
            case "seq", "optional" -> true;
            case "group" -> true;
            case "choice" -> !"longest".equals(n.selection());
            case "repeat" -> n.terminator() == null;
            case "literal" -> Boolean.TRUE.equals(n.caseSensitive()) && !Boolean.TRUE.equals(n.wordBoundary()) && n.value() != null;
            case "any" -> true;
            case "tokenRef" -> leafCondition(tokens.get(n.symbolId()).spec()) != null;
            case "ruleRef" -> {
                Rule rule = rules.get(n.symbolId());
                GrammarIr.Recipe recipe = recipes.get(rule.symbolId());
                // 値を作る規則（mapping・skip）は結果の Match が変わるので畳まない。
                yield !Boolean.TRUE.equals(rule.skip()) && rule.mappingTypeId() == null
                    && (recipe == null || Set.of("transparent", "text").contains(recipe.kind()));
            }
            default -> false;
        };
    }

    /** 単一 code point の終端が真になる条件（{@code c} は {@code Session.cpAt} の値。{@code Session} の判定と同じ式）。 */
    private String leafCondition(TokenSpec spec) {
        return switch (spec.kind()) {
            case "charRange" -> "c >= " + spec.min() + " && c <= " + spec.max();
            case "negation" -> {
                var excluded = spec.excluded().codePoints().distinct().sorted().mapToObj(v -> "c != " + v).toList();
                yield excluded.isEmpty() ? "true" : String.join(" && ", excluded);
            }
            case "any" -> "true";
            case "builtin" -> switch (spec.which()) {
                case "Digit" -> "c >= 48 && c <= 57";
                case "Alphabet" -> "c >= 65 && c <= 90 || c >= 97 && c <= 122";
                case "AlphabetUnderScore" -> "c >= 65 && c <= 90 || c >= 97 && c <= 122 || c == 95";
                case "AlphabetNumericUnderScore" -> "c >= 65 && c <= 90 || c >= 97 && c <= 122 || c >= 48 && c <= 57 || c == 95";
                case "Space" -> "c == 32 || c >= 9 && c <= 13";
                // Session.builtin と同じ配列（同じ定数 field）を同じ二分探索で引く。
                case "CodePointSet" -> "java.util.Arrays.binarySearch(" + constant.apply("int[]", "new int[]{"
                    + spec.codePoints().stream().map(Object::toString).collect(Collectors.joining(",")) + "}") + ", c) >= 0";
                default -> null;
            };
            default -> null;
        };
    }

    private String scan(String id) { return "scan" + exprIds.get(id); }

    /** {@code e{i}C} の先頭に置く走査経路（根でなければ空）。{@code resetFalse} は参照経路が本体の評価後に残す {@code f.reset}。 */
    String fastPath(String id) {
        if (!roots.contains(id)) return "";
        String reset = resetsToFalse(id) ? " f.reset = false;" : "";
        return "if (s.scan && !f.invert && f.c == f.m) { Match r = s.scanned(f, " + scan(id) + "(s.chars, f.c));" + reset + " return r; }\n        ";
    }
    /**
     * 参照経路で {@code e{i}C}（本体）を評価した後の {@code f.reset}。{@code setFalse} の式は e-method の入口で
     * {@code f.reset = false} にし、連接・選択・反復は入口の値へ戻す。規則参照と delegate の group は
     * 呼び先の e-method をそのまま通すので、呼び先の {@code setFalse} が外へ残る。
     */
    private boolean resetsToFalse(String id) {
        ExprNode n = expressions.get(id).node();
        String target = switch (n.kind()) {
            case "ruleRef" -> rules.get(n.symbolId()).body();
            case "group" -> expressions.get(id).execution().transaction().equals("delegate") ? n.child() : null;
            default -> null;
        };
        if (target == null) return false;
        return expressions.get(target).execution().entryFlag().equals("setFalse") || resetsToFalse(target);
    }

    /** 領域内の全式ぶんの走査関数（式 id 順に安定）。 */
    String functions() {
        if (members.isEmpty()) return "";
        var ordered = new TreeMap<Integer, String>();
        for (String id : members) ordered.put(exprIds.get(id), id);
        var out = new StringBuilder();
        for (String id : ordered.values())
            out.append("    /** D-080: 観測を残さない終端領域の走査。{@code (終了位置 << 1) | 値なし}、失敗は -1。 */\n")
                .append("    private static long ").append(scan(id)).append("(char[] t, int p) {\n        ")
                .append(body(id).replace("\n", "\n        ")).append("\n    }\n");
        return out.toString();
    }

    private String body(String id) {
        Expr expr = expressions.get(id);
        ExprNode n = expr.node();
        return switch (n.kind()) {
            case "tokenRef" -> leaf(leafCondition(tokens.get(n.symbolId()).spec()));
            case "any" -> leaf("true");
            case "literal" -> literal(n.value());
            case "seq" -> {
                if (n.items().isEmpty()) yield "return (long) p << 1;";
                var s = new StringBuilder("long r, a = 0;\n");
                for (String item : n.items())
                    s.append("r = ").append(scan(item)).append("(t, p); if (r < 0) return -1L; a |= r; p = (int) (r >>> 1);\n");
                yield s.append("return (long) p << 1 | a & 1L;").toString();
            }
            case "group" -> "return " + scan(n.child()) + "(t, p);";
            case "ruleRef" -> "return " + scan(rules.get(n.symbolId()).body()) + "(t, p);";
            case "choice" -> {
                // Session.choice は宣言順に試す（predict の eligible / 候補除外は失敗が確定した候補を飛ばすだけ）。
                var s = new StringBuilder("long r;\n");
                for (String alternative : n.alternatives())
                    s.append("r = ").append(scan(alternative)).append("(t, p); if (r >= 0) return r;\n");
                yield s.append("return -1L;").toString();
            }
            // Session.optional は repeat(0, 1)。成功すれば子の値、失敗なら 0 進行の「値なし」。
            case "optional" -> "long r = " + scan(n.child()) + "(t, p);\nreturn r >= 0 ? r : (long) p << 1 | 1L;";
            case "repeat" -> repeat(id, n);
            default -> throw new IllegalStateException(n.kind());
        };
    }
    private static String leaf(String condition) {
        return "if (p >= t.length) return -1L;\nint c = Session.cpAt(t, p);\nreturn " + condition + " ? (long) (p + Character.charCount(c)) << 1 : -1L;";
    }
    /** {@code Session.literal}（case sensitive・語境界なし）。空の literal は参照経路でも失敗する。 */
    private static String literal(String value) {
        if (value.isEmpty()) return "return -1L;";
        int length = value.length();
        if (length > 8) return "return Session.startsWith(t, p, " + JavaEmitter.q(value) + ") ? (long) (p + " + length + ") << 1 : -1L;";
        var s = new StringBuilder("if (p > t.length - " + length);
        for (int i = 0; i < length; i++) s.append(" || t[p").append(i == 0 ? "" : " + " + i).append("] != ").append((int) value.charAt(i));
        return s.append(") return -1L;\nreturn (long) (p + ").append(length).append(") << 1;").toString();
    }
    /**
     * {@code Session.repeat}（終端子なし・trivia なし）: 子が成功したら数え、進まなかったか上限に達したら止める。
     * 上限なしで一括走査表があるときは、表に載る文字（1 文字で候補が確定し 1 code point だけ消費する）を
     * 本体を呼ばずに食べる。表の周回は 1 code point の終端の成功なので値を持つ（値なし bit は立たない）。
     */
    private String repeat(String id, ExprNode n) {
        int min = n.min() == null ? 0 : n.min();
        String max = n.max() == null ? "Integer.MAX_VALUE" : n.max().toString();
        ScanClass table = n.max() == null ? classes.get(id) : null;
        var s = new StringBuilder("int count = 0; long a = 0, r;\n");
        String child = scan(n.child());
        if (table != null) {
            long low = table.asciiLow().toBigInteger().longValue(), high = table.asciiHigh().toBigInteger().longValue();
            String ranges = table.ranges().isEmpty() ? "false" : table.ranges().stream()
                .map(r -> "c >= " + r.start() + " && c <= " + r.end()).collect(Collectors.joining(" || "));
            String bulk = "while (p < t.length) {\n"
                + "    char ch = t[p];\n"
                + "    if (ch < 128) { if (((ch < 64 ? 0x" + Long.toHexString(low) + "L >>> ch : 0x" + Long.toHexString(high) + "L >>> (ch - 64)) & 1L) == 0) break; p++; }\n"
                + "    else { int c = Session.cpAt(t, p); if (!(" + ranges + ")) break; p += Character.charCount(c); }\n"
                + "    count++;\n"
                + "}\n";
            if (Boolean.TRUE.equals(table.complete())) s.append(bulk);
            else s.append("while (true) {\n").append(bulk.indent(4))
                .append("    int before = p;\n")
                .append("    r = ").append(child).append("(t, p); if (r < 0) break;\n")
                .append("    a |= r; p = (int) (r >>> 1); count++;\n")
                .append("    if (before == p) break;\n}\n");
        } else {
            s.append("while (true) {\n")
                .append("    int before = p;\n")
                .append("    r = ").append(child).append("(t, p); if (r < 0) break;\n")
                .append("    a |= r; p = (int) (r >>> 1); count++;\n")
                .append("    if (before == p || count >= ").append(max).append(") break;\n}\n");
        }
        var bounds = new ArrayList<String>();
        if (min > 0) bounds.add("count < " + min);
        if (n.max() != null) bounds.add("count > " + max);
        if (!bounds.isEmpty()) s.append("if (").append(String.join(" || ", bounds)).append(") return -1L;\n");
        // Session.repeat の値列: 0 周なら空列（値なし）、そうでなければ周回の結合（どれかが値なしなら値なし）。
        return s.append("return (long) p << 1 | (count == 0 ? 1L : a & 1L);").toString();
    }
}
