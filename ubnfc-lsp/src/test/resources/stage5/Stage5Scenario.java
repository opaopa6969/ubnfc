import static org.junit.jupiter.api.Assertions.*;

import com.google.gson.GsonBuilder;
import java.io.*;
import java.math.BigDecimal;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;
import org.eclipse.lsp4j.*;
import org.eclipse.lsp4j.debug.*;
import org.eclipse.lsp4j.debug.launch.DSPLauncher;
import org.eclipse.lsp4j.debug.services.*;
import org.eclipse.lsp4j.jsonrpc.json.MessageJsonHandler;
import org.eclipse.lsp4j.launch.LSPLauncher;
import org.eclipse.lsp4j.services.*;
import org.unlaxer.ubnfc.examples.stage5.SumEvaluatorBridge;
import org.unlaxer.ubnfc.examples.stage5.generated.Stage5AST;
import org.unlaxer.ubnfc.examples.stage5.generated.Stage5Parser;
import org.unlaxer.ubnfc.examples.stage5.generated.api.ParseOptions;
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
            assertTrue(caps.getInlayHintProvider().getLeft());
            assertNull(caps.getSignatureHelpProvider());
            assertNull(server.signatureHelp(new SignatureHelpParams(id(), new Position(0, 0)))
                .get(30, TimeUnit.SECONDS));
            assertNull(caps.getDocumentFormattingProvider());
            assertNull(caps.getCodeActionProvider());
            // 直接解析で fixture の成立を検証し、RPC の didOpen 後の要求は診断通知を待って送る。
            assertTrue(((LanguageServerImpl) wire.local).analysis(URI, TEXT).primary().ok());
            wire.open(TEXT);
            assertTrue(format(wire).isEmpty());
            var hints = new LinkedHashMap<String, Object>();
            hints.put("provider", caps.getInlayHintProvider().getLeft());
            var complete = server.inlayHint(hints(0, 0, 12, 0)).get(30, TimeUnit.SECONDS);
            assertEquals(List.of("name:", "value:", "name:", "name:", "value:", "name:", "text:"),
                complete.stream().map(hint -> hint.getLabel().getLeft()).toList());
            hints.put("complete", complete);
            var utf16 = server.inlayHint(hints(1, 13, 1, 21)).get(30, TimeUnit.SECONDS);
            assertEquals(List.of(new Position(1, 13)), utf16.stream().map(InlayHint::getPosition).toList());
            hints.put("utf16Range", utf16);
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
            var incomplete = server.inlayHint(hints(0, 0, 12, 12)).get(30, TimeUnit.SECONDS);
            assertTrue(incomplete.isEmpty());
            hints.put("incomplete", incomplete);
            rename.put("invalid", server.rename(rename(2, 5, "renamed")).get(30, TimeUnit.SECONDS));
            assertTrue(((WorkspaceEdit) rename.get("invalid")).getChanges().isEmpty());
            rename.put("prefixFolding", server.foldingRange(new FoldingRangeRequestParams(id())).get(30, TimeUnit.SECONDS));
            wire.closeDocument();
            rename.put("closed", server.rename(rename(2, 5, "renamed")).get(30, TimeUnit.SECONDS));
            rename.put("closedFolding", server.foldingRange(new FoldingRangeRequestParams(id())).get(30, TimeUnit.SECONDS));
            out.put("rename-scope", json(rename));
            var closed = server.inlayHint(hints(0, 0, 12, 0)).get(30, TimeUnit.SECONDS);
            assertTrue(closed.isEmpty());
            hints.put("closed", closed);
            wire.open("let a = 1; use a; { note \"/* 偽 */\"; } /* 単一行 */");
            assertTrue(server.foldingRange(new FoldingRangeRequestParams(id())).get(30, TimeUnit.SECONDS).isEmpty());
            wire.closeDocument();
            wire.open("let = 1;");
            var invalid = server.inlayHint(hints(0, 0, 0, 8)).get(30, TimeUnit.SECONDS);
            assertTrue(invalid.isEmpty());
            hints.put("invalid", invalid);
            var protocolJson = new MessageJsonHandler(Map.of()).getGson().newBuilder().setPrettyPrinting().create();
            out.put("inlay-hints", protocolJson.toJson(hints) + "\n");
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
        out.put("signature-help", signatureHelp());
        out.put("formatting", formatting());
        out.put("syntax-quick-fix", QuickFixScenario.run());
        out.put("evaluator", evaluator(work));
        out.put("dap-queries", dapQueries(work));
        out.put("dap-conditions", dapConditions(work));
        out.put("dap-hit-condition", dapHitCondition(work));
        out.put("dap-logpoints", dapLogpoints(work));
        out.put("dap-set-expression", dapSetExpression(work));
        out.put("dap-reverse", dapReverse(work));
        return out;
    }

    private static String signatureHelp() throws Exception {
        var out = new LinkedHashMap<String, Object>();
        try (var wire = new Wire(new LanguageServerImpl(Map.of(), null, new DemoSignatureHelp()))) {
            var capability = wire.lsp.initialize(new InitializeParams()).get(30, TimeUnit.SECONDS)
                .getCapabilities().getSignatureHelpProvider();
            assertEquals(List.of("(", ","), capability.getTriggerCharacters());
            out.put("provider", capability);
            out.put("complete", signatureAt(wire, "sum(1, |2)", "sum(left, right)", 1));
            out.put("incomplete", signatureAt(wire, "sum(1, |", "sum(left, right)", 1));
            out.put("nested", signatureAt(wire, "sum(1, inc(|2))", "inc(value)", 0));
            out.put("outer", signatureAt(wire, "sum(inc(1), |2)", "sum(left, right)", 1));
            out.put("utf16", signatureAt(wire, "/* 😀 */ sum(1,|2)", "sum(left, right)", 1));
            out.put("unknown", signatureAt(wire, "sum(1, unknown(|2))", null, null));
            out.put("outside", signatureAt(wire, "sum(1, 2)|", null, null));
            out.put("closed", wire.lsp.getTextDocumentService()
                .signatureHelp(new SignatureHelpParams(id(), new Position(0, 4))).get(30, TimeUnit.SECONDS));
        }
        return new MessageJsonHandler(Map.of()).getGson().newBuilder().serializeNulls().setPrettyPrinting()
            .create().toJson(out) + "\n";
    }

    private static SignatureHelp signatureAt(Wire wire, String marked, String label, Integer argument) throws Exception {
        int column = marked.indexOf('|');
        wire.open(marked.replace("|", ""));
        var response = wire.lsp.getTextDocumentService().signatureHelp(new SignatureHelpParams(id(), new Position(0, column)))
            .get(30, TimeUnit.SECONDS);
        wire.closeDocument();
        if (label == null) {
            assertNull(response);
        } else {
            assertNotNull(response);
            assertEquals(label, response.getSignatures().getFirst().getLabel());
            assertEquals(argument, response.getActiveParameter());
        }
        return response;
    }

    private static String formatting() throws Exception {
        var out = new LinkedHashMap<String, Object>();
        try (var wire = new Wire(new LanguageServerImpl(Map.of(), null, null, new DemoDocumentFormatter()))) {
            var capability = wire.lsp.initialize(new InitializeParams()).get(30, TimeUnit.SECONDS)
                .getCapabilities().getDocumentFormattingProvider();
            assertTrue(capability.getLeft());
            out.put("provider", capability.getLeft());

            out.put("complete", checkFormatting(wire, "let  total=5;", "let total = 5;\n", new Position(0, 13)));
            String last = "note  \"😀 // keep  /* spaces */\";}";
            String source = "{\r\n/* keep  😀\r\n x */ let  total=5; // keep trailing spaces  \r\n" + last;
            String formatted = "{\n  /* keep  😀\r\n x */\n  let total = 5;\n  // keep trailing spaces  \n"
                + "  note \"😀 // keep  /* spaces */\";\n}\n";
            out.put("utf16", checkFormatting(wire, source, formatted, new Position(3, last.length())));
            out.put("alreadyFormatted", checkFormatting(wire, formatted, null, null));
            out.put("incomplete", checkFormatting(wire, "let total =", null, null));
            out.put("invalid", checkFormatting(wire, "let = 5;", null, null));
            assertTrue(format(wire).isEmpty(), "閉じた文書");
        }
        try (var wire = new Wire(new LanguageServerImpl(Map.of(), null, null, document -> "let total = 6;"))) {
            wire.lsp.initialize(new InitializeParams()).get(30, TimeUnit.SECONDS);
            out.put("changedToken", checkFormatting(wire, "let  total=5;", null, null));
        }
        return json(out);
    }

    private static List<? extends TextEdit> checkFormatting(Wire wire, String source, String formatted, Position end)
            throws Exception {
        wire.open(source);
        var edits = format(wire);
        wire.closeDocument();
        if (formatted == null) {
            assertTrue(edits.isEmpty());
        } else {
            assertEquals(List.of(new TextEdit(new Range(new Position(0, 0), end), formatted)), edits);
            var before = Stage5Parser.parseEntry("Stage5", "Program", source, ParseOptions.DEFAULT);
            var after = Stage5Parser.parseEntry("Stage5", "Program", formatted, ParseOptions.DEFAULT);
            assertTrue(before.ok() && after.ok());
            assertTrue(before.ast().isPresent());
            assertEquals(before.ast(), after.ast());
        }
        return edits;
    }

    private static List<? extends TextEdit> format(Wire wire) throws Exception {
        var params = new DocumentFormattingParams(id(), new FormattingOptions(2, true));
        return wire.lsp.getTextDocumentService().formatting(params).get(30, TimeUnit.SECONDS);
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

    private static String dapQueries(Path work) throws Exception {
        Path file = work.resolve("queries.demo");
        String text = "/* 😀 */ let alpha = 2; let beta = 3;\r\n\r\nnote \"ok\";\r\n";
        Files.writeString(file, text);
        var source = new Source();
        source.setPath(file.toString());
        var other = new Source();
        other.setPath(work.resolve("other.demo").toString());
        int betaColumn = text.indexOf("let beta");
        var out = new LinkedHashMap<String, Object>();
        try (var wire = new DapWire(new DebugAdapter(Map.of(), new SumEvaluatorBridge()))) {
            var dap = wire.proxy;
            var init = new InitializeRequestArguments();
            init.setLinesStartAt1(false);
            init.setColumnsStartAt1(false);
            var caps = dap.initialize(init).get(30, TimeUnit.SECONDS);
            assertTrue(caps.getSupportsBreakpointLocationsRequest());
            assertFalse(caps.getSupportsConditionalBreakpoints());
            assertTrue(caps.getSupportsHitConditionalBreakpoints());
            assertTrue(caps.getSupportsLogPoints());
            assertFalse(caps.getSupportsSetExpression());
            assertFalse(caps.getSupportsStepBack());
            assertFalse(caps.getSupportsEvaluateForHovers());
            out.put("capabilities", caps);
            var beforeLaunch = locations(dap, source, 0, null, null, null);
            assertEquals(0, beforeLaunch.length);
            out.put("beforeLaunch", beforeLaunch);

            dap.launch(Map.of("program", file.toString())).get(30, TimeUnit.SECONDS);
            var all = locations(dap, source, 0, null, 2, null);
            var columns = Arrays.stream(all).filter(site -> site.getLine() == 0).map(BreakpointLocation::getColumn).toList();
            assertTrue(columns.contains(betaColumn));
            assertEquals(columns.stream().distinct().sorted().toList(), columns);
            out.put("locations", all);
            var exact = locations(dap, source, 0, betaColumn, null, betaColumn);
            assertEquals(1, exact.length);
            assertEquals(betaColumn, exact[0].getColumn());
            out.put("utf16Range", exact);
            var emptyLine = locations(dap, source, 1, null, null, null);
            var foreignLocations = locations(dap, other, 0, null, null, null);
            var reversed = locations(dap, source, 2, null, 0, null);
            assertEquals(0, emptyLine.length);
            assertEquals(0, foreignLocations.length);
            assertEquals(0, reversed.length);
            out.put("emptyLine", emptyLine);
            out.put("foreignSource", foreignLocations);
            out.put("reversedRange", reversed);

            var watch = new EvaluateArguments();
            watch.setContext("watch");
            watch.setExpression("total");
            out.put("watchBeforeStop", rejected(() -> dap.evaluate(watch).get(30, TimeUnit.SECONDS)));

            var exactBreakpoint = sourceBreakpoint(0, betaColumn);
            var condition = sourceBreakpoint(0, betaColumn);
            condition.setCondition("total > 0");
            var hitCondition = sourceBreakpoint(0, betaColumn);
            hitCondition.setHitCondition("3");
            var logpoint = sourceBreakpoint(0, betaColumn);
            logpoint.setLogMessage("total={total}");
            var registered = setBreakpoints(dap, source, exactBreakpoint, sourceBreakpoint(0, betaColumn - 1),
                condition, hitCondition, logpoint);
            assertEquals(List.of(true, false, false, true, true),
                Arrays.stream(registered).map(Breakpoint::isVerified).toList());
            out.put("registered", registered);
            var foreignRegistration = setBreakpoints(dap, other, exactBreakpoint);
            assertFalse(foreignRegistration[0].isVerified());
            out.put("foreignRegistration", foreignRegistration);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var resume = new ContinueArguments();
            resume.setThreadId(1);
            dap.continue_(resume).get(30, TimeUnit.SECONDS);
            var stopped = topFrame(dap);
            assertEquals(betaColumn, stopped.getColumn());
            out.put("stoppedAt", List.of(stopped.getLine(), stopped.getColumn()));
            watch.setFrameId(1); // 現在の let から見える親の Program 枠
            var value = dap.evaluate(watch).get(30, TimeUnit.SECONDS);
            assertEquals("5", value.getResult());
            out.put("watch", value);

            watch.setFrameId(2); // 存在するが現在のスタックにはない alpha の枠
            out.put("watchForeignFrame", rejected(() -> dap.evaluate(watch).get(30, TimeUnit.SECONDS)));
            watch.setFrameId(null);
            watch.setExpression("unknown");
            out.put("watchUnknown", rejected(() -> dap.evaluate(watch).get(30, TimeUnit.SECONDS)));
            watch.setExpression("total");
            rejectedStepBack(dap, out);
            out.put("watchAfterStepBack", dap.evaluate(watch).get(30, TimeUnit.SECONDS));
            assertEquals(value, dap.evaluate(watch).get(30, TimeUnit.SECONDS));

            setBreakpoints(dap, source, sourceBreakpoint(0, null));
            Files.writeString(file, "let broken =");
            dap.launch(Map.of("program", file.toString())).get(30, TimeUnit.SECONDS);
            var failedLocations = locations(dap, source, 0, null, null, null);
            assertEquals(0, failedLocations.length);
            out.put("failedLaunchLocations", failedLocations);
            out.put("failedLaunchWatch", rejected(() -> dap.evaluate(watch).get(30, TimeUnit.SECONDS)));

            Files.writeString(file, "let single = 7;");
            dap.launch(Map.of("program", file.toString())).get(30, TimeUnit.SECONDS);
            var relaunched = locations(dap, source, 0, null, null, null);
            assertEquals(1, relaunched.length);
            out.put("relaunchedLocations", relaunched);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var resetValue = dap.evaluate(watch).get(30, TimeUnit.SECONDS);
            assertEquals("7", resetValue.getResult());
            out.put("relaunchedWatch", resetValue);
            dap.continue_(resume).get(30, TimeUnit.SECONDS);
            assertEquals(0, dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getTotalFrames());
            dap.disconnect(new DisconnectArguments()).get(30, TimeUnit.SECONDS);
        }
        try (var wire = new DapWire(new DebugAdapter(Map.of()))) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertTrue(caps.getSupportsBreakpointLocationsRequest());
            assertFalse(caps.getSupportsStepBack());
            dap.launch(Map.of("program", file.toString())).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var next = new StepInArguments();
            next.setThreadId(1);
            dap.stepIn(next).get(30, TimeUnit.SECONDS);
            var all = locations(dap, source, 1, 1, null, 1);
            assertEquals(1, all.length);
            assertEquals(1, all[0].getColumn());
            var noop = new LinkedHashMap<String, Object>();
            noop.put("locations", all);
            var watch = new EvaluateArguments();
            watch.setContext("watch");
            watch.setExpression("total");
            var noWatch = rejected(() -> dap.evaluate(watch).get(30, TimeUnit.SECONDS));
            noop.put("watch", noWatch);
            rejectedStepBack(dap, noop);
            var afterStepBack = rejected(() -> dap.evaluate(watch).get(30, TimeUnit.SECONDS));
            assertEquals(noWatch, afterStepBack);
            noop.put("watchAfterStepBack", afterStepBack);
            out.put("noop", noop);
        }
        return json(out);
    }

    private static String dapConditions(Path work) throws Exception {
        Path file = work.resolve("conditions.demo");
        Files.writeString(file, "let n = 0;\nlet broken = 2;\nlet unsure = 3;\nlet zero = 0; { let n = 3; }\n");
        var source = new Source();
        source.setPath(file.toString());
        var out = new LinkedHashMap<String, Object>();
        EvaluatorBridge bridge = new EvaluatorBridge() {
            @Override public boolean supportsConditionalBreakpoints() { return true; }

            @Override public Optional<Boolean> evaluateCondition(String expression, Object node) {
                return switch (expression) {
                    case "n > 0" -> Optional.of(node instanceof Stage5AST.Let value
                        && value.name().equals("n") && new java.math.BigDecimal(value.value()).signum() > 0);
                    case "failed" -> throw new IllegalArgumentException("評価失敗");
                    default -> Optional.empty();
                };
            }
        };
        try (var wire = new DapWire(new DebugAdapter(Map.of(), bridge))) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertTrue(caps.getSupportsConditionalBreakpoints());
            out.put("supportsConditionalBreakpoints", caps.getSupportsConditionalBreakpoints());
            var launch = Map.<String, Object>of("program", file.toString(), "stopOnEntry", false);
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            var registered = setBreakpoints(dap, source, conditionalBreakpoint(1, 1, "n > 0"),
                conditionalBreakpoint(2, null, "failed"), conditionalBreakpoint(3, null, "unknown"),
                conditionalBreakpoint(4, null, "n > 0"));
            assertTrue(Arrays.stream(registered).allMatch(Breakpoint::isVerified));
            out.put("registered", registered);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var frame = topFrame(dap);
            assertEquals(4, frame.getLine());
            assertEquals(17, frame.getColumn());
            assertEquals(List.of("stderr:条件付き breakpoint の評価に失敗しました。\n",
                "stderr:条件付き breakpoint の式を評価できません。\n", "stopped:breakpoint"), wire.breakpointEvents);
            out.put("events", List.copyOf(wire.breakpointEvents));
            out.put("stoppedAt", List.of(frame.getLine(), frame.getColumn()));
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var frames = dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getTotalFrames();
            assertEquals(0, frames);
            assertEquals(3, wire.breakpointEvents.size());
            out.put("relaunchedFrames", frames);
        }
        try (var wire = new DapWire(new DebugAdapter(Map.of()))) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertFalse(caps.getSupportsConditionalBreakpoints());
            dap.launch(Map.of("program", file.toString())).get(30, TimeUnit.SECONDS);
            var denied = setBreakpoints(dap, source, conditionalBreakpoint(4, null, "n > 0"));
            assertFalse(denied[0].isVerified());
            out.put("noop", denied[0]);
        }
        return json(out);
    }

    private static SourceBreakpoint conditionalBreakpoint(int line, Integer column, String expression) {
        var breakpoint = sourceBreakpoint(line, column);
        breakpoint.setCondition(expression);
        return breakpoint;
    }

    private static String dapHitCondition(Path work) throws Exception {
        Path file = work.resolve("hits.demo");
        Files.writeString(file, "let a = 1; let b = 2; let c = 3;\n");
        var source = new Source();
        source.setPath(file.toString());
        var launch = Map.<String, Object>of("program", file.toString(), "stopOnEntry", false);
        var resume = new ContinueArguments();
        resume.setThreadId(1);
        var out = new LinkedHashMap<String, Object>();

        try (var wire = new DapWire(new DebugAdapter(Map.of()))) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertTrue(caps.getSupportsHitConditionalBreakpoints());
            out.put("supportsHitConditionalBreakpoints", caps.getSupportsHitConditionalBreakpoints());
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            var first = setBreakpoints(dap, source, hitBreakpoint("3"), hitBreakpoint("0"),
                hitBreakpoint(">= 3"), hitBreakpoint("2147483648"), hitBreakpoint("３"), hitBreakpoint(""));
            assertEquals(List.of(true, false, false, false, false, false),
                Arrays.stream(first).map(Breakpoint::isVerified).toList());
            out.put("registrations", first);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            // Program と let a は同じ (1,1) で、第 1・2 回。let b が第 3 回。
            var third = topFrame(dap);
            assertEquals(12, third.getColumn());
            out.put("thirdHit", List.of(third.getLine(), third.getColumn()));
            dap.continue_(resume).get(30, TimeUnit.SECONDS);
            assertEquals(0, dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getTotalFrames());

            dap.launch(launch).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var frames = dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getTotalFrames();
            assertEquals(0, frames);
            out.put("relaunchedFrames", frames);

            dap.launch(launch).get(30, TimeUnit.SECONDS);
            setBreakpoints(dap, source, hitBreakpoint("2"));
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var second = topFrame(dap);
            assertEquals(1, second.getColumn());
            assertEquals(2, dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getTotalFrames());
            setBreakpoints(dap, source, hitBreakpoint("2"));
            dap.continue_(resume).get(30, TimeUnit.SECONDS);
            var afterReset = topFrame(dap);
            assertEquals(23, afterReset.getColumn());
            out.put("afterReregistration", List.of(afterReset.getLine(), afterReset.getColumn()));
            assertEquals(List.of("stopped:breakpoint", "stopped:breakpoint", "stopped:breakpoint"), wire.breakpointEvents);
            out.put("events", List.copyOf(wire.breakpointEvents));
        }

        EvaluatorBridge bridge = new EvaluatorBridge() {
            @Override public boolean supportsConditionalBreakpoints() { return true; }

            @Override public Optional<Boolean> evaluateCondition(String expression, Object node) {
                if (node instanceof Stage5AST.Let value && value.name().equals("b"))
                    return Optional.of(expression.equals("b"));
                throw new IllegalArgumentException("回数に達するまでは条件を評価しない");
            }
        };
        try (var wire = new DapWire(new DebugAdapter(Map.of(), bridge))) {
            var dap = wire.proxy;
            dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            var both = hitBreakpoint("3");
            both.setCondition("b");
            assertTrue(setBreakpoints(dap, source, both)[0].isVerified());
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            assertEquals(12, topFrame(dap).getColumn());
            assertEquals(List.of("stopped:breakpoint"), wire.breakpointEvents);
            out.put("combinedEvents", List.copyOf(wire.breakpointEvents));
        }
        return json(out);
    }

    private static SourceBreakpoint hitBreakpoint(String threshold) {
        var breakpoint = sourceBreakpoint(1, null);
        breakpoint.setHitCondition(threshold);
        return breakpoint;
    }

    private static String dapLogpoints(Path work) throws Exception {
        Path file = work.resolve("logpoints.demo");
        Files.writeString(file, "let a = 2;\nlet b = 3;\nuse a;\nuse b;\n");
        var source = new Source();
        source.setPath(file.toString());
        var launch = Map.<String, Object>of("program", file.toString(), "stopOnEntry", false);
        var out = new LinkedHashMap<String, Object>();
        var sum = new SumEvaluatorBridge();
        EvaluatorBridge bridge = new EvaluatorBridge() {
            @Override public void onLaunch(String program, String text, Object root) {
                sum.onLaunch(program, text, root);
            }
            @Override public boolean supportsConditionalBreakpoints() { return true; }
            @Override public Optional<Boolean> evaluateCondition(String expression, Object node) {
                return Optional.of(expression.equals("true"));
            }
            @Override public boolean supportsLogpointExpressions() { return true; }
            @Override public Optional<String> evaluateLogpointExpression(String expression, Object node) {
                if (expression.equals("failed")) throw new IllegalArgumentException("評価失敗");
                return sum.evaluateLogpointExpression(expression, node);
            }
        };
        try (var wire = new DapWire(new DebugAdapter(Map.of(), bridge))) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertTrue(caps.getSupportsLogPoints());
            out.put("supportsLogPoints", caps.getSupportsLogPoints());
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            var suppressed = logpoint(3, "抑止={failed}");
            suppressed.setCondition("false");
            var combined = logpoint(4, "併用={total}");
            combined.setCondition("true");
            combined.setHitCondition("1");
            var registered = setBreakpoints(dap, source, logpoint(2, "total={ total } / {{{total}}}"),
                logpoint(3, "部分出力なし={total} {unknown}"), suppressed, logpoint(3, "{failed}"),
                sourceBreakpoint(4, null), combined);
            assertTrue(Arrays.stream(registered).allMatch(Breakpoint::isVerified));
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            assertEquals(4, topFrame(dap).getLine());
            assertEquals(List.of("console:total=5 / {5}\n", "stderr:logpoint の式を評価できません。\n",
                "stderr:logpoint の評価に失敗しました。\n", "console:併用=5\n", "stopped:breakpoint"),
                wire.breakpointEvents);
            out.put("events", List.copyOf(wire.breakpointEvents));
            Files.writeString(file, "let a = 7;\nuse a;\n");
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            assertEquals(0, dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getTotalFrames());
            assertEquals(5, wire.breakpointEvents.size());
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            assertTrue(setBreakpoints(dap, source, logpoint(2, "total={total}"))[0].isVerified());
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            assertEquals(0, dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getTotalFrames());
            assertEquals("console:total=7\n", wire.breakpointEvents.getLast());
            out.put("afterRelaunch", List.copyOf(wire.breakpointEvents.subList(5, wire.breakpointEvents.size())));
        }
        try (var wire = new DapWire(new DebugAdapter(Map.of()))) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertTrue(caps.getSupportsLogPoints());
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            var registered = setBreakpoints(dap, source, logpoint(2, "literal={{total}}"),
                logpoint(2, "{total}"), logpoint(2, "{ "), logpoint(2, "{ }"), logpoint(2, "{ {x} }"));
            assertEquals(List.of(true, false, false, false, false), Arrays.stream(registered).map(Breakpoint::isVerified).toList());
            out.put("noopRegistration", registered);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            assertEquals(0, dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getTotalFrames());
            assertEquals(List.of("console:literal={total}\n"), wire.breakpointEvents);
            out.put("noopEvents", List.copyOf(wire.breakpointEvents));
        }
        return json(out);
    }

    private static SourceBreakpoint logpoint(int line, String message) {
        var breakpoint = sourceBreakpoint(line, null);
        breakpoint.setLogMessage(message);
        return breakpoint;
    }

    private static String dapSetExpression(Path work) throws Exception {
        Path file = work.resolve("assignment.demo");
        Files.writeString(file, "let counter = 5;\nlet sibling = 2;\n");
        var launch = Map.<String, Object>of("program", file.toString());
        var out = new LinkedHashMap<String, Object>();
        EvaluatorBridge readOnly = new EvaluatorBridge() {
            @Override public Optional<Evaluation> evaluate(String expression, Object node, String context) {
                return "watch".equals(context) && node instanceof Stage5AST.Let value && value.name().equals(expression)
                    ? Optional.of(new Evaluation(value.value(), "number")) : Optional.empty();
            }
            @Override public Optional<Evaluation> setVariable(String container, String name, String value, Object node) {
                throw new AssertionError("setExpression は setVariable に委譲しない");
            }
        };
        var watch = new EvaluateArguments();
        watch.setContext("watch");
        watch.setExpression("counter");
        try (var wire = new DapWire(new DebugAdapter(Map.of(), readOnly))) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertFalse(caps.getSupportsSetExpression());
            out.put("supportsSetExpression", caps.getSupportsSetExpression());
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var step = new StepInArguments();
            step.setThreadId(1);
            dap.stepIn(step).get(30, TimeUnit.SECONDS);
            var frame = topFrame(dap);
            var value = dap.evaluate(watch).get(30, TimeUnit.SECONDS);
            assertEquals("5", value.getResult());
            out.put("validTargetRejected", rejectedAssignment(dap, "7", frame.getId()));
            out.put("invalidValueRejected", rejectedAssignment(dap, "arbitrary()", frame.getId()));
            out.put("foreignFrameRejected", rejectedAssignment(dap, "7", frame.getId() + 1));
            assertEquals(frame, topFrame(dap));
            assertEquals(value, dap.evaluate(watch).get(30, TimeUnit.SECONDS));
            assertEquals(List.of("stopped:entry", "stopped:step"), wire.breakpointEvents);
            out.put("unchangedFrame", List.of(frame.getId(), frame.getLine(), frame.getColumn()));
            out.put("unchangedValue", value.getResult());

            Files.writeString(file, "let counter = 9;\n");
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            dap.stepIn(step).get(30, TimeUnit.SECONDS);
            out.put("relaunchedRejected", rejectedAssignment(dap, "7", null));
            var relaunched = dap.evaluate(watch).get(30, TimeUnit.SECONDS);
            assertEquals("9", relaunched.getResult());
            out.put("relaunchedValue", relaunched.getResult());
        }
        try (var wire = new DapWire(new DebugAdapter(Map.of()))) {
            var dap = wire.proxy;
            assertFalse(dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS).getSupportsSetExpression());
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var frame = topFrame(dap);
            out.put("noopRejected", rejectedAssignment(dap, "7", null));
            assertEquals(frame, topFrame(dap));
        }
        return json(out);
    }

    private static String rejectedAssignment(IDebugProtocolServer dap, String value, Integer frameId) throws Exception {
        var args = new SetExpressionArguments();
        args.setExpression("counter");
        args.setValue(value);
        args.setFrameId(frameId);
        return rejected(() -> dap.setExpression(args).get(30, TimeUnit.SECONDS));
    }

    private static String dapReverse(Path work) throws Exception {
        Path file = work.resolve("reverse.demo");
        Files.writeString(file, "let a = 2;\nlet b = 3;\nlet c = 4;\nlet d = 1;\n");
        var source = new Source();
        source.setPath(file.toString());
        var launch = Map.<String, Object>of("program", file.toString());
        var advance = new StepInArguments();
        advance.setThreadId(1);
        var resume = new ContinueArguments();
        resume.setThreadId(1);
        var stepBack = new StepBackArguments();
        stepBack.setThreadId(1);
        var reverse = new ReverseContinueArguments();
        reverse.setThreadId(1);
        var out = new LinkedHashMap<String, Object>();
        var bridge = new HistoricalTotals();
        try (var wire = new DapWire(new DebugAdapter(Map.of(), bridge))) {
            var dap = wire.proxy;
            var caps = dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS);
            assertTrue(caps.getSupportsStepBack());
            out.put("supportsStepBack", caps.getSupportsStepBack());
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            var breakpoint = sourceBreakpoint(2, null);
            breakpoint.setHitCondition("1");
            assertTrue(setBreakpoints(dap, source, breakpoint)[0].isVerified());
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var entry = reversePoint(dap, 1, "0");
            out.put("atStart", rejected(() -> dap.stepBack(stepBack).get(30, TimeUnit.SECONDS)));
            out.put("reverseAtStart", rejected(() -> dap.reverseContinue(reverse).get(30, TimeUnit.SECONDS)));
            assertEquals(entry, reversePoint(dap, 1, "0"));

            dap.stepIn(advance).get(30, TimeUnit.SECONDS);
            var a = reversePoint(dap, 1, "2");
            dap.continue_(resume).get(30, TimeUnit.SECONDS);
            var b = reversePoint(dap, 2, "5");
            out.put("beforeStepBack", b);
            dap.stepBack(stepBack).get(30, TimeUnit.SECONDS);
            assertEquals(a, reversePoint(dap, 1, "2"));
            out.put("afterStepBack", a);
            dap.continue_(resume).get(30, TimeUnit.SECONDS);
            assertEquals(b, reversePoint(dap, 2, "5"));

            dap.stepIn(advance).get(30, TimeUnit.SECONDS);
            reversePoint(dap, 3, "9");
            dap.stepIn(advance).get(30, TimeUnit.SECONDS);
            var d = reversePoint(dap, 4, "10");
            int stops = wire.breakpointEvents.size();
            bridge.rejectRestore = true;
            out.put("rejectedRestore", rejected(() -> dap.stepBack(stepBack).get(30, TimeUnit.SECONDS)));
            assertEquals(d, reversePoint(dap, 4, "10"));
            bridge.throwOnRestore = true;
            out.put("failedReverseRestore", rejected(() -> dap.reverseContinue(reverse).get(30, TimeUnit.SECONDS)));
            assertEquals(d, reversePoint(dap, 4, "10"));
            assertEquals(stops, wire.breakpointEvents.size());
            out.put("afterRestoreFailures", d);
            bridge.rejectRestore = false;
            bridge.throwOnRestore = false;
            dap.reverseContinue(reverse).get(30, TimeUnit.SECONDS);
            assertEquals(b, reversePoint(dap, 2, "5"));
            out.put("afterReverseContinue", b);
            out.put("events", List.copyOf(wire.breakpointEvents));

            Files.writeString(file, "let single = 7;\n");
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            out.put("relaunched", rejected(() -> dap.stepBack(stepBack).get(30, TimeUnit.SECONDS)));
            assertEquals(entry, reversePoint(dap, 1, "0"));
            dap.stepIn(advance).get(30, TimeUnit.SECONDS);
            reversePoint(dap, 1, "7");
            dap.reverseContinue(reverse).get(30, TimeUnit.SECONDS);
            out.put("reverseToEntry", reversePoint(dap, 1, "0"));
            assertEquals("stopped:entry", wire.breakpointEvents.getLast());

            bridge.skipSnapshot = true;
            dap.stepIn(advance).get(30, TimeUnit.SECONDS);
            out.put("missingSnapshot", rejected(() -> dap.stepBack(stepBack).get(30, TimeUnit.SECONDS)));
            reversePoint(dap, 1, "7");
        }
        try (var wire = new DapWire(new DebugAdapter(Map.of()))) {
            var dap = wire.proxy;
            assertFalse(dap.initialize(new InitializeRequestArguments()).get(30, TimeUnit.SECONDS).getSupportsStepBack());
            dap.launch(launch).get(30, TimeUnit.SECONDS);
            dap.configurationDone(new ConfigurationDoneArguments()).get(30, TimeUnit.SECONDS);
            var entry = topFrame(dap);
            out.put("noopStepBack", rejected(() -> dap.stepBack(stepBack).get(30, TimeUnit.SECONDS)));
            out.put("noopReverseContinue", rejected(() -> dap.reverseContinue(reverse).get(30, TimeUnit.SECONDS)));
            assertEquals(entry, topFrame(dap));
            assertEquals(List.of("stopped:entry"), wire.breakpointEvents);
        }
        return json(out);
    }

    private static List<Object> reversePoint(IDebugProtocolServer dap, int line, String total) throws Exception {
        var frame = topFrame(dap);
        assertEquals(line, frame.getLine());
        var watch = new EvaluateArguments();
        watch.setContext("watch");
        watch.setExpression("total");
        var value = dap.evaluate(watch).get(30, TimeUnit.SECONDS);
        assertEquals(total, value.getResult());
        return List.of(frame.getId(), frame.getLine(), frame.getColumn(), value.getResult());
    }

    private static final class HistoricalTotals implements EvaluatorBridge {
        private record Total(BigDecimal value) implements Snapshot { }
        private BigDecimal total = BigDecimal.ZERO;
        volatile boolean rejectRestore;
        volatile boolean throwOnRestore;
        volatile boolean skipSnapshot;

        @Override public void onLaunch(String program, String text, Object root) { total = BigDecimal.ZERO; }
        @Override public void onEnterNode(Object node) {
            if (node instanceof Stage5AST.Let value) total = total.add(new BigDecimal(value.value()));
        }
        @Override public Optional<Evaluation> evaluate(String expression, Object node, String context) {
            return "watch".equals(context) && "total".equals(expression)
                ? Optional.of(new Evaluation(total.toPlainString(), "number")) : Optional.empty();
        }
        @Override public boolean supportsReverseExecution() { return true; }
        @Override public Optional<Snapshot> snapshot() {
            return skipSnapshot ? Optional.empty() : Optional.of(new Total(total));
        }
        @Override public boolean restore(Snapshot snapshot) {
            if (throwOnRestore) throw new IllegalStateException("評価器の復元失敗");
            if (rejectRestore || !(snapshot instanceof Total restored)) return false;
            total = restored.value();
            return true;
        }
    }

    private static BreakpointLocation[] locations(IDebugProtocolServer dap, Source source,
            int line, Integer column, Integer endLine, Integer endColumn) throws Exception {
        var args = new BreakpointLocationsArguments();
        args.setSource(source);
        args.setLine(line);
        args.setColumn(column);
        args.setEndLine(endLine);
        args.setEndColumn(endColumn);
        return dap.breakpointLocations(args).get(30, TimeUnit.SECONDS).getBreakpoints();
    }

    private static SourceBreakpoint sourceBreakpoint(int line, Integer column) {
        var breakpoint = new SourceBreakpoint();
        breakpoint.setLine(line);
        breakpoint.setColumn(column);
        return breakpoint;
    }

    private static Breakpoint[] setBreakpoints(IDebugProtocolServer dap, Source source,
            SourceBreakpoint... breakpoints) throws Exception {
        var args = new SetBreakpointsArguments();
        args.setSource(source);
        args.setBreakpoints(breakpoints);
        return dap.setBreakpoints(args).get(30, TimeUnit.SECONDS).getBreakpoints();
    }

    private static StackTraceArguments stackRequest() {
        var args = new StackTraceArguments();
        args.setThreadId(1);
        return args;
    }

    private static StackFrame topFrame(IDebugProtocolServer dap) throws Exception {
        return dap.stackTrace(stackRequest()).get(30, TimeUnit.SECONDS).getStackFrames()[0];
    }

    private static void rejectedStepBack(IDebugProtocolServer dap, Map<String, Object> out) throws Exception {
        var before = topFrame(dap);
        var stepBack = new StepBackArguments();
        stepBack.setThreadId(1);
        out.put("stepBack", rejected(() -> dap.stepBack(stepBack).get(30, TimeUnit.SECONDS)));
        var after = topFrame(dap);
        assertEquals(before, after);
        out.put("unchangedFrame", List.of(after.getId(), after.getLine(), after.getColumn()));
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
    private static InlayHintParams hints(int startLine, int startColumn, int endLine, int endColumn) {
        return new InlayHintParams(id(), new Range(new Position(startLine, startColumn), new Position(endLine, endColumn)));
    }

    private static RenameParams rename(int line, int column, String name) {
        return new RenameParams(id(), new Position(line, column), name);
    }

    /** パイプと専用 executor は最後に閉じ、テストが通信スレッドを残さないようにする。 */
    static final class Wire implements AutoCloseable, LanguageClient {
        final LanguageServer local, lsp;
        final PipedInputStream serverIn = new PipedInputStream(), clientIn = new PipedInputStream();
        final PipedOutputStream clientOut = new PipedOutputStream(serverIn), serverOut = new PipedOutputStream(clientIn);
        final ExecutorService executor = Executors.newCachedThreadPool();
        final Future<?> serverListening, clientListening;
        volatile CompletableFuture<PublishDiagnosticsParams> diagnostic = new CompletableFuture<>();
        Wire(LanguageServer local) throws Exception {
            this.local = local;
            var s = LSPLauncher.createServerLauncher(local, serverIn, serverOut, executor, consumer -> consumer);
            ((LanguageClientAware) local).connect(s.getRemoteProxy());
            var c = LSPLauncher.createClientLauncher(this, clientIn, clientOut, executor, consumer -> consumer);
            serverListening = s.startListening(); clientListening = c.startListening(); lsp = c.getRemoteProxy();
        }
        List<Diagnostic> open(String text) throws Exception { return open(URI, text); }
        List<Diagnostic> open(String uri, String text) throws Exception {
            diagnostic = new CompletableFuture<>();
            lsp.getTextDocumentService().didOpen(new DidOpenTextDocumentParams(new TextDocumentItem(uri, "demo", 1, text)));
            return diagnostics(uri);
        }
        List<Diagnostic> change(String text, int version) throws Exception {
            diagnostic = new CompletableFuture<>();
            var document = new VersionedTextDocumentIdentifier(URI, version);
            var changes = List.of(new TextDocumentContentChangeEvent(text));
            lsp.getTextDocumentService().didChange(new DidChangeTextDocumentParams(document, changes));
            return diagnostics(URI);
        }
        private List<Diagnostic> diagnostics(String uri) throws Exception {
            var published = diagnostic.get(30, TimeUnit.SECONDS);
            assertEquals(uri, published.getUri());
            return published.getDiagnostics();
        }
        void closeDocument() throws Exception {
            diagnostic = new CompletableFuture<>();
            lsp.getTextDocumentService().didClose(new DidCloseTextDocumentParams(id()));
            diagnostic.get(30, TimeUnit.SECONDS);
        }
        public void publishDiagnostics(PublishDiagnosticsParams params) { diagnostic.complete(params); }
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
        final List<String> breakpointEvents = new CopyOnWriteArrayList<>();
        DapWire(DebugAdapter adapter) throws Exception {
            var s = DSPLauncher.createServerLauncher(adapter, serverIn, serverOut, executor, consumer -> consumer);
            adapter.connect(s.getRemoteProxy());
            var client = new IDebugProtocolClient() {
                @Override public void stopped(StoppedEventArguments args) {
                    breakpointEvents.add("stopped:" + args.getReason());
                }
                @Override public void output(OutputEventArguments args) {
                    String category = args.getCategory();
                    if ("stderr".equals(category) || "console".equals(category))
                        breakpointEvents.add(category + ":" + args.getOutput());
                }
            };
            var c = DSPLauncher.createClientLauncher(client, clientIn, clientOut, executor, consumer -> consumer);
            serverListening = s.startListening(); clientListening = c.startListening(); proxy = c.getRemoteProxy();
        }
        public void close() throws Exception {
            serverListening.cancel(true); clientListening.cancel(true);
            clientOut.close(); serverOut.close(); serverIn.close(); clientIn.close(); executor.shutdownNow();
        }
    }
}
