package org.ubnfc.p4.lsp.dap;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * 1 セッション分の台本を流して、結果を決定的な JSON 文字列にする（golden の本体）。
 *
 * <p>絶対パスは環境で変わるので、{@code program} は必ずファイル名に畳む。
 * variables は入れ子を<b>深さ 2 まで</b>展開する（それ以上は変化に弱いだけで情報が増えない）。
 */
final class DapSession {
    private static final int VARIABLE_DEPTH = 2;

    private final Path program;
    private final List<Integer> breakpointLines;
    private final StringBuilder out = new StringBuilder();

    DapSession(Path program, List<Integer> breakpointLines) {
        this.program = program;
        this.breakpointLines = List.copyOf(breakpointLines);
    }

    String run() throws Exception {
        try (var client = new ScriptedDebugClient(program)) {
            var capabilities = client.initialize();
            out.append("{\n  \"capabilities\": {")
               .append("\"supportsConfigurationDoneRequest\": ").append(capabilities.getSupportsConfigurationDoneRequest())
               .append(", \"supportsEvaluateForHovers\": ").append(capabilities.getSupportsEvaluateForHovers())
               .append(", \"supportsSetVariable\": ").append(capabilities.getSupportsSetVariable())
               .append(", \"supportsStepBack\": ").append(capabilities.getSupportsStepBack())
               .append("},\n");

            client.launch(program, true);
            out.append("  \"breakpoints\": [");
            var breakpoints = client.setBreakpoints(program, breakpointLines);
            for (int i = 0; i < breakpoints.length; i++) {
                var breakpoint = breakpoints[i];
                out.append(i == 0 ? "" : ", ").append("{\"line\": ").append(breakpoint.getLine())
                   .append(", \"verified\": ").append(breakpoint.isVerified())
                   .append(", \"message\": ").append(quote(breakpoint.getMessage())).append('}');
            }
            out.append("],\n");

            client.configurationDone();
            out.append("  \"threads\": [");
            var threads = client.threads();
            for (int i = 0; i < threads.length; i++)
                out.append(i == 0 ? "" : ", ").append("{\"id\": ").append(threads[i].getId())
                   .append(", \"name\": ").append(quote(threads[i].getName())).append('}');
            out.append("],\n  \"stops\": [\n");

            var stops = new ArrayList<String>();
            stops.add(stop("configurationDone", client));
            for (int i = 0; i < 2; i++) { client.stepIn(); stops.add(stop("stepIn", client)); }
            for (int i = 0; i < 3; i++) { client.next(); stops.add(stop("next", client)); }
            client.stepOut();
            stops.add(stop("stepOut", client));
            client.continue_();
            stops.add(stop("continue", client));
            for (int i = 0; i < stops.size(); i++)
                out.append(stops.get(i)).append(i + 1 < stops.size() ? ",\n" : "\n");
            out.append("  ],\n");

            out.append("  \"evaluate\": ").append(quote(normalize(client.evaluate("1 + 1")))).append(",\n");
            out.append("  \"setVariable\": ").append(quote(normalize(client.setVariable("x", "1")))).append(",\n");
            out.append("  \"events\": [");
            var events = client.events();
            for (int i = 0; i < events.size(); i++)
                out.append(i == 0 ? "" : ", ").append(quote(normalize(events.get(i))));
            out.append("]\n}\n");
        }
        return out.toString();
    }

    /** 直前の要求で止まったなら、そこでの stackTrace / scopes / variables を書き出す。 */
    private String stop(String after, ScriptedDebugClient client) throws Exception {
        var text = new StringBuilder("    {\"after\": ").append(quote(after))
            .append(", \"reason\": ").append(quote(client.lastStopReason()))
            .append(", \"terminated\": ").append(client.terminated());
        var frames = client.stackTrace();
        text.append(", \"stackTrace\": [");
        for (int i = 0; i < frames.length; i++) {
            var frame = frames[i];
            text.append(i == 0 ? "" : ", ").append("{\"name\": ").append(quote(frame.getName()))
                .append(", \"line\": ").append(frame.getLine()).append(", \"column\": ").append(frame.getColumn())
                .append(", \"endLine\": ").append(frame.getEndLine()).append(", \"endColumn\": ").append(frame.getEndColumn())
                .append(", \"source\": ").append(quote(frame.getSource() == null ? null : frame.getSource().getName()))
                .append('}');
        }
        text.append(']');
        if (frames.length > 0) {
            text.append(", \"scopes\": [");
            var scopes = client.scopes(frames[0].getId());
            for (int i = 0; i < scopes.length; i++) {
                text.append(i == 0 ? "" : ", ").append("{\"name\": ").append(quote(scopes[i].getName()))
                    .append(", \"variables\": ")
                    .append(variables(client, scopes[i].getVariablesReference(), VARIABLE_DEPTH)).append('}');
            }
            text.append(']');
        }
        return text.append('}').toString();
    }

    private String variables(ScriptedDebugClient client, int reference, int depth) throws Exception {
        if (reference == 0 || depth == 0) return "[]";
        var items = new ArrayList<String>();
        for (var variable : client.variables(reference)) {
            var text = new StringBuilder("{\"name\": ").append(quote(variable.getName()))
                .append(", \"value\": ").append(quote(normalize(variable.getValue())))
                .append(", \"type\": ").append(quote(variable.getType()));
            if (variable.getVariablesReference() != 0 && depth > 1)
                text.append(", \"children\": ").append(variables(client, variable.getVariablesReference(), depth - 1));
            items.add(text.append('}').toString());
        }
        return "[" + String.join(", ", items) + "]";
    }

    /** 絶対パスを畳む（golden を機械に依存させない）。 */
    private String normalize(String value) {
        if (value == null) return null;
        return value.replace(program.toAbsolutePath().toString(), "<program>")
            .replace(program.toString(), "<program>");
    }

    private static String quote(String value) {
        if (value == null) return "null";
        var out = new StringBuilder("\"");
        value.codePoints().forEach(point -> {
            switch (point) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                default -> {
                    if (point < 0x20) out.append(String.format("\\u%04x", point));
                    else out.appendCodePoint(point);
                }
            }
        });
        return out.append('"').toString();
    }
}
