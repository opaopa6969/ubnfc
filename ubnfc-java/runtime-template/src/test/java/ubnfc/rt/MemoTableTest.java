// ubnfc runtime template: MemoTableTest
package ubnfc.rt;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;

class MemoTableTest {
    @Test
    void storesOneHundredThousandEntriesAcrossGrowth() {
        MemoTable table = new MemoTable(2);
        int count = 100_000;
        for (int i = 0; i < count; i++) {
            table.put(i, 2 * i, 2 * i + 1, i & 255, i / 7,
                    (i & 1) == 0, 2 * i + 3, 2 * i + 5, 3 * i + 1);
            assertTrue(table.size() <= table.capacity() / 2);
        }
        assertEquals(count, table.size());
        assertEquals(262_144, table.capacity());
        // 互いに素な歩幅で、挿入順に依存せず全件を検証する。
        for (int n = 0; n < count; n++) {
            int i = (int) ((long) n * 79_919 % count);
            int slot = table.get(i, 2 * i, 2 * i + 1, i & 255, i / 7);
            assertTrue(slot >= 0, "Missing entry " + i);
            assertEquals((i & 1) == 0, table.ok(slot));
            assertEquals(2 * i + 3, table.consumedEnd(slot));
            assertEquals(2 * i + 5, table.matchedEnd(slot));
            assertEquals(3 * i + 1, table.payload(slot));
        }
        assertEquals(-1, table.get(count, 0, 0, 0, 0));
    }

    @Test
    void everyKeyComponentDistinguishesEntries() {
        int[][] keys = {
                {0, 0, 0, 0, 0}, {1, 0, 0, 0, 0}, {0, 1, 0, 0, 0},
                {0, 0, 1, 0, 0}, {0, 0, 0, 1, 0}, {0, 0, 0, 0, 1},
                {0x80_0000, 0, 0, 0, 0}, {0, Integer.MAX_VALUE, 0, 0, 0},
                {0, 0, Integer.MAX_VALUE, 0, 0}, {0, 0, 0, 255, 0},
                {0, 0, 0, 0, Integer.MIN_VALUE}, {0, 0, 0, 0, Integer.MAX_VALUE},
                {MemoTable.MAX_EXPR_ID, Integer.MAX_VALUE, Integer.MAX_VALUE, 255, -1}
        };
        MemoTable table = new MemoTable();
        for (int i = 0; i < keys.length; i++) {
            int[] k = keys[i];
            table.put(k[0], k[1], k[2], k[3], k[4], true, 0, 0, i);
        }
        assertEquals(keys.length, table.size());
        for (int i = 0; i < keys.length; i++) {
            int[] k = keys[i];
            assertEquals(i, table.payload(table.get(k[0], k[1], k[2], k[3], k[4])));
        }
    }

    @Test
    void overwritingAtHalfFullDoesNotGrowOrIncreaseSize() {
        MemoTable table = new MemoTable(2);
        table.put(0, 0, 0, 0, 0, true, 10, 12, 1);
        int slot = table.put(0, 0, 0, 0, 0, false, 0, 3, -1);
        assertEquals(1, table.size());
        assertEquals(2, table.capacity());
        assertFalse(table.ok(slot));
        assertEquals(0, table.consumedEnd(slot));
        assertEquals(3, table.matchedEnd(slot));
        assertEquals(-1, table.payload(slot));
    }

    @Test
    void clearRemovesSuccessAndFailureEntriesAndAllowsReuse() {
        MemoTable table = new MemoTable();
        table.put(0, 0, 0, 0, 0, false, 0, 0, 0);
        table.put(1, 2, 3, 4, 5, true, 8, 9, 10);
        int capacity = table.capacity();
        table.clear();
        assertEquals(0, table.size());
        assertEquals(-1, table.get(0, 0, 0, 0, 0));
        assertEquals(-1, table.get(1, 2, 3, 4, 5));
        assertEquals(capacity, table.capacity());
        int slot = table.put(0, 0, 0, 0, 0, true, 1, 2, 3);
        assertEquals(3, table.payload(slot));
    }

    @Test
    void linearProbeSurvivesCollisionsAndUpdates() {
        MemoTable table = new MemoTable(8);
        int initial = table.put(0, 0, 0, 0, 0, true, 0, 0, 10);
        int collider = -1;
        for (int expr = 1; expr < 1000; expr++) {
            table.clear();
            if (table.put(expr, 0, 0, 0, 0, true, 0, 0, 0) == initial) {
                collider = expr;
                break;
            }
        }
        assertTrue(collider > 0);
        table.clear();
        table.put(0, 0, 0, 0, 0, true, 0, 0, 10);
        table.put(collider, 0, 0, 0, 0, true, 0, 0, 20);
        assertNotEquals(table.get(0, 0, 0, 0, 0), table.get(collider, 0, 0, 0, 0));
        table.put(collider, 0, 0, 0, 0, false, 1, 2, 30);
        assertEquals(10, table.payload(table.get(0, 0, 0, 0, 0)));
        assertEquals(30, table.payload(table.get(collider, 0, 0, 0, 0)));
        assertEquals(2, table.size());
        assertEquals(-1, table.get(1001, 0, 0, 0, 0));
    }

    @Test
    void rejectsUnrepresentableKeysInsteadOfAliasingThem() {
        MemoTable table = new MemoTable();
        table.put(0, 0, 0, 0, 0, true, 0, 0, 0);
        assertThrows(IllegalArgumentException.class, () -> table.get(-1, 0, 0, 0, 0));
        assertThrows(IllegalArgumentException.class, () -> table.get(0x100_0000, 0, 0, 0, 0));
        assertThrows(IllegalArgumentException.class, () -> table.get(0, 0, 0, 256, 0));
        assertThrows(IllegalArgumentException.class, () -> table.get(0, 0, 0, -1, 0));
        assertThrows(IllegalArgumentException.class, () -> table.get(0, -1, 0, 0, 0));
        assertThrows(IllegalArgumentException.class, () -> table.get(0, 0, -1, 0, 0));
        assertThrows(IllegalArgumentException.class,
                () -> table.put(0x100_0000, 0, 0, 0, 0, true, 0, 0, 1));
        assertEquals(1, table.size());
        assertThrows(IndexOutOfBoundsException.class, () -> table.payload(-1));
    }
}
