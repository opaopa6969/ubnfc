package generated.api;
import java.lang.reflect.InvocationTargetException;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
/** Canonical AST field order follows the generated record declaration. */
public final class Json {
    private Json() {}
    public static String canonical(ParseResult<?> result) { return write(value(result.ast(), result.nodeSpans())); }
    private static Object value(Object object, Map<Object, Span> spans) {
        if (object instanceof Optional<?> optional) return value(optional.orElse(null), spans);
        if (object instanceof List<?> list) return list.stream().map(v -> value(v, spans)).toList();
        if (object instanceof Enum<?> e) return e.name();
        if (object == null || !object.getClass().isRecord()) return object;
        Span span = spans.get(object);
        if (span == null) throw new IllegalArgumentException("Missing node span");
        var fields = new LinkedHashMap<String, Object>();
        try {
            for (var component : object.getClass().getRecordComponents())
                fields.put(component.getName(), value(component.getAccessor().invoke(object), spans));
        } catch (IllegalAccessException | InvocationTargetException error) { throw new IllegalStateException(error); }
        var node = new LinkedHashMap<String, Object>();
        node.put("type", object.getClass().getSimpleName()); node.put("span", List.of(span.start(), span.end())); node.put("fields", fields);
        return node;
    }
    public static String write(Object value) {
        if (value == null) return "null";
        if (value instanceof String text) {
            var out = new StringBuilder("\"");
            for (int i = 0; i < text.length(); i++) {
                char ch = text.charAt(i);
                switch (ch) {
                    case '"' -> out.append("\\\""); case '\\' -> out.append("\\\\");
                    case '\n' -> out.append("\\n"); case '\r' -> out.append("\\r"); case '\t' -> out.append("\\t");
                    default -> { if (ch < 32) out.append(String.format("\\u%04x", (int) ch)); else out.append(ch); }
                }
            }
            return out.append('"').toString();
        }
        if (value instanceof Boolean || value instanceof Number) return value.toString();
        if (value instanceof Map<?, ?> map) return "{" + String.join(",", map.entrySet().stream()
            .map(e -> write(e.getKey()) + ":" + write(e.getValue())).toList()) + "}";
        if (value instanceof List<?> list) return "[" + String.join(",", list.stream().map(Json::write).toList()) + "]";
        throw new IllegalArgumentException("Not JSON: " + value.getClass());
    }
}
