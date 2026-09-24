package __PACKAGE__;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.eclipse.lsp4j.CompletionItem;
import org.eclipse.lsp4j.CompletionItemKind;
import org.eclipse.lsp4j.Position;
import __API_PACKAGE__.ParseOptions;
import __API_PACKAGE__.TokenScanner;

/**
 * カーソル位置の補完候補。
 *
 * <p>生成パーサは「この位置で何が来られるか」を返す API を持たない。<b>契約を変えずに</b>同じものを得るため、
 * <b>カーソル直前の識別子を取り除いた前半だけを 1 つの文書として解析する</b>。
 * 前半は必ずカーソル位置で失敗するので（全入力要求つき）、主診断の {@code expected} ラベルが
 * そのまま「ここに書ける終端」になる（D-023 の二段構えで、失敗時の診断は生成パーサ自身が
 * 決定的に再解析して返すものと一致する）。
 *
 * <p>それに、カーソルより前で宣言済みの記号（{@code scope().declarations()}）を足し、
 * カーソル下の識別子の綴りで絞る。
 */
public final class Completion {
    private Completion() {}

    /** 識別子の一部として扱う code point。 */
    static boolean identifier(int codePoint) {
        return Character.isLetterOrDigit(codePoint) || codePoint == '_';
    }

    /** カーソルから左へ識別子を遡った開始位置（code point）。 */
    static int wordStart(String text, int cursorCp) {
        int[] points = text.codePoints().toArray();
        int start = Math.max(0, Math.min(cursorCp, points.length));
        while (start > 0 && identifier(points[start - 1])) start--;
        return start;
    }

    /** カーソル下の綴り。 */
    static String word(String text, int startCp, int cursorCp) {
        int[] points = text.codePoints().toArray();
        int end = Math.max(0, Math.min(cursorCp, points.length));
        var out = new StringBuilder();
        for (int i = Math.max(0, startCp); i < end; i++) out.appendCodePoint(points[i]);
        return out.toString();
    }

    public static List<CompletionItem> at(Analysis analysis, Position position, Map<String, TokenScanner> scanners) {
        int cursor = analysis.positions().offsetOf(position);
        int start = wordStart(analysis.text(), cursor);
        String prefix = word(analysis.text(), start, cursor);
        var items = new LinkedHashMap<String, CompletionItem>();
        for (String label : expected(analysis.text(), start, scanners)) {
            String literal = literal(label);
            if (literal == null || literal.isBlank() || Meta.TRIVIA_LITERALS.contains(literal)) continue;
            var item = new CompletionItem(literal);
            item.setKind(keyword(literal) ? CompletionItemKind.Keyword : CompletionItemKind.Operator);
            item.setDetail("ここに書ける終端");
            items.putIfAbsent(literal, item);
        }
        for (var declaration : analysis.symbols().all()) {
            if (!declaration.declaration() || declaration.offsetCp() >= start) continue;
            var item = new CompletionItem(declaration.name());
            item.setKind(CompletionItemKind.Variable);
            item.setDetail("宣言済みの記号");
            items.put(declaration.name(), item);
        }
        var out = new ArrayList<CompletionItem>();
        for (var item : items.values())
            if (prefix.isEmpty() || item.getLabel().regionMatches(true, 0, prefix, 0, prefix.length())) out.add(item);
        out.sort(java.util.Comparator.comparing(CompletionItem::getLabel));
        return out;
    }

    /**
     * 前半だけを解析して得た「期待した終端」のラベル。
     *
     * <p>診断観測を明示的に要求する（{@code diagnostics=true}）。認識専用で解析すると、
     * D-023 のとおり<b>成功した解析の hints は空</b>になるが、補完が欲しいのはまさにその hints だからである。
     * 主 {@code expected} ではなく {@code farthestExpected}（最遠到達位置で期待していた集合）を優先するのは、
     * 入口が {@code { X }} のような 0 回以上の繰返しだと前半が「0 件で成功 → 末尾入力」になり、
     * 主 {@code expected} が {@code end of input} だけになるため。
     */
    static List<String> expected(String text, int cutCp, Map<String, TokenScanner> scanners) {
        String head = Analysis.head(text, cutCp);
        try {
            var result = __PARSER_CLASS__.parseEntry(LanguageServerImpl.GRAMMAR, LanguageServerImpl.ENTRY, head,
                new ParseOptions(false, true, false, true, scanners, ParseOptions.DEFAULT_MAX_DEPTH, false, true));
            var diagnostic = result.diagnostics();
            if (diagnostic == null) return List.of();
            return diagnostic.farthestExpected().isEmpty() ? diagnostic.expected() : diagnostic.farthestExpected();
        } catch (RuntimeException error) {
            return List.of();
        }
    }

    /** {@code 'import'} のような literal ラベルから中身を取り出す。終端の種別名（NUMBER 等）は候補にしない。 */
    static String literal(String label) {
        if (label == null || label.length() < 2) return null;
        if (label.charAt(0) != '\'' || label.charAt(label.length() - 1) != '\'') return null;
        return label.substring(1, label.length() - 1);
    }

    static boolean keyword(String literal) {
        return Meta.KEYWORDS.contains(literal);
    }
}
