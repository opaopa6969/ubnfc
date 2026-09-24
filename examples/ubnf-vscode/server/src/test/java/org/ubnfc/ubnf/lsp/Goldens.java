package org.ubnfc.ubnf.lsp;

import java.util.ArrayList;
import java.util.List;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DocumentSymbol;
import org.eclipse.lsp4j.Hover;
import org.eclipse.lsp4j.Location;
import org.eclipse.lsp4j.Range;
import org.eclipse.lsp4j.SymbolInformation;
import org.eclipse.lsp4j.jsonrpc.messages.Either;

/**
 * golden の書き出し。依存を足さないため小さな JSON writer を手書きする（並びは常に同じ）。
 */
final class Goldens {
    private Goldens() {}

    static String diagnostics(List<Diagnostic> diagnostics) {
        var out = new StringBuilder("[\n");
        for (int i = 0; i < diagnostics.size(); i++) {
            var diagnostic = diagnostics.get(i);
            var range = diagnostic.getRange();
            out.append("  {\n")
               .append("    \"range\": [").append(range.getStart().getLine()).append(", ")
               .append(range.getStart().getCharacter()).append(", ")
               .append(range.getEnd().getLine()).append(", ")
               .append(range.getEnd().getCharacter()).append("],\n")
               .append("    \"severity\": \"").append(diagnostic.getSeverity()).append("\",\n")
               .append("    \"source\": ").append(quote(diagnostic.getSource())).append(",\n")
               .append("    \"code\": ").append(quote(diagnostic.getCode() == null ? null : diagnostic.getCode().getLeft())).append(",\n")
               .append("    \"message\": ").append(quote(diagnostic.getMessage())).append('\n')
               .append("  }").append(i + 1 < diagnostics.size() ? "," : "").append('\n');
        }
        return out.append("]\n").toString();
    }

    static String range(Range range) {
        return "[" + range.getStart().getLine() + ", " + range.getStart().getCharacter() + ", "
            + range.getEnd().getLine() + ", " + range.getEnd().getCharacter() + "]";
    }

    static String locations(List<? extends Location> locations) {
        var items = new ArrayList<String>();
        for (Location location : locations) items.add(range(location.getRange()));
        return "[" + String.join(", ", items) + "]";
    }

    static String hover(Hover hover) {
        if (hover == null) return "null";
        String text = hover.getContents() == null || hover.getContents().getRight() == null ? null
            : hover.getContents().getRight().getValue();
        return "{\"range\": " + (hover.getRange() == null ? "null" : range(hover.getRange()))
            + ", \"markdown\": " + quote(text) + "}";
    }

    static String symbols(List<Either<SymbolInformation, DocumentSymbol>> symbols, String indent) {
        var items = new ArrayList<String>();
        for (var symbol : symbols) if (symbol.isRight()) items.add(symbol(symbol.getRight(), indent + "  "));
        return list(items, indent);
    }

    private static String symbol(DocumentSymbol symbol, String indent) {
        var out = new StringBuilder("{\"name\": ").append(quote(symbol.getName()))
            .append(", \"kind\": \"").append(symbol.getKind()).append('"')
            .append(", \"detail\": ").append(quote(symbol.getDetail()))
            .append(", \"range\": ").append(range(symbol.getRange()))
            .append(", \"selectionRange\": ").append(range(symbol.getSelectionRange()));
        if (symbol.getChildren() != null && !symbol.getChildren().isEmpty()) {
            var children = new ArrayList<String>();
            for (var child : symbol.getChildren()) children.add(symbol(child, indent + "  "));
            out.append(", \"children\": ").append(list(children, indent));
        }
        return out.append('}').toString();
    }

    /** semantic tokens の 5 つ組を人が読める形に戻す。 */
    static String semanticTokens(List<Integer> data, List<String> legend, String indent) {
        var items = new ArrayList<String>();
        int line = 0;
        int column = 0;
        for (int i = 0; i + 4 < data.size(); i += 5) {
            int deltaLine = data.get(i);
            line += deltaLine;
            column = deltaLine == 0 ? column + data.get(i + 1) : data.get(i + 1);
            String type = data.get(i + 3) < legend.size() ? legend.get(data.get(i + 3)) : "?";
            items.add("{\"line\": " + line + ", \"char\": " + column + ", \"length\": " + data.get(i + 2)
                + ", \"type\": " + quote(type) + "}");
        }
        return list(items, indent);
    }

    static String list(List<String> items, String indent) {
        if (items.isEmpty()) return "[]";
        var out = new StringBuilder("[\n");
        for (int i = 0; i < items.size(); i++)
            out.append(indent).append("  ").append(items.get(i)).append(i + 1 < items.size() ? "," : "").append('\n');
        return out.append(indent).append(']').toString();
    }

    static String strings(List<String> values) {
        var items = new ArrayList<String>();
        for (String value : values) items.add(quote(value));
        return "[" + String.join(", ", items) + "]";
    }

    static String quote(String value) {
        if (value == null) return "null";
        var out = new StringBuilder("\"");
        value.codePoints().forEach(point -> {
            switch (point) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                default -> {
                    if (point < 0x20) out.append(String.format("\\u%04x", point));
                    else out.appendCodePoint(point);
                }
            }
        });
        return out.append('"').toString();
    }
}
