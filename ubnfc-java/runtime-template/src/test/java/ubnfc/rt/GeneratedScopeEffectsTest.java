// ubnfc runtime template: GeneratedScopeEffectsTest
package ubnfc.rt;

import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

/** generated-scope-effects.md のイベント列を直接駆動する。emitter の検証ではない。 */
class GeneratedScopeEffectsTest {
    @Test
    void parseTimeScopeNestingNeedsNoEvaluationModeMetadata() {
        ScopeStore store = new ScopeStore();
        // lexical/dynamic は生成 metadata。store は解析時のネストだけを扱う。
        store.enter();
        store.declare("body", 2);
        assertEquals(1, store.depth());
        assertTrue(store.isDeclared("body"));
        store.leave();
        assertEquals(0, store.depth());
        assertFalse(store.isDeclared("body"));
        assertEquals(List.of(new ScopeStore.Decl("body", 2)), store.allDeclarations());
    }

    @ParameterizedTest
    @ValueSource(booleans = {false, true})
    void scopedDeclarationIsPublishedToParentAfterLeaving(boolean hasParent) {
        ScopeStore store = new ScopeStore();
        if (hasParent) {
            store.enter();
        }
        store.enter();
        store.declare("private", 1);
        store.leave();
        store.declare("ruleName", 2);
        assertEquals(hasParent ? 1 : 0, store.depth());
        assertFalse(store.isDeclared("private"));
        assertTrue(store.isDeclared("ruleName"));
        assertEquals(List.of(new ScopeStore.Decl("ruleName", 2)), store.declaredInCurrentScope());
    }

    @Test
    void absentOptionalCaptureEmitsNoEvents() {
        ScopeStore store = new ScopeStore();
        emit(store, "unused", null, true, true);
        assertEquals(0, store.checkpoint());
        assertTrue(store.allDeclarations().isEmpty());
        assertTrue(store.references().isEmpty());
    }

    @Test
    void repeatedCapturesEmitEveryOccurrenceInCompletionOrder() {
        ScopeStore store = new ScopeStore();
        String source = "a b a";
        for (Span span : List.of(new Span(0, 1), new Span(2, 3), new Span(4, 5))) {
            emit(store, source, span, true, false);
        }
        assertEquals(List.of(new ScopeStore.Decl("a", 0), new ScopeStore.Decl("b", 2),
                new ScopeStore.Decl("a", 4)), store.allDeclarations());
        assertEquals(4, store.resolve("a").offsetCp());
    }

    @Test
    void nestedCapturesPublishInnerBeforeOuter() {
        ScopeStore store = new ScopeStore();
        String source = "outer(inner)";
        emit(store, source, new Span(6, 11), true, false);
        emit(store, source, new Span(0, 12), true, false);
        assertEquals(List.of(new ScopeStore.Decl("inner", 6), new ScopeStore.Decl("outer(inner)", 0)),
                store.allDeclarations());
    }

    @Test
    void onlyExplicitlySelectedRuleLocalCaptureProducesEvents() {
        ScopeStore store = new ScopeStore();
        String source = "other child selected";
        // siteId の解決は生成側。別 token・別 rule・再帰内部を推測して渡してはならない。
        emit(store, source, new Span(12, 20), true, false);
        assertEquals(List.of(new ScopeStore.Decl("selected", 12)), store.allDeclarations());
        assertFalse(store.isDeclared("other"));
        assertFalse(store.isDeclared("child"));
    }

    @Test
    void declarationPrecedesReferenceInSameRule() {
        ScopeStore store = new ScopeStore();
        emit(store, "name", new Span(0, 4), true, true);
        assertEquals(List.of(new ScopeStore.Reference("name", 0, 4)), store.references());
        assertTrue(store.diagnostics().isEmpty());
    }

    @Test
    void undefinedReferenceRecordsWarningWithoutARecognitionException() {
        ScopeStore store = new ScopeStore();
        assertDoesNotThrow(() -> emit(store, "missing", new Span(0, 7), false, true));
        assertEquals(List.of(new ScopeStore.Diagnostic("未定義のシンボル: 'missing'", 0, 7, Severity.WARNING)),
                store.diagnostics());
        assertEquals(List.of(new ScopeStore.Reference("missing", 0, 7)), store.references());
    }

    @Test
    void trimAdjustsCodePointOffsetsAndPreservesQuotesAndEscapes() {
        ScopeStore store = new ScopeStore();
        String source = "😀\u0000\t \"😀\\n\" \r\n";
        emit(store, source, new Span(2, source.length()), true, true);
        assertEquals(List.of(new ScopeStore.Decl("\"😀\\n\"", 4)), store.allDeclarations());
        assertEquals(List.of(new ScopeStore.Reference("\"😀\\n\"", 4, 5)), store.references());
    }

