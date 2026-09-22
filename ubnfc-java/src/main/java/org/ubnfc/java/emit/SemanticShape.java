package org.ubnfc.java.emit;

import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import org.ubnfc.java.ir.GrammarIr.Expr;
import org.ubnfc.java.ir.GrammarIr.Rule;
import org.ubnfc.java.ir.Validation;

/** Value boundaries in transparent helpers; punctuation is not a semantic value. */
final class SemanticShape {
    record Shape(String kind, boolean many) {}
    private static final Shape TEXT = new Shape("text", false);
    private final Map<String, Expr> expressions;
    private final Map<String, Rule> rules;
    SemanticShape(Map<String, Expr> expressions, Map<String, Rule> rules) {
        this.expressions = expressions; this.rules = rules;
    }
    Shape of(String id) { return of(id, new HashSet<>()); }
    private Shape of(String id, Set<String> seen) {
        if (!seen.add(id)) return TEXT;
        var node = expressions.get(id).node();
        if (node.kind().equals("ruleRef")) {
            var rule = rules.get(node.symbolId());
            if (rule.mappingTypeId() != null) return new Shape("node", false);
            return of(rule.body(), seen);
        }
        Shape result = null;
        for (String child : node.kind().equals("separated") ? java.util.List.of(node.element()) : Validation.children(node)) {
            Shape shape = of(child, new HashSet<>(seen));
            if (node.kind().equals("seq") && shape.kind().equals("text")) continue;
            result = result == null ? shape : new Shape(result.kind().equals(shape.kind()) ? shape.kind() : "mixed",
                result.many() || shape.many() || node.kind().equals("seq"));
        }
        if (result == null) return TEXT;
        return new Shape(result.kind(), result.many() || !result.kind().equals("text") &&
            Set.of("repeat", "separated").contains(node.kind()));
    }
}
