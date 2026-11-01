import static org.junit.jupiter.api.Assertions.*;

import com.google.gson.GsonBuilder;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;
import org.eclipse.lsp4j.*;
import org.eclipse.lsp4j.debug.*;
import org.eclipse.lsp4j.debug.launch.DSPLauncher;
import org.eclipse.lsp4j.debug.services.*;
import org.eclipse.lsp4j.launch.LSPLauncher;
import org.eclipse.lsp4j.services.*;
import org.unlaxer.ubnfc.examples.stage5.SumEvaluatorBridge;
import org.unlaxer.ubnfc.examples.stage5.lsp.LanguageServerImpl;
import org.unlaxer.ubnfc.examples.stage5.lsp.dap.*;

/** テスト時に生成サーバと一緒にコンパイルする台本。通信も実物の lsp4j を通す。 */
public final class Stage5Scenario {
    private static final String URI = "file:///stage5.demo";
    private static final String TEXT = "/* 😀\r\n説明 😀 */ let alpha = 2;\r\nuse alpha;\r\n{\r\n let beta = 3;\r\n {\r\n  use alpha;\r\n }\r\n}\r\nnote \"/* 偽\r\nコメント */\";\r\n// /* 行コメント内\r\n";

    public static Map<String, String> run(Path work) throws Exception {
        var out = new TreeMap<String, String>();
        try (var wire = new Wire(new LanguageServerImpl(Map.of()))) {
            var server = wire.lsp.getTextDocumentService();
            var caps = wire.lsp.initialize(new InitializeParams()).get(30, TimeUnit.SECONDS).getCapabilities();
            assertTrue(caps.getRenameProvider().getLeft());
            assertTrue(caps.getFoldingRangeProvider().getLeft());
            // 直接解析で fixture の成立を検証し、RPC の didOpen 後の要求は診断通知を待って送る。
            assertTrue(((LanguageServerImpl) wire.local).analysis(URI, TEXT).primary().ok());
            wire.open(TEXT);
            var rename = new LinkedHashMap<String, Object>();
            rename.put("reference", server.rename(rename(2, 5, "renamed")).get(30, TimeUnit.SECONDS));
            rename.put("declaration", server.rename(rename(1, 15, "renamed")).get(30, TimeUnit.SECONDS));
            rename.put("unknown", server.rename(rename(3, 0, "renamed")).get(30, TimeUnit.SECONDS));
            rename.put("blank", server.rename(rename(2, 5, " ")).get(30, TimeUnit.SECONDS));
            var refs = server.references(new ReferenceParams(id(), new Position(2, 5), new ReferenceContext(true))).get(30, TimeUnit.SECONDS);
            assertEquals(refs.size(), ((WorkspaceEdit) rename.get("reference")).getChanges().get(URI).size());
            assertEquals(rename.get("reference"), rename.get("declaration"));
            var folding = server.foldingRange(new FoldingRangeRequestParams(id())).get(30, TimeUnit.SECONDS);
            assertEquals(4, folding.size(), "scope 3 件と実コメント 1 件だけ");
            assertEquals(1, folding.stream().filter(f -> "comment".equals(f.getKind())).count());
            out.put("folding", json(folding));
            // 壊れた文書では部分 rename を返さない。folding は成功した前半だけを使う。
            wire.closeDocument();
            wire.open(TEXT + "let broken =");
            rename.put("invalid", server.rename(rename(2, 5, "renamed")).get(30, TimeUnit.SECONDS));
            assertTrue(((WorkspaceEdit) rename.get("invalid")).getChanges().isEmpty());
            rename.put("prefixFolding", server.foldingRange(new FoldingRangeRequestParams(id())).get(30, TimeUnit.SECONDS));
            wire.closeDocument();
            rename.put("closed", server.rename(rename(2, 5, "renamed")).get(30, TimeUnit.SECONDS));
            rename.put("closedFolding", server.foldingRange(new FoldingRangeRequestParams(id())).get(30, TimeUnit.SECONDS));
            out.put("rename-scope", json(rename));
            wire.open("let a = 1; use a; { note \"/* 偽 */\"; } /* 単一行 */");
            assertTrue(server.foldingRange(new FoldingRangeRequestParams(id())).get(30, TimeUnit.SECONDS).isEmpty());
            wire.closeDocument();
            wire.open("/* 未完了\r\nコメント");
            assertTrue(server.foldingRange(new FoldingRangeRequestParams(id())).get(30, TimeUnit.SECONDS).isEmpty());
        }
        // scope 宣言を持たない sameRuleText 文法でも Symbols の lexical fallback から出る。
        try (var wire = new Wire(new org.unlaxer.ubnfc.examples.ubnf.lsp.LanguageServerImpl(Map.of()))) {
            wire.lsp.initialize(new InitializeParams()).get(30, TimeUnit.SECONDS);
            wire.open("grammar Demo {\n @root\n Expr ::= Word Word ;\n /* 😀 */ Word ::= 'x' ;\n}\n");
            var edit = wire.lsp.getTextDocumentService().rename(rename(2, 11, "Term")).get(30, TimeUnit.SECONDS);
            assertEquals(3, edit.getChanges().get(URI).size());
            out.put("rename-lexical", json(edit));
        }
        out.put("evaluator", evaluator(work));
        return out;
    }