    @Test
    void trimUsesExactlyU0000ThroughU0020AndIgnoresEmptyNames() {
        ScopeStore store = new ScopeStore();
        emit(store, "\u0000\t \r\n", new Span(0, 5), true, true);
        assertEquals(0, store.checkpoint());
        emit(store, "\u00a0name\u00a0", new Span(0, 6), true, true);
        assertTrue(store.isDeclared("\u00a0name\u00a0"));
        assertEquals(6, store.references().getFirst().len());
    }

    @Test
    void failedChoiceBranchLeavesNoSemanticEffectsForNextCandidate() {
        ScopeStore store = new ScopeStore();
        Cursor cursor = new Cursor();
        int scopeCheckpoint = store.checkpoint();
        long cursorCheckpoint = cursor.checkpoint();
        store.enter();
        emit(store, "x", new Span(0, 1), true, true);
        cursor.advance(1);
        store.restore(scopeCheckpoint);
        cursor.restore(cursorCheckpoint);
        assertEquals(0, cursor.consumed);
        assertFalse(store.isDeclared("x"));
        emit(store, "x", new Span(0, 1), false, true);
        assertTrue(store.allDeclarations().isEmpty());
        assertEquals(1, store.references().size());
        assertEquals(Severity.WARNING, store.diagnostics().getFirst().severity());
    }

    @ParameterizedTest
    @ValueSource(booleans = {false, true})
    void lookaheadRestoresStateWhilePreservingItsOwnCursorContract(boolean negative) {
        ScopeStore store = new ScopeStore();
        Cursor cursor = new Cursor();
        Diagnostics syntax = new Diagnostics();
        int scopeCheckpoint = store.checkpoint();
        long cursorCheckpoint = cursor.checkpoint();
        if (negative) {
            syntax.suppressBegin();
        }
        emit(store, "x", new Span(0, 1), true, false);
        assertTrue(store.isDeclared("x")); // 先読み中の後続の子からは宣言が見える。
        emit(store, "x", new Span(0, 1), false, true);
        cursor.advanceMatchedOnly(1);
        syntax.failAt(1, 99);
        store.restore(scopeCheckpoint);
        if (negative) {
            syntax.suppressEnd();
            cursor.restore(cursorCheckpoint);
        }
        assertEquals(0, cursor.consumed);
        assertEquals(negative ? 0 : 1, cursor.matched);
        assertFalse(store.isDeclared("x"));
        assertTrue(store.allDeclarations().isEmpty());
        assertTrue(store.references().isEmpty());
        assertTrue(store.diagnostics().isEmpty());
        assertEquals(negative ? -1 : 1, syntax.farthest());
    }

    @Test
    void semanticRollbackDoesNotDiscardSyntaxFarthestFailure() {
        ScopeStore store = new ScopeStore();
        Diagnostics syntax = new Diagnostics();
        int checkpoint = store.checkpoint();
        store.addDiagnostic("semantic", 2, 1, Severity.WARNING);
        syntax.enterRule(7);
        syntax.failAt(9, 10);
        syntax.leaveRule();
        store.restore(checkpoint);
        assertTrue(store.diagnostics().isEmpty());
        assertEquals(9, syntax.farthest());
        assertEquals(7, syntax.deepestRule());
    }

    @Test
    void nonScopeBackrefCanRecordErrorWithoutDeclaringASymbol() {
        ScopeStore store = new ScopeStore();
        // 同一 rule 内のテキスト比較に失敗した生成側が ERROR を明示的に記録する。
        store.addDiagnostic("backref mismatch", 4, 3, Severity.ERROR);
        assertEquals(Severity.ERROR, store.diagnostics().getFirst().severity());
        assertTrue(store.allDeclarations().isEmpty());
        assertTrue(store.references().isEmpty());
    }

    private record Span(int start, int end) {}

    /** 生成側から渡すイベントの fixture。capture 検出・選択は実装していない。 */
    private static void emit(ScopeStore store, String source, Span capture, boolean declares, boolean reference) {
        if (capture == null) {
            return;
        }
        int start = capture.start();
        int end = capture.end();
        while (start < end && source.charAt(start) <= 0x20) {
            start++;
        }
        while (start < end && source.charAt(end - 1) <= 0x20) {
            end--;
        }
        String name = source.substring(start, end);
        Input input = new Input(source);
        int offset = input.cpIndex(start);
        int length = input.cpLength(start, end);
        if (declares) {
            store.declare(name, offset);
        }
        if (reference && !name.isEmpty()) {
            store.addReference(name, offset, length);
            if (!store.isDeclared(name)) {
                store.addDiagnostic("未定義のシンボル: '" + name + "'", offset, length, Severity.WARNING);
            }
        }
    }
}
