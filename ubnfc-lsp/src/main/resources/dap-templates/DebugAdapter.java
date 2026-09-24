package __PACKAGE__;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.function.Supplier;
import org.eclipse.lsp4j.debug.Breakpoint;
import org.eclipse.lsp4j.debug.Capabilities;
import org.eclipse.lsp4j.debug.ConfigurationDoneArguments;
import org.eclipse.lsp4j.debug.ContinueArguments;
import org.eclipse.lsp4j.debug.ContinueResponse;
import org.eclipse.lsp4j.debug.DisconnectArguments;
import org.eclipse.lsp4j.debug.EvaluateArguments;
import org.eclipse.lsp4j.debug.EvaluateResponse;
import org.eclipse.lsp4j.debug.ExitedEventArguments;
import org.eclipse.lsp4j.debug.InitializeRequestArguments;
import org.eclipse.lsp4j.debug.NextArguments;
import org.eclipse.lsp4j.debug.OutputEventArguments;
import org.eclipse.lsp4j.debug.PauseArguments;
import org.eclipse.lsp4j.debug.Scope;
import org.eclipse.lsp4j.debug.ScopesArguments;
import org.eclipse.lsp4j.debug.ScopesResponse;
import org.eclipse.lsp4j.debug.SetBreakpointsArguments;
import org.eclipse.lsp4j.debug.SetBreakpointsResponse;
import org.eclipse.lsp4j.debug.SetVariableArguments;
import org.eclipse.lsp4j.debug.SetVariableResponse;
import org.eclipse.lsp4j.debug.Source;
import org.eclipse.lsp4j.debug.StackFrame;
import org.eclipse.lsp4j.debug.StackTraceArguments;
import org.eclipse.lsp4j.debug.StackTraceResponse;
import org.eclipse.lsp4j.debug.StepInArguments;
import org.eclipse.lsp4j.debug.StepOutArguments;
import org.eclipse.lsp4j.debug.StoppedEventArguments;
import org.eclipse.lsp4j.debug.TerminatedEventArguments;
import org.eclipse.lsp4j.debug.ThreadsResponse;
import org.eclipse.lsp4j.debug.Variable;
import org.eclipse.lsp4j.debug.VariablesArguments;
import org.eclipse.lsp4j.debug.VariablesResponse;
import org.eclipse.lsp4j.debug.services.IDebugProtocolClient;
import org.eclipse.lsp4j.debug.services.IDebugProtocolServer;
import org.eclipse.lsp4j.jsonrpc.ResponseErrorException;
import org.eclipse.lsp4j.jsonrpc.messages.ResponseError;
import org.eclipse.lsp4j.jsonrpc.messages.ResponseErrorCode;
import __API_PACKAGE__.ParseOptions;
import __API_PACKAGE__.ParseResult;
import __API_PACKAGE__.TokenScanner;
import __LSP_PACKAGE__.Analysis;
import __LSP_PACKAGE__.Diagnostics;
import __LSP_PACKAGE__.LanguageServerImpl;
import __LSP_PACKAGE__.Meta;
import __LSP_PACKAGE__.Positions;

/**
 * __GRAMMAR__ 用に生成された<b>文法デバッガ</b>（DAP、opt-in feature）。
 *
 * <p>「文法デバッガ」とは、<b>入力をこの文法でどう構造化したか</b>をステップ実行で見せるもので、
 * 言語の意味論（式の値）は計算しない。旧スタック（{@code unlaxer-dsl} の {@code DAPGenerator}）の
 * base 生成と同じ立場で、評価器は言語作者が {@link EvaluatorBridge} で差し込む（issue #28 段階 4）。
 *
 * <h2>ステップの意味論</h2>
 * <p>{@code launch} で入力を生成パーサに掛け、得た<b>型付き AST を前順（親 → 子、子は文法の宣言順）
 * で平らにした列</b>をステップ列とする（{@link Steps}）。位置はステップの節点の範囲で、
 * {@code stackTrace} は根から現在の節点までの鎖をそのまま枠にする。
 * {@code stepIn} は列の次（＝子があれば最初の子）、{@code next} は<b>現在と同じか浅い深さ</b>の次、
 * {@code stepOut} は<b>より浅い深さ</b>の次、{@code continue} は<b>開始行にブレークポイントがある</b>
 * 次のステップへ進む。いずれも進み先が無ければ {@code terminated}。
 * 旧スタックの token モードは<b>葉だけ</b>を左から右へ辿る後順で、{@code next} しか無かったため
 * 「入る／出る」を表せなかった。ubnfc は型付き AST を持っているので前順にし、
 * 親の枠が子より外側に来る（＝普通の debugger と同じ見え方になる）ようにした。
 */
