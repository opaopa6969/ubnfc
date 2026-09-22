package target;

import static org.junit.jupiter.api.Assertions.*;
import java.util.List;
import java.util.concurrent.Callable;
import java.util.concurrent.Executors;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;
import org.junit.jupiter.params.provider.ValueSource;
import target.api.*;
import target.operators.OperatorsAST;
import target.operators.OperatorsParser;
import target.power.PowerAST;
import target.power.PowerParser;

class SemanticsTest {
    static Stream<Arguments> numbers() {
        return Stream.of(Arguments.of("0", 0), Arguments.of("-17", -17), Arguments.of("+17", 17),
            Arguments.of("0004", 4), Arguments.of("2147483647", Integer.MAX_VALUE),
            Arguments.of("-2147483648", Integer.MIN_VALUE), Arguments.of("1.5", null),
            Arguments.of("1.", null), Arguments.of(".5", null), Arguments.of("1e2", null),
            Arguments.of("-1e-2", null), Arguments.of("2147483648", null), Arguments.of("-2147483649", null));
    }

    @ParameterizedTest @MethodSource("numbers")
    void numberRecognitionAndIntMappingAreSeparate(String input, Integer expected) {
        var recognized = OperatorsParser.parse(input, ParseOptions.RECOGNIZE);
        assertTrue(recognized.ok()); assertEquals(input.length(), recognized.consumedCp());
        if (expected == null) {
            var failure = assertThrows(MappingException.class, () -> OperatorsParser.parse(input));
            assertTrue(failure.syntax().ok());
            assertEquals(new Span(0, input.length()), failure.span());
            assertInstanceOf(NumberFormatException.class, failure.getCause());
        } else assertEquals(expected.doubleValue(), Evaluator.evaluate(OperatorsParser.parse(input).ast().orElseThrow()));
    }

    @ParameterizedTest @ValueSource(strings = {"", "+", "-", ".", "+.", "1+", "2^"})
    void missingRequiredNumberNeverDefaultsToZero(String input) {
        assertFalse(OperatorsParser.parse(input).ok());
        assertTrue(OperatorsParser.parse(input).ast().isEmpty());
        assertFalse(PowerParser.parse(input).ok());
    }

    @Test void leftAssociativeListsAndPrecedenceRemainDistinct() {
        var r = OperatorsParser.parse("10-3-2*4");
        var root = (OperatorsAST.Binary) r.ast().orElseThrow();
        assertEquals(List.of("-", "-"), root.op());
        assertEquals(2, root.right().size());
        var product = (OperatorsAST.Binary) root.right().get(1);
        assertEquals(List.of("*"), product.op());
        assertEquals(-1.0, Evaluator.evaluate(root));
    }

    @Test void rightRecursionHasZeroOrOneRightPerLevel() {
        var r = PowerParser.parse("2^3^2");
        var root = (PowerAST.Power) r.ast().orElseThrow();
        assertEquals("2", root.left()); assertEquals(List.of("^"), root.op());
        var right = root.right().getFirst();
        assertEquals("3", right.left()); assertEquals(1, right.right().size());
        assertEquals("2", right.right().getFirst().left());
        assertTrue(right.right().getFirst().right().isEmpty());
        assertEquals(512.0, Evaluator.evaluate(root));
        assertEquals(new Span(0, 5), r.nodeSpans().get(root));
        assertEquals(new Span(2, 5), r.nodeSpans().get(right));
        assertEquals(new Span(4, 5), r.nodeSpans().get(right.right().getFirst()));
        assertEquals(List.of("left", "op", "left", "op", "left", "right", "right"),
            r.captures().stream().map(Capture::name).toList());
    }

