package org.ubnfc.p4.lsp;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.eclipse.lsp4j.Position;
import org.junit.jupiter.api.Test;

/** 生成された {@link Positions} の code point ↔ UTF-16 変換。BMP 外と改行 3 種を固定する。 */
class PositionsTest {
    private static final String EMOJI = "😀"; // U+1F600: 1 code point / 2 UTF-16 code unit

    @Test void asciiOffsetsAreIdentity() {
        var positions = new Positions("abc\ndef\n");
        assertEquals(new Position(0, 0), positions.positionOf(0));
        assertEquals(new Position(0, 3), positions.positionOf(3));
        assertEquals(new Position(1, 0), positions.positionOf(4));
        assertEquals(new Position(2, 0), positions.positionOf(8));
        assertEquals(8, positions.lengthCp());
    }

    @Test void nonBmpCharactersCountTwoUtf16Units() {
        // 1 行目: "x" + 😀 + 😀 + "y" = 4 code point / 6 UTF-16 code unit
        var positions = new Positions("x" + EMOJI + EMOJI + "y\nz");
        assertEquals(new Position(0, 1), positions.positionOf(1));
        assertEquals(new Position(0, 3), positions.positionOf(2));
        assertEquals(new Position(0, 5), positions.positionOf(3));
        assertEquals(new Position(0, 6), positions.positionOf(4));
        // 改行のあとは行が変わるので column は 0 に戻る（前の行の BMP 外は影響しない）
        assertEquals(new Position(1, 0), positions.positionOf(5));
        assertEquals(new Position(1, 1), positions.positionOf(6));
    }

    @Test void previousLineNonBmpDoesNotShiftLaterLines() {
        // 旧実装の誤り（code point 位置を UTF-16 index として使う）はここで column が 2 ずれた。
        var positions = new Positions("'" + EMOJI + EMOJI + "'\ncall bonus(1)\n");
        // 1 行目 "'😀😀'" は 4 code point / 6 UTF-16 code unit。2 行目の先頭は code point 5。
        assertEquals(new Position(1, 5), positions.positionOf(10));
        assertEquals(new Position(1, 10), positions.positionOf(15));
    }

    @Test void handlesCrlfAndLoneCr() {
        var positions = new Positions("a\r\nb\rc");
        assertEquals(new Position(0, 1), positions.positionOf(1));
        assertEquals(new Position(1, 0), positions.positionOf(3));
        assertEquals(new Position(2, 0), positions.positionOf(5));
    }

    @Test void roundTripsThroughOffsetOf() {
        String text = "var $x" + EMOJI + " = 1;\ncall f(" + EMOJI + ");\n";
        var positions = new Positions(text);
        for (int cp = 0; cp <= positions.lengthCp(); cp++)
            assertEquals(cp, positions.offsetOf(positions.positionOf(cp)), "code point " + cp);
    }

    @Test void clampsOutOfRange() {
        var positions = new Positions("ab\n");
        assertEquals(new Position(0, 0), positions.positionOf(-5));
        assertEquals(new Position(1, 0), positions.positionOf(99));
        assertEquals(3, positions.offsetOf(new Position(99, 99)));
    }
}
