// ubnfc runtime template: Diagnostics
package org.ubnfc.p4.generated.rt;

import java.util.Arrays;
import java.util.Objects;

/**
 * 構文診断。semantic 診断の rollback とは独立し、成功 rule 内の失敗も保持する。
 *
 * <p>rule stack は不変の親リンク連結リスト（{@link Node}）で共有し、各 accumulator は
 * （先頭 node, 長さ）の組だけを持つ。失敗は global と最内 frame にだけ記録し、
 * frame を離れるときに親 frame へ吸収する（観測結果は全 frame への即時記録と同値）。
 */
public final class Diagnostics {
    /** enterRule ごとに 1 つ。親リンクだけを持ち、後から変更しない。 */
    private static final class Node {
        final int ruleId;
        final Node parent;
        Node(int ruleId, Node parent) { this.ruleId = ruleId; this.parent = parent; }
    }

    /** 配列を外部へ共有しない immutable な局所診断。stack は保存 rule からの suffix。 */
    public static final class Snapshot {
        private final int farthest, reached;
        private final int consumed, matched;
        private final int[] expected;
        private final Node stackTop;
        private final int stackLength;

        private Snapshot(Accumulator accumulator) {
            farthest = accumulator.farthest; reached = accumulator.reached; consumed = accumulator.consumed; matched = accumulator.matched;
            expected = Arrays.copyOf(accumulator.expected, accumulator.expectedSize);
            stackTop = accumulator.stackTop; stackLength = accumulator.stackLength;
        }

        public int farthest() {
            return farthest;
        }

        public int consumed() { return consumed; }
        public int matched() { return matched; }

        public int[] expected() {
            return expected.clone();
        }

        public int deepestRule() {
            return stackLength == 0 ? -1 : stackTop.ruleId;
        }

        public int[] ruleStack() {
            return materialize(stackTop, stackLength);
        }
    }

    private static int[] materialize(Node top, int length) {
        int[] stack = new int[length];
        Node node = top;
        for (int i = length - 1; i >= 0; i--) { stack[i] = node.ruleId; node = node.parent; }
        return stack;
    }

    /** frame ごとに再利用する（深さで pool）。expected の重複除去は id の bitmap。 */
    private static final class Accumulator {
        int farthest = -1, reached = -1;
        int consumed, matched;
        int[] expected = new int[8];
        int expectedSize;
        long[] bits = new long[8];
        Node stackTop;
        int stackLength;

        void reset() {
            clearExpected();
            farthest = -1; reached = -1; consumed = 0; matched = 0; stackTop = null; stackLength = 0;
        }

        private void clearExpected() {
            for (int i = 0; i < expectedSize; i++) bits[expected[i] >>> 6] &= ~(1L << expected[i]);
            expectedSize = 0;
        }

        void fail(int pos, int expectedId, Node top, int length, int c, int m) {
            if (pos < farthest) {
                return;
            }
            if (pos > farthest) {
                farthest = pos; consumed = c; matched = m;
                clearExpected();
                stackTop = top; stackLength = length;
            } else if (length > stackLength) {
                stackTop = top; stackLength = length;
            }
            add(expectedId);
        }

        private void add(int expectedId) {
            int word = expectedId >>> 6;
            if (word >= bits.length) bits = Arrays.copyOf(bits, Math.max(word + 1, bits.length * 2));
            long mask = 1L << expectedId;
            if ((bits[word] & mask) != 0) {
                return;
            }
            bits[word] |= mask;
            if (expectedSize == expected.length) {
                expected = Arrays.copyOf(expected, expected.length * 2);
            }
            expected[expectedSize++] = expectedId;
        }

        /** 離れる子 frame の記録を取り込む。delta はこの frame から子 frame までの rule 数。 */
        void absorb(Accumulator child, int delta) {
            if (child.reached > reached) reached = child.reached;
            if (child.farthest < 0 || child.farthest < farthest) {
                return;
            }
            int length = child.stackLength + delta;
            if (child.farthest > farthest) {
                farthest = child.farthest; consumed = child.consumed; matched = child.matched;
                clearExpected();
                stackTop = child.stackTop; stackLength = length;
            } else if (length > stackLength) {
                stackTop = child.stackTop; stackLength = length;
            }
            for (int i = 0; i < child.expectedSize; i++) add(child.expected[i]);
        }
    }

    /** A speculative parse may discard diagnostics only after its outcome is known. */
    public static final class Checkpoint {
        private final Snapshot global;
        private final Snapshot[] locals;
        private Checkpoint(Diagnostics diagnostics) {
            global = new Snapshot(diagnostics.global);
            locals = new Snapshot[diagnostics.frameCount];
            for (int i = 0; i < locals.length; i++) locals[i] = new Snapshot(diagnostics.frameLocal[i]);
        }
    }

    public Checkpoint checkpoint() { return new Checkpoint(this); }

    public void restore(Checkpoint checkpoint) {
        if (frameCount != checkpoint.locals.length) throw new IllegalStateException("Unbalanced diagnostic frames");
        restoreInto(global, checkpoint.global);
        for (int i = 0; i < frameCount; i++) restoreInto(frameLocal[i], checkpoint.locals[i]);
    }

    private static void restoreInto(Accumulator target, Snapshot snapshot) {
        target.clearExpected();
        target.farthest = snapshot.farthest; target.reached = snapshot.reached; target.consumed = snapshot.consumed; target.matched = snapshot.matched;
        for (int expected : snapshot.expected) target.add(expected);
        target.stackTop = snapshot.stackTop; target.stackLength = snapshot.stackLength;
    }

    /** Expression memo needs a local accumulator without inventing a rule in the stack. */
    public void enterExpression() { push(-1); }

