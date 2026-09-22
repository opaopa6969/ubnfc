// ubnfc runtime template: Diagnostics
package org.ubnfc.p4.generated.rt;

import java.util.Arrays;
import java.util.Objects;

/**
 * 構文診断。semantic 診断の rollback とは独立し、成功 rule 内の失敗も保持する。
 *
 * <p>実装: 失敗は (a) 解析全体の accumulator（global）へ即時に、(b) 追記専用の event log へ、
 * (c) 最内 frame の要約（最遠位置・cursor・rule stack）へ O(1) で記録する。frame の expected
 * 集合は保持せず、必要になったとき（memo hit の merge、snapshot の expected()）に log の区間
 * [logStart, logEnd) から「最遠位置に一致する event」を集めて作る。frame を離れるときは要約だけを
 * 親へ吸収する。負の先読みの restore は global と live frame の要約を戻し、その区間の event を
 * 「先読みを囲む frame の走査からは除外」する（先読み内で memo 化された rule の snapshot は影響を
 * 受けない）。観測結果は「失敗ごとに全 frame へ即時記録する」実装と同値。
 *
 * <p>rule stack は不変の親リンク連結リスト（{@link Node}）で共有し、要約は（先頭 node, 長さ）だけを持つ。
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
        private final Node stackTop;
        private final int stackLength;
        private final Diagnostics owner;
        private final int logStart, logEnd;
        private int[] expected;

        private Snapshot(Diagnostics owner, int farthest, int reached, int consumed, int matched, Node stackTop, int stackLength, int logStart, int logEnd, int[] expected) {
            this.owner = owner; this.farthest = farthest; this.reached = reached; this.consumed = consumed; this.matched = matched;
            this.stackTop = stackTop; this.stackLength = stackLength; this.logStart = logStart; this.logEnd = logEnd; this.expected = expected;
        }

        public int farthest() {
            return farthest;
        }

        public int consumed() { return consumed; }
        public int matched() { return matched; }

        public int[] expected() {
            return ids().clone();
        }

        private int[] ids() {
            if (expected == null) expected = owner.collect(farthest, logStart, logEnd);
            return expected;
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

    /** 解析全体の集約。expected の重複除去は id ごとの世代 stamp（log 走査用とは別）。 */
    private static final class Global {
        int farthest = -1, reached = -1;
        int consumed, matched;
        int[] expected = new int[16];
        int expectedSize;
        Node stackTop;
        int stackLength;
        private int[] stamp;
        private int generation = 1;
        Global(int expectedIdHint) { stamp = new int[Math.max(16, expectedIdHint)]; }

        private void nextGeneration() {
            if (++generation == Integer.MAX_VALUE) { Arrays.fill(stamp, 0); generation = 1; }
        }

        private boolean mark(int expectedId) {
            if (expectedId >= stamp.length) stamp = Arrays.copyOf(stamp, Math.max(expectedId + 1, stamp.length * 2));
            if (stamp[expectedId] == generation) return false;
            stamp[expectedId] = generation;
            return true;
        }

        void fail(int pos, int expectedId, Node top, int length, int c, int m) {
            if (pos < farthest) {
                return;
            }
            if (pos > farthest) {
                farthest = pos; consumed = c; matched = m;
                expectedSize = 0; nextGeneration();
                stackTop = top; stackLength = length;
            } else if (length > stackLength) {
                stackTop = top; stackLength = length;
            }
            if (mark(expectedId)) {
                if (expectedSize == expected.length) expected = Arrays.copyOf(expected, expected.length * 2);
                expected[expectedSize++] = expectedId;
            }
        }

        Snapshot snapshot(Diagnostics owner) {
            return new Snapshot(owner, farthest, reached, consumed, matched, stackTop, stackLength, 0, 0, Arrays.copyOf(expected, expectedSize));
        }

        void restore(Snapshot snapshot) {
            farthest = snapshot.farthest; reached = snapshot.reached; consumed = snapshot.consumed; matched = snapshot.matched;
            stackTop = snapshot.stackTop; stackLength = snapshot.stackLength;
            int[] ids = snapshot.ids();
            expected = Arrays.copyOf(ids, Math.max(16, ids.length)); expectedSize = ids.length;
            nextGeneration();
            for (int id : ids) mark(id);
        }
    }

    /** log 走査（collect）の重複除去に使う stamp。 */
    private int[] stamp;
    private int generation = 1;

    private void nextGeneration() {
        if (++generation == Integer.MAX_VALUE) { Arrays.fill(stamp, 0); generation = 1; }
    }

    /** 現在の世代で未登録なら登録して true。 */
    private boolean mark(int expectedId) {
        if (expectedId >= stamp.length) stamp = Arrays.copyOf(stamp, Math.max(expectedId + 1, stamp.length * 2));
        if (stamp[expectedId] == generation) return false;
        stamp[expectedId] = generation;
        return true;
    }

    /** A speculative parse may discard diagnostics only after its outcome is known. */
    public static final class Checkpoint {
        private final Snapshot global;
        private final int frames, logSize;
        private final int[] farthest, reached, consumed, matched, stackLength;
        private final Node[] stackTop;
        private Checkpoint(Diagnostics d) {
            global = d.global.snapshot(d);
            frames = d.frameCount; logSize = d.logSize;
            farthest = Arrays.copyOf(d.frameFarthest, frames); reached = Arrays.copyOf(d.frameReached, frames);
            consumed = Arrays.copyOf(d.frameConsumed, frames); matched = Arrays.copyOf(d.frameMatched, frames);
            stackLength = Arrays.copyOf(d.frameStackLength, frames); stackTop = Arrays.copyOf(d.frameStackTop, frames);
        }
    }

    public Checkpoint checkpoint() { return new Checkpoint(this); }

    public void restore(Checkpoint checkpoint) {
        if (frameCount != checkpoint.frames) throw new IllegalStateException("Unbalanced diagnostic frames");
        global.restore(checkpoint.global);
        System.arraycopy(checkpoint.farthest, 0, frameFarthest, 0, frameCount); System.arraycopy(checkpoint.reached, 0, frameReached, 0, frameCount);
        System.arraycopy(checkpoint.consumed, 0, frameConsumed, 0, frameCount); System.arraycopy(checkpoint.matched, 0, frameMatched, 0, frameCount);
        System.arraycopy(checkpoint.stackLength, 0, frameStackLength, 0, frameCount); System.arraycopy(checkpoint.stackTop, 0, frameStackTop, 0, frameCount);
        if (logSize > checkpoint.logSize) {
            if (excludedCount == excludedStart.length) { excludedStart = Arrays.copyOf(excludedStart, excludedCount * 2); excludedEnd = Arrays.copyOf(excludedEnd, excludedCount * 2); }
            excludedStart[excludedCount] = checkpoint.logSize; excludedEnd[excludedCount] = logSize; excludedCount++;
        }
    }

    /** Expression memo needs a local accumulator without inventing a rule in the stack. */
    public void enterExpression() { push(-1); }

    private final Global global;
    // frame は深さで並ぶ並列配列。要約だけを持つ。
    private int[] frameRule = new int[32], frameBase = new int[32], frameLogStart = new int[32];
    private int[] frameFarthest = new int[32], frameReached = new int[32], frameConsumed = new int[32], frameMatched = new int[32], frameStackLength = new int[32];
    private Node[] frameStackTop = new Node[32];
    private int frameCount;
    // event log: (pos, id)。stack は frame 要約が持つので log には要らない。
    private int[] logPos, logId;
    private int logSize;
    // 負の先読みで捨てた区間。囲む frame の走査からだけ除外する。
    private int[] excludedStart = new int[4], excludedEnd = new int[4];
    private int excludedCount;
    private Node top;
    private int ruleCount;
    private int suppressionDepth;

    public Diagnostics() { this(64); }

    /** expectedIdHint は想定される expected id の上限（stamp 配列の初期長）。 */
    public Diagnostics(int expectedIdHint) { this(expectedIdHint, 256); }

    /** logHint は想定される失敗 event 数（log 配列の初期長。P4 では入力 1 文字あたり約 40）。 */
    public Diagnostics(int expectedIdHint, int logHint) {
        stamp = new int[Math.max(16, expectedIdHint)]; global = new Global(expectedIdHint);
        logPos = new int[Math.max(16, logHint)]; logId = new int[logPos.length];
    }

    private void push(int ruleId) {
        if (frameCount == frameRule.length) {
            int n = frameCount * 2;
            frameRule = Arrays.copyOf(frameRule, n); frameBase = Arrays.copyOf(frameBase, n); frameLogStart = Arrays.copyOf(frameLogStart, n);
            frameFarthest = Arrays.copyOf(frameFarthest, n); frameReached = Arrays.copyOf(frameReached, n); frameConsumed = Arrays.copyOf(frameConsumed, n);
            frameMatched = Arrays.copyOf(frameMatched, n); frameStackLength = Arrays.copyOf(frameStackLength, n); frameStackTop = Arrays.copyOf(frameStackTop, n);
        }
        int i = frameCount++;
        frameRule[i] = ruleId; frameBase[i] = ruleCount; frameLogStart[i] = logSize;
        frameFarthest[i] = -1; frameReached[i] = -1; frameConsumed[i] = 0; frameMatched[i] = 0; frameStackLength[i] = 0; frameStackTop[i] = null;
    }

    private void log(int pos, int id) {
        if (logSize == logPos.length) {
            int n = logSize * 2;
            logPos = Arrays.copyOf(logPos, n); logId = Arrays.copyOf(logId, n);
        }
        logPos[logSize] = pos; logId[logSize] = id; logSize++;
    }

    /** 最内 frame の要約へ 1 件の失敗を記録する（Global.fail と同じ規則、expected は log 側）。 */
    private void failTop(int pos, Node node, int length, int c, int m) {
        int i = frameCount - 1;
        int farthest = frameFarthest[i];
        if (pos < farthest) return;
        if (pos > farthest) {
            frameFarthest[i] = pos; frameConsumed[i] = c; frameMatched[i] = m; frameStackTop[i] = node; frameStackLength[i] = length;
        } else if (length > frameStackLength[i]) {
            frameStackTop[i] = node; frameStackLength[i] = length;
        }
    }

    /** [start, end) の log から pos に一致する event の id を出現順・重複なしで集める。囲む先読み除外区間は飛ばす。 */
    private int[] collect(int pos, int start, int end) {
        if (pos < 0 || start >= end) return new int[0];
        nextGeneration();
        int[] result = new int[8];
        int size = 0;
        int i = start;
        while (i < end) {
            int skipTo = -1;
            for (int x = 0; x < excludedCount; x++) if (excludedStart[x] >= start && excludedStart[x] <= i && i < excludedEnd[x] && excludedEnd[x] <= end && excludedEnd[x] > skipTo) skipTo = excludedEnd[x];
            if (skipTo >= 0) { i = skipTo; continue; }
            if (logPos[i] == pos && mark(logId[i])) {
                if (size == result.length) result = Arrays.copyOf(result, size * 2);
                result[size++] = logId[i];
            }
            i++;
        }
        return size == result.length ? result : Arrays.copyOf(result, size);
    }

    public int farthest() {
        return global.farthest;
    }

    /** 最内 frame の最遠失敗位置。frame 外では global。snapshot() を作らずに読む。 */
    public int localFarthest() {
        return frameCount == 0 ? global.farthest : frameFarthest[frameCount - 1];
    }

    public int reached() { return global.reached; }

    public void reach(int position) {
        if (suppressionDepth != 0) return;
        if (position > global.reached) global.reached = position;
        if (frameCount > 0 && position > frameReached[frameCount - 1]) frameReached[frameCount - 1] = position;
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
        if (frameRule[leaving] >= 0) { top = top.parent; ruleCount--; }
        Snapshot result = snapshot ? new Snapshot(this, frameFarthest[leaving], frameReached[leaving], frameConsumed[leaving], frameMatched[leaving],
            frameStackTop[leaving], frameStackLength[leaving], frameLogStart[leaving], logSize, null) : null;
        if (frameCount > 0) {
            int parent = frameCount - 1;
            if (frameReached[leaving] > frameReached[parent]) frameReached[parent] = frameReached[leaving];
            int farthest = frameFarthest[leaving];
            if (farthest >= 0 && farthest >= frameFarthest[parent]) {
                int length = frameStackLength[leaving] + frameBase[leaving] - frameBase[parent];
                if (farthest > frameFarthest[parent]) {
                    frameFarthest[parent] = farthest; frameConsumed[parent] = frameConsumed[leaving]; frameMatched[parent] = frameMatched[leaving];
                    frameStackTop[parent] = frameStackTop[leaving]; frameStackLength[parent] = length;
                } else if (length > frameStackLength[parent]) {
                    frameStackTop[parent] = frameStackTop[leaving]; frameStackLength[parent] = length;
                }
            }
        }
        return result;
    }

    /** pos は呼出し側が選んだ一貫した位置単位。UTF-16 / code point 変換は行わない。 */
    public void failAt(int pos, int expectedId) { failAt(pos, expectedId, pos, pos); }

    public void failureCursor(int pos, int consumed, int matched) {
        if (suppressionDepth != 0) return;
        if (global.farthest == pos) { global.consumed = consumed; global.matched = matched; }
        for (int i = 0; i < frameCount; i++) if (frameFarthest[i] == pos) { frameConsumed[i] = consumed; frameMatched[i] = matched; }
    }

    public void failAt(int pos, int expectedId, int consumed, int matched) {
        if (pos < 0 || expectedId < 0) {
            throw new IllegalArgumentException("Negative failure position or expected id");
        }
        if (suppressionDepth != 0) {
            return;
        }
        global.fail(pos, expectedId, top, ruleCount, consumed, matched);
        if (frameCount > 0) {
            failTop(pos, top, ruleCount - frameBase[frameCount - 1], consumed, matched);
            log(pos, expectedId);
        }
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
        if (frameCount == 0) return global.snapshot(this);
        int i = frameCount - 1;
        return new Snapshot(this, frameFarthest[i], frameReached[i], frameConsumed[i], frameMatched[i], frameStackTop[i], frameStackLength[i], frameLogStart[i], logSize, null);
    }

    /** memo hit 時、保存 rule を enter する前に呼ぶ。呼出元の stack を付け替える。 */
    public void merge(Snapshot snapshot) {
        Objects.requireNonNull(snapshot);
        if (suppressionDepth != 0) {
            return;
        }
        reach(snapshot.reached);
        if (snapshot.farthest < 0) return;
        boolean local = frameCount > 0;
        if (snapshot.farthest < global.farthest && (!local || snapshot.farthest < frameFarthest[frameCount - 1])) return;
        int[] ids = snapshot.ids();
        if (ids.length == 0) return;
        // 保存 stack を現在の stack の上に付け替える。
        Node rebased = top;
        int[] saved = materialize(snapshot.stackTop, snapshot.stackLength);
        for (int ruleId : saved) rebased = new Node(ruleId, rebased);
        int length = ruleCount + snapshot.stackLength;
        for (int id : ids) global.fail(snapshot.farthest, id, rebased, length, snapshot.consumed, snapshot.matched);
        if (local) {
            int localLength = ruleCount - frameBase[frameCount - 1] + snapshot.stackLength;
            failTop(snapshot.farthest, rebased, localLength, snapshot.consumed, snapshot.matched);
            for (int id : ids) log(snapshot.farthest, id);
        }
    }
}
