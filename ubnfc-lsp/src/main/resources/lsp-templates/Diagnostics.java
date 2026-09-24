package __PACKAGE__;

import java.util.ArrayList;
import java.util.List;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DiagnosticRelatedInformation;
import org.eclipse.lsp4j.DiagnosticSeverity;
import org.eclipse.lsp4j.Location;
import __RT_PACKAGE__.Severity;

/**
 * 生成パーサの解析結果を LSP の診断へ写す。位置変換は {@link Positions} だけが持つ。
 *
 * <p>出るのは 3 種類。
 * <ul>
 *   <li>構文エラー — 主位置・{@code expected} 表示ラベル・最深規則</li>
 *   <li>scope 診断 — 未定義参照など（WARNING）</li>
 *   <li>前半だけの scope 診断 — 全文が通らない文書で、主失敗位置までを解析し直して得たもの。
 *       {@code code} を {@code scope-prefix} にして区別できるようにする（D-062）</li>
 * </ul>
 */
public final class Diagnostics {
    /** LSP の {@code Diagnostic.source}。VS Code の問題パネルに出る発生源名。 */
    public static final String SOURCE = "__SOURCE__";
    /** 期待ラベルをメッセージへ並べる上限。超えた分は件数だけ書く。 */
    private static final int MAX_EXPECTED = 8;

    private Diagnostics() {}

    public static List<Diagnostic> of(Analysis analysis) {
        var out = new ArrayList<Diagnostic>();
        var positions = analysis.positions();
        if (analysis.failure() != null) {
            out.add(simple(positions.rangeOf(0, 0), "解析器の内部エラー: " + analysis.failure(),
                DiagnosticSeverity.Error, "internal"));
            return out;
        }
        var primary = analysis.primary();
        if (primary != null && !primary.ok()) out.add(syntax(analysis.uri(), positions, primary.diagnostics()));
        var scope = analysis.scope();
        if (scope != null)
            for (var diagnostic : scope.diagnostics()) out.add(scope(positions, diagnostic, analysis.scopedIsPrefix()));
        return out;
    }

    private static Diagnostic simple(org.eclipse.lsp4j.Range range, String message, DiagnosticSeverity severity, String code) {
        var diagnostic = new Diagnostic(range, message);
        diagnostic.setSeverity(severity);
        diagnostic.setSource(SOURCE);
        diagnostic.setCode(code);
        return diagnostic;
    }

    private static Diagnostic syntax(String uri, Positions positions, __API_PACKAGE__.Diagnostic info) {
        // trailing_input の主位置は「採用した prefix の消費末尾」（D-024）。編集者に見せたいのは
        // そこではなく最遠到達位置なので、診断の位置だけ farthest へ寄せる（ParseResult は変えない）。
        boolean trailing = info != null && "trailing_input".equals(info.kind());
        int offset = trailing && info.farthestCp() >= 0 ? info.farthestCp()
            : info == null || info.offset() < 0 ? 0 : info.offset();
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

    private static Diagnostic scope(Positions positions, __RT_PACKAGE__.ScopeStore.Diagnostic info, boolean prefix) {
        var diagnostic = new Diagnostic(
            positions.rangeOf(info.offset(), info.offset() + Math.max(0, info.len())),
            prefix ? info.msg() + "（構文エラーより前の範囲だけを見た結果）" : info.msg());
        diagnostic.setSeverity(severity(info.severity()));
        diagnostic.setSource(SOURCE);
        diagnostic.setCode(prefix ? "scope-prefix" : "scope");
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
        boolean trailing = "trailing_input".equals(info.kind());
        List<String> expected = trailing ? info.farthestExpected() : info.expected();
        if (trailing && (expected == null || expected.isEmpty()))
            return "構文エラー: 入力の終わりを期待しました（余分な入力）";
        if (expected == null || expected.isEmpty()) return "構文エラー: ここでは解析できません";
        var text = new StringBuilder(trailing ? "構文エラー（余分な入力）: " : "構文エラー: ");
        int shown = Math.min(MAX_EXPECTED, expected.size());
        for (int i = 0; i < shown; i++) {
            if (i > 0) text.append(" | ");
            text.append(expected.get(i));
        }
        if (expected.size() > shown) text.append(" ほか ").append(expected.size() - shown).append(" 件");
        return text.append(" を期待しました").toString();
    }
}
