package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.CompilerHarness.*;
import static org.ubnfc.java.IrFixture.*;
import static org.ubnfc.java.M3FeatureTest.*;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.ubnfc.java.ir.Json;

class M3RecoveryTest {
    static List<?> recoveries(Object result) throws Exception { return (List<?>) get(result, "recoveries"); }
    static List<?> span(Object value) throws Exception { return List.of(get(value, "start"), get(value, "end")); }
    static Map<String, Object> driver(CompilerHarness h, String input) throws Exception {
        var execute = h.loader.loadClass(h.pkg + ".driver.Driver").getDeclaredMethod("execute", Map.class); execute.setAccessible(true);
        return Json.object(execute.invoke(null, obj("id", 1, "grammar", "Probe", "entry", null, "input", input, "options", obj("lexical", true, "scope", true))));
    }
    static void assertRegion(Object result, String mode, int start, int end, String pattern, int skippedEnd) throws Exception {
        assertEquals(1, recoveries(result).size()); Object recovery = recoveries(result).getFirst();
        assertEquals(mode, get(recovery, "mode")); assertEquals(List.of(start, end), span(get(recovery, "span")));
        assertEquals(pattern, get(recovery, "syncPattern")); assertEquals(List.of(start, skippedEnd), span(get(recovery, "skippedSpan")));
        Object d = get(recovery, "diagnostic"); assertEquals("recovery", get(d, "kind")); assertEquals("ERROR", get(d, "severity"));
        assertEquals("recovery", get(d, "origin")); assertEquals(start, get(d, "offset")); assertEquals(end - start, get(d, "length"));
    }
    @Test void syncAndAutoProjectHealthyAstAndExposeDiagnostics() throws Exception {
        var row = Json.object(Json.array(oracle("recovery-sync").get("cases")).getFirst());
        for (String mode : List.of("sync", "auto")) try (var h = new CompilerHarness(ir("recovery-" + mode), PKG)) {
            for (boolean memo : List.of(false, true)) {
                Object result = h.parse("bad;ok;", h.options(true, true, true, memo, Map.of()));
                assertOracleSyntax(h, row, result); assertEquals(row.get("ast"), h.canonical(result));
                assertRegion(result, mode.toUpperCase(java.util.Locale.ROOT), 0, 4, mode.equals("sync") ? ";" : "ok", mode.equals("sync") ? 3 : 4);
                Object recovery = recoveries(result).getFirst(); assertEquals("Probe::Stmt", get(recovery, "ruleId"));
                assertEquals(List.of(0L), get(recovery, "captureOccurrences"));
                assertEquals(0, get(get(recovery, "diagnostic"), "farthestCp"));
                assertEquals(List.of(), get(get(result, "scope"), "diagnostics"));
                assertTrue(recoveries(h.parse("ok;ok;")).isEmpty());
                Object failure = h.parse("bad", h.options(true, true, true, memo, Map.of())); assertFalse(ok(failure)); assertTrue(recoveries(failure).isEmpty());
                assertEquals(0, get(failure, "consumedCp"));
                assertEquals(List.of("';'", "'ok'", "RootParser", "__CaptureSite"), get(get(failure, "diagnostics"), "expected"));
                assertEquals(List.of("';'", "'ok'"), get(get(recovery, "diagnostic"), "expected"));
            }
            var response = driver(h, "bad;ok;"); var diagnostics = Json.array(response.get("diagnostics"));
            assertEquals(1, diagnostics.size()); assertEquals("recovery", Json.object(diagnostics.getFirst()).get("kind"));
            assertEquals(row.get("ast"), Json.read(Json.write(response.get("ast")))); assertNull(response.get("mappingError"));
        }
    }
    @Test void nonBmpStopsAtSyncAndKeepsPrefixCaptures() throws Exception {
        try (var h = new CompilerHarness(ir("recovery-sync"), PKG)) {
            Object result = h.parse("😀;x"); assertFalse(ok(result)); assertNull(ast(result));
            assertEquals(2, get(result, "consumedCp")); assertEquals(2, get(result, "matchedCp"));
            assertRegion(result, "SYNC", 0, 2, ";", 1); assertEquals(List.of(List.of("items", 0, 2)), spans(result));
            var response = driver(h, "😀;x"); assertEquals(true, response.get("accepted")); assertEquals(false, response.get("allConsumed"));
            var diagnostics = Json.array(response.get("diagnostics")); assertEquals(2, diagnostics.size());
            assertEquals("recovery", Json.object(diagnostics.get(0)).get("kind")); assertEquals("trailing_input", Json.object(diagnostics.get(1)).get("kind"));
            assertEquals(2, Json.object(diagnostics.get(1)).get("offset"));
            assertEquals(List.of("end of input"), Json.object(diagnostics.get(1)).get("expected"));
        }
    }
    @Test void rootAnnotationDoesNotWrapEntryAndSkipCompilesD017() throws Exception {
        try (var h = new CompilerHarness(ir("recovery-root"), PKG)) {
            for (Object value : Json.array(oracle("recovery-root").get("cases"))) {
                var row = Json.object(value); Object result = h.parse((String) row.get("input"));
                assertOracleSyntax(h, row, result); assertEquals(row.get("ast"), h.canonical(result)); assertTrue(recoveries(result).isEmpty());
                if (!ok(result)) assertEquals(List.of("';'", "'ok'"), get(get(result, "diagnostics"), "expected"));
            }
        }
        try (var h = new CompilerHarness(ir("recovery-skip"), PKG)) {
            Object result = h.parse("bad;ok;"); assertTrue(ok(result));
            assertEquals(Json.object(Json.array(oracle("recovery-sync").get("cases")).getFirst()).get("ast"), h.canonical(result));
            assertRegion(result, "SKIP", 0, 4, "ok", 4);
            Object eof = h.parse("bad"); assertTrue(ok(eof)); assertEquals(3, get(eof, "consumedCp"));
            assertRegion(eof, "SKIP", 0, 3, null, 3);
            assertEquals(Json.read("{\"type\":\"Root\",\"span\":[0,3],\"fields\":{\"items\":[]}}"), h.canonical(eof));
            assertFalse(ok(h.parse("")));
        }
    }
    @Test void discardedNumericMappingAndScopeEffectsDoNotLeakFromRecovery() throws Exception {
        var f = new IrFixture(); var number = obj("kind", "number", "java", "int32Exact", "rust", "lexeme");
        String token = f.token("Number", "NUM", obj("kind", "builtin", "which", "Number", "codePoints", List.of()));
        String value = f.capture(token, "value", "number", "scalar", number); f.rule("Number", token);
        f.mapping("Number", "Number", List.of(field("value", number, "scalar")), List.of(recipeField("value", value, "number", "scalar")));
        f.effects.add(obj("ruleId", "Number", "mode", "lexical", "events", List.of(obj("when", "successAfterLeave", "action", "declare", "captureSiteId", value, "name", "value"))));
        f.rule("Stmt", f.seq("Stmt", f.ref("Stmt", "Number"), f.literal("Stmt", ";")));
        String item = f.ref("Root", "Stmt"); recovery(f, "Stmt", item, "SYNC", List.of(";"), true);
        String items = f.capture(item, "items", "node", "list", obj("kind", "node", "typeId", "Number"));
        f.rule("Root", f.expression("Root", "repeat", "child", item, "min", 1, "max", null, "terminator", null));
        f.mapping("Root", "Root", List.of(field("items", obj("kind", "node", "typeId", "Number"), "list")), List.of(recipeField("items", items, "node", "list")));
        try (var h = f.compile()) {
            for (boolean memo : List.of(false, true)) {
                Object result = h.parse("1.5!;2;", h.options(true, true, true, memo, Map.of())); assertTrue(ok(result));
                assertEquals(Json.read("{\"type\":\"Root\",\"span\":[0,7],\"fields\":{\"items\":[{\"type\":\"Number\",\"span\":[5,6],\"fields\":{\"value\":2}}]}}"), h.canonical(result));
                assertRegion(result, "SYNC", 0, 5, ";", 4);
                var declarations = (List<?>) get(get(result, "scope"), "declarations"); assertEquals(1, declarations.size());
                assertEquals("2", get(declarations.getFirst(), "name")); assertEquals(5, get(declarations.getFirst(), "offsetCp"));
                assertEquals(1, ((List<?>) get(get(result, "scope"), "events")).size());
                Object two = h.parse("x;y;2;"); assertEquals(2, recoveries(two).size());
                assertEquals(List.of(0, 2), span(get(recoveries(two).getFirst(), "span")));
                assertEquals(List.of(2, 4), span(get(recoveries(two).getLast(), "span")));
            }
        }
    }
    static void recovery(IrFixture f, String rule, String site, String mode, List<String> patterns, boolean consume) {
        f.recoveries.add(obj("ruleId", rule, "mode", mode, "resolvedSyncPatterns", patterns, "consumeSync", consume,
            "noSyncAction", mode.equals("SKIP") ? "consumeToEof" : "fail", "progressRule", mode.equals("SKIP") ? "legacySkip" : "legacySync",
            "applicationSites", List.of(site), "astProjection", "legacyMarker", "recipeId", null));
    }
    @Test void explicitAutoFollowFallbackAndSkipCodePointProgress() throws Exception {
        for (String mode : List.of("AUTO", "SKIP")) for (boolean follow : List.of(false, true)) {
            var f = new IrFixture(); f.rule("Item", f.literal("Item", "ok")); String ref = f.ref("Root", "Item");
            recovery(f, "Item", ref, mode, follow ? List.of(";") : List.of(), false);
            f.rule("Root", follow ? f.seq("Root", ref, f.literal("Root", ";")) : ref);
            try (var h = f.compile()) {
                Object result = h.parse(follow || mode.equals("AUTO") ? "😀;" : "😀"); assertTrue(ok(result));
                assertEquals(mode.equals("SKIP") && !follow ? 1 : 2, get(result, "consumedCp"));
                assertEquals(List.of(0, follow || mode.equals("SKIP") ? 1 : 2), span(get(recoveries(result).getFirst(), "span")));
                assertFalse(ok(h.parse("")));
                if (follow && mode.equals("AUTO")) assertFalse(ok(h.parse(";")));
                if (follow && mode.equals("SKIP")) {
                    Object atSync = h.parse(";;"); assertTrue(ok(atSync)); assertRegion(atSync, "SKIP", 0, 1, ";", 1);
                }
            }
        }
    }
    @Test void missingScalarPropagatesThroughParentsAndOptionalListAbsorbIt() throws Exception {
        for (String card : List.of("scalar", "optional", "list")) {
            var f = new IrFixture(); String good = f.literal("Item", "ok"); String value = f.capture(good, "value", "text", "scalar", obj("kind", "text"));
            f.rule("Item", f.seq("Item", good, f.literal("Item", ";")));
            f.mapping("Item", "Item", List.of(field("value", obj("kind", "text"), "scalar")), List.of(recipeField("value", value, "text", "scalar")));
            String item = f.ref("Middle", "Item"); String child = f.capture(item, "child", "node", "scalar", obj("kind", "node", "typeId", "Item"));
            recovery(f, "Item", item, "SYNC", List.of(";"), true); f.rule("Middle", item);
            f.mapping("Middle", "Middle", List.of(field("child", obj("kind", "node", "typeId", "Item"), "scalar")), List.of(recipeField("child", child, "node", "scalar")));
            String middle = f.ref("Root", "Middle"); String outer = f.capture(middle, "child", "node", card, obj("kind", "node", "typeId", "Middle"));
            f.rule("Root", middle); f.mapping("Root", "Root", List.of(field("child", obj("kind", "node", "typeId", "Middle"), card)), List.of(recipeField("child", outer, "node", card)));
            try (var h = f.compile()) {
                Object bad = h.parse("bad;"); assertTrue(ok(bad)); assertEquals(1, recoveries(bad).size());
                if (card.equals("scalar")) assertNull(ast(bad));
                else assertEquals(card.equals("optional") ? null : List.of(), Json.object(Json.object(h.canonical(bad)).get("fields")).get("child"));
                assertNotNull(ast(h.parse("ok;")));
            }
        }
    }
    @Test void recoveryRollbackMemoReplayLongestAndLookahead() throws Exception {
        for (String context : List.of("retry", "discard", "longest", "look")) {
            var f = new IrFixture(); f.rule("Item", f.literal("Item", "ok"));
            String site = f.ref("Carrier", "Item"); f.rule("Carrier", site); recovery(f, "Item", site, "SYNC", List.of(";"), true);
            String first = f.ref("Root", "Carrier");
            String body = switch (context) {
                case "retry" -> f.choice("Root", "ordered", f.seq("Root", first, f.literal("Root", "x")), f.seq("Root", f.ref("Root", "Carrier"), f.literal("Root", "y")));
                case "discard" -> f.choice("Root", "ordered", f.seq("Root", first, f.literal("Root", "x")), f.literal("Root", "bad;y"));
                case "longest" -> f.choice("Root", "longest", first, f.literal("Root", "bad;y"));
                default -> f.seq("Root", f.expression("Root", "lookahead", "child", first), f.literal("Root", "bad;y"));
            };
            f.rule("Root", body);
            f.memos.add(obj("exprId", site, "cacheSuccess", true, "cacheFailure", true, "reason", "defaultSafe", "omissionProof", null));
            try (var h = f.compile()) {
                for (boolean memo : List.of(false, true)) {
                    Object result = h.parse("bad;y", h.options(false, true, true, memo, Map.of())); assertTrue(ok(result));
                    assertEquals(context.equals("retry") ? 1 : 0, recoveries(result).size());
                    assertEquals(List.of(), get(get(result, "scope"), "diagnostics"));
                }
            }
        }
    }
}
