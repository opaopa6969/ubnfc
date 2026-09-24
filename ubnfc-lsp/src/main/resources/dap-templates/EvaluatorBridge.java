package __PACKAGE__;

import java.util.List;
import java.util.Optional;

/**
 * 「文法デバッガ」に言語の意味論を差し込むための拡張点（issue #28 段階 4 の案 b）。
 *
 * <p>生成される {@link DebugAdapter} は<b>文法デバッガ</b>である。入力を生成パーサで解析し、
 * 型付き AST を深さ優先で歩くだけで、<b>式の値は計算しない</b>。値が要る debugger
 * （Debug Console の {@code evaluate}、watch、{@code setVariable}）は言語ごとの評価器が要るので、
 * ubnfc は<b>この契約だけ</b>を生成し、実装は言語作者に任せる。
 *
 * <p>旧スタック（{@code unlaxer-dsl} の {@code DAPGenerator}）も同じ切り分けだった。
 * base 生成の {@code {Name}DebugAdapter} は文法デバッガで、
 * {@code evaluate} / {@code setVariable} を持たない。
 * TinyExpression P4 だけが生成物を<b>継承</b>して
 * {@code TinyExpressionP4DebugAdapterExt}（{@code evaluate} / {@code setVariable} /
 * {@code variables} の上書き）を書き、{@code TinyExpressionDapRuntimeBridge} 経由で
 * 手書きの {@code Calculator} / {@code CalculationContext} / {@code ExecutionBackend} を呼んでいた。
 * ubnfc では<b>継承のかわりに合成</b>にしてある。同じ材料は次のように移る。
 *
 * <pre>{@code
 * // 旧: TinyExpressionP4DebugAdapterExt extends TinyExpressionP4DebugAdapter
 * // 新: EvaluatorBridge を 1 つ書いて DebugAdapter に渡すだけ（生成物は 1 行も継承しない）
 * public final class P4EvaluatorBridge implements EvaluatorBridge {
 *     private final TinyExpressionDapRuntimeBridge runtime = new TinyExpressionDapRuntimeBridge();
 *
 *     @Override public void onLaunch(String program, String text, Object root) {
 *         runtime.load(text);                       // 旧 Ext のコンストラクタ相当
 *     }
 *     @Override public List<NamedValue> variables(Object node) {
 *         return runtime.probe(node).entrySet().stream()  // 旧 runtimeProbeVariables 相当
 *             .map(e -> new NamedValue(e.getKey(), String.valueOf(e.getValue()), "runtime")).toList();
 *     }
 *     @Override public Optional<Evaluation> evaluate(String expression, Object node, String context) {
 *         return Optional.of(new Evaluation(runtime.evaluate(expression), "number"));
 *     }
 * }
 *
 * // 起動側（VSIX の DapMain、またはテスト）
 * new DebugAdapter(scanners, new P4EvaluatorBridge());
 * }</pre>
 *
 * <p>既定は {@link #NOOP}。このとき {@code evaluate} / {@code setVariable} は
 * <b>黙って空や 0 を返さず</b>、「文法デバッガでは未対応。{@code EvaluatorBridge} を実装せよ」という
 * 明示的なエラーになる。
 */
public interface EvaluatorBridge {
    /** variables パネルに足す 1 行。{@code children} を持たせると入れ子にできる。 */
    record NamedValue(String name, String value, String type, List<NamedValue> children) {
        public NamedValue { children = List.copyOf(children); }
        public NamedValue(String name, String value, String type) { this(name, value, type, List.of()); }
    }

    /** {@code evaluate} / {@code setVariable} の答え。 */
    record Evaluation(String result, String type, List<NamedValue> children) {
        public Evaluation { children = List.copyOf(children); }
        public Evaluation(String result, String type) { this(result, type, List.of()); }
    }

    /** {@code launch} が解析に成功した直後に 1 度だけ呼ばれる。{@code root} は型付き AST の根。 */
    default void onLaunch(String program, String text, Object root) { }

    /** ステップが節点へ入った（旧 {@code onBeforeStep}）。 */
    default void onEnterNode(Object node) { }

    /** ステップが節点から出た（旧 {@code onAfterStep}）。 */
    default void onExitNode(Object node) { }

    /** その節点で variables に足す値（旧 {@code runtimeVariables} / {@code additionalVariables}）。 */
    default List<NamedValue> variables(Object node) { return List.of(); }

    /**
     * Debug Console / watch の式評価。{@code Optional.empty()} を返すと
     * 「文法デバッガでは未対応」の error になる（既定）。
     *
     * @param context DAP の {@code EvaluateArguments.context}（{@code "repl"} / {@code "watch"} / {@code "hover"}）
     */
    default Optional<Evaluation> evaluate(String expression, Object node, String context) { return Optional.empty(); }

    /** variables パネルからの書き換え。{@code Optional.empty()} を返すと未対応の error になる（既定）。 */
    default Optional<Evaluation> setVariable(String container, String name, String value, Object node) {
        return Optional.empty();
    }

    /** 何もしない既定。文法デバッガはこれで動く。 */
    EvaluatorBridge NOOP = new EvaluatorBridge() { };

    /**
     * クラス名から実装を読む（無指定・空なら {@link #NOOP}）。
     * {@code DapMain} の {@code --evaluator <fqcn>} と
     * システムプロパティ {@code ubnfc.dap.evaluator} がこれを使う。
     */
    static EvaluatorBridge load(String className) {
        if (className == null || className.isBlank()) return NOOP;
        try {
            return (EvaluatorBridge) Class.forName(className).getDeclaredConstructor().newInstance();
        } catch (ReflectiveOperationException | RuntimeException error) {
            throw new IllegalStateException("EvaluatorBridge を読めません: " + className, error);
        }
    }
}
