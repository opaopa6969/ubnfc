package org.unlaxer.tinyexpression.p4.ubnfc;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.ubnfc.p4.generated.api.Json;
import org.unlaxer.tinyexpression.p4.P4PreferredAstMapper;
import org.unlaxer.tinyexpression.parser.ExpressionType;
import org.unlaxer.tinyexpression.parser.ExpressionTypes;

/**
 * 旧ファサードと ubnfc 版ファサードを同一 JVM 内で同じ入力に流し、canonical JSON /
 * selectionMode / 失敗の型を突き合わせる。差分は消さずに全部記録する。
 */
final class ParityRun {

    record Case(String source, String origin, ExpressionType preferredType,
                String preferredAstSimpleName, boolean negative) {}

    record Outcome(String selectionMode, String json, String failure) {
        static Outcome ok(String selectionMode, String json) {
            return new Outcome(selectionMode, json, null);
        }

        static Outcome failed(Throwable failure) {
            return new Outcome(null, null,
                failure.getClass().getName() + ": " + failure.getMessage());
        }
    }

    record Row(Case testCase, Outcome legacy, Outcome fresh, String verdict, String reason) {}

    private ParityRun() {}

    static List<Case> corpus() throws IOException {
        var cases = new ArrayList<Case>();
        try (InputStream in = ParityRun.class.getResourceAsStream("/parity/te-formulas.json")) {
            String text = new String(in.readAllBytes(), StandardCharsets.UTF_8);
            for (Object entry : Json.array(Json.read(text))) {
                Map<String, Object> value = Json.object(entry);
                cases.add(new Case(
                    (String) value.get("source"),
                    (String) value.get("origin"),
                    expressionType((String) value.get("preferredType")),
                    (String) value.get("preferredAstSimpleName"),
                    Boolean.TRUE.equals(value.get("negative"))));
            }
        }
        Path fixtures = Path.of(System.getProperty("parity.fixtures",
            "../p4-java/src/test/resources/fixtures"));
        if (Files.isDirectory(fixtures)) {
            try (var stream = Files.list(fixtures)) {
                for (Path file : stream.sorted().toList()) {
                    if (!file.getFileName().toString().endsWith(".tiny")) continue;
                    cases.add(new Case(Files.readString(file),
                        "p4-java fixture " + file.getFileName(), null, null,
                        file.getFileName().toString().startsWith("invalid")
                            || file.getFileName().toString().startsWith("complex-half")
                            || file.getFileName().toString().startsWith("complex-tail")));
                }
            }
        }
        return cases;
    }

    private static ExpressionType expressionType(String name) {
        return name == null || name.isBlank() ? null : ExpressionTypes.valueOf(name);
    }

    static Row compare(Case testCase) {
        Outcome legacy = run(testCase, true);
        Outcome fresh = run(testCase, false);
        if (legacy.failure() != null || fresh.failure() != null) {
            if (legacy.failure() != null && fresh.failure() != null) {
                return new Row(testCase, legacy, fresh, "both-reject",
                    sameFailureKind(legacy.failure(), fresh.failure())
                        ? "同じ例外型" : "例外型が違う");
            }
            return new Row(testCase, legacy, fresh,
                legacy.failure() != null ? "legacy-only-reject" : "ubnfc-only-reject", "");
        }
        if (!legacy.selectionMode().equals(fresh.selectionMode())) {
            return new Row(testCase, legacy, fresh, "mode-differs", "");
        }
        if (!legacy.json().equals(fresh.json())) {
            return new Row(testCase, legacy, fresh, "ast-differs", classify(testCase));
        }
        return new Row(testCase, legacy, fresh, "same", "");
    }

    /** 差分の分類は読みやすさのためだけで、差分は消さない。 */
    private static String classify(Case testCase) {
        return testCase.source().codePoints().anyMatch(c -> c > 0xFFFF)
            ? "非 BMP 文字を含む（旧実装の UTF-16 混在 span）" : "要調査";
    }

    private static boolean sameFailureKind(String legacy, String fresh) {
        return legacy.substring(0, legacy.indexOf(':'))
            .equals(fresh.substring(0, fresh.indexOf(':')));
    }

    private static Outcome run(Case testCase, boolean legacy) {
        try {
            if (testCase.preferredAstSimpleName() != null) {
                long deadline = System.nanoTime() + 10_000L * 1_000_000L;
                if (legacy) {
                    var parsed = P4PreferredAstMapper.parseByAstSimpleNameDetailed(
                        testCase.source(), testCase.preferredAstSimpleName(), deadline);
                    return Outcome.ok(parsed.selectionMode(),
                        CanonicalAst.json(parsed.ast(), parsed.sourceText()));
                }
                var parsed = UbnfcP4PreferredAstMapper.parseByAstSimpleNameDetailed(
                    testCase.source(), testCase.preferredAstSimpleName(), deadline);
                return Outcome.ok(parsed.selectionMode(),
                    CanonicalAst.json(parsed.ast(), parsed.sourceText()));
            }
            if (legacy) {
                var parsed = P4PreferredAstMapper.parseDetailed(
                    testCase.source(), testCase.preferredType());
                return Outcome.ok(parsed.selectionMode(),
                    CanonicalAst.json(parsed.ast(), parsed.sourceText()));
            }
            var parsed = UbnfcP4PreferredAstMapper.parseDetailed(
                testCase.source(), testCase.preferredType());
            return Outcome.ok(parsed.selectionMode(),
                CanonicalAst.json(parsed.ast(), parsed.sourceText()));
        } catch (RuntimeException | StackOverflowError failure) {
            return Outcome.failed(failure);
        }
    }

    static void writeReport(List<Row> rows, Path out) throws IOException {
        Files.createDirectories(out.getParent());
        var text = new StringBuilder();
        text.append("verdict\torigin\tlegacy_mode\tubnfc_mode\treason\tsource\n");
        for (Row row : rows) {
            text.append(row.verdict()).append('\t')
                .append(row.testCase().origin()).append('\t')
                .append(escape(row.legacy().selectionMode() == null
                    ? row.legacy().failure() : row.legacy().selectionMode())).append('\t')
                .append(escape(row.fresh().selectionMode() == null
                    ? row.fresh().failure() : row.fresh().selectionMode())).append('\t')
                .append(row.reason()).append('\t')
                .append(escape(row.testCase().source())).append('\n');
        }
        Files.writeString(out, text.toString());
    }

    private static String escape(String value) {
        return value == null ? "" : value.replace("\\", "\\\\").replace("\n", "\\n").replace("\t", "\\t");
    }

    static Map<String, Integer> tally(List<Row> rows) {
        var counts = new LinkedHashMap<String, Integer>();
        for (Row row : rows) counts.merge(row.verdict(), 1, Integer::sum);
        return counts;
    }
}
