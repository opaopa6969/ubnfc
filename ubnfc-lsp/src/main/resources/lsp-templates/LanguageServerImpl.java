package __PACKAGE__;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import org.eclipse.lsp4j.CompletionItem;
import org.eclipse.lsp4j.CompletionList;
import org.eclipse.lsp4j.CompletionOptions;
import org.eclipse.lsp4j.CompletionParams;
import org.eclipse.lsp4j.DefinitionParams;
import org.eclipse.lsp4j.FoldingRange;
import org.eclipse.lsp4j.FoldingRangeRequestParams;
import org.eclipse.lsp4j.RenameParams;
import org.eclipse.lsp4j.TextEdit;
import org.eclipse.lsp4j.WorkspaceEdit;
import org.eclipse.lsp4j.DidChangeConfigurationParams;
import org.eclipse.lsp4j.DidChangeTextDocumentParams;
import org.eclipse.lsp4j.DidChangeWatchedFilesParams;
import org.eclipse.lsp4j.DidCloseTextDocumentParams;
import org.eclipse.lsp4j.DidOpenTextDocumentParams;
import org.eclipse.lsp4j.DidSaveTextDocumentParams;
import org.eclipse.lsp4j.DocumentSymbol;
import org.eclipse.lsp4j.DocumentSymbolParams;
import org.eclipse.lsp4j.HoverParams;
import org.eclipse.lsp4j.InitializeParams;
import org.eclipse.lsp4j.InitializeResult;
import org.eclipse.lsp4j.Location;
import org.eclipse.lsp4j.LocationLink;
import org.eclipse.lsp4j.PublishDiagnosticsParams;
import org.eclipse.lsp4j.ReferenceParams;
import org.eclipse.lsp4j.SemanticTokensParams;
import org.eclipse.lsp4j.SemanticTokensWithRegistrationOptions;
import org.eclipse.lsp4j.ServerCapabilities;
import org.eclipse.lsp4j.ServerInfo;
import org.eclipse.lsp4j.SymbolInformation;
import org.eclipse.lsp4j.SymbolKind;
import org.eclipse.lsp4j.TextDocumentSyncKind;
import org.eclipse.lsp4j.jsonrpc.messages.Either;
import org.eclipse.lsp4j.services.LanguageClient;
import org.eclipse.lsp4j.services.LanguageClientAware;
import org.eclipse.lsp4j.services.LanguageServer;
import org.eclipse.lsp4j.services.TextDocumentService;
import org.eclipse.lsp4j.services.WorkspaceService;
import __API_PACKAGE__.TokenScanner;

