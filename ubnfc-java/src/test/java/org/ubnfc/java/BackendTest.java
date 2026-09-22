package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.CompilerHarness.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestFactory;
import org.ubnfc.java.emit.JavaEmitter;
import org.ubnfc.java.ir.IrReader;
import org.ubnfc.java.ir.Json;
import org.ubnfc.java.ir.Schema;

class BackendTest {
    private static final Path TARGET = Path.of("../examples/handwritten-target/java/src/test/resources");
    private static final Path TINY = Path.of("../ir/examples/tinycalc.ir.json");
    private static final Map<String, CompilerHarness> harnesses = new LinkedHashMap<>();
    @BeforeAll static void compile() throws Exception {
        for (String grammar : List.of("tinycalc", "operators", "power")) {
            Path path = grammar.equals("tinycalc") ? TINY : Path.of("src/test/resources/ir/" + grammar + ".ir.json");
            var ir = IrReader.read(path);
            new Schema(Json.read(Files.readString(Path.of("../ir/schema/grammar-ir.v1.schema.json"))), false).validate(Json.read(Files.readString(path)));
            harnesses.put(grammar, new CompilerHarness(ir, "test.generated." + grammar));
        }
    }
    @AfterAll static void close() throws Exception { for (var harness : harnesses.values()) harness.close(); }
    private static List<?> rows(Path path) throws Exception { return Json.array(Json.read(Files.readString(path))); }
    @TestFactory Stream<DynamicTest> corpus() throws Exception {
        var tests = new ArrayList<DynamicTest>();
        for (String grammar : List.of("operators", "power")) {
            var rows = rows(TARGET.resolve("corpus/" + grammar + ".json"));
            var golden = rows(TARGET.resolve("golden/" + grammar + ".json"));
            assertEquals(grammar.equals("operators") ? 24 : 28, rows.size());
            CompilerHarness h = harnesses.get(grammar);
            for (int i = 0; i < rows.size(); i++) {
                var row = Json.object(rows.get(i)); var expected = Json.object(golden.get(i));
                String input = (String) row.get("input"); boolean accepted = row.containsKey("value") && !row.containsKey("modes");
                tests.add(DynamicTest.dynamicTest(grammar + ": " + input, () -> {
                    Object result = h.parse(input), recognition = h.parse(input, h.options.getField("RECOGNIZE").get(null));
                    assertEquals(accepted, ok(result), "full-input acceptance"); assertEquals(accepted, ok(recognition));
                    assertEquals(get(recognition, "consumedCp"), get(result, "consumedCp"));
                    assertEquals(get(recognition, "matchedCp"), get(result, "matchedCp"));
                    assertEquals(get(recognition, "diagnostics"), get(result, "diagnostics"));
                    assertTrue(((java.util.Optional<?>) get(recognition, "ast")).isEmpty());
                    if (accepted) {
                        assertEquals(input.codePointCount(0, input.length()), get(result, "consumedCp"));
                        assertEquals(get(result, "consumedCp"), get(result, "matchedCp"));
                        assertEquals(((Number) row.get("value")).doubleValue(), evaluate(h.canonical(result)), 0.0);
                    }
                    Object actual = h.canonical(result);
                    if (grammar.equals("operators") && input.equals("1e2/4-0.5")) {
                        assertEquals(Boolean.TRUE, expected.get("mappingError"));
                        assertNotNull(actual); // Correct String mapping removes the handwritten int failure.
                        assertEquals(Json.read(Files.readString(Path.of("src/test/resources/golden/operators-decimal.json"))), actual);
                    } else {
                        Object ast = expected.get("ast");
                        if (grammar.equals("operators") && ast != null) ast = correctedNumbers(ast, Json.array(expected.get("captures")), input);
                        assertEquals(ast, actual, "canonical AST vs handwritten target");
                    }
                    var diagnostic = Json.object(expected.get("diagnostics"));
                    assertEquals(((Number) diagnostic.get("farthestCp")).intValue(), get(get(result, "diagnostics"), "farthestCp"));
                    assertLegacyExpected(Json.array(diagnostic.get("expected")), result);
                    List<?> expectedCaptures = Json.array(expected.get("captures")), actualCaptures = (List<?>) get(result, "captures");
                    assertEquals(expectedCaptures.size(), actualCaptures.size());
                    for (int c = 0; c < actualCaptures.size(); c++) {
                        var capture = Json.object(expectedCaptures.get(c)); Object actualCapture = actualCaptures.get(c);
                        assertEquals(capture.get("name"), get(actualCapture, "name"));
                        assertEquals(capture.get("span"), List.of(new java.math.BigDecimal(get(get(actualCapture, "span"), "start").toString()), new java.math.BigDecimal(get(get(actualCapture, "span"), "end").toString())));
                    }
                }));
            }
        }
        return tests.stream();
    }
    private static void assertLegacyExpected(List<?> expected, Object result) throws Exception {
        Object diagnostic = get(result, "diagnostics");
        // The handwritten golden's expected is the old farthest accumulator, with an
        // injected EOF hint. Keep that file intact and check the two new API fields separately.
        assertEquals(expected.stream().map(Object::toString).filter(s -> !s.equals("EOF")).sorted().toList(),
            get(diagnostic, "farthestExpected"));
        if (get(diagnostic, "kind").equals("trailing_input")) assertEquals(List.of("end of input"), get(diagnostic, "expected"));
        else assertEquals(get(diagnostic, "farthestExpected"), get(diagnostic, "expected"));
    }
    private static Object correctedNumbers(Object value, List<?> captures, String input) {
        if (value instanceof List<?> list) return list.stream().map(v -> correctedNumbers(v, captures, input)).toList();
        if (!(value instanceof Map<?, ?>)) return value;
        var map = Json.object(value);
        if ("Number".equals(map.get("type"))) {
            var span = Json.array(map.get("span")); int start = ((Number) span.get(0)).intValue(), end = ((Number) span.get(1)).intValue();
            for (Object item : captures) {
                var capture = Json.object(item); if (!"value".equals(capture.get("name"))) continue;
                var extent = Json.array(capture.get("span")); int a = ((Number) extent.get(0)).intValue(), b = ((Number) extent.get(1)).intValue();
                if (a >= start && b <= end) {
                    map.put("fields", Map.of("value", input.substring(input.offsetByCodePoints(0, a), input.offsetByCodePoints(0, b)).strip())); return map;
                }
            }
            fail("Missing value capture");
        }
        map.replaceAll((key, child) -> correctedNumbers(child, captures, input)); return map;
    }
    private static double numeric(String text) {
        int comment = text.indexOf('/'); return Double.parseDouble((comment < 0 ? text : text.substring(0, comment)).strip());
    }
    private static double evaluate(Object ast) {
        var node = Json.object(ast); var fields = Json.object(node.get("fields"));
        if (node.get("type").equals("Number")) return numeric((String) fields.get("value"));
        if (node.get("type").equals("Power")) {
            double left = numeric((String) fields.get("left")); var right = Json.array(fields.get("right"));
            return right.isEmpty() ? left : Math.pow(left, evaluate(right.getFirst()));
        }
        double value = evaluate(fields.get("left")); var ops = Json.array(fields.get("op")); var rights = Json.array(fields.get("right"));
        for (int i = 0; i < ops.size(); i++) {
            double right = evaluate(rights.get(i));
            value = switch ((String) ops.get(i)) { case "+" -> value + right; case "-" -> value - right; case "*" -> value * right; case "/" -> value / right; default -> throw new IllegalArgumentException(); };
        }
        return value;
    }
    @TestFactory Stream<DynamicTest> tinycalc() throws Exception {
        CompilerHarness h = harnesses.get("tinycalc");
        return rows(Path.of("src/test/resources/golden/tinycalc.json")).stream().map(Json::object).map(row -> DynamicTest.dynamicTest("tinycalc: " + row.get("input"), () -> {
            Object result = h.parse((String) row.get("input")); assertEquals(row.get("ok"), ok(result));
            assertEquals(row.get("ast"), h.canonical(result));
            assertEquals(((Number) row.get("offset")).intValue(), get(get(result, "diagnostics"), "offset"));
            assertLegacyExpected(Json.array(row.get("expected")), result);
        }));
    }
    @Test void deterministicSourcesAndUnknownKeys() throws Exception {
        String text = Files.readString(TINY); var emitter = new JavaEmitter(IrReader.read(text), "deterministic.sample");
        var raw = Json.object(Json.read(text)); raw.put("futureMetadata", Map.of("ignored", true));
        assertEquals(emitter.sources(), new JavaEmitter(IrReader.read(Json.write(raw)), "deterministic.sample").sources());
        assertEquals(emitter.sources(), new JavaEmitter(IrReader.read(text), "deterministic.sample").sources());
        assertTrue(emitter.sources().values().stream().noneMatch(s -> s.contains("org.ubnfc.java")));
    }
    @Test void rejectUnsupportedBeforeWriting() throws Exception {
        var raw = Json.object(Json.read(Files.readString(TINY))); raw.put("requiredFeatures", List.of("unknownFutureFeature"));
        assertTrue(assertThrows(IllegalArgumentException.class, () -> IrReader.read(Json.write(raw))).getMessage().contains("unknownFutureFeature"));
        raw.put("requiredFeatures", List.of("eval"));
        assertTrue(assertThrows(IllegalArgumentException.class, () -> IrReader.read(Json.write(raw))).getMessage().contains("eval"));
        raw.put("requiredFeatures", List.of()); raw.put("analysisVersion", "future");
        assertTrue(assertThrows(IllegalArgumentException.class, () -> IrReader.read(Json.write(raw))).getMessage().contains("analysisVersion"));
    }
    @Test void memoOffAndStructuredOccurrenceReplay() throws Exception {
        var h = harnesses.get("power"); String input = "2 /*😀*/ ^3^2";
        Object on = h.parse(input, h.options(true, true, true, true, Map.of()));
        Object off = h.parse(input, h.options(true, true, true, false, Map.of()));
        assertEquals(h.canonical(off), h.canonical(on));
        assertEquals(get(off, "captures"), get(on, "captures")); assertEquals(get(off, "lexical"), get(on, "lexical"));
        assertEquals(get(off, "diagnostics"), get(on, "diagnostics"));
        var lexical = (List<?>) get(on, "lexical"); assertFalse(lexical.isEmpty());
        var ids = new java.util.HashSet<Object>(); for (var item : lexical) assertTrue(ids.add(get(item, "occurrenceId")));
        for (var item : lexical) assertTrue((Long) get(item, "parentOccurrenceId") == -1 || ids.contains(get(item, "parentOccurrenceId")));
    }
    @Test void jsonStrictness() {
        for (String bad : List.of("{\"a\":1,\"a\":2}", "[1,]", "01", "1e", "\"\\x\"", "true false", "\"\n\"")) assertThrows(IllegalArgumentException.class, () -> Json.read(bad), bad);
        String text = "{\"x\":[null,true,12,\"😀\\n\"]}"; assertEquals(Json.read(text), Json.read(Json.write(Json.read(text))));
    }
}