    private static String evaluator(Path work) throws Exception {
        Path program = work.resolve("program.demo");
        Files.writeString(program, TEXT);
        // --evaluator と同じロード経路。生成物を継承するクラスは無い。
        var bridge = EvaluatorBridge.load(SumEvaluatorBridge.class.getName());
        var adapter = new DebugAdapter(Map.of(), bridge);
        try (var wire = new DapWire(adapter)) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertFalse(caps.getSupportsStepBack());
            dap.launch(Map.of("program", program.toString(), "stopOnEntry", true)).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var answer = new LinkedHashMap<String, Object>();
            var eval = new EvaluateArguments();
            eval.setExpression("total"); eval.setContext("repl");
            answer.put("evaluate", dap.evaluate(eval).get(30, TimeUnit.SECONDS));
            assertEquals("5", dap.evaluate(eval).get(30, TimeUnit.SECONDS).getResult());
            var scopes = new ScopesArguments(); scopes.setFrameId(1);
            var variables = new VariablesArguments();
            variables.setVariablesReference(dap.scopes(scopes).get(30, TimeUnit.SECONDS).getScopes()[0].getVariablesReference());
            answer.put("variables", Arrays.stream(dap.variables(variables).get(30, TimeUnit.SECONDS).getVariables())
                .filter(value -> "total".equals(value.getName())).toList());
            eval.setExpression("unknown");
            answer.put("unknownExpression", rejected(() -> dap.evaluate(eval).get(30, TimeUnit.SECONDS)));
            // 再 launch で集計をリセットする。NOOP の場合は従来どおり明示エラー。
            Files.writeString(program, "let single = 7;");
            dap.launch(Map.of("program", program.toString())).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            eval.setExpression("total");
            answer.put("relaunch", dap.evaluate(eval).get(30, TimeUnit.SECONDS));
            dap.disconnect(new DisconnectArguments()).get(30, TimeUnit.SECONDS);
            var noop = new DebugAdapter(Map.of());
            answer.put("noop", rejected(() -> noop.evaluate(eval).get(30, TimeUnit.SECONDS)));
            return json(answer);
        }
    }

    private static String rejected(Callable<?> action) throws Exception {
        try { action.call(); fail("未対応の式が成功した"); return ""; }
        catch (ExecutionException error) {
            Throwable cause = error.getCause();
            if (cause instanceof org.eclipse.lsp4j.jsonrpc.ResponseErrorException rpc)
                return rpc.getResponseError().getMessage();
            throw error;
        }
    }
    private static String json(Object value) { return new GsonBuilder().setPrettyPrinting().create().toJson(value) + "\n"; }
    private static TextDocumentIdentifier id() { return new TextDocumentIdentifier(URI); }
    private static RenameParams rename(int line, int column, String name) {
        return new RenameParams(id(), new Position(line, column), name);
    }

    /** パイプと専用 executor は最後に閉じ、テストが通信スレッドを残さないようにする。 */
    private static final class Wire implements AutoCloseable, LanguageClient {
        final LanguageServer local, lsp;
        final PipedInputStream serverIn = new PipedInputStream(), clientIn = new PipedInputStream();
        final PipedOutputStream clientOut = new PipedOutputStream(serverIn), serverOut = new PipedOutputStream(clientIn);
        final ExecutorService executor = Executors.newCachedThreadPool();
        final Future<?> serverListening, clientListening;
        volatile CompletableFuture<Void> diagnostic = new CompletableFuture<>();
        Wire(LanguageServer local) throws Exception {
            this.local = local;
            var s = LSPLauncher.createServerLauncher(local, serverIn, serverOut, executor, consumer -> consumer);
            ((LanguageClientAware) local).connect(s.getRemoteProxy());
            var c = LSPLauncher.createClientLauncher(this, clientIn, clientOut, executor, consumer -> consumer);
            serverListening = s.startListening(); clientListening = c.startListening(); lsp = c.getRemoteProxy();
        }
        void open(String text) throws Exception {
            diagnostic = new CompletableFuture<>();
            lsp.getTextDocumentService().didOpen(new DidOpenTextDocumentParams(new TextDocumentItem(URI, "demo", 1, text)));
            diagnostic.get(30, TimeUnit.SECONDS);
        }
        void closeDocument() throws Exception {
            diagnostic = new CompletableFuture<>();
            lsp.getTextDocumentService().didClose(new DidCloseTextDocumentParams(id()));
            diagnostic.get(30, TimeUnit.SECONDS);
        }
        public void publishDiagnostics(PublishDiagnosticsParams params) { diagnostic.complete(null); }
        public void telemetryEvent(Object value) {}
        public void logMessage(MessageParams params) {}
        public void showMessage(MessageParams params) {}
        public CompletableFuture<MessageActionItem> showMessageRequest(ShowMessageRequestParams params) {
            return CompletableFuture.completedFuture(null);
        }
        public void close() throws Exception {
            serverListening.cancel(true); clientListening.cancel(true);
            clientOut.close(); serverOut.close(); serverIn.close(); clientIn.close(); executor.shutdownNow();
        }
    }
    private static final class DapWire implements AutoCloseable {
        final PipedInputStream serverIn = new PipedInputStream(), clientIn = new PipedInputStream();
        final PipedOutputStream clientOut = new PipedOutputStream(serverIn), serverOut = new PipedOutputStream(clientIn);
        final ExecutorService executor = Executors.newCachedThreadPool();
        final Future<?> serverListening, clientListening;
        final IDebugProtocolServer proxy;
        DapWire(DebugAdapter adapter) throws Exception {
            var s = DSPLauncher.createServerLauncher(adapter, serverIn, serverOut, executor, consumer -> consumer);
            adapter.connect(s.getRemoteProxy());
            var c = DSPLauncher.createClientLauncher(new IDebugProtocolClient() {}, clientIn, clientOut, executor, consumer -> consumer);
            serverListening = s.startListening(); clientListening = c.startListening(); proxy = c.getRemoteProxy();
        }
        public void close() throws Exception {
            serverListening.cancel(true); clientListening.cancel(true);
            clientOut.close(); serverOut.close(); serverIn.close(); clientIn.close(); executor.shutdownNow();
        }
    }
}
