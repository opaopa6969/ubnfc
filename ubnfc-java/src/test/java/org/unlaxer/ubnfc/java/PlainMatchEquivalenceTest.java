package org.unlaxer.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.unlaxer.ubnfc.java.CompilerHarness.*;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
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
 * D-075（認識専用経路の成功値が {@code Match} を作らない）が参照経路と同じ観測を残すことを、
 * {@code @scope} 効果を持たない 2 文法の全 corpus 入力・接頭辞・1 文字破壊で確かめる。
 *
 * <p>短絡するのは {@code tree=false} の経路だけなので、比較は認識専用の options（診断あり /
 * なしの両方）で行う。AST を作る options では {@code Session#plain} が偽になり、両変種は
 * 同じ機械語を通る（それも 1 件だけ確かめる）。
 */
class PlainMatchEquivalenceTest {
    private static final Path TARGET = Path.of("../examples/handwritten-target/java/src/test/resources");
    private static final Map<String, CompilerHarness> harnesses = new LinkedHashMap<>();

    @BeforeAll static void compile() throws Exception {
        for (String grammar : List.of("operators", "power")) {
            var ir = IrReader.read(Path.of("src/test/resources/ir/" + grammar + ".ir.json"));
            harnesses.put(grammar, new CompilerHarness(ir, "plain.generated." + grammar));
        }
    }
    @AfterAll static void close() throws Exception { for (var harness : harnesses.values()) harness.close(); }

    private static void plainMatch(CompilerHarness h, boolean value) throws Exception {
        h.loader.loadClass(h.pkg + ".internal.Session").getField("plainMatch").setBoolean(null, value);
    }

    /** 公開される観測を 1 本にまとめる（受理・cursor・主診断・出現表・scope）。 */
    private static Object observe(CompilerHarness h, String input, Object options) throws Exception {
        Object result = h.parse(input, options);
        var out = new LinkedHashMap<String, Object>();
        out.put("ok", ok(result));
        out.put("consumed", get(result, "consumedCp"));
        out.put("matched", get(result, "matchedCp"));
        out.put("diagnostics", String.valueOf(get(result, "diagnostics")));
        out.put("failure", String.valueOf(get(result, "nativeFailure")));
        out.put("captures", String.valueOf(get(result, "captures")));
        out.put("scope", String.valueOf(get(result, "scope")));
        out.put("canonical", String.valueOf(h.canonical(result)));
        return out;
    }

    private static List<String> inputs(String grammar) throws Exception {
        var out = new ArrayList<String>();
        for (Object row : Json.array(Json.read(Files.readString(TARGET.resolve("corpus/" + grammar + ".json"))))) {
            String input = (String) Json.object(row).get("input");
            out.add(input);
            for (int cut = 0; cut <= input.length(); cut++) {
                out.add(input.substring(0, cut));
                out.add(input.substring(0, cut) + "@" + input.substring(Math.min(input.length(), cut + 1)));
            }
        }
        return out;
    }

    @TestFactory Stream<DynamicTest> recognitionObservationsAreIdentical() throws Exception {
        var tests = new ArrayList<DynamicTest>();
        for (String grammar : List.of("operators", "power")) {
            CompilerHarness h = harnesses.get(grammar);
            Object fast = h.options.getField("RECOGNIZE").get(null);
            Object diagnosed = h.options.getMethod("withDiagnostics").invoke(fast);
            for (String input : inputs(grammar)) {
                tests.add(DynamicTest.dynamicTest(grammar + ": " + input, () -> {
                    for (Object options : List.of(fast, diagnosed)) {
                        plainMatch(h, false);
                        Object before = observe(h, input, options);
                        plainMatch(h, true);
                        Object after = observe(h, input, options);
                        plainMatch(h, false);
                        assertEquals(before, after, "D-075 の on / off で観測が違う");
                    }
                }));
            }
        }
        return tests.stream();
    }

    /** AST を作る経路は {@code tree=true} なので {@code plain} が立たない（観測も割当も同じ）。 */
    @Test void astPathIsUnaffected() throws Exception {
        CompilerHarness h = harnesses.get("operators");
        for (String input : inputs("operators")) {
            plainMatch(h, false);
            Object before = observe(h, input, h.options.getField("DEFAULT").get(null));
            plainMatch(h, true);
            Object after = observe(h, input, h.options.getField("DEFAULT").get(null));
            plainMatch(h, false);
            assertEquals(before, after);
        }
    }

    /**
     * 生成側の条件: {@code @scope} 効果を 1 つも持たない文法だけが {@code plainRecognition=true}
     * を渡す。効果を持つ文法（P4）は参照経路のままになる。
     */
    @Test void onlyScopeFreeGrammarsGetThePlainPath() throws Exception {
        String operators = new JavaEmitter(IrReader.read(Path.of("src/test/resources/ir/operators.ir.json")), "probe.operators")
            .sources().get("probe/operators/OperatorsParser.java");
        assertTrue(operators.contains("new Session(source, options, K0, true, true)"), operators.lines().filter(l -> l.contains("new Session")).toList().toString());
        Path scoped = Path.of("../ir/frozen/tinyexpression-p4-f86ce8a5.ir.json");
        String p4 = new JavaEmitter(IrReader.read(scoped), "probe.p4").sources().get("probe/p4/TinyExpressionP4Parser.java");
        assertTrue(p4.contains("new Session(source, options, K0, true, false)"), p4.lines().filter(l -> l.contains("new Session")).toList().toString());
    }
}
