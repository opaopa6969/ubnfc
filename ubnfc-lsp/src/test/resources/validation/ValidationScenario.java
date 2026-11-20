import static org.junit.jupiter.api.Assertions.*;

import com.google.gson.GsonBuilder;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;
import org.eclipse.lsp4j.*;
import org.eclipse.lsp4j.launch.LSPLauncher;
import org.eclipse.lsp4j.services.*;

/** 生成物と一緒にコンパイルし、実 front と実 JSON-RPC で応答を固定する。 */
public final class ValidationScenario {
    public static Map<String, String> run(Path work, Path front, Path fixture) throws Exception {
        var out = new TreeMap<String, String>();
        String valid = "grammar Demo { @root Start ::= 'x' ; }";
        Path document = work.resolve("文法 😀.ubnf");
        Files.writeString(document, valid);
        String uri = document.toUri().toString();
        try (var wire = new Wire(new org.unlaxer.ubnfc.examples.ubnf.lsp.LanguageServerImpl(Map.of(), front), uri)) {
            wire.initialize(null);
            var answers = new LinkedHashMap<String, Object>();
            answers.put("normal", wire.open(valid));
            assertEquals(List.of(), answers.get("normal"));
            var warning = wire.change("grammar Demo { // 😀\n Start ::= 'x' ; }");
            assertCode(warning, "W-GENERAL-NO-ROOT");
            answers.put("warning", warning);
            var undefined = wire.change("grammar Demo { // 😀\n @root Start ::= Missing ; }");
            assertCode(undefined, "E-RULE-UNDEFINED");
            answers.put("undefined", undefined);
            var mapping = wire.change("grammar Demo { @root @mapping(Node, params=[missing]) Start ::= 'x' ; }");
            assertCode(mapping, "E-MAPPING-MISSING-CAPTURE");
            answers.put("mapping", mapping);
            var recursion = wire.change("grammar Demo { @root Start ::= Start 'x' | 'x' ; }");
            assertCode(recursion, "W-LEFT-RECURSION");
            answers.put("leftRecursion", recursion);
            assertEquals(valid, Files.readString(document), "未保存の変更を書き戻さない");
            var invalid = wire.change("grammar Demo { // 😀\n @root Start ::= 'x' ");
            assertCode(invalid, "E-SYNTAX");
            assertTrue(invalid.stream().anyMatch(d -> "ubnfc".equals(d.getSource())), "生成パーサの構文診断も返す");
            answers.put("syntax", invalid);
            answers.put("updated", wire.change(valid));
            assertEquals(List.of(), answers.get("updated"));
            answers.put("closed", wire.closeDocument());
            out.put("front", json(answers).replace(uri, "file:///validation/demo.ubnf"));
        }
        // ../ を含む import を一時 snapshot からも読める。import 先に意味エラーがあれば隠さない。
        Path sub = Files.createDirectories(work.resolve("project/sub"));
        Path library = work.resolve("project/lib.ubnf");
        Files.writeString(library, "grammar Library { Item ::= 'x' ; }");
        String imported = "grammar Demo { @import lib from '../lib.ubnf' @root Start ::= lib.Item ; }";
        try (var wire = new Wire(new org.unlaxer.ubnfc.examples.ubnf.lsp.LanguageServerImpl(Map.of(), front),
                sub.resolve("main.ubnf").toUri().toString())) {
            wire.initialize(null);
            var first = wire.open(imported);
            assertFalse(first.stream().anyMatch(d -> d.getSeverity() == DiagnosticSeverity.Error), first.toString());
            Files.writeString(library, "grammar Library { Item ::= Missing ; }");
            var changed = wire.watched(library);
            assertCode(changed, "E-RULE-UNDEFINED");
            out.put("imports", json(new TreeMap<>(Map.of("normal", first, "updated", changed))));
        }
        // pkg: の manifest / lock は元文書の隣で読む。一時ファイルの隣へ探しに行かない。
        Files.copy(fixture.resolve("ubnf.json"), sub.resolve("ubnf.json"));
        Files.copy(fixture.resolve("ubnf.lock.json"), sub.resolve("ubnf.lock.json"));
        try (var wire = new Wire(new org.unlaxer.ubnfc.examples.ubnf.lsp.LanguageServerImpl(Map.of(), front),
                sub.resolve("package.ubnf").toUri().toString())) {
            wire.initialize(null);
            var packaged = wire.open("grammar Demo { @import layout from 'pkg:std/layout' @root Start ::= 'x' ; }");
            assertTrue(packaged.isEmpty(), packaged.toString());
            out.put("package", json(packaged));
        }
        // 未指定なら front を起動しない。存在しない実行ファイルは構文診断と共存する明示診断。
        var off = new org.unlaxer.ubnfc.examples.ubnf.lsp.LanguageServerImpl(Map.of());
        assertTrue(off.analyze(uri, "grammar Demo { Start ::= Missing ; }").isEmpty());
        try (var wire = new Wire(new org.unlaxer.ubnfc.examples.ubnf.lsp.LanguageServerImpl(Map.of(), work.resolve("absent")), uri)) {
            wire.initialize(null);
            var missing = wire.open(valid);
            assertCode(missing, "front-unavailable");
            out.put("front-unavailable", json(missing));
        }
        Path script = work.resolve("broken-front");
        Files.writeString(script, "#!/bin/sh\nprintf '%s' 'not JSON'\n");
        assertTrue(script.toFile().setExecutable(true, true));
        try (var wire = new Wire(new org.unlaxer.ubnfc.examples.ubnf.lsp.LanguageServerImpl(Map.of(), script), uri)) {
            wire.initialize(null);
            var errors = new LinkedHashMap<String, Object>();
            var badJson = wire.open(valid);
            assertCode(badJson, "front-unavailable"); errors.put("badJson", badJson);
            Files.writeString(script, "#!/bin/sh\nexit 2\n");
            var failed = wire.change(valid);
            assertCode(failed, "front-failed"); errors.put("failed", failed);
            Files.writeString(script, "#!/bin/sh\nexec sleep 30\n");
            var timeout = wire.change(valid);
            assertCode(timeout, "front-timeout"); errors.put("timeout", timeout);
            out.put("front-errors", json(errors));
        }
        // 生成サーバの型を変更せず、@catalog の公開 capture 出現表を照合する。
        Path catalog = work.resolve("names.tecatalog");
        Files.copy(fixture.resolve("names.tecatalog"), catalog);
        var local = new org.unlaxer.ubnfc.examples.catalog.lsp.LanguageServerImpl(Map.of());
        assertThrows(IllegalArgumentException.class,
            () -> new org.unlaxer.ubnfc.examples.catalog.lsp.LanguageServerImpl(Map.of(), front));
        assertTrue(local.analyze(uri, "unknown;").isEmpty(), "カタログ無指定では照合しない");
        try (var wire = new Wire(local, uri)) {
            var filtered = Map.of("catalogs", Map.of("names", Map.of("path", catalog.toString(), "context", "names")));
            wire.initialize(filtered);
            var answers = new LinkedHashMap<String, Object>();
            answers.put("normal", wire.open("alpha;"));
            assertEquals(List.of(), answers.get("normal"));
            var prefix = wire.change("item_one; item_; item;");
            assertEquals(2, prefix.size(), "prefixWithSuffix は区切りと接尾辞長も照合する");
            answers.put("prefix", prefix);
            wire.change("other;");
            assertEquals(List.of(), wire.configure(Map.of("catalogs", Map.of("names", catalog.toString()))),
                "ファイルだけの指定なら行 context で除外しない");
            wire.configure(filtered);
            String unknownText = "/* 😀 */ unknown; other;";
            var unknown = wire.change(unknownText);
            assertCode(unknown, "catalog-unknown");
            assertEquals(2, unknown.size());
            assertEquals(9, unknown.get(0).getRange().getStart().getCharacter(), "BMP 外文字後も UTF-16");
            answers.put("unknown", unknown);
            Files.writeString(catalog, "tinyexpression-catalog-v1\nexact|unknown|更新 😀|names\nexact|other|同 context に更新|names\n");
            answers.put("updated", wire.watched(catalog));
            assertEquals(List.of(), answers.get("updated"));
            Files.copy(fixture.resolve("invalid.tecatalog"), catalog, StandardCopyOption.REPLACE_EXISTING);
            var broken = wire.save();
            assertCode(broken, "catalog-invalid");
            assertEquals(1, broken.size(), "不正ファイルを空カタログとして誤判定しない");
            answers.put("invalid", broken);
            Files.delete(catalog);
            var missing = wire.watched(catalog);
            assertCode(missing, "catalog-missing");
            answers.put("missing", missing);
            answers.put("disabled", wire.configure(Map.of()));
            assertEquals(List.of(), answers.get("disabled"));
            var badConfig = wire.configure(Map.of("catalogs", Map.of("names", "relative.tecatalog")));
            answers.put("badConfig", badConfig);
            assertCode(badConfig, "catalog-config");
            wire.configure(Map.of());
            Files.copy(fixture.resolve("names.tecatalog"), catalog);
            wire.configure(filtered);
            var syntax = wire.change("/* 😀 */ unknown; ?");
            assertTrue(syntax.stream().anyMatch(d -> "ubnfc".equals(d.getSource())));
            answers.put("syntax", syntax);
            answers.put("repaired", wire.change("alpha;"));
            assertEquals(List.of(), answers.get("repaired"));
            answers.put("closed", wire.closeDocument());
            out.put("catalog", json(answers).replace(uri, "file:///validation/demo.ubnf"));
        }
        return out;
    }

