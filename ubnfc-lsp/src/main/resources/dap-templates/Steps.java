package __PACKAGE__;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import __API_PACKAGE__.Span;

/**
 * 型付き AST を<b>前順（親 → 子、子は文法の宣言順）</b>で平らにしたステップ列。
 *
 * <p>ステップ 1 つが AST の節点 1 つに対応し、{@code index} がそのまま前順の位置になる。
 * {@code parent} は親ステップの {@code index}（根は -1）なので、stackTrace の
 * 「根から現在の節点まで」はこの鎖を辿るだけで作れる。
 *
 * <p>座標は生成パーサと同じ code point（LSP/DAP へ渡すときだけ
 * {@code Positions} が UTF-16 へ直す）。{@code ParseResult.nodeSpans()} に
 * 範囲が無い節点（欄を持たない記録型など）は<b>子の範囲の合併</b>で埋め、
 * 子も無ければ 0 幅にする。
 */
public final class Steps {
    /** ステップ 1 つ。 */
    public record Step(int index, Object node, String typeName, int startCp, int endCp, int depth, int parent) { }

    private Steps() {}

    /** 前順で平らにする。{@code root} が null なら空。 */
    public static List<Step> of(Object root, Map<Object, Span> spans) {
        var out = new ArrayList<Step>();
        if (root != null) visit(root, spans, 0, -1, out);
        return List.copyOf(out);
    }

    private static int[] visit(Object node, Map<Object, Span> spans, int depth, int parent, List<Step> out) {
        int index = out.size();
        out.add(null); // 前順の位置を先に予約する（子は index+1 以降に並ぶ）
        Span own = spans == null ? null : spans.get(node);
        int start = own == null ? Integer.MAX_VALUE : own.start();
        int end = own == null ? Integer.MIN_VALUE : own.end();
        for (var field : AstWalk.fieldsOf(node))
            for (Object child : AstWalk.childrenOf(field.value())) {
                int[] span = visit(child, spans, depth + 1, index, out);
                start = Math.min(start, span[0]);
                end = Math.max(end, span[1]);
            }
        if (start > end) { start = 0; end = 0; }
        out.set(index, new Step(index, node, AstWalk.typeName(node), start, end, depth, parent));
        return new int[]{start, end};
    }

    /** 根から {@code index} までの鎖（根が先頭）。 */
    public static List<Step> chain(List<Step> steps, int index) {
        var reversed = new ArrayList<Step>();
        for (int at = index; at >= 0; at = steps.get(at).parent()) reversed.add(steps.get(at));
        var out = new ArrayList<Step>(reversed.size());
        for (int i = reversed.size() - 1; i >= 0; i--) out.add(reversed.get(i));
        return out;
    }
}
