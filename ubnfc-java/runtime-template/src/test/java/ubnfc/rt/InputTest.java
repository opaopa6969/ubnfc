// ubnfc runtime template: InputTest
package ubnfc.rt;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

class InputTest {
    @ParameterizedTest
    @ValueSource(strings = {"", "ascii", "日本語", "a😀b😀終", "😀😀", "\ud800x\udc00", "\ud800😀\udc00"})
    void everyCodePointBoundaryRoundTrips(String source) {
        Input input = new Input(source);
        int count = source.codePointCount(0, source.length());
        assertEquals(source.length(), input.length());
        for (int cp = 0; cp <= count; cp++) {
            int utf16 = source.offsetByCodePoints(0, cp);
            assertEquals(utf16, input.utf16Index(cp));
            assertEquals(cp, input.cpIndex(utf16));
            assertEquals(cp, input.cpLength(0, utf16));
            if (cp < count) {
                assertEquals(source.codePointAt(utf16), input.codePointAt(utf16));
            }
            for (int endCp = cp; endCp <= count; endCp++) {
                assertEquals(endCp - cp, input.cpLength(utf16, source.offsetByCodePoints(0, endCp)));
            }
        }
    }

    @Test
    void supplementaryOffsetsHaveExplicitExpectedValues() {
        Input input = new Input("a😀b😀終");
        int[] boundaries = {0, 1, 3, 4, 6, 7};
        for (int cp = 0; cp < boundaries.length; cp++) {
            assertEquals(boundaries[cp], input.utf16Index(cp));
            assertEquals(cp, input.cpIndex(boundaries[cp]));
        }
        assertEquals(0x1f600, input.codePointAt(1));
        assertEquals(3, input.cpLength(1, 6));
    }

    @Test
    void literalsUsePrefixMatchingWithoutWordBoundaries() {
        Input input = new Input("letter😀suffix");
        assertTrue(input.startsWith(0, "let"));
        assertTrue(input.startsWith(6, "😀"));
        assertFalse(input.startsWith(0, "Let"));
        assertFalse(input.startsWith(7, "😀"));
        assertTrue(input.startsWith(input.length(), ""));
        assertFalse(input.startsWith(input.length(), "x"));
    }

    @Test
    void literalComparisonMatchesJdkIncludingSupplementaryCaseAndMalformedUtf16() {
        String[] values = {"", "Iıİi", "Σσς", "Kk", "ßSS", "ſS", "😀", "\ud801\udc00",
                "\ud801\udc28", "\ud83a\udd00", "\ud83a\udd22", "\ud800", "\udc00", "A\ud800B"};
        for (String source : values) {
            Input input = new Input(source);
            for (String literal : values) {
                for (int offset = -1; offset <= source.length() + 1; offset++) {
                    assertEquals(source.startsWith(literal, offset), input.startsWith(offset, literal));
                    assertEquals(source.regionMatches(true, offset, literal, 0, literal.length()),
                            input.startsWithIgnoreCase(offset, literal));
                }
            }
        }
        Input input = new Input("x");
        assertFalse(input.startsWith(Integer.MAX_VALUE, ""));
        assertFalse(input.startsWith(Integer.MIN_VALUE, ""));
        assertFalse(input.startsWithIgnoreCase(Integer.MAX_VALUE, ""));
    }

    @Test
    void positionsInsideSurrogatePairsAreNotPublicSpanBoundaries() {
        Input input = new Input("a😀b");
        assertThrows(IllegalArgumentException.class, () -> input.cpIndex(2));
        assertThrows(IllegalArgumentException.class, () -> input.cpLength(1, 2));
        assertThrows(IllegalArgumentException.class, () -> input.cpLength(2, 3));
    }

    @Test
    void rejectsOutOfRangeIndicesAndReversedSpans() {
        Input input = new Input("😀");
        assertThrows(IndexOutOfBoundsException.class, () -> input.cpIndex(-1));
        assertThrows(IndexOutOfBoundsException.class, () -> input.cpIndex(3));
        assertThrows(IndexOutOfBoundsException.class, () -> input.utf16Index(-1));
        assertThrows(IndexOutOfBoundsException.class, () -> input.utf16Index(2));
        assertThrows(IndexOutOfBoundsException.class, () -> input.codePointAt(2));
        assertThrows(IndexOutOfBoundsException.class, () -> input.cpLength(2, 0));
        assertThrows(IndexOutOfBoundsException.class, () -> new Input("").codePointAt(0));
    }
}
