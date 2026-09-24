package org.ubnfc.p4.lsp.dap;

import java.io.PipedInputStream;
import java.io.PipedOutputStream;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import org.eclipse.lsp4j.debug.ConfigurationDoneArguments;
import org.eclipse.lsp4j.debug.ContinueArguments;
import org.eclipse.lsp4j.debug.DisconnectArguments;
import org.eclipse.lsp4j.debug.EvaluateArguments;
import org.eclipse.lsp4j.debug.ExitedEventArguments;
import org.eclipse.lsp4j.debug.InitializeRequestArguments;
import org.eclipse.lsp4j.debug.NextArguments;
import org.eclipse.lsp4j.debug.OutputEventArguments;
import org.eclipse.lsp4j.debug.ScopesArguments;
import org.eclipse.lsp4j.debug.SetBreakpointsArguments;
import org.eclipse.lsp4j.debug.SetVariableArguments;
import org.eclipse.lsp4j.debug.Source;
import org.eclipse.lsp4j.debug.SourceBreakpoint;
import org.eclipse.lsp4j.debug.StackTraceArguments;
import org.eclipse.lsp4j.debug.StepInArguments;
import org.eclipse.lsp4j.debug.StepOutArguments;
import org.eclipse.lsp4j.debug.StoppedEventArguments;
import org.eclipse.lsp4j.debug.TerminatedEventArguments;
import org.eclipse.lsp4j.debug.VariablesArguments;
import org.eclipse.lsp4j.debug.launch.DSPLauncher;
import org.eclipse.lsp4j.debug.services.IDebugProtocolClient;
import org.eclipse.lsp4j.debug.services.IDebugProtocolServer;
import org.ubnfc.p4.P4Scanners;

/**
 * 台本どおりに動く DAP クライアント。生成された {@link DebugAdapter} と実物の lsp4j launcher を
 * パイプでつないでいるので、LSP の {@code ScriptedClient} と同じく<b>本物の JSON-RPC を通る</b>
 * （アダプタのメソッドを直接呼ぶのとは違う）。
 *
 * <p>イベントは要求の応答より先に届く（アダプタは応答を返す前に {@code stopped} などを書き、
 * lsp4j は受信順に 1 つずつ処理する）ので、要求の {@code get()} が返った時点で
 * そこまでのイベントは {@link #events()} に入っている。
 */
final class ScriptedDebugClient implements AutoCloseable {
    private static final long TIMEOUT = 120;

    private final IDebugProtocolServer proxy;
    private final List<String> events = java.util.Collections.synchronizedList(new ArrayList<>());
    private volatile String lastStopReason;
    private volatile boolean terminated;

    private final class Collector implements IDebugProtocolClient {
        @Override public void initialized() { events.add("initialized"); }
        @Override public void stopped(StoppedEventArguments args) {
            lastStopReason = args.getReason();
            events.add("stopped:" + args.getReason());
        }
        @Override public void terminated(TerminatedEventArguments args) { terminated = true; events.add("terminated"); }
        @Override public void exited(ExitedEventArguments args) { events.add("exited:" + args.getExitCode()); }
        @Override public void output(OutputEventArguments args) {
            events.add("output:" + args.getCategory() + ":" + args.getOutput().strip());
        }
    }

    ScriptedDebugClient(Path program) throws Exception {
        var toServer = new PipedInputStream();
        var clientOut = new PipedOutputStream(toServer);
        var toClient = new PipedInputStream();
        var serverOut = new PipedOutputStream(toClient);
        var adapter = new DebugAdapter(P4Scanners.ALL);
        var serverLauncher = DSPLauncher.createServerLauncher(adapter, toServer, serverOut);
        adapter.connect(serverLauncher.getRemoteProxy());
        var clientLauncher = DSPLauncher.createClientLauncher(new Collector(), toClient, clientOut);
        serverLauncher.startListening();
        clientLauncher.startListening();
        this.proxy = clientLauncher.getRemoteProxy();
    }

    List<String> events() { return List.copyOf(events); }
    String lastStopReason() { return lastStopReason; }
    boolean terminated() { return terminated; }

    org.eclipse.lsp4j.debug.Capabilities initialize() throws Exception {
        var args = new InitializeRequestArguments();
        args.setAdapterID("ubnfc-grammar-debugger");
        args.setLinesStartAt1(true);
        args.setColumnsStartAt1(true);
        return proxy.initialize(args).get(TIMEOUT, TimeUnit.SECONDS);
    }

