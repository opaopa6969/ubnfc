package target.driver;

import java.math.BigDecimal;
import java.util.ArrayList;
import java.util.LinkedHashMap;

/** 依存不要の strict JSON reader。id の数値精度を保持する。 */
final class JsonReader {
    private final String source;
    private int at;
    private JsonReader(String source) { this.source = source; }
    static Object read(String source) {
        var reader = new JsonReader(source);
        Object result = reader.value();
        reader.space();
        if (reader.at != source.length()) throw reader.error();
        return result;
    }
    private void space() { while (at < source.length() && " \t\n\r".indexOf(source.charAt(at)) >= 0) at++; }
    private boolean accept(char ch) { space(); if (at < source.length() && source.charAt(at) == ch) { at++; return true; } return false; }
    private void require(char ch) { if (!accept(ch)) throw error(); }
    private char take() { if (at == source.length()) throw error(); return source.charAt(at++); }
    private boolean digit() { return at < source.length() && source.charAt(at) >= '0' && source.charAt(at) <= '9'; }
    private void digits() { int start = at; while (digit()) at++; if (start == at) throw error(); }
    private Object value() {
        space();
        if (at == source.length()) throw error();
        if (source.charAt(at) == '"') return string();
        if (accept('{')) {
            var object = new LinkedHashMap<String, Object>();
            if (accept('}')) return object;
            do {
                space(); String key = string(); require(':');
                if (object.containsKey(key)) throw error();
                object.put(key, value());
            } while (accept(','));
            require('}'); return object;
        }
        if (accept('[')) {
            var array = new ArrayList<Object>();
            if (accept(']')) return array;
            do { array.add(value()); } while (accept(','));
            require(']'); return array;
        }
        for (String literal : new String[]{"true", "false", "null"}) {
            if (source.startsWith(literal, at)) {
                at += literal.length();
                return literal.equals("null") ? null : literal.equals("true");
            }
        }
        int start = at;
        if (source.charAt(at) == '-') at++;
        if (at < source.length() && source.charAt(at) == '0') at++;
        else digits();
        if (at < source.length() && source.charAt(at) == '.') { at++; digits(); }
        if (at < source.length() && "eE".indexOf(source.charAt(at)) >= 0) {
            at++;
            if (at < source.length() && "+-".indexOf(source.charAt(at)) >= 0) at++;
            digits();
        }
        return new BigDecimal(source.substring(start, at));
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
                case 'n' -> '\n'; case 'r' -> '\r'; case 't' -> '\t'; case 'b' -> '\b'; case 'f' -> '\f';
                case 'u' -> {
                    int code = 0;
                    for (int i = 0; i < 4; i++) {
                        char hex = take();
                        if ("0123456789abcdefABCDEF".indexOf(hex) < 0) throw error();
                        code = code * 16 + Character.digit(hex, 16);
                    }
                    yield (char) code;
                }
                default -> throw error();
            });
        }
    }
    private IllegalArgumentException error() { return new IllegalArgumentException("不正な JSON: " + at); }
}
