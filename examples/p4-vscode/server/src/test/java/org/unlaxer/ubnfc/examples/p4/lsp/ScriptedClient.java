package org.unlaxer.ubnfc.examples.p4.lsp;

import java.io.PipedInputStream;
import java.io.PipedOutputStream;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import org.eclipse.lsp4j.CompletionItem;
import org.eclipse.lsp4j.CompletionParams;
import org.eclipse.lsp4j.DefinitionParams;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DidOpenTextDocumentParams;
import org.eclipse.lsp4j.DocumentSymbol;
import org.eclipse.lsp4j.DocumentSymbolParams;
import org.eclipse.lsp4j.Hover;
import org.eclipse.lsp4j.HoverParams;
import org.eclipse.lsp4j.InitializeParams;
import org.eclipse.lsp4j.InitializeResult;
import org.eclipse.lsp4j.Location;
import org.eclipse.lsp4j.MessageActionItem;
import org.eclipse.lsp4j.MessageParams;
import org.eclipse.lsp4j.Position;
import org.eclipse.lsp4j.PublishDiagnosticsParams;
import org.eclipse.lsp4j.ReferenceContext;
import org.eclipse.lsp4j.ReferenceParams;
import org.eclipse.lsp4j.SemanticTokens;
import org.eclipse.lsp4j.SemanticTokensParams;
import org.eclipse.lsp4j.ShowMessageRequestParams;
import org.eclipse.lsp4j.SymbolInformation;
import org.eclipse.lsp4j.TextDocumentIdentifier;
import org.eclipse.lsp4j.TextDocumentItem;
import org.eclipse.lsp4j.jsonrpc.messages.Either;
import org.eclipse.lsp4j.launch.LSPLauncher;
import org.eclipse.lsp4j.services.LanguageClient;
import org.eclipse.lsp4j.services.LanguageServer;
import org.unlaxer.ubnfc.examples.p4.P4Scanners;

/**
 * 台本どおりに動く LSP クライアント。initialize → didOpen → 各 capability の要求を JSON-RPC で往復する。
 * 実物の lsp4j launcher を両端に置くので、サーバのメソッドを直接呼ぶのとは違い protocol を通る。
 */
final class ScriptedClient implements AutoCloseable {
    private static final long TIMEOUT = 120;
    private final LanguageServer proxy;
    private final Collector collector = new Collector();
    private final InitializeResult initialized;

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
        this.initialized = proxy.initialize(new InitializeParams()).get(30, TimeUnit.SECONDS);
        proxy.initialized(new org.eclipse.lsp4j.InitializedParams());
    }

    InitializeResult initialized() { return initialized; }

    /** 文書を開いて、その URI 宛ての publishDiagnostics を 1 本待つ。 */
    List<Diagnostic> diagnosticsOf(String uri, String text) throws Exception {
        var expected = collector.expect(uri);
        proxy.getTextDocumentService().didOpen(new DidOpenTextDocumentParams(
            new TextDocumentItem(uri, LanguageServerImpl.LANGUAGE_ID, 1, text)));
        return expected.get(TIMEOUT, TimeUnit.SECONDS);
    }

    private static TextDocumentIdentifier id(String uri) { return new TextDocumentIdentifier(uri); }

    List<? extends Location> definition(String uri, int line, int character) throws Exception {
        var result = proxy.getTextDocumentService()
            .definition(new DefinitionParams(id(uri), new Position(line, character))).get(TIMEOUT, TimeUnit.SECONDS);
        return result == null || result.getLeft() == null ? List.of() : result.getLeft();
    }

    List<? extends Location> references(String uri, int line, int character, boolean includeDeclaration) throws Exception {
        var params = new ReferenceParams(id(uri), new Position(line, character), new ReferenceContext(includeDeclaration));
        var result = proxy.getTextDocumentService().references(params).get(TIMEOUT, TimeUnit.SECONDS);
        return result == null ? List.of() : result;
    }

    List<Either<SymbolInformation, DocumentSymbol>> documentSymbols(String uri) throws Exception {
        var result = proxy.getTextDocumentService()
            .documentSymbol(new DocumentSymbolParams(id(uri))).get(TIMEOUT, TimeUnit.SECONDS);
        return result == null ? List.of() : result;
    }

    Hover hover(String uri, int line, int character) throws Exception {
        return proxy.getTextDocumentService()
            .hover(new HoverParams(id(uri), new Position(line, character))).get(TIMEOUT, TimeUnit.SECONDS);
    }

    List<CompletionItem> completion(String uri, int line, int character) throws Exception {
        var params = new CompletionParams(id(uri), new Position(line, character));
        var result = proxy.getTextDocumentService().completion(params).get(TIMEOUT, TimeUnit.SECONDS);
        if (result == null) return List.of();
        return result.isLeft() ? result.getLeft() : result.getRight().getItems();
    }

    SemanticTokens semanticTokens(String uri) throws Exception {
        return proxy.getTextDocumentService()
            .semanticTokensFull(new SemanticTokensParams(id(uri))).get(TIMEOUT, TimeUnit.SECONDS);
    }

    @Override public void close() throws Exception {
        proxy.shutdown().get(30, TimeUnit.SECONDS);
        proxy.exit();
    }
}
