package ubnfc.scanners;

import generated.api.TokenScanner;
import generated.api.TokenScanner.*;
import generated.rt.Input;
import java.util.List;
import java.util.Map;

/** 外部依存なし。全 scanner は pure: 入力と ScanState だけを読み、効果を返さない。 */
public final class Registry {
    public static final boolean PURE = true;
    private Registry() {}
    public static Map<String, TokenScanner> scanners() {
        return Map.ofEntries(
            Map.entry("DoubleQuotedParser", scanner(Kind.DOUBLE)),
            Map.entry("org.unlaxer.parser.elementary.DoubleQuotedParser", scanner(Kind.DOUBLE)),
            Map.entry("SingleQuotedParser", scanner(Kind.SINGLE)),
            Map.entry("org.unlaxer.parser.elementary.SingleQuotedParser", scanner(Kind.SINGLE)),
            Map.entry("org.unlaxer.tinyexpression.parser.StringLiteralParser", scanner(Kind.STRING)),
            Map.entry("org.unlaxer.tinyexpression.parser.javalang.CodeStartParser", scanner(Kind.START)),
            Map.entry("org.unlaxer.tinyexpression.parser.javalang.CodeEndParser", scanner(Kind.END)),
            Map.entry("oracle.ExternalCursorParser", scanner(Kind.EXTERNAL)));
    }
    private enum Kind { DOUBLE, SINGLE, STRING, START, END, EXTERNAL }
    private static TokenScanner scanner(Kind kind) {
        return (input, state) -> new Scan(input, state).run(kind);
    }
    private static final class Scan {
        final Input input;
        final ScanState state;
        int p, valueStart, valueEnd, failure;
        String expected = "token";
        Scan(Input input, ScanState state) {
            this.input = input; this.state = state;
            p = state.mode() == Mode.matchOnly ? state.matched() : state.consumed();
        }
        int cp() { return p < input.length() ? input.codePointAt(p) : -1; }
        boolean fail(String label) { failure = p; expected = label; return false; }
        boolean word(String word) {
            // unlaxer-common/src/main/java/org/unlaxer/parser/elementary/WordParser.java:67-82: invert は末尾でなく各 terminal に適用。
            int end = p;
            for (int i = 0; i < word.codePointCount(0, word.length()) && end < input.length(); i++)
                end += Character.charCount(input.codePointAt(end));
            boolean equal = input.startsWith(p, word);
            if (p == input.length() || equal == state.invertMatch()) return fail("'" + word + "'");
            p = end; return true;
        }
        boolean identifier() {
            // unlaxer-common/src/main/java/org/unlaxer/parser/clang/IdentifierParser.java:24-31 と posix/Alphabet*UnderScoreParser。
            if (cp() < 0 || head(cp()) == state.invertMatch()) return fail("identifier");
            p += Character.charCount(cp());
            while (cp() >= 0 && (head(cp()) || cp() >= '0' && cp() <= '9') != state.invertMatch())
                p += Character.charCount(cp());
            return true;
        }
        boolean head(int c) { return c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'; }
        boolean quoted(int quote) {
            // unlaxer-common/src/main/java/org/unlaxer/parser/elementary/QuotedParser.java:55-71、EscapeInQuotedParser.java:22-26。
            // 任意1 code point の escape、改行/NUL 可。引用符は value に対して single のみ除く。
            int start = p;
            if (cp() != quote) return fail("quote");
            p++;
            if (state.invertMatch()) return fail("closing quote");
            while (cp() >= 0) {
                int c = cp(); p += Character.charCount(c);
                if (c == quote) {
                    valueStart = quote == '\'' ? start + 1 : start;
                    valueEnd = quote == '\'' ? p - 1 : p;
                    return true;
                }
                if (c == '\\' && !state.invertMatch()) {
                    if (cp() < 0) return fail("escaped character");
                    p += Character.charCount(cp());
                }
            }
            return fail("closing quote");
        }
        boolean fence(boolean start) {
            // tinyexpression/src/main/java/org/unlaxer/tinyexpression/parser/javalang/
            // CodeStartParser.java:32-40、CodeEndParser.java:21-26。
            // tinyexpression/src/main/java/org/unlaxer/tinyexpression/parser/javatype/JavaClassNameParser.java:28-39 は identifier ('.' identifier)*。
            // unlaxer-common/src/main/java/org/unlaxer/parser/elementary/StartOfLineParser.java:38-53。
            boolean sol = p == 0 || input.startsWith(p - 1, "\r") || input.startsWith(p - 1, "\n");
            if (!sol || state.invertMatch()) return fail("start of line");
            if (!word("```")) return false;
            if (start) {
                if (!identifier() || !word(":") || !identifier()) return false;
                while (cp() == '.') { p++; if (!identifier()) return false; }
            }
            valueEnd = p;
            // unlaxer-common/src/main/java/org/unlaxer/parser/elementary/LineTerminatorParser.java:20-26: CRLF / CR / LF / EOF。
            if (input.startsWith(p, "\r\n")) p += 2;
            else if (cp() == '\r' || cp() == '\n') p++;
            else if (cp() >= 0) return fail("end of line");
            return true;
        }
        ScanResult run(Kind kind) {
            // ChainInterface.java:17 は reset=false、ChoiceInterface.java:28-29 は begin で reset を参照。
            if (kind == Kind.STRING && state.mode() == Mode.matchOnly && state.resetMatchedWithConsumed())
                p = state.consumed();
            valueStart = p;
            boolean ok = switch (kind) {
                case DOUBLE -> quoted('"');
                case SINGLE -> quoted('\'');
                // tinyexpression/src/main/java/org/unlaxer/tinyexpression/parser/StringLiteralParser.java:19-25: double / single の選択。
                case STRING -> quoted(cp() == '"' ? '"' : '\'');
                case START -> fence(true);
                case END -> fence(false);
                // corpus/oracle/tools/ExternalCursorParser.java:8-10: 😀 消費 + x の MatchOnly。
                case EXTERNAL -> word("😀");
            };
            int consumed = state.mode() == Mode.matchOnly ? state.consumed() : p;
            if (kind == Kind.EXTERNAL && ok) {
                valueEnd = p;
                ok = word("x");
            }
            if (!ok) {
                // ExternalCursorParser.java:10 / ParseContext.java:1097,1111-1148:
                // Chain の先頭失敗では子の候補も収集され、MatchOnly の 'x' を含む。
                var diagnostics = kind == Kind.EXTERNAL && expected.equals("'😀'")
                    ? List.of(new ScanDiagnostic(failure, expected), new ScanDiagnostic(failure, "'x'"))
                    : List.of(new ScanDiagnostic(failure, expected));
                return new ScanResult(false, state.consumed(), state.matched(),
                    null, diagnostics, List.of());
            }
            return new ScanResult(true, consumed, p, valueStart, valueEnd, List.of(), List.of());
        }
    }
}
