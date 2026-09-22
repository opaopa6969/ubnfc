package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.CompilerHarness.*;
import static org.ubnfc.java.IrFixture.*;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

class Round2Test {
    @Test void quantifiedCapturesMapIndividualValuesAndKeepEmptyOptionalAbsent() throws Exception {
        var f = new IrFixture();
        String repeated = f.expression("Root", "repeat", "child", f.expression("Root", "any"), "min", 0, "max", null, "terminator", null);
        String site = f.capture(repeated, "values", "text", "list", obj("kind", "text"));
        f.rule("Root", repeated);
        f.mapping("Root", "Values", List.of(field("values", obj("kind", "text"), "list")), List.of(recipeField("values", site, "text", "list")));
        try (var h = f.compile()) {
            assertEquals(List.of("a", "😀"), org.ubnfc.java.ir.Json.object(org.ubnfc.java.ir.Json.object(h.canonical(h.parse("a😀"))).get("fields")).get("values"));
            assertEquals(List.of(), get(h.parse(""), "captures"));
        }
    }
    @Test void quotedMappedTextSurvivesUnmappedHelperRule() throws Exception {
        var f = new IrFixture();
        f.rule("Helper", f.token("Helper", "STRING", obj("kind", "builtin", "which", "SingleQuoted", "codePoints", List.of())));
        String helper = f.ref("Root", "Helper");
        String site = f.capture(helper, "value", "text", "scalar", obj("kind", "text")); f.rule("Root", helper);
        f.mapping("Root", "Value", List.of(field("value", obj("kind", "text"), "scalar")), List.of(recipeField("value", site, "text", "scalar")));
        try (var h = f.compile()) {
            var ast = org.ubnfc.java.ir.Json.object(h.canonical(h.parse("'😀'")));
            assertEquals("😀", org.ubnfc.java.ir.Json.object(ast.get("fields")).get("value"));
        }
    }
    @Test void scopeUsesEveryRepeatedCaptureAndRollbackRestoresParentState() throws Exception {
        var f = new IrFixture();
        String item = f.literal("Root", "a");
        String site = f.capture(item, "name", "text", "list", obj("kind", "text"));
        f.rule("Root", f.expression("Root", "repeat", "child", item, "min", 1, "max", null, "terminator", null));
        f.effects.add(obj("ruleId", "Root", "mode", "lexical", "events", List.of(obj("when", "successAfterLeave", "action", "declare", "captureSiteId", site, "name", "name"))));
        try (var h = f.compile()) {
            for (boolean memo : List.of(false, true)) {
                Object options = h.options(false, false, true, memo, Map.of());
                var declarations = (List<?>) get(get(h.parse("aa", options), "scope"), "declarations");
                assertEquals(2, declarations.size()); assertEquals(0, get(declarations.get(0), "offsetCp")); assertEquals(1, get(declarations.get(1), "offsetCp"));
                Object rolled = h.parser.getMethod("parseEntryRollback", String.class, String.class, CharSequence.class, h.options).invoke(null, "Probe", null, "aa", options);
                assertEquals(0, get(rolled, "consumedCp")); assertEquals(0, get(rolled, "matchedCp"));
                assertEquals(List.of(), get(get(rolled, "scope"), "declarations"));
            }
        }
    }
    @Test void lexicalIncludesThreeUncapturedAdoptedTokensOnly() throws Exception {
        var f = new IrFixture();
        f.rule("Root", f.choice("Root", "ordered", f.seq("Root", f.literal("Root", "a"), f.literal("Root", "b"), f.literal("Root", "x")), f.literal("Root", "a")));
        try (var h = f.compile()) {
            var lexical = (List<?>) get(h.parse("abx", h.options(false, true, true, true, Map.of())), "lexical");
            var tokens = new java.util.ArrayList<Object>();
            for (Object occurrence : lexical) if (get(occurrence, "token") != null) tokens.add(get(occurrence, "token"));
            assertEquals(List.of("a", "b", "x"), tokens);
        }
    }
    private static IrFixture regex(String pattern, Map<String, Object> dfa) throws Exception {
        var f = new IrFixture();
        f.rule("Root", f.token("Root", "RX", obj("kind", "regex", "pattern", pattern, "regularSubset", true,
            "matching", "lookingAt", "zeroWidth", "fail", "javaFallback", "java.util.regex.Pattern", "rustFallback", "generationError", "dfa", dfa)));
        return f;
    }
    private static Map<String, Object> dfa(List<Integer> accepting, List<Map<String, Object>> transitions) {
        return obj("states", 3, "start", 0, "accepting", accepting, "transitions", transitions, "semantics", "leftmostFirst", "anchoredAtStart", true, "zeroWidth", "fail");
    }
    @Test void orderedDfaUsesLastPositiveCandidateAndCodePoints() throws Exception {
        // The compiled plan is authoritative, and may omit lower-priority alternative transitions.
        var first = regex("a|ab", dfa(List.of(1), List.of(obj("from", 0, "to", 1, "ranges", List.of(List.of(97, 97))))));
        try (var h = first.compile()) {
            Object result = h.parse("ab", h.options(false, false, true, true, Map.of()));
            assertTrue(ok(result)); assertEquals(1, get(result, "consumedCp"));
        }
        var greedy = regex(".+", dfa(List.of(1), List.of(
            obj("from", 0, "to", 1, "ranges", List.of(List.of(0x1f600, 0x1f600))),
            obj("from", 1, "to", 1, "ranges", List.of(List.of(0x1f600, 0x1f600))),
            obj("from", 1, "to", 2, "ranges", List.of(List.of(120, 120))))));
        try (var h = greedy.compile()) {
            Object result = h.parse("😀😀x", h.options(false, false, false, true, Map.of()));
            assertTrue(ok(result)); assertEquals(2, get(result, "consumedCp"));
        }
        try (var h = regex("a*", dfa(List.of(0), List.of())).compile()) { assertFalse(ok(h.parse(""))); }
        try (var h = regex("a|ab", null).compile()) { assertEquals(1, get(h.parse("ab", h.options(false, false, false, true, Map.of())), "consumedCp")); }
    }
    @Test void malformedDfaCannotSilentlyFallBack() throws Exception {
        var bad = dfa(List.of(1), List.of(obj("from", 0, "to", 9, "ranges", List.of(List.of(97, 97)))));
        assertThrows(IllegalArgumentException.class, () -> regex("a", bad).build());
        bad.put("semantics", "unknown");
        assertThrows(IllegalArgumentException.class, () -> regex("a", bad).build());
    }
}