/**
 * __GRAMMAR__ 用に生成された LSP サーバ（段階 5）。
 *
 * <p>提供するもの: 診断（構文・scope・前半 scope）/ definition / references / document symbols /
 * semantic tokens / completion / hover / rename / folding range。増分解析は無い（D-060）。
 *
 * <p>生成パーサは依存ゼロのまま。lsp4j に依存するのはこの薄い包み側だけ（opt-in feature）。
 *
 * <p>増分解析は無い。{@code didOpen} / {@code didChange} のたびに<b>全文を解析し直す</b>。
 * 認識モード（D-023 の二段構え）で解析し、失敗したときだけ生成パーサ自身が診断モードで
 * 決定的に再解析する。lexical（semantic tokens 用）は要求されたときだけ解析し直す。
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
    private final java.nio.file.Path validator;
    private final CatalogDiagnostics catalogs = new CatalogDiagnostics();
    private LanguageClient client;

    public LanguageServerImpl() { this(scanners(SCANNER_CLASS)); }

    public LanguageServerImpl(Map<String, TokenScanner> scanners) { this(scanners, null); }

    /** front は UBNF 編集用サーバでだけ有効。実行ファイルは起動側が指定する。 */
    public LanguageServerImpl(Map<String, TokenScanner> scanners, java.nio.file.Path validator) {
        this.scanners = Map.copyOf(scanners);
        if (validator != null && (!GRAMMAR.equals("UBNF") || !validator.isAbsolute()))
            throw new IllegalArgumentException("--validator は UBNF サーバに実行ファイルの絶対パスを指定してください");
        this.validator = validator;
    }

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

    @Override public void connect(LanguageClient client) { this.client = client; }

    @Override public CompletableFuture<InitializeResult> initialize(InitializeParams params) {
        catalogs.configure(params.getInitializationOptions());
        var capabilities = new ServerCapabilities();
        var sync = new org.eclipse.lsp4j.TextDocumentSyncOptions();
        sync.setOpenClose(true);
        sync.setChange(TextDocumentSyncKind.Full);
        sync.setSave(new org.eclipse.lsp4j.SaveOptions(false));
        capabilities.setTextDocumentSync(sync);
        capabilities.setDefinitionProvider(true);
        capabilities.setReferencesProvider(true);
        capabilities.setRenameProvider(true);
        capabilities.setFoldingRangeProvider(true);
        capabilities.setDocumentSymbolProvider(true);
        capabilities.setHoverProvider(true);
        var completion = new CompletionOptions();
        completion.setResolveProvider(false);
        capabilities.setCompletionProvider(completion);
        var semantic = new SemanticTokensWithRegistrationOptions(SemanticTokens.legend());
        semantic.setFull(true);
        semantic.setRange(false);
        capabilities.setSemanticTokensProvider(semantic);
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

    @Override public void didSave(DidSaveTextDocumentParams params) {
        String uri = params.getTextDocument().getUri();
        var analysis = documents.get(uri);
        if (analysis != null) publish(uri, diagnostics(analysis));
    }

    @Override public void didChangeConfiguration(DidChangeConfigurationParams params) {
        catalogs.configure(params.getSettings());
        refresh();
    }

    @Override public void didChangeWatchedFiles(DidChangeWatchedFilesParams params) { refresh(); }

    private void refresh() {
        for (String uri : new java.util.TreeSet<>(documents.uris())) {
            var analysis = documents.get(uri);
            if (analysis != null) publish(uri, diagnostics(analysis));
        }
    }

    // ---- capability -------------------------------------------------------

    @Override public CompletableFuture<Either<List<? extends Location>, List<? extends LocationLink>>>
            definition(DefinitionParams params) {
        var analysis = documents.get(params.getTextDocument().getUri());
        var out = new ArrayList<Location>();
        if (analysis != null) {
            var symbols = analysis.symbols();
            var symbol = symbols.at(analysis.positions().offsetOf(params.getPosition()));
            if (symbol != null)
                for (var declaration : symbols.definitions(symbol))
                    out.add(new Location(analysis.uri(),
                        analysis.positions().rangeOf(declaration.offsetCp(), declaration.endCp())));
        }
        return CompletableFuture.completedFuture(Either.forLeft(out));
    }

    @Override public CompletableFuture<List<? extends Location>> references(ReferenceParams params) {
        var analysis = documents.get(params.getTextDocument().getUri());
        var out = new ArrayList<Location>();
        if (analysis != null) {
            var symbols = analysis.symbols();
            var symbol = symbols.at(analysis.positions().offsetOf(params.getPosition()));
            if (symbol != null) {
                boolean includeDeclaration = params.getContext() != null && params.getContext().isIncludeDeclaration();
                for (var found : symbols.named(symbol.name()))
                    if (!found.declaration() || includeDeclaration)
                        out.add(new Location(analysis.uri(),
                            analysis.positions().rangeOf(found.offsetCp(), found.endCp())));
            }
        }
        return CompletableFuture.completedFuture(out);
    }

    /** definition / references と同じ文書内・綴り単位の索引を使う。部分解析では変更を返さない。 */
    @Override public CompletableFuture<WorkspaceEdit> rename(RenameParams params) {
        var analysis = documents.get(params.getTextDocument().getUri());
        var changes = new java.util.LinkedHashMap<String, List<TextEdit>>();
        if (analysis != null && analysis.primary() != null && analysis.primary().ok()
                && params.getNewName() != null && !params.getNewName().isBlank()) {
            var symbols = analysis.symbols();
            var symbol = symbols.at(analysis.positions().offsetOf(params.getPosition()));
            if (symbol != null) {
                var ranges = new java.util.TreeMap<Integer, Symbols.Symbol>();
                for (var found : symbols.named(symbol.name())) ranges.putIfAbsent(found.offsetCp(), found);
                var edits = new ArrayList<TextEdit>();
                for (var found : ranges.values())
                    edits.add(new TextEdit(analysis.positions().rangeOf(found.offsetCp(), found.endCp()), params.getNewName()));
                if (!edits.isEmpty()) changes.put(analysis.uri(), edits);
            }
        }
        return CompletableFuture.completedFuture(new WorkspaceEdit(changes));
    }

    @Override public CompletableFuture<List<FoldingRange>> foldingRange(FoldingRangeRequestParams params) {
        var analysis = documents.get(params.getTextDocument().getUri());
        return CompletableFuture.completedFuture(analysis == null ? List.of() : FoldingRanges.of(analysis));
    }

    @Override public CompletableFuture<List<Either<SymbolInformation, DocumentSymbol>>>
            documentSymbol(DocumentSymbolParams params) {
        var analysis = documents.get(params.getTextDocument().getUri());
        var out = new ArrayList<Either<SymbolInformation, DocumentSymbol>>();
        if (analysis != null)
            for (var node : analysis.symbols().documentSymbols())
                out.add(Either.forRight(symbol(analysis, node)));
        return CompletableFuture.completedFuture(out);
    }

    private static DocumentSymbol symbol(Analysis analysis, Symbols.Node node) {
        var positions = analysis.positions();
        var children = new ArrayList<DocumentSymbol>();
        for (var child : node.children()) children.add(symbol(analysis, child));
        var result = new DocumentSymbol(node.name(),
            node.container() ? SymbolKind.Namespace : SymbolKind.Variable,
            positions.rangeOf(node.startCp(), node.endCp()),
            positions.rangeOf(node.nameStartCp(), node.nameStartCp() + node.nameLengthCp()));
        result.setDetail(node.detail());
        if (!children.isEmpty()) result.setChildren(children);
        return result;
    }

    @Override public CompletableFuture<org.eclipse.lsp4j.Hover> hover(HoverParams params) {
        var analysis = documents.get(params.getTextDocument().getUri());
        return CompletableFuture.completedFuture(analysis == null ? null : Hovers.at(analysis, params.getPosition()));
    }

    @Override public CompletableFuture<Either<List<CompletionItem>, CompletionList>> completion(CompletionParams params) {
        var analysis = documents.get(params.getTextDocument().getUri());
        List<CompletionItem> items = analysis == null ? List.of()
            : Completion.at(analysis, params.getPosition(), scanners);
        return CompletableFuture.completedFuture(Either.forLeft(items));
    }

    @Override public CompletableFuture<org.eclipse.lsp4j.SemanticTokens> semanticTokensFull(SemanticTokensParams params) {
        var analysis = documents.get(params.getTextDocument().getUri());
        List<Integer> data = analysis == null ? List.of() : SemanticTokens.data(analysis);
        return CompletableFuture.completedFuture(new org.eclipse.lsp4j.SemanticTokens(data));
    }

    // ---- 解析 --------------------------------------------------------------

    /** 1 文書を全文解析して診断を返す。テストからも直接呼べるように公開している。 */
    public List<org.eclipse.lsp4j.Diagnostic> analyze(String uri, String text) {
        return diagnostics(analysis(uri, text));
    }

    /** 1 文書を解析して覚える。 */
    public Analysis analysis(String uri, String text) {
        var analysis = Analysis.of(uri, text, scanners);
        documents.put(uri, analysis);
        return analysis;
    }

    private void update(String uri, String text) {
        publish(uri, diagnostics(analysis(uri, text)));
    }

    private List<org.eclipse.lsp4j.Diagnostic> diagnostics(Analysis analysis) {
        var out = new ArrayList<>(Diagnostics.of(analysis));
        out.addAll(FrontValidation.of(analysis, validator));
        out.addAll(catalogs.of(analysis));
        return out;
    }

    private void publish(String uri, List<org.eclipse.lsp4j.Diagnostic> diagnostics) {
        if (client != null) client.publishDiagnostics(new PublishDiagnosticsParams(uri, diagnostics));
    }
}
