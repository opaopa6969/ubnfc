package org.ubnfc.p4;

import static org.junit.jupiter.api.Assertions.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestFactory;
import org.ubnfc.p4.generated.TinyExpressionP4AST;
import org.ubnfc.p4.generated.TinyExpressionP4Parser;
import org.ubnfc.p4.generated.api.Json;
import org.ubnfc.p4.generated.api.ParseOptions;

class P4Test {
    private static final Path FIXTURES = Path.of("src/test/resources/fixtures");
    @TestFactory Stream<DynamicTest> fixtures() throws Exception {
        try (var paths = Files.list(FIXTURES)) {
            return paths.sorted().toList().stream().map(path -> DynamicTest.dynamicTest(path.getFileName().toString(), () -> {
                String text = Files.readString(path);
                String name = path.getFileName().toString();
                boolean comparison = name.startsWith("comparison-heavy");
                boolean invalid = List.of("complex-half.tiny", "complex-tail.tiny", "invalid-syntax.tiny").contains(name);
                for (boolean ast : new boolean[]{false, true}) {
                    var result = TinyExpressionP4Parser.parse(text, P4Scanners.options(ast));
                    // comparison-heavy の root 拒否は既知の契約不足。成功として数えない。
                    assertEquals(!comparison && !invalid, result.ok(), result.diagnostics().toString());
                    if (result.ok()) {
                        assertEquals(text.codePointCount(0, text.length()), result.consumed());
                        assertEquals(result.consumed(), result.matched());
                        if (ast) assertEquals(Files.readString(Path.of("src/test/resources/golden", name + ".json")).stripTrailing(), Json.canonical(result));
                        else assertTrue(result.ast().isEmpty());
                    } else if (comparison) assertEquals(5, result.diagnostics().offset());
                }
                if (comparison) {
                    var bool = TinyExpressionP4Parser.parseEntry("TinyExpressionP4", "BooleanExpression", text, P4Scanners.options(true));
                    assertTrue(bool.ok(), bool.diagnostics().toString());
                    assertEquals(text.codePointCount(0, text.length()), bool.consumed());
                    assertEquals(Files.readString(Path.of("src/test/resources/golden/boolean-entry", name + ".json")).stripTrailing(), Json.canonical(bool));
                }
            }));
        }
    }
    @Test void reviewedComplexShape() throws Exception {
        var result = TinyExpressionP4Parser.parse(Files.readString(FIXTURES.resolve("complex.tiny")), P4Scanners.options(true));
        var formula = (TinyExpressionP4AST.FormulaExpr) result.ast().orElseThrow();
        assertEquals(2, formula.declarations().size());
        assertEquals("price", ((TinyExpressionP4AST.NumberVariableDeclarationExpr) formula.declarations().get(0)).varName());
        assertEquals("country", ((TinyExpressionP4AST.StringVariableDeclarationExpr) formula.declarations().get(1)).varName());
        assertEquals(1, formula.methods().size());
        var counts = new java.util.HashMap<String, Integer>();
        count(Json.canonicalValue(result), counts);
        assertEquals(1, counts.get("IfExpr"));
        assertEquals(2, counts.get("BranchExpressionExpr"));
        assertEquals(1, counts.get("NumberMatchExpr"));
        assertEquals(2, counts.get("NumberCaseExpr"));
        assertEquals(1, counts.get("NumberDefaultCaseExpr"));
        assertEquals(1, counts.get("NumberMethodDeclarationExpr"));
    }
    @Test void reviewedScaledShapes() throws Exception {
        for (int factor : new int[]{4, 16, 64}) {
            var tree = (Map<?, ?>) Json.read(Files.readString(Path.of(
                "src/test/resources/golden/complex-x" + factor + ".tiny.json")));
            var fields = (Map<?, ?>) tree.get("fields");
            assertEquals(2 * factor, ((List<?>) fields.get("declarations")).size());
            assertEquals(factor, ((List<?>) fields.get("methods")).size());
            var counts = new java.util.HashMap<String, Integer>();
            count(tree, counts);
            assertEquals(factor, counts.get("IfExpr"));
            assertEquals(2 * factor, counts.get("BranchExpressionExpr"));
            assertEquals(factor, counts.get("NumberMatchExpr"));
            assertEquals(2 * factor, counts.get("NumberCaseExpr"));
            assertEquals(factor, counts.get("NumberDefaultCaseExpr"));
        }
    }
    private static void count(Object value, Map<String, Integer> counts) {
        if (value instanceof Map<?, ?> map) {
            if (map.get("type") instanceof String type && map.containsKey("span")) counts.merge(type, 1, Integer::sum);
            map.values().forEach(v -> count(v, counts));
        } else if (value instanceof List<?> list) list.forEach(v -> count(v, counts));
    }
    @Test void orderedChoiceGapIsExplicit() {
        assertFalse(TinyExpressionP4Parser.parse("1>=1", P4Scanners.options(true)).ok());
        assertTrue(TinyExpressionP4Parser.parseEntry("TinyExpressionP4", "BooleanExpression", "1>=1", P4Scanners.options(true)).ok());
    }
    @Test void memoDoesNotChangeCanonicalTree() throws Exception {
        String source = Files.readString(FIXTURES.resolve("complex.tiny"));
        var off = new ParseOptions(true, true, false, false, P4Scanners.ALL);
        assertEquals(Json.canonical(TinyExpressionP4Parser.parse(source, off)),
            Json.canonical(TinyExpressionP4Parser.parse(source, P4Scanners.options(true))));
    }
    @Test void generatedSourcesAreByteIdentical() throws Exception {
        var process = new ProcessBuilder("bash", "scripts/generate.sh", "--check").redirectErrorStream(true).start();
        String output = new String(process.getInputStream().readAllBytes(), java.nio.charset.StandardCharsets.UTF_8);
        assertEquals(0, process.waitFor(), output);
    }
}
