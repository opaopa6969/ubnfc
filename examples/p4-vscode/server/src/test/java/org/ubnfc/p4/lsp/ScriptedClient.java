package org.ubnfc.p4.lsp;

import java.io.PipedInputStream;
import java.io.PipedOutputStream;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DidOpenTextDocumentParams;
import org.eclipse.lsp4j.InitializeParams;
import org.eclipse.lsp4j.MessageActionItem;
import org.eclipse.lsp4j.MessageParams;
import org.eclipse.lsp4j.PublishDiagnosticsParams;
import org.eclipse.lsp4j.ShowMessageRequestParams;
import org.eclipse.lsp4j.TextDocumentItem;
import org.eclipse.lsp4j.launch.LSPLauncher;
import org.eclipse.lsp4j.services.LanguageClient;
import org.eclipse.lsp4j.services.LanguageServer;
import org.ubnfc.p4.P4Scanners;

/**
 * 台本どおりに動く LSP クライアント。initialize → didOpen → publishDiagnostics を JSON-RPC で往復する。
 * 実物の lsp4j launcher を両端に置くので、サーバのメソッドを直接呼ぶのとは違い protocol を通る。
 */
final class ScriptedClient implements AutoCloseable {
    private final LanguageServer proxy;
    private final Collector collector = new Collector();
    private final List<Thread> owned = new ArrayList<>();

    static final class Collector implements LanguageClient {
        private final Map<String, CompletableFuture<List<Diagnostic>>> pending = new java.util.concurrent.ConcurrentHashMap<>();

        CompletableFuture<List<Diagnostic>> expect(String uri) {
            return pending.computeIfAbsent(uri, key -> new CompletableFuture<>());
        }

        @Override public void publishDiagnostics(PublishDiagnosticsParams params) {
            expect(params.getUri()).complete(List.copyOf(params.getDiagnostics()));
        }
        @Override public void telemetryEvent(Object object) { }
        @Override public void logMessage(MessageParams message) { }
        @Override public void showMessage(MessageParams message) { }
        @Override public CompletableFuture<MessageActionItem> showMessageRequest(ShowMessageRequestParams params) {
            return CompletableFuture.completedFuture(null);
        }
    }

    ScriptedClient() throws Exception {
        var toServer = new PipedInputStream();
        var clientOut = new PipedOutputStream(toServer);
        var toClient = new PipedInputStream();
        var serverOut = new PipedOutputStream(toClient);
        var server = new LanguageServerImpl(P4Scanners.ALL);
        var serverLauncher = LSPLauncher.createServerLauncher(server, toServer, serverOut);
        server.connect(serverLauncher.getRemoteProxy());
        var clientLauncher = LSPLauncher.createClientLauncher(collector, toClient, clientOut);
        serverLauncher.startListening();
        clientLauncher.startListening();
        this.proxy = clientLauncher.getRemoteProxy();
        proxy.initialize(new InitializeParams()).get(30, TimeUnit.SECONDS);
        proxy.initialized(new org.eclipse.lsp4j.InitializedParams());
    }

    /** 文書を開いて、その URI 宛ての publishDiagnostics を 1 本待つ。 */
    List<Diagnostic> diagnosticsOf(String uri, String text) throws Exception {
        var expected = collector.expect(uri);
        proxy.getTextDocumentService().didOpen(new DidOpenTextDocumentParams(
            new TextDocumentItem(uri, LanguageServerImpl.LANGUAGE_ID, 1, text)));
        return expected.get(120, TimeUnit.SECONDS);
    }

    @Override public void close() throws Exception {
        proxy.shutdown().get(30, TimeUnit.SECONDS);
        proxy.exit();
        for (Thread thread : owned) thread.interrupt();
    }
}