public class DebugAdapter implements IDebugProtocolServer {
    /** variables の値の表示上限（旧 {@code TinyExpressionDapRuntimeBridge.VALUE_LIMIT} と同じ）。 */
    private static final int VALUE_LIMIT = 200;
    private static final int THREAD_ID = 1;

    private final Map<String, TokenScanner> scanners;
    private final EvaluatorBridge bridge;

    private IDebugProtocolClient client;
    private boolean linesStartAt1 = true;
    private boolean columnsStartAt1 = true;

    private String program = "";
    private String text = "";
    private Positions positions = new Positions("");
    private ParseResult<?> result;
    private List<Steps.Step> steps = List.of();
    private final java.util.Set<Integer> breakpointLines = new java.util.LinkedHashSet<>();
    private int cursor = -1;
    private boolean stopOnEntry = true;
    private boolean launched;
    private boolean terminated;

    /** variablesReference → その中身を作る手続き。stop のたびに作り直す。 */
    private final List<Supplier<List<Variable>>> handles = new ArrayList<>();

    private final CompletableFuture<Void> exit = new CompletableFuture<>();

    public DebugAdapter() { this(LanguageServerImpl.scanners(LanguageServerImpl.SCANNER_CLASS), EvaluatorBridge.NOOP); }

    public DebugAdapter(Map<String, TokenScanner> scanners) { this(scanners, EvaluatorBridge.NOOP); }

    public DebugAdapter(Map<String, TokenScanner> scanners, EvaluatorBridge bridge) {
        this.scanners = Map.copyOf(scanners);
        this.bridge = bridge == null ? EvaluatorBridge.NOOP : bridge;
    }

    public void connect(IDebugProtocolClient client) { this.client = client; }

    /** {@code disconnect} が来たら完了する。{@code DapMain} がこれを待つ。 */
    public CompletableFuture<Void> exited() { return exit; }

    // ---- 起動 --------------------------------------------------------------

    @Override public CompletableFuture<Capabilities> initialize(InitializeRequestArguments args) {
        if (args != null) {
            linesStartAt1 = !Boolean.FALSE.equals(args.getLinesStartAt1());
            columnsStartAt1 = !Boolean.FALSE.equals(args.getColumnsStartAt1());
        }
        var capabilities = new Capabilities();
        capabilities.setSupportsConfigurationDoneRequest(true);
        // 値を出せるのは EvaluatorBridge を実装したときだけ。無いのに true を返すと
        // VS Code が watch / hover を出し続けて「黙って空」に見えるので、正直に false を返す。
        capabilities.setSupportsEvaluateForHovers(false);
        capabilities.setSupportsSetVariable(false);
        capabilities.setSupportsConditionalBreakpoints(false);
        capabilities.setSupportsHitConditionalBreakpoints(false);
        capabilities.setSupportsStepBack(false);
        capabilities.setSupportsTerminateRequest(false);
        capabilities.setExceptionBreakpointFilters(new org.eclipse.lsp4j.debug.ExceptionBreakpointsFilter[0]);
        return CompletableFuture.completedFuture(capabilities);
    }

