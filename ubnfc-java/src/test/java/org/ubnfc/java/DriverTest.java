package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.IrFixture.*;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.ubnfc.java.emit.JavaEmitter;
import org.ubnfc.java.ir.IrReader;
import org.ubnfc.java.ir.Json;

@Timeout(60)
class DriverTest {
    private static final Path OPERATORS = Path.of("src/test/resources/ir/operators.ir.json");
    private static CompilerHarness operators;
    @BeforeAll static void compile() throws Exception {
        operators = new CompilerHarness(IrReader.read(OPERATORS), "custom.driverprobe");
    }
    @AfterAll static void close() throws Exception { operators.close(); }
    private static Map<String, Object> request(Object id, String grammar, String entry, String input, boolean lexical, boolean scope) {
        return obj("id", id, "grammar", grammar, "entry", entry, "input", input, "options", obj("lexical", lexical, "scope", scope));
    }
    private static Map<String, Object> request(String input) { return request("test", "Operators", null, input, true, true); }
    private static List<Map<String, Object>> exchange(CompilerHarness h, List<String> requests) throws Exception {
        var process = new ProcessBuilder("java", "-cp", h.directory.resolve("classes").toString(), h.pkg + ".driver.Driver")
            .redirectError(h.directory.resolve("driver.stderr").toFile()).start();
        var result = new ArrayList<Map<String, Object>>();
        try {
            try (var input = new OutputStreamWriter(process.getOutputStream(), StandardCharsets.UTF_8);
                 var output = new BufferedReader(new InputStreamReader(process.getInputStream(), StandardCharsets.UTF_8))) {
                for (String request : requests) {
                    input.write(request + "\n"); input.flush();
                    // Read before EOF: every response must be flushed by the driver.
                    String line = output.readLine(); assertNotNull(line); result.add(Json.object(Json.read(line)));
                }
            }
            assertEquals(0, process.waitFor());
            assertEquals("", Files.readString(h.directory.resolve("driver.stderr")));
            return result;
        } finally { process.destroyForcibly(); }
    }
    private static Map<String, Object> exchange(CompilerHarness h, Map<String, Object> request) throws Exception {
        return exchange(h, List.of(Json.write(request))).getFirst();
    }
    private static int integer(Object value) { return ((Number) value).intValue(); }

