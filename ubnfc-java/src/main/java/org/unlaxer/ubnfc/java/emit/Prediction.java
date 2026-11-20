package org.unlaxer.ubnfc.java.emit;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.unlaxer.ubnfc.java.ir.GrammarIr;
import org.unlaxer.ubnfc.java.ir.GrammarIr.*;

/**
 * 通常の {@code |} への保守的な候補除外（設計 v1 §3.4）。Rust backend の {@code predict.rs} と同じ
 * 「trivia 越しの先頭 code point 集合」を求め、呼出し位置の文字が集合に無い候補は最初の
 * significant token で失敗すると証明する。
 *
 * <p>Java 版は再生を関数展開ではなく静的な表（{@link Guard}）で行う。除外の実行時条件を
 * 「呼出し位置で trivia が 1 文字も進まない」場合に限る（区切りの先頭文字も allowed に入れて
 * 該当すれば本物の候補へ戻す）ので、候補が記録したはずの失敗は全て同じ位置 P に並ぶ。
 * その位置での観測は
 * <ul>
 *   <li>expected 集合 = 記録順・重複除去済みの label 列（{@code Global.fail} / frame 区間の規則と同じ）</li>
 *   <li>rule 経路 = 最大深さの記録のうち最初のものの rule stack（{@code length > stackLength} でのみ更新）</li>
 *   <li>consumed / matched = P（{@code f.reset || f.c == f.m} を実行時条件にしているため）</li>
 * </ul>
 * だけで決まるので、label 列・最深 suffix・最大 rule 深さを生成時に畳み込める。
 *
 * <p>再生と本物の差は memo（lookup / store）と rule ごとの局所 frame だけで、どちらも観測量を
 * 変えない（memo 有無の同値は既存の性質、frame は「最内 frame へ記録して親へ吸収する」規則が
 * 「囲む frame へ直接記録する」と同値）。深さ上限に近い状態では再生せず本物の候補を呼ぶ。
 */
final class Prediction {
    /** 再生関数 1 つが展開する node 数の上限（Rust backend と同じ）。 */
    private static final int INLINE_MAX = 192;
    /** trivia group label の位置を表す marker。 */
    static final String TRIVIA_LABEL = "\u0000trivia";

    /** 昇順・互いに素な閉区間の集合。 */
    static final class Ranges {
        private final List<int[]> items = new ArrayList<>();
        static Ranges of(int lo, int hi) { var r = new Ranges(); if (lo <= hi) r.items.add(new int[] {lo, hi}); return r; }
        static Ranges single(int c) { return of(c, c); }
        boolean isEmpty() { return items.isEmpty(); }
        List<int[]> items() { return items; }
        Ranges union(Ranges other) {
            var all = new ArrayList<int[]>(items.size() + other.items.size());
            all.addAll(items); all.addAll(other.items);
            all.sort((a, b) -> a[0] != b[0] ? Integer.compare(a[0], b[0]) : Integer.compare(a[1], b[1]));
            var out = new Ranges();
            for (int[] range : all) {
                if (!out.items.isEmpty()) {
                    int[] last = out.items.get(out.items.size() - 1);
                    if (range[0] <= (last[1] == Integer.MAX_VALUE ? last[1] : last[1] + 1)) {
                        last[1] = Math.max(last[1], range[1]);
                        continue;
                    }
                }
                out.items.add(new int[] {range[0], range[1]});
            }
            return out;
        }
        boolean equalsRanges(Ranges other) {
            if (items.size() != other.items.size()) return false;
            for (int i = 0; i < items.size(); i++)
                if (items.get(i)[0] != other.items.get(i)[0] || items.get(i)[1] != other.items.get(i)[1]) return false;
            return true;
        }
    }

    /** trivia 越しに「成功し得る先頭 code point」。policy は skipped を読む前に必ず通る trivia。 */
    private static final class Known {
        boolean nullable;
        Ranges direct = new Ranges(), skipped = new Ranges();
        Integer policy;
        Known copy() { var k = new Known(); k.nullable = nullable; k.direct = direct; k.skipped = skipped; k.policy = policy; return k; }
        boolean same(Known other) {
            return other != null && nullable == other.nullable && java.util.Objects.equals(policy, other.policy)
                && direct.equalsRanges(other.direct) && skipped.equalsRanges(other.skipped);
        }
    }

