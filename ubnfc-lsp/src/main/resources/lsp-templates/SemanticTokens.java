package __PACKAGE__;

import java.util.ArrayList;
import java.util.List;
import org.eclipse.lsp4j.SemanticTokensLegend;

/**
 * {@code ParseResult.lexical()} の終端を LSP の semantic tokens に写す。
 *
 * <p>種別は {@link Meta#TOKEN_TYPE}（式 → 種別）で引く。写像は生成時に IR から作ってあり、
 * literal は「語として読めるか」で keyword / operator、tokenRef は token の種別
 * （builtin Number → number、Identifier → variable、引用符始まりの extern → string …）で決まる。
 *
 * <p>LSP の token は<b>行をまたげない</b>ので、複数行にまたがる終端（コードブロック等）は行ごとに割る。
 */
public final class SemanticTokens {
    private SemanticTokens() {}

    public static SemanticTokensLegend legend() {
        return new SemanticTokensLegend(new ArrayList<>(Meta.TOKEN_TYPES), List.of());
    }

    /** legend の索引。未知の種別は -1。 */
    static int index(String type) { return Meta.TOKEN_TYPES.indexOf(type); }

    /** LSP の 5 つ組（deltaLine, deltaStartChar, length, tokenType, tokenModifiers）の並び。 */
    public static List<Integer> data(Analysis analysis) {
        var positions = analysis.positions();
        record Item(int line, int column, int length, int type) {}
        var items = new ArrayList<Item>();
        int previousEnd = -1;
        var terminals = new ArrayList<>(analysis.lexical());
        terminals.sort(java.util.Comparator.comparingInt((__API_PACKAGE__.ParseResult.Lexical item) -> item.span().start())
            .thenComparingInt(item -> item.span().end()));
        for (var terminal : terminals) {
            if (terminal.token() == null) continue;
            int type = index(Meta.TOKEN_TYPE.get(terminal.exprId()));
            if (type < 0) continue;
            int start = terminal.span().start();
            int end = terminal.span().end();
            if (end <= start || start < previousEnd) continue;
            previousEnd = end;
            var from = positions.positionOf(start);
            var to = positions.positionOf(end);
            if (from.getLine() == to.getLine()) {
                items.add(new Item(from.getLine(), from.getCharacter(), to.getCharacter() - from.getCharacter(), type));
                continue;
            }
            // 行をまたぐ終端は行ごとに割る。
            for (int line = from.getLine(); line <= to.getLine(); line++) {
                int column = line == from.getLine() ? from.getCharacter() : 0;
                int width = line == to.getLine() ? to.getCharacter() - column : lineWidth(positions, line) - column;
                if (width > 0) items.add(new Item(line, column, width, type));
            }
        }
        var out = new ArrayList<Integer>(items.size() * 5);
        int line = 0;
        int column = 0;
        for (Item item : items) {
            int deltaLine = item.line() - line;
            out.add(deltaLine);
            out.add(deltaLine == 0 ? item.column() - column : item.column());
            out.add(item.length());
            out.add(item.type());
            out.add(0);
            line = item.line();
            column = item.column();
        }
        return out;
    }

    /** 行の UTF-16 幅（改行を含まない）。 */
    private static int lineWidth(Positions positions, int line) {
        return positions.columnsOf(line);
    }
}
