package __PACKAGE__;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * 型付き AST の走査。<b>文法ごとに生成時へ畳み込んだ {@code switch}</b> なので反射を使わない。
 *
 * <p>旧スタック（{@code unlaxer-dsl} の {@code DAPRuntimeEmitter.collectAstNodeMeta}）は
 * {@code Class.getMethods()} を舐めて子を探していた。この順序は JVM が保証しないので、
 * 旧 DAP の AST ステップ列は<b>実行ごとに並びが変わり得る</b>。ubnfc は IR の
 * {@code semantics.astTypes}（型と欄の並び）から {@code switch} を生成するので、
 * 並びは<b>文法の宣言順で決まり決定的</b>になる。
 *
 * <p>手で編集しない（{@code ubnfc-lsp --emit-dap} が毎回上書きする）。
 */
public final class AstWalk {
    private AstWalk() {}

    /** AST の 1 欄。{@code value} は節点・文字列・数・{@code List}・{@code Optional} のいずれか。 */
    public record Field(String name, Object value) {}

    /** この値が AST の節点か（生成された union 型に属するか）。 */
    public static boolean isNode(Object value) {
        return __NODE_CHECK__;
    }

    /** 節点の型名（{@code FormulaExpr} のような局所名）。 */
    public static String typeName(Object node) {
        if (node == null) return "null";
        return switch (node) {
__TYPE_NAME_CASES__
            default -> node.getClass().getSimpleName();
        };
    }

    /** 節点の型 → その AST 型を作る規則の id（{@code Meta} の表を引くのに使う）。無ければ null。 */
    public static String ruleIdOf(Object node) {
        if (node == null) return null;
        return switch (node) {
__RULE_ID_CASES__
            default -> null;
        };
    }

    /** 節点の欄を<b>文法の宣言順</b>で返す。 */
    public static List<Field> fieldsOf(Object node) {
        var out = new ArrayList<Field>();
        if (node == null) return out;
        switch (node) {
__FIELD_CASES__
            default -> { }
        }
        return out;
    }

    /** 欄の値に含まれる子節点を宣言順に取り出す（{@code List} は要素順、{@code Optional} は中身）。 */
    public static List<Object> childrenOf(Object value) {
        var out = new ArrayList<Object>();
        collect(value, out);
        return out;
    }

    private static void collect(Object value, List<Object> out) {
        if (value == null) return;
        if (value instanceof Optional<?> optional) { optional.ifPresent(inner -> collect(inner, out)); return; }
        if (value instanceof List<?> list) { for (Object item : list) collect(item, out); return; }
        if (isNode(value)) out.add(value);
    }
}
