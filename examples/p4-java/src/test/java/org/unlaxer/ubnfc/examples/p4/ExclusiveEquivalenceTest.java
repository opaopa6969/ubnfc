package org.unlaxer.ubnfc.examples.p4;

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
import org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4Parser;
import org.unlaxer.ubnfc.examples.p4.generated.api.ParseOptions;
import org.unlaxer.ubnfc.examples.p4.generated.api.ParseResult;
import org.unlaxer.ubnfc.examples.p4.generated.internal.Session;

/**
 * D-066（排他 predict の空振り短絡）が参照経路（元順で全候補を試す経路）と同じ観測を残すことを、
 * 全 fixture の完全入力・code point 境界での切り詰め・1 文字破壊で確かめる。短絡するのは
 * fast mode（{@code diag=false}）だけなので、比較も認識専用の options で行う（診断モードでは
 * {@code Session#exclusiveReady} が偽になり、両経路は同じ機械語を通る）。
 *
 * <p>P4 で短絡が出るのは {@code AddOp ::= '+' | '-'} と {@code MulOp ::= '*' | '/'} の 2 か所。
 */
final class ExclusiveEquivalenceTest {
    private static final ParseOptions FAST =
        new ParseOptions(false, true, false, true, P4Scanners.ALL, ParseOptions.DEFAULT_MAX_DEPTH, false, false);

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

    /** 認識専用経路が公開する観測を 1 本の文字列にする。 */
    private static String observe(String source, String entry) {
        ParseResult<?> r = entry == null ? TinyExpressionP4Parser.parse(source, FAST)
            : TinyExpressionP4Parser.parseEntry("TinyExpressionP4", entry, source, FAST);
        var out = new LinkedHashMap<String, Object>();
        out.put("ok", r.ok());
        out.put("consumed", r.consumedCp());
        out.put("matched", r.matchedCp());
        var d = r.diagnostics();
        var failure = r.nativeFailure();
        out.put("diagnostic", Map.of("kind", d.kind(), "offset", d.offset(), "farthest", d.farthestCp(),
            "expected", d.expected(), "deepestRule", String.valueOf(d.deepestRule()), "ruleStack", d.ruleStack(),
            "farthestExpected", d.farthestExpected(), "severity", d.severity(), "origin", d.origin()));
        out.put("nativeFailure", failure == null ? "null"
            : List.of(failure.offset(), failure.consumed(), failure.matched(), failure.expected()));
        var scope = r.scope();
        out.put("declarations", scope.declarations().stream().map(String::valueOf).toList());
        out.put("references", scope.references().stream().map(String::valueOf).toList());
        out.put("scopeDiagnostics", scope.diagnostics().stream().map(String::valueOf).toList());
        out.put("scopeEvents", scope.events().stream().map(String::valueOf).toList());
        out.put("scopeDepth", scope.depth());
        out.put("recoveries", r.recoveries().stream().map(String::valueOf).toList());
        return String.valueOf(out);
    }

    @Test
    void shortCircuitedExclusiveChoicesLeaveTheSameObservationsAsTheReferencePath() throws IOException {
        int compared = 0;
        try {
            for (String source : inputs()) {
                for (String entry : new String[] {null, "BooleanExpression", "Expression"}) {
                    Session.exclusive = true;
                    String shortCircuited = observe(source, entry);
                    Session.exclusive = false;
                    String reference = observe(source, entry);
                    assertEquals(reference, shortCircuited,
                        "entry=" + entry + " input=" + source.substring(0, Math.min(80, source.length())));
                    compared++;
                }
            }
        } finally {
            Session.exclusive = true;
        }
        assertTrue(compared > 2000, "比較件数: " + compared);
    }
}
