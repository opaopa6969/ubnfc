import java.util.ArrayDeque;
import java.util.List;
import java.util.regex.Pattern;
import org.unlaxer.ubnfc.examples.stage5.lsp.Analysis;
import org.unlaxer.ubnfc.examples.stage5.lsp.SignatureHelpProvider;

/** 数値・関数呼び出し・ブロックコメントだけを扱う署名アダプターの例。 */
public final class DemoSignatureHelp implements SignatureHelpProvider {
    private static final Pattern TOKEN = Pattern.compile("/\\*.*?\\*/|\\p{Alpha}\\w*|\\S", Pattern.DOTALL);

    private static final class Frame {
        final String name;
        final int start;
        int active;

        Frame(String name, int start) {
            this.name = name;
            this.start = start;
        }

        Call call(int end) {
            return switch (name) {
                case "sum" -> new Call(start, end, "sum(left, right)", List.of("left", "right"), active);
                case "inc" -> new Call(start, end, "inc(value)", List.of("value"), active);
                default -> null;
            };
        }
    }

    @Override public List<String> triggerCharacters() { return List.of("(", ","); }

    @Override public Call at(Analysis document, int cursorCp) {
        String text = document.text();
        var stack = new ArrayDeque<Frame>();
        var tokens = TOKEN.matcher(text);
        String previous = "";
        while (tokens.find()) {
            String token = tokens.group();
            if (token.startsWith("/*")) continue;
            int offset = text.codePointCount(0, tokens.start());
            switch (token) {
                case "(" -> stack.push(new Frame(previous, offset + 1));
                case "," -> {
                    if (!stack.isEmpty() && offset < cursorCp) stack.peek().active++;
                }
                case ")" -> {
                    if (!stack.isEmpty()) {
                        var frame = stack.pop();
                        if (frame.start <= cursorCp && cursorCp <= offset) return frame.call(offset);
                    }
                }
                default -> { }
            }
            previous = token;
        }
        for (var frame : stack)
            if (frame.start <= cursorCp) return frame.call(document.positions().lengthCp());
        return null;
    }
}
