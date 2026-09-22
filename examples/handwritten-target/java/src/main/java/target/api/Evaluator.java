package target.api;

import target.operators.OperatorsAST;
import target.power.PowerAST;

/** corpus の value と同じ double 演算。Java int の葉から始めても除算は実数除算。 */
public final class Evaluator {
    private Evaluator() {}
    public static double evaluate(OperatorsAST node) {
        if (node instanceof OperatorsAST.Number number) return number.value();
        var binary = (OperatorsAST.Binary) node;
        double value = evaluate(binary.left());
        for (int i = 0; i < binary.op().size(); i++) {
            double right = evaluate(binary.right().get(i));
            value = switch (binary.op().get(i)) {
                case "+" -> value + right;
                case "-" -> value - right;
                case "*" -> value * right;
                case "/" -> value / right;
                default -> throw new IllegalArgumentException(binary.op().get(i));
            };
        }
        return value;
    }

    public static double evaluate(PowerAST node) {
        var power = (PowerAST.Power) node;
        // corpus の probe.rs.txt と同じく alias 内に残る末尾コメントを評価時だけ除く。
        int comment = power.left().indexOf('/');
        double left = Double.parseDouble((comment < 0 ? power.left() : power.left().substring(0, comment)).strip());
        return power.right().isEmpty() ? left : Math.pow(left, evaluate(power.right().getFirst()));
    }
}
