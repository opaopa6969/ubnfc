// ubnfc runtime template: ScopeStoreTest
package ubnfc.rt;

import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

/** transactional-scopes.md と packrat-memoization.md #269 の操作契約。 */
class ScopeStoreTest {
    @Test
    void redeclarationWinsLookupAndKeepsBothHistoryEvents() {
        ScopeStore store = new ScopeStore();
        store.enter();
        store.declare("x", 2);
        store.declare("x", 9);
        assertEquals(new ScopeStore.Decl("x", 9), store.resolve("x"));
        assertEquals(List.of(new ScopeStore.Decl("x", 9)), store.declaredInCurrentScope());
        assertEquals(List.of(new ScopeStore.Decl("x", 2), new ScopeStore.Decl("x", 9)), store.allDeclarations());
    }

    @Test
    void lookupSearchesInnerThenOuterThenGlobal() {
        ScopeStore store = new ScopeStore();
        store.declare("x", 1);
        store.declare("global", 2);
        store.enter();
        store.declare("x", 3);
        store.enter();
        assertEquals(3, store.resolve("x").offsetCp());
        assertEquals(2, store.resolve("global").offsetCp());
        store.declare("x", 4);
        assertEquals(4, store.resolve("x").offsetCp());
        store.leave();
        assertEquals(3, store.resolve("x").offsetCp());
        store.leave();
        assertEquals(1, store.resolve("x").offsetCp());
        assertNull(store.resolve("missing"));
    }

    @Test
    void leavingHidesNamesAndRetainsSuccessfulEvents() {
        ScopeStore store = new ScopeStore();
        store.enter();
        store.declare("local", 3);
        store.addReference("local", 9, 5);
        store.addDiagnostic("note", 9, 5, Severity.INFO);
        store.leave();
        assertEquals(0, store.depth());
        assertFalse(store.isDeclared("local"));
        assertTrue(store.declaredInCurrentScope().isEmpty());
        assertEquals(1, store.allDeclarations().size());
        assertEquals(1, store.references().size());
        assertEquals(1, store.diagnostics().size());
    }

    @Test
    void leaveAtDepthZeroIsAnExactNoOp() {
        ScopeStore store = new ScopeStore();
        store.declare("global", 0);
        int checkpoint = store.checkpoint();
        int version = store.stateVersion();
        store.leave();
        assertEquals(checkpoint, store.checkpoint());
        assertEquals(version, store.stateVersion());
        assertTrue(store.isDeclared("global"));
    }

    @Test
    void emptyAndNullNamesAreNoOpsAndDoNotResolve() {
        ScopeStore store = new ScopeStore();
        store.declare("", 0);
        store.declare(null, 0);
        store.addReference("", 0, 0);
        store.addReference(null, 0, 0);
        assertEquals(0, store.checkpoint());
        assertEquals(0, store.stateVersion());
        assertFalse(store.isDeclared(""));
        assertNull(store.resolve(null));
        assertTrue(store.allDeclarations().isEmpty());
        assertTrue(store.references().isEmpty());
    }

    @Test
    void referenceRecordingDoesNotAutomaticallyDiagnoseUndefinedNames() {
        ScopeStore store = new ScopeStore();
        store.addReference("missing", 4, 7);
        assertEquals(List.of(new ScopeStore.Reference("missing", 4, 7)), store.references());
        assertTrue(store.diagnostics().isEmpty());
        assertFalse(store.isDeclared("missing"));
    }

