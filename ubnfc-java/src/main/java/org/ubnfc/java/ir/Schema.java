package org.ubnfc.java.ir;

import java.math.BigDecimal;
import java.util.HashSet;
import java.util.Map;
import java.util.Objects;
import java.util.regex.Pattern;

/** Validator for the vocabulary used by grammar-ir.v1 (not a general JSON Schema implementation). */
public final class Schema {
    private final Map<String, Object> root;
    private final boolean unknownKeys;
    public Schema(Object schema, boolean allowUnknownKeys) { root = Json.object(schema); unknownKeys = allowUnknownKeys; }
    public void validate(Object value) { check(root, value, "$ "); }
    private void check(Map<String, Object> schema, Object value, String path) {
        if (schema.containsKey("$ref")) {
            String name = (String) schema.get("$ref");
            check(Json.object(Json.object(root.get("$defs")).get(name.substring(name.lastIndexOf('/') + 1))), value, path);
            return;
        }
        for (String union : new String[]{"oneOf", "anyOf"}) if (schema.containsKey(union)) {
            int matches = 0;
            for (Object branch : Json.array(schema.get(union))) {
                try { check(Json.object(branch), value, path); matches++; } catch (IllegalArgumentException ignored) { }
            }
            if (matches == 0 || union.equals("oneOf") && matches != 1) fail(path, union + " mismatch");
            return;
        }
        if (schema.containsKey("const") && !Objects.equals(schema.get("const"), value)) fail(path, "contract must be " + schema.get("const"));
        if (schema.containsKey("enum") && !Json.array(schema.get("enum")).contains(value)) fail(path, "unsupported value " + value);
        if (schema.get("type") instanceof java.util.List<?> types) {
            for (Object alternative : types) {
                var branch = new java.util.LinkedHashMap<>(schema); branch.put("type", alternative);
                try { check(branch, value, path); return; } catch (IllegalArgumentException ignored) { }
            }
            fail(path, "type mismatch");
        }
        String type = (String) schema.get("type");
        if (type == null) return;
        boolean valid = switch (type) {
            case "object" -> value instanceof Map<?, ?>;
            case "array" -> value instanceof java.util.List<?>;
            case "string" -> value instanceof String;
            case "boolean" -> value instanceof Boolean;
            case "integer" -> value instanceof BigDecimal n && n.stripTrailingZeros().scale() <= 0;
            case "number" -> value instanceof BigDecimal;
            case "null" -> value == null;
            default -> throw new IllegalArgumentException("Unknown schema type " + type);
        };
        if (!valid) fail(path, "expected " + type);
        if (value instanceof Map<?, ?>) {
            var object = Json.object(value);
            var properties = schema.containsKey("properties") ? Json.object(schema.get("properties")) : Map.<String, Object>of();
            if (schema.containsKey("required")) for (Object key : Json.array(schema.get("required")))
                if (!object.containsKey(key)) fail(path, "missing " + key);
            for (var entry : object.entrySet()) {
                Object child = properties.get(entry.getKey());
                if (child != null) check(Json.object(child), entry.getValue(), path + "." + entry.getKey());
                else if (schema.get("additionalProperties") instanceof Map<?, ?> extra) check(Json.object(extra), entry.getValue(), path);
                else if (!unknownKeys && Boolean.FALSE.equals(schema.get("additionalProperties"))) fail(path, "unknown key " + entry.getKey());
            }
        } else if (value instanceof java.util.List<?> list) {
            if (schema.containsKey("minItems") && list.size() < number(schema.get("minItems"))) fail(path, "too few items");
            if (Boolean.TRUE.equals(schema.get("uniqueItems")) && new HashSet<>(list).size() != list.size()) fail(path, "duplicate item");
            if (schema.containsKey("items")) for (int i = 0; i < list.size(); i++) check(Json.object(schema.get("items")), list.get(i), path + "[" + i + "]");
        } else if (value instanceof String text) {
            if (schema.containsKey("minLength") && text.codePointCount(0, text.length()) < number(schema.get("minLength"))) fail(path, "too short");
            if (schema.containsKey("pattern") && !Pattern.compile((String) schema.get("pattern")).matcher(text).find()) fail(path, "invalid format " + text);
        } else if (value instanceof BigDecimal n) {
            if (schema.containsKey("minimum") && n.compareTo((BigDecimal) schema.get("minimum")) < 0) fail(path, "below minimum");
            if (schema.containsKey("maximum") && n.compareTo((BigDecimal) schema.get("maximum")) > 0) fail(path, "above maximum");
        }
    }
    private static int number(Object value) { return ((BigDecimal) value).intValueExact(); }
    private static void fail(String path, String reason) { throw new IllegalArgumentException(path + ": " + reason); }
}
