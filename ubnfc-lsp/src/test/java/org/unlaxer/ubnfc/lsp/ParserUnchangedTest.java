package org.unlaxer.ubnfc.lsp;

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
 *   <li>{@code org.unlaxer.ubnfc.lsp.Main} を同じ出力先へ重ねても、{@code org.unlaxer.ubnfc.java.Main} が書いた
 *       ファイルは 1 byte も変わらない（追加されるだけ）</li>
 *   <li>生成パーサのソースに lsp4j / LSP への言及が 1 つも無い</li>
 *   <li>{@code --emit-lsp} を渡さないと何も出さない</li>
 * </ul>
 */
class ParserUnchangedTest {
    private static final Path IR = Path.of("..", "ir", "frozen",
        "tinyexpression-p4-f86ce8a5.ir.json");

    @Test void lspEmissionOnlyAddsFiles(@TempDir Path out) throws Exception {
        org.unlaxer.ubnfc.java.Main.main(new String[]{
            "--ir", IR.toString(), "--out", out.toString(), "--package", "org.unlaxer.ubnfc.examples.p4.generated", "--no-emit-driver"});
        Map<Path, byte[]> before = snapshot(out);
        assertFalse(before.isEmpty(), "パーサが生成されていない");

        Main.main(new String[]{
            "--emit-lsp", "--ir", IR.toString(), "--out", out.toString(),
            "--package", "org.unlaxer.ubnfc.examples.p4.lsp", "--parser-package", "org.unlaxer.ubnfc.examples.p4.generated",
            "--language-id", "tinyexpression-p4", "--scanner-class", "org.unlaxer.ubnfc.examples.p4.P4Scanners"});
        Map<Path, byte[]> after = snapshot(out);

        for (var entry : before.entrySet())
            assertArrayEquals(entry.getValue(), after.get(entry.getKey()), "LSP 生成でパーサ側が変わった: " + entry.getKey());
        assertTrue(after.size() > before.size(), "LSP のファイルが増えていない");
        for (Path path : after.keySet())
            if (!before.containsKey(path))
                assertTrue(path.toString().replace('\\', '/').startsWith("org/unlaxer/ubnfc/examples/p4/lsp/"),
                    "LSP の出力が別パッケージへ漏れている: " + path);
    }

    @Test void generatedParserNeverMentionsLsp4j(@TempDir Path out) throws Exception {
        org.unlaxer.ubnfc.java.Main.main(new String[]{
            "--ir", IR.toString(), "--out", out.toString(), "--package", "org.unlaxer.ubnfc.examples.p4.generated"});
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

    @Test void dapRequiresLsp(@TempDir Path out) {
        var error = assertThrows(IllegalArgumentException.class, () -> Main.main(new String[]{
            "--emit-dap", "--ir", IR.toString(), "--out", out.toString()}));
        assertTrue(error.getMessage().contains("--emit-dap"), error.getMessage());
    }

    /**
     * DAP は opt-in（D-067）。{@code --emit-dap} を足しても
     * <b>パーサも LSP の 11 ファイルも 1 byte も変わらず</b>、
     * {@code <lsp-package>.dap} にファイルが増えるだけ。
     */
    @Test void dapEmissionOnlyAddsFiles(@TempDir Path out) throws Exception {
        String[] parser = {"--ir", IR.toString(), "--out", out.toString(),
            "--package", "org.unlaxer.ubnfc.examples.p4.generated", "--no-emit-driver"};
        String[] lsp = {"--emit-lsp", "--ir", IR.toString(), "--out", out.toString(),
            "--package", "org.unlaxer.ubnfc.examples.p4.lsp", "--parser-package", "org.unlaxer.ubnfc.examples.p4.generated",
            "--language-id", "tinyexpression-p4", "--scanner-class", "org.unlaxer.ubnfc.examples.p4.P4Scanners"};

        org.unlaxer.ubnfc.java.Main.main(parser);
        Main.main(lsp);
        Map<Path, byte[]> before = snapshot(out);

        var withDap = new java.util.ArrayList<String>(java.util.List.of(lsp));
        withDap.add("--emit-dap");
        Main.main(withDap.toArray(new String[0]));
        Map<Path, byte[]> after = snapshot(out);

        for (var entry : before.entrySet())
            assertArrayEquals(entry.getValue(), after.get(entry.getKey()),
                "DAP 生成でパーサ／LSP 側が変わった: " + entry.getKey());
        assertTrue(after.size() > before.size(), "DAP のファイルが増えていない");
        for (Path path : after.keySet())
            if (!before.containsKey(path))
                assertTrue(path.toString().replace('\\', '/').startsWith("org/unlaxer/ubnfc/examples/p4/lsp/dap/"),
                    "DAP の出力が別パッケージへ漏れている: " + path);
    }

    @Test void dapTemplatesHaveNoLeftoverPlaceholders() throws IOException {
        var lsp = new LspEmitter(org.unlaxer.ubnfc.java.ir.IrReader.read(IR), "org.unlaxer.ubnfc.examples.p4.lsp",
            "org.unlaxer.ubnfc.examples.p4.generated", "tinyexpression-p4", "org.unlaxer.ubnfc.examples.p4.P4Scanners", null, null);
        var dap = new DapEmitter(lsp, null);
        var sources = dap.sources();
        assertEquals(5, sources.size());
        assertEquals("org.unlaxer.ubnfc.examples.p4.lsp.dap", dap.packageName());
        assertEquals("org.unlaxer.ubnfc.examples.p4.lsp.dap.DapMain", dap.mainClass());
        for (var entry : sources.entrySet()) {
            assertFalse(entry.getValue().contains("__"), entry.getKey() + " に未置換の placeholder がある");
            assertTrue(entry.getValue().startsWith("package org.unlaxer.ubnfc.examples.p4.lsp.dap;"), entry.getKey());
        }
        // 反射ではなく IR から畳み込んだ switch であること（旧スタックとの違い）。
        String walk = sources.get("org/unlaxer/ubnfc/examples/p4/lsp/dap/AstWalk.java");
        assertFalse(walk.contains("java.lang.reflect"), "AstWalk が反射 API を使っている");
        assertEquals(86, dap.recordTypes().size());
        for (String type : dap.recordTypes())
            assertTrue(walk.contains("case " + type + " "), "AstWalk に case が無い: " + type);
        assertEquals(java.util.List.of("org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4AST"), dap.unionTypes());
        assertEquals("TinyExpressionP4::Formula",
            dap.ruleIds().get("TinyExpressionP4AST.FormulaExpr"));
    }

    @Test void templatesHaveNoLeftoverPlaceholders() throws IOException {
        var emitter = new LspEmitter(org.unlaxer.ubnfc.java.ir.IrReader.read(IR), "org.unlaxer.ubnfc.examples.p4.lsp",
            "org.unlaxer.ubnfc.examples.p4.generated", "tinyexpression-p4", "org.unlaxer.ubnfc.examples.p4.P4Scanners", null, null);
        var sources = emitter.sources();
        assertEquals(11, sources.size());
        assertEquals("TinyExpressionP4", emitter.grammar());
        assertEquals("Formula", emitter.entry());
        assertEquals("org.unlaxer.ubnfc.examples.p4.lsp.ServerMain", emitter.serverMainClass());
        for (var entry : sources.entrySet()) {
            assertFalse(entry.getValue().contains("__"), entry.getKey() + " に未置換の placeholder がある");
            assertTrue(entry.getValue().startsWith("package org.unlaxer.ubnfc.examples.p4.lsp;"), entry.getKey());
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