    @Test void framingIdsErrorsAndContinuation() throws Exception {
        var id = Json.object(Json.read("{\"nested\":[1.2300,null,\"😀\\n\"],\"large\":9007199254740993}"));
        id.put("surrogate", String.valueOf((char) 0xd800));
        var valid = request(id, "Operators", "Term", "2*3", true, true);
        var rows = exchange(operators, List.of("{bad", "{\"id\":1,\"id\":2}", "{\"id\":1} true", "[]",
            Json.write(request(null, "Unknown", null, "1", true, true)),
            Json.write(request(null, "Operators", "Unknown", "1", true, true)),
            Json.write(request(null, "Operators", "", "1", true, true)),
            Json.write(valid).replace(String.valueOf((char) 0xd800), "\\" + "uD800")));
        for (var row : rows.subList(0, rows.size() - 1)) { assertTrue(row.get("error") instanceof String); assertNull(row.get("id")); }
        var result = rows.getLast(); assertEquals(id, result.get("id"));
        assertEquals(true, result.get("allConsumed")); assertEquals(6.0, ((Number) result.get("value")).doubleValue());
    }
    @Test void unicodeCapturesPrefixAndFullInputObservations() throws Exception {
        String input = "2/*😀*/+3";
        var result = exchange(operators, request(input));
        assertEquals(true, result.get("accepted")); assertEquals(true, result.get("allConsumed"));
        assertEquals(8, integer(result.get("consumed"))); assertEquals(8, integer(result.get("matched")));
        for (Object item : Json.array(result.get("captures"))) {
            var capture = Json.object(item); var span = Json.array(capture.get("span"));
            assertEquals(input.substring(input.offsetByCodePoints(0, integer(span.get(0))), input.offsetByCodePoints(0, integer(span.get(1)))), capture.get("text"));
        }
        assertEquals(Json.read(Files.readString(Path.of("src/test/resources/golden/operators-decimal.json"))),
            exchange(operators, request("1e2/4-0.5")).get("ast"));
        var partial = exchange(operators, request("1+"));
        assertEquals(true, partial.get("accepted")); assertEquals(false, partial.get("allConsumed"));
        assertEquals(1, integer(partial.get("consumed"))); assertNull(partial.get("ast")); assertNull(partial.get("value"));
        assertFalse(Json.array(partial.get("prefixCaptures")).isEmpty());
        assertNotNull(Json.object(Json.array(partial.get("prefixCaptures")).getFirst()).get("binding"));
        assertEquals("succeeded", partial.get("parsedStatus"));
        var limited = request("1"); limited.put("options", obj("lexical", true, "scope", true, "maxDepth", 1));
        assertEquals(false, exchange(operators, limited).get("accepted"));
        var diagnostic = Json.object(Json.array(partial.get("diagnostics")).getFirst());
        assertEquals("trailing_input", diagnostic.get("kind")); assertEquals(1, integer(diagnostic.get("offset")));
        assertEquals(List.of("'('", "NUMBER"), diagnostic.get("farthestExpected"));
        assertEquals(List.of("end of input"), diagnostic.get("expected"));
    }
    @Test void numericFoldAndNonFiniteValues() throws Exception {
        var rows = exchange(operators, List.of(Json.write(request("1/0")), Json.write(request("10-3-2")),
            Json.write(request("1e2/4-0.5")), Json.write(request(null, "Operators", "NumericLiteral", "12", true, true))));
        assertTrue(rows.get(0).get("error").toString().contains("Non-finite"));
        assertEquals(5.0, ((Number) rows.get(1).get("value")).doubleValue());
        assertEquals(24.5, ((Number) rows.get(2).get("value")).doubleValue());
        assertEquals(true, rows.get(3).get("allConsumed")); assertEquals(12.0, ((Number) rows.get(3).get("value")).doubleValue());
    }
    @Test void scopeEventsAndOptionsDoNotChangeParsing() throws Exception {
        var f = new IrFixture();
        String declared = f.literal("Declare", "😀"), used = f.literal("Use", "z");
        String declaration = f.capture(declared, "name", "text", "scalar", obj("kind", "text"));
        String reference = f.capture(used, "name", "text", "scalar", obj("kind", "text"));
        f.rule("Declare", declared); f.rule("Use", used);
        f.rule("Root", f.seq("Root", f.ref("Root", "Declare"), f.ref("Root", "Use")));
        f.effects.add(obj("ruleId", "Declare", "mode", "lexical", "events", List.of(obj("when", "successAfterLeave", "action", "declare", "captureSiteId", declaration, "name", "var"))));
        f.effects.add(obj("ruleId", "Use", "mode", "lexical", "events", List.of(obj("when", "afterDeclarations", "action", "use", "captureSiteId", reference, "name", "var"))));
        try (var h = f.compile()) {
            var on = exchange(h, request(null, "Probe", null, "😀z", true, true));
            var off = exchange(h, request(null, "Probe", null, "😀z", false, false));
            assertEquals(Json.read("[{\"name\":\"😀\",\"sourceOffset\":0}]"), on.get("declarations"));
            assertEquals(Json.read("[{\"name\":\"z\",\"offset\":1,\"length\":1}]"), on.get("references"));
            var diagnostic = Json.object(Json.array(on.get("diagnostics")).getFirst());
            assertEquals("WARNING", diagnostic.get("kind")); assertEquals(1, integer(diagnostic.get("offset")));
            assertEquals(List.of(), diagnostic.get("expected"));
            for (String key : List.of("captures", "prefixCaptures", "lexical", "declarations", "references", "scope", "scopeDepth")) { assertFalse(off.containsKey(key)); on.remove(key); }
            assertEquals(on, off); assertEquals(true, on.get("allConsumed"));
        }
    }
    @Test void mappingFailurePreservesSyntax() throws Exception {
        var f = new IrFixture(); String text = f.literal("Root", "2147483648");
        String site = f.capture(text, "value", "number", "scalar", obj("kind", "number", "java", "int32Exact", "rust", "lexeme")); f.rule("Root", text);
        f.mapping("Root", "Number", List.of(field("value", obj("kind", "number", "java", "int32Exact", "rust", "lexeme"), "scalar")), List.of(recipeField("value", site, "number", "scalar")));
        try (var h = f.compile()) {
            var result = exchange(h, request(null, "Probe", null, "2147483648", true, true));
            assertEquals(true, result.get("accepted")); assertEquals(true, result.get("allConsumed"));
            assertEquals(10, integer(result.get("consumed"))); assertEquals(10, integer(result.get("matched")));
            assertNull(result.get("ast")); assertNull(result.get("value"));
            assertTrue(result.get("mappingMessage").toString().contains("NUMBER cannot be mapped to int"));
            assertTrue(result.get("mappingError").toString().startsWith("java.lang.NumberFormatException:"));
            assertEquals("java.lang.NumberFormatException", result.get("mappingErrorKind"));
            assertEquals(1, Json.array(result.get("captures")).size());
        }
    }
    @Test void everyGrammarAndItsOwnRootAreDispatched() throws Exception {
        var f = new IrFixture();
        f.rule("Root", f.literal("Root", "a")); f.rule("OtherRoot", f.literal("OtherRoot", "b")); f.rule("NoRoot", f.literal("NoRoot", "c"));
        f.mapping("Root", "Primary", List.of(), List.of()); f.mapping("OtherRoot", "Secondary", List.of(), List.of());
        var primary = Json.object(Json.array(f.root.get("grammarUnits")).getFirst());
        var other = Json.object(primary); other.put("id", "grammar:Other"); other.put("name", "Other");
        var noRoot = Json.object(primary); noRoot.put("id", "grammar:NoRoot"); noRoot.put("name", "NoRoot");
        var empty = Json.object(primary); empty.put("id", "grammar:Empty"); empty.put("name", "Empty");
        f.root.put("grammarUnits", List.of(primary, other, noRoot, empty));
        f.symbols.get(1).put("grammarId", "grammar:Other"); f.symbols.get(1).put("name", "Root");
        f.symbols.get(2).put("grammarId", "grammar:NoRoot");
        f.rules.get(1).put("annotations", List.of(obj("name", "root", "arguments", Map.of(), "source", f.symbols.get(1).get("source"))));
        try (var h = f.compile()) {
            var rows = exchange(h, List.of(Json.write(request(0, "Probe", null, "a", true, true)),
                Json.write(request(1, "Other", null, "b", true, true)), Json.write(request(2, "Other", "Root", "b", true, true)),
                Json.write(request(3, "NoRoot", "NoRoot", "c", true, true)), Json.write(request(4, "NoRoot", null, "c", true, true)),
                Json.write(request(5, "Empty", null, "", true, true))));
            for (var row : rows.subList(0, 4)) assertEquals(true, row.get("allConsumed"), row.toString());
            assertEquals("Primary", Json.object(rows.get(0).get("ast")).get("type"));
            assertEquals("Secondary", Json.object(rows.get(1).get("ast")).get("type"));
            assertTrue(rows.get(4).get("error") instanceof String);
            assertTrue(rows.get(5).get("error") instanceof String);
        }
    }
    @Test void unsupportedEvaluationIsAbsentAndLeafWithoutTextDoesNotCrash() throws Exception {
        var f = new IrFixture(); f.rule("Root", f.literal("Root", "c")); f.mapping("Root", "Leaf", List.of(), List.of());
        try (var h = f.compile()) {
            var response = exchange(h, request(null, "Probe", null, "c", true, true));
            assertEquals(true, response.get("allConsumed")); assertFalse(response.containsKey("error"));
            assertFalse(response.containsKey("value")); assertEquals(List.of(), response.get("nodes"));
        }
    }
    @Test void emissionIsDefaultAndCanBeDisabledByApiOrCli() throws Exception {
        var ir = IrReader.read(OPERATORS);
        assertTrue(new JavaEmitter(ir, "test.flags").sources().containsKey("test/flags/driver/Driver.java"));
        assertFalse(new JavaEmitter(ir, "test.flags", false).sources().containsKey("test/flags/driver/Driver.java"));
        int counter = 0;
        for (List<String> flags : List.of(List.of("--emit-driver"), List.of("--emit-driver", "true"), List.of("--emit-driver", "false"), List.of("--no-emit-driver"))) {
            Path out = operators.directory.resolve("flags" + counter++);
            var args = new ArrayList<>(flags); args.addAll(List.of("--ir", OPERATORS.toString(), "--out", out.toString(), "--package", "test.flags"));
            Main.main(args.toArray(String[]::new));
            assertEquals(counter <= 2, Files.isRegularFile(out.resolve("test/flags/driver/Driver.java")));
        }
        assertThrows(IllegalArgumentException.class, () -> Main.main(new String[]{"--emit-driver", "maybe"}));
        assertThrows(IllegalArgumentException.class, () -> Main.main(new String[]{"--emit-driver", "--no-emit-driver"}));
    }
}
