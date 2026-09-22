package generated.internal;
import java.util.List;
import java.util.Map;
/** Immutable construction commands. No public AST or numeric conversion during recognition. */
public record Recipe(String id, int start, int end, Map<String, List<Value>> fields, String fallback) {
    public Recipe { fields = Map.copyOf(fields); }
    public record Value(int start, int end, List<Recipe> nodes, String text) {
        public Value { nodes = List.copyOf(nodes); }
        public Value(int start, int end, List<Recipe> nodes) { this(start, end, nodes, null); }
    }
}