    private final Accumulator global = new Accumulator();
    /** frame は深さで並ぶ並列配列。accumulator は深さごとに再利用する。 */
    private int[] frameRule = new int[32], frameBase = new int[32];
    private Accumulator[] frameLocal = new Accumulator[32];
    private int frameCount;
    private Node top;
    private int ruleCount;
    private int suppressionDepth;

    private Accumulator push(int ruleId) {
        if (frameCount == frameLocal.length) {
            frameRule = Arrays.copyOf(frameRule, frameCount * 2); frameBase = Arrays.copyOf(frameBase, frameCount * 2);
            frameLocal = Arrays.copyOf(frameLocal, frameCount * 2);
        }
        Accumulator local = frameLocal[frameCount];
        if (local == null) frameLocal[frameCount] = local = new Accumulator(); else local.reset();
        frameRule[frameCount] = ruleId; frameBase[frameCount] = ruleCount; frameCount++;
        return local;
    }

    public int farthest() {
        return global.farthest;
    }

    /** 最内 frame の最遠失敗位置。frame 外では global。snapshot() を作らずに読む。 */
    public int localFarthest() {
        return frameCount == 0 ? global.farthest : frameLocal[frameCount - 1].farthest;
    }

    public int reached() { return global.reached; }

    public void reach(int position) {
        if (suppressionDepth != 0) return;
        if (position > global.reached) global.reached = position;
        if (frameCount > 0) {
            Accumulator local = frameLocal[frameCount - 1];
            if (position > local.reached) local.reached = position;
        }
    }

    public int[] expected() {
        return Arrays.copyOf(global.expected, global.expectedSize);
    }

    public int deepestRule() {
        return global.stackLength == 0 ? -1 : global.stackTop.ruleId;
    }

    public int[] ruleStack() {
        return materialize(global.stackTop, global.stackLength);
    }

    public void enterRule(int ruleId) {
        if (ruleId < 0) {
            throw new IllegalArgumentException("Negative rule id");
        }
        push(ruleId);
        top = new Node(ruleId, top); ruleCount++;
    }

    /** 成功・失敗のどちらでも同じ終了操作。戻り値を memo payload に保存できる。 */
    public Snapshot leaveRule() { return leaveRule(true); }

    /** snapshot=false は payload を保存しない呼出し向け（null を返す）。親への吸収は同じ。 */
    public Snapshot leaveRule(boolean snapshot) {
        if (frameCount == 0) {
            throw new IllegalStateException("No active rule");
        }
        int leaving = --frameCount;
        Accumulator local = frameLocal[leaving];
        if (frameRule[leaving] >= 0) { top = top.parent; ruleCount--; }
        Snapshot result = snapshot ? new Snapshot(local) : null;
        if (frameCount > 0) frameLocal[frameCount - 1].absorb(local, frameBase[leaving] - frameBase[frameCount - 1]);
        return result;
    }

    /** pos は呼出し側が選んだ一貫した位置単位。UTF-16 / code point 変換は行わない。 */
    public void failAt(int pos, int expectedId) { failAt(pos, expectedId, pos, pos); }

    public void failureCursor(int pos, int consumed, int matched) {
        if (suppressionDepth != 0) return;
        if (global.farthest == pos) { global.consumed = consumed; global.matched = matched; }
        for (int i = 0; i < frameCount; i++) {
            Accumulator local = frameLocal[i];
            if (local.farthest == pos) { local.consumed = consumed; local.matched = matched; }
        }
    }

    public void failAt(int pos, int expectedId, int consumed, int matched) {
        if (pos < 0 || expectedId < 0) {
            throw new IllegalArgumentException("Negative failure position or expected id");
        }
        if (suppressionDepth != 0) {
            return;
        }
        global.fail(pos, expectedId, top, ruleCount, consumed, matched);
        if (frameCount > 0) frameLocal[frameCount - 1].fail(pos, expectedId, top, ruleCount - frameBase[frameCount - 1], consumed, matched);
    }

    public void suppressBegin() {
        suppressionDepth = Math.incrementExact(suppressionDepth);
    }

    public void suppressEnd() {
        if (suppressionDepth == 0) {
            throw new IllegalStateException("No diagnostic suppression to end");
        }
        suppressionDepth--;
    }

    /** rule 内では最内 rule の局所結果、rule 外では解析全体の結果を返す。 */
    public Snapshot snapshot() {
        return new Snapshot(frameCount == 0 ? global : frameLocal[frameCount - 1]);
    }

    /** memo hit 時、保存 rule を enter する前に呼ぶ。呼出元の stack を付け替える。 */
    public void merge(Snapshot snapshot) {
        Objects.requireNonNull(snapshot);
        if (suppressionDepth != 0) {
            return;
        }
        reach(snapshot.reached);
        if (snapshot.farthest < 0 || snapshot.expected.length == 0) return;
        Accumulator local = frameCount == 0 ? null : frameLocal[frameCount - 1];
        if (snapshot.farthest < global.farthest && (local == null || snapshot.farthest < local.farthest)) return;
        // 保存 stack を現在の stack の上に付け替える。
        Node rebased = top;
        int[] saved = materialize(snapshot.stackTop, snapshot.stackLength);
        for (int ruleId : saved) rebased = new Node(ruleId, rebased);
        int length = ruleCount + snapshot.stackLength;
        for (int expected : snapshot.expected) global.fail(snapshot.farthest, expected, rebased, length, snapshot.consumed, snapshot.matched);
        if (local != null) {
            int localLength = ruleCount - frameBase[frameCount - 1] + snapshot.stackLength;
            for (int expected : snapshot.expected) local.fail(snapshot.farthest, expected, rebased, localLength, snapshot.consumed, snapshot.matched);
        }
    }
}
