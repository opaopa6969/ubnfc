package __PACKAGE__;

import java.util.ArrayList;
import org.eclipse.lsp4j.Hover;
import org.eclipse.lsp4j.MarkupContent;
import org.eclipse.lsp4j.MarkupKind;
import org.eclipse.lsp4j.Position;

/**
 * カーソル下の記号・終端の説明。
 *
 * <p>説明文の出所は 3 段。
 * <ol>
 *   <li>規則の {@code @doc('…')} — IR の {@code semantics.rules[].annotations} に既に載っており、
 *       {@link Meta#RULE_DOC} として生成時に畳み込んである</li>
 *   <li>{@code @declares(description='…')}</li>
 *   <li>いずれも無ければ、記号の種類と宣言位置だけを出す</li>
 * </ol>
 */
public final class Hovers {
    private Hovers() {}

    public static Hover at(Analysis analysis, Position position) {
        int offset = analysis.positions().offsetOf(position);
        var symbols = analysis.symbols();
        var symbol = symbols.at(offset);
        var lines = new ArrayList<String>();
        int startCp;
        int endCp;
        if (symbol != null) {
            startCp = symbol.offsetCp();
            endCp = symbol.endCp();
            lines.add("**" + symbol.name() + "** — " + (symbol.declaration() ? "宣言" : "参照"));
            String ruleId = symbols.ruleAt(symbol.offsetCp());
            var declarations = symbols.definitions(symbol);
            if (!symbol.declaration()) {
                if (declarations.isEmpty()) lines.add("宣言が見つかりません。");
                else for (var declaration : declarations) {
                    var at = analysis.positions().positionOf(declaration.offsetCp());
                    lines.add("宣言: " + (at.getLine() + 1) + " 行 " + (at.getCharacter() + 1) + " 桁");
                    if (ruleId == null) ruleId = symbols.ruleAt(declaration.offsetCp());
                }
            }
            String kind = Meta.RULE_KIND.get(ruleId);
            if (kind != null) lines.add("種類: `" + kind + "`");
            String documentation = documentation(ruleId);
            if (documentation != null) lines.add(documentation);
        } else {
            var terminal = symbols.terminalAt(offset);
            if (terminal == null) return null;
            startCp = terminal.span().start();
            endCp = terminal.span().end();
            lines.add("`" + terminal.token() + "`");
            String rule = Meta.localName(terminal.ruleId());
            if (rule != null) lines.add("規則: `" + rule + "`");
            String documentation = documentation(terminal.ruleId());
            if (documentation != null) lines.add(documentation);
        }
        var hover = new Hover(new MarkupContent(MarkupKind.MARKDOWN, String.join("\n\n", lines)));
        hover.setRange(analysis.positions().rangeOf(startCp, endCp));
        return hover;
    }

    private static String documentation(String ruleId) {
        String text = Meta.documentation(ruleId);
        return text == null || text.isBlank() ? null : text;
    }
}
