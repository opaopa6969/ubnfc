package target.power;

import static org.junit.jupiter.api.Assertions.*;
import java.util.function.IntBinaryOperator;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;
import target.api.ParseOptions;
import target.internal.Session;

class RuleMemoTest {
    static Stream<Arguments> rules() {
        return Stream.of(Arguments.of((IntBinaryOperator) PowerParser::parseExpr, "2^3^2"),
            Arguments.of((IntBinaryOperator) PowerParser::parseAtom, "2"),
            Arguments.of((IntBinaryOperator) PowerParser::parseRecursive, "2^3"));
    }
    @ParameterizedTest @MethodSource("rules")
    void everyRuleMemoizesSuccessAndFailure(IntBinaryOperator parse, String valid) {
        for (String input : new String[]{valid, "!"}) {
            var s = new Session(input, ParseOptions.DEFAULT, "Expr", "Atom", "Expr.recursive");
            Session.CURRENT.set(s);
            try {
                int first = parse.applyAsInt(0, 0);
                assertEquals(input.equals(valid) ? 0 : -1, first);
                int consumed = s.consumed, matched = s.matched, hits = s.memoHits;
                var recipe = s.recipe; var events = s.events; var diagnostics = s.diagnostic();
                assertEquals(first, parse.applyAsInt(0, 0));
                assertEquals(hits + 1, s.memoHits);
                assertEquals(consumed, s.consumed); assertEquals(matched, s.matched);
                assertSame(recipe, s.recipe); assertSame(events, s.events);
                assertEquals(diagnostics, s.diagnostic());
            } finally { Session.CURRENT.remove(); }
        }
    }

    @Test void baseFallbackActuallyRevisitsSuccessAndFailureMemo() {
        for (String input : new String[]{"2", "!"}) {
            var s = new Session(input, ParseOptions.DEFAULT, "Expr", "Atom", "Expr.recursive");
            Session.CURRENT.set(s);
            try {
                PowerParser.parseExpr(0, 0);
                assertEquals(1, s.memoHits);
            } finally { Session.CURRENT.remove(); }
        }
    }
}
