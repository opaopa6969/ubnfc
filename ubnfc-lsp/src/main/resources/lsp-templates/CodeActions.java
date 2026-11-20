package __PACKAGE__;

import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import org.eclipse.lsp4j.CodeAction;
import org.eclipse.lsp4j.CodeActionKind;
import org.eclipse.lsp4j.CodeActionParams;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.Position;
import org.eclipse.lsp4j.Range;
import org.eclipse.lsp4j.TextDocumentEdit;
import org.eclipse.lsp4j.TextEdit;
import org.eclipse.lsp4j.VersionedTextDocumentIdentifier;
import org.eclipse.lsp4j.WorkspaceEdit;
import org.eclipse.lsp4j.jsonrpc.messages.Either;

final class CodeActions {
    private CodeActions() {}

    static CodeAction at(DocumentStore.Snapshot document, CodeActionParams params, SyntaxQuickFix quickFix) {
        var context = params.getContext();
        if (context.getOnly() != null && !context.getOnly().contains(CodeActionKind.QuickFix)) return null;

        var analysis = document.analysis();
        var current = Diagnostics.primarySyntax(analysis, document.diagnosticId());
        if (current == null || analysis.primary().diagnostics() == null
                || !contains(params.getRange(), current.getRange().getStart())
                || context.getDiagnostics().stream().noneMatch(sent -> matches(sent, current, document.diagnosticId())))
            return null;

        var insertion = quickFix.forDiagnostic(analysis, analysis.primary().diagnostics());
        if (insertion == null || insertion.title() == null || insertion.title().isBlank()
                || insertion.text() == null || insertion.text().isEmpty()) return null;

        Position start = current.getRange().getStart();
        if (start.getCharacter() > analysis.positions().columnsOf(start.getLine())) return null;
        String source = analysis.text();
        int offset = source.offsetByCodePoints(0, analysis.positions().offsetOf(start));
        if (!analysis.parsesCleanly(source.substring(0, offset) + insertion.text() + source.substring(offset))) return null;

        var edit = new TextEdit(new Range(start, start), insertion.text());
        var identifier = new VersionedTextDocumentIdentifier(analysis.uri(), document.version());
        var action = new CodeAction(insertion.title());
        action.setKind(CodeActionKind.QuickFix);
        action.setDiagnostics(List.of(current));
        action.setEdit(new WorkspaceEdit(List.of(Either.forLeft(new TextDocumentEdit(identifier, List.of(edit))))));
        return action;
    }

    private static boolean contains(Range range, Position position) {
        return compare(range.getStart(), position) <= 0 && compare(position, range.getEnd()) <= 0;
    }

    private static int compare(Position left, Position right) {
        int line = Integer.compare(left.getLine(), right.getLine());
        return line != 0 ? line : Integer.compare(left.getCharacter(), right.getCharacter());
    }

    private static boolean matches(Diagnostic sent, Diagnostic current, String expectedId) {
        return Objects.equals(sent.getSource(), current.getSource())
            && Objects.equals(sent.getCode(), current.getCode())
            && Objects.equals(sent.getRange(), current.getRange())
            && expectedId.equals(diagnosticId(sent.getData()));
    }

    private static Object diagnosticId(Object data) {
        if (data instanceof Map<?, ?> map) return map.get(Diagnostics.QUICK_FIX_ID);
        if (data instanceof JsonObject object && object.get(Diagnostics.QUICK_FIX_ID) instanceof JsonPrimitive id
                && id.isString()) return id.getAsString();
        return null;
    }
}