    /** 除外候補の静的な再生計画。 */
    record Guard(Ranges allowed, List<String> labels, String triviaPolicyId, List<Integer> suffix, int maxRuleDepth) {}

    private final GrammarIr ir;
    private final Map<String, Expr> expressions;
    private final Map<String, Rule> rules;
    private final Map<String, Token> tokens;
    private final Map<String, Integer> ruleIds;
    private final DiagnosticPlan diagnosticPlan;
    private final java.util.function.Function<Expr, String> literalLabel;
    /** policy id -> 正規化 index（区切り集合が等しい policy を同一視）。 */
    private final Map<String, Integer> canonicalPolicy = new HashMap<>();
    private final List<TriviaPolicy> policyByIndex = new ArrayList<>();
    private final List<String> order = new ArrayList<>();
    private final Map<String, Known> starts = new HashMap<>();
    private final Set<String> unknown = new HashSet<>();
    private final Set<String> recoverySites = new HashSet<>();
    private final Set<String> entryEffectRules = new HashSet<>();
    private final Set<String> recoveryRules = new HashSet<>();
    private final Map<String, Map<String, List<String>>> entryPolicies = new HashMap<>();
    /** 文法中の全 trivia 区切りの先頭 code point（保守的に allowed へ足す）。 */
    private final Ranges allTriviaFirst;
    private final String anyPolicyId;

    Prediction(GrammarIr ir, Map<String, Expr> expressions, Map<String, Rule> rules, Map<String, Token> tokens,
               Map<String, Integer> ruleIds, DiagnosticPlan diagnosticPlan,
               java.util.function.Function<Expr, String> literalLabel) {
        this.ir = ir; this.expressions = expressions; this.rules = rules; this.tokens = tokens;
        this.ruleIds = ruleIds; this.diagnosticPlan = diagnosticPlan; this.literalLabel = literalLabel;
        var policies = ir.semantics().trivia().policies();
        for (int i = 0; i < policies.size(); i++) {
            int canonical = i;
            for (int j = 0; j < i; j++)
                if (sameDelimiters(policies.get(j), policies.get(i))) { canonical = canonicalPolicy.get(policies.get(j).id()); break; }
            canonicalPolicy.put(policies.get(i).id(), canonical);
            policyByIndex.add(policies.get(canonical));
        }
        ir.semantics().recovery().forEach(r -> { recoverySites.addAll(r.applicationSites()); recoveryRules.add(r.ruleId()); });
        for (ScopeEffect effect : ir.semantics().scope().effects())
            if (effect.events().stream().anyMatch(e -> "entry".equals(e.when()))) entryEffectRules.add(effect.ruleId());
        for (TriviaInsertionPlanItem item : ir.semantics().trivia().insertionPlan())
            entryPolicies.computeIfAbsent(item.boundary().ownerExprId(), key -> new HashMap<>())
                .computeIfAbsent(item.boundary().at() + ":" + item.boundary().childIndex(), key -> new ArrayList<>())
                .add(item.policyId());
        Ranges first = new Ranges(); String any = null;
        for (TriviaPolicy policy : policies) {
            if (policy.effectiveDelimiterSet().isEmpty()) continue;
            any = policy.id();
            for (Delimiter delimiter : policy.effectiveDelimiterSet()) {
                if ("characters".equals(delimiter.kind())) for (int cp : delimiter.codePoints()) first = first.union(Ranges.single(cp));
                else if (delimiter.open() != null && !delimiter.open().isEmpty()) first = first.union(Ranges.single(delimiter.open().codePointAt(0)));
                else { first = Ranges.of(0, 0x10ffff); break; }
            }
        }
        allTriviaFirst = first; anyPolicyId = any;
        fixpoint();
    }

    private static boolean sameDelimiters(TriviaPolicy a, TriviaPolicy b) {
        var left = a.effectiveDelimiterSet(); var right = b.effectiveDelimiterSet();
        if (left.size() != right.size()) return false;
        for (int i = 0; i < left.size(); i++) {
            Delimiter x = left.get(i), y = right.get(i);
            if (!java.util.Objects.equals(x.label(), y.label()) || !java.util.Objects.equals(x.expression(), y.expression()) || !java.util.Objects.equals(x.kind(), y.kind()) || !java.util.Objects.equals(x.open(), y.open())
                || !java.util.Objects.equals(x.close(), y.close()) || !java.util.Objects.equals(x.codePoints(), y.codePoints())) return false;
        }
        return true;
    }