    @Override public CompletableFuture<Void> launch(Map<String, Object> args) {
        Map<String, Object> arguments = args == null ? Map.of() : new LinkedHashMap<>(args);
        // initialized は launch の中で送る（旧スタックと同じ）。こうすると setBreakpoints が
        // 「解析済みのステップ列」を見られるので、verified をその場で判定できる。
        // lsp4j は受信スレッド上で 1 要求ずつ処理するので、次の要求はこの launch が返るまで読まれない。
        if (client != null) client.initialized();
        program = string(arguments.get("program"));
        stopOnEntry = !Boolean.FALSE.equals(arguments.get("stopOnEntry"));
        launched = true;
        if (program.isBlank()) { fail("program が指定されていません"); return done(); }
        try {
            text = Files.readString(Path.of(program), StandardCharsets.UTF_8);
        } catch (Exception error) {
            fail("入力を読めません: " + program + " (" + error.getClass().getSimpleName() + ")");
            return done();
        }
        positions = new Positions(text);
        try {
            result = __PARSER_CLASS__.parseEntry(LanguageServerImpl.GRAMMAR, LanguageServerImpl.ENTRY, text,
                new ParseOptions(true, true, false, true, scanners, ParseOptions.DEFAULT_MAX_DEPTH));
        } catch (RuntimeException error) {
            fail("解析器の内部エラー: " + error);
            return done();
        }
        if (!result.ok() || result.ast().isEmpty()) { fail(null); return done(); }
        steps = Steps.of(result.ast().get(), result.nodeSpans());
        cursor = -1;
        bridge.onLaunch(program, text, result.ast().get());
        output("stdout", "Parsed " + program + ": " + steps.size() + " AST node step(s)\n");
        return done();
    }

    /** 解析に失敗した。LSP と同じ文面の診断を {@code output} で流してから終了する。 */
    private void fail(String message) {
        if (message != null) output("stderr", message + "\n");
        else
            for (var diagnostic : Diagnostics.of(Analysis.of(uri(), text, scanners))) {
                var start = diagnostic.getRange().getStart();
                output("stderr", program + ":" + (start.getLine() + 1) + ":" + (start.getCharacter() + 1)
                    + ": " + diagnostic.getSeverity() + ": " + diagnostic.getMessage() + "\n");
            }
        steps = List.of();
        terminate();
    }

    private String uri() { return Path.of(program.isBlank() ? "." : program).toUri().toString(); }

    @Override public CompletableFuture<Void> configurationDone(ConfigurationDoneArguments args) {
        if (terminated || steps.isEmpty()) return done();
        if (stopOnEntry) { move(0, "entry"); return done(); }
        resume();
        return done();
    }

    // ---- ブレークポイント ---------------------------------------------------

    @Override public CompletableFuture<SetBreakpointsResponse> setBreakpoints(SetBreakpointsArguments args) {
        breakpointLines.clear();
        var requested = args == null || args.getBreakpoints() == null
            ? new org.eclipse.lsp4j.debug.SourceBreakpoint[0] : args.getBreakpoints();
        var out = new Breakpoint[requested.length];
        for (int i = 0; i < requested.length; i++) {
            int line = requested[i].getLine();
            int zeroBased = linesStartAt1 ? line - 1 : line;
            var breakpoint = new Breakpoint();
            breakpoint.setLine(line);
            // 旧スタックは検証せず常に verified=true を返していた。ubnfc はステップ列と突き合わせ、
            // その行から始まる AST 節点が 1 つも無ければ verified=false + 理由を返す。
            boolean verified = false;
            for (var step : steps) if (lineOf(step.startCp()) == zeroBased) { verified = true; break; }
            breakpoint.setVerified(verified);
            if (verified) breakpointLines.add(zeroBased);
            else breakpoint.setMessage("この行から始まる AST 節点がありません（文法デバッガは節点の開始行で止まります）");
            out[i] = breakpoint;
        }
        var response = new SetBreakpointsResponse();
        response.setBreakpoints(out);
        return CompletableFuture.completedFuture(response);
    }

    // ---- 実行 --------------------------------------------------------------

    @Override public CompletableFuture<ContinueResponse> continue_(ContinueArguments args) {
        if (!terminated) resume();
        var response = new ContinueResponse();
        response.setAllThreadsContinued(true);
        return CompletableFuture.completedFuture(response);
    }

    private void resume() {
        for (int at = cursor + 1; at < steps.size(); at++)
            if (breakpointLines.contains(lineOf(steps.get(at).startCp()))) { move(at, "breakpoint"); return; }
        terminate();
    }

    @Override public CompletableFuture<Void> next(NextArguments args) {
        if (terminated) return done();
        int depth = current() == null ? Integer.MAX_VALUE : current().depth();
        for (int at = cursor + 1; at < steps.size(); at++)
            if (steps.get(at).depth() <= depth) { move(at, "step"); return done(); }
        terminate();
        return done();
    }

    @Override public CompletableFuture<Void> stepIn(StepInArguments args) {
        if (terminated) return done();
        if (cursor + 1 < steps.size()) move(cursor + 1, "step"); else terminate();
        return done();
    }

