package org.ubnfc.p4;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.ubnfc.p4.generated.TinyExpressionP4Parser;
import org.ubnfc.p4.generated.api.ParseOptions;
import org.ubnfc.p4.generated.api.ParseResult;

/**
 * D-023 / D-035 の二段構え（認識専用経路）の同値性。全 fixture の完全入力・code point 境界での
 * 切り詰め・1 文字破壊を、認識専用（観測を要求しない）と認識＋観測要求で比べる。
 *
 * <p>契約: 受理・cursor・scope は常に一致する。診断は「失敗・末尾入力なら再解析で完全に一致、
 * 成功なら hints が空」。出現表は要求したときだけ出る。同じ入力を 2 度解析した観測も一致する
 * （解析が決定的であることの確認で、Rust 側 `diagnostics.rs` の同値テストと同じ役割）。
 */
final class TwoModeEquivalenceTest {
    private static final ParseOptions FAST =
        new ParseOptions(false, true, false, true, P4Scanners.ALL, ParseOptions.DEFAULT_MAX_DEPTH, false, false);
    private static final ParseOptions FULL = FAST.withObservations(true, true);

    private static List<String> inputs() throws IOException {
        var out = new ArrayList<String>();
        try (var files = Files.list(Path.of("src/test/resources/fixtures"))) {
            for (Path file : files.filter(p -> p.toString().endsWith(".tiny")).sorted().toList()) {
                String text = Files.readString(file);
                out.add(text);
                if (text.length() > 4000) continue;
                int step = Math.max(1, text.length() / 40);
                for (int cut = 0; cut < text.length(); cut += step) {
                    out.add(text.substring(0, cut));
                    out.add(text.substring(0, cut) + "@" + text.substring(Math.min(text.length(), cut + 1)));
                }
            }
        }
        return out;
    }

    private static ParseResult<?> parse(String source, String entry, ParseOptions options) {
        return entry == null ? TinyExpressionP4Parser.parse(source, options)
            : TinyExpressionP4Parser.parseEntry("TinyExpressionP4", entry, source, options);
    }

    /** 認識経路が常に守るもの（受理・cursor・scope・recovery）。 */
    private static String core(ParseResult<?> r) {
        var out = new LinkedHashMap<String, Object>();
        out.put("ok", r.ok());
        out.put("consumed", r.consumedCp());
        out.put("matched", r.matchedCp());
        var scope = r.scope();
        out.put("declarations", scope.declarations().stream().map(String::valueOf).toList());
        out.put("references", scope.references().stream().map(String::valueOf).toList());
        out.put("scopeDiagnostics", scope.diagnostics().stream().map(String::valueOf).toList());
        out.put("scopeEvents", scope.events().stream().map(String::valueOf).toList());
        out.put("scopeDepth", scope.depth());
        out.put("recoveries", r.recoveries().stream().map(String::valueOf).toList());
        return String.valueOf(out);
    }

    private static String diagnostics(ParseResult<?> r) {
        var d = r.diagnostics();
        var failure = r.nativeFailure();
        return String.valueOf(Map.of("kind", d.kind(), "offset", d.offset(), "farthest", d.farthestCp(),
            "expected", d.expected(), "deepestRule", String.valueOf(d.deepestRule()), "ruleStack", d.ruleStack(),
            "farthestExpected", d.farthestExpected(), "severity", d.severity(), "origin", d.origin(),
            "nativeFailure", failure == null ? "null"
                : List.of(failure.offset(), failure.consumed(), failure.matched(), failure.expected())));
    }

    @Test
    void recognitionOnlyKeepsTheContractAndReParsesDeterministicallyOnFailure() throws IOException {
        int compared = 0;
        for (String source : inputs()) {
            for (String entry : new String[] {null, "BooleanExpression", "Expression"}) {
                ParseResult<?> fast = parse(source, entry, FAST);
                ParseResult<?> full = parse(source, entry, FULL);
                String where = "entry=" + entry + " input=" + source.substring(0, Math.min(80, source.length()));
                assertEquals(core(full), core(fast), where);
                if (!full.ok()) {
                    // D-023: 失敗・末尾入力は診断モードで再解析されるので観測は完全に一致する。
                    assertEquals(diagnostics(full), diagnostics(fast), where);
                } else {
                    // 成功経路は観測を記録しない（hints が空）。
                    assertTrue(fast.diagnostics().expected().isEmpty(), where);
                    assertTrue(fast.diagnostics().farthestExpected().isEmpty(), where);
                }
                // D-035: 出現表は要求したときだけ。
                assertTrue(fast.captures().isEmpty(), where);
                assertTrue(fast.lexical().isEmpty(), where);
                assertTrue(fast.ast().isEmpty(), where);
                // 決定的: 同じ入力の 2 度目も同じ観測。
                ParseResult<?> again = parse(source, entry, FAST);
                assertEquals(core(fast), core(again), where);
                assertEquals(diagnostics(fast), diagnostics(again), where);
                compared++;
            }
        }
        assertTrue(compared > 1000, "比較件数: " + compared);
    }
}