    @Test
    void failedTransactionRestoresPoppedScopesGlobalsAndAllLogs() {
        ScopeStore store = new ScopeStore();
        store.declare("global", 1);
        store.enter();
        store.declare("local", 2);
        store.addReference("local", 3, 5);
        store.addDiagnostic("old", 3, 5, Severity.HINT);
        List<ScopeStore.Decl> beforeDecls = List.copyOf(store.allDeclarations());
        List<ScopeStore.Reference> beforeRefs = List.copyOf(store.references());
        List<ScopeStore.Diagnostic> beforeDiags = List.copyOf(store.diagnostics());
        int version = store.stateVersion();
        int checkpoint = store.checkpoint();
        store.declare("local", 8);
        store.leave();
        store.declare("global", 10);
        store.declare("discard", 11);
        store.enter();
        store.declare("nested", 12);
        store.addReference("discard", 13, 7);
        store.clearDiagnostics();
        store.addDiagnostic("new", 13, 7, Severity.ERROR);
        store.leave();
        store.restore(checkpoint);
        assertEquals(1, store.depth());
        assertEquals(new ScopeStore.Decl("local", 2), store.resolve("local"));
        assertEquals(new ScopeStore.Decl("global", 1), store.resolve("global"));
        assertFalse(store.isDeclared("nested"));
        assertFalse(store.isDeclared("discard"));
        assertEquals(beforeDecls, store.allDeclarations());
        assertEquals(beforeRefs, store.references());
        assertEquals(beforeDiags, store.diagnostics());
        assertEquals(version, store.stateVersion());
    }

    @Test
    void successfulChildChangesRemainUndoableByParent() {
        ScopeStore store = new ScopeStore();
        int parent = store.checkpoint();
        store.enter();
        store.declare("parent", 1);
        store.enter();
        store.declare("child", 2);
        store.leave(); // 子成功。commit に相当する操作で journal を捨てない。
        store.declare("childRule", 2);
        store.restore(parent);
        assertEquals(0, store.depth());
        assertEquals(0, store.stateVersion());
        assertTrue(store.allDeclarations().isEmpty());
        assertNull(store.resolve("childRule"));
    }

    @Test
    void firstMutationAfterNestedCheckpointsRestoresToEmptyBaseline() {
        ScopeStore store = new ScopeStore();
        int outer = store.checkpoint();
        int inner = store.checkpoint();
        store.declare("first", 0);
        store.restore(inner);
        store.declare("second", 1);
        store.restore(outer);
        assertTrue(store.allDeclarations().isEmpty());
        assertTrue(store.declaredInCurrentScope().isEmpty());
    }

    @Test
    void nestedRollbackKeepsEarlierOuterChanges() {
        ScopeStore store = new ScopeStore();
        store.declare("x", 1);
        int outer = store.checkpoint();
        store.declare("x", 2);
        int inner = store.checkpoint();
        store.declare("x", 3);
        store.restore(inner);
        assertEquals(2, store.resolve("x").offsetCp());
        store.restore(outer);
        assertEquals(1, store.resolve("x").offsetCp());
        store.restore(outer);
        assertEquals(1, store.allDeclarations().size());
    }

    @Test
    void liveViewsNeverExposeDiscardedEventsAfterRestore() {
        ScopeStore store = new ScopeStore();
        List<ScopeStore.Decl> declarations = store.allDeclarations();
        List<ScopeStore.Reference> references = store.references();
        List<ScopeStore.Diagnostic> diagnostics = store.diagnostics();
        store.declare("keep", 0);
        int checkpoint = store.checkpoint();
        store.declare("discard", 1);
        store.addReference("discard", 2, 7);
        store.addDiagnostic("discard", 2, 7, Severity.WARNING);
        List<ScopeStore.Decl> detached = List.copyOf(declarations);
        assertThrows(UnsupportedOperationException.class, () -> declarations.clear());
        assertThrows(UnsupportedOperationException.class, () -> references.clear());
        assertThrows(UnsupportedOperationException.class, () -> diagnostics.clear());
        store.restore(checkpoint);
        assertEquals(List.of(new ScopeStore.Decl("keep", 0)), declarations);
        assertTrue(references.isEmpty());
        assertTrue(diagnostics.isEmpty());
        assertEquals(2, detached.size());
    }

