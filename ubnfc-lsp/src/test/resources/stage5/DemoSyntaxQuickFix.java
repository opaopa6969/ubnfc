import org.unlaxer.ubnfc.examples.stage5.generated.api.Diagnostic;
import org.unlaxer.ubnfc.examples.stage5.lsp.Analysis;
import org.unlaxer.ubnfc.examples.stage5.lsp.SyntaxQuickFix;

/** Stage5 の let 文末尾に不足したセミコロンを補う。 */
public final class DemoSyntaxQuickFix implements SyntaxQuickFix {
    @Override public Insertion forDiagnostic(Analysis document, Diagnostic diagnostic) {
        int end = document.positions().lengthCp();
        if ("syntax".equals(diagnostic.kind()) && "Let".equals(diagnostic.deepestRule())
                && diagnostic.offset() == end && diagnostic.length() == 0
                && diagnostic.expected().contains("';'"))
            return new Insertion("末尾に ; を挿入", ";");
        return null;
    }
}
