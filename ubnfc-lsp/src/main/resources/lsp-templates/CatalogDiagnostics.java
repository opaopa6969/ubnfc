package __PACKAGE__;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.NoSuchFileException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DiagnosticSeverity;

/** @catalog の context と明示指定した UTF-8 ファイルを照合する。自動探索や暗黙の既定値はない。 */
public final class CatalogDiagnostics {
    private record Binding(Path path, String context) {}
    private record Entry(String head, int minSuffix, boolean prefix) {
        boolean matches(String text) {
            return prefix ? text.startsWith(head) && text.length() - head.length() >= minSuffix : text.equals(head);
        }
    }
    private Map<String, Binding> paths = Map.of();
    private boolean invalid;

    /** initializationOptions と didChangeConfiguration.settings の共通形式: {catalogs:{context:絶対パス}}。 */
    public void configure(Object settings) {
        var next = new TreeMap<String, Binding>();
        invalid = false;
        try {
            JsonElement json = new Gson().toJsonTree(settings);
            if (json != null && !json.isJsonNull()) {
                var object = json.getAsJsonObject();
                if (object.has("catalogs"))
                    for (var entry : object.getAsJsonObject("catalogs").entrySet()) {
                        var value = entry.getValue();
                        var pathValue = value.isJsonObject() ? value.getAsJsonObject().get("path") : value;
                        if (pathValue == null || !pathValue.isJsonPrimitive() || !pathValue.getAsJsonPrimitive().isString())
                            throw new IllegalArgumentException();
                        String context = null;
                        if (value.isJsonObject() && value.getAsJsonObject().has("context")) {
                            var filter = value.getAsJsonObject().get("context");
                            if (!filter.isJsonPrimitive() || !filter.getAsJsonPrimitive().isString()) throw new IllegalArgumentException();
                            context = filter.getAsString();
                        }
                        Path path = Path.of(pathValue.getAsString());
                        if (!path.isAbsolute() || !path.toString().endsWith(".tecatalog")) throw new IllegalArgumentException();
                        next.put(entry.getKey(), new Binding(path.normalize(), context));
                    }
            }
            paths = next;
        } catch (RuntimeException error) {
            paths = Map.of(); invalid = true;
        }
    }

    public List<Diagnostic> of(Analysis analysis) {
        if (invalid) return List.of(problem(analysis, "catalog-config", "catalogs は context と .tecatalog の絶対パスの対応表で指定してください"));
        if (paths.isEmpty()) return List.of();
        var out = new ArrayList<Diagnostic>();
        var captures = analysis.catalogs();
        for (var entry : paths.entrySet()) {
            String context = entry.getKey();
            List<Entry> names;
            try {
                names = read(entry.getValue().path(), entry.getValue().context());
            } catch (NoSuchFileException error) {
                out.add(problem(analysis, "catalog-missing", "カタログが見つかりません: " + context)); continue;
            } catch (IOException error) {
                out.add(problem(analysis, "catalog-io", "カタログを読めません: " + context)); continue;
            } catch (IllegalArgumentException error) {
                out.add(problem(analysis, "catalog-invalid", "カタログの内容が不正です: " + context)); continue;
            }
            // 成功した全文、または Analysis が保証する成功した前半の capture だけを使う。
            for (var capture : captures.getOrDefault(context, List.of())) {
                var span = capture.span();
                String text = Analysis.head(analysis.text(), span.end());
                text = text.substring(text.offsetByCodePoints(0, span.start())).strip();
                String captured = text;
                if (names.stream().noneMatch(name -> name.matches(captured))) {
                    var diagnostic = problem(analysis, analysis.scopedIsPrefix() ? "catalog-prefix" : "catalog-unknown",
                        "カタログにない値: " + text + "（" + context + "）"
                        + (analysis.scopedIsPrefix() ? "（構文エラーより前の範囲だけを見た結果）" : ""));
                    diagnostic.setRange(analysis.positions().rangeOf(span.start(), span.end()));
                    out.add(diagnostic);
                }
            }
        }
        return out;
    }

    /** canonical v1 の exact / prefixWithSuffix。IR の context と行の用途分類は別物。 */
    private static List<Entry> read(Path path, String context) throws IOException {
        if (Files.size(path) > 1_048_576) throw new IllegalArgumentException();
        var lines = Files.readAllLines(path);
        if (lines.isEmpty() || !lines.get(0).equals("tinyexpression-catalog-v1")) throw new IllegalArgumentException();
        var names = new ArrayList<Entry>();
        for (String line : lines.subList(1, lines.size())) {
            if (line.isBlank() || line.startsWith("#")) continue;
            String[] fields = line.split("\\|", -1);
            boolean prefix = fields[0].equals("prefixWithSuffix");
            if (fields.length < 3 || fields[1].isBlank()
                    || (prefix ? fields.length != 6 : !fields[0].equals("exact") || fields.length > 4))
                throw new IllegalArgumentException();
            String rowContext = prefix ? fields[5].strip() : fields.length == 4 ? fields[3].strip() : "";
            int minSuffix = prefix ? Integer.parseInt(fields[3]) : 0;
            if (minSuffix < 0) throw new IllegalArgumentException();
            if (context == null || rowContext.isBlank() || rowContext.equals(context))
                names.add(new Entry(fields[1].strip() + (prefix ? fields[2] : ""), minSuffix, prefix));
        }
        return names;
    }

    private static Diagnostic problem(Analysis analysis, String code, String message) {
        var diagnostic = FrontValidation.problem(analysis, code, message);
        diagnostic.setSource("ubnfc-catalog");
        diagnostic.setSeverity(DiagnosticSeverity.Warning);
        return diagnostic;
    }
}