    private List<Integer> policiesAt(String exprId, String at, Integer childIndex) {
        var byExpr = entryPolicies.get(exprId);
        if (byExpr == null) return List.of();
        var ids = byExpr.get(at + ":" + childIndex);
        if (ids == null) return List.of();
        return ids.stream().map(canonicalPolicy::get).toList();
    }

    // ---- 先頭文字集合の最小不動点 ----

    private void fixpoint() {
        order.addAll(expressions.keySet());
        for (String id : order) starts.put(id, new Known());
        for (int round = 0; round < 1000; round++) {
            boolean changed = false;
            for (String id : order) {
                boolean wasUnknown = unknown.contains(id);
                Known before = starts.get(id);
                Known next = compute(id);
                if (next == null) {
                    if (!wasUnknown) { unknown.add(id); starts.remove(id); changed = true; }
                } else if (wasUnknown || !next.same(before)) {
                    unknown.remove(id); starts.put(id, next); changed = true;
                }
            }
            if (!changed) return;
        }
        // 収束しない文法は保守的に全て Unknown にする（除外しない）。
        unknown.addAll(order); starts.clear();
    }

    private Known start(String id) { return unknown.contains(id) ? null : starts.get(id); }

    /** trivia 境界: direct を skipped へ移す。policy が混ざれば Unknown。 */
    private Known wrap(int policy, Known k) {
        if (k == null || k.policy != null && k.policy != policy) return null;
        var out = new Known();
        out.nullable = k.nullable;
        out.skipped = k.direct.union(k.skipped);
        out.policy = out.skipped.isEmpty() ? null : policy;
        return out;
    }

    private Known wrapAll(Known k, List<Integer> policies) {
        for (int i = policies.size() - 1; i >= 0; i--) {
            k = wrap(policies.get(i), k);
            if (k == null) return null;
        }
        return k;
    }

    private Known union(Known a, Known b, boolean nullable) {
        if (a == null || b == null) return null;
        Integer policy;
        if (a.policy != null && b.policy != null && !a.policy.equals(b.policy)) return null;
        policy = a.policy != null ? a.policy : b.policy;
        var out = new Known();
        out.nullable = nullable; out.direct = a.direct.union(b.direct); out.skipped = a.skipped.union(b.skipped); out.policy = policy;
        return out;
    }

    private Known compute(String id) {
        Expr expr = expressions.get(id);
        ExprNode n = expr.node();
        if (!"inherit".equals(expr.execution().childMode())) return null;
        Known inner;
        switch (n.kind()) {
            case "literal" -> {
                if (!Boolean.TRUE.equals(n.caseSensitive())) return null;
                inner = new Known();
                if (n.value().isEmpty()) inner.nullable = true;
                else inner.direct = Ranges.single(n.value().codePointAt(0));
            }
            case "tokenRef" -> inner = tokenStart(n.symbolId());
            case "seq" -> {
                Known acc = new Known(); acc.nullable = true;
                for (int index = 0; index < n.items().size(); index++) {
                    var policies = new ArrayList<Integer>();
                    for (int j = 0; j < index; j++) policies.addAll(policiesAt(id, "afterChild", j));
                    Known item = wrapAll(start(n.items().get(index)), policies);
                    if (item == null) return null;
                    acc = union(acc, item, acc.nullable && item.nullable);
                    if (acc == null) return null;
                    if (!item.nullable) break;
                }
                inner = acc;
            }
            case "choice" -> {
                Known acc = new Known();
                for (String alt : n.alternatives()) {
                    Known a = start(alt);
                    if (a == null) return null;
                    acc = union(acc, a, acc.nullable || a.nullable);
                    if (acc == null) return null;
                }
                inner = acc;
            }
            case "group" -> inner = start(n.child()) == null ? null : start(n.child()).copy();
            case "optional" -> {
                Known k = start(n.child());
                if (k == null) return null;
                inner = k.copy(); inner.nullable = true;
            }
            case "repeat" -> {
                if (n.terminator() != null || Integer.valueOf(0).equals(n.max())) return null;
                Known k = start(n.child());
                if (k == null) return null;
                inner = k.copy(); inner.nullable = k.nullable || Integer.valueOf(0).equals(n.min());
            }
            case "ruleRef" -> {
                Rule rule = rules.get(n.symbolId());
                if (rule == null) return null;
                Known k = start(rule.body());
                inner = k == null ? null : k.copy();
            }
            default -> { return null; }
        }
        if (inner == null) return null;
        var entry = new ArrayList<Integer>(policiesAt(id, "entry", null));
        entry.addAll(policiesAt(id, "helperEntry", null));
        return wrapAll(inner, entry);
    }

