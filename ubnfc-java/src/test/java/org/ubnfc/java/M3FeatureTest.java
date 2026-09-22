package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.CompilerHarness.*;
import static org.ubnfc.java.IrFixture.*;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Modifier;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Stream;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestFactory;
import org.ubnfc.java.ir.GrammarIr;
import org.ubnfc.java.ir.IrReader;
import org.ubnfc.java.ir.Json;

/** Real front-produced IRs and immutable oracle observations, plus D-017/D-018 expectations. */
class M3FeatureTest {
    static final String PKG = "test.generated.m3";
    static GrammarIr ir(String name) throws Exception { return IrReader.read(Path.of("src/test/resources/ir/m3/" + name + ".ir.json")); }
    static Map<String, Object> raw(String name) throws Exception { return Json.object(Json.read(Files.readString(Path.of("src/test/resources/ir/m3/" + name + ".ir.json")))); }
    static Map<String, Object> oracle(String name) throws Exception { return Json.object(Json.read(Files.readString(Path.of("../corpus/oracle/" + name + ".json")))); }
    static Object ast(Object result) throws Exception { return ((Optional<?>) get(result, "ast")).orElse(null); }
    static int integer(Object number) { return ((Number) number).intValue(); }
    static List<Object> spans(Object result) throws Exception {
        var out = new ArrayList<Object>();
        for (Object c : (List<?>) get(result, "captures")) out.add(List.of(get(c, "name"), get(get(c, "span"), "start"), get(get(c, "span"), "end")));
        return out;
    }
    static void assertOracleSyntax(CompilerHarness h, Map<String, Object> row, Object result) throws Exception {
        assertEquals(row.get("accepted"), ok(result), row.get("id").toString());
        assertEquals(integer(row.get("consumed")), get(result, "consumedCp"));
        assertEquals(integer(row.get("matched")), get(result, "matchedCp"));
        var expectedCaptures = new ArrayList<Object>();
        for (Object item : Json.array(row.get("captures"))) {
            var c = Json.object(item); var span = Json.array(c.get("span"));
            expectedCaptures.add(List.of(c.get("name"), integer(span.get(0)), integer(span.get(1))));
        }
        assertEquals(expectedCaptures, spans(result));
    }
    static CompilerHarness semantics(GrammarIr ir, String methods) throws Exception {
        String source = "package " + PKG + "; public final class Semantics extends ProbeEvaluator<Object> { " + methods + " }";
        return new CompilerHarness(ir, PKG, Map.of(PKG.replace('.', '/') + "/Semantics.java", source));
    }
    static Object eval(CompilerHarness h, Object value) throws Exception {
        Class<?> implementation = h.loader.loadClass(PKG + ".Semantics");
        return implementation.getMethod("eval", h.loader.loadClass(PKG + ".ProbeAST")).invoke(implementation.getConstructor().newInstance(), value);
    }
    @TestFactory Stream<DynamicTest> evaluatorOracleAndExecution() {
        return Stream.of("binary_arithmetic", "variable_ref", "conditional", "passthrough", "literal", "manual", "unknown").map(kind -> DynamicTest.dynamicTest(kind, () -> {
            String fixture = "eval-" + kind;
            String methods = switch (kind) {
                case "binary_arithmetic" -> "protected Object evalLeaf(String s) { return Integer.valueOf(s); } protected Object applyBinary(String op, Object a, Object b) { return (Integer) a + (Integer) b; }";
                case "variable_ref" -> "protected Object resolveVariable(String name) { return name; }";
                case "manual", "unknown" -> "protected Object evalLiteral(ProbeAST.Literal node) { return node.value(); }";
                default -> "";
            };
            try (var h = semantics(ir(fixture), methods)) {
                var data = oracle(fixture); var row = Json.object(Json.array(data.get("cases")).getFirst());
                Object result = h.parse((String) row.get("input")); assertOracleSyntax(h, row, result); assertEquals(row.get("ast"), h.canonical(result));
                Class<?> evaluator = h.loader.loadClass(PKG + ".ProbeEvaluator");
                for (Object method : Json.array(Json.object(data.get("evaluator")).get("methods"))) {
                    var m = Json.object(method);
                    var actual = Stream.of(evaluator.getDeclaredMethods()).filter(v -> v.getName().equals(m.get("name"))).findFirst().orElseThrow();
                    assertEquals(m.get("abstract"), Modifier.isAbstract(actual.getModifiers()), m.toString());
                }
                Object expected = switch (kind) { case "binary_arithmetic", "conditional" -> 3; case "variable_ref" -> "x"; default -> 7; };
                assertEquals(expected, eval(h, ast(result)));
                if (kind.equals("binary_arithmetic")) {
                    assertEquals(6, eval(h, ast(h.parse("1+2+3"))));
                    assertEquals(1, eval(h, ast(h.parse("1"))));
                    Class<?> literal = h.loader.loadClass(PKG + ".ProbeAST$Literal"), binary = h.loader.loadClass(PKG + ".ProbeAST$Binary");
                    assertEquals(42, eval(h, binary.getConstructor(literal, List.class, List.class).newInstance(null, List.of("42"), List.of())));
                    var steps = new ArrayList<Object>();
                    Object counter = h.loader.loadClass(PKG + ".ProbeEvaluator$StepCounterStrategy").getConstructor(java.util.function.BiConsumer.class)
                        .newInstance((java.util.function.BiConsumer<Integer, Object>) (i, node) -> steps.add(i));
                    Object evaluatorInstance = h.loader.loadClass(PKG + ".Semantics").getConstructor().newInstance();
                    evaluator.getMethod("setDebugStrategy", h.loader.loadClass(PKG + ".ProbeEvaluator$DebugStrategy")).invoke(evaluatorInstance, counter);
                    evaluator.getMethod("eval", h.loader.loadClass(PKG + ".ProbeAST")).invoke(evaluatorInstance, ast(result));
                    assertEquals(List.of(0, 1, 2), steps);
                }
            }
        }));
    }
    @Test void evaluatorPrefixAndSumDispatch() throws Exception {
        var r = raw("eval-variable_ref"); var semantics = Json.object(r.get("semantics")); r.put("semantics", semantics);
        var rules = new ArrayList<Object>(Json.array(semantics.get("rules"))); var rule = Json.object(rules.getFirst()); rules.set(0, rule); semantics.put("rules", rules);
        rule.put("eval", obj("kind", "variable_ref", "strategy", "auto", "params", obj("strip_prefix", "😀$")));
        try (var h = semantics(IrReader.read(Json.write(r)), "protected Object resolveVariable(String name) { return name; }")) {
            var variable = h.loader.loadClass(PKG + ".ProbeAST$Variable");
            assertEquals("x", eval(h, variable.getConstructor(String.class).newInstance("😀$x")));
            assertEquals("x", eval(h, variable.getConstructor(String.class).newInstance("x")));
        }
        try (var h = semantics(ir("typeof"), "protected Object evalA(ProbeAST.A n) { return \"A\"; } protected Object evalB(ProbeAST.B n) { return \"B\"; } protected Object evalPair(ProbeAST.Pair n) { return eval(n.left()); } protected Object evalNode(ProbeAST.Node n) { throw new AssertionError(\"broad callback\"); }")) {
            assertEquals("A", eval(h, ast(h.parse("a:a")))); assertEquals("B", eval(h, ast(h.parse("b:b"))));
        }
        var sum = raw("typeof"); var s = Json.object(sum.get("semantics")); sum.put("semantics", s);
        var rs = new ArrayList<Object>(Json.array(s.get("rules"))); s.put("rules", rs); var union = Json.object(rs.get(1)); rs.set(1, union);
        union.put("eval", obj("kind", "literal", "strategy", "auto", "params", Map.of()));
        assertTrue(assertThrows(IllegalArgumentException.class, () -> IrReader.read(Json.write(sum))).getMessage().contains("not sum"));
    }
    @Test void documentedIrFixturesCompile() throws Exception {
        for (String path : List.of("known__eval-default", "positive__elements")) {
            var fixture = IrReader.read(Path.of("../ir/fixtures/rust__unlaxer-ubnf__tests__fixtures__" + path + ".ubnf.ir.json"));
            try (var h = new CompilerHarness(fixture, PKG)) { assertFalse(ok(h.parse(""))); }
        }
    }
    @Test void passthroughDispatchesNodesAndConditionalEvaluatesOnlySelectedBranch() throws Exception {
        var f = new IrFixture(); String word = f.literal("Leaf", "true"), value = f.capture(word, "value", "text", "scalar", obj("kind", "text"));
        f.rule("Leaf", word); f.mapping("Leaf", "Literal", List.of(field("value", obj("kind", "text"), "scalar")), List.of(recipeField("value", value, "text", "scalar")));
        f.rules.getLast().put("eval", obj("kind", "literal", "strategy", "auto", "params", Map.of()));
        String child = f.ref("Wrapper", "Leaf"), wrapped = f.capture(child, "value", "node", "scalar", obj("kind", "node", "typeId", "Literal"));
        f.rule("Wrapper", child); f.mapping("Wrapper", "Wrapper", List.of(field("value", obj("kind", "node", "typeId", "Literal"), "scalar")), List.of(recipeField("value", wrapped, "node", "scalar")));
        f.rules.getLast().put("eval", obj("kind", "passthrough", "strategy", "auto", "params", Map.of()));
        var fields = new ArrayList<Map<String, Object>>(); var mappings = new ArrayList<Map<String, Object>>(); var refs = new ArrayList<String>();
        for (String name : List.of("condition", "thenExpr", "elseExpr")) {
            String ref = f.ref("Root", "Wrapper"); refs.add(ref);
            String site = f.capture(ref, name, "node", "scalar", obj("kind", "node", "typeId", "Wrapper"));
            fields.add(field(name, obj("kind", "node", "typeId", "Wrapper"), "scalar")); mappings.add(recipeField(name, site, "node", "scalar"));
        }
        f.rule("Root", f.seq("Root", refs.toArray(String[]::new))); f.mapping("Root", "Conditional", fields, mappings);
        f.rules.getLast().put("eval", obj("kind", "conditional", "strategy", "auto", "params", Map.of()));
        try (var h = semantics(f.build(), "")) {
            var literal = h.loader.loadClass(PKG + ".ProbeAST$Literal"); var wrapper = h.loader.loadClass(PKG + ".ProbeAST$Wrapper");
            var literalCtor = literal.getConstructor(String.class); var wrap = wrapper.getConstructor(literal);
            var conditional = h.loader.loadClass(PKG + ".ProbeAST$Conditional").getConstructor(wrapper, wrapper, wrapper);
            Object yes = wrap.newInstance(literalCtor.newInstance("yes"));
            assertEquals("yes", eval(h, conditional.newInstance(wrap.newInstance(literalCtor.newInstance("TrUe")), yes, null)));
            assertEquals("yes", eval(h, conditional.newInstance(wrap.newInstance(literalCtor.newInstance("false")), null, yes)));
        }
    }
    @Test void typeofOracleMappingFailurePreservesSyntax() throws Exception {
        try (var h = new CompilerHarness(ir("typeof"), PKG)) {
            for (Object value : Json.array(oracle("typeof").get("cases"))) for (boolean memo : List.of(false, true)) {
                var row = Json.object(value); String input = (String) row.get("input");
                Object syntax = h.parse(input, h.options(false, true, true, memo, Map.of())); assertOracleSyntax(h, row, syntax);
                if (row.get("mappingError") == null) {
                    Object result = h.parse(input, h.options(true, true, true, memo, Map.of())); assertEquals(row.get("ast"), h.canonical(result));
                    if (!ok(result)) { assertEquals(2, get(get(result, "diagnostics"), "offset")); assertEquals(List.of("'a'", "'b'", "__CaptureSite"), get(get(result, "diagnostics"), "expected")); }
                } else {
                    var error = assertThrows(InvocationTargetException.class, () -> h.parse(input, h.options(true, true, true, memo, Map.of()))).getCause();
                    assertEquals("MappingException", error.getClass().getSimpleName()); assertOracleSyntax(h, row, get(error, "syntax"));
                    String message = ((String) row.get("mappingError")).substring("java.lang.IllegalArgumentException: ".length());
                    assertTrue(error.getMessage().startsWith(message)); assertEquals(0, get(get(error, "span"), "start")); assertEquals(3, get(get(error, "span"), "end"));
                }
            }
        }
    }
    @Test void errorOracleHintOnceWithAndWithoutMemo() throws Exception {
        try (var h = new CompilerHarness(ir("error"), PKG)) {
            for (Object value : Json.array(oracle("error").get("cases"))) for (boolean memo : List.of(false, true)) {
                var row = Json.object(value); Object result = h.parse((String) row.get("input"), h.options(true, true, true, memo, Map.of()));
                assertOracleSyntax(h, row, result); assertEquals(row.get("ast"), h.canonical(result));
                assertEquals(ok(result) ? List.of() : List.of("'a'", "need a"), get(get(result, "diagnostics"), "expected"));
                if (!ok(result)) assertEquals(0, get(get(result, "diagnostics"), "offset"));
            }
        }
        var f = new IrFixture(); String error = f.expression("Error", "error", "message", "need a");
        f.expressions.getFirst().put("execution", obj("entryFlag", "preserve", "transaction", "once", "childMode", "inherit", "failureCursor", "restoreTransaction", "primitiveProfile", null));
        f.rule("Error", error); f.rule("Root", f.choice("Root", "ordered", f.ref("Root", "Error"), f.ref("Root", "Error")));
        try (var h = f.compile()) {
            for (boolean memo : List.of(false, true)) {
                Object result = h.parse("x", h.options(false, true, false, memo, Map.of()));
                assertFalse(ok(result)); assertEquals(List.of("need a"), get(get(result, "diagnostics"), "expected"));
            }
        }
    }
    @Test void enumOracleAndCollisionD017() throws Exception {
        for (String fixture : List.of("enum-plain", "enum", "enum-collision")) try (var h = new CompilerHarness(ir(fixture), PKG)) {
            Class<?> enumeration = h.loader.loadClass(PKG + ".ProbeAST$Mode");
            assertEquals("ON", enumeration.getMethod("fromText", String.class).invoke(null, "on").toString());
            String second = fixture.equals("enum-collision") ? "ON" : "off";
            String variant = fixture.equals("enum-collision") ? "ON_2" : "OFF";
            assertEquals(variant, enumeration.getMethod("fromText", String.class).invoke(null, second).toString());
            for (String input : List.of("on", second)) {
                Object result = h.parse(input); assertTrue(ok(result)); assertEquals(input.length(), get(result, "consumedCp"));
                if (!fixture.equals("enum-plain")) assertEquals(input.equals("on") ? "ON" : variant, get(ast(result), "mode").toString());
            }
            assertFalse(ok(h.parse("other")));
            assertThrows(InvocationTargetException.class, () -> enumeration.getMethod("fromText", String.class).invoke(null, "other"));
            if (fixture.equals("enum-plain")) for (Object value : Json.array(oracle(fixture).get("cases"))) {
                var row = Json.object(value); Object result = h.parse((String) row.get("input")); assertOracleSyntax(h, row, result); assertEquals(row.get("ast"), h.canonical(result));
            }
        }
    }
    @Test void enumSuffixDoesNotStealAnExistingName() throws Exception {
        var f = new IrFixture(); f.rule("Root", f.choice("Root", "ordered", f.literal("Root", "a"), f.literal("Root", "b"), f.literal("Root", "c"), f.literal("Root", "d")));
        var variants = new ArrayList<Map<String, Object>>();
        for (int i = 0; i < 4; i++) variants.add(obj("literal", String.valueOf((char) ('a' + i)), "variant", "Variant" + i, "javaName", i == 2 ? "ON_2" : "ON", "rustName", "Variant" + i));
        f.types.add(obj("id", "Mode", "name", "Mode", "kind", "enum", "enclosingTypeId", null, "ruleId", "Root", "variants", variants));
        f.recipes.add(obj("id", "recipe:Root", "ruleId", "Root", "resultTypeId", "Mode", "kind", "enum", "fields", List.of(), "phase", "selectedMapping", "onFailure", "mappingFailure", "transformId", null, "leafFallback", null));
        try (var h = f.compile()) {
            for (int i = 0; i < 4; i++) assertEquals(List.of("ON", "ON_3", "ON_2", "ON_4").get(i), h.canonical(h.parse(String.valueOf((char) ('a' + i)))));
        }
    }
}
