package generated.internal;

/** 認識専用。成功時の UTF-16 終了位置だけを返し、途中結果は Session に書き込まない。 */
public final class Ll1Machine {
    private Ll1Machine() {}
    private static final int MAX_FRAMES = 256, MAX_CALLS = 32;
    private static final int SEQUENCE = 0, CHOICE = 1, OPTIONAL = 2, REPEAT = 3, CALL = 4,
        LITERAL = 5, CLASS = 6, EOF = 7, EMPTY = 8;

    @FunctionalInterface
    public interface Scan { long run(char[] input, int position); }
    public record Edge(int target, int[] ranges) {}
    public record Op(int kind, Scan scan, int[] data, Edge[] edges, int target, int min, int max,
                     String word, boolean boundary) {}

    public static Op sequence(Scan scan, int[] targets) {
        return new Op(SEQUENCE, scan, targets, null, 0, 0, 0, null, false);
    }
    public static Op choice(Scan scan, int probeDepth, Edge... edges) {
        return new Op(CHOICE, scan, null, edges, 0, 0, probeDepth, null, false);
    }
    public static Op optional(Scan scan, int target) {
        return new Op(OPTIONAL, scan, null, null, target, 0, 1, null, false);
    }
    public static Op repeat(Scan scan, int target, int min, int max) {
        return new Op(REPEAT, scan, null, null, target, min, max, null, false);
    }
    public static Op call(Scan scan, int target) {
        return new Op(CALL, scan, null, null, target, 0, 0, null, false);
    }
    public static Op literal(Scan scan, String word, boolean boundary) {
        return new Op(LITERAL, scan, null, null, 0, 0, 0, word, boundary);
    }
    public static Op characterClass(Scan scan, int[] ranges) {
        return new Op(CLASS, scan, ranges, null, 0, 0, 0, null, false);
    }
    public static Op eof(Scan scan) {
        return new Op(EOF, scan, null, null, 0, 0, 0, null, false);
    }
    public static Op empty(Scan scan) {
        return new Op(EMPTY, scan, null, null, 0, 0, 0, null, false);
    }

    private static boolean contains(int[] ranges, int cp) {
        for (int i = 0; i < ranges.length; i += 2) {
            if (cp < ranges[i]) return false;
            if (cp <= ranges[i + 1]) return true;
        }
        return false;
    }

    static int run(Op[] ops, int root, char[] input, int position, int remainingDepth, boolean scan) {
        // 続行命令、列の添字または反復回数、巻戻し位置を frame ごとに保持する。
        Op[] frames = new Op[MAX_FRAMES];
        int[] next = new int[MAX_FRAMES], start = new int[MAX_FRAMES];
        int size = 0, calls = 0, current = root, p = position;
        enter: while (size < MAX_FRAMES) {
            Op op = ops[current];
            boolean ok = true;
            if (scan && op.scan != null) {
                long result = op.scan.run(input, p);
                ok = result >= 0;
                if (ok) p = (int) (result >>> 1);
            } else switch (op.kind) {
                case SEQUENCE -> {
                    if (op.data.length == 0) break;
                    frames[size] = op;
                    next[size++] = 1;
                    current = op.data[0];
                    continue;
                }
                case CHOICE -> {
                    // 省く候補も、先頭文字で確実に失敗するまでに深さ上限を超え得る。
                    if (op.max > MAX_CALLS || op.max > remainingDepth - calls) return -1;
                    int cp = p < input.length ? Session.cpAt(input, p) : -1;
                    if (cp >= 0xd800 && cp <= 0xdfff) return -1;
                    for (Edge edge : op.edges) {
                        if (contains(edge.ranges, cp)) {
                            current = edge.target;
                            continue enter;
                        }
                    }
                    ok = false;
                }
                case OPTIONAL, REPEAT -> {
                    frames[size] = op;
                    start[size] = p;
                    next[size++] = 0;
                    current = op.target;
                    continue;
                }
                case CALL -> {
                    if (calls >= MAX_CALLS || calls >= remainingDepth) return -1;
                    calls++;
                    frames[size++] = op;
                    current = op.target;
                    continue;
                }
                case LITERAL -> {
                    int end = p + op.word.length();
                    ok = Session.startsWith(input, p, op.word)
                        && !(op.boundary && end < input.length && Session.identifierChar(Session.cpAt(input, end)));
                    if (ok) p = end;
                }
                case CLASS -> {
                    int cp = p < input.length ? Session.cpAt(input, p) : -1;
                    // Rust の scalar 区間外でも Java の参照経路は孤立 surrogate を読める。
                    if (cp >= 0xd800 && cp <= 0xdfff) return -1;
                    ok = contains(op.data, cp);
                    if (ok) p += Character.charCount(cp);
                }
                case EOF -> ok = p == input.length;
                case EMPTY -> { }
                default -> throw new AssertionError(op.kind);
            }
            while (size > 0) {
                int top = --size;
                Op frame = frames[top];
                switch (frame.kind) {
                    case CALL -> calls--;
                    case SEQUENCE -> {
                        if (ok && next[top] < frame.data.length) {
                            current = frame.data[next[top]++];
                            size++;
                            continue enter;
                        }
                    }
                    case OPTIONAL -> {
                        if (!ok) { p = start[top]; ok = true; }
                    }
                    case REPEAT -> {
                        if (ok) {
                            if (p == start[top]) return -1;
                            if (++next[top] < frame.max) {
                                start[top] = p;
                                current = frame.target;
                                size++;
                                continue enter;
                            }
                        } else {
                            p = start[top];
                        }
                        ok = next[top] >= frame.min;
                    }
                    default -> throw new AssertionError(frame.kind);
                }
            }
            return ok ? p : -1;
        }
        return -1;
    }
}
