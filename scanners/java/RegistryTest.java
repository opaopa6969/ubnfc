package ubnfc.scanners;

import generated.api.TokenScanner;
import generated.api.TokenScanner.*;
import generated.rt.Input;

/** -ea で実行。公開5状態、位置単位、value span、rollback を実 API に対して検証する。 */
public final class RegistryTest {
    private static ScanResult scan(String name, String input, int c, int m, Mode mode, boolean invert, boolean reset) {
        TokenScanner scanner = Registry.scanners().get(name);
        assert scanner != null : name;
        return scanner.scan(new Input(input), new ScanState(c, m, mode, invert, reset));
    }
    private static void result(ScanResult r, boolean ok, int c, int m, int start, int end) {
        assert r.ok() == ok : r;
        assert r.consumedEnd() == c && r.matchedEnd() == m : r;
        assert r.effects().isEmpty() : r;
        if (ok) assert r.valueSpan().equals(new ValueSpan(start, end)) : r;
        else assert !r.diagnostics().isEmpty() : r;
    }
    public static void main(String[] args) {
        String string = "org.unlaxer.tinyexpression.parser.StringLiteralParser";
        String start = "org.unlaxer.tinyexpression.parser.javalang.CodeStartParser";
        String end = "org.unlaxer.tinyexpression.parser.javalang.CodeEndParser";
        String external = "oracle.ExternalCursorParser";
        result(scan(string, "'😀'", 0, 0, Mode.consumed, false, true), true, 4, 4, 1, 3);
        result(scan("DoubleQuotedParser", "\"\\😀\"", 0, 0, Mode.matchOnly, false, false), true, 0, 5, 0, 5);
        result(scan(string, "''\"b\"", 0, 2, Mode.matchOnly, false, true), true, 0, 2, 1, 1);
        result(scan(string, "''\"b\"", 0, 2, Mode.matchOnly, false, false), true, 0, 5, 2, 5);
        result(scan(string, "'x", 0, 0, Mode.consumed, false, true), false, 0, 0, 0, 0);
        result(scan(string, "'x'", 0, 0, Mode.consumed, true, true), false, 0, 0, 0, 0);
        result(scan(external, "😀x", 0, 0, Mode.consumed, false, true), true, 2, 3, 0, 2);
        result(scan(external, "😀x", 0, 0, Mode.matchOnly, false, false), true, 0, 3, 0, 2);
        result(scan(external, "😀y", 0, 0, Mode.consumed, false, true), false, 0, 0, 0, 0);
        result(scan(external, "ay", 0, 0, Mode.consumed, true, false), true, 1, 2, 0, 1);
        result(scan(start, "x\r\n```java:a.B\r\n", 3, 3, Mode.consumed, false, true), true, 16, 16, 3, 14);
        result(scan(end, "```\n", 0, 0, Mode.matchOnly, false, true), true, 0, 4, 0, 3);
        for (String input : new String[]{" ```", "```x", "``", "````"})
            result(scan(end, input, 0, 0, Mode.consumed, false, true), false, 0, 0, 0, 0);
        for (String input : new String[]{"```λ:a.B\n", "```java:a.$B\n", "```java:a..B\n", "```java:a.B.\n"})
            result(scan(start, input, 0, 0, Mode.consumed, false, true), false, 0, 0, 0, 0);
        assert !Registry.scanners().containsKey("other.StringLiteralParser");
        assert Registry.PURE;
    }
}
