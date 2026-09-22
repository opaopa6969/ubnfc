package target.driver;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import target.api.*;
import target.operators.OperatorsAST;
import target.operators.OperatorsParser;
import target.power.PowerAST;
import target.power.PowerParser;

/** JDK のみで動く harness JSON Lines driver。stdout は応答行専用。 */
public final class Driver {
    private Driver() {}

    public static void main(String[] args) throws Exception {
        var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        var output = new PrintWriter(new OutputStreamWriter(System.out, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            Object id = null;
            Object response;
            try {
                Object decoded = JsonReader.read(line);
                if (!(decoded instanceof Map<?, ?> request)) throw new IllegalArgumentException("object が必要です");
                id = request.get("id");
                if (!request.containsKey("id")) throw new IllegalArgumentException("id が必要です");
                response = execute(request);
            } catch (RuntimeException error) {
                response = Json.object("id", id, "error", error.toString());
            }
            output.println(Json.write(response));
            output.flush();
            if (output.checkError()) throw new java.io.IOException("stdout の書込みに失敗しました");
        }
    }

    private static String string(Map<?, ?> request, String key) {
        if (!(request.get(key) instanceof String value)) throw new IllegalArgumentException(key + " は文字列が必要です");
        return value;
    }
    private static boolean flag(Map<?, ?> options, String key) {
        if (!(options.get(key) instanceof Boolean value)) throw new IllegalArgumentException(key + " は bool が必要です");
        return value;
    }
    private static ParseResult<?> parse(String grammar, String input, ParseOptions options) {
        return grammar.equals("Operators") ? OperatorsParser.parse(input, options) : PowerParser.parse(input, options);
    }
    private static Object execute(Map<?, ?> request) {
        String grammar = string(request, "grammar");
        String input = string(request, "input");
        String root = switch (grammar) {
            case "Operators" -> "Expression";
            case "Power" -> "Expr";
            default -> throw new IllegalArgumentException("未対応の grammar: " + grammar);
        };
        if (request.get("entry") != null && !root.equals(request.get("entry")))
            throw new IllegalArgumentException("未対応の entry: " + request.get("entry"));
        if (!(request.get("options") instanceof Map<?, ?> options)) throw new IllegalArgumentException("options が必要です");
        boolean lexical = flag(options, "lexical"), scope = flag(options, "scope");
        var prefix = parse(grammar, input, new ParseOptions(false, false));
        ParseResult<?> result;
        String mapping = null;
        try {
            result = parse(grammar, input, ParseOptions.DEFAULT);
        } catch (MappingException error) {
            result = error.syntax();
            mapping = error.getMessage();
        }
        Object ast = result.ast().isPresent() ? Json.node(result.ast().get(), result.nodeSpans()) : null;
        Double value = null;
        if (result.ast().isPresent()) {
            Object node = result.ast().get();
            value = node instanceof OperatorsAST operators ? Evaluator.evaluate(operators) : Evaluator.evaluate((PowerAST) node);
            if (!Double.isFinite(value)) throw new IllegalArgumentException("評価値が有限の JSON 数値ではありません");
        }
        var diagnostics = new ArrayList<Object>();
        Diagnostic diagnostic = result.diagnostics();
        if (!diagnostic.expected().isEmpty()) diagnostics.add(Json.object("kind", "syntax",
            "offset", diagnostic.farthestCp(), "expected", diagnostic.expected()));
        var response = Json.object("id", request.get("id"), "accepted", prefix.ok(), "allConsumed", result.ok(),
            "consumed", prefix.consumedCp(), "matched", prefix.matchedCp(), "ast", ast,
            "diagnostics", diagnostics, "mappingError", mapping, "value", value);
        if (lexical) {
            var captures = new ArrayList<Object>();
            for (Capture capture : result.captures()) {
                int start = input.offsetByCodePoints(0, capture.span().start());
                int end = input.offsetByCodePoints(0, capture.span().end());
                captures.add(Json.object("name", capture.name(), "span", List.of(capture.span().start(), capture.span().end()),
                    "text", input.substring(start, end)));
            }
            response.put("captures", captures);
        }
        if (scope) { response.put("declarations", List.of()); response.put("references", List.of()); }
        return response;
    }
}
