package generated.driver;

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
import generated.__AST__;
import generated.__PARSER__;
import generated.api.*;

/** Standalone UTF-8 JSON Lines harness driver. stdout contains only responses. */
public final class Driver {
    private Driver() {}
    private static final Set<Class<?>> OPERATOR_TYPES = Set.of(__OPERATOR_TYPES__);
    private static final Map<String, String> BINDINGS = __BINDINGS__;
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
        Map<String, Object> observations = request.get("observations") instanceof Map<?, ?> ? Json.object(request.get("observations")) : Map.of();
        int maxDepth = options.get("maxDepth") instanceof Number number ? new java.math.BigDecimal(number.toString()).intValueExact() : ParseOptions.DEFAULT_MAX_DEPTH;
        var prefix = __PARSER__.parseEntry(grammar, entry, input, new ParseOptions(false, false, lexical, true, Map.of(), maxDepth));
        ParseResult<?> result;
        String mapping = null, mappingKind = null, mappingMessage = null;
        try {
            result = __PARSER__.parseEntry(grammar, entry, input, new ParseOptions(true, true, lexical, true, Map.of(), maxDepth));
        } catch (MappingException error) {
            result = error.syntax();
            mapping = error.getCause() == null ? error.getMessage() : error.getCause().toString();
            mappingMessage = error.getMessage();
            mappingKind = (error.getCause() == null ? error : error.getCause()).getClass().getName();
        }
        Object ast = Json.canonicalValue(result);
        Object value;
        boolean evaluationError = false;
        try { value = evaluate(ast, observations.get("family"), result.ast().isPresent()
            && OPERATOR_TYPES.contains(result.ast().get().getClass())); }
        catch (IllegalArgumentException error) {
            if (!observations.containsKey("family")) throw error;
            value = UNOBSERVED; evaluationError = true;
        }
        if (ast == null && result.ok() && mapping == null) value = UNOBSERVED;
        var diagnostics = new ArrayList<Object>();
        for (var recovery : result.recoveries()) {
            Diagnostic d = recovery.diagnostic();
            diagnostics.add(object("kind", d.kind(), "offset", d.offset(), "length", d.length(), "severity", d.severity(),
                "origin", d.origin(), "ruleId", recovery.ruleId(), "expected", d.expected(),
                "farthestOffset", d.farthestCp(), "farthestExpected", d.farthestExpected()));
        }
        Diagnostic diagnostic = result.diagnostics();
        if ((!diagnostic.kind().equals("none") || result.recoveries().isEmpty() && !diagnostic.expected().isEmpty())
            && (diagnostic.farthestCp() >= 0 || !diagnostic.expected().isEmpty()))
            diagnostics.add(object("kind", diagnostic.kind().equals("none") ? "syntax" : diagnostic.kind(),
                "offset", diagnostic.offset() < 0 ? diagnostic.farthestCp() : diagnostic.offset(),
                "expected", diagnostic.expected(), "farthestOffset", diagnostic.farthestCp(), "farthestExpected", diagnostic.farthestExpected(), "origin", "Mapper.diagnose"));
        for (var diagnosticEvent : result.scope().diagnostics())
            diagnostics.add(object("kind", diagnosticEvent.severity().name(), "offset", diagnosticEvent.offset(), "expected", List.of(),
                "message", diagnosticEvent.msg(), "length", diagnosticEvent.len(), "severity", diagnosticEvent.severity().name(), "origin", "ScopeStore"));
        var response = object("id", request.get("id"), "accepted", prefix.ok(), "allConsumed", result.ok(),
            "consumed", prefix.consumedCp(), "matched", prefix.matchedCp(), "ast", ast,
            "diagnostics", diagnostics, "mappingError", mapping, "parsedStatus", prefix.ok() ? "succeeded" : "failed");
        if (value != UNOBSERVED) response.put("value", value);
        if (evaluationError) response.put("evaluationError", true);
        if (mappingKind != null) { response.put("mappingErrorKind", mappingKind); response.put("mappingMessage", mappingMessage); }
        var failure = prefix.nativeFailure();
        if (failure != null) response.put("nativeFailure", object("offset", failure.offset(), "consumed", failure.consumed(),
            "matched", failure.matched(), "expected", failure.expected()));
        response.put("recoveries", result.recoveries().stream().map(r -> object("ruleId", r.ruleId(), "mode", r.mode(),
            "span", List.of(r.span().start(), r.span().end()), "syncPattern", r.syncPattern(),
            "skippedSpan", List.of(r.skippedSpan().start(), r.skippedSpan().end()),
            "syncSpan", r.syncSpan() == null ? null : List.of(r.syncSpan().start(), r.syncSpan().end()),
            "captureOccurrences", r.captureOccurrences())).toList());
        if (lexical) {
            response.put("captures", captures(result, input, observations.containsKey("captureNames"), "oracle".equals(observations.get("family"))));
            response.put("prefixCaptures", captures(prefix, input, false, true));
            response.put("lexical", result.lexical().stream().map(item -> object("occurrenceId", item.occurrenceId(),
                "parentOccurrenceId", item.parentOccurrenceId(), "ruleId", item.ruleId(), "exprId", item.exprId(),
                "span", List.of(item.span().start(), item.span().end()), "completionOrder", item.completionOrder(), "token", item.token())).toList());
        }
        if (scope) {
            Map<String, Object> state = scope(result, observations);
            response.put("declarations", state.get("declarations"));
            response.put("references", state.get("references"));
            response.put("scope", state);
            response.put("scopeDepth", result.scope().depth());
            if (Boolean.TRUE.equals(observations.get("parentRollback"))) {
                var rolled = __PARSER__.parseEntryRollback(grammar, entry, input, new ParseOptions(false, false, lexical, true, Map.of(), maxDepth));
                var rollbackScope = scope(rolled, observations); rollbackScope.remove("events"); rollbackScope.remove("mode");
                response.put("rollback", object("cursor", List.of(rolled.consumedCp(), rolled.matchedCp()), "scope", rollbackScope));
            }
        }
        if (ast != null) {
            var nodes = new ArrayList<Object>();
            collectNodes(ast, nodes);
            response.put("nodes", nodes);
            var texts = new ArrayList<Object>();
            collectTexts(result.ast().orElse(null), result.nodeSpans(), texts);
            response.put("texts", texts);
            if (observations.containsKey("captureNames")) { response.put("valueSpans", List.of()); response.put("retainedValueSpans", List.of()); }
            if (observations.get("valueSpanField") instanceof String field && ast instanceof Map<?, ?> node && node.get("fields") instanceof Map<?, ?> fields) {
                Object root = result.ast().orElse(null);
                var selected = new ArrayList<Object>(values(field(root, field)));
                if (observations.get("valueSpanListField") instanceof String listField) selected.addAll(values(field(root, listField)));
                var valueSpans = new ArrayList<Object>();
                for (Object selectedValue : selected) {
                    Span span = result.nodeSpans().get(selectedValue);
                    if (span != null) valueSpans.add(List.of(span.start(), span.end()));
                }
                response.put("valueSpans", valueSpans); response.put("retainedValueSpans", valueSpans);
            }
        }
        return response;
    }
    private static List<Object> captures(ParseResult<?> result, String input, boolean ruleCompletion, boolean bindings) {
        var captures = new ArrayList<Object>();
        var selected = ruleCompletion ? result.captures().stream().sorted(java.util.Comparator.comparingLong(Capture::ruleCompletionOrder)).toList() : result.captures();
        for (Capture capture : selected) {
            var item = object("name", capture.name(), "span", List.of(capture.span().start(), capture.span().end()),
                "text", input.substring(input.offsetByCodePoints(0, capture.span().start()), input.offsetByCodePoints(0, capture.span().end())));
            if (bindings) item.put("binding", BINDINGS.get(capture.siteId()));
            captures.add(item);
        }
        return captures;
    }
    private static Map<String, Object> scope(ParseResult<?> result, Map<String, Object> observations) {
        var scope = result.scope();
        var state = object("depth", scope.depth(), "mode", scope.events().stream().map(ParseResult.ScopeEvent::mode).distinct().toList(),
            "events", scope.events().stream().filter(e -> Set.of("enter", "leave", "declare", "use").contains(e.action()))
                .map(e -> object("order", e.order(), "action", e.action().equals("use") ? "reference" : e.action(), "mode", e.mode(),
                    "name", e.name(), "offset", e.offsetCp(), "length", e.lengthCp())).toList(),
            "declarations", scope.declarations().stream().map(d -> object("name", d.name(), "sourceOffset", d.offsetCp())).toList(),
            "references", scope.references().stream().map(r -> object("name", r.name(), "offset", r.offset(), "length", r.len())).toList(),
            "diagnostics", scope.diagnostics().stream().map(d -> object("message", d.msg(), "offset", d.offset(), "length", d.len(), "severity", d.severity().name())).toList());
        if (observations.get("queries") instanceof List<?> queries) state.put("resolved", queries.stream().map(q -> {
            var declaration = scope.resolved().get(q);
            return object("query", q, "symbol", declaration == null ? null : object("name", declaration.name(), "sourceOffset", declaration.offsetCp()));
        }).toList());
        return state;
    }
    private static final Object UNOBSERVED = new Object();
    private static List<?> values(Object value) { return value == null ? List.of() : value instanceof List<?> list ? list : List.of(value); }
    private static String tagged(Object value) {
        if (value instanceof String text) return "T:" + text;
        if (value instanceof Map<?, ?> node && node.get("fields") instanceof Map<?, ?> fields) {
            Object text = fields.containsKey("text") ? fields.get("text") : fields.get("value");
            return switch (String.valueOf(node.get("type"))) {
                case "Leaf" -> "L:" + text;
                case "Other", "OtherLeaf" -> "O:" + text;
                case "Value", "Shared" -> tagged(text);
                default -> throw new IllegalArgumentException("Unsupported semantic value: " + node.get("type"));
            };
        }
        throw new IllegalArgumentException("Unsupported semantic value");
    }
    private static String tags(Object value) { return values(value).stream().map(Driver::tagged).collect(java.util.stream.Collectors.joining(",")); }
    private static Object evaluate(Object ast, Object family, boolean operator) {
        if (ast == null) return null;
        if (!(ast instanceof Map<?, ?> node) || !(node.get("fields") instanceof Map<?, ?> fields)) return UNOBSERVED;
        if ("semantic-cardinality".equals(family) && fields.containsKey("values")) return tags(fields.get("values"));
        if ("mixed-values".equals(family)) {
            if (fields.containsKey("head") && fields.containsKey("maybe") && fields.containsKey("items"))
                return tagged(fields.get("head")) + "|" + (fields.get("maybe") == null ? "-" : tagged(fields.get("maybe"))) + "|" + tags(fields.get("items"));
            if (fields.containsKey("value")) return tagged(fields.get("value"));
        }
        if ("cardinality".equals(family) && fields.containsKey("flags")) {
            double total = fields.get("head") == null ? 0 : numeric(fields.get("head"));
            for (Object item : values(fields.get("values"))) total += numeric(item);
            return total + (fields.get("tail") == null ? 0 : 1) + values(fields.get("flags")).size();
        }
        if (operator || Set.of("Number", "Literal", "Binary", "Power", "Negation", "Conditional").contains(node.get("type")) || "right-associative".equals(family) || "associative".equals(family) || "evolution".equals(family)) {
            Double value = numeric(ast); return value == null ? UNOBSERVED : value;
        }
        return UNOBSERVED;
    }
    private static Object field(Object node, String name) {
        if (node == null || !node.getClass().isRecord()) return null;
        for (var component : node.getClass().getRecordComponents()) if (component.getName().equals(name)) {
            try { Object value = component.getAccessor().invoke(node); return value instanceof java.util.Optional<?> optional ? optional.orElse(null) : value; }
            catch (ReflectiveOperationException error) { throw new IllegalStateException(error); }
        }
        return null;
    }
    private static void collectTexts(Object value, Map<Object, Span> spans, List<Object> texts) {
        if (value instanceof String text && spans.get(value) instanceof Span span) texts.add(List.of(span.start(), span.end(), text));
        else if (value instanceof java.util.Optional<?> optional) optional.ifPresent(v -> collectTexts(v, spans, texts));
        else if (value instanceof List<?> list) { for (Object child : list) collectTexts(child, spans, texts); }
        else if (value != null && value.getClass().isRecord()) {
            try { for (var component : value.getClass().getRecordComponents()) collectTexts(component.getAccessor().invoke(value), spans, texts); }
            catch (ReflectiveOperationException error) { throw new IllegalStateException(error); }
        }
    }
    private static void collectNodes(Object value, List<Object> nodes) {
        if (value instanceof List<?> list) { for (Object child : list) collectNodes(child, nodes); }
        else if (value instanceof Map<?, ?> node && node.get("fields") instanceof Map<?, ?> fields) {
            if (Set.of("Leaf", "Other", "OtherLeaf").contains(node.get("type"))) {
                var span = (List<?>) node.get("span");
                Object text = fields.containsKey("text") ? fields.get("text") : fields.get("value");
                if (text instanceof String) nodes.add(List.of(span.get(0), span.get(1), text));
            }
            for (Object child : fields.values()) collectNodes(child, nodes);
        }
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
        if (node.get("type").equals("Negation")) { Double child = numeric(fields.get("value")); return child == null ? null : -child; }
        if (node.get("type").equals("Conditional")) {
            Double condition = numeric(fields.get("condition"));
            return condition == null ? null : numeric(fields.get(condition != 0 ? "thenExpr" : "elseExpr"));
        }
        if (fields.size() == 1 && fields.containsKey("value")) return numeric(fields.get("value"));
        if (fields.size() != 3 || !fields.containsKey("left") || !fields.containsKey("op") || !fields.containsKey("right")) return null;
        List<?> ops = values(fields.get("op")), rights = values(fields.get("right"));
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
