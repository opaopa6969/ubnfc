package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.CompilerHarness.*;
import static org.ubnfc.java.IrFixture.*;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

class TokenTest {
    private record Case(Map<String, Object> spec, List<String> accepted, List<String> rejected) {}
    @TestFactory Stream<DynamicTest> primitiveProfiles() {
        return Stream.of(
            new Case(obj("kind", "builtin", "which", "Number", "codePoints", List.of()), List.of("+1", "-0", ".5", "1.", "1.e2", "-.5e-2"), List.of(".", "+", "1e+", "１２")),
            new Case(obj("kind", "builtin", "which", "Identifier", "codePoints", List.of()), List.of("_", "aZ_09"), List.of("é", "9a", "😀")),
            new Case(obj("kind", "caseInsensitive", "word", "AbC", "comparison", "javaEqualsIgnoreCase", "wordBoundary", false), List.of("abc", "ABC"), List.of("ab", "abcd")),
            new Case(obj("kind", "charRange", "min", 97, "max", 99), List.of("a", "c"), List.of("d", "ab", "😀")),
            // D-063: 補助面の charRange。Java の照合は code point 単位（Character.codePointAt）なので
            // surrogate 対 1 組をちょうど 1 文字として読む。
            new Case(obj("kind", "charRange", "min", 0x1F600, "max", 0x1F607), List.of("😀", "😇"), List.of("a", "😈", "\ud83d")),
            new Case(obj("kind", "charRange", "min", 0xE000, "max", 0x10FFFF), List.of("😀", "\ue000"), List.of("a", "\ud7ff")),
            new Case(obj("kind", "negation", "excluded", "x😀"), List.of("a", "😁"), List.of("x", "😀", "")),
            new Case(obj("kind", "regex", "pattern", "(?:😀|[a-z])+", "regularSubset", false, "matching", "lookingAt", "zeroWidth", "fail", "javaFallback", "java.util.regex.Pattern", "rustFallback", "generationError"), List.of("😀ab", "abc"), List.of("", "1abc")),
            new Case(obj("kind", "regex", "pattern", "a*", "regularSubset", false, "matching", "lookingAt", "zeroWidth", "fail", "javaFallback", "java.util.regex.Pattern", "rustFallback", "generationError"), List.of("a", "aaa"), List.of("", "b")),
            new Case(obj("kind", "eof"), List.of(""), List.of("a", "😀"))
        ).map(test -> DynamicTest.dynamicTest(test.spec().toString(), () -> {
            var fixture = new IrFixture(); fixture.rule("Root", fixture.token("Root", "TOKEN", test.spec()));
            try (var harness = fixture.compile()) {
                for (String input : test.accepted()) {
                    Object result = harness.parse(input); assertTrue(ok(result), input);
                    assertEquals(input.codePointCount(0, input.length()), get(result, "consumedCp"));
                }
                for (String input : test.rejected()) assertFalse(ok(harness.parse(input)), input);
            }
        }));
    }
}