    private Known tokenStart(String symbolId) {
        Token token = tokens.get(symbolId);
        if (token == null) return null;
        var dfaPlan = ir.plan().lexical().stream()
            .filter(p -> p.tokenId().equals(symbolId) && "dfa".equals(p.strategy()) && p.dfa() != null).findFirst().orElse(null);
        if (dfaPlan != null) {
            var startState = dfaPlan.dfa().states().stream().filter(s -> s.id().equals(dfaPlan.dfa().startState())).findFirst().orElse(null);
            if (startState == null) return null;
            Ranges r = new Ranges();
            for (var transition : startState.transitions()) r = r.union(Ranges.of(transition.range().start(), transition.range().end()));
            return known(r);
        }
        TokenSpec spec = token.spec();
        return switch (spec.kind()) {
            case "builtin" -> switch (spec.which()) {
                case "Number" -> known(chars("+-.0123456789"));
                case "Identifier", "AlphabetUnderScore" -> known(Ranges.of('A', 'Z').union(Ranges.of('a', 'z')).union(Ranges.single('_')));
                case "AlphabetNumericUnderScore" -> known(Ranges.of('A', 'Z').union(Ranges.of('a', 'z')).union(Ranges.of('0', '9')).union(Ranges.single('_')));
                case "Alphabet" -> known(Ranges.of('A', 'Z').union(Ranges.of('a', 'z')));
                case "Digit" -> known(Ranges.of('0', '9'));
                case "SingleQuoted" -> known(Ranges.single('\''));
                case "Quoted" -> known(Ranges.single('"'));
                case "Space" -> known(Ranges.of(9, 13).union(Ranges.single(32)));
                case "CodePointSet" -> {
                    Ranges r = new Ranges();
                    for (int cp : spec.codePoints()) r = r.union(Ranges.single(cp));
                    yield known(r);
                }
                default -> null;
            };
            case "charRange" -> known(Ranges.of(spec.min(), spec.max()));
            case "caseInsensitive" -> null;
            // scanner registry の宣言（IR firstChars）がある Extern だけ Known。宣言は
            // 「集合外の先頭文字では消費なしで失敗する」必要条件。
            case "extern" -> {
                if (spec.firstChars() == null || spec.firstChars().isEmpty()) yield null;
                Ranges r = new Ranges();
                for (CharRange range : spec.firstChars()) r = r.union(Ranges.of(range.start(), range.end()));
                yield known(r);
            }
            default -> null;
        };
    }

    private static Known known(Ranges direct) { var k = new Known(); k.direct = direct; return k; }
    private static Ranges chars(String text) {
        Ranges r = new Ranges();
        for (int i = 0; i < text.length(); i++) r = r.union(Ranges.single(text.charAt(i)));
        return r;
    }

    // ---- 候補ごとの再生計画 ----

    /** 除外できるなら計画を、できないなら null を返す。 */
    Guard guard(String alternativeId) {
        Known k = start(alternativeId);
        if (k == null || k.nullable) return null;
        Ranges reachable = k.direct.union(k.skipped);
        if (reachable.isEmpty()) return null;
        ExprNode node = expressions.get(alternativeId).node();
        // 葉は本物の呼出しの方が安い（Rust backend と同じ）。
        if (node.kind().equals("literal") || node.kind().equals("tokenRef")) return null;
        var fold = new Fold();
        fold.walk(alternativeId);
        if (!fold.ok || fold.labels.isEmpty()) return null;
        if (fold.sawTrivia && anyPolicyId == null) return null;
        var labels = new ArrayList<String>();
        for (String label : fold.labels) if (!labels.contains(label)) labels.add(label);
        return new Guard(reachable.union(allTriviaFirst), List.copyOf(labels),
            fold.sawTrivia ? policyByIndex.get(fold.policy).id() : null, List.copyOf(fold.bestSuffix), fold.maxStack);
    }

