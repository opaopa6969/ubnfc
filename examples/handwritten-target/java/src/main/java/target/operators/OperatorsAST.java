package target.operators;

import java.util.List;

public sealed interface OperatorsAST {
    record Binary(OperatorsAST left, List<String> op, List<OperatorsAST> right) implements OperatorsAST {
        public Binary { op = List.copyOf(op); right = List.copyOf(right); }
    }
    record Number(int value) implements OperatorsAST {}
}
