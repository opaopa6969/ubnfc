package __PACKAGE__;

import com.google.gson.JsonParser;
import java.io.IOException;
import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.concurrent.TimeUnit;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DiagnosticSeverity;

/** 起動時に明示指定した front を呼ぶ。検査規則を JVM 側へ複製しない。 */
public final class FrontValidation {
    private FrontValidation() {}

    public static List<Diagnostic> of(Analysis analysis, Path executable) {
        if (executable == null) return List.of();
        Path work = null;
        Process process = null;
        try {
            work = Files.createTempDirectory("ubnfc-validation-");
            Path source = snapshot(work.resolve("source"), analysis);
            Path output = work.resolve("report.json");
            process = new ProcessBuilder(executable.toString(), "validate", "--grammar", source.toString(),
                "--format", "json").redirectOutput(output.toFile()).redirectError(ProcessBuilder.Redirect.DISCARD).start();
            if (!process.waitFor(10, TimeUnit.SECONDS))
                return List.of(problem(analysis, "front-timeout", "文法の意味検査が制限時間を超えました"));
            // validate は意味エラーでも JSON を返す（終了値 1）。I/O エラー等は JSON と混ぜない。
            if (process.exitValue() != 0 && process.exitValue() != 1 || Files.size(output) > 1_048_576)
                return List.of(problem(analysis, "front-failed", "文法の意味検査に失敗しました"));
            var report = JsonParser.parseString(Files.readString(output)).getAsJsonObject();
            var out = new ArrayList<Diagnostic>();
            for (var value : report.getAsJsonArray("issues")) {
                var issue = value.getAsJsonObject();
                String message = issue.get("message").getAsString();
                String hint = issue.get("hint").getAsString();
                if (!hint.isBlank()) message += "（" + hint + "）";
                // front の公開 JSON には span がない。推測した規則位置を返さず文書先頭に集約する。
                var diagnostic = problem(analysis, issue.get("code").getAsString(), message);
                diagnostic.setSeverity("WARNING".equals(issue.get("severity").getAsString())
                    ? DiagnosticSeverity.Warning : DiagnosticSeverity.Error);
                out.add(diagnostic);
            }
            return out;
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            return List.of(problem(analysis, "front-cancelled", "文法の意味検査が中断されました"));
        } catch (IOException | RuntimeException error) {
            return List.of(problem(analysis, "front-unavailable", "文法の意味検査を実行できません（実行ファイル・JSON 応答を確認してください）"));
        } finally {
            if (process != null && process.isAlive()) process.destroyForcibly();
            if (work != null) {
                // 元文書・import 先には書き込まない。
                try (var paths = Files.walk(work)) {
                    for (Path path : paths.sorted(Comparator.reverseOrder()).toList()) Files.deleteIfExists(path);
                } catch (IOException ignored) { /* 一時領域の削除失敗で診断を隠さない。 */ }
            }
        }
    }

    /** import の STRING 終端だけを絶対パスへ写す。ディレクトリ全体の複製・探索はしない。 */
    private static Path snapshot(Path root, Analysis analysis) throws IOException {
        Files.createDirectories(root);
        URI uri = URI.create(analysis.uri());
        var text = new StringBuilder(analysis.text());
        if ("file".equals(uri.getScheme())) {
            Path parent = Path.of(uri).toAbsolutePath().normalize().getParent();
            var imports = analysis.lexical().stream()
                .filter(item -> "UBNF::ImportDecl".equals(item.ruleId()) && item.token() != null
                    && (item.token().startsWith("'") || item.token().startsWith("\"")))
                .sorted(Comparator.comparingInt((__API_PACKAGE__.ParseResult.Lexical item) -> item.span().start()).reversed()).toList();
            for (var item : imports) {
                String path = JsonParser.parseString(item.token()).getAsString();
                if (!path.startsWith("pkg:") && Path.of(path).isAbsolute()) continue;
                String absolute;
                if (path.startsWith("pkg:")) {
                    String id = path.substring(4);
                    if (java.util.Arrays.stream(id.split("/", -1)).anyMatch(part -> part.isEmpty()
                            || part.equals(".") || part.equals("..") || part.contains("\\") || part.contains(":"))) continue;
                    // front の公開仮想パス。元文書の manifest / lock / hash 検査をそのまま使う。
                    absolute = parent.resolve(".ubnf-packages").resolve(id).toString();
                } else absolute = parent.resolve(path).normalize().toString();
                String quoted = "'" + absolute.replace("\\", "\\\\").replace("'", "\\'") + "'";
                int start = analysis.text().offsetByCodePoints(0, item.span().start());
                int end = analysis.text().offsetByCodePoints(0, item.span().end());
                text.replace(start, end, quoted);
            }
        }
        Path source = root.resolve("document.ubnf");
        Files.writeString(source, text);
        return source;
    }

    static Diagnostic problem(Analysis analysis, String code, String message) {
        var diagnostic = new Diagnostic(analysis.positions().rangeOf(0, 0), message);
        diagnostic.setSource("ubnfc-front");
        diagnostic.setSeverity(DiagnosticSeverity.Error);
        diagnostic.setCode(code);
        return diagnostic;
    }
}
