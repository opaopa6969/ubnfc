package org.unlaxer.tinyexpression.p4.ubnfc;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.Test;

/**
 * tinyexpression のテストが facade に渡す式（324 件）と p4-java の 16 fixture について、
 * 旧 facade と ubnfc 版 facade の {@code ParsedAst} を突き合わせる。
 * 結果は {@code target/parity/report.tsv} に全件残す。
 */
class ParityTest {

    @Test
    void sameParsedAstForEveryFormula() throws Exception {
        // 旧実装は深い入力で秒単位かかり、共有マシンの負荷次第で既定 10 秒の期限に当たる。
        // 期限切れは実装の差ではないので、比較の間は両側とも期限を外す。
        String previous = System.setProperty("tinyexpression.p4.parse.timeout.millis", "0");
        try {
            compareAll();
        } finally {
            if (previous == null) System.clearProperty("tinyexpression.p4.parse.timeout.millis");
            else System.setProperty("tinyexpression.p4.parse.timeout.millis", previous);
        }
    }

    private void compareAll() throws Exception {
        List<ParityRun.Case> corpus = ParityRun.corpus();
        assertTrue(corpus.size() >= 300, "コーパスが読めていない: " + corpus.size());
        var rows = corpus.stream().map(ParityRun::compare).toList();
        ParityRun.writeReport(rows, Path.of("target/parity/report.tsv"));
        System.out.println("parity: " + ParityRun.tally(rows));
        for (var row : rows) {
            if (!row.verdict().equals("same") && !row.verdict().equals("both-reject")) {
                System.out.println("  " + row.verdict() + " | " + row.testCase().origin()
                    + " | " + row.reason() + " | legacy=" + row.legacy().selectionMode()
                    + " ubnfc=" + row.fresh().selectionMode());
            }
        }
        var unexpected = rows.stream()
            .filter(row -> !row.verdict().equals("same") && !row.verdict().equals("both-reject")
                || row.verdict().equals("both-reject") && !row.legacy().failure().equals(row.fresh().failure()))
            .map(row -> row.verdict() + " " + row.testCase().origin())
            .toList();
        assertEquals(List.of(), unexpected, "旧と新で ParsedAst または失敗が食い違う");
    }
}
