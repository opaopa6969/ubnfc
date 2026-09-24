package __PACKAGE__;

import java.util.ArrayList;
import java.util.List;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DiagnosticRelatedInformation;
import org.eclipse.lsp4j.DiagnosticSeverity;
import org.eclipse.lsp4j.Location;
import __API_PACKAGE__.ParseResult;
import __RT_PACKAGE__.Severity;

/**
 * 生成パーサの {@link ParseResult} を LSP の診断へ写す。位置変換は {@link Positions} だけが持つ。
 *
 * <p>出るのは 2 種類だけ（段階 2 の範囲）。
 * <ul>
 *   <li>構文エラー — {@code result.diagnostics()} の主位置・{@code expected} 表示ラベル・最深規則</li>
 *   <li>scope 診断 — {@code result.scope().diagnostics()}（未定義参照は WARNING）</li>
 * </ul>
 */
public final class Diagnostics {
    /** LSP の {@code Diagnostic.source}。VS Code の問題パネルに出る発生源名。 */
    public static final String SOURCE = "__SOURCE__";
    /** 期待ラベルをメッセージへ並べる上限。超えた分は件数だけ書く。 */
    private static final int MAX_EXPECTED = 8;

    private Diagnostics() {}

    public static List<Diagnostic> of(String uri, String text, ParseResult<?> result) {
        return of(uri, new Positions(text), result);
    }

    public static List<Diagnostic> of(String uri, Positions positions, ParseResult<?> result) {
        var out = new ArrayList<Diagnostic>();
        if (!result.ok()) out.add(syntax(uri, positions, result.diagnostics()));
        var scope = result.scope();
        if (scope != null)
            for (var diagnostic : scope.diagnostics()) out.add(scope(positions, diagnostic));
        return out;
    }

    private static Diagnostic syntax(String uri, Positions positions, __API_PACKAGE__.Diagnostic info) {
        int offset = info == null || info.offset() < 0 ? 0 : info.offset();
        int length = info == null ? 0 : Math.max(0, info.length());
        var range = positions.rangeOf(offset, offset + length);
        var diagnostic = new Diagnostic(range, message(info));
        diagnostic.setSeverity(DiagnosticSeverity.Error);
        diagnostic.setSource(SOURCE);
        if (info != null) {
            String rule = info.deepestRule();
            diagnostic.setCode(rule == null ? info.kind() : rule);
            if (rule != null)
                diagnostic.setRelatedInformation(List.of(new DiagnosticRelatedInformation(
                    new Location(uri, range), "最深規則: " + rule)));
        }
        return diagnostic;
    }

    private static Diagnostic scope(Positions positions, __RT_PACKAGE__.ScopeStore.Diagnostic info) {
        var diagnostic = new Diagnostic(
            positions.rangeOf(info.offset(), info.offset() + Math.max(0, info.len())), info.msg());
        diagnostic.setSeverity(severity(info.severity()));
        diagnostic.setSource(SOURCE);
        diagnostic.setCode("scope");
        return diagnostic;
    }

    static DiagnosticSeverity severity(Severity severity) {
        if (severity == null) return DiagnosticSeverity.Error;
        return switch (severity) {
            case ERROR -> DiagnosticSeverity.Error;
            case WARNING -> DiagnosticSeverity.Warning;
            case INFO -> DiagnosticSeverity.Information;
            case HINT -> DiagnosticSeverity.Hint;
        };
    }

    /** 期待ラベルからメッセージを組む。ラベルは IR の {@code diagnostics.expectedDisplay} の Java 表示。 */
    static String message(__API_PACKAGE__.Diagnostic info) {
        if (info == null) return "構文エラー";
        if ("trailing_input".equals(info.kind())) return "構文エラー: 入力の終わりを期待しました（余分な入力）";
        List<String> expected = info.expected();
        if (expected == null || expected.isEmpty()) return "構文エラー: ここでは解析できません";
        var text = new StringBuilder("構文エラー: ");
        int shown = Math.min(MAX_EXPECTED, expected.size());
        for (int i = 0; i < shown; i++) {
            if (i > 0) text.append(" | ");
            text.append(expected.get(i));
        }
        if (expected.size() > shown) text.append(" ほか ").append(expected.size() - shown).append(" 件");
        return text.append(" を期待しました").toString();
    }
}