    private static void assertCode(List<Diagnostic> diagnostics, String code) {
        assertTrue(diagnostics.stream().anyMatch(d -> d.getCode() != null && code.equals(d.getCode().getLeft())), diagnostics.toString());
    }
    private static String json(Object value) { return new GsonBuilder().setPrettyPrinting().create().toJson(value) + "\n"; }

    private static final class Wire implements AutoCloseable, LanguageClient {
        final LanguageServer lsp;
        final String uri;
        int version;
        final PipedInputStream serverIn = new PipedInputStream(), clientIn = new PipedInputStream();
        final PipedOutputStream clientOut = new PipedOutputStream(serverIn), serverOut = new PipedOutputStream(clientIn);
        final ExecutorService executor = Executors.newCachedThreadPool();
        final Future<?> serverListening, clientListening;
        volatile CompletableFuture<List<Diagnostic>> diagnostic = new CompletableFuture<>();
        Wire(LanguageServer local, String uri) throws Exception {
            this.uri = uri;
            var s = LSPLauncher.createServerLauncher(local, serverIn, serverOut, executor, consumer -> consumer);
            ((LanguageClientAware) local).connect(s.getRemoteProxy());
            var c = LSPLauncher.createClientLauncher(this, clientIn, clientOut, executor, consumer -> consumer);
            serverListening = s.startListening(); clientListening = c.startListening(); lsp = c.getRemoteProxy();
        }
        void initialize(Object options) throws Exception {
            var params = new InitializeParams(); params.setInitializationOptions(options);
            var sync = lsp.initialize(params).get(30, TimeUnit.SECONDS).getCapabilities().getTextDocumentSync().getRight();
            assertEquals(TextDocumentSyncKind.Full, sync.getChange());
            assertTrue(sync.getOpenClose());
            assertNotNull(sync.getSave(), "保存による再検査をクライアントへ通知する");
        }
        List<Diagnostic> await(Runnable notify) throws Exception {
            diagnostic = new CompletableFuture<>(); notify.run(); return diagnostic.get(30, TimeUnit.SECONDS);
        }
        List<Diagnostic> open(String text) throws Exception {
            return await(() -> lsp.getTextDocumentService().didOpen(new DidOpenTextDocumentParams(new TextDocumentItem(uri, "demo", ++version, text))));
        }
        List<Diagnostic> change(String text) throws Exception {
            return await(() -> lsp.getTextDocumentService().didChange(new DidChangeTextDocumentParams(
                new VersionedTextDocumentIdentifier(uri, ++version), List.of(new TextDocumentContentChangeEvent(text)))));
        }
        List<Diagnostic> closeDocument() throws Exception {
            return await(() -> lsp.getTextDocumentService().didClose(new DidCloseTextDocumentParams(new TextDocumentIdentifier(uri))));
        }
        List<Diagnostic> save() throws Exception {
            return await(() -> lsp.getTextDocumentService().didSave(new DidSaveTextDocumentParams(new TextDocumentIdentifier(uri))));
        }
        List<Diagnostic> watched(Path path) throws Exception {
            return await(() -> lsp.getWorkspaceService().didChangeWatchedFiles(new DidChangeWatchedFilesParams(
                List.of(new FileEvent(path.toUri().toString(), FileChangeType.Changed)))));
        }
        List<Diagnostic> configure(Object options) throws Exception {
            return await(() -> lsp.getWorkspaceService().didChangeConfiguration(new DidChangeConfigurationParams(options)));
        }
        public void publishDiagnostics(PublishDiagnosticsParams params) { diagnostic.complete(params.getDiagnostics()); }
        public void telemetryEvent(Object value) {}
        public void logMessage(MessageParams params) {}
        public void showMessage(MessageParams params) {}
        public CompletableFuture<MessageActionItem> showMessageRequest(ShowMessageRequestParams params) { return CompletableFuture.completedFuture(null); }
        public void close() throws Exception {
            serverListening.cancel(true); clientListening.cancel(true);
            clientOut.close(); serverOut.close(); serverIn.close(); clientIn.close(); executor.shutdownNow();
        }
    }
}
