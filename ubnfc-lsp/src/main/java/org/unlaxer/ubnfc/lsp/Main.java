package org.unlaxer.ubnfc.lsp;

import java.io.IOException;
import java.nio.file.Path;
import org.unlaxer.ubnfc.java.ir.IrReader;

/**
 * LSP サーバ生成器の入口。LSP/DAP は opt-in なので、<b>明示的に {@code --emit-lsp} を渡したときだけ</b>
 * 何かを出す（D-060）。既定のパーサ生成（{@code org.unlaxer.ubnfc.java.Main}）はこの module を知らない。
 */
public final class Main {
    private static final String USAGE = """
        usage: ubnfc-lsp --emit-lsp --ir <grammar.ir.json> --out <dir> [options]
          --package <x.y>          生成する LSP サーバのパッケージ（既定: <parser-package>.lsp）
          --parser-package <x.y>   生成パーサのパッケージ（org.unlaxer.ubnfc.java.Main の --package と同じ値。既定: 文法の package、無ければ org.unlaxer.ubnfc.generated.<文法名小文字>）
          --language-id <id>       LSP / VS Code の languageId（既定: 文法名の小文字）
          --scanner-class <fqcn>   extern token scanner を持つクラス（public static Map<String,TokenScanner> ALL）
          --server-name <name>     initialize が返す serverInfo.name
          --server-version <v>     initialize が返す serverInfo.version
          --emit-tmlanguage <path> TextMate 文法（*.tmLanguage.json）も書く（生成物・毎回上書き）
          --emit-dap               DAP（文法デバッガ）も出す（既定 off。--emit-lsp と併用する）
          --dap-package <x.y>      DAP の出力パッケージ（既定: <package>.dap）
        LSP / DAP は opt-in feature。--emit-lsp を付けないと何も出さない。
        --emit-dap は LSP 包みの Positions / Meta / Analysis を使うので --emit-lsp が要る。""";

    private Main() {}

    public static void main(String[] args) throws IOException {
        if (args.length == 0 || args.length == 1 && args[0].equals("--help")) { System.out.println(USAGE); return; }
        String ir = null, out = null, pkg = null, parserPkg = null, languageId = null;
        String scannerClass = null, serverName = null, serverVersion = null, tmLanguage = null, dapPkg = null;
        boolean emitLsp = false;
        boolean emitDap = false;
        for (int i = 0; i < args.length; i++) {
            String option = args[i];
            if (option.equals("--emit-lsp")) { emitLsp = true; continue; }
            if (option.equals("--emit-dap")) { emitDap = true; continue; }
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
                case "--dap-package" -> { if (dapPkg != null) throw new IllegalArgumentException("Duplicate --dap-package"); dapPkg = value; }
                default -> throw new IllegalArgumentException("Unknown option: " + option);
            }
        }
        if (!emitLsp && emitDap)
            throw new IllegalArgumentException("--emit-dap は --emit-lsp と一緒に使ってください"
                + "（DAP は LSP 包みの Positions / Meta / Analysis / Diagnostics を使います）");
        if (!emitLsp) throw new IllegalArgumentException("--emit-lsp が要ります（LSP は opt-in feature）");
        if (!emitDap && dapPkg != null) throw new IllegalArgumentException("--dap-package には --emit-dap が要ります");
        if (ir == null || out == null) throw new IllegalArgumentException("--ir と --out が要ります");
        var grammarIr = IrReader.read(Path.of(ir));
        var emitter = new LspEmitter(grammarIr, pkg, parserPkg, languageId, scannerClass, serverName, serverVersion);
        emitter.write(Path.of(out));
        if (emitDap) new DapEmitter(emitter, dapPkg).write(Path.of(out));
        if (tmLanguage != null) {
            Path path = Path.of(tmLanguage);
            if (path.getParent() != null) java.nio.file.Files.createDirectories(path.getParent());
            java.nio.file.Files.writeString(path,
                new TmLanguageEmitter(grammarIr, emitter.languageId(), emitter.grammar()).json(),
                java.nio.charset.StandardCharsets.UTF_8);
        }
    }
}
