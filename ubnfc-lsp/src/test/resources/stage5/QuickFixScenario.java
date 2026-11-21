import static org.junit.jupiter.api.Assertions.*;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import org.eclipse.lsp4j.*;
import org.eclipse.lsp4j.jsonrpc.json.MessageJsonHandler;
import org.eclipse.lsp4j.jsonrpc.messages.Either;
import org.unlaxer.ubnfc.examples.stage5.lsp.LanguageServerImpl;

/** 現在の診断と LSP のバージョン付き編集を JSON-RPC で検証する。 */
final class QuickFixScenario {
    private static final String URI = "file:///stage5.demo";
    private static final String MISSING = "let n = 1";

    static String run() throws Exception {
        var out = new LinkedHashMap<String, Object>();
        var initialize = new InitializeParams();
        var workspace = new WorkspaceClientCapabilities();
        var editCapabilities = new WorkspaceEditCapabilities();
        editCapabilities.setDocumentChanges(true);
        workspace.setWorkspaceEdit(editCapabilities);
        var capabilities = new ClientCapabilities();
        capabilities.setWorkspace(workspace);
        initialize.setCapabilities(capabilities);

        try (var wire = new Stage5Scenario.Wire(new LanguageServerImpl(Map.of()))) {
            assertNull(wire.lsp.initialize(initialize).get(30, TimeUnit.SECONDS).getCapabilities().getCodeActionProvider());
            var errors = wire.open(MISSING);
            assertTrue(actions(wire, errors, point(9), null).isEmpty());
        }
        try (var wire = new Stage5Scenario.Wire(new LanguageServerImpl(Map.of(), null, null, null, new DemoSyntaxQuickFix()))) {
            wire.lsp.initialize(new InitializeParams()).get(30, TimeUnit.SECONDS);
            assertTrue(actions(wire, wire.open(MISSING), point(9), null).isEmpty(), "バージョン付き編集が必要");
        }
        var local = new LanguageServerImpl(Map.of(), null, null, null, new DemoSyntaxQuickFix());
        try (var wire = new Stage5Scenario.Wire(local)) {
            var provider = wire.lsp.initialize(initialize).get(30, TimeUnit.SECONDS).getCapabilities().getCodeActionProvider();
            assertEquals(List.of(CodeActionKind.QuickFix), provider.getRight().getCodeActionKinds());
            out.put("provider", provider.getRight());

            var original = wire.open(MISSING);
            out.put("before", original);
            var action = fix(wire, original, 9, 1);
            out.put("fix", action);
            out.put("wrongRange", empty(wire, original, point(0), null));
            out.put("wrongKind", empty(wire, original, point(9), List.of(CodeActionKind.Refactor)));
            out.put("withoutDiagnostic", empty(wire, List.of(), point(9), null));

            var foreign = wire.open("file:///other.demo", MISSING);
            out.put("otherDocument", empty(wire, foreign, point(9), null));
            out.put("after", wire.change(apply(MISSING, action), 2));
            assertEquals(List.of(), out.get("after"));
            out.put("normal", empty(wire, original, point(9), null));

            var current = wire.change("let n = 2", 3);
            assertEquals(original.getFirst().getRange(), current.getFirst().getRange());
            assertEquals(original.getFirst().getMessage(), current.getFirst().getMessage());
            assertEquals(original.getFirst().getCode(), current.getFirst().getCode());
            out.put("stale", empty(wire, original, point(9), null));
            assertNotNull(fix(wire, current, 9, 3));

            var ambiguous = wire.change("let n =", 4);
            out.put("ambiguous", empty(wire, ambiguous, point(7), null));
            var unclosedBlock = wire.change("{ " + MISSING, 5);
            out.put("repairStillInvalid", empty(wire, unclosedBlock, point(11), null));

            String bmp = "note \"😀\"; " + MISSING;
            var unicode = wire.change(bmp, 6);
            var unicodeAction = fix(wire, unicode, 20, 6);
            out.put("utf16", unicodeAction);
            assertTrue(wire.change(apply(bmp, unicodeAction), 7).isEmpty());

            wire.closeDocument();
            out.put("closed", empty(wire, original, point(9), null));
            wire.open(MISSING);
            out.put("reopened", empty(wire, original, point(9), null));
        }
        return new MessageJsonHandler(Map.of()).getGson().newBuilder().setPrettyPrinting().create().toJson(out) + "\n";
    }

    private static CodeAction fix(Stage5Scenario.Wire wire, List<Diagnostic> errors, int column, int version)
            throws Exception {
        var range = point(column);
        var result = actions(wire, errors, range, List.of(CodeActionKind.QuickFix));
        assertEquals(1, result.size());
        var action = result.getFirst().getRight();
        assertEquals(CodeActionKind.QuickFix, action.getKind());
        var change = action.getEdit().getDocumentChanges().getFirst().getLeft();
        assertEquals(new VersionedTextDocumentIdentifier(URI, version), change.getTextDocument());
        assertEquals(List.of(new TextEdit(range, ";")), change.getEdits());
        return action;
    }

    private static List<Either<Command, CodeAction>> empty(Stage5Scenario.Wire wire, List<Diagnostic> errors,
            Range range, List<String> only) throws Exception {
        var actions = actions(wire, errors, range, only);
        assertTrue(actions.isEmpty());
        return actions;
    }

    private static List<Either<Command, CodeAction>> actions(Stage5Scenario.Wire wire, List<Diagnostic> errors,
                                                            Range range, List<String> only) throws Exception {
        var params = new CodeActionParams(new TextDocumentIdentifier(URI), range, new CodeActionContext(errors, only));
        return wire.lsp.getTextDocumentService().codeAction(params).get(30, TimeUnit.SECONDS);
    }

    private static String apply(String text, CodeAction action) {
        var edit = action.getEdit().getDocumentChanges().getFirst().getLeft().getEdits().getFirst();
        int offset = edit.getRange().getStart().getCharacter();
        return text.substring(0, offset) + edit.getNewText() + text.substring(offset);
    }

    private static Range point(int column) {
        var position = new Position(0, column);
        return new Range(position, position);
    }
}
