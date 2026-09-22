package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.IrFixture.obj;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.ubnfc.java.ir.GrammarIr;
import org.ubnfc.java.ir.IrReader;
import org.ubnfc.java.ir.Json;

@Timeout(60)
class LeftRecTest {
    private static final List<String> INPUTS = List.of("1+2*3", "(1+2)*3", "1-2-3", "1+");

    private static List<Map<String, Object>> objects(Map<String, Object> parent, String key) {
        var result = new ArrayList<>(Json.array(parent.get(key)).stream().map(Json::object).toList());
        parent.put(key, result);
        return result;
    }
    private static Map<String, Object> find(List<Map<String, Object>> values, String key, Object value) {
        return values.stream().filter(item -> value.equals(item.get(key))).findFirst().orElseThrow();
    }
    private static final class Fixture {
        final Map<String, Object> root, semantics;
        final List<Map<String, Object>> rules, expressions, captures, recipes, normalizations, ruleAnalyses;
        Fixture(boolean leftrec) throws Exception {
            root = Json.object(Json.read(Files.readString(Path.of("../ir/fixtures/story__code__calc"
                + (leftrec ? "-leftrec" : "") + ".ubnf.ir.json"))));
            semantics = Json.object(root.get("semantics")); root.put("semantics", semantics);
            rules = objects(semantics, "rules"); expressions = objects(semantics, "expressions");
            captures = objects(semantics, "captures"); recipes = objects(semantics, "recipes");
            normalizations = objects(semantics, "normalizations");
            var analysis = Json.object(root.get("analysis")); root.put("analysis", analysis);
            ruleAnalyses = objects(analysis, "rules");
        }
        Map<String, Object> node(Object id) { return Json.object(find(expressions, "id", id).get("node")); }
        Fixture withoutHistory() {
            normalizations.clear();
            ruleAnalyses.forEach(rule -> rule.put("leftRecursion", "none"));
            return this;
        }
        GrammarIr build() { return IrReader.read(Json.write(root)); }
    }

    // D-016 excludes mappings from the left-recursive source. Attach calc's mapping
    // after lowering, without changing executable nodes. Neither fixture has @eval;
    // evaluate() below folds the canonical AST produced by the compiled parser.
    private static Fixture mappedLeftrec() throws Exception {
        var ir = new Fixture(true); var calc = new Fixture(false);
        ir.semantics.put("astTypes", calc.semantics.get("astTypes"));
        ir.root.put("requiredFeatures", calc.root.get("requiredFeatures"));
        for (var original : calc.rules) {
            if (original.get("mappingTypeId") == null) continue;
            String ruleId = original.get("symbolId").toString().replace("TinyCalc::", "TinyCalcLR::");
            var rule = find(ir.rules, "symbolId", ruleId);
            rule.put("mappingTypeId", original.get("mappingTypeId")); rule.put("mappingParams", original.get("mappingParams"));
            var body = ir.node(rule.get("body")); assertEquals("seq", body.get("kind"));
            var items = Json.array(body.get("items"));
            var repeat = ir.node(items.get(1)); assertEquals("repeat", repeat.get("kind"));
            var pair = ir.node(repeat.get("child")); assertEquals("seq", pair.get("kind"));
            var pairItems = Json.array(pair.get("items"));
            var sites = List.of(items.getFirst(), pairItems.getFirst(), pairItems.get(1));
            var recipe = Json.object(find(calc.recipes, "ruleId", original.get("symbolId")));
            recipe.put("ruleId", ruleId);
            var fields = objects(recipe, "fields");
            for (int i = 0; i < 3; i++) {
                String name = List.of("left", "op", "right").get(i);
                var capture = Json.object(calc.captures.stream().filter(c -> c.get("ruleId").equals(original.get("symbolId"))
                    && c.get("name").equals(name)).findFirst().orElseThrow());
                String exprId = sites.get(i).toString(), siteId = exprId + "/capture/0";
                capture.put("ruleId", ruleId); capture.put("exprId", exprId); capture.put("siteId", siteId);
                find(ir.expressions, "id", exprId).put("captureSiteIds", List.of(siteId));
                find(fields, "field", name).put("siteIds", List.of(siteId));
                ir.captures.add(capture);
            }
            ir.recipes.add(recipe);
        }
        return ir;
    }

