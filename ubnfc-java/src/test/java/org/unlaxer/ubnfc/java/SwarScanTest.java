package org.unlaxer.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.unlaxer.ubnfc.java.CompilerHarness.*;
import static org.unlaxer.ubnfc.java.IrFixture.*;
import static org.unlaxer.ubnfc.java.ScanEquivalenceTest.*;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;
import org.unlaxer.ubnfc.java.emit.JavaEmitter;
import org.unlaxer.ubnfc.java.ir.IrReader;

class SwarScanTest {
    @Test void wordBoundariesAndFixedSeedMatchReference() throws Exception {
        var ir = IrReader.read(Path.of("../ir/examples/json-v2.ir.json"));
        String parser = "scan/swar/JsonParser.java";
        String scalar = new JavaEmitter(ir, "scan.swar").sources().get(parser);
        String word = new JavaEmitter(ir, "scan.swar", true, true).sources().get(parser);
        assertFalse(scalar.contains("scanSwarWord"));
        assertTrue(word.contains("long word = scanSwarWord"));

        var inputs = new ArrayList<String>();
        for (int length = 0; length <= 40; length++) {
            String prefix = "a".repeat(length);
            for (String stop : List.of("", "\\\\", "\\\"", "\\uD83D\\uDE00", "\\x", "é", "😀", "\u007f", "\u0080",
                    "\u8000", "\uFFFF", "\uD83D", "\uDE00", "\r", "\"")) {
                inputs.add("[\"" + prefix + stop + "z".repeat(12) + "\",1234567890]");
            }
            inputs.add("\"" + prefix); // 末尾と未閉鎖文字列
            inputs.add(" ".repeat(length) + "1" + "9".repeat(length) + ".0123456789e+1234");
            inputs.add(" ".repeat(length) + "-."); // 1 回以上の走査の下限を満たさない
        }
        var random = new Random(29L);
        var alphabet = List.of("a", "Z", "0", "9", "\"", "\\", "\n", "é", "😀", "\\n", "\\u0000");
        for (int i = 0; i < 256; i++) {
            var input = new StringBuilder("[\"");
            for (int j = 0; j < 64; j++) input.append(alphabet.get(random.nextInt(alphabet.size())));
            inputs.add(input.append("\",0]").toString());
        }
        assertEquals(994, inputs.size());

        try (var h = new CompilerHarness(ir, "scan.swar", Map.of(), true)) {
            var execute = h.loader.loadClass(h.pkg + ".driver.Driver").getDeclaredMethod("execute", Map.class);
            execute.setAccessible(true);
            var options = optionsOf(h);
            for (int i = 0; i < inputs.size(); i++) {
                String input = inputs.get(i);
                String label = "入力 " + i + " " + input;
                var base = Map.of("id", 1, "grammar", "Json", "entry", "Json", "input", input);
                var request = new java.util.HashMap<String, Object>(base);
                request.put("options", Map.of("scan", false, "scope", true, "lexical", false));
                Object off = execute.invoke(null, request);
                request.put("options", Map.of("scan", true, "scope", true, "lexical", false));
                assertEquals(off, execute.invoke(null, request), label);

                // ドライバの fast mode が省く出現表・診断・lexical と rollback も比較する。
                for (Object option : options) {
                    scanning(h, false);
                    Object before = observe(h, input, option);
                    scanning(h, true);
                    assertEquals(before, observe(h, input, option), label);
                }
            }
        }
    }

    @Test void observedScopeAndCaptureOutsideScanArePreserved() throws Exception {
        var f = new IrFixture();
        String digit = f.token("Digits", "DIGIT", obj("kind", "builtin", "which", "Digit", "codePoints", List.of()));
        String repeat = f.expression("Digits", "repeat", "child", digit, "min", 1, "max", null, "terminator", null);
        f.rule("Digits", repeat);
        f.plan.put("scans", List.of(obj("kind", "charClassRepetition", "exprId", repeat, "root", true,
            "class", obj("asciiLow", 0x03ff000000000000L, "asciiHigh", 0, "ranges", List.of(), "complete", true),
            "ruleIds", List.of(), "proof", obj("kind", "terminalOnlyScan", "explanation", "数字だけの反復",
                "exprIds", List.of(repeat)))));
        String ref = f.ref("Root", "Digits");
        var text = obj("kind", "text");
        String site = f.capture(ref, "value", "text", "scalar", text);
        f.rule("Root", ref);
        f.mapping("Root", "Root", List.of(field("value", text, "scalar")),
            List.of(recipeField("value", site, "text", "scalar")));
        f.effects.add(obj("ruleId", "Root", "mode", "lexical", "events",
            List.of(obj("when", "successAfterLeave", "action", "declare", "captureSiteId", site, "name", "value"))));

        try (var h = new CompilerHarness(f.build(), "scan.swar.scope", Map.of(), true)) {
            String input = "01234567890123456789";
            var fast = h.options.getMethod("withObservations", boolean.class, boolean.class)
                .invoke(h.options.getField("DEFAULT").get(null), false, false);
            scanning(h, true);
            Object result = h.parse(input, fast);
            assertTrue(ok(result));
            assertEquals(1, ((List<?>) get(get(result, "scope"), "events")).size());
            assertFalse(((Map<?, ?>) get(result, "nodeSpans")).isEmpty());
            Object withScan = observe(h, input, fast);
            scanning(h, false);
            assertEquals(withScan, observe(h, input, fast));
        }
    }
}
