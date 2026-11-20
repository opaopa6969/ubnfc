package __PACKAGE__;

/** 言語固有の書式規則で、構文上正常な文書を整形する。 */
@FunctionalInterface
public interface DocumentFormatter {
    /**
     * コメント、文字列、意味を保持した全文を返す。
     * 適用できない場合は null、すでに整形済みなら元の全文を返す。
     */
    String format(Analysis document);
}
