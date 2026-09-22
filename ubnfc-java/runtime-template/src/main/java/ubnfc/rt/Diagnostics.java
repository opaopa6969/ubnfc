// ubnfc runtime template: Diagnostics
package ubnfc.rt;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Objects;

/** 構文診断。semantic 診断の rollback とは独立し、成功 rule 内の失敗も保持する。 */
public final class Diagnostics {
    /** 配列を外部へ共有しない immutable な局所診断。stack は保存 rule からの suffix。 */
    public static final class Snapshot {
        private final int farthest;
        private final int[] expected;
        private final int[] ruleStack;

        private Snapshot(Accumulator accumulator) {
            farthest = accumulator.farthest;
            expected = Arrays.copyOf(accumulator.expected, accumulator.expectedSize);
            ruleStack = accumulator.ruleStack.clone();
        }

        public int farthest() {
            return farthest;
        }

        public int[] expected() {
            return expected.clone();
        }

        public int deepestRule() {
            return ruleStack.length == 0 ? -1 : ruleStack[ruleStack.length - 1];
        }

        public int[] ruleStack() {
            return ruleStack.clone();
        }
    }

    private static final class Accumulator {
        int farthest = -1;
        int[] expected = new int[4];
        int expectedSize;
        int[] ruleStack = new int[0];

        void fail(int pos, int expectedId, int[] stack) {
            if (pos < farthest) {
                return;
            }
            if (pos > farthest) {
                farthest = pos;
                expectedSize = 0;
                ruleStack = stack;
            } else if (stack.length > ruleStack.length) {
                ruleStack = stack;
            }
            for (int i = 0; i < expectedSize; i++) {
                if (expected[i] == expectedId) {
                    return;
                }
            }
            if (expectedSize == expected.length) {
                expected = Arrays.copyOf(expected, expected.length * 2);
            }
            expected[expectedSize++] = expectedId;
        }
    }

    private record Frame(int ruleId, Accumulator local) {}

    private final Accumulator global = new Accumulator();
    private final ArrayList<Frame> frames = new ArrayList<>();
    private int suppressionDepth;

    public int farthest() {
        return global.farthest;
    }

    public int[] expected() {
        return Arrays.copyOf(global.expected, global.expectedSize);
    }

    public int deepestRule() {
        return global.ruleStack.length == 0 ? -1 : global.ruleStack[global.ruleStack.length - 1];
    }

    public int[] ruleStack() {
        return global.ruleStack.clone();
    }

    public void enterRule(int ruleId) {
        if (ruleId < 0) {
            throw new IllegalArgumentException("Negative rule id");
        }
        frames.add(new Frame(ruleId, new Accumulator()));
    }

    /** 成功・失敗のどちらでも同じ終了操作。戻り値を memo payload に保存できる。 */
    public Snapshot leaveRule() {
        if (frames.isEmpty()) {
            throw new IllegalStateException("No active rule");
        }
        return new Snapshot(frames.removeLast().local());
    }

    /** pos は呼出し側が選んだ一貫した位置単位。UTF-16 / code point 変換は行わない。 */
    public void failAt(int pos, int expectedId) {
        if (pos < 0 || expectedId < 0) {
            throw new IllegalArgumentException("Negative failure position or expected id");
        }
        if (suppressionDepth != 0) {
            return;
        }
        global.fail(pos, expectedId, stackSuffix(0, null));
        for (int i = 0; i < frames.size(); i++) {
            frames.get(i).local().fail(pos, expectedId, stackSuffix(i, null));
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
        return new Snapshot(frames.isEmpty() ? global : frames.getLast().local());
    }

    /** memo hit 時、保存 rule を enter する前に呼ぶ。呼出元の stack を付け替える。 */
    public void merge(Snapshot snapshot) {
        Objects.requireNonNull(snapshot);
        if (suppressionDepth != 0 || snapshot.farthest < 0) {
            return;
        }
        mergeInto(global, snapshot, stackSuffix(0, snapshot.ruleStack));
        for (int i = 0; i < frames.size(); i++) {
            mergeInto(frames.get(i).local(), snapshot, stackSuffix(i, snapshot.ruleStack));
        }
    }

    private static void mergeInto(Accumulator target, Snapshot snapshot, int[] stack) {
        for (int expected : snapshot.expected) {
            target.fail(snapshot.farthest, expected, stack);
        }
    }

    private int[] stackSuffix(int start, int[] suffix) {
        int prefixSize = frames.size() - start;
        int[] stack = new int[prefixSize + (suffix == null ? 0 : suffix.length)];
        for (int i = 0; i < prefixSize; i++) {
            stack[i] = frames.get(start + i).ruleId();
        }
        if (suffix != null) {
            System.arraycopy(suffix, 0, stack, prefixSize, suffix.length);
        }
        return stack;
    }
}