    @Test
    void repeatedDiagnosticClearsRestoreInOriginalOrder() {
        ScopeStore store = new ScopeStore();
        store.addDiagnostic("first", 0, 1, Severity.ERROR);
        store.addDiagnostic("second", 1, 1, Severity.WARNING);
        List<ScopeStore.Diagnostic> view = store.diagnostics();
        List<ScopeStore.Diagnostic> original = List.copyOf(view);
        int checkpoint = store.checkpoint();
        store.clearDiagnostics();
        store.addDiagnostic("third", 2, 1, Severity.INFO);
        store.clearDiagnostics();
        store.addDiagnostic("fourth", 3, 1, Severity.HINT);
        store.restore(checkpoint);
        assertEquals(original, view);
    }

    @Test
    void onlyParserVisibleMutationsAdvanceVersion() {
        ScopeStore store = new ScopeStore();
        assertEquals(0, store.stateVersion());
        store.enter();
        assertEquals(1, store.stateVersion());
        store.declare("x", 0);
        assertEquals(2, store.stateVersion());
        store.declare("x", 0);
        assertEquals(3, store.stateVersion());
        store.addReference("x", 1, 1);
        store.addDiagnostic("note", 1, 1, Severity.INFO);
        assertEquals(3, store.stateVersion());
        store.clearDiagnostics();
        assertEquals(4, store.stateVersion());
        store.clearDiagnostics(); // 空の clear は原典同様に no-op。
        assertEquals(4, store.stateVersion());
        store.leave();
        assertEquals(5, store.stateVersion());
    }

    @Test
    void restoreRewindsVersionButNeverReusesBranchEpoch() {
        ScopeStore store = new ScopeStore();
        MemoTable memo = new MemoTable();
        store.declare("base", 0);
        int checkpoint = store.checkpoint();
        int baseline = store.stateVersion();
        memo.put(1, 0, 0, 0, baseline, false, 0, 0, 10);
        store.declare("branchA", 1);
        int branchA = store.stateVersion();
        memo.put(1, 0, 0, 0, branchA, true, 1, 1, 11);
        store.restore(checkpoint);
        assertEquals(baseline, store.stateVersion());
        assertFalse(memo.ok(memo.get(1, 0, 0, 0, store.stateVersion())));
        store.declare("branchB", 1);
        assertTrue(store.stateVersion() > branchA);
        assertEquals(-1, memo.get(1, 0, 0, 0, store.stateVersion()));
    }

    @Test
    void rollbackOfWriteOnlyLogsLeavesVersionUnchanged() {
        ScopeStore store = new ScopeStore();
        store.enter();
        int version = store.stateVersion();
        int checkpoint = store.checkpoint();
        store.addReference("x", 0, 1);
        store.addDiagnostic("note", 0, 1, Severity.WARNING);
        store.restore(checkpoint);
        assertEquals(version, store.stateVersion());
        store.declare("x", 0);
        assertEquals(version + 1, store.stateVersion());
    }

    @ParameterizedTest
    @EnumSource(Severity.class)
    void allSeveritiesPreserveCodePointPositions(Severity severity) {
        ScopeStore store = new ScopeStore();
        store.addDiagnostic("message", 2, 3, severity);
        assertEquals(List.of(new ScopeStore.Diagnostic("message", 2, 3, severity)), store.diagnostics());
    }

    @Test
    void separateSessionsDoNotShareState() {
        ScopeStore first = new ScopeStore();
        ScopeStore second = new ScopeStore();
        first.declare("x", 1);
        assertFalse(second.isDeclared("x"));
        assertEquals(0, second.stateVersion());
    }

    @Test
    void rejectsCheckpointOutsideCurrentHistory() {
        ScopeStore store = new ScopeStore();
        assertThrows(IllegalArgumentException.class, () -> store.restore(-1));
        assertThrows(IllegalArgumentException.class, () -> store.restore(1));
        assertEquals(0, store.depth());
    }
}
