package target;

import static org.junit.jupiter.api.Assertions.*;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import java.util.stream.IntStream;
import java.util.stream.Stream;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;
import target.api.*;
import target.operators.OperatorsParser;
import target.operators.OperatorsAST;
import target.power.PowerParser;
import target.power.PowerAST;

class CorpusTest {
    @SuppressWarnings("unchecked")
    static List<Map<String, Object>> rows(String resource) throws Exception {
        try (var stream = CorpusTest.class.getResourceAsStream(resource)) {
            assertNotNull(stream, resource);
            return (List<Map<String, Object>>) JsonReader.read(new String(stream.readAllBytes(), StandardCharsets.UTF_8));
        }
    }

    static ParseResult<?> parse(String grammar, String input, ParseOptions options) {
        return grammar.equals("operators") ? OperatorsParser.parse(input, options) : PowerParser.parse(input, options);
    }

    @TestFactory Stream<DynamicTest> operators() throws Exception { return corpus("operators", 24); }
    @TestFactory Stream<DynamicTest> power() throws Exception { return corpus("power", 28); }

    private Stream<DynamicTest> corpus(String grammar, int count) throws Exception {
        var rows = rows("/corpus/" + grammar + ".json");
        var golden = rows("/golden/" + grammar + ".json");
        assertEquals(count, rows.size()); assertEquals(count, golden.size());
        return IntStream.range(0, count).mapToObj(i -> DynamicTest.dynamicTest(grammar + "[" + i + "] " + rows.get(i).get("input"), () -> {
            var row = rows.get(i);
            String input = (String) row.get("input");
            boolean accepted = row.containsKey("value") && !row.containsKey("modes");
            boolean mappingFailure = grammar.equals("operators") && input.equals("1e2/4-0.5");
            ParseResult<?> recognition = parse(grammar, input, ParseOptions.RECOGNIZE);
            assertEquals(accepted, recognition.ok(), "元文法の全入力受理");
            assertTrue(recognition.ast().isEmpty()); assertTrue(recognition.nodeSpans().isEmpty());
            ParseResult<?> result;
            MappingException error = null;
            if (mappingFailure) {
                error = assertThrows(MappingException.class, () -> parse(grammar, input, ParseOptions.DEFAULT));
                result = error.syntax();
                assertInstanceOf(NumberFormatException.class, error.getCause());
                assertTrue(result.ok()); assertTrue(result.ast().isEmpty());
            } else {
                result = parse(grammar, input, ParseOptions.DEFAULT);
                assertEquals(accepted, result.ok());
                if (accepted) {
                    Object ast = result.ast().orElseThrow();
                    double value = ast instanceof OperatorsAST operators ? Evaluator.evaluate(operators) : Evaluator.evaluate((PowerAST) ast);
                    assertEquals((Double) row.get("value"), value, 0.0);
                    assertFalse(result.nodeSpans().isEmpty());
                } else {
                    assertTrue(result.ast().isEmpty()); assertTrue(result.captures().isEmpty());
                }
            }
            if (accepted) {
                assertEquals(input.codePointCount(0, input.length()), result.consumedCp());
                assertEquals(result.consumedCp(), result.matchedCp());
            }
            assertEquals(recognition.consumedCp(), result.consumedCp());
            assertEquals(recognition.matchedCp(), result.matchedCp());
            assertEquals(recognition.diagnostics(), result.diagnostics());
            assertEquals(golden.get(i), JsonReader.read(Json.write(Json.result(input, result, error))));
        }));
    }
}
