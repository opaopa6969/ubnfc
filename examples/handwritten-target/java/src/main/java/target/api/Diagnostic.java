package target.api;

import java.util.List;

public record Diagnostic(int farthestCp, List<String> expected, String deepestRule,
                         List<String> ruleStack) {
    public Diagnostic {
        expected = List.copyOf(expected);
        ruleStack = List.copyOf(ruleStack);
    }
}