    /** 最初の significant token で失敗する経路を、記録される label 列へ畳み込む。 */
    private final class Fold {
        final List<String> labels = new ArrayList<>();
        final Deque<Integer> stack = new ArrayDeque<>();
        List<Integer> bestSuffix = List.of();
        int bestDepth = -1, maxStack, nodes;
        Integer policy;
        boolean sawTrivia, ok = true;
        final Set<String> visiting = new HashSet<>();

        private void record(String label) {
            labels.add(label);
            int depth = stack.size();
            if (depth > bestDepth) { bestDepth = depth; bestSuffix = suffix(); }
        }

        /** 外側から内側の順。 */
        private List<Integer> suffix() {
            var out = new ArrayList<Integer>(stack);
            java.util.Collections.reverse(out);
            return out;
        }

        private void trivia(String exprId, String at) {
            for (int p : policiesAt(exprId, at, null)) {
                if (policyByIndex.get(p).effectiveDelimiterSet().isEmpty()) continue;
                if (policy == null) policy = p;
                else if (policy != p) { ok = false; return; }
                sawTrivia = true;
                record(TRIVIA_LABEL);
            }
        }

        void walk(String id) {
            if (!ok) return;
            if (++nodes > INLINE_MAX) { ok = false; return; }
            Known k = start(id);
            if (k == null || k.nullable) { ok = false; return; }
            Expr expr = expressions.get(id);
            ExprNode n = expr.node();
            if (!"inherit".equals(expr.execution().childMode())) { ok = false; return; }
            switch (n.kind()) {
                case "seq" -> {
                    if (n.items().isEmpty()) { ok = false; return; }
                    trivia(id, "entry"); trivia(id, "helperEntry");
                    if (!ok) return;
                    walk(n.items().get(0));
                }
                case "group" -> {
                    if (!"delegate".equals(expr.execution().transaction())) { trivia(id, "entry"); trivia(id, "helperEntry"); }
                    if (!ok) return;
                    walk(n.child());
                }
                case "choice" -> {
                    if ("longest".equals(n.selection())) { ok = false; return; }
                    var plan = ir.plan().choices().stream().filter(p -> p.exprId().equals(id)).findFirst().orElse(null);
                    if (plan != null && "longest".equals(plan.strategy())) { ok = false; return; }
                    for (String alternative : n.alternatives()) { walk(alternative); if (!ok) return; }
                }
                case "ruleRef" -> {
                    String symbol = n.symbolId();
                    if (recoverySites.contains(id) || entryEffectRules.contains(symbol) || recoveryRules.contains(symbol)) { ok = false; return; }
                    Rule rule = rules.get(symbol);
                    Integer ruleId = ruleIds.get(symbol);
                    if (rule == null || ruleId == null || !visiting.add(symbol)) { ok = false; return; }
                    stack.push(ruleId);
                    maxStack = Math.max(maxStack, stack.size());
                    walk(rule.body());
                    if (ok) {
                        for (Recovery recovery : ir.semantics().recovery())
                            if (recovery.ruleId().equals(symbol)) for (String pattern : recovery.resolvedSyncPatterns()) record("'" + pattern + "'");
                        for (String hint : diagnosticPlan.rule(symbol)) record(hint);
                    }
                    stack.pop(); visiting.remove(symbol);
                    if (!ok) return;
                }
                case "literal" -> {
                    if (n.value().isEmpty()) { ok = false; return; }
                    record(literalLabel.apply(expr));
                }
                default -> { ok = false; return; }
            }
            if (!ok) return;
            for (String hint : diagnosticPlan.expression(id)) record(hint);
        }
    }

    /** 除外できる候補が 1 つでもある選択か。 */
    boolean ordered(Expr expr) {
        if ("longest".equals(expr.node().selection())) return false;
        var plan = ir.plan().choices().stream().filter(p -> p.exprId().equals(expr.id())).findFirst().orElse(null);
        return plan == null || "ordered".equals(plan.strategy());
    }

}