    @Override public CompletableFuture<Void> stepOut(StepOutArguments args) {
        if (terminated) return done();
        int depth = current() == null ? 0 : current().depth();
        for (int at = cursor + 1; at < steps.size(); at++)
            if (steps.get(at).depth() < depth) { move(at, "step"); return done(); }
        terminate();
        return done();
    }

    /**
     * 止めるものが無い（ステップ列は {@code launch} のときに全部できていて、走り続ける実行が無い）。
     * ただし黙って成功だけ返すと編集者が「一時停止中」のまま待つので、
     * いま止まっている位置で {@code stopped(pause)} を返し直す。状態は変えない。
     */
    @Override public CompletableFuture<Void> pause(PauseArguments args) {
        if (!terminated && current() != null) stop("pause");
        return done();
    }

    @Override public CompletableFuture<Void> disconnect(DisconnectArguments args) {
        // 旧スタックはここで System.exit(0) していた。テストから叩けなくなるので、
        // 生成物は「終わったことを知らせる」だけにして、プロセスを畳むのは DapMain の仕事にする。
        exit.complete(null);
        return done();
    }

    private void move(int at, String reason) {
        var previous = current();
        if (previous != null) bridge.onExitNode(previous.node());
        cursor = at;
        bridge.onEnterNode(steps.get(at).node());
        stop(reason);
    }

    private void stop(String reason) {
        handles.clear();
        var stopped = new StoppedEventArguments();
        stopped.setReason(reason);
        stopped.setThreadId(THREAD_ID);
        stopped.setAllThreadsStopped(true);
        if (client != null) client.stopped(stopped);
    }

    private void terminate() {
        if (terminated) return;
        terminated = true;
        cursor = -1;
        if (launched && !program.isBlank()) output("stdout", "Completed: " + program + "\n");
        if (client != null) {
            client.terminated(new TerminatedEventArguments());
            var exited = new ExitedEventArguments();
            exited.setExitCode(0);
            client.exited(exited);
        }
    }

    // ---- 状態の問い合わせ ---------------------------------------------------

    @Override public CompletableFuture<ThreadsResponse> threads() {
        var thread = new org.eclipse.lsp4j.debug.Thread();
        thread.setId(THREAD_ID);
        thread.setName("__GRAMMAR__ parse");
        var response = new ThreadsResponse();
        response.setThreads(new org.eclipse.lsp4j.debug.Thread[]{thread});
        return CompletableFuture.completedFuture(response);
    }

    @Override public CompletableFuture<StackTraceResponse> stackTrace(StackTraceArguments args) {
        var response = new StackTraceResponse();
        var current = current();
        if (current == null) {
            response.setStackFrames(new StackFrame[0]);
            response.setTotalFrames(0);
            return CompletableFuture.completedFuture(response);
        }
        // 根 → 現在の鎖を、DAP の並び（0 が現在）に合わせて反転する。
        var chain = Steps.chain(steps, current.index());
        var frames = new ArrayList<StackFrame>();
        for (int i = chain.size() - 1; i >= 0; i--) {
            var step = chain.get(i);
            var frame = new StackFrame();
            frame.setId(step.index() + 1);
            frame.setName(step.typeName());
            frame.setSource(source());
            var start = positions.positionOf(step.startCp());
            var end = positions.positionOf(step.endCp());
            frame.setLine(line(start.getLine()));
            frame.setColumn(column(start.getCharacter()));
            frame.setEndLine(line(end.getLine()));
            frame.setEndColumn(column(end.getCharacter()));
            frames.add(frame);
        }
        int from = Math.min(frames.size(),
            args == null || args.getStartFrame() == null ? 0 : Math.max(0, args.getStartFrame()));
        int to = args == null || args.getLevels() == null || args.getLevels() <= 0 ? frames.size()
            : Math.min(frames.size(), from + args.getLevels());
        response.setStackFrames(frames.subList(from, Math.max(from, to)).toArray(new StackFrame[0]));
        response.setTotalFrames(frames.size());
        return CompletableFuture.completedFuture(response);
    }

