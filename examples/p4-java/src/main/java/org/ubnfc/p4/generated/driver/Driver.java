package org.ubnfc.p4.generated.driver;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.regex.Pattern;
import org.ubnfc.p4.generated.TinyExpressionP4AST;
import org.ubnfc.p4.generated.TinyExpressionP4Parser;
import org.ubnfc.p4.generated.api.*;

/** Standalone UTF-8 JSON Lines harness driver. stdout contains only responses. */
public final class Driver {
    private Driver() {}
    private static final Set<Class<?>> OPERATOR_TYPES = Set.of(TinyExpressionP4AST.BinaryExpr.class);
    private static final Pattern NUMBER = Pattern.compile("[+-]?(?:[0-9]+(?:\\.[0-9]*)?|\\.[0-9]+)(?:[eE][+-]?[0-9]+)?");

    public static void main(String[] args) throws java.io.IOException {
        var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        var output = new PrintWriter(new OutputStreamWriter(System.out, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            Object id = null;
            String response;
            try {
                Map<String, Object> request = Json.object(Json.read(line));
                id = request.get("id");
                if (!request.containsKey("id")) throw new IllegalArgumentException("Missing id");
                response = Json.write(execute(request));
            } catch (RuntimeException error) {
                response = Json.write(object("id", id, "error", error.toString()));
            }
            output.println(response);
            output.flush();
            if (output.checkError()) throw new java.io.IOException("Cannot write stdout");
        }
    }
    private static String string(Map<?, ?> request, String key) {
        if (request.get(key) instanceof String value) return value;
        throw new IllegalArgumentException(key + " must be a string");
    }
    private static boolean flag(Map<?, ?> options, String key) {
        if (options.get(key) instanceof Boolean value) return value;
        throw new IllegalArgumentException(key + " must be a boolean");
    }
    private static Map<String, Object> object(Object... fields) {
        var map = new LinkedHashMap<String, Object>();
        for (int i = 0; i < fields.length; i += 2) map.put((String) fields[i], fields[i + 1]);
        return map;
    }
    private static Object execute(Map<String, Object> request) {
        String grammar = string(request, "grammar"), input = string(request, "input");
        if (!request.containsKey("entry")) throw new IllegalArgumentException("Missing entry");
        String entry = request.get("entry") == null ? null : string(request, "entry");
        Map<String, Object> options = Json.object(request.get("options"));
        boolean lexical = flag(options, "lexical"), scope = flag(options, "scope");
        var prefix = TinyExpressionP4Parser.parseEntry(grammar, entry, input, new ParseOptions(false, false));
        ParseResult<?> result;
        String mapping = null;
        try {
            result = TinyExpressionP4Parser.parseEntry(grammar, entry, input, new ParseOptions(true, true, lexical, true, Map.of()));
        } catch (MappingException error) {
            result = error.syntax();
            mapping = error.getMessage();
        }
        Object ast = Json.canonicalValue(result);
        Double value = null;
        if (result.ast().isPresent() && OPERATOR_TYPES.contains(result.ast().get().getClass()))
            value = numeric(ast);
        var diagnostics = new ArrayList<Object>();
        for (var recovery : result.recoveries()) {
            Diagnostic d = recovery.diagnostic();
            diagnostics.add(object("kind", d.kind(), "offset", d.offset(), "length", d.length(), "severity", d.severity(),
                "origin", d.origin(), "ruleId", recovery.ruleId(), "expected", d.expected(),
                "farthestOffset", d.farthestCp(), "farthestExpected", d.farthestExpected()));
        }
        Diagnostic diagnostic = result.diagnostics();
        if (!diagnostic.kind().equals("none") || result.recoveries().isEmpty() && !diagnostic.expected().isEmpty())
            diagnostics.add(object("kind", diagnostic.kind().equals("none") ? "syntax" : diagnostic.kind(),
                "offset", diagnostic.offset() < 0 ? diagnostic.farthestCp() : diagnostic.offset(),
                "expected", diagnostic.expected(), "farthestOffset", diagnostic.farthestCp(), "farthestExpected", diagnostic.farthestExpected()));
        for (var diagnosticEvent : result.scope().diagnostics())
            diagnostics.add(object("kind", "scope", "offset", diagnosticEvent.offset(), "expected", List.of(),
                "message", diagnosticEvent.msg(), "length", diagnosticEvent.len(), "severity", diagnosticEvent.severity().name()));
        var response = object("id", request.get("id"), "accepted", prefix.ok(), "allConsumed", result.ok(),
            "consumed", prefix.consumedCp(), "matched", prefix.matchedCp(), "ast", ast,
            "diagnostics", diagnostics, "mappingError", mapping, "value", value);
        response.put("recoveries", result.recoveries().stream().map(r -> object("ruleId", r.ruleId(), "mode", r.mode(),
            "span", List.of(r.span().start(), r.span().end()), "syncPattern", r.syncPattern(),
            "skippedSpan", List.of(r.skippedSpan().start(), r.skippedSpan().end()),
            "syncSpan", r.syncSpan() == null ? null : List.of(r.syncSpan().start(), r.syncSpan().end()),
            "captureOccurrences", r.captureOccurrences())).toList());
        if (lexical) {
            var captures = new ArrayList<Object>();
            for (Capture capture : result.captures()) {
                int start = input.offsetByCodePoints(0, capture.span().start());
                int end = input.offsetByCodePoints(0, capture.span().end());
                captures.add(object("name", capture.name(), "span", List.of(capture.span().start(), capture.span().end()),
                    "text", input.substring(start, end)));
            }
            response.put("captures", captures);
        }
        if (scope) {
            response.put("declarations", result.scope().declarations().stream()
                .map(d -> object("name", d.name(), "sourceOffset", d.offsetCp())).toList());
            response.put("references", result.scope().references().stream()
                .map(r -> object("name", r.name(), "offset", r.offset(), "length", r.len())).toList());
        }
        return response;
    }
    /** Fold only numeric leaves and the canonical left/op/right lists, never source input. */
    private static Double numeric(Object value) {
        if (value instanceof Number number) return finite(number.doubleValue());
        if (value instanceof String text) {
            var matcher = NUMBER.matcher(text.strip());
            if (!matcher.lookingAt()) return null;
            String remainder = text.strip().substring(matcher.end()).stripLeading();
            // Captured text can include trailing trivia. Do not accept arbitrary suffixes.
            while (!remainder.isEmpty()) {
                if (remainder.startsWith("//")) {
                    int end = remainder.indexOf('\n');
                    remainder = end < 0 ? "" : remainder.substring(end + 1).stripLeading();
                } else if (remainder.startsWith("/*")) {
                    int end = remainder.indexOf("*/", 2);
                    if (end < 0) return null;
                    remainder = remainder.substring(end + 2).stripLeading();
                } else return null;
            }
            return finite(Double.parseDouble(matcher.group()));
        }
        if (!(value instanceof Map<?, ?> node) || !(node.get("fields") instanceof Map<?, ?> fields)) return null;
        if (fields.size() == 1 && fields.containsKey("value")) return numeric(fields.get("value"));
        if (fields.size() != 3 || !fields.containsKey("left") || !(fields.get("op") instanceof List<?> ops)
            || !(fields.get("right") instanceof List<?> rights)) return null;
        // assocLists leafFallback stores its numeric leaf in the single op slot.
        if (fields.get("left") == null && rights.isEmpty() && ops.size() == 1) return numeric(ops.getFirst());
        if (ops.size() != rights.size()) return null;
        Double left = numeric(fields.get("left"));
        if (left == null) return null;
        for (int i = 0; i < ops.size(); i++) {
            Double right = numeric(rights.get(i));
            if (right == null || !(ops.get(i) instanceof String op)) return null;
            left = switch (op.strip()) {
                case "+" -> left + right; case "-" -> left - right;
                case "*" -> left * right; case "/" -> left / right;
                case "^" -> Math.pow(left, right);
                default -> null;
            };
            if (left == null) return null;
            finite(left);
        }
        return left;
    }
    private static double finite(double value) {
        if (!Double.isFinite(value)) throw new IllegalArgumentException("Non-finite evaluation value");
        return value;
    }
}
