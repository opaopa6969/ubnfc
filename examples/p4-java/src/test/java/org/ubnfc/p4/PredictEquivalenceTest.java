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
import org.ubnfc.p4.generated.api.Json;
import org.ubnfc.p4.generated.api.ParseResult;
import org.ubnfc.p4.generated.internal.Session;

/**
 * 候補除外（emit/Prediction が生成する Guard）が参照経路（全候補を実際に試す経路）と同じ観測を
 * 残すことを、全 fixture の完全入力・code point 境界での切り詰め・1 文字破壊で確かめる。
 * 比較対象は受理・cursor・主診断（offset / expected / 最深 rule / rule 経路）・farthest・
 * nativeFailure・capture・scope（宣言・参照・診断・イベント）・recovery・AST。
 */
final class PredictEquivalenceTest {
    private static List<String> inputs() throws IOException {
        var out = new ArrayList<String>();
        try (var files = Files.list(Path.of("src/test/resources/fixtures"))) {
            for (Path file : files.filter(p -> p.toString().endsWith(".tiny")).sorted().toList()) {
                String text = Files.readString(file);
                out.add(text);
                // 大きい fixture（x16 / x64）は同じ構造の繰り返しなので完全入力だけを見る。
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

    /** 観測できる全ての結果を 1 本の文字列にする。 */
    private static String observe(String source, boolean ast, boolean lexical, String entry) {
        var options = new org.ubnfc.p4.generated.api.ParseOptions(ast, true, lexical, true, P4Scanners.options(ast).scanners());
        ParseResult<?> result;
        String mapping = null;
        try {
            result = entry == null ? TinyExpressionP4Parser.parse(source, options)
                : TinyExpressionP4Parser.parseEntry("TinyExpressionP4", entry, source, options);
        } catch (RuntimeException error) {
            return "throw:" + error.getClass().getName() + ":" + error.getMessage();
        }
        var out = new LinkedHashMap<String, Object>();
        out.put("ok", result.ok());
        out.put("consumed", result.consumedCp());
        out.put("matched", result.matchedCp());
        var d = result.diagnostics();
        out.put("diagnostic", Map.of("kind", d.kind(), "offset", d.offset(), "farthest", d.farthestCp(),
            "expected", d.expected(), "deepestRule", String.valueOf(d.deepestRule()), "ruleStack", d.ruleStack(),
            "farthestExpected", d.farthestExpected(), "severity", d.severity(), "origin", d.origin()));
        var failure = result.nativeFailure();
        out.put("nativeFailure", failure == null ? null
            : List.of(failure.offset(), failure.consumed(), failure.matched(), failure.expected()));
        out.put("captures", result.captures().stream().map(c -> List.of(c.occurrenceId(), c.siteId(), String.valueOf(c.name()),
            c.span().start(), c.span().end(), c.completionOrder(), c.ruleCompletionOrder())).toList());
        out.put("lexical", result.lexical().stream().map(l -> List.of(l.occurrenceId(), l.parentOccurrenceId(), l.ruleId(),
            l.exprId(), l.span().start(), l.span().end(), l.completionOrder(), String.valueOf(l.token()))).toList());
        var scope = result.scope();
        out.put("declarations", scope.declarations().stream().map(String::valueOf).toList());
        out.put("references", scope.references().stream().map(String::valueOf).toList());
        out.put("scopeDiagnostics", scope.diagnostics().stream().map(String::valueOf).toList());
        out.put("scopeEvents", scope.events().stream().map(String::valueOf).toList());
        out.put("scopeDepth", scope.depth());
        out.put("recoveries", result.recoveries().stream().map(String::valueOf).toList());
        out.put("mapping", mapping);
        String tree;
        try { tree = Json.canonical(result); } catch (RuntimeException error) { tree = "ast-error:" + error; }
        out.put("ast", tree);
        return String.valueOf(out);
    }

    @Test
    void excludedCandidatesLeaveTheSameObservationsAsTheReferencePath() throws IOException {
        List<String> inputs = inputs();
        int compared = 0;
        try {
            for (String source : inputs) {
                for (String entry : new String[] {null, "BooleanExpression", "Expression"}) {
                    for (boolean ast : new boolean[] {false, true}) {
                        Session.predict = true;
                        String predicted = observe(source, ast, true, entry);
                        Session.predict = false;
                        String reference = observe(source, ast, true, entry);
                        assertEquals(reference, predicted, "entry=" + entry + " ast=" + ast
                            + " input=" + source.substring(0, Math.min(80, source.length())));
                        compared++;
                    }
                }
            }
        } finally {
            Session.predict = true;
        }
        assertTrue(compared > 2000, "比較件数: " + compared);
    }
}
