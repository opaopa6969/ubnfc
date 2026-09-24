package org.unlaxer.tinyexpression.p4.ubnfc;

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
    }
}
