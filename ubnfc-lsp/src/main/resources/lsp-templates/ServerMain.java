package __PACKAGE__;

import java.io.InputStream;
import java.io.OutputStream;
import org.eclipse.lsp4j.launch.LSPLauncher;

/** stdio で話す LSP サーバの入口。VS Code 拡張は {@code java -jar <このjar>} で起動する。 */
public final class ServerMain {
    private ServerMain() {}

    public static void main(String[] args) throws Exception {
        String scannerClass = LanguageServerImpl.SCANNER_CLASS;
        for (int i = 0; i < args.length; i++) {
            if (!args[i].equals("--scanners")) throw new IllegalArgumentException("不明な引数: " + args[i]);
            if (++i == args.length) throw new IllegalArgumentException("--scanners <クラス名> が必要です");
            scannerClass = args[i];
        }
        InputStream in = System.in;
        OutputStream out = System.out;
        // stdout は JSON-RPC 専用。生成物の println が混ざらないよう stderr へ寄せる。
        System.setOut(new java.io.PrintStream(new java.io.FileOutputStream(java.io.FileDescriptor.err), true,
            java.nio.charset.StandardCharsets.UTF_8));
        var server = new LanguageServerImpl(LanguageServerImpl.scanners(scannerClass));
        var launcher = LSPLauncher.createServerLauncher(server, in, out);
        server.connect(launcher.getRemoteProxy());
        launcher.startListening().get();
    }
}