    @Test void chainTriviaAndTransparentParenthesesMatchSourceSpans() {
        var r = OperatorsParser.parse(" 1 + 2*3 ");
        var root = (OperatorsAST.Binary) r.ast().orElseThrow();
        var left = (OperatorsAST.Binary) root.left();
        var product = (OperatorsAST.Binary) root.right().getFirst();
        assertEquals(new Span(0, 9), r.nodeSpans().get(root));
        assertEquals(new Span(1, 3), r.nodeSpans().get(left));
        assertEquals(new Span(1, 3), r.nodeSpans().get(left.left()));
        assertEquals(new Span(5, 9), r.nodeSpans().get(product));
        assertEquals(new Span(5, 6), r.nodeSpans().get(product.left()));
        assertEquals(new Span(7, 9), r.nodeSpans().get(product.right().getFirst()));
        var parenthesized = OperatorsParser.parse("((7))");
        var outer = (OperatorsAST.Binary) parenthesized.ast().orElseThrow();
        var inner = ((OperatorsAST.Binary) outer.left()).left();
        assertEquals(new Span(1, 4), parenthesized.nodeSpans().get(inner));
    }

    @Test void groupCaptureRetainsCommentWhileNumericRecipeUsesTokenSpan() {
        var r = OperatorsParser.parse("2/*😀*/+3");
        assertEquals(5.0, Evaluator.evaluate(r.ast().orElseThrow()));
        assertEquals(new Span(0, 6), r.captures().getFirst().span());
        assertEquals("value", r.captures().getFirst().name());
        var p = PowerParser.parse("2/*😀*/^3");
        assertEquals("2/*😀*/", ((PowerAST.Power) p.ast().orElseThrow()).left());
        assertEquals(8.0, Evaluator.evaluate(p.ast().orElseThrow()));
        assertEquals(8, p.consumedCp());
    }

    @ParameterizedTest @ValueSource(strings = {" ", "\t", "\r", "\n", "\u000b", "\f", "/*😀*/", "//😀\r\n", "//😀\n", "//😀\r"})
    void javaStyleTriviaAtChainAndHelperBoundaries(String trivia) {
        String input = trivia + "1" + trivia + "+" + trivia + "2" + trivia;
        var r = OperatorsParser.parse(input);
        assertTrue(r.ok()); assertEquals(3.0, Evaluator.evaluate(r.ast().orElseThrow()));
        assertEquals(input.codePointCount(0, input.length()), r.consumedCp());
    }

    @Test void triviaDoesNotEnterAtomicNumbersOrBroadenWhitespace() {
        for (String input : List.of("1 2", "- 2", "1 .2", "1e 2", "\u00a01", "\u20031"))
            assertFalse(OperatorsParser.parse(input).ok(), input);
        assertTrue(OperatorsParser.parse("1//tail").ok());
        assertTrue(OperatorsParser.parse("1/* outer /* inner */").ok()); // 非入れ子。
        var unterminated = OperatorsParser.parse("1/*😀");
        assertFalse(unterminated.ok()); assertEquals(1, unterminated.consumedCp());
        assertEquals(4, unterminated.diagnostics().farthestCp());
        assertTrue(unterminated.diagnostics().expected().contains("'*/'"));
    }

    @Test void failureDiagnosticsRetainFarthestExpectedAndDeepestRule() {
        var missing = OperatorsParser.parse("1+");
        assertFalse(missing.ok()); assertEquals(1, missing.consumedCp());
        assertEquals(2, missing.diagnostics().farthestCp());
        assertEquals(List.of("NUMBER", "'('"), missing.diagnostics().expected());
        assertEquals("NumericLiteral.group", missing.diagnostics().deepestRule());
        var parenthesis = OperatorsParser.parse("(1+2");
        assertEquals(4, parenthesis.diagnostics().farthestCp());
        assertTrue(parenthesis.diagnostics().expected().contains("')'"));
        var unicode = OperatorsParser.parse("1+/*😀*/)");
        assertEquals(7, unicode.diagnostics().farthestCp());
        assertTrue(unicode.diagnostics().expected().contains("NUMBER"));
    }

