package target.internal;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import target.api.*;
import target.rt.Diagnostics;
import target.rt.Input;
import target.rt.MemoTable;

/** 生成時に埋め込める共通部分。rule の選択・trivia 配置は各 parser に明記する。 */
public final class Session {
    public static final ThreadLocal<Session> CURRENT = new ThreadLocal<>();
    public static final int MISS = -2;
    public static final int NUMBER = 0, PLUS = 1, MINUS = 2, STAR = 3, SLASH = 4,
        OPEN = 5, CLOSE = 6, CARET = 7, EOF = 8, BLOCK_END = 9;
    private static final String[] EXPECTED = {
        "NUMBER", "'+'", "'-'", "'*'", "'/'", "'('", "')'", "'^'", "EOF", "'*/'"
    };
    public final String source;
    public final Input input;
    public final ParseOptions options;
    public final Diagnostics diagnostics = new Diagnostics();
    public final MemoTable memo = new MemoTable();
    private final String[] rules;
    private final ArrayList<Payload> payloads = new ArrayList<>();
    public int consumed, matched;
    public Recipe recipe;
    public Events events;
    public int memoHits;
    private record Payload(Recipe recipe, Events events, Diagnostics.Snapshot diagnostics) {}

    public Session(String source, ParseOptions options, String... rules) {
        this.source = source;
        this.input = new Input(source);
        this.options = options;
        this.rules = rules.clone();
    }

    public int begin(int rule, int c, int m) {
        int slot = memo.get(rule, c, m, options.buildAst() ? 1 : 0, 0);
        if (slot >= 0) {
            memoHits++;
            var payload = payloads.get(memo.payload(slot));
            diagnostics.merge(payload.diagnostics()); // enter 前に呼出元 stack を付け直す。
            consumed = memo.consumedEnd(slot);
            matched = memo.matchedEnd(slot);
            recipe = payload.recipe(); events = payload.events();
            return memo.ok(slot) ? 0 : -1;
        }
        diagnostics.enterRule(rule);
        consumed = c; matched = m; recipe = null; events = null;
        return MISS;
    }

    public int end(int rule, int c, int m, int status) {
        if (status < 0) { consumed = c; matched = m; recipe = null; events = null; }
        int handle = payloads.size();
        payloads.add(new Payload(recipe, events, diagnostics.leaveRule()));
        memo.put(rule, c, m, options.buildAst() ? 1 : 0, 0,
            status == 0, consumed, matched, handle);
        return status;
    }

    public void advanceTo(int end) {
        matched += end - consumed;
        consumed = end;
    }

    public int cp(int at) { return at < input.length() ? input.codePointAt(at) : -1; }
    private boolean digit(int at) { int ch = cp(at); return ch >= '0' && ch <= '9'; }

    /** NUMBER の optional exponent は不成立なら mantissa 末尾へ戻す。 */
    public boolean number() {
        int start = consumed, p = start;
        if (cp(p) == '+' || cp(p) == '-') p++;
        int digits = p;
        while (digit(p)) p++;
        boolean hasDigits = p > digits;
        if (cp(p) == '.') {
            p++;
            int fraction = p;
            while (digit(p)) p++;
            if (!hasDigits && p == fraction) { diagnostics.failAt(p, NUMBER); return false; }
        } else if (!hasDigits) { diagnostics.failAt(p, NUMBER); return false; }
        int mantissa = p;
        if (cp(p) == 'e' || cp(p) == 'E') {
            p++;
            if (cp(p) == '+' || cp(p) == '-') p++;
            int exponent = p;
            while (digit(p)) p++;
            if (p == exponent) { diagnostics.failAt(p, NUMBER); p = mantissa; }
        }
        advanceTo(p);
        return true;
    }

    public boolean literal(String text, int expected) {
        if (input.startsWith(consumed, text)) { advanceTo(consumed + text.length()); return true; }
        diagnostics.failAt(consumed, expected);
        return false;
    }

    /** javaStyle = ASCII 6 空白 + // + 非入れ子 block。失敗 comment は消費しない。 */
    public void trivia() {
        int p = consumed;
        while (true) {
            int ch = cp(p);
            if (ch == ' ' || ch >= 9 && ch <= 13) { p++; continue; }
            if (input.startsWith(p, "//")) {
                p += 2;
                while (cp(p) >= 0 && cp(p) != '\r' && cp(p) != '\n')
                    p += Character.charCount(cp(p));
                continue;
            }
            if (input.startsWith(p, "/*")) {
                int end = p + 2;
                while (end < input.length() && !input.startsWith(end, "*/"))
                    end += Character.charCount(cp(end));
                if (end == input.length()) { diagnostics.failAt(end, BLOCK_END); break; }
                p = end + 2;
                continue;
            }
            break;
        }
        advanceTo(p);
    }

    public Events capture(Events previous, int site, String name, int start, int end) {
        return options.buildAst() ? Events.concat(previous, Events.capture(site, name, start, end)) : null;
    }

    public Recipe leaf(Recipe.Kind kind, int start, int end, int valueStart, int valueEnd) {
        return options.buildAst()
            ? new Recipe(kind, start, end, valueStart, valueEnd, null, List.of(), List.of()) : null;
    }

    public Span span(int start, int end) { return new Span(input.cpIndex(start), input.cpIndex(end)); }

    public Diagnostic diagnostic() {
        return new Diagnostic(diagnostics.farthest() < 0 ? -1 : input.cpIndex(diagnostics.farthest()),
            Arrays.stream(diagnostics.expected()).mapToObj(i -> EXPECTED[i]).toList(),
            diagnostics.deepestRule() < 0 ? null : rules[diagnostics.deepestRule()],
            Arrays.stream(diagnostics.ruleStack()).mapToObj(i -> rules[i]).toList());
    }

    public <N> ParseResult<N> result(int status) {
        boolean ok = status == 0;
        if (ok && options.requireFullInput() && consumed != input.length()) {
            diagnostics.failAt(consumed, EOF);
            ok = false;
        }
        return new ParseResult<>(ok, input.cpIndex(consumed), input.cpIndex(matched), Optional.empty(),
            Map.of(), ok ? Events.materialize(events, input) : List.of(), diagnostic());
    }
}
