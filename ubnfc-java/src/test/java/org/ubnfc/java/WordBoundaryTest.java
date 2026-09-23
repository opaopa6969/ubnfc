package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.CompilerHarness.*;
import static org.ubnfc.java.IrFixture.*;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

/**
 * リテラルの語境界（D-040）。識別子構成文字は ASCII [A-Za-z0-9_] だけで、
 * 境界違反はリテラル不一致とまったく同じ失敗記録になる。
 */
class WordBoundaryTest {
    private Object prefix(CompilerHarness h, String input) throws Exception {
        return h.parse(input, h.options(false, false, false, true, Map.of()));
    }

    @Test void boundaryRejectsOnlyWhenTheNextCodePointIsAnIdentifierCharacter() throws Exception {
        var off = new IrFixture(); off.rule("Root", off.literal("Root", "let", false));
        try (var h = off.compile()) {
            // v1: 境界なし。接頭辞一致のまま。
            for (String input : List.of("let", "letter", "let_", "let9", "let+")) {
                assertTrue(ok(prefix(h, input)), input);
            }
        }
        var on = new IrFixture(); on.rule("Root", on.literal("Root", "let", true));
        try (var h = on.compile()) {
            for (String input : List.of("let", "let+", "let ", "letあ", "let😀")) {
                assertTrue(ok(prefix(h, input)), input);
            }
            for (String input : List.of("letter", "let_", "let9", "letA")) {
                assertFalse(ok(prefix(h, input)), input);
            }
        }
    }

    @Test void aBoundaryViolationIsRecordedExactlyLikeAMismatch() throws Exception {
        var f = new IrFixture(); f.rule("Root", f.literal("Root", "let", true));
        try (var h = f.compile()) {
            Object violated = prefix(h, "letter"), mismatched = prefix(h, "lex");
            assertFalse(ok(violated)); assertFalse(ok(mismatched));
            assertEquals(get(mismatched, "consumedCp"), get(violated, "consumedCp"));
            Object a = get(violated, "diagnostics"), b = get(mismatched, "diagnostics");
            assertEquals(get(b, "offset"), get(a, "offset"));
            assertEquals(get(b, "expected"), get(a, "expected"));
            assertEquals(get(b, "farthestCp"), get(a, "farthestCp"));
        }
    }

    @Test void caseInsensitiveTokensFollowTheSameRule() throws Exception {
        for (boolean boundary : List.of(false, true)) {
            var f = new IrFixture();
            f.rule("Root", f.token("Root", "KW", obj("kind", "caseInsensitive", "word", "select",
                "comparison", "javaEqualsIgnoreCase", "wordBoundary", boundary)));
            try (var h = f.compile()) {
                assertTrue(ok(prefix(h, "SELECT")), "boundary=" + boundary);
                assertEquals(!boundary, ok(prefix(h, "SELECTed")), "boundary=" + boundary);
            }
        }
    }
}