    @Test void incompleteExponentRollsBackButKeepsLocalDiagnostic() {
        var full = OperatorsParser.parse("1e+");
        assertFalse(full.ok()); assertEquals(1, full.consumedCp());
        assertEquals(3, full.diagnostics().farthestCp());
        var prefix = OperatorsParser.parse("1e+", ParseOptions.PREFIX);
        assertTrue(prefix.ok()); assertEquals(1.0, Evaluator.evaluate(prefix.ast().orElseThrow()));
        assertEquals(3, prefix.diagnostics().farthestCp());
    }

    @Test void failedBranchesNeverPerformNumericMapping() {
        for (String input : List.of("(2147483648", "1+(2147483648", "1+1e2)")) {
            var result = assertDoesNotThrow(() -> OperatorsParser.parse(input));
            assertFalse(result.ok()); assertTrue(result.ast().isEmpty()); assertTrue(result.captures().isEmpty());
        }
        var prefix = OperatorsParser.parse("1+(2147483648", ParseOptions.PREFIX);
        assertEquals(1.0, Evaluator.evaluate(prefix.ast().orElseThrow()));
        assertEquals(3, prefix.captures().size());
        assertThrows(MappingException.class, () -> OperatorsParser.parse("1+2147483648"));
    }

    @Test void equalRecordsHaveIndependentIdentitySpansAndStableResults() {
        var r = OperatorsParser.parse("1+1");
        var root = (OperatorsAST.Binary) r.ast().orElseThrow();
        var a = ((OperatorsAST.Binary) root.left()).left();
        var b = ((OperatorsAST.Binary) root.right().getFirst()).left();
        assertEquals(a, b); assertNotSame(a, b);
        assertEquals(new Span(0, 1), r.nodeSpans().get(a));
        assertEquals(new Span(2, 3), r.nodeSpans().get(b));
        assertEquals(5, r.nodeSpans().size());
        String json = Json.canonical(r);
        OperatorsParser.parse("3+4");
        assertEquals(json, Json.canonical(r));
        assertThrows(UnsupportedOperationException.class, () -> r.nodeSpans().clear());
        assertThrows(UnsupportedOperationException.class, () -> root.op().clear());
        assertThrows(UnsupportedOperationException.class, () -> r.captures().clear());
    }

    @Test void jsonEscapesControlsQuotesBackslashesAndSurrogates() {
        String value = "\"\\\b\f\n\r\t\u0000😀\ud800";
        assertEquals(value, JsonReader.read(Json.write(value)));
        assertEquals("{\"type\":\"Power\",\"span\":[0,1],\"fields\":{\"left\":\"2\",\"op\":[],\"right\":[]}}",
            Json.canonical(PowerParser.parse("2")));
    }

    @Test void sessionsAreIsolatedAcrossConcurrentParses() throws Exception {
        try (var executor = Executors.newFixedThreadPool(4)) {
            var tasks = java.util.stream.IntStream.range(0, 100).<Callable<Double>>mapToObj(i ->
                () -> i % 2 == 0 ? Evaluator.evaluate(OperatorsParser.parse("1+2*3").ast().orElseThrow())
                    : Evaluator.evaluate(PowerParser.parse("2^3^2").ast().orElseThrow())).toList();
            var results = executor.invokeAll(tasks);
            for (int i = 0; i < results.size(); i++) assertEquals(i % 2 == 0 ? 7.0 : 512.0, results.get(i).get());
        }
    }

    @Test void longFlatInputHasFlatOperandLists() {
        var r = OperatorsParser.parse("1+".repeat(511) + "1");
        var root = (OperatorsAST.Binary) r.ast().orElseThrow();
        assertEquals(511, root.op().size()); assertEquals(511, root.right().size());
        assertEquals(512.0, Evaluator.evaluate(root));
        assertEquals(1023, r.consumedCp());
    }
}
