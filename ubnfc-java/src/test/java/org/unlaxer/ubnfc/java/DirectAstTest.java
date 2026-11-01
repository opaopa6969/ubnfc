package org.unlaxer.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.unlaxer.ubnfc.java.CompilerHarness.*;
import static org.unlaxer.ubnfc.java.IrFixture.*;

import java.lang.reflect.InvocationTargetException;
import java.util.List;
import org.junit.jupiter.api.Test;

/** 最終木だけを構築する契約。投機的な規則成功は AST の公開を意味しない。 */
class DirectAstTest {
    private static IrFixture grammar(String strategy, String rejected) throws Exception {
        var f = new IrFixture();
        var numeric = obj("kind", "number", "java", "int32Exact", "rust", "lexeme");
        String word = f.literal("Discarded", rejected);
        String site = f.capture(word, "value", "number", "scalar", numeric);
        f.rule("Discarded", word);
        f.mapping("Discarded", "Discarded", List.of(field("value", numeric, "scalar")),
            List.of(recipeField("value", site, "number", "scalar")));
        String accepted = f.literal("Selected", rejected + "!");
        String text = f.capture(accepted, "value", "text", "scalar", obj("kind", "text"));
        f.rule("Selected", accepted);
        f.mapping("Selected", "Selected", List.of(field("value", obj("kind", "text"), "scalar")),
            List.of(recipeField("value", text, "text", "scalar")));
        String candidate = f.ref("Root", "Discarded");
        if (strategy.equals("ordered")) candidate = f.seq("Root", candidate, f.literal("Root", "?"));
        f.rule("Root", f.choice("Root", strategy, candidate, f.ref("Root", "Selected")));
        return f;
    }
    private static void direct(CompilerHarness h, boolean enabled) throws Exception {
        h.loader.loadClass(h.pkg + ".internal.Session").getField("directAst").setBoolean(null, enabled);
    }
    @Test void discardedOverflowDoesNotEscapeOrderedOrLongestChoice() throws Exception {
        for (String strategy : List.of("ordered", "longest")) {
            try (var h = new CompilerHarness(grammar(strategy, "999999999999999999999").build(), "direct." + strategy)) {
                for (boolean memo : List.of(false, true)) {
                    Object opts = h.options(true, true, false, memo, java.util.Map.of());
                    direct(h, false); Object reference = h.parse("999999999999999999999!", opts);
                    direct(h, true); Object result = h.parse("999999999999999999999!", opts);
                    assertTrue(ok(result));
                    assertEquals(h.canonical(reference), h.canonical(result));
                    assertEquals(((java.util.Map<?, ?>) get(reference, "nodeSpans")).size(),
                        ((java.util.Map<?, ?>) get(result, "nodeSpans")).size());
                }
            }
        }
    }
    @Test void discardedSuccessfulNodesDoNotAppearInPublicSpans() throws Exception {
        try (var h = new CompilerHarness(grammar("longest", "1").build(), "direct.spans")) {
            for (boolean enabled : List.of(false, true)) {
                direct(h, enabled);
                Object result = h.parse("1!");
                assertTrue(ok(result));
                var spans = (java.util.Map<?, ?>) get(result, "nodeSpans");
                assertTrue(spans.keySet().stream().noneMatch(x -> x.getClass().getSimpleName().equals("Discarded")));
                assertEquals(2, spans.size(), "採用された AST と text だけを公開する");
            }
        }
    }
    @Test void selectedOverflowKeepsSyntaxSpanMessageAndCause() throws Exception {
        try (var h = new CompilerHarness(grammar("longest", "999999999999999999999").build(), "direct.failure")) {
            Object expected = null;
            for (boolean enabled : List.of(false, true)) {
                direct(h, enabled);
                Throwable failure = assertThrows(InvocationTargetException.class, () -> h.parser
                    .getMethod("parseEntry", String.class, String.class, CharSequence.class, h.options)
                    .invoke(null, "Probe", "Discarded", "999999999999999999999", h.options.getField("DEFAULT").get(null))).getCause();
                assertEquals("MappingException", failure.getClass().getSimpleName());
                Object syntax = get(failure, "syntax");
                assertTrue(ok(syntax));
                var observation = List.of(failure.getMessage(), get(failure, "span"),
                    failure.getCause().getClass().getName(), failure.getCause().getMessage(),
                    get(syntax, "consumedCp"), get(syntax, "diagnostics"));
                if (!enabled) expected = observation; else assertEquals(expected, observation);
            }
        }
    }
}
