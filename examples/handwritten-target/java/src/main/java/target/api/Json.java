package target.api;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import target.operators.OperatorsAST;
import target.power.PowerAST;

/** JDK のみ。固定した field 順で canonical AST と結果を出力する。 */
public final class Json {
    private Json() {}

    public static String canonical(ParseResult<?> result) {
        return write(result.ast().map(node -> node(node, result.nodeSpans())).orElse(null));
    }

    public static Map<String, Object> object(Object... pairs) {
        var map = new LinkedHashMap<String, Object>();
        for (int i = 0; i < pairs.length; i += 2) map.put((String) pairs[i], pairs[i + 1]);
        return map;
    }

    public static Object node(Object node, Map<?, Span> spans) {
        Span span = spans.get(node);
        if (span == null) throw new IllegalArgumentException("Missing node span");
        Map<String, Object> fields;
        String type;
        if (node instanceof OperatorsAST.Number number) {
            type = "Number";
            fields = object("value", number.value());
        } else if (node instanceof OperatorsAST.Binary binary) {
            type = "Binary";
            fields = object("left", node(binary.left(), spans), "op", binary.op(),
                "right", binary.right().stream().map(right -> node(right, spans)).toList());
        } else if (node instanceof PowerAST.Power power) {
            type = "Power";
            fields = object("left", power.left(), "op", power.op(),
                "right", power.right().stream().map(right -> node(right, spans)).toList());
        } else throw new IllegalArgumentException("Unknown AST: " + node.getClass());
        return object("type", type, "span", List.of(span.start(), span.end()), "fields", fields);
    }

    public static Object result(String input, ParseResult<?> result, MappingException mapping) {
        var captures = new ArrayList<Object>();
        for (var capture : result.captures()) captures.add(object("siteId", capture.siteId(),
            "name", capture.name(), "span", List.of(capture.span().start(), capture.span().end())));
        Diagnostic d = result.diagnostics();
        return object("input", input, "ok", result.ok(), "consumedCp", result.consumedCp(),
            "matchedCp", result.matchedCp(), "mappingError", mapping != null,
            "ast", result.ast().map(node -> node(node, result.nodeSpans())).orElse(null),
            "captures", captures, "diagnostics", object("farthestCp", d.farthestCp(),
                "expected", d.expected(), "deepestRule", d.deepestRule(), "ruleStack", d.ruleStack()));
    }

    public static String write(Object value) {
        var out = new StringBuilder();
        append(out, value);
        return out.toString();
    }

    private static void append(StringBuilder out, Object value) {
        if (value == null) out.append("null");
        else if (value instanceof String text) string(out, text);
        else if (value instanceof Boolean || value instanceof java.lang.Number) out.append(value);
        else if (value instanceof Map<?, ?> map) {
            out.append('{');
            boolean first = true;
            for (var entry : map.entrySet()) {
                if (!first) out.append(',');
                first = false;
                string(out, (String) entry.getKey()); out.append(':'); append(out, entry.getValue());
            }
            out.append('}');
        } else if (value instanceof Iterable<?> list) {
            out.append('[');
            boolean first = true;
            for (var item : list) {
                if (!first) out.append(',');
                first = false;
                append(out, item);
            }
            out.append(']');
        } else throw new IllegalArgumentException("Unsupported JSON value: " + value.getClass());
    }

    private static void string(StringBuilder out, String text) {
        out.append('"');
        for (int i = 0; i < text.length(); i++) {
            char ch = text.charAt(i);
            switch (ch) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                case '\b' -> out.append("\\b");
                case '\f' -> out.append("\\f");
                default -> {
                    if (ch < 0x20 || Character.isSurrogate(ch)) {
                        out.append("\\u");
                        for (int shift = 12; shift >= 0; shift -= 4)
                            out.append("0123456789abcdef".charAt(ch >> shift & 15));
                    } else out.append(ch);
                }
            }
        }
        out.append('"');
    }
}
