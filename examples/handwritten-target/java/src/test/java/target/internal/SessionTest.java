package target.internal;

import static org.junit.jupiter.api.Assertions.*;
import java.util.List;
import org.junit.jupiter.api.Test;
import target.api.ParseOptions;
import target.rt.Cursor;

class SessionTest {
    @Test void literalIsPrefixWithoutWordBoundaryAndPreservesBothCursors() {
        var s = new Session("letter😀", ParseOptions.RECOGNIZE, "Root");
        s.consumed = 0; s.matched = 1;
        assertTrue(s.literal("let", Session.NUMBER));
        assertEquals(3, s.consumed); assertEquals(4, s.matched);
        assertEquals(3, s.input.cpIndex(s.consumed));
        assertFalse(s.literal("x", Session.NUMBER));
        assertEquals(3, s.consumed); assertEquals(4, s.matched);
        var cursor = new Cursor(0, 0);
        cursor.advanceMatchedOnly(1);
        cursor.advance(3);
        assertEquals(s.consumed, cursor.consumed); assertEquals(s.matched, cursor.matched);
    }

    @Test void memoKeysIncludeBothCursorsAndReplayRecipeAndCaptures() {
        var s = new Session("12", ParseOptions.DEFAULT, "Root");
        assertEquals(Session.MISS, s.begin(0, 0, 0));
        s.advanceTo(1);
        s.recipe = s.leaf(Recipe.Kind.NUMBER, 0, 1, 0, 1);
        Recipe adopted = s.recipe;
        s.events = s.capture(null, 0, "value", 0, 1);
        assertEquals(0, s.end(0, 0, 0, 0));
        assertEquals(Session.MISS, s.begin(0, 0, 1));
        assertEquals(-1, s.end(0, 0, 1, -1));
        assertEquals(0, s.begin(0, 0, 0));
        assertSame(adopted, s.recipe);
        assertEquals(1, s.consumed); assertEquals(1, s.matched);
        assertEquals(1, Events.materialize(s.events, s.input).size());
        assertEquals(-1, s.begin(0, 0, 1));
        assertEquals(0, s.consumed); assertEquals(1, s.matched);
        assertNull(s.recipe); assertNull(s.events);
        assertEquals(2, s.memoHits);
    }

    @Test void memoDiagnosticsRebaseTheCallingRuleStack() {
        var s = new Session("x", ParseOptions.RECOGNIZE, "First", "Second", "Leaf");
        s.diagnostics.enterRule(0);
        assertEquals(Session.MISS, s.begin(2, 0, 0));
        s.diagnostics.failAt(0, Session.NUMBER);
        s.end(2, 0, 0, -1);
        s.diagnostics.leaveRule();
        s.diagnostics.enterRule(1);
        assertEquals(-1, s.begin(2, 0, 0));
        var local = s.diagnostics.leaveRule();
        assertArrayEquals(new int[]{1, 2}, local.ruleStack());
        assertArrayEquals(new int[]{Session.NUMBER}, local.expected());
        assertEquals(0, local.farthest());
    }

    @Test void emptyIterationIsCountedBeforeProgressGuardAndBounds() {
        int count = 0;
        do {
            count++; // EMPTY は consumed を変えない。
        } while (RepeatProgress.more(count, 4, 4, 2));
        assertEquals(1, count);
        assertFalse(RepeatProgress.accepts(count, 2, 2));
        assertTrue(RepeatProgress.accepts(count, 1, 2));
        assertTrue(RepeatProgress.accepts(0, 0, Integer.MAX_VALUE));
        assertFalse(RepeatProgress.more(2, 4, 5, 2));
    }

    @Test void persistentCaptureEventsSurviveAnAbandonedSiblingAndKeepCompletionOrder() {
        var s = new Session("12", ParseOptions.DEFAULT, "Rule");
        Events inner = s.capture(null, 1, "value", 0, 0);
        Events outer = s.capture(inner, 2, "value", 0, 1);
        s.capture(inner, 3, "abandoned", 1, 2);
        var captures = Events.materialize(outer, s.input);
        assertEquals(List.of(1, 2), captures.stream().map(c -> c.siteId()).toList());
        assertEquals(0, captures.getFirst().span().end());
    }
}
