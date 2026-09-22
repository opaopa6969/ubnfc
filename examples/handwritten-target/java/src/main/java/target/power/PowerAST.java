package target.power;

import java.util.List;

public sealed interface PowerAST {
    // Atom ::= NUMBER の非 mapping alias は String。直接 capture の int とは別契約。
    record Power(String left, List<String> op, List<Power> right) implements PowerAST {
        public Power { op = List.copyOf(op); right = List.copyOf(right); }
    }
}