    private static Object node(String type, int start, int end, Object left, List<String> op, List<Object> right) {
        return obj("type", type, "span", List.of(start, end), "fields", obj("left", left, "op", op, "right", right));
    }
    private static Object number(String value, int start) { return node("MulExpr", start, start + 1, value, List.of(), List.of()); }
    private static List<Object> golden() {
        // Round-trip to the same JSON numeric representation as canonical().
        var expected = new ArrayList<Object>();
        expected.add(node("AddExpr", 0, 5, number("1", 0), List.of("+"), List.of(
            node("MulExpr", 2, 5, "2", List.of("*"), List.of("3")))));
        expected.add(node("AddExpr", 0, 7, node("MulExpr", 0, 7,
            node("AddExpr", 1, 4, number("1", 1), List.of("+"), List.of(number("2", 3))),
            List.of("*"), List.of("3")), List.of(), List.of()));
        expected.add(node("AddExpr", 0, 5, number("1", 0), List.of("-", "-"), List.of(number("2", 2), number("3", 4))));
        expected.add(null);
        return expected.stream().map(value -> Json.read(Json.write(value))).toList();
    }
    private static double evaluate(Object ast) {
        if (ast instanceof String text) return Double.parseDouble(text);
        var fields = Json.object(Json.object(ast).get("fields"));
        double value = evaluate(fields.get("left"));
        var ops = Json.array(fields.get("op")); var rights = Json.array(fields.get("right"));
        assertEquals(ops.size(), rights.size());
        for (int i = 0; i < ops.size(); i++) {
            double right = evaluate(rights.get(i));
            value = switch (ops.get(i).toString()) {
                case "+" -> value + right; case "-" -> value - right;
                case "*" -> value * right; case "/" -> value / right;
                default -> throw new IllegalArgumentException("Unknown operator: " + ops.get(i));
            };
        }
        return value;
    }

    @Test void loweredCalcCompilesAndMatchesCanonicalAstSpansAndValues() throws Exception {
        var fixtures = List.of(new Fixture(true), mappedLeftrec(), mappedLeftrec().withoutHistory(), new Fixture(false));
        var expected = golden();
        var observations = new ArrayList<List<Object>>();
        for (int variant = 0; variant < fixtures.size(); variant++) {
            var rows = new ArrayList<Object>();
            try (var h = new CompilerHarness(fixtures.get(variant).build(), "test.leftrec.variant" + variant)) {
                for (boolean memo : List.of(false, true)) for (int i = 0; i < INPUTS.size(); i++) {
                    Object result = h.parse(INPUTS.get(i), h.options(true, true, false, memo, Map.of()));
                    Object ast = h.canonical(result);
                    assertEquals(i < 3, CompilerHarness.ok(result), INPUTS.get(i));
                    var row = obj("ok", CompilerHarness.ok(result), "consumed", CompilerHarness.get(result, "consumed"),
                        "matched", CompilerHarness.get(result, "matched"));
                    rows.add(row);
                    if (variant > 0) {
                        assertEquals(expected.get(i), ast, INPUTS.get(i));
                        if (i < 3) {
                            assertEquals(new double[]{7.0, 9.0, -4.0}[i], evaluate(ast));
                            assertEquals(INPUTS.get(i).length(), CompilerHarness.get(result, "consumed"));
                            assertEquals(INPUTS.get(i).length(), CompilerHarness.get(result, "matched"));
                        }
                    } else if (i == 3) assertNull(ast);
                }
            }
            observations.add(rows);
        }
        for (var rows : observations) assertEquals(observations.getLast(), rows);
    }

    @Test void unloweredDirectAndIndirectCyclesAreRejectedWithoutTrustingHistory() throws Exception {
        for (boolean history : List.of(false, true)) {
            var ir = new Fixture(true); var rewrite = ir.normalizations.getFirst();
            find(ir.rules, "symbolId", rewrite.get("ruleId")).put("body", rewrite.get("originalBody"));
            // Keep normalization references valid while the active body is recursive.
            rewrite.put("rewrittenBody", rewrite.get("originalBody"));
            if (!history) ir.withoutHistory();
            var error = assertThrows(IllegalArgumentException.class, ir::build);
            assertTrue(error.getMessage().contains("unloweredLeftRecursion"), error.getMessage());
        }
        var ir = new Fixture(true).withoutHistory();
        var factor = ir.expressions.stream().filter(e -> e.get("ruleId").equals("TinyCalcLR::Factor")
            && Json.object(e.get("node")).get("kind").equals("tokenRef")).findFirst().orElseThrow();
        factor.put("node", obj("kind", "ruleRef", "symbolId", "TinyCalcLR::Expression"));
        var error = assertThrows(IllegalArgumentException.class, ir::build);
        assertTrue(error.getMessage().contains("unloweredLeftRecursion"), error.getMessage());
    }

    @Test void nullablePrefixCyclesAreRejectedButConsumingRecursionIsAllowed() throws Exception {
        for (boolean nullable : List.of(false, true)) {
            var ir = new IrFixture();
            String prefix = ir.literal("Root", "a");
            if (nullable) prefix = ir.expression("Root", "optional", "child", prefix);
            ir.rule("Root", ir.seq("Root", prefix, ir.ref("Root", "Root")));
            // Conservative/stale analysis must neither invent nor hide a cycle.
            ir.expressionAnalyses.forEach(analysis -> analysis.put("nullable", !nullable));
            if (nullable) {
                var error = assertThrows(IllegalArgumentException.class, ir::build);
                assertTrue(error.getMessage().contains("unloweredLeftRecursion"), error.getMessage());
            } else assertDoesNotThrow(ir::build);
        }
    }
}
