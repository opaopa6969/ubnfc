package target.power;

import java.util.IdentityHashMap;
import java.util.List;
import java.util.Optional;
import target.api.*;
import target.internal.*;

/** Power.ubnf の Base '^' Self | Base。各ノードの op/right は 0 または 1 要素。 */
public final class PowerParser {
    private PowerParser() {}
    private static final int EXPR = 0, ATOM = 1, RECURSIVE = 2;

    public static ParseResult<PowerAST> parse(CharSequence source) {
        return parse(source, ParseOptions.DEFAULT);
    }

    public static ParseResult<PowerAST> parse(CharSequence source, ParseOptions options) {
        var s = new Session(source.toString(), options, "Expr", "Atom", "Expr.recursive");
        Session previous = Session.CURRENT.get();
        Session.CURRENT.set(s);
        try {
            ParseResult<PowerAST> syntax = s.result(parseExpr(0, 0));
            if (!syntax.ok() || !options.buildAst()) return syntax;
            var spans = new IdentityHashMap<PowerAST, Span>();
            var ast = build(s.recipe, s, spans);
            return new ParseResult<>(true, syntax.consumedCp(), syntax.matchedCp(), Optional.of(ast),
                spans, syntax.captures(), syntax.diagnostics());
        } finally {
            if (previous == null) Session.CURRENT.remove(); else Session.CURRENT.set(previous);
        }
    }

    private static PowerAST.Power build(Recipe recipe, Session s, IdentityHashMap<PowerAST, Span> spans) {
        var right = recipe.right.isEmpty() ? List.<PowerAST.Power>of()
            : List.of(build(recipe.right.getFirst(), s, spans));
        var ast = new PowerAST.Power(s.source.substring(recipe.left.valueStart, recipe.left.valueEnd).strip(),
            recipe.operators, right);
        spans.put(ast, s.span(recipe.start, recipe.end));
        return ast;
    }

    static int parseExpr(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(EXPR, consumed, matched);
        if (memo != Session.MISS) return memo;
        if (parseRecursive(consumed, matched) == 0) return s.end(EXPR, consumed, matched, 0);
        // 右枝不成立なら同じ開始状態の Base。Atom の成功 memo もここで再生される。
        if (parseAtom(consumed, matched) < 0) return s.end(EXPR, consumed, matched, -1);
        Recipe left = s.recipe;
        s.events = s.capture(s.events, 0, "left", consumed, s.consumed);
        s.recipe = s.options.buildAst() ? new Recipe(Recipe.Kind.POWER, consumed, s.consumed,
            0, 0, left, List.of(), List.of()) : null;
        return s.end(EXPR, consumed, matched, 0);
    }

    static int parseRecursive(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(RECURSIVE, consumed, matched);
        if (memo != Session.MISS) return memo;
        s.trivia(); // 右再帰候補の chain のみ。Expr choice 自体には入れない。
        int leftStart = s.consumed;
        if (parseAtom(s.consumed, s.matched) < 0) return s.end(RECURSIVE, consumed, matched, -1);
        Recipe left = s.recipe;
        Events events = s.capture(s.events, 0, "left", leftStart, s.consumed);
        s.trivia();
        int opStart = s.consumed;
        if (!s.literal("^", Session.CARET)) return s.end(RECURSIVE, consumed, matched, -1);
        events = s.capture(events, 1, "op", opStart, s.consumed);
        s.trivia();
        int rightStart = s.consumed;
        if (parseExpr(s.consumed, s.matched) < 0) return s.end(RECURSIVE, consumed, matched, -1);
        Recipe right = s.recipe;
        events = s.capture(Events.concat(events, s.events), 2, "right", rightStart, s.consumed);
        s.trivia();
        s.recipe = s.options.buildAst() ? new Recipe(Recipe.Kind.POWER, consumed, s.consumed,
            0, 0, left, List.of("^"), List.of(right)) : null;
        s.events = events;
        return s.end(RECURSIVE, consumed, matched, 0);
    }

    static int parseAtom(int consumed, int matched) {
        Session s = Session.CURRENT.get();
        int memo = s.begin(ATOM, consumed, matched);
        if (memo != Session.MISS) return memo;
        s.trivia();
        if (!s.number()) return s.end(ATOM, consumed, matched, -1);
        s.trivia();
        // 非 mapping rule を capture した String は rule 全文。mapper と同じ strip を採用後に行う。
        s.recipe = s.leaf(Recipe.Kind.TEXT, consumed, s.consumed, consumed, s.consumed);
        return s.end(ATOM, consumed, matched, 0);
    }
}
