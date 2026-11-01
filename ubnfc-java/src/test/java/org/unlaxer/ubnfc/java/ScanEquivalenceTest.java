package org.unlaxer.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.unlaxer.ubnfc.java.CompilerHarness.*;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestFactory;
import org.unlaxer.ubnfc.java.emit.JavaEmitter;
import org.unlaxer.ubnfc.java.ir.IrReader;
import org.unlaxer.ubnfc.java.ir.Json;

/**
 * D-080（D-072 の Java 版: {@code plan.scans} の領域を生成走査関数へ畳む）が参照経路と同じ観測を
 * 残すことを、{@code plan.scans} を持つ 2 文法（JSON / formula-info）の入力・接頭辞・1 文字破壊で確かめる。
 *
 * <p>形は {@code FactorEquivalenceTest} / {@code ExclusiveEquivalenceTest} と同じ:
 * 走査 on / off（{@code Session.scanning}）× 認識 / AST × 出現表あり / なし × 診断あり / なし。
 * 走査が走るのは「診断なし・出現表なし・lexical なし」の解析だけで、他の組合せは両変種が同じ機械語を
 * 通ることを確かめる。
 */
class ScanEquivalenceTest {
    private static final Map<String, CompilerHarness> harnesses = new LinkedHashMap<>();

    @BeforeAll static void compile() throws Exception {
        harnesses.put("json", new CompilerHarness(IrReader.read(Path.of("../ir/examples/json-v2.ir.json")), "scan.generated.json"));
        harnesses.put("formula-info", new CompilerHarness(IrReader.read(Path.of("../ir/examples/formula-info.ir.json")), "scan.generated.formulainfo"));
    }
    @AfterAll static void close() throws Exception { for (var harness : harnesses.values()) harness.close(); }

    private static void scanning(CompilerHarness h, boolean value) throws Exception {
        h.loader.loadClass(h.pkg + ".internal.Session").getField("scanning").setBoolean(null, value);
    }

    /** 公開される観測を 1 本にまとめる（受理・cursor・主診断・失敗位置・出現表・scope・typed AST と span）。 */
    private static Object observe(CompilerHarness h, String input, Object options) throws Exception {
        Object result;
        try { result = h.parse(input, options); }
        catch (java.lang.reflect.InvocationTargetException mapping) { return "throws " + mapping.getCause(); }
        var out = new LinkedHashMap<String, Object>();
        out.put("ok", ok(result));
        out.put("consumed", get(result, "consumedCp"));
        out.put("matched", get(result, "matchedCp"));
        out.put("diagnostics", String.valueOf(get(result, "diagnostics")));
        out.put("failure", String.valueOf(get(result, "nativeFailure")));
        out.put("captures", String.valueOf(get(result, "captures")));
        out.put("lexical", String.valueOf(get(result, "lexical")));
        out.put("scope", String.valueOf(get(result, "scope")));
        out.put("ast", String.valueOf(get(result, "ast")));
        out.put("canonical", String.valueOf(h.canonical(result)));
        // rollback 解析は D-023 の再解析を通らず、fast session の到達位置を診断位置として返す。
        String grammar = h.parser.getSimpleName().substring(0, h.parser.getSimpleName().length() - "Parser".length());
        Object rolled = h.parser.getMethod("parseEntryRollback", String.class, String.class, CharSequence.class, h.options)
            .invoke(null, grammar, null, input, options);
        out.put("rollback", ok(rolled) + " " + get(rolled, "consumedCp") + " " + get(rolled, "matchedCp") + " " + get(rolled, "diagnostics") + " " + get(rolled, "scope"));
        return out;
    }

    private static List<String> seeds(String grammar) throws Exception {
        var out = new ArrayList<String>();
        if (grammar.equals("json")) {
            try (var files = Files.list(Path.of("../corpus/spec/json"))) {
                for (Path file : files.sorted().toList())
                    for (Object row : Json.array(Json.object(Json.read(Files.readString(file))).get("cases")))
                        if (Json.object(row).get("input") instanceof String input) out.add(input);
                        else {
                            // inputHex の行は byte 列。UTF-8 として読めない byte は置換文字になる（それも 1 入力）。
                            String hex = (String) Json.object(row).get("inputHex");
                            byte[] bytes = java.util.HexFormat.of().parseHex(hex);
                            out.add(new String(bytes, java.nio.charset.StandardCharsets.UTF_8));
                        }
            }
            // 走査の境界: 表の ASCII / 非 ASCII / 補助面 / 孤立 surrogate、表から本体への落下（escape）、
            // 有界反復（HEX{4}）、optional の欠落（値なし bit）、空白の各文字。
            out.addAll(List.of("\"\"", "\"a\\\"b\\\\c\\/d\\be\\ff\\ng\\rh\\ti\\u00e9j\\uD83D\\uDE00\"", "\"é漢字😀\"",
                "\"\uD83D\"", "\"\uDE00x\"", "\"\\u12\"", "\"\\u12G4\"", "\"\\x\"", "\"\u0001\"",
                "0", "-0", "-", "01", "1.", "1.5", "1e", "1e+", "1E-07", "-12.340e+5", "123456789012345678901234567890",
                " \t\r\n[ \t\r\n1 \t\r\n, \t\r\n{ \"k\" : [true,false,null] } \t\r\n] \t\r\n", "[]", "{}", "[ ]", "{ }",
                "[1,]", "{\"a\":1,}", "{\"a\" 1}", "[\"\\u0041\", 1e5, -0.5E+3]", "truex", "nul"));
        } else {
            try (var files = Files.list(Path.of("../examples/formula-info/samples"))) {
                for (Path file : files.sorted().toList()) out.add(Files.readString(file));
            }
        }
        return out;
    }

