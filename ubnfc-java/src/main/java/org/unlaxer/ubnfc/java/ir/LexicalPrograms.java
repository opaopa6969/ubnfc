package org.unlaxer.ubnfc.java.ir;

import java.util.HashSet;
import java.util.Set;
import org.unlaxer.ubnfc.java.ir.GrammarIr.LexicalExpression;

/** Validate untrusted IR before emitting an atomic, locally scoped token program. */
final class LexicalPrograms {
    private LexicalPrograms() {}
    static void validate(LexicalExpression e) { validate(e, new HashSet<>(), 0, new int[]{0}); }
    private static void require(boolean ok, String message) {
        if (!ok) throw new IllegalArgumentException("lexical program: " + message);
    }
    private static void validate(LexicalExpression e, Set<String> bound, int depth, int[] nodes) {
        require(e != null && e.op() != null && e.text() != null && e.min() != null && e.max() != null, "missing field");
        require(depth <= 128 && ++nodes[0] <= 4096, "expansion limit");
        int arity = switch (e.op()) {
            case "CAPTURE", "LOOK", "NOT", "SCOPE", "REPEAT" -> 1;
            case "SEQUENCE", "CHOICE" -> e.children().isEmpty() ? -1 : e.children().size();
            case "LITERAL", "ANY", "EOF", "BOF", "BOL", "EOL", "RANGE", "EXCEPT", "BACKREF" -> 0;
            default -> throw new IllegalArgumentException("unresolved or unknown lexical op " + e.op());
        };
        require(e.children().size() == arity, "arity");
        switch (e.op()) {
            case "RANGE" -> require(e.min() >= 0 && e.min() <= e.max() && e.max() <= 0x10ffff
                && !(e.min() >= 0xd800 && e.min() <= 0xdfff) && !(e.max() >= 0xd800 && e.max() <= 0xdfff), "scalar range");
            case "BACKREF" -> require(bound.contains(e.text()), "unbound SAME_AS");
            case "CHOICE" -> {
                Set<String> definite = null;
                for (var child : e.children()) {
                    Set<String> branch = new HashSet<>(bound);
                    validate(child, branch, depth + 1, nodes);
                    if (definite == null) definite = branch; else definite.retainAll(branch);
                }
                bound.addAll(definite);
            }
            case "LOOK", "NOT", "SCOPE" -> validate(e.children().get(0), e.op().equals("SCOPE")
                ? new HashSet<>() : new HashSet<>(bound), depth + 1, nodes);
            case "REPEAT" -> {
                require(e.min() >= 0 && e.max() >= -1 && (e.max() < 0 || e.min() <= e.max()), "repeat bounds");
                Set<String> iteration = new HashSet<>(bound);
                validate(e.children().get(0), iteration, depth + 1, nodes);
                require(e.max() >= 0 || !nullable(e.children().get(0)), "nullable unbounded repeat");
                if (e.min() > 0) bound.addAll(iteration);
            }
            case "CAPTURE" -> { validate(e.children().get(0), bound, depth + 1, nodes); bound.add(e.text()); }
            case "SEQUENCE" -> { for (var child : e.children()) validate(child, bound, depth + 1, nodes); }
            default -> { }
        }
    }
    static boolean nullable(LexicalExpression e) {
        return switch (e.op()) {
            case "LITERAL" -> e.text().isEmpty();
            case "ANY", "RANGE", "EXCEPT" -> false;
            case "SEQUENCE" -> e.children().stream().allMatch(LexicalPrograms::nullable);
            case "CHOICE" -> e.children().stream().anyMatch(LexicalPrograms::nullable);
            case "REPEAT" -> e.min() == 0 || nullable(e.children().get(0));
            case "CAPTURE", "SCOPE" -> nullable(e.children().get(0));
            default -> true;
        };
    }
}
