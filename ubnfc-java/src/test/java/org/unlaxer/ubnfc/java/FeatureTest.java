package org.unlaxer.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.unlaxer.ubnfc.java.CompilerHarness.*;
import static org.unlaxer.ubnfc.java.IrFixture.*;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Proxy;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import org.unlaxer.ubnfc.java.emit.JavaEmitter;
import org.unlaxer.ubnfc.java.ir.Json;

class FeatureTest {
    @Test void cursorContractAndZeroWidthRepeats() throws Exception {
        for (int[] bounds : List.of(new int[]{1, 2}, new int[]{2, 2}, new int[]{0, 0})) {
            var f = new IrFixture(); String empty = f.expression("Root", "empty");
            f.rule("Root", f.expression("Root", "repeat", "child", empty, "min", bounds[0], "max", bounds[1], "terminator", null));
            try (var h = f.compile()) {
                Object result = h.parse("ab", h.options(false, false, false, true, Map.of()));
                assertEquals(bounds[0] <= 1, ok(result)); assertEquals(0, get(result, "consumedCp"));
                assertEquals(bounds[0] == 1 ? 1 : 0, get(result, "matchedCp"));
            }
        }
        var f = new IrFixture();
        String look = f.expression("Root", "lookahead", "child", f.literal("Root", "a"));
        String until = f.token("Root", "UNTIL", obj("kind", "until", "terminator", "#", "missingTerminator", "consumeToEofAndSucceed", "emptyTerminator", "consumeToEof", "preserveMatched", true));
        f.rule("Root", f.seq("Root", look, until));
        try (var h = f.compile()) {
            Object result = h.parse("a#", h.options(false, false, false, true, Map.of()));
            assertTrue(ok(result)); assertEquals(0, get(result, "consumedCp")); assertEquals(2, get(result, "matchedCp"));
        }
    }
    @Test void matchOnlyProgressAndNegativeSuppressionReset() throws Exception {
        var f = new IrFixture();
        String repeated = f.expression("Root", "repeat", "child", f.expression("Root", "any"), "min", 2, "max", 2, "terminator", null);
        String look = f.expression("Root", "lookahead", "child", repeated);
        String negative = f.expression("Root", "negativeLookahead", "child", f.literal("Root", "xx"));
        // Empty Word fails; use an empty sequence for epsilon and preserve R=true here.
        String body = f.seq("Root", look, negative, f.seq("Root"));
        var execution = Json.object(f.expressions.getLast().get("execution"));
        execution.put("entryFlag", "preserve"); f.expressions.getLast().put("execution", execution);
        f.rule("Root", body);
        try (var h = f.compile()) {
            Object result = h.parse("ab", h.options(false, false, false, true, Map.of()));
            assertTrue(ok(result)); assertEquals(0, get(result, "consumedCp")); assertEquals(0, get(result, "matchedCp"));
            assertEquals(List.of(), get(get(result, "diagnostics"), "expected"));
        }
        var n = new IrFixture(); n.rule("Root", n.expression("Root", "negativeLookahead", "child", n.literal("Root", "a")));
        try (var h = n.compile()) { Object result = h.parse("a"); assertFalse(ok(result)); assertFalse(((List<?>) get(get(result, "diagnostics"), "expected")).isEmpty()); }
    }
    @Test void orderedLongestPredictAndFullFallback() throws Exception {
        for (String strategy : List.of("ordered", "longest", "predictive")) {
            var f = new IrFixture(); String a = f.literal("Root", "a"), ab = f.literal("Root", "ab");
            String choice = f.choice("Root", strategy, a, ab); f.rule("Root", choice);
            if (strategy.equals("predictive")) for (var analysis : f.expressionAnalyses)
                analysis.put("mayStart", obj("certainty", "conservative", "ranges", List.of(obj("start", 97, "end", 97)), "eof", false, "zeroConsumeCondition", "never", "includesTrivia", true));
            try (var h = f.compile()) {
                Object result = h.parse("ab", h.options(false, false, false, true, Map.of()));
                assertTrue(ok(result)); assertEquals(strategy.equals("longest") ? 2 : 1, get(result, "consumedCp"));
                if (strategy.equals("predictive")) assertTrue(new JavaEmitter(f.build(), "test.predict").sources().values().stream().anyMatch(s -> s.contains("switch (s.cp")));
            }
        }
        var f = new IrFixture(); String a = f.literal("Root", "x"), b = f.literal("Root", "ab");
        f.rule("Root", f.choice("Root", "predictive", a, b));
        // The fallback is observable even when front's conservative filter selects no candidate.
        for (var analysis : f.expressionAnalyses) analysis.put("mayStart", obj("certainty", "conservative", "ranges", List.of(), "eof", false, "zeroConsumeCondition", "never", "includesTrivia", true));
        try (var h = f.compile()) { assertTrue(ok(h.parse("ab"))); }
    }
    @Test void deferredIntMappingOptionalListAndAbandonedBranches() throws Exception {
        var f = new IrFixture();
        String token = f.token("Number", "NUMBER", obj("kind", "builtin", "which", "Number", "codePoints", List.of()));
        String capture = f.capture(token, "value", "number", "scalar", obj("kind", "number", "java", "int32Exact", "rust", "lexeme"));
        f.rule("Number", token);
        f.mapping("Number", "Number", List.of(field("value", obj("kind", "number", "java", "int32Exact", "rust", "lexeme"), "scalar")), List.of(recipeField("value", capture, "number", "scalar")));
        String discarded = f.seq("Root", f.ref("Root", "Number"), f.literal("Root", "x"));
        f.rule("Root", f.choice("Root", "ordered", discarded, f.literal("Root", "1.5"), f.ref("Root", "Number")));
        try (var h = f.compile()) {
            assertTrue(ok(h.parse("1.5"))); // Numeric branch loses; no NumberFormatException.
            Object integer = h.parse("42"); assertEquals(Json.read("{\"type\":\"Number\",\"span\":[0,2],\"fields\":{\"value\":42}}"), h.canonical(integer));
            for (String text : List.of("1e2", "2147483648", "1.")) {
                var error = assertThrows(InvocationTargetException.class, () -> h.parse(text)).getCause();
                assertEquals("MappingException", error.getClass().getSimpleName());
                assertInstanceOf(NumberFormatException.class, error.getCause()); assertTrue(ok(get(error, "syntax")));
            }
        }
    }
    @Test void scopesUndoOnLongestAndFailedSequence() throws Exception {
        var f = new IrFixture(); String word = f.literal("Declaration", "a"); String site = f.capture(word, "name", "text", "scalar", obj("kind", "text"));
        f.rule("Declaration", word);
        f.effects.add(obj("ruleId", "Declaration", "mode", "lexical", "events", List.of(obj("when", "successAfterLeave", "action", "declare", "captureSiteId", site, "name", "variable"))));
        f.rule("Root", f.choice("Root", "longest", f.ref("Root", "Declaration"), f.literal("Root", "ab")));
        try (var h = f.compile()) {
            Object result = h.parse("ab"); assertTrue(ok(result)); assertEquals(List.of(), get(get(result, "scope"), "declarations"));
            Object selected = h.parse("a"); assertEquals(1, ((List<?>) get(get(selected, "scope"), "declarations")).size());
        }
    }
    @Test void scopeEntryLeaveReferenceAndCatalog() throws Exception {
        var f = new IrFixture();
        String name = f.literal("Declaration", "a"); String declared = f.capture(name, "name", "text", "scalar", obj("kind", "text")); f.rule("Declaration", name);
        f.effects.add(obj("ruleId", "Declaration", "mode", "lexical", "events", List.of(
            obj("when", "entry", "action", "enter", "captureSiteId", null, "name", null),
            obj("when", "successBeforeLeave", "action", "leave", "captureSiteId", null, "name", null),
            obj("when", "successAfterLeave", "action", "declare", "captureSiteId", declared, "name", "var"))));
        String use = f.literal("Use", "b"); String used = f.capture(use, "name", "text", "scalar", obj("kind", "text")); f.rule("Use", use);
        f.effects.add(obj("ruleId", "Use", "mode", "lexical", "events", List.of(obj("when", "afterDeclarations", "action", "use", "captureSiteId", used, "name", "var"))));
        f.semantics.put("catalogs", List.of(obj("context", "vars", "ruleId", "Declaration", "captureSiteId", declared)));
        f.rule("Root", f.seq("Root", f.ref("Root", "Declaration"), f.ref("Root", "Use")));
        try (var h = f.compile()) {
            Object result = h.parse("ab"); assertTrue(ok(result)); Object scope = get(result, "scope");
            assertEquals(1, ((List<?>) get(scope, "declarations")).size()); assertEquals(1, ((List<?>) get(scope, "references")).size()); assertEquals(1, ((List<?>) get(scope, "diagnostics")).size());
            assertEquals(1, ((List<?>) ((Map<?, ?>) get(result, "catalogs")).get("vars")).size());
        }
    }
    @Test void externMemoReplaysEffectsExactlyOnceAndUnsafeClosureDoesNotCache() throws Exception {
        for (boolean safe : List.of(true, false)) {
            var f = new IrFixture(); String token = f.token("Token", "EXTERNAL", obj("kind", "extern", "logicalId", "scan", "javaBinding", null, "rustBinding", null, "originalClassName", "Scanner", "inputState", List.of(), "resultFields", List.of("ok", "consumedEnd", "matchedEnd", "valueSpan", "diagnostics", "effects")));
            var safety = Json.object(f.tokens.getFirst().get("memoSafety")); safety.put("success", safe); safety.put("failure", safe); f.tokens.getFirst().put("memoSafety", safety);
            f.rule("Token", token); f.rule("Root", f.choice("Root", "ordered", f.seq("Root", f.ref("Root", "Token"), f.literal("Root", "x")), f.seq("Root", f.ref("Root", "Token"), f.literal("Root", "y"))));
            try (var h = f.compile()) {
                var count = new AtomicInteger(); Class<?> scannerType = h.loader.loadClass(h.pkg + ".api.TokenScanner");
                var effect = h.loader.loadClass(h.pkg + ".api.TokenScanner$Effect").getConstructor(String.class, String.class, int.class, int.class).newInstance("declare", "a", 0, 1);
                var ctor = h.loader.loadClass(h.pkg + ".api.TokenScanner$ScanResult").getConstructor(boolean.class, int.class, int.class, int.class, int.class, List.class, List.class);
                Object scanner = Proxy.newProxyInstance(h.loader, new Class<?>[]{scannerType}, (proxy, method, args) -> { count.incrementAndGet(); int c = (Integer) get(args[1], "consumed"); return ctor.newInstance(true, c + 1, c + 1, c, c + 1, List.of(), List.of(effect)); });
                Object result = h.parse("ay", h.options(false, true, false, true, Map.of("scan", scanner)));
                assertTrue(ok(result)); assertEquals(safe ? 1 : 2, count.get()); assertEquals(1, ((List<?>) get(get(result, "scope"), "declarations")).size());
            }
        }
    }
    @Test void recoveryOnlyAtDeclaredReferenceAndMappingFailure() throws Exception {
        var f = new IrFixture(); f.rule("Bad", f.literal("Bad", "a")); String ref = f.ref("Root", "Bad"); f.rule("Root", f.seq("Root", ref, f.literal("Root", "z")));
        f.recoveries.add(obj("ruleId", "Bad", "mode", "SYNC", "resolvedSyncPatterns", List.of(";"), "consumeSync", true, "noSyncAction", "fail", "progressRule", "legacySync", "applicationSites", List.of(ref), "astProjection", "mappingFailure", "recipeId", null));
        try (var h = f.compile()) {
            Object result = h.parse("bad;z", h.options(false, true, false, true, Map.of())); assertTrue(ok(result)); assertEquals(5, get(result, "consumedCp"));
            assertFalse(ok(h.parse("badz", h.options(false, true, false, true, Map.of()))));
            var error = assertThrows(InvocationTargetException.class, () -> h.parse("bad;z")).getCause(); assertEquals("MappingException", error.getClass().getSimpleName());
        }
    }
    @Test void nestedRecordsCommonFieldsAndEnum() throws Exception {
        var f = new IrFixture(); String value = f.literal("Item", "a"); String site = f.capture(value, "value", "text", "scalar", obj("kind", "text")); f.rule("Item", value);
        f.mapping("Item", "Item", List.of(field("value", obj("kind", "text"), "scalar")), List.of(recipeField("value", site, "text", "scalar")));
        f.types.getFirst().put("enclosingTypeId", "Node"); f.types.getFirst().put("implements", List.of("Node"));
        f.types.add(obj("id", "Node", "name", "Node", "kind", "transparentUnion", "enclosingTypeId", null, "variants", List.of("Item")));
        f.commons.add(obj("interfaceTypeId", "Node", "variantTypeIds", List.of("Item"), "field", field("value", obj("kind", "text"), "scalar")));
        String color = f.choice("Color", "ordered", f.literal("Color", "red"), f.literal("Color", "blue")); f.rule("Color", color);
        f.types.add(obj("id", "Color", "name", "Color", "kind", "enum", "enclosingTypeId", null, "ruleId", "Color", "variants", List.of(obj("literal", "red", "variant", "Red", "javaName", "RED", "rustName", "Red"), obj("literal", "blue", "variant", "Blue", "javaName", "BLUE", "rustName", "Blue"))));
        f.recipes.add(obj("id", "recipe:Color", "ruleId", "Color", "resultTypeId", "Color", "kind", "enum", "fields", List.of(), "phase", "selectedMapping", "onFailure", "mappingFailure", "transformId", null, "leafFallback", null));
        f.rule("Root", f.choice("Root", "ordered", f.ref("Root", "Item"), f.ref("Root", "Color")));
        try (var h = f.compile()) {
            assertEquals("Item", Json.object(h.canonical(h.parse("a"))).get("type")); assertEquals("RED", h.canonical(h.parse("red")));
            assertTrue(h.loader.loadClass(h.pkg + ".ProbeAST$Node").isSealed());
            assertEquals(String.class, h.loader.loadClass(h.pkg + ".ProbeAST$Node").getMethod("value").getReturnType());
        }
    }
    @Test void separatedPairRollbackAndCodePointTokens() throws Exception {
        var f = new IrFixture(); String elem = f.token("Root", "QUOTED", obj("kind", "builtin", "which", "Quoted", "codePoints", List.of()));
        f.rule("Root", f.expression("Root", "separated", "element", elem, "separator", f.literal("Root", ","), "min", 1, "pairRollback", true));
        try (var h = f.compile()) {
            Object result = h.parse("\"😀\",\"\\😀\","); assertFalse(ok(result)); assertEquals(8, get(result, "consumedCp"));
            assertTrue(ok(h.parse("\"😀\",\"\\😀\"")));
        }
    }
    @Test void optionalAndListCardinalityPreserveEmptyCaptures() throws Exception {
        var f = new IrFixture();
        String repeated = f.literal("Root", "a"), optional = f.literal("Root", "b"), empty = f.seq("Root");
        String a = f.capture(repeated, "items", "text", "list", obj("kind", "text"));
        String b = f.capture(optional, "maybe", "text", "optional", obj("kind", "text"));
        String e = f.capture(empty, "empty", "text", "scalar", obj("kind", "text"));
        f.rule("Root", f.seq("Root", f.expression("Root", "repeat", "child", repeated, "min", 0, "max", null, "terminator", null), f.expression("Root", "optional", "child", optional), empty));
        f.mapping("Root", "Values", List.of(field("items", obj("kind", "text"), "list"), field("maybe", obj("kind", "text"), "optional"), field("empty", obj("kind", "text"), "scalar")),
            List.of(recipeField("items", a, "text", "list"), recipeField("maybe", b, "text", "optional"), recipeField("empty", e, "text", "scalar")));
        try (var h = f.compile()) {
            Object result = h.parse("aab");
            assertEquals(Json.read("{\"items\":[\"a\",\"a\"],\"maybe\":\"b\",\"empty\":\"\"}"), Json.object(h.canonical(result)).get("fields"));
            var captures = (List<?>) get(result, "captures"); assertEquals(4, captures.size());
            assertEquals(get(get(captures.getLast(), "span"), "start"), get(get(captures.getLast(), "span"), "end"));
            assertEquals(Json.read("{\"items\":[],\"maybe\":null,\"empty\":\"\"}"), Json.object(h.canonical(h.parse(""))).get("fields"));
        }
    }
    @Test void loweredLeftRecursionIsUsedAndUnloweredCyclesAreRejected() throws Exception {
        var f = new IrFixture(); String original = f.ref("Root", "Root");
        String rewritten = f.seq("Root", f.literal("Root", "a"), f.expression("Root", "repeat", "child", f.literal("Root", "+a"), "min", 0, "max", null, "terminator", null));
        f.rule("Root", rewritten);
        f.semantics.put("normalizations", List.of(obj("ruleId", "Root", "originalBody", original, "rewrittenBody", rewritten, "kind", "directLeftRecursionExtension", "proof", obj("kind", "leftRecursionNormalization", "exprIds", List.of(original, rewritten), "explanation", "Base followed by nonnullable suffix loop"))));
        try (var h = f.compile()) { assertTrue(ok(h.parse("a+a+a"))); }
        f.rules.getFirst().put("body", original); f.semantics.put("normalizations", List.of());
        assertTrue(assertThrows(IllegalArgumentException.class, f::build).getMessage().contains("unloweredLeftRecursion"));
    }
    @Test void scopeTrimMovesCodePointOffset() throws Exception {
        var f = new IrFixture();
        String name = f.seq("Root", f.literal("Root", "\u0000\t name \r\n"));
        String site = f.capture(name, "name", "text", "scalar", obj("kind", "text"));
        f.rule("Root", f.seq("Root", f.literal("Root", "😀"), name));
        f.effects.add(obj("ruleId", "Root", "mode", "lexical", "events", List.of(obj("when", "successAfterLeave", "action", "declare", "captureSiteId", site, "name", "var"))));
        try (var h = f.compile()) {
            Object result = h.parse("😀\u0000\t name \r\n");
            Object declaration = ((List<?>) get(get(result, "scope"), "declarations")).getFirst();
            assertEquals("name", get(declaration, "name")); assertEquals(4, get(declaration, "offsetCp"));
        }
    }

