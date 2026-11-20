package org.unlaxer.ubnfc.examples.p4;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4Parser;
import org.unlaxer.ubnfc.examples.p4.generated.api.Json;
import org.unlaxer.ubnfc.examples.p4.generated.api.ParseOptions;
import org.unlaxer.ubnfc.examples.p4.generated.api.ParseResult;
import org.unlaxer.ubnfc.examples.p4.generated.internal.Session;

/** 規則完了の出来事からの直接構築と旧 Recipe 経路の全観測を比較する。 */
final class DirectAstEquivalenceTest {
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

    private static String observe(String source, String entry, boolean ast, boolean occurrences) {
        var options = new ParseOptions(ast, true, false, true, P4Scanners.ALL,
            ParseOptions.DEFAULT_MAX_DEPTH, occurrences, false);
        ParseResult<?> result;
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
        out.put("diagnostic", List.of(d.kind(), d.offset(), d.farthestCp(), d.expected(),
            String.valueOf(d.deepestRule()), d.ruleStack(), d.farthestExpected(), d.severity(), d.origin()));
        var failure = result.nativeFailure();
        out.put("nativeFailure", failure == null ? null
            : List.of(failure.offset(), failure.consumed(), failure.matched(), failure.expected()));
        out.put("captures", result.captures().stream().map(c -> List.of(c.occurrenceId(), c.siteId(),
            String.valueOf(c.name()), c.span().start(), c.span().end(), c.completionOrder(), c.ruleCompletionOrder())).toList());
        var scope = result.scope();
        out.put("declarations", scope.declarations().stream().map(String::valueOf).toList());
        out.put("references", scope.references().stream().map(String::valueOf).toList());
        out.put("scopeDiagnostics", scope.diagnostics().stream().map(String::valueOf).toList());
        out.put("scopeEvents", scope.events().stream().map(String::valueOf).toList());
        out.put("scopeDepth", scope.depth());
        out.put("recoveries", result.recoveries().stream().map(String::valueOf).toList());
        String tree;
        try { tree = Json.canonical(result); } catch (RuntimeException error) { tree = "ast-error:" + error; }
        out.put("ast", tree);
        return String.valueOf(out);
    }

    @Test
    void directEventsLeaveTheSameObservationsAsRecipes() throws IOException {
        int compared = 0;
        try {
            for (String source : inputs()) {
                for (String entry : new String[] {null, "BooleanExpression", "Expression"}) {
                    for (boolean ast : new boolean[] {false, true}) {
                        for (boolean occurrences : new boolean[] {false, true}) {
                            Session.directAst = true;
                            String bypassed = observe(source, entry, ast, occurrences);
                            Session.directAst = false;
                            String reference = observe(source, entry, ast, occurrences);
                            assertEquals(reference, bypassed, "entry=" + entry + " ast=" + ast + " occ=" + occurrences
                                + " input=" + source.substring(0, Math.min(80, source.length())));
                            compared++;
                        }
                    }
                }
            }
        } finally {
            Session.directAst = true;
        }
        assertTrue(compared > 4000, "比較件数: " + compared);
    }
    @Test
    void directLeafFallbackKeepsTheCompletePublicSpanTable() throws IOException {
        String source = Files.readString(Path.of("src/test/resources/fixtures/complex.tiny"));
        var options = P4Scanners.options(true).withObservations(false, false);
        try {
            Session.directAst = false;
            var reference = TinyExpressionP4Parser.parse(source, options);
            Session.directAst = true;
            var direct = TinyExpressionP4Parser.parse(source, options);
            assertEquals(Json.canonical(reference), Json.canonical(direct));
            var before = reference.nodeSpans().entrySet().stream()
                .map(e -> e.getKey().getClass().getName() + ":" + e.getKey() + ":" + e.getValue()).sorted().toList();
            var after = direct.nodeSpans().entrySet().stream()
                .map(e -> e.getKey().getClass().getName() + ":" + e.getKey() + ":" + e.getValue()).sorted().toList();
            assertEquals(before, after, "leaf の原文へ余計な span を付けず、AST 自身の span を保持する");
        } finally {
            Session.directAst = true;
        }
    }

}
