package org.ubnfc.java.emit;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import org.ubnfc.java.ir.GrammarIr;
import org.ubnfc.java.ir.GrammarIr.*;
import org.ubnfc.java.ir.Validation;

/**
 * 順序付き選択の候補列の左因数分解（`crates/ubnfc-rust/src/factor.rs` の Java 版、D-060）。
 *
 * <p>`X A | X B` の形（連続する候補 Seq が構造的に等しい先頭要素列を持つ）で、共有接頭辞を
 * 1 回だけ評価し、続きを宣言順に試す trie を作る。PEG の意味論では X は同じ状態で同じ結果を
 * 返す（memo が保証する決定性）ので `X A | X B ≡ X (A | B)`。
 *
 * <p>Rust 版との差（本 Java 版が追加で課す制約。理由は報告 §3）:
 * <ul>
 *   <li>共有要素は capture site を<b>高々 1 つ</b>しか持てない（Rust と同じ）。採用候補が別のときは
 *       {@code Session.retag} でその 1 件を採用候補の site / name に付け替える。</li>
 *   <li>適用は<b>診断を記録しない経路</b>（{@code Session.factor} = {@code !diag}）に限る。
 *       主診断 DAG・display frame の同値は D-023 の二段構え（失敗・末尾入力は診断モードで
 *       再解析）が保証するので、参照経路と同じ診断が必ず出る。</li>
 * </ul>
 *
 * <p>対象外（Rust と同じ）: 最長一致・予測選択、Seq でない候補、先頭要素が RuleRef / literal /
 * token 以外、execution・trivia policy・capture 規則・label が一致しない、要素や候補 Seq が
 * memo される、群内で候補除外（{@link Prediction}）の条件が一致しない。
 */
final class Factoring {
    /** 先頭要素を共有する連続候補の区間 [start, end)（候補 index は選択の宣言順）。 */
    record Group(int start, int end) {}

    private final GrammarIr ir;
    private final Map<String, Expr> expressions;
    private final Map<String, Capture> captures;
    private final SemanticShape semanticShapes;
    private final Set<String> memoized;
    private final Set<String> recoverySites;
    private final DiagnosticPlan diagnosticPlan;
    private final java.util.function.Function<Expr, String> labelText;
    private final java.util.function.Predicate<String> scopeSite;

    Factoring(GrammarIr ir, Map<String, Expr> expressions, Map<String, Rule> rules, Map<String, Capture> captures,
              SemanticShape semanticShapes, DiagnosticPlan diagnosticPlan,
              java.util.function.Function<Expr, String> labelText, java.util.function.Predicate<String> scopeSite) {
        this.ir = ir; this.expressions = expressions; this.captures = captures; this.semanticShapes = semanticShapes;
        this.diagnosticPlan = diagnosticPlan; this.labelText = labelText; this.scopeSite = scopeSite;
        this.recoverySites = ir.semantics().recovery().stream().flatMap(r -> r.applicationSites().stream())
            .collect(java.util.stream.Collectors.toSet());
        var bodies = rules.values().stream().map(Rule::body).collect(java.util.stream.Collectors.toSet());
        this.memoized = ir.plan().memo().entries().stream()
            .filter(e -> Boolean.TRUE.equals(e.cacheSuccess()) || Boolean.TRUE.equals(e.cacheFailure()))
            .map(MemoEntry::exprId).filter(id -> !bodies.contains(id))
            .collect(java.util.stream.Collectors.toSet());
    }

    /** 候補 Seq の要素列。Seq でなければ null。 */
    List<String> items(String id) {
        ExprNode node = expressions.get(id).node();
        return node.kind().equals("seq") ? node.items() : null;
    }

    private List<String> policies(String id, String at, Integer index) {
        var out = new ArrayList<String>();
        for (var p : ir.semantics().trivia().insertionPlan()) {
            Boundary b = p.boundary();
            if (b.ownerExprId().equals(id) && b.at().equals(at) && Objects.equals(b.childIndex(), index)) out.add(p.policyId());
        }
        return out;
    }

