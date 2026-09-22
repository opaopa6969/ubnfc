package target;

import java.util.ArrayList;
import java.util.LinkedHashMap;

/** corpus/golden 専用の小さな strict JSON reader。本体の依存を増やさない。 */
final class JsonReader {
    private final String source;
    private int at;
    private JsonReader(String source) { this.source = source; }
    static Object read(String source) {
        var reader = new JsonReader(source);
        Object value = reader.value();
        reader.space();
        if (reader.at != source.length()) throw reader.error();
        return value;
    }
    private void space() { while (at < source.length() && " \r\n\t".indexOf(source.charAt(at)) >= 0) at++; }
    private char take() { if (at == source.length()) throw error(); return source.charAt(at++); }
    private boolean accept(char ch) { space(); if (at < source.length() && source.charAt(at) == ch) { at++; return true; } return false; }
    private void require(char ch) { if (!accept(ch)) throw error(); }
    private Object value() {
        space();
        if (at == source.length()) throw error();
        char ch = source.charAt(at);
        if (ch == '"') return string();
        if (accept('[')) {
            var list = new ArrayList<>();
            if (accept(']')) return list;
            do { list.add(value()); } while (accept(','));
            require(']'); return list;
        }
        if (accept('{')) {
            var map = new LinkedHashMap<String, Object>();
            if (accept('}')) return map;
            do { space(); String key = string(); require(':'); map.put(key, value()); } while (accept(','));
            require('}'); return map;
        }
        for (String literal : new String[]{"true", "false", "null"}) if (source.startsWith(literal, at)) {
            at += literal.length(); return literal.equals("null") ? null : literal.equals("true");
        }
        int start = at;
        if (source.charAt(at) == '-') at++;
        while (at < source.length() && Character.isDigit(source.charAt(at))) at++;
        if (at < source.length() && source.charAt(at) == '.') {
            at++;
            while (at < source.length() && Character.isDigit(source.charAt(at))) at++;
        }
        if (at < source.length() && (source.charAt(at) == 'e' || source.charAt(at) == 'E')) {
            at++;
            if (at < source.length() && "+-".indexOf(source.charAt(at)) >= 0) at++;
            while (at < source.length() && Character.isDigit(source.charAt(at))) at++;
        }
        if (start == at) throw error();
        return Double.valueOf(source.substring(start, at));
    }
    private String string() {
        require('"');
        var out = new StringBuilder();
        while (true) {
            char ch = take();
            if (ch == '"') return out.toString();
            if (ch < 0x20) throw error();
            if (ch != '\\') { out.append(ch); continue; }
            out.append(switch (take()) {
                case '"' -> '"'; case '\\' -> '\\'; case '/' -> '/';
                case 'b' -> '\b'; case 'f' -> '\f'; case 'n' -> '\n'; case 'r' -> '\r'; case 't' -> '\t';
                case 'u' -> {
                    if (at + 4 > source.length()) throw error();
                    char decoded = (char) Integer.parseInt(source.substring(at, at + 4), 16);
                    at += 4; yield decoded;
                }
                default -> throw error();
            });
        }
    }
    private IllegalArgumentException error() { return new IllegalArgumentException("Invalid JSON at " + at); }
}
