// ubnfc runtime template: CursorTest
package ubnfc.rt;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;

class CursorTest {
    @Test
    void consumedAndMatchedAdvanceIndependently() {
        Cursor cursor = new Cursor();
        cursor.advanceMatchedOnly(2);
        assertEquals(0, cursor.consumed);
        assertEquals(2, cursor.matched);
        cursor.advance(3);
        assertEquals(3, cursor.consumed);
        assertEquals(5, cursor.matched);
        cursor.resetMatchedToConsumed();
        assertEquals(3, cursor.matched);
    }

    @Test
    void nestedCheckpointsPreserveBothPositions() {
        Cursor cursor = new Cursor(7, 11);
        long outer = cursor.checkpoint();
        cursor.advance(2);
        long inner = cursor.checkpoint();
        cursor.advanceMatchedOnly(5);
        cursor.restore(inner);
        assertEquals(9, cursor.consumed);
        assertEquals(13, cursor.matched);
        cursor.restore(outer);
        assertEquals(7, cursor.consumed);
        assertEquals(11, cursor.matched);
    }

    @Test
    void packingPreservesHighPositionBits() {
        Cursor cursor = new Cursor(Integer.MAX_VALUE - 3, Integer.MAX_VALUE);
        long checkpoint = cursor.checkpoint();
        cursor.restore(0);
        cursor.restore(checkpoint);
        assertEquals(Integer.MAX_VALUE - 3, cursor.consumed);
        assertEquals(Integer.MAX_VALUE, cursor.matched);
    }

    @Test
    void invalidAdvanceDoesNotPartiallyModifyCursor() {
        Cursor cursor = new Cursor(1, Integer.MAX_VALUE);
        long original = cursor.checkpoint();
        assertThrows(ArithmeticException.class, () -> cursor.advance(1));
        assertEquals(original, cursor.checkpoint());
        assertThrows(ArithmeticException.class, () -> cursor.advanceMatchedOnly(1));
        assertThrows(IllegalArgumentException.class, () -> cursor.advance(-1));
        assertThrows(IllegalArgumentException.class, () -> cursor.advanceMatchedOnly(-1));
        assertEquals(original, cursor.checkpoint());
        assertThrows(IllegalArgumentException.class, () -> new Cursor(2, 1));
    }

    @Test
    void zeroWidthIterationHasNoCursorProgress() {
        Cursor cursor = new Cursor();
        long before = cursor.checkpoint();
        cursor.advance(0);
        assertEquals(before, cursor.checkpoint());
        cursor.advanceMatchedOnly(1);
        assertEquals(0, cursor.consumed);
        assertEquals(1, cursor.matched);
    }
}