    /** 共有できる要素の性質（構造・実行 flag・trivia・capture 規則・label）。null なら共有不可。 */
    private String elementShape(String id) {
        Expr expr = expressions.get(id);
        if (!expr.execution().childMode().equals("inherit") || memoized.contains(id)) return null;
        ExprNode n = expr.node();
        String node = switch (n.kind()) {
            case "ruleRef" -> "rule " + n.symbolId();
            case "tokenRef" -> "token " + n.symbolId();
            case "literal" -> "literal " + n.caseSensitive() + " " + Boolean.TRUE.equals(n.wordBoundary()) + " " + n.value();
            default -> null;
        };
        if (node == null) return null;
        // 採用候補の site へ付け替える（Session.retag）ので site は 1 つまで。
        if (expr.captureSiteIds().size() > 1 || recoverySites.contains(id)) return null;
        var shape = new StringBuilder(node).append('\u0000').append(expr.execution().entryFlag())
            .append('\u0000').append(expr.execution().transaction());
        for (String at : List.of("entry", "helperEntry", "exit", "helperExit"))
            shape.append('\u0000').append(policies(id, at, null));
        for (String site : expr.captureSiteIds())
            shape.append('\u0000').append(captures.get(site).spanRule()).append('\u0000').append(captures.get(site).skip())
                .append('\u0000').append(scopeSite.test(site));
        shape.append('\u0000').append(labelText.apply(expr))
            .append('\u0000').append(diagnosticPlan.tokenLabel(id))
            .append('\u0000').append(diagnosticPlan.expression(id))
            .append('\u0000').append(semanticShapes.of(id).kind()).append('\u0000').append(semanticShapes.of(id).many());
        return shape.toString();
    }

    /** 候補 Seq の群の共通性質（execution・入口 trivia）。null なら群にできない。 */
    private String seqShape(String id) {
        Expr expr = expressions.get(id);
        if (!expr.execution().childMode().equals("inherit") || memoized.contains(id)) return null;
        if (!expr.node().kind().equals("seq")) return null;
        return expr.execution().entryFlag() + "\u0000" + expr.execution().transaction()
            + "\u0000" + policies(id, "entry", null) + "\u0000" + policies(id, "helperEntry", null);
    }

    /** 候補 a と b が index depth の要素を共有できるか（それ以前は検査済み）。 */
    boolean share(String a, String b, int depth) {
        List<String> ia = items(a), ib = items(b);
        if (ia == null || ib == null || ia.size() <= depth || ib.size() <= depth) return false;
        String sa = elementShape(ia.get(depth)), sb = elementShape(ib.get(depth));
        return sa != null && sa.equals(sb) && policies(a, "afterChild", depth).equals(policies(b, "afterChild", depth));
    }

    /**
     * 順序付き選択 `alternatives` の分解を計画する。群が無ければ空。
     * `guards` は候補ごとの候補除外条件のテキスト（生成コードと同じ正規化）。
     */
    List<Group> plan(List<String> alternatives, List<String> guards) {
        var groups = new ArrayList<Group>();
        int i = 0;
        while (i < alternatives.size()) {
            int j = i + 1;
            if (seqShape(alternatives.get(i)) != null) {
                while (j < alternatives.size()
                    && Objects.equals(seqShape(alternatives.get(j)), seqShape(alternatives.get(i)))
                    && share(alternatives.get(i), alternatives.get(j), 0)
                    && Objects.equals(guards.get(i), guards.get(j))) j++;
            }
            if (j - i >= 2) groups.add(new Group(i, j));
            i = j;
        }
        return groups;
    }

    /** 群 [start,end) を depth まで共有済みとして、次の段の下位群に分ける（Rust `level` と同じ）。 */
    List<Group> level(List<String> alternatives, Group group, int depth) {
        var out = new ArrayList<Group>();
        int i = group.start();
        while (i < group.end()) {
            int j = i + 1;
            boolean complete = items(alternatives.get(i)).size() <= depth;
            if (!complete) while (j < group.end() && share(alternatives.get(i), alternatives.get(j), depth)) j++;
            out.add(new Group(i, j));
            if (complete) break; // 接頭辞で完結する候補より後ろは参照経路でも到達しない。
            i = j;
        }
        return out;
    }

}
