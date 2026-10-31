package __PACKAGE__;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import org.eclipse.lsp4j.FoldingRange;
import org.eclipse.lsp4j.FoldingRangeKind;

/** scope の enter/leave と、終端間にある trivia のブロックコメントを折り畳む。 */
public final class FoldingRanges {
    private FoldingRanges() {}

    public static List<FoldingRange> of(Analysis analysis) {
        var out = new ArrayList<FoldingRange>();
        var scope = analysis.scope();
        if (scope != null) {
            var stack = new ArrayDeque<Integer>();
            for (var event : scope.events()) {
                if ("enter".equals(event.action())) stack.push(event.offsetCp());
                else if ("leave".equals(event.action()) && !stack.isEmpty())
                    add(out, analysis.positions(), stack.pop(), event.offsetCp(), null);
            }
        }
        // 成功した終端は文字列も含めて不可侵。trivia は公開 trace に載らないので隙間だけを読む。
        if (analysis.scoped() != null) {
            var terminals = analysis.lexical().stream().filter(item -> item.token() != null)
                .sorted(Comparator.comparingInt((__API_PACKAGE__.ParseResult.Lexical item) -> item.span().start()))
                .toList();
            int previous = 0;
            for (var terminal : terminals) {
                int start = terminal.span().start();
                if (start > previous) comments(analysis, previous, start, out);
                previous = Math.max(previous, terminal.span().end());
            }
            int end = analysis.scoped().consumedCp();
            if (end > previous) comments(analysis, previous, end, out);
        }
        out.sort(Comparator.comparingInt(FoldingRange::getStartLine)
            .thenComparingInt(FoldingRange::getStartCharacter)
            .thenComparingInt(FoldingRange::getEndLine).thenComparingInt(FoldingRange::getEndCharacter));
        return out.stream().distinct().toList();
    }

    /** 終端間の隙間。途中に未知の文字があれば推測で先へ進まない。 */
    private static void comments(Analysis analysis, int startCp, int endCp, List<FoldingRange> out) {
        String text = analysis.text();
        int start = text.offsetByCodePoints(0, startCp);
        int end = text.offsetByCodePoints(0, endCp);
        for (int at = start; at < end;) {
            int point = text.codePointAt(at);
            if (Character.isWhitespace(point)) { at += Character.charCount(point); continue; }
            int next = at;
            for (var delimiter : Meta.COMMENT_DELIMITERS) {
                if (!text.startsWith(delimiter.open(), at)) continue;
                if (delimiter.close() == null) {
                    next = at + delimiter.open().length();
                    while (next < end && text.charAt(next) != '\n' && text.charAt(next) != '\r') next++;
                } else {
                    int close = text.indexOf(delimiter.close(), at + delimiter.open().length());
                    if (close < 0 || close + delimiter.close().length() > end) return;
                    next = close + delimiter.close().length();
                    add(out, analysis.positions(), text.codePointCount(0, at), text.codePointCount(0, next),
                        FoldingRangeKind.Comment);
                }
                break;
            }
            if (next <= at) return;
            at = next;
        }
    }

    /** 半開区間の終端が次の行頭なら、その前の行までを折り畳む。1 行の範囲は出さない。 */
    private static void add(List<FoldingRange> out, Positions positions, int start, int end, String kind) {
        if (end <= start) return;
        var from = positions.positionOf(start);
        var to = positions.positionOf(end);
        int endLine = to.getLine();
        int endCharacter = to.getCharacter();
        if (endCharacter == 0 && endLine > from.getLine()) {
            endLine--;
            endCharacter = positions.columnsOf(endLine);
        }
        if (endLine <= from.getLine()) return;
        var range = new FoldingRange(from.getLine(), endLine);
        range.setStartCharacter(from.getCharacter());
        range.setEndCharacter(endCharacter);
        range.setKind(kind);
        out.add(range);
    }
}
