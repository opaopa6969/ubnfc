// ubnfc runtime template: MemoTable
package org.ubnfc.p4.generated.rt;

import java.util.Arrays;

/** 解析セッション専用の boxing なし memo table。payload は生成側 arena への int handle。 */
public final class MemoTable {
    public static final int MAX_EXPR_ID = 0x00ff_ffff;
    public static final int MAX_MODE = 0xff;
    private long[] expressionKeys;
    private long[] positionKeys;
    // 0 = empty, 1 = failure, 2 = success: zero-valued keys remain usable.
    private int[] outcomes;
    private int[] consumedEnds;
    private int[] matchedEnds;
    private int[] payloads;
    private int size;

    public MemoTable() {
        this(16);
    }

    public MemoTable(int initialCapacity) {
        if (initialCapacity < 2 || initialCapacity > (1 << 29)) {
            throw new IllegalArgumentException("Capacity must be between 2 and 2^29");
        }
        int capacity = 2;
        while (capacity < initialCapacity) {
            capacity <<= 1;
        }
        allocate(capacity);
    }

    /** 未登録は -1。返した slot は次の put/clear まで有効。 */
    public int get(int exprId, int consumed, int matched, int mode, int stateVersion) {
        long expression = expressionKey(exprId, mode, stateVersion);
        long position = positionKey(consumed, matched);
        int slot = locate(expression, position);
        return outcomes[slot] == 0 ? -1 : slot;
    }

    public int put(int exprId, int consumed, int matched, int mode, int stateVersion,
                   boolean ok, int consumedEnd, int matchedEnd, int payload) {
        long expression = expressionKey(exprId, mode, stateVersion);
        long position = positionKey(consumed, matched);
        int slot = locate(expression, position);
        if (outcomes[slot] == 0) {
            if (size == outcomes.length / 2) {
                grow();
                slot = locate(expression, position);
            }
            expressionKeys[slot] = expression;
            positionKeys[slot] = position;
            size++;
        }
        outcomes[slot] = ok ? 2 : 1;
        consumedEnds[slot] = consumedEnd;
        matchedEnds[slot] = matchedEnd;
        payloads[slot] = payload;
        return slot;
    }

    public boolean ok(int slot) {
        checkSlot(slot);
        return outcomes[slot] == 2;
    }

    public int consumedEnd(int slot) {
        checkSlot(slot);
        return consumedEnds[slot];
    }

    public int matchedEnd(int slot) {
        checkSlot(slot);
        return matchedEnds[slot];
    }

    public int payload(int slot) {
        checkSlot(slot);
        return payloads[slot];
    }

    public void clear() {
        Arrays.fill(outcomes, 0);
        size = 0;
    }

    public int size() {
        return size;
    }

    public int capacity() {
        return outcomes.length;
    }

    private static long expressionKey(int exprId, int mode, int stateVersion) {
        // 24 + 8 + 32 bits. Reject overflow rather than silently aliasing distinct keys.
        if ((exprId & ~MAX_EXPR_ID) != 0 || (mode & ~MAX_MODE) != 0) {
            throw new IllegalArgumentException("ExprId requires 24 bits and mode requires 8 bits");
        }
        return ((long) exprId << 40) | ((long) mode << 32) | (stateVersion & 0xffff_ffffL);
    }

    private static long positionKey(int consumed, int matched) {
        if (consumed < 0 || matched < 0) {
            throw new IllegalArgumentException("Negative UTF-16 position");
        }
        return ((long) consumed << 32) | (matched & 0xffff_ffffL);
    }

    private int locate(long expression, long position) {
        long hash = mix(expression) ^ Long.rotateLeft(mix(position), 23);
        int mask = outcomes.length - 1;
        int slot = (int) (hash ^ (hash >>> 32)) & mask;
        while (outcomes[slot] != 0
                && (expressionKeys[slot] != expression || positionKeys[slot] != position)) {
            slot = (slot + 1) & mask;
        }
        return slot;
    }

    private static long mix(long value) {
        value ^= value >>> 30;
        value *= 0xbf58_476d_1ce4_e5b9L;
        value ^= value >>> 27;
        value *= 0x94d0_49bb_1331_11ebL;
        return value ^ (value >>> 31);
    }

    private void grow() {
        if (outcomes.length == (1 << 29)) {
            throw new IllegalStateException("Memo table capacity exhausted");
        }
        long[] oldExpressions = expressionKeys;
        long[] oldPositions = positionKeys;
        int[] oldOutcomes = outcomes;
        int[] oldConsumed = consumedEnds;
        int[] oldMatched = matchedEnds;
        int[] oldPayloads = payloads;
        allocate(oldOutcomes.length << 1);
        for (int i = 0; i < oldOutcomes.length; i++) {
            if (oldOutcomes[i] != 0) {
                int slot = locate(oldExpressions[i], oldPositions[i]);
                expressionKeys[slot] = oldExpressions[i];
                positionKeys[slot] = oldPositions[i];
                outcomes[slot] = oldOutcomes[i];
                consumedEnds[slot] = oldConsumed[i];
                matchedEnds[slot] = oldMatched[i];
                payloads[slot] = oldPayloads[i];
            }
        }
    }

    private void allocate(int capacity) {
        expressionKeys = new long[capacity];
        positionKeys = new long[capacity];
        outcomes = new int[capacity];
        consumedEnds = new int[capacity];
        matchedEnds = new int[capacity];
        payloads = new int[capacity];
    }

    private void checkSlot(int slot) {
        if (slot < 0 || slot >= outcomes.length || outcomes[slot] == 0) {
            throw new IndexOutOfBoundsException("No memo entry at slot " + slot);
        }
    }
}
