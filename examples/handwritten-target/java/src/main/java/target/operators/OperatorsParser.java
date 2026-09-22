package target.operators;

import java.util.ArrayList;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Optional;
import target.api.*;
import target.internal.*;

/** Operators.ubnf の保守的な段別 lowering。戻り値 0/-1、両カーソル等は Session の出力欄。 */
public final class OperatorsParser {
    private OperatorsParser() {}
    private static final int EXPRESSION = 0, TERM = 1, FACTOR = 2, NUMERIC = 3,
        ADD = 4, MUL = 5, NUMBER_GROUP = 6, EXPRESSION_TAIL = 7, TERM_TAIL = 8;

    public static ParseResult<OperatorsAST> parse(CharSequence source) {
        return parse(source, ParseOptions.DEFAULT);
    }

    public static ParseResult<OperatorsAST> parse(CharSequence source, ParseOptions options) {
        var s = new Session(source.toString(), options, "Expression", "Term", "Factor", "NumericLiteral",
            "AddOp", "MulOp", "NumericLiteral.group", "Expression.repeat", "Term.repeat");
        Session previous = Session.CURRENT.get();
        Session.CURRENT.set(s);
        try {
            ParseResult<OperatorsAST> syntax = s.result(parseExpression(0, 0));
            if (!syntax.ok() || !options.buildAst()) return syntax;
            var spans = new IdentityHashMap<OperatorsAST, Span>();
            OperatorsAST ast = build(s.recipe, s, syntax, spans);
            return new ParseResult<>(true, syntax.consumedCp(), syntax.matchedCp(), Optional.of(ast),
                spans, syntax.captures(), syntax.diagnostics());
        } finally {
            if (previous == null) Session.CURRENT.remove(); else Session.CURRENT.set(previous);
        }
    }

    private static OperatorsAST build(Recipe recipe, Session s, ParseResult<?> syntax,
                                       IdentityHashMap<OperatorsAST, Span> spans) {
        OperatorsAST node;
        if (recipe.kind == Recipe.Kind.NUMBER) {
            int value;
            try {
                // value capture の trivia span と、NUMBER token の変換範囲は別に保持する。
                value = Integer.parseInt(s.source.substring(recipe.valueStart, recipe.valueEnd));
            } catch (NumberFormatException error) {
                throw new MappingException(syntax, s.span(recipe.valueStart, recipe.valueEnd), error);
            }
            node = new OperatorsAST.Number(value);
        } else {
            OperatorsAST left = build(recipe.left, s, syntax, spans);
            var right = new ArrayList<OperatorsAST>(recipe.right.size());
            for (Recipe item : recipe.right) right.add(build(item, s, syntax, spans));
            node = new OperatorsAST.Binary(left, recipe.operators, right);
        }
        spans.put(node, s.span(recipe.start, recipe.end));
        return node;
    }

    static int parseExpression(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(EXPRESSION, consumed, matched);
        if (memo != Session.MISS) return memo;
        s.trivia();
        int start = s.consumed;
        if (parseTerm(s.consumed, s.matched) < 0) return s.end(EXPRESSION, consumed, matched, -1);
        Recipe left = s.recipe;
        Events events = s.capture(s.events, 0, "left", start, s.consumed);
        s.trivia();
        var ops = s.options.buildAst() ? new ArrayList<String>() : null;
        var rights = s.options.buildAst() ? new ArrayList<Recipe>() : null;
        int count = 0;
        while (true) {
            int before = s.consumed;
            if (parseExpressionTail(s.consumed, s.matched) < 0) break;
            if (s.options.buildAst()) {
                ops.add(s.recipe.operators.getFirst());
                rights.add(s.recipe.right.getFirst());
                events = Events.concat(events, s.events);
            }
            count++; // 成功したゼロ幅の周回も一回と数えてから止める (§3.3)。
            if (!RepeatProgress.more(count, before, s.consumed, Integer.MAX_VALUE)) break;
        }
        s.trivia();
        s.recipe = s.options.buildAst() ? new Recipe(Recipe.Kind.BINARY, consumed, s.consumed,
            0, 0, left, ops, rights) : null;
        s.events = events;
        return s.end(EXPRESSION, consumed, matched, 0);
    }

    static int parseTerm(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(TERM, consumed, matched);
        if (memo != Session.MISS) return memo;
        s.trivia();
        int start = s.consumed;
        if (parseFactor(s.consumed, s.matched) < 0) return s.end(TERM, consumed, matched, -1);
        Recipe left = s.recipe;
        Events events = s.capture(s.events, 3, "left", start, s.consumed);
        s.trivia();
        var ops = s.options.buildAst() ? new ArrayList<String>() : null;
        var rights = s.options.buildAst() ? new ArrayList<Recipe>() : null;
        int count = 0;
        while (true) {
            int before = s.consumed;
            if (parseTermTail(s.consumed, s.matched) < 0) break;
            if (s.options.buildAst()) {
                ops.add(s.recipe.operators.getFirst());
                rights.add(s.recipe.right.getFirst());
                events = Events.concat(events, s.events);
            }
            count++; // 成功したゼロ幅の周回も一回と数えてから止める (§3.3)。
            if (!RepeatProgress.more(count, before, s.consumed, Integer.MAX_VALUE)) break;
        }
        s.trivia();
        s.recipe = s.options.buildAst() ? new Recipe(Recipe.Kind.BINARY, consumed, s.consumed,
            0, 0, left, ops, rights) : null;
        s.events = events;
        return s.end(TERM, consumed, matched, 0);
    }

