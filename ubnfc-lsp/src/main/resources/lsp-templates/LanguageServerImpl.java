package __PACKAGE__;

import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import org.eclipse.lsp4j.DidChangeConfigurationParams;
import org.eclipse.lsp4j.DidChangeTextDocumentParams;
import org.eclipse.lsp4j.DidChangeWatchedFilesParams;
import org.eclipse.lsp4j.DidCloseTextDocumentParams;
import org.eclipse.lsp4j.DidOpenTextDocumentParams;
import org.eclipse.lsp4j.DidSaveTextDocumentParams;
import org.eclipse.lsp4j.InitializeParams;
import org.eclipse.lsp4j.InitializeResult;
import org.eclipse.lsp4j.PublishDiagnosticsParams;
import org.eclipse.lsp4j.ServerCapabilities;
import org.eclipse.lsp4j.ServerInfo;
import org.eclipse.lsp4j.TextDocumentSyncKind;
import org.eclipse.lsp4j.services.LanguageClient;
import org.eclipse.lsp4j.services.LanguageClientAware;
import org.eclipse.lsp4j.services.LanguageServer;
import org.eclipse.lsp4j.services.TextDocumentService;
import org.eclipse.lsp4j.services.WorkspaceService;
import __API_PACKAGE__.ParseOptions;
import __API_PACKAGE__.ParseResult;
import __API_PACKAGE__.TokenScanner;

/**
 * __GRAMMAR__ 用に生成された LSP サーバ（段階 2: 診断のみ）。
 *
 * <p>生成パーサは依存ゼロのまま。lsp4j に依存するのはこの薄い包み側だけ
 * （D-060、opt-in feature）。
 *
 * <p>増分解析は無い。{@code didOpen} / {@code didChange} のたびに<b>全文を解析し直す</b>。
 * 認識モード（D-023 の二段構え）で解析し、失敗したときだけ生成パーサ自身が診断モードで
 * 決定的に再解析する。成功する文書では診断観測を一切作らないので、打鍵ごとの費用は
 * 認識だけの費用になる。
 */
public final class LanguageServerImpl implements LanguageServer, LanguageClientAware,
        TextDocumentService, WorkspaceService {
    /** 文法が要求する extern token scanner を持つクラス（{@code public static Map<String,TokenScanner> ALL}）。 */
    public static final String SCANNER_CLASS = "__SCANNER_CLASS__";
    public static final String LANGUAGE_ID = "__LANGUAGE_ID__";
    public static final String GRAMMAR = "__GRAMMAR__";
    public static final String ENTRY = "__ENTRY__";

    private final DocumentStore documents = new DocumentStore();
    private final Map<String, TokenScanner> scanners;
    private LanguageClient client;

    public LanguageServerImpl() { this(scanners(SCANNER_CLASS)); }

    public LanguageServerImpl(Map<String, TokenScanner> scanners) { this.scanners = Map.copyOf(scanners); }

    /** scanner クラスを反射で読む。生成パーサ側の型でしか表現できないので名前で結ぶ。 */
    @SuppressWarnings("unchecked")
    public static Map<String, TokenScanner> scanners(String className) {
        if (className == null || className.isBlank()) return Map.of();
        try {
            Object value = Class.forName(className).getField("ALL").get(null);
            return value instanceof Map<?, ?> map ? (Map<String, TokenScanner>) map : Map.of();
        } catch (ReflectiveOperationException | RuntimeException error) {
            throw new IllegalStateException("scanner クラスを読めません: " + className, error);
        }
    }

    /** 診断を要求しない認識モード。失敗したら生成パーサが診断モードで解析し直す（D-023）。 */
    public ParseOptions options() {
        return new ParseOptions(false, true, false, true, scanners, ParseOptions.DEFAULT_MAX_DEPTH, false, false);
    }

    @Override public void connect(LanguageClient client) { this.client = client; }

    @Override public CompletableFuture<InitializeResult> initialize(InitializeParams params) {
        var capabilities = new ServerCapabilities();
        capabilities.setTextDocumentSync(TextDocumentSyncKind.Full);
        var result = new InitializeResult(capabilities);
        result.setServerInfo(new ServerInfo("__SERVER_NAME__", "__SERVER_VERSION__"));
        return CompletableFuture.completedFuture(result);
    }

    @Override public CompletableFuture<Object> shutdown() { return CompletableFuture.completedFuture(null); }

    @Override public void exit() { }

    @Override public TextDocumentService getTextDocumentService() { return this; }

    @Override public WorkspaceService getWorkspaceService() { return this; }

    @Override public void didOpen(DidOpenTextDocumentParams params) {
        var document = params.getTextDocument();
        update(document.getUri(), document.getText());
    }

    @Override public void didChange(DidChangeTextDocumentParams params) {
        var changes = params.getContentChanges();
        if (changes.isEmpty()) return;
        // full sync: 最後の変更が文書全体。
        update(params.getTextDocument().getUri(), changes.get(changes.size() - 1).getText());
    }

    @Override public void didClose(DidCloseTextDocumentParams params) {
        String uri = params.getTextDocument().getUri();
        documents.remove(uri);
        publish(uri, List.of());
    }

    @Override public void didSave(DidSaveTextDocumentParams params) { }

    @Override public void didChangeConfiguration(DidChangeConfigurationParams params) { }

    @Override public void didChangeWatchedFiles(DidChangeWatchedFilesParams params) { }

    /** 1 文書を全文解析して診断を publish する。テストからも直接呼べるように公開している。 */
    public List<org.eclipse.lsp4j.Diagnostic> analyze(String uri, String text) {
        ParseResult<?> result;
        try {
            result = __PARSER_CLASS__.parseEntry(GRAMMAR, ENTRY, text, options());
        } catch (RuntimeException error) {
            var positions = new Positions(text);
            var diagnostic = new org.eclipse.lsp4j.Diagnostic(positions.rangeOf(0, 0),
                "解析器の内部エラー: " + error);
            diagnostic.setSeverity(org.eclipse.lsp4j.DiagnosticSeverity.Error);
            diagnostic.setSource(Diagnostics.SOURCE);
            return List.of(diagnostic);
        }
        return Diagnostics.of(uri, text, result);
    }

    private void update(String uri, String text) {
        documents.put(uri, text);
        publish(uri, analyze(uri, text));
    }

    private void publish(String uri, List<org.eclipse.lsp4j.Diagnostic> diagnostics) {
        if (client != null) client.publishDiagnostics(new PublishDiagnosticsParams(uri, diagnostics));
    }
}
