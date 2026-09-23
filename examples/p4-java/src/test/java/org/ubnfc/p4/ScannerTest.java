package org.ubnfc.p4;

import static org.junit.jupiter.api.Assertions.*;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.ubnfc.p4.generated.TinyExpressionP4Parser;
import org.ubnfc.p4.generated.api.TokenScanner;
import org.ubnfc.p4.generated.api.TokenScanner.*;
import org.ubnfc.p4.generated.rt.Input;

class ScannerTest {
    private ScanResult scan(String token, String text, int c, int m, Mode mode, boolean invert, boolean reset) {
        return P4Scanners.ALL.get("TinyExpressionP4::" + token).scan(new Input(text), new ScanState(c, m, mode, invert, reset));
    }
    private void accepts(String token, String text, int length) {
        accepts(token, text, length, value(text, length));
    }
    private void accepts(String token, String text, int length, ValueSpan value) {
        var result = scan(token, text, 0, 0, Mode.consumed, false, true);
        assertTrue(result.ok(), text);
        assertEquals(length, result.consumedEnd(), text);
        assertEquals(length, result.matchedEnd(), text);
        assertEquals(value, result.valueSpan(), text);
        assertTrue(result.effects().isEmpty());
    }
    /** 旧 Mapper の文字列変換は単引用符だけ value から除く（generators.md #132）。
     *  共有 registry（scanners/{java,rust}）と Rust backend もこの規則。 */
    private ValueSpan value(String text, int length) {
        return text.startsWith("'") && length >= 2 ? new ValueSpan(1, length - 1) : new ValueSpan(0, length);
    }
    private void rejects(String token, String text) {
        var result = scan(token, text, 0, 0, Mode.consumed, false, true);
        assertFalse(result.ok(), text);
        assertEquals(0, result.consumedEnd());
        assertEquals(0, result.matchedEnd());
    }
    @Test void quotedAcceptance() {
        for (String text : List.of("''", "\"\"", "'こんにちは😀'", "'a\nb'", "\"a\rb\"", "'a\\qb'", "'a\\\nb'", "'a\\\'b'", "\"a\\\"b\"", "'a\\\\b'", "'\u0000'"))
            accepts("STRING", text, text.length());
        accepts("STRING", "'a'tail", 3, new ValueSpan(1, 2));
        // 二重引用符は value に残る。`''` は空の value になる。
        accepts("STRING", "\"'a'\"", 5, new ValueSpan(0, 5));
        accepts("STRING", "''", 2, new ValueSpan(1, 1));
        for (String text : List.of("", "a", "'", "'a", "'a\\", "'a\\'", "'a\"")) rejects("STRING", text);
    }
    @Test void fences() {
        for (String suffix : List.of("", "\r", "\n", "\r\n")) {
            String start = "```java:a.b_C9" + suffix;
            accepts("CODE_START", start, start.length());
            accepts("CODE_END", "```" + suffix, 3 + suffix.length());
        }
        accepts("CODE_START", "```_x:Foo\nbody", 10);
        for (String text : List.of(" ```", "``", "``` ", "```x", "```\u2028")) rejects("CODE_END", text);
        for (String text : List.of("```java:9Foo", "```java:A.", "```java:A$B", "```java:日本", "```java:A ", "```java :A", "```9java:A")) rejects("CODE_START", text);
        assertTrue(scan("CODE_END", "x\r```\n", 2, 2, Mode.consumed, false, true).ok());
        assertTrue(scan("CODE_END", "x\n```\n", 2, 2, Mode.consumed, false, true).ok());
        assertFalse(scan("CODE_END", "x```\n", 1, 1, Mode.consumed, false, true).ok());
    }
    @Test void fiveStateInputsAndPurity() {
        String text = "x'😀'!";
        var matched = scan("STRING", text, 0, 1, Mode.matchOnly, false, false);
        assertTrue(matched.ok()); assertEquals(0, matched.consumedEnd()); assertEquals(5, matched.matchedEnd());
        // `'😀'` は UTF-16 で 4 単位。単引用符を除いた [2,4) が value。
        assertEquals(new ValueSpan(2, 4), matched.valueSpan());
        assertFalse(scan("STRING", text, 0, 1, Mode.matchOnly, false, true).ok());
        assertEquals(scan("STRING", text, 1, 5, Mode.consumed, false, false),
            scan("STRING", text, 1, 5, Mode.consumed, false, false));
        // 反転は token 全体の否定ではなく、旧 parser の子に伝播する。
        var inverse = scan("STRING", "ab", 0, 0, Mode.consumed, true, true);
        assertTrue(inverse.ok()); assertEquals(2, inverse.consumedEnd());
        assertFalse(scan("CODE_END", "```", 0, 0, Mode.consumed, true, true).ok());
    }
    @Test void fencedCodeRunsThroughExternsAndUntil() {
        var result = TinyExpressionP4Parser.parse("```java:a.B\nclass B {}\n```\n1", P4Scanners.options(true));
        assertTrue(result.ok(), result.diagnostics().toString());
        assertEquals(28, result.consumed());
    }
}