    @Override public CompletableFuture<ScopesResponse> scopes(ScopesArguments args) {
        var response = new ScopesResponse();
        int index = args == null ? -1 : args.getFrameId() - 1;
        if (index < 0 || index >= steps.size()) { response.setScopes(new Scope[0]); return CompletableFuture.completedFuture(response); }
        var step = steps.get(index);
        var out = new ArrayList<Scope>();
        out.add(scope("AST node", () -> nodeVariables(step)));
        var symbols = visibleSymbols(step);
        if (!symbols.isEmpty()) out.add(scope("Symbols", () -> symbols));
        response.setScopes(out.toArray(new Scope[0]));
        return CompletableFuture.completedFuture(response);
    }

    private Scope scope(String name, Supplier<List<Variable>> contents) {
        var scope = new Scope();
        scope.setName(name);
        scope.setExpensive(false);
        scope.setVariablesReference(handle(contents));
        return scope;
    }

    @Override public CompletableFuture<VariablesResponse> variables(VariablesArguments args) {
        var response = new VariablesResponse();
        int reference = args == null ? 0 : args.getVariablesReference();
        List<Variable> out = reference >= 1 && reference <= handles.size()
            ? handles.get(reference - 1).get() : List.of();
        response.setVariables(out.toArray(new Variable[0]));
        return CompletableFuture.completedFuture(response);
    }

    // ---- 値 ----------------------------------------------------------------

    private List<Variable> nodeVariables(Steps.Step step) {
        var out = new ArrayList<Variable>();
        out.add(variable("#type", step.typeName(), "astType", 0));
        out.add(variable("#text", quote(slice(step.startCp(), step.endCp())), "text", 0));
        var start = positions.positionOf(step.startCp());
        var end = positions.positionOf(step.endCp());
        out.add(variable("#range", line(start.getLine()) + ":" + column(start.getCharacter())
            + "-" + line(end.getLine()) + ":" + column(end.getCharacter()), "range", 0));
        String documentation = Meta.documentation(AstWalk.ruleIdOf(step.node()));
        if (documentation != null) out.add(variable("#doc", documentation, "text", 0));
        for (var field : AstWalk.fieldsOf(step.node())) out.add(field(field.name(), field.value()));
        for (var extra : bridge.variables(step.node())) out.add(named(extra));
        return out;
    }

    private Variable field(String name, Object value) {
        if (value instanceof java.util.Optional<?> optional)
            return optional.isEmpty() ? variable(name, "(none)", "optional", 0) : field(name, optional.get());
        if (value instanceof List<?> list) {
            if (list.isEmpty()) return variable(name, "[]", "list", 0);
            return variable(name, "List(" + list.size() + ")", "list", handle(() -> elements(list)));
        }
        if (AstWalk.isNode(value)) {
            var child = stepOf(value);
            return child == null ? variable(name, AstWalk.typeName(value), "astNode", 0)
                : variable(name, AstWalk.typeName(value), "astNode", handle(() -> nodeVariables(child)));
        }
        if (value == null) return variable(name, "null", null, 0);
        if (value instanceof String string) return variable(name, quote(string), "text", 0);
        return variable(name, String.valueOf(value), value.getClass().getSimpleName(), 0);
    }

    private List<Variable> elements(List<?> list) {
        var out = new ArrayList<Variable>();
        for (int i = 0; i < list.size(); i++) out.add(field("[" + i + "]", list.get(i)));
        return out;
    }

    /** その節点の位置までに見えている宣言（scope 観測を持つ文法だけ）。 */
    private List<Variable> visibleSymbols(Steps.Step step) {
        var out = new ArrayList<Variable>();
        if (result == null || result.scope() == null) return out;
        for (var declaration : result.scope().declarations()) {
            if (declaration.offsetCp() > step.startCp()) continue;
            var at = positions.positionOf(declaration.offsetCp());
            out.add(variable(declaration.name(), "declared at " + line(at.getLine()) + ":" + column(at.getCharacter()),
                "declaration", 0));
        }
        return out;
    }

    private Variable named(EvaluatorBridge.NamedValue value) {
        var children = value.children();
        return variable(value.name(), value.value(), value.type(),
            children.isEmpty() ? 0 : handle(() -> children.stream().map(this::named).toList()));
    }

    private static Variable variable(String name, String value, String type, int reference) {
        var out = new Variable();
        out.setName(name);
        out.setValue(truncate(value));
        out.setType(type);
        out.setVariablesReference(reference);
        return out;
    }

