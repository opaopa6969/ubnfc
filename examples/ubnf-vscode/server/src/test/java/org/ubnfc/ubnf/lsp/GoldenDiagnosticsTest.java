package org.ubnfc.ubnf.lsp;

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
 * {@code non-bmp.ubnf} は BMP 外文字（U+1F600 2 個）をブロックコメントに置いてある。
 * code point 位置を UTF-16 index として渡す実装だと、同じ行の後続の column が 2 ずれる
 * （旧 tinyexpression-p4-lsp-vscode で実測した誤り）。この golden がそれを止める。
 */
class GoldenDiagnosticsTest {
    private static final Path FIXTURES = Path.of("src/test/resources/fixtures");
    private static final Path GOLDEN = Path.of("src/test/resources/golden");
    private static final boolean UPDATE = Boolean.getBoolean("ubnfc.golden.update");

    @TestFactory Stream<DynamicTest> fixtures() throws Exception {
        var paths = Files.list(FIXTURES).filter(p -> p.toString().endsWith(".ubnf")).sorted().toList();
        var client = new ScriptedClient();
        var tests = paths.stream().map(path -> DynamicTest.dynamicTest(path.getFileName().toString(), () -> {
            String text = Files.readString(path, StandardCharsets.UTF_8);
            String actual = Goldens.diagnostics(client.diagnosticsOf(path.toUri().toString(), text));
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

}
