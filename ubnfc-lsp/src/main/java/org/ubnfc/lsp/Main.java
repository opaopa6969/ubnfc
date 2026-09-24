package org.ubnfc.lsp;

import java.io.IOException;
import java.nio.file.Path;
import org.ubnfc.java.ir.IrReader;

/**
 * LSP サーバ生成器の入口。LSP/DAP は opt-in なので、<b>明示的に {@code --emit-lsp} を渡したときだけ</b>
 * 何かを出す（D-060）。既定のパーサ生成（{@code org.ubnfc.java.Main}）はこの module を知らない。
 */
public final class Main {
    private static final String USAGE = """
        usage: ubnfc-lsp --emit-lsp --ir <grammar.ir.json> --out <dir> [options]
          --package <x.y>          生成する LSP サーバのパッケージ（既定: <parser-package>.lsp）
          --parser-package <x.y>   生成パーサのパッケージ（org.ubnfc.java.Main の --package と同じ値）
          --language-id <id>       LSP / VS Code の languageId（既定: 文法名の小文字）
          --scanner-class <fqcn>   extern token scanner を持つクラス（public static Map<String,TokenScanner> ALL）
          --server-name <name>     initialize が返す serverInfo.name
          --server-version <v>     initialize が返す serverInfo.version
          --emit-tmlanguage <path> TextMate 文法（*.tmLanguage.json）も書く（生成物・毎回上書き）
        LSP は opt-in feature。--emit-lsp を付けないと何も出さない。--emit-dap は未実装（段階 4）。""";

    private Main() {}

    public static void main(String[] args) throws IOException {
        if (args.length == 0 || args.length == 1 && args[0].equals("--help")) { System.out.println(USAGE); return; }
        String ir = null, out = null, pkg = null, parserPkg = null, languageId = null;
        String scannerClass = null, serverName = null, serverVersion = null, tmLanguage = null;
        boolean emitLsp = false;
        for (int i = 0; i < args.length; i++) {
            String option = args[i];
            if (option.equals("--emit-lsp")) { emitLsp = true; continue; }
            if (option.equals("--emit-dap"))
                throw new IllegalArgumentException("--emit-dap は未実装（LSP と同じ opt-in 機構で段階 4）");
            if (i + 1 == args.length) throw new IllegalArgumentException("Missing value for " + option);
            String value = args[++i];
            switch (option) {
                case "--ir" -> { if (ir != null) throw new IllegalArgumentException("Duplicate --ir"); ir = value; }
                case "--out" -> { if (out != null) throw new IllegalArgumentException("Duplicate --out"); out = value; }
                case "--package" -> { if (pkg != null) throw new IllegalArgumentException("Duplicate --package"); pkg = value; }
                case "--parser-package" -> { if (parserPkg != null) throw new IllegalArgumentException("Duplicate --parser-package"); parserPkg = value; }
                case "--language-id" -> { if (languageId != null) throw new IllegalArgumentException("Duplicate --language-id"); languageId = value; }
                case "--scanner-class" -> { if (scannerClass != null) throw new IllegalArgumentException("Duplicate --scanner-class"); scannerClass = value; }
                case "--server-name" -> { if (serverName != null) throw new IllegalArgumentException("Duplicate --server-name"); serverName = value; }
                case "--server-version" -> { if (serverVersion != null) throw new IllegalArgumentException("Duplicate --server-version"); serverVersion = value; }
                case "--emit-tmlanguage" -> { if (tmLanguage != null) throw new IllegalArgumentException("Duplicate --emit-tmlanguage"); tmLanguage = value; }
                default -> throw new IllegalArgumentException("Unknown option: " + option);
            }
        }
        if (!emitLsp) throw new IllegalArgumentException("--emit-lsp が要ります（LSP は opt-in feature）");
        if (ir == null || out == null) throw new IllegalArgumentException("--ir と --out が要ります");
        var grammarIr = IrReader.read(Path.of(ir));
        var emitter = new LspEmitter(grammarIr, pkg, parserPkg, languageId, scannerClass, serverName, serverVersion);
        emitter.write(Path.of(out));
        if (tmLanguage != null) {
            Path path = Path.of(tmLanguage);
            if (path.getParent() != null) java.nio.file.Files.createDirectories(path.getParent());
            java.nio.file.Files.writeString(path,
                new TmLanguageEmitter(grammarIr, emitter.languageId(), emitter.grammar()).json(),
                java.nio.charset.StandardCharsets.UTF_8);
        }
    }
}
