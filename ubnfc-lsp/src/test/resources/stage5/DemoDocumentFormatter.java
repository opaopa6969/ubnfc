import java.util.Comparator;
import java.util.regex.Pattern;
import org.unlaxer.ubnfc.examples.stage5.generated.api.ParseResult;
import org.unlaxer.ubnfc.examples.stage5.lsp.Analysis;
import org.unlaxer.ubnfc.examples.stage5.lsp.DocumentFormatter;

/** stage5.ubnf の javaStyle trivia と文・ブロックの書式。 */
public final class DemoDocumentFormatter implements DocumentFormatter {
    private static final Pattern TRIVIA = Pattern.compile("\\p{javaWhitespace}+|//[^\\r\\n]*|/\\*.*?\\*/", Pattern.DOTALL);

    @Override public String format(Analysis document) {
        String source = document.text();
        var tokens = document.lexical().stream()
            .filter(item -> item.token() != null && !item.token().isEmpty())
            .sorted(Comparator.comparingInt((ParseResult.Lexical item) -> item.span().start()))
            .toList();
        var out = new StringBuilder();
        int previousCp = 0;
        int previousChar = 0;
        int depth = 0;
        for (var item : tokens) {
            var span = item.span();
            int start = source.offsetByCodePoints(previousChar, span.start() - previousCp);
            if (!comments(source, previousChar, start, depth, out)) return null;
            String token = item.token();
            if (token.equals("}")) depth--;
            if (lineStart(out)) {
                out.append("  ".repeat(depth));
            } else if (!token.equals(";")) {
                out.append(' ');
            }
            out.append(token);
            if (token.equals("{")) depth++;
            if (token.equals(";") || token.equals("{") || token.equals("}")) out.append('\n');
            previousCp = span.end();
            previousChar = source.offsetByCodePoints(start, span.end() - span.start());
        }
        return comments(source, previousChar, source.length(), depth, out) ? out.toString() : null;
    }

    private static boolean comments(String source, int start, int end, int depth, StringBuilder out) {
        var trivia = TRIVIA.matcher(source).region(start, end);
        int offset = start;
        while (trivia.lookingAt()) {
            String item = trivia.group();
            if (item.startsWith("/")) {
                if (!lineStart(out)) out.append('\n');
                out.append("  ".repeat(depth)).append(item).append('\n');
            }
            offset = trivia.end();
            trivia.region(offset, end);
        }
        return offset == end;
    }

    private static boolean lineStart(StringBuilder out) {
        return out.isEmpty() || out.charAt(out.length() - 1) == '\n';
    }
}
