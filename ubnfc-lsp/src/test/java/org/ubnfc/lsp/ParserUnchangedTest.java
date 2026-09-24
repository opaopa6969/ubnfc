package org.ubnfc.lsp;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * LSP は opt-in の feature（D-060）。「off のときは何も変わらない」を機械で押さえる。
 *
 * <ul>
 *   <li>{@code org.ubnfc.lsp.Main} を同じ出力先へ重ねても、{@code org.ubnfc.java.Main} が書いた
 *       ファイルは 1 byte も変わらない（追加されるだけ）</li>
 *   <li>生成パーサのソースに lsp4j / LSP への言及が 1 つも無い</li>
 *   <li>{@code --emit-lsp} を渡さないと何も出さない</li>
 * </ul>
 */
class ParserUnchangedTest {
    private static final Path IR = Path.of("..", "ir", "fixtures",
        "tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json");

    @Test void lspEmissionOnlyAddsFiles(@TempDir Path out) throws Exception {
        org.ubnfc.java.Main.main(new String[]{
            "--ir", IR.toString(), "--out", out.toString(), "--package", "org.ubnfc.p4.generated", "--no-emit-driver"});
        Map<Path, byte[]> before = snapshot(out);
        assertFalse(before.isEmpty(), "パーサが生成されていない");

        Main.main(new String[]{
            "--emit-lsp", "--ir", IR.toString(), "--out", out.toString(),
            "--package", "org.ubnfc.p4.lsp", "--parser-package", "org.ubnfc.p4.generated",
            "--language-id", "tinyexpression-p4", "--scanner-class", "org.ubnfc.p4.P4Scanners"});
        Map<Path, byte[]> after = snapshot(out);

        for (var entry : before.entrySet())
            assertArrayEquals(entry.getValue(), after.get(entry.getKey()), "LSP 生成でパーサ側が変わった: " + entry.getKey());
        assertTrue(after.size() > before.size(), "LSP のファイルが増えていない");
        for (Path path : after.keySet())
            if (!before.containsKey(path))
                assertTrue(path.toString().replace('\\', '/').startsWith("org/ubnfc/p4/lsp/"),
                    "LSP の出力が別パッケージへ漏れている: " + path);
    }

    @Test void generatedParserNeverMentionsLsp4j(@TempDir Path out) throws Exception {
        org.ubnfc.java.Main.main(new String[]{
            "--ir", IR.toString(), "--out", out.toString(), "--package", "org.ubnfc.p4.generated"});
        for (var entry : snapshot(out).entrySet()) {
            String source = new String(entry.getValue(), StandardCharsets.UTF_8);
            assertFalse(source.contains("lsp4j"), entry.getKey() + " が lsp4j に触れている");
            assertFalse(source.contains("org.eclipse."), entry.getKey() + " が eclipse に触れている");
        }
    }

    @Test void emitLspIsRequired(@TempDir Path out) {
        var error = assertThrows(IllegalArgumentException.class, () -> Main.main(new String[]{
            "--ir", IR.toString(), "--out", out.toString()}));
        assertTrue(error.getMessage().contains("--emit-lsp"), error.getMessage());
    }

    @Test void dapIsNotImplementedYet(@TempDir Path out) {
        var error = assertThrows(IllegalArgumentException.class, () -> Main.main(new String[]{
            "--emit-dap", "--ir", IR.toString(), "--out", out.toString()}));
        assertTrue(error.getMessage().contains("--emit-dap"), error.getMessage());
    }

    @Test void templatesHaveNoLeftoverPlaceholders() throws IOException {
        var emitter = new LspEmitter(org.ubnfc.java.ir.IrReader.read(IR), "org.ubnfc.p4.lsp",
            "org.ubnfc.p4.generated", "tinyexpression-p4", "org.ubnfc.p4.P4Scanners", null, null);
        var sources = emitter.sources();
        assertEquals(5, sources.size());
        assertEquals("TinyExpressionP4", emitter.grammar());
        assertEquals("Formula", emitter.entry());
        assertEquals("org.ubnfc.p4.lsp.ServerMain", emitter.serverMainClass());
        for (var entry : sources.entrySet()) {
            assertFalse(entry.getValue().contains("__"), entry.getKey() + " に未置換の placeholder がある");
            assertTrue(entry.getValue().startsWith("package org.ubnfc.p4.lsp;"), entry.getKey());
        }
    }

    private static Map<Path, byte[]> snapshot(Path root) throws IOException {
        var result = new LinkedHashMap<Path, byte[]>();
        try (Stream<Path> paths = Files.walk(root)) {
            for (Path path : paths.filter(Files::isRegularFile).sorted().toList())
                result.put(root.relativize(path), Files.readAllBytes(path));
        }
        return result;
    }
}
