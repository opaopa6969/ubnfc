package org.ubnfc.p4.lsp;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.stream.Stream;
import org.eclipse.lsp4j.Diagnostic;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

/**
 * 台本クライアントを生成サーバにぶつけて、診断の JSON を golden と突き合わせる。
 *
 * <p>golden の更新は {@code mvn test -Dubnfc.golden.update=true}。
 * {@code non-bmp.tiny} は BMP 外文字（U+1F600 2 個）を前の行に置いてある。
 * code point 位置を UTF-16 index として渡す実装だと、後続行の column が 2 ずれる
 * （旧 tinyexpression-p4-lsp-vscode で実測した誤り）。この golden がそれを止める。
 */
class GoldenDiagnosticsTest {
    private static final Path FIXTURES = Path.of("src/test/resources/fixtures");
    private static final Path GOLDEN = Path.of("src/test/resources/golden");
    private static final boolean UPDATE = Boolean.getBoolean("ubnfc.golden.update");

    @TestFactory Stream<DynamicTest> fixtures() throws Exception {
        var paths = Files.list(FIXTURES).filter(p -> p.toString().endsWith(".tiny")).sorted().toList();
        var client = new ScriptedClient();
        var tests = paths.stream().map(path -> DynamicTest.dynamicTest(path.getFileName().toString(), () -> {
            String text = Files.readString(path, StandardCharsets.UTF_8);
            String actual = render(client.diagnosticsOf(path.toUri().toString(), text));
            Path golden = GOLDEN.resolve(path.getFileName() + ".json");
            if (UPDATE) {
                Files.createDirectories(GOLDEN);
                Files.writeString(golden, actual, StandardCharsets.UTF_8);
            }
            assertEquals(Files.readString(golden, StandardCharsets.UTF_8), actual, path.getFileName().toString());
        })).toList();
        return Stream.concat(tests.stream(),
            Stream.of(DynamicTest.dynamicTest("shutdown", client::close)));
    }

    /** 安定した並びで書く小さな JSON writer（依存を足さないため手書き）。 */
    static String render(List<Diagnostic> diagnostics) {
        var out = new StringBuilder("[\n");
        for (int i = 0; i < diagnostics.size(); i++) {
            var diagnostic = diagnostics.get(i);
            var range = diagnostic.getRange();
            out.append("  {\n")
               .append("    \"range\": [").append(range.getStart().getLine()).append(", ")
               .append(range.getStart().getCharacter()).append(", ")
               .append(range.getEnd().getLine()).append(", ")
               .append(range.getEnd().getCharacter()).append("],\n")
               .append("    \"severity\": \"").append(diagnostic.getSeverity()).append("\",\n")
               .append("    \"source\": ").append(quote(diagnostic.getSource())).append(",\n")
               .append("    \"code\": ").append(quote(diagnostic.getCode() == null ? null : diagnostic.getCode().getLeft())).append(",\n")
               .append("    \"message\": ").append(quote(diagnostic.getMessage())).append('\n')
               .append("  }").append(i + 1 < diagnostics.size() ? "," : "").append('\n');
        }
        return out.append("]\n").toString();
    }

    private static String quote(String value) {
        if (value == null) return "null";
        var out = new StringBuilder("\"");
        value.codePoints().forEach(point -> {
            switch (point) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                default -> {
                    if (point < 0x20) out.append(String.format("\\u%04x", point));
                    else out.appendCodePoint(point);
                }
            }
        });
        return out.append('"').toString();
    }
}