    @Test void mixedCaptureAndTypeNameShadowingCompileWithoutWarnings() throws Exception {
        var f = new IrFixture(); String item = f.literal("Item", "a");
        String leaf = f.capture(item, "text", "text", "scalar", obj("kind", "text")); f.rule("Item", item);
        f.mapping("Item", "String", List.of(field("text", obj("kind", "text"), "scalar")), List.of(recipeField("text", leaf, "text", "scalar")));
        String choice = f.choice("Root", "ordered", f.ref("Root", "Item"), f.literal("Root", "b"));
        var mixed = obj("kind", "mixed", "members", List.of(obj("kind", "text"), obj("kind", "node", "typeId", "String")));
        String site = f.capture(choice, "value", "mixed", "scalar", mixed); f.rule("Root", choice);
        f.mapping("Root", "Box", List.of(field("value", mixed, "scalar")), List.of(recipeField("value", site, "mixed", "scalar")));
        try (var h = f.compile()) {
            assertEquals(Map.of("value", "b"), Json.object(h.canonical(h.parse("b"))).get("fields"));
            var value = Json.object(Json.object(Json.object(h.canonical(h.parse("a"))).get("fields")).get("value"));
            assertEquals("String", value.get("type")); assertEquals(Map.of("text", "a"), value.get("fields"));
        }
    }

}
