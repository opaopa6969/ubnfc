package org.ubnfc.java.ir;

import java.io.IOException;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.math.BigDecimal;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

public final class IrReader {
    private IrReader() {}
    public static final Set<String> FEATURES = Set.of("dualCursor", "cursorFrameFlag", "orderedChoice",
        "predictiveChoice", "longestChoice", "triviaInsertion", "captures", "deferredMapping", "scopeUndo",
        "backendDiagnostics", "memoReplay", "externTokens", "regex", "operators", "leftRecursionRewrite",
        "recovery", "imports", "enum", "commonField", "catalog", "structuredOccurrences", "eval", "typeof");
    public static GrammarIr read(Path path) throws IOException { return read(Files.readString(path)); }
    public static GrammarIr read(String source) {
        Object value = Json.read(source);
        var root = Json.object(value);
        for (Object feature : Json.array(root.get("requiredFeatures")))
            if (!FEATURES.contains(feature)) throw new IllegalArgumentException("Unsupported requiredFeatures: " + feature);
        try (var stream = IrReader.class.getResourceAsStream("/schema/grammar-ir.v1.schema.json")) {
            if (stream == null) throw new IllegalStateException("Missing bundled IR schema");
            var schema = Json.object(Json.read(new String(stream.readAllBytes(), StandardCharsets.UTF_8)));
            // Accept the ordered DFA metadata while retaining compatibility with pre-round2 IR.
            var definitions = Json.object(schema.get("$defs"));
            var dfa = Json.object(definitions.get("RegexDfa"));
            var required = new ArrayList<>(Json.array(dfa.get("required"))); required.remove("longestMatch");
            dfa.put("required", required);
            var properties = Json.object(dfa.get("properties")); properties.put("semantics", Map.of("const", "leftmostFirst"));
            dfa.put("properties", properties); definitions.put("RegexDfa", dfa); schema.put("$defs", definitions);
            new Schema(schema, true).validate(value);
        } catch (IOException error) { throw new IllegalStateException(error); }
        GrammarIr ir = (GrammarIr) bind(GrammarIr.class, value);
        Validation.validate(ir);
        return ir;
    }
    private static Object bind(Type type, Object value) {
        if (type instanceof ParameterizedType parameterized) {
            if (parameterized.getRawType() == List.class) {
                if (value == null) return List.of();
                var result = new ArrayList<Object>();
                for (Object item : Json.array(value)) result.add(bind(parameterized.getActualTypeArguments()[0], item));
                return Collections.unmodifiableList(result);
            }
            return value == null ? Map.of() : Collections.unmodifiableMap(new LinkedHashMap<>(Json.object(value)));
        }
        if (value == null) return null;
        Class<?> clazz = (Class<?>) type;
        if (clazz == Integer.class) return ((BigDecimal) value).intValueExact();
        if (!clazz.isRecord()) return value;
        if (clazz == GrammarIr.AstTypeVariantsItem.class && value instanceof String id)
            return new GrammarIr.AstTypeVariantsItem(null, null, null, id);
        var map = Json.object(value);
        var fields = clazz.getRecordComponents();
        var types = new Class<?>[fields.length];
        var values = new Object[fields.length];
        for (int i = 0; i < fields.length; i++) {
            String name = fields[i].getName().replaceFirst("_$", "");
            types[i] = fields[i].getType();
            values[i] = bind(fields[i].getGenericType(), map.get(name));
        }
        try { return clazz.getDeclaredConstructor(types).newInstance(values); }
        catch (ReflectiveOperationException error) { throw new IllegalArgumentException("Cannot bind " + clazz.getName(), error); }
    }
}
