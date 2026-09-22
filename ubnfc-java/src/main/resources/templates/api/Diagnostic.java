package generated.api;
import java.util.List;
public record Diagnostic(String kind, int offset, int farthestCp, List<String> expected,
                         String deepestRule, List<String> ruleStack, List<String> farthestExpected) {
    public Diagnostic { expected = List.copyOf(expected); ruleStack = List.copyOf(ruleStack); farthestExpected = List.copyOf(farthestExpected); }
    public Diagnostic(String kind, int offset, int farthestCp, List<String> expected, String deepestRule, List<String> ruleStack) {
        this(kind, offset, farthestCp, expected, deepestRule, ruleStack, expected);
    }
}