    private int handle(Supplier<List<Variable>> contents) {
        handles.add(contents);
        return handles.size();
    }

    private Steps.Step stepOf(Object node) {
        for (var step : steps) if (step.node() == node) return step;
        return null;
    }

    // ---- 拡張点（案 b）------------------------------------------------------

    @Override public CompletableFuture<EvaluateResponse> evaluate(EvaluateArguments args) {
        var step = args == null || args.getFrameId() == null ? current() : stepAt(args.getFrameId() - 1);
        var answer = bridge.evaluate(args == null ? "" : args.getExpression(),
            step == null ? null : step.node(), args == null ? null : args.getContext());
        if (answer.isEmpty()) return unsupported("evaluate");
        var response = new EvaluateResponse();
        response.setResult(truncate(answer.get().result()));
        response.setType(answer.get().type());
        response.setVariablesReference(answer.get().children().isEmpty() ? 0
            : handle(() -> answer.get().children().stream().map(this::named).toList()));
        return CompletableFuture.completedFuture(response);
    }

    @Override public CompletableFuture<SetVariableResponse> setVariable(SetVariableArguments args) {
        var step = current();
        var answer = bridge.setVariable(args == null ? null : String.valueOf(args.getVariablesReference()),
            args == null ? null : args.getName(), args == null ? null : args.getValue(),
            step == null ? null : step.node());
        if (answer.isEmpty()) return unsupported("setVariable");
        var response = new SetVariableResponse();
        response.setValue(truncate(answer.get().result()));
        response.setType(answer.get().type());
        response.setVariablesReference(0);
        return CompletableFuture.completedFuture(response);
    }

    private static <T> CompletableFuture<T> unsupported(String request) {
        return CompletableFuture.failedFuture(new ResponseErrorException(new ResponseError(
            ResponseErrorCode.RequestFailed,
            request + " は文法デバッガでは未対応です（値を計算する評価器を持っていません）。"
                + "言語の評価器を差し込むには EvaluatorBridge を実装し、DebugAdapter に渡してください"
                + "（生成された EvaluatorBridge.java の Javadoc に例があります）。", null)));
    }

    // ---- 小道具 ------------------------------------------------------------

    private Steps.Step current() { return stepAt(cursor); }

    private Steps.Step stepAt(int index) { return index < 0 || index >= steps.size() ? null : steps.get(index); }

    private int lineOf(int offsetCp) { return positions.positionOf(offsetCp).getLine(); }

    private int line(int zeroBased) { return linesStartAt1 ? zeroBased + 1 : zeroBased; }

    private int column(int zeroBased) { return columnsStartAt1 ? zeroBased + 1 : zeroBased; }

    private Source source() {
        var out = new Source();
        out.setPath(program);
        if (!program.isBlank()) {
            Path path = Path.of(program).getFileName();
            out.setName(path == null ? program : path.toString());
        }
        return out;
    }

    /** code point 半開区間の本文。 */
    private String slice(int startCp, int endCp) {
        int from = charIndex(Math.max(0, startCp));
        int to = charIndex(Math.max(0, endCp));
        return text.substring(from, Math.max(from, to));
    }

    /** code point 位置 → UTF-16 の char index（範囲外は末尾へ丸める）。 */
    private int charIndex(int offsetCp) {
        int index = 0;
        for (int remaining = offsetCp; remaining > 0 && index < text.length(); remaining--)
            index += Character.charCount(text.codePointAt(index));
        return Math.min(index, text.length());
    }

    private void output(String category, String message) {
        if (client == null) return;
        var event = new OutputEventArguments();
        event.setCategory(category);
        event.setOutput(message);
        client.output(event);
    }

    private static CompletableFuture<Void> done() { return CompletableFuture.completedFuture(null); }

    private static String string(Object value) { return value == null ? "" : String.valueOf(value); }

    private static String quote(String value) { return "\"" + value.replace("\\", "\\\\").replace("\"", "\\\"") + "\""; }

    private static String truncate(String value) {
        if (value == null) return null;
        String flat = value.replace("\r\n", " ").replace('\n', ' ').replace('\r', ' ');
        return flat.length() <= VALUE_LIMIT ? flat : flat.substring(0, VALUE_LIMIT) + "...";
    }
}