    void launch(Path program, boolean stopOnEntry) throws Exception {
        proxy.launch(Map.of("program", program.toString(), "stopOnEntry", stopOnEntry)).get(TIMEOUT, TimeUnit.SECONDS);
    }

    org.eclipse.lsp4j.debug.Breakpoint[] setBreakpoints(Path program, List<Integer> lines) throws Exception {
        var args = new SetBreakpointsArguments();
        var source = new Source();
        source.setPath(program.toString());
        source.setName(program.getFileName().toString());
        args.setSource(source);
        var requested = new SourceBreakpoint[lines.size()];
        for (int i = 0; i < lines.size(); i++) {
            requested[i] = new SourceBreakpoint();
            requested[i].setLine(lines.get(i));
        }
        args.setBreakpoints(requested);
        return proxy.setBreakpoints(args).get(TIMEOUT, TimeUnit.SECONDS).getBreakpoints();
    }

    /** 次の要求で止まらなかったことを golden で見分けられるよう、直前の理由を消す。 */
    private void armStop() { lastStopReason = null; }

    void configurationDone() throws Exception {
        armStop();
        proxy.configurationDone(new ConfigurationDoneArguments()).get(TIMEOUT, TimeUnit.SECONDS);
    }

    org.eclipse.lsp4j.debug.Thread[] threads() throws Exception {
        return proxy.threads().get(TIMEOUT, TimeUnit.SECONDS).getThreads();
    }

    void continue_() throws Exception {
        armStop();
        var args = new ContinueArguments();
        args.setThreadId(1);
        proxy.continue_(args).get(TIMEOUT, TimeUnit.SECONDS);
    }

    void next() throws Exception {
        armStop();
        var args = new NextArguments();
        args.setThreadId(1);
        proxy.next(args).get(TIMEOUT, TimeUnit.SECONDS);
    }

    void stepIn() throws Exception {
        armStop();
        var args = new StepInArguments();
        args.setThreadId(1);
        proxy.stepIn(args).get(TIMEOUT, TimeUnit.SECONDS);
    }

    void stepOut() throws Exception {
        armStop();
        var args = new StepOutArguments();
        args.setThreadId(1);
        proxy.stepOut(args).get(TIMEOUT, TimeUnit.SECONDS);
    }

    org.eclipse.lsp4j.debug.StackFrame[] stackTrace() throws Exception {
        var args = new StackTraceArguments();
        args.setThreadId(1);
        return proxy.stackTrace(args).get(TIMEOUT, TimeUnit.SECONDS).getStackFrames();
    }

    org.eclipse.lsp4j.debug.Scope[] scopes(int frameId) throws Exception {
        var args = new ScopesArguments();
        args.setFrameId(frameId);
        return proxy.scopes(args).get(TIMEOUT, TimeUnit.SECONDS).getScopes();
    }

    org.eclipse.lsp4j.debug.Variable[] variables(int reference) throws Exception {
        var args = new VariablesArguments();
        args.setVariablesReference(reference);
        return proxy.variables(args).get(TIMEOUT, TimeUnit.SECONDS).getVariables();
    }

    /** {@code evaluate} は文法デバッガでは未対応。返るエラー文面を golden に残す。 */
    String evaluate(String expression) {
        var args = new EvaluateArguments();
        args.setExpression(expression);
        args.setContext("repl");
        try {
            var response = proxy.evaluate(args).get(TIMEOUT, TimeUnit.SECONDS);
            return "ok: " + response.getResult();
        } catch (Exception error) {
            return "error: " + rootMessage(error);
        }
    }

    String setVariable(String name, String value) {
        var args = new SetVariableArguments();
        args.setVariablesReference(1);
        args.setName(name);
        args.setValue(value);
        try {
            var response = proxy.setVariable(args).get(TIMEOUT, TimeUnit.SECONDS);
            return "ok: " + response.getValue();
        } catch (Exception error) {
            return "error: " + rootMessage(error);
        }
    }

    private static String rootMessage(Throwable error) {
        Throwable cause = error;
        while (cause.getCause() != null) cause = cause.getCause();
        String message = cause.getMessage();
        return message == null ? cause.getClass().getSimpleName() : message;
    }

    @Override public void close() throws Exception {
        proxy.disconnect(new DisconnectArguments()).get(TIMEOUT, TimeUnit.SECONDS);
    }
}
