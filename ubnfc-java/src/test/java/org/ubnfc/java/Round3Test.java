package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.CompilerHarness.*;
import static org.ubnfc.java.IrFixture.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.ubnfc.java.ir.IrReader;
import org.ubnfc.java.ir.Json;

class Round3Test {
    @Test void evolutionFailureCandidatesMatchIndependentOracleWithAndWithoutMemo() throws Exception {
        var ir = IrReader.read(Path.of("src/test/resources/ir/round3/evolution-v3.ir.json"));
        var oracle = Json.object(Json.read(Files.readString(Path.of("../corpus/oracle/evolution-v3.json"))));
        try (var h = new CompilerHarness(ir, "test.generated.round3")) {
            for (Object item : Json.array(oracle.get("cases"))) {
                var row = Json.object(item);
                var diagnostics = Json.array(row.get("diagnostics"));
                if (diagnostics.isEmpty()) continue;
                var expected = Json.object(diagnostics.getFirst());
                for (boolean memo : List.of(false, true)) {
                    Object parsed = h.parse((String) row.get("input"), h.options(false, true, true, memo, Map.of()));
                    Object actual = get(parsed, "diagnostics");
                    assertEquals(expected.get("kind"), get(actual, "kind"), row.get("id").toString());
                    assertEquals(((Number) expected.get("offset")).intValue(), get(actual, "offset"), row.get("id").toString());
                    assertEquals(((Number) expected.get("farthestOffset")).intValue(), get(actual, "farthestCp"), row.get("id").toString());
                    assertEquals(expected.get("farthestExpected"), get(actual, "farthestExpected"), row.get("id").toString());
                }
            }
        }
    }
    @Test void transparentSemanticCollectionsKeepTextNodesAndExcludeSeparators() throws Exception {
        for (String fixture : List.of("semantic-separated", "semantic-repeat")) {
            var ir = IrReader.read(Path.of("src/test/resources/ir/round3/" + fixture + ".ir.json"));
            try (var h = new CompilerHarness(ir, "test.generated.round3")) {
                String input = fixture.endsWith("separated") ? "[a,x,😀]" : "[ax😀]";
                for (boolean memo : List.of(false, true)) {
                    Object result = h.parse(input, h.options(true, true, true, memo, Map.of()));
                    var values = Json.array(Json.object(Json.object(h.canonical(result)).get("fields")).get("values"));
                    assertEquals(3, values.size()); assertEquals("a", values.get(0)); assertEquals("😀", values.get(2));
                    assertEquals("Leaf", Json.object(values.get(1)).get("type"));
                    assertEquals("x", Json.object(Json.object(values.get(1)).get("fields")).get("text"));
                }
                if (fixture.endsWith("repeat")) assertEquals(List.of(), Json.object(Json.object(h.canonical(h.parse("[]"))).get("fields")).get("values"));
            }
        }
    }
    @Test void memoHitReplaysSuccessfulLookaheadReachAndNotDiscardsIt() throws Exception {
        var f = new IrFixture();
        String a = f.literal("Peek", "a");
        f.rule("Peek", f.expression("Peek", "lookahead", "child", a));
        String call = f.ref("Root", "Peek");
        String failed = f.seq("Root", call, f.literal("Root", "z"));
        f.rule("Root", f.seq("Root", f.expression("Root", "negativeLookahead", "child", failed), call));
        try (var h = f.compile()) {
            for (boolean memo : List.of(false, true)) {
                Object parsed = h.parse("ab", h.options(false, true, true, memo, Map.of()));
                assertFalse(ok(parsed)); assertEquals(0, get(parsed, "consumedCp")); assertEquals(1, get(parsed, "matchedCp"));
                Object diagnostic = get(parsed, "diagnostics");
                assertEquals("trailing_input", get(diagnostic, "kind"));
                assertEquals(1, get(diagnostic, "farthestCp"));
                assertEquals(List.of(), get(diagnostic, "farthestExpected"));
            }
        }
    }

    @Test void externalValueSpanSurvivesAliasWhileCaptureKeepsConsumedExtent() throws Exception {
        var f = new IrFixture();
        f.rule("Alias", f.token("Alias", "EXTERNAL", obj("kind", "extern", "logicalId", "scan", "javaBinding", null,
            "rustBinding", null, "originalClassName", "Scanner", "inputState", List.of(),
            "resultFields", List.of("ok", "consumedEnd", "matchedEnd", "valueSpan", "diagnostics", "effects"))));
        String ref = f.ref("Root", "Alias");
        String site = f.capture(ref, "value", "text", "scalar", obj("kind", "text")); f.rule("Root", ref);
        f.mapping("Root", "Value", List.of(field("value", obj("kind", "text"), "scalar")), List.of(recipeField("value", site, "text", "scalar")));
        try (var h = f.compile()) {
            Class<?> scannerType = h.loader.loadClass(h.pkg + ".api.TokenScanner");
            var constructor = h.loader.loadClass(h.pkg + ".api.TokenScanner$ScanResult")
                .getConstructor(boolean.class, int.class, int.class, int.class, int.class, List.class, List.class);
            Object scanner = java.lang.reflect.Proxy.newProxyInstance(h.loader, new Class<?>[]{scannerType},
                (proxy, method, args) -> constructor.newInstance(true, 4, 4, 1, 3, List.of(), List.of()));
            Object parsed = h.parse("'😀'", h.options(true, true, true, true, Map.of("scan", scanner)));
            assertEquals("😀", Json.object(Json.object(h.canonical(parsed)).get("fields")).get("value"));
            Object span = get(((List<?>) get(parsed, "captures")).getFirst(), "span");
            assertEquals(0, get(span, "start")); assertEquals(3, get(span, "end"));
        }
    }

    @Test void directSeparatedCaptureBindsElementsWithoutSeparators() throws Exception {
        var f = new IrFixture();
        String separated = f.expression("Root", "separated", "element", f.literal("Root", "a"),
            "separator", f.literal("Root", ","), "min", 1, "pairRollback", true);
        String site = f.capture(separated, "values", "text", "list", obj("kind", "text")); f.rule("Root", separated);
        f.mapping("Root", "Values", List.of(field("values", obj("kind", "text"), "list")), List.of(recipeField("values", site, "text", "list")));
        try (var h = f.compile()) {
            Object parsed = h.parse("a,a");
            assertEquals(List.of("a", "a"), Json.object(Json.object(h.canonical(parsed)).get("fields")).get("values"));
            var captures = (List<?>) get(parsed, "captures"); assertEquals(2, captures.size());
            assertEquals(2, get(get(captures.get(1), "span"), "start"));
        }
    }
}
