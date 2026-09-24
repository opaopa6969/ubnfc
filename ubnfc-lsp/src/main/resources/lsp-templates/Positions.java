package __PACKAGE__;

import java.util.ArrayList;
import java.util.List;
import org.eclipse.lsp4j.Position;
import org.eclipse.lsp4j.Range;

/**
 * 生成パーサの座標（Unicode code point・半開区間、IR の {@code contracts.spanUnit == "codePoint"}）と
 * LSP の座標（0 起点の行 × <b>UTF-16 code unit</b>、{@code positionEncoding} 既定）の相互変換。
 *
 * <p>変換はこの 1 箇所だけで行う。旧スタックの mapper は code point 数を UTF-16 index として
 * そのまま渡していたため、BMP 外（サロゲートペア）を含む行で位置がずれた。ここでは行頭を
 * <b>char index と code point index の両方</b>で持ち、行内は 1 code point ずつ
 * {@link Character#charCount} を足して数える。
 *
 * <p>行の区切りは LSP と同じく {@code \n} / {@code \r\n} / {@code \r}。
 */
public final class Positions {
    private final String text;
    private final int[] lineStartChar;
    private final int[] lineStartCp;
    private final int lengthCp;

    public Positions(String text) {
        this.text = text;
        var chars = new ArrayList<Integer>();
        var codePoints = new ArrayList<Integer>();
        chars.add(0);
        codePoints.add(0);
        int cp = 0;
        for (int index = 0; index < text.length();) {
            int point = text.codePointAt(index);
            index += Character.charCount(point);
            cp++;
            if (point == '\n') {
                chars.add(index);
                codePoints.add(cp);
            } else if (point == '\r') {
                if (index < text.length() && text.charAt(index) == '\n') {
                    index++;
                    cp++;
                }
                chars.add(index);
                codePoints.add(cp);
            }
        }
        this.lengthCp = cp;
        this.lineStartChar = toArray(chars);
        this.lineStartCp = toArray(codePoints);
    }

    private static int[] toArray(List<Integer> values) {
        int[] result = new int[values.size()];
        for (int i = 0; i < result.length; i++) result[i] = values.get(i);
        return result;
    }

    /** 文書全体の code point 数。 */
    public int lengthCp() { return lengthCp; }

    /** 行数（末尾が改行で終わる文書では「空の最終行」を 1 行として数える）。 */
    public int lineCount() { return lineStartChar.length; }

    /** code point 位置 → LSP {@link Position}。範囲外は文書の両端へ丸める。 */
    public Position positionOf(int offsetCp) {
        int target = Math.max(0, Math.min(offsetCp, lengthCp));
        int line = lineOf(target);
        int index = lineStartChar[line];
        for (int remaining = target - lineStartCp[line]; remaining > 0; remaining--)
            index += Character.charCount(text.codePointAt(index));
        return new Position(line, index - lineStartChar[line]);
    }

    /** LSP {@link Position} → code point 位置。サロゲートペアの内側を指したら次の code point へ丸める。 */
    public int offsetOf(Position position) {
        if (position == null) return 0;
        int line = Math.max(0, Math.min(position.getLine(), lineStartChar.length - 1));
        int limit = line + 1 < lineStartChar.length ? lineStartChar[line + 1] : text.length();
        int index = lineStartChar[line];
        int cp = lineStartCp[line];
        int character = Math.max(0, position.getCharacter());
        for (int units = 0; index < limit && units < character;) {
            int width = Character.charCount(text.codePointAt(index));
            index += width;
            units += width;
            cp++;
        }
        return cp;
    }

    /** code point の半開区間 → LSP {@link Range}。 */
    public Range rangeOf(int startCp, int endCp) {
        int start = Math.max(0, Math.min(startCp, lengthCp));
        int end = Math.max(start, Math.min(endCp, lengthCp));
        return new Range(positionOf(start), positionOf(end));
    }

    /** 行 {@code line} の UTF-16 幅（行末の改行を含まない）。 */
    public int columnsOf(int line) {
        if (line < 0 || line >= lineStartChar.length) return 0;
        int start = lineStartChar[line];
        int end = line + 1 < lineStartChar.length ? lineStartChar[line + 1] : text.length();
        while (end > start && (text.charAt(end - 1) == '\n' || text.charAt(end - 1) == '\r')) end--;
        return end - start;
    }

    private int lineOf(int offsetCp) {
        int low = 0;
        int high = lineStartCp.length - 1;
        while (low < high) {
            int mid = (low + high + 1) >>> 1;
            if (lineStartCp[mid] <= offsetCp) low = mid; else high = mid - 1;
        }
        return low;
    }
}
