// ubnfc runtime template: DiagnosticsTest
package ubnfc.rt;

import static org.junit.jupiter.api.Assertions.*;

import java.util.stream.IntStream;
import org.junit.jupiter.api.Test;

class DiagnosticsTest {
    @Test
    void failureCursorsSurviveMemoReplayAndSpeculationRollback() {
        var diagnostics = new Diagnostics();
        diagnostics.enterRule(1);
        diagnostics.failAt(2, 7, 0, 2);
        diagnostics.failureCursor(2, 1, 3);
        var memo = diagnostics.leaveRule();
        assertEquals(1, memo.consumed()); assertEquals(3, memo.matched());
        var other = new Diagnostics();
        other.merge(memo);
        var checkpoint = other.checkpoint();
        other.failAt(9, 8, 9, 10);
        other.restore(checkpoint);
        assertEquals(2, other.farthest());
        assertEquals(1, other.snapshot().consumed()); assertEquals(3, other.snapshot().matched());
    }

    @Test
    void speculationRestoresGlobalAndEveryEnclosingLocal() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.failAt(10, 1);
        diagnostics.enterRule(7);
        diagnostics.failAt(2, 2);
        var mark = diagnostics.checkpoint();
        diagnostics.enterRule(8);
        diagnostics.failAt(20, 3);
        diagnostics.leaveRule();
        diagnostics.restore(mark);
        assertEquals(10, diagnostics.farthest());
        assertArrayEquals(new int[]{1}, diagnostics.expected());
        assertEquals(2, diagnostics.snapshot().farthest());
        assertArrayEquals(new int[]{2}, diagnostics.leaveRule().expected());
    }

    @Test
    void expressionMemoDoesNotDuplicateTheCallingRuleInItsStack() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.enterRule(7);
        diagnostics.enterExpression();
        diagnostics.enterRule(8);
        diagnostics.failAt(2, 1);
        diagnostics.leaveRule();
        var expression = diagnostics.leaveRule();
        diagnostics.leaveRule();
        assertArrayEquals(new int[]{8}, expression.ruleStack());
        diagnostics.enterRule(9);
        diagnostics.merge(expression);
        assertArrayEquals(new int[]{9, 8}, diagnostics.snapshot().ruleStack());
    }

    @Test
    void emptyDiagnosticsHaveNoFailureOrDeepestRule() {
        Diagnostics diagnostics = new Diagnostics();
        assertEquals(-1, diagnostics.farthest());
        assertEquals(-1, diagnostics.deepestRule());
        assertArrayEquals(new int[0], diagnostics.expected());
        assertEquals(-1, diagnostics.snapshot().farthest());
    }

    @Test
    void fartherFailureReplacesEqualAddsAndEarlierIsIgnored() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.failAt(2, 10);
        diagnostics.failAt(2, 11);
        diagnostics.failAt(2, 10);
        assertArrayEquals(new int[] {10, 11}, diagnostics.expected());
        diagnostics.failAt(1, 99);
        assertArrayEquals(new int[] {10, 11}, diagnostics.expected());
        diagnostics.failAt(3, 12);
        assertEquals(3, diagnostics.farthest());
        assertArrayEquals(new int[] {12}, diagnostics.expected());
    }

    @Test
    void expectedSetGrowsWithoutLosingInsertionOrderOrAddingDuplicates() {
        Diagnostics diagnostics = new Diagnostics();
        for (int i = 0; i < 40; i++) {
            diagnostics.failAt(0, i);
            diagnostics.failAt(0, i);
        }
        assertArrayEquals(IntStream.range(0, 40).toArray(), diagnostics.expected());
    }

    @Test
    void deepestRuleAtFarthestPositionSurvivesRuleExit() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.enterRule(1);
        diagnostics.failAt(4, 10);
        diagnostics.enterRule(2);
        diagnostics.failAt(4, 11);
        diagnostics.leaveRule();
        diagnostics.enterRule(3);
        diagnostics.failAt(3, 12);
        diagnostics.leaveRule();
        diagnostics.leaveRule();
        assertEquals(2, diagnostics.deepestRule());
        assertArrayEquals(new int[] {1, 2}, diagnostics.ruleStack());
        assertArrayEquals(new int[] {10, 11}, diagnostics.expected());
    }

    @Test
    void equallyDeepFailuresKeepFirstRuleAndFartherShallowFailureReplacesIt() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.enterRule(1);
        diagnostics.failAt(4, 10);
        diagnostics.leaveRule();
        diagnostics.enterRule(2);
        diagnostics.failAt(4, 11);
        assertEquals(1, diagnostics.deepestRule());
        diagnostics.leaveRule();
        diagnostics.failAt(5, 12);
        assertEquals(-1, diagnostics.deepestRule());
        assertArrayEquals(new int[] {12}, diagnostics.expected());
    }

    @Test
    void successfulRulesStillContributeTheirInternalFailures() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.enterRule(7);
        diagnostics.failAt(8, 12); // 失敗候補の後に別候補が成功した場合も leave は共通。
        Diagnostics.Snapshot memo = diagnostics.leaveRule();
        assertEquals(8, diagnostics.farthest());
        assertEquals(7, memo.deepestRule());
    }

    @Test
    void ruleLocalSnapshotExcludesEarlierSiblingFailure() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.enterRule(1);
        diagnostics.enterRule(2);
        diagnostics.failAt(20, 200);
        diagnostics.leaveRule();
        diagnostics.enterRule(3);
        diagnostics.failAt(5, 300);
        Diagnostics.Snapshot local = diagnostics.snapshot();
        assertEquals(5, local.farthest());
        assertArrayEquals(new int[] {300}, local.expected());
        assertArrayEquals(new int[] {3}, local.ruleStack());
        assertEquals(20, diagnostics.farthest());
        diagnostics.leaveRule();
        Diagnostics.Snapshot outer = diagnostics.leaveRule();
        assertEquals(20, outer.farthest());
        assertArrayEquals(new int[] {1, 2}, outer.ruleStack());
    }

    @Test
    void nestedSuppressionDiscardsFailuresInGlobalAndEveryLocalFrame() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.enterRule(1);
        diagnostics.failAt(2, 10);
        diagnostics.suppressBegin();
        diagnostics.enterRule(2);
        diagnostics.failAt(20, 20);
        diagnostics.suppressBegin();
        diagnostics.failAt(30, 30);
        diagnostics.suppressEnd();
        diagnostics.failAt(40, 40);
        assertEquals(-1, diagnostics.leaveRule().farthest());
        diagnostics.suppressEnd();
        assertEquals(2, diagnostics.farthest());
        assertEquals(2, diagnostics.snapshot().farthest());
        diagnostics.failAt(3, 11);
        assertEquals(3, diagnostics.leaveRule().farthest());
    }

    @Test
    void snapshotIsIndependentAndDoesNotExposeMutableArrays() {
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.enterRule(1);
        diagnostics.failAt(1, 10);
        Diagnostics.Snapshot snapshot = diagnostics.snapshot();
        snapshot.expected()[0] = 999;
        snapshot.ruleStack()[0] = 999;
        diagnostics.expected()[0] = 888;
        diagnostics.ruleStack()[0] = 888;
        diagnostics.failAt(2, 20);
        assertEquals(1, snapshot.farthest());
        assertArrayEquals(new int[] {10}, snapshot.expected());
        assertArrayEquals(new int[] {1}, snapshot.ruleStack());
    }

    @Test
    void memoReplayRebasesStackAndPropagatesIntoOuterLocalFrames() {
        Diagnostics original = new Diagnostics();
        original.enterRule(100);
        original.enterRule(7);
        original.enterRule(8);
        original.failAt(5, 10);
        original.leaveRule();
        Diagnostics.Snapshot cached = original.leaveRule();
        assertArrayEquals(new int[] {7, 8}, cached.ruleStack());

        Diagnostics replay = new Diagnostics();
        replay.enterRule(200);
        replay.enterRule(201);
        replay.merge(cached);
        assertArrayEquals(new int[] {200, 201, 7, 8}, replay.ruleStack());
        assertArrayEquals(new int[] {201, 7, 8}, replay.snapshot().ruleStack());
        replay.leaveRule();
        Diagnostics.Snapshot enclosing = replay.leaveRule();
        assertArrayEquals(new int[] {200, 201, 7, 8}, enclosing.ruleStack());
        assertEquals(8, replay.deepestRule());
        assertArrayEquals(new int[] {10}, replay.expected());
    }

    @Test
    void replayMatchesOrdinaryExecutionObservations() {
        Diagnostics original = new Diagnostics();
        original.enterRule(7);
        original.failAt(4, 10);
        original.enterRule(8);
        original.failAt(4, 11);
        original.leaveRule();
        Diagnostics.Snapshot cached = original.leaveRule();
        Diagnostics replay = new Diagnostics();
        replay.enterRule(200);
        replay.failAt(4, 9);
        replay.merge(cached);
        Diagnostics direct = new Diagnostics();
        direct.enterRule(200);
        direct.failAt(4, 9);
        direct.enterRule(7);
        direct.failAt(4, 10);
        direct.enterRule(8);
        direct.failAt(4, 11);
        direct.leaveRule();
        direct.leaveRule();
        assertEquals(direct.farthest(), replay.farthest());
        assertEquals(direct.deepestRule(), replay.deepestRule());
        assertArrayEquals(direct.expected(), replay.expected());
        assertArrayEquals(direct.ruleStack(), replay.ruleStack());
        assertArrayEquals(direct.snapshot().expected(), replay.snapshot().expected());
    }

    @Test
    void replayUpdatesLocalFrameEvenWhenGlobalHasFartherFailure() {
        Diagnostics cached = new Diagnostics();
        cached.enterRule(7);
        cached.failAt(3, 10);
        Diagnostics.Snapshot local = cached.leaveRule();
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.failAt(20, 99);
        diagnostics.enterRule(2);
        diagnostics.merge(local);
        assertEquals(20, diagnostics.farthest());
        assertArrayEquals(new int[] {99}, diagnostics.expected());
        assertEquals(3, diagnostics.snapshot().farthest());
        assertArrayEquals(new int[] {2, 7}, diagnostics.snapshot().ruleStack());
    }

    @Test
    void mergeRespectsSuppressionAndIgnoresEmptySnapshots() {
        Diagnostics cached = new Diagnostics();
        cached.failAt(8, 10);
        Diagnostics diagnostics = new Diagnostics();
        diagnostics.enterRule(1);
        diagnostics.merge(new Diagnostics().snapshot());
        diagnostics.suppressBegin();
        diagnostics.merge(cached.snapshot());
        diagnostics.suppressEnd();
        assertEquals(-1, diagnostics.farthest());
        assertEquals(-1, diagnostics.snapshot().farthest());
        diagnostics.merge(cached.snapshot());
        diagnostics.merge(cached.snapshot());
        assertArrayEquals(new int[] {10}, diagnostics.expected());
    }

    @Test
    void rejectsUnbalancedFramesAndSuppression() {
        Diagnostics diagnostics = new Diagnostics();
        assertThrows(IllegalStateException.class, diagnostics::leaveRule);
        assertThrows(IllegalStateException.class, diagnostics::suppressEnd);
        assertThrows(IllegalArgumentException.class, () -> diagnostics.failAt(-1, 0));
        assertThrows(IllegalArgumentException.class, () -> diagnostics.enterRule(-1));
    }
}
