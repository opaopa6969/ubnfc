// ubnfc runtime template: MemoTable
package org.ubnfc.p4.generated.rt;

/** 解析セッション専用の boxing なし memo table。payload は生成側 arena への int handle。 */
public final class MemoTable {
    public static final int MAX_EXPR_ID = 0x00ff_ffff;
    public static final int MAX_MODE = 0xff;
    // 1 slot = 4 long: [expression key, position key, consumedEnd<<32 | matchedEnd, outcome<<32 | payload].
    // outcome 0 = empty, 1 = failure, 2 = success (so the last word is non-zero for every entry). One cache line per probe.
    private static final int STRIDE = 4;
    /** 解析ごとの大きな表を thread local に 1 組だけ持ち越す（内容は持ち越さない）。 */
    private static final ThreadLocal<long[]> POOL = new ThreadLocal<>();
    /** 持ち越す上限（32 MB）。これを超える表は毎回確保する。 */
    private static final int POOL_RETAIN_WORDS = 1 << 22;
    private long[] table;
    private int capacity;
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
        return table[slot * STRIDE + 3] == 0 ? -1 : slot;
    }

    public int put(int exprId, int consumed, int matched, int mode, int stateVersion,
                   boolean ok, int consumedEnd, int matchedEnd, int payload) {
        long expression = expressionKey(exprId, mode, stateVersion);
        long position = positionKey(consumed, matched);
        int slot = locate(expression, position);
        if (table[slot * STRIDE + 3] == 0) {
            if (size == capacity / 2) {
                grow();
                slot = locate(expression, position);
            }
            table[slot * STRIDE] = expression;
            table[slot * STRIDE + 1] = position;
            size++;
        }
        table[slot * STRIDE + 2] = ((long) consumedEnd << 32) | (matchedEnd & 0xffff_ffffL);
        table[slot * STRIDE + 3] = ((long) (ok ? 2 : 1) << 32) | (payload & 0xffff_ffffL);
        return slot;
    }

    public boolean ok(int slot) {
        checkSlot(slot);
        return (table[slot * STRIDE + 3] >>> 32) == 2;
    }

    public int consumedEnd(int slot) {
        checkSlot(slot);
        return (int) (table[slot * STRIDE + 2] >>> 32);
    }

    public int matchedEnd(int slot) {
        checkSlot(slot);
        return (int) table[slot * STRIDE + 2];
    }

    public int payload(int slot) {
        checkSlot(slot);
        return (int) table[slot * STRIDE + 3];
    }

    public void clear() {
        for (int i = 0; i < capacity; i++) table[i * STRIDE + 3] = 0;
        size = 0;
    }

    public int size() {
        return size;
    }

    public int capacity() {
        return capacity;
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
        // 1 回の乗算混合。上位 bit を使うので 2 冪 mask でも位置と式 id の下位 bit 偏りを拾わない。
        long hash = (expression ^ (position * 0x9e37_79b9_7f4a_7c15L)) * 0xbf58_476d_1ce4_e5b9L;
        int mask = capacity - 1;
        int slot = (int) (hash >>> 32) & mask;
        long[] t = table;
        while (t[slot * STRIDE + 3] != 0
                && (t[slot * STRIDE] != expression || t[slot * STRIDE + 1] != position)) {
            slot = (slot + 1) & mask;
        }
        return slot;
    }

    private void grow() {
        if (capacity == (1 << 29)) {
            throw new IllegalStateException("Memo table capacity exhausted");
        }
        long[] old = table;
        int oldCapacity = capacity;
        allocate(oldCapacity << 1);
        for (int i = 0; i < oldCapacity; i++) {
            if (old[i * STRIDE + 3] != 0) {
                int slot = locate(old[i * STRIDE], old[i * STRIDE + 1]);
                System.arraycopy(old, i * STRIDE, table, slot * STRIDE, STRIDE);
            }
        }
    }

    /**
     * 解析の終わりに表を pool へ返し、最小の表に戻す。内容は捨てるので次の解析の観測結果は変わらない。
     * 返した後も put/get は（小さい表で）正しく動く。
     */
    public void release() {
        long[] current = table;
        this.capacity = 2;
        table = new long[2 * STRIDE];
        size = 0;
        if (current.length > POOL_RETAIN_WORDS) return;
        long[] pooled = POOL.get();
        if (pooled == null || pooled.length < current.length) POOL.set(current);
    }

    private void allocate(int capacity) {
        this.capacity = capacity;
        int words = capacity * STRIDE;
        long[] pooled = POOL.get();
        // 同じ大きさ（最大 8 倍まで）の表があれば、outcome 語だけ消して再利用する。
        if (pooled != null && pooled.length >= words && pooled.length <= (long) words * 8) {
            POOL.remove();
            table = pooled;
            for (int i = 0; i < capacity; i++) table[i * STRIDE + 3] = 0;
            return;
        }
        table = new long[words];
    }

    private void checkSlot(int slot) {
        if (slot < 0 || slot >= capacity || table[slot * STRIDE + 3] == 0) {
            throw new IndexOutOfBoundsException("No memo entry at slot " + slot);
        }
    }
}