    static int parseExpressionTail(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(EXPRESSION_TAIL, consumed, matched);
        if (memo != Session.MISS) return memo;
        s.trivia();
        int opStart = s.consumed;
        if (parseAddOp(s.consumed, s.matched) < 0) return s.end(EXPRESSION_TAIL, consumed, matched, -1);
        int opEnd = s.consumed;
        Events events = s.capture(s.events, 1, "op", opStart, opEnd);
        s.trivia();
        int rightStart = s.consumed;
        if (parseTerm(s.consumed, s.matched) < 0) return s.end(EXPRESSION_TAIL, consumed, matched, -1);
        Recipe right = s.recipe;
        events = s.capture(Events.concat(events, s.events), 2, "right", rightStart, s.consumed);
        s.trivia();
        s.recipe = s.options.buildAst() ? new Recipe(Recipe.Kind.BINARY, consumed, s.consumed, 0, 0,
            null, List.of(s.source.substring(opStart, opEnd)), List.of(right)) : null;
        s.events = events;
        return s.end(EXPRESSION_TAIL, consumed, matched, 0);
    }

    static int parseTermTail(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(TERM_TAIL, consumed, matched);
        if (memo != Session.MISS) return memo;
        s.trivia();
        int opStart = s.consumed;
        if (parseMulOp(s.consumed, s.matched) < 0) return s.end(TERM_TAIL, consumed, matched, -1);
        int opEnd = s.consumed;
        Events events = s.capture(s.events, 4, "op", opStart, opEnd);
        s.trivia();
        int rightStart = s.consumed;
        if (parseFactor(s.consumed, s.matched) < 0) return s.end(TERM_TAIL, consumed, matched, -1);
        Recipe right = s.recipe;
        events = s.capture(Events.concat(events, s.events), 5, "right", rightStart, s.consumed);
        s.trivia();
        s.recipe = s.options.buildAst() ? new Recipe(Recipe.Kind.BINARY, consumed, s.consumed, 0, 0,
            null, List.of(s.source.substring(opStart, opEnd)), List.of(right)) : null;
        s.events = events;
        return s.end(TERM_TAIL, consumed, matched, 0);
    }

    static int parseFactor(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(FACTOR, consumed, matched);
        if (memo != Session.MISS) return memo;
        // 宣言順 PEG。choice 自体には trivia を足さない。
        if (parseNumericLiteral(consumed, matched) == 0) return s.end(FACTOR, consumed, matched, 0);
        s.trivia(); // '(' Expression ')' の匿名 chain 先頭。
        if (!s.literal("(", Session.OPEN)) return s.end(FACTOR, consumed, matched, -1);
        s.trivia();
        if (parseExpression(s.consumed, s.matched) < 0) return s.end(FACTOR, consumed, matched, -1);
        Recipe inner = s.recipe;
        Events events = s.events;
        s.trivia();
        if (!s.literal(")", Session.CLOSE)) return s.end(FACTOR, consumed, matched, -1);
        s.trivia();
        s.recipe = inner; // 括弧は透明。内側 Expression 自身の span を広げない。
        s.events = events;
        return s.end(FACTOR, consumed, matched, 0);
    }

    static int parseNumericLiteral(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(NUMERIC, consumed, matched);
        if (memo != Session.MISS) return memo;
        s.trivia();
        int captureStart = s.consumed;
        if (parseNumberGroup(s.consumed, s.matched) < 0) return s.end(NUMERIC, consumed, matched, -1);
        Recipe token = s.recipe;
        Events events = s.capture(s.events, 6, "value", captureStart, s.consumed);
        s.trivia();
        s.recipe = s.options.buildAst() ? s.leaf(Recipe.Kind.NUMBER, consumed, s.consumed,
            token.valueStart, token.valueEnd) : null;
        s.events = events;
        return s.end(NUMERIC, consumed, matched, 0);
    }

    static int parseNumberGroup(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(NUMBER_GROUP, consumed, matched);
        if (memo != Session.MISS) return memo;
        s.trivia();
        int start = s.consumed;
        if (!s.number()) return s.end(NUMBER_GROUP, consumed, matched, -1);
        int end = s.consumed;
        s.trivia();
        s.recipe = s.leaf(Recipe.Kind.TEXT, consumed, s.consumed, start, end);
        return s.end(NUMBER_GROUP, consumed, matched, 0);
    }

    static int parseAddOp(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(ADD, consumed, matched);
        if (memo != Session.MISS) return memo;
        int status = s.literal("+", Session.PLUS) || s.literal("-", Session.MINUS) ? 0 : -1;
        return s.end(ADD, consumed, matched, status);
    }

    static int parseMulOp(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(MUL, consumed, matched);
        if (memo != Session.MISS) return memo;
        int status = s.literal("*", Session.STAR) || s.literal("/", Session.SLASH) ? 0 : -1;
        return s.end(MUL, consumed, matched, status);
    }
}
