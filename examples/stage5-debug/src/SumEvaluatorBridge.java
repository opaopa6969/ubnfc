package org.unlaxer.ubnfc.examples.stage5;

import java.math.BigDecimal;
import java.util.List;
import java.util.Optional;
import org.unlaxer.ubnfc.examples.stage5.generated.Stage5AST;
import org.unlaxer.ubnfc.examples.stage5.lsp.dap.EvaluatorBridge;

/**
 * 生成物を継承せずに差し込む最小の評価器。Program 内の let の数値を加算する。
 * use は索引の実演用で、値を変更しない。これは逐次実行器ではなく AST 全体の純粋な集計例。
 */
public final class SumEvaluatorBridge implements EvaluatorBridge {
    private BigDecimal total = BigDecimal.ZERO;

    @Override public void onLaunch(String program, String text, Object root) {
        total = sum(root); // セッションを再利用しても前回の値を持ち越さない。
    }

    private static BigDecimal sum(Object node) {
        if (node instanceof Stage5AST.Let value) return new BigDecimal(value.value());
        if (node instanceof Stage5AST.Program program)
            return program.items().stream().map(SumEvaluatorBridge::sum).reduce(BigDecimal.ZERO, BigDecimal::add);
        if (node instanceof Stage5AST.Block block)
            return block.items().stream().map(SumEvaluatorBridge::sum).reduce(BigDecimal.ZERO, BigDecimal::add);
        return BigDecimal.ZERO;
    }

    @Override public List<NamedValue> variables(Object node) {
        return List.of(new NamedValue("total", total.toPlainString(), "number"));
    }

    @Override public Optional<Evaluation> evaluate(String expression, Object node, String context) {
        return "total".equals(expression.strip())
            ? Optional.of(new Evaluation(total.toPlainString(), "number")) : Optional.empty();
    }
}
