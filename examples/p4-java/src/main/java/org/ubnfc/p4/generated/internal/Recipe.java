package org.ubnfc.p4.generated.internal;
import java.util.List;
import java.util.Map;
/** Immutable construction commands. No public AST or numeric conversion during recognition. */
public record Recipe(String id, int start, int end, Map<String, List<Value>> fields, String fallback) {
    public Recipe { fields = Map.copyOf(fields); }
    /** elements retains transparent helper values separately from the enclosing capture extent. */
    public record Value(int start, int end, List<Recipe> nodes, String text, List<Value> elements) {
        public Value(int start, int end, List<Recipe> nodes, String text) { this(start, end, nodes, text, null); }
        public Value { nodes = List.copyOf(nodes); elements = elements == null ? null : List.copyOf(elements); }
        public Value(int start, int end, List<Recipe> nodes) { this(start, end, nodes, null); }
    }
}
