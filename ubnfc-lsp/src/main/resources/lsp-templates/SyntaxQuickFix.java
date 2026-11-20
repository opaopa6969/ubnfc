package __PACKAGE__;

import __API_PACKAGE__.Diagnostic;

/** 言語側が主構文診断に明示的に対応付ける挿入。 */
@FunctionalInterface
public interface SyntaxQuickFix {
    /** 診断の開始位置に挿入する候補を返す。修正が一意に決まらなければ null。 */
    Insertion forDiagnostic(Analysis document, Diagnostic diagnostic);

    record Insertion(String title, String text) {}
}
