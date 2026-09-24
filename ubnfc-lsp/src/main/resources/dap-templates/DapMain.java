package __PACKAGE__;

import java.io.InputStream;
import java.io.OutputStream;
import org.eclipse.lsp4j.debug.launch.DSPLauncher;
import __LSP_PACKAGE__.LanguageServerImpl;

/**
 * stdio で話す DAP サーバ（文法デバッガ）の入口。
 *
 * <p>LSP サーバと<b>同じ fat jar</b> に入る。VS Code 拡張は LSP を {@code java -jar <jar>}、
 * DAP を {@code java -cp <jar> __PACKAGE__.DapMain} で起動する
 * （旧スタックの {@code tinycalc-vscode} / {@code tinyexpression-p4-lsp-vscode} と同じ形）。
 *
 * <pre>
 *   --scanners &lt;fqcn&gt;    extern token scanner を持つクラス（既定は生成時の値）
 *   --evaluator &lt;fqcn&gt;   {@link EvaluatorBridge} の実装（既定は NOOP＝文法デバッガのまま）
 * </pre>
 */
public final class DapMain {
    private DapMain() {}

    public static void main(String[] args) throws Exception {
        String scannerClass = LanguageServerImpl.SCANNER_CLASS;
        String evaluatorClass = System.getProperty("ubnfc.dap.evaluator", "");
        for (int i = 0; i < args.length; i++) {
            switch (args[i]) {
                case "--scanners" -> {
                    if (++i == args.length) throw new IllegalArgumentException("--scanners <クラス名> が必要です");
                    scannerClass = args[i];
                }
                case "--evaluator" -> {
                    if (++i == args.length) throw new IllegalArgumentException("--evaluator <クラス名> が必要です");
                    evaluatorClass = args[i];
                }
                default -> throw new IllegalArgumentException("不明な引数: " + args[i]);
            }
        }
        InputStream in = System.in;
        OutputStream out = System.out;
        // stdout は DAP 専用。生成物の println が混ざらないよう stderr へ寄せる。
        System.setOut(new java.io.PrintStream(new java.io.FileOutputStream(java.io.FileDescriptor.err), true,
            java.nio.charset.StandardCharsets.UTF_8));
        var adapter = new DebugAdapter(LanguageServerImpl.scanners(scannerClass), EvaluatorBridge.load(evaluatorClass));
        var launcher = DSPLauncher.createServerLauncher(adapter, in, out);
        adapter.connect(launcher.getRemoteProxy());
        launcher.startListening();
        // 旧スタックは disconnect の中で System.exit(0) していた。ここでは adapter に知らせてもらい、
        // プロセスを畳むのはこの入口だけの責任にする（生成物を in-process のテストから叩けるように）。
        adapter.exited().join();
        System.exit(0);
    }
}
