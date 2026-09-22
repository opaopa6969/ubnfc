package target.internal;

import java.util.List;

/** AST ではない immutable な構築命令。memo の参照中は再利用・truncate しない。 */
public final class Recipe {
    public enum Kind { NUMBER, BINARY, POWER, TEXT }
    public final Kind kind;
    public final int start, end, valueStart, valueEnd;
    public final Recipe left;
    public final List<Recipe> right;
    public final List<String> operators;

    public Recipe(Kind kind, int start, int end, int valueStart, int valueEnd,
                  Recipe left, List<String> operators, List<Recipe> right) {
        this.kind = kind;
        this.start = start;
        this.end = end;
        this.valueStart = valueStart;
        this.valueEnd = valueEnd;
        this.left = left;
        this.operators = List.copyOf(operators);
        this.right = List.copyOf(right);
    }
}