    /** 各入力と、その接頭辞・1 文字破壊（短い入力だけ。長い sample は全体だけ）。 */
    private static List<String> inputs(String grammar) throws Exception {
        var out = new LinkedHashSet<String>();
        for (String input : seeds(grammar)) {
            out.add(input);
            if (input.length() > 200) continue;
            for (int cut = 0; cut <= input.length(); cut++) {
                out.add(input.substring(0, cut));
                out.add(input.substring(0, cut) + "@" + input.substring(Math.min(input.length(), cut + 1)));
            }
        }
        return List.copyOf(out);
    }

    private static List<Object> optionsOf(CompilerHarness h) throws Exception {
        Object recognize = h.options.getField("RECOGNIZE").get(null);
        Object ast = h.options.getField("DEFAULT").get(null);
        var method = h.options.getMethod("withObservations", boolean.class, boolean.class);
        var out = new ArrayList<Object>();
        for (Object base : List.of(recognize, ast))
            for (boolean occurrences : List.of(false, true))
                for (boolean diagnostics : List.of(false, true)) out.add(method.invoke(base, occurrences, diagnostics));
        // lexical を要求する解析（走査しない側）も 1 つ。
        out.add(h.options(true, true, true, true, Map.of()));
        return out;
    }

    @TestFactory Stream<DynamicTest> observationsAreIdentical() throws Exception {
        var tests = new ArrayList<DynamicTest>();
        for (var entry : harnesses.entrySet()) {
            CompilerHarness h = entry.getValue();
            List<Object> options = optionsOf(h);
            for (String input : inputs(entry.getKey())) {
                String name = input.length() > 40 ? input.substring(0, 40) + "…" : input;
                tests.add(DynamicTest.dynamicTest(entry.getKey() + ": " + name, () -> {
                    for (Object option : options) {
                        scanning(h, false);
                        Object before = observe(h, input, option);
                        scanning(h, true);
                        Object after = observe(h, input, option);
                        assertEquals(before, after, "D-080 の on / off で観測が違う: " + option);
                    }
                }));
            }
        }
        return tests.stream();
    }

    /**
     * 走査が実際に走っていることの証人で、同時に唯一の観測差（資源上限）の向きの固定。
     * {@code [1]} の参照経路は Json → Value → Array → Value → Number → Int の 6 段を規則として呼ぶが、
     * 走査は Number の本体（{@code Int} を含む領域）を 1 本で読むので 5 段で済む。maxDepth=5 では
     * 参照経路だけが上限に当たる。**逆向き（走査だけが上限に当たる）は起こらない。**
     */
    @Test void scanOnlyEverReducesResourceUse() throws Exception {
        CompilerHarness h = harnesses.get("json");
        Object options = h.options.getConstructor(boolean.class, boolean.class, boolean.class, boolean.class, Map.class, int.class, boolean.class, boolean.class)
            .newInstance(false, true, false, true, Map.of(), 5, false, false);
        scanning(h, false);
        assertFalse(ok(h.parse("[1]", options)));
        scanning(h, true);
        assertTrue(ok(h.parse("[1]", options)));
        scanning(h, true);
    }

    /** {@code plan.scans} が空の文法には走査関数も走査経路も出ない（生成 parser は従来と同じ）。 */
    @Test void grammarsWithoutScanPlansAreUnchanged() throws Exception {
        for (String ir : List.of("src/test/resources/ir/operators.ir.json", "src/test/resources/ir/power.ir.json",
                "../ir/frozen/tinyexpression-p4-f86ce8a5.ir.json")) {
            var emitted = new JavaEmitter(IrReader.read(Path.of(ir)), "probe.noscan").sources();
            String parser = emitted.entrySet().stream().filter(e -> e.getKey().endsWith("Parser.java")).findFirst().orElseThrow().getValue();
            assertFalse(parser.contains("scanned(") || parser.contains("private static long scan"), ir);
        }
        String json = new JavaEmitter(IrReader.read(Path.of("../ir/examples/json-v2.ir.json")), "probe.json").sources().get("probe/json/JsonParser.java");
        assertEquals(20, json.lines().filter(l -> l.contains("s.scanned(f, scan")).count(), "JSON の根 20 個すべてに走査経路が付く");
    }
}
