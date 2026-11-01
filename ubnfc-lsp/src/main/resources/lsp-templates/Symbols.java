package __PACKAGE__;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import __API_PACKAGE__.ParseResult;

/**
 * 位置 ↔ 記号の索引。座標はすべて code point（LSP への変換は {@link Positions} だけが行う）。
 *
 * <p>出所は 2 つある。
 * <ol>
 *   <li><b>scope 観測</b> — {@code scope().declarations()} / {@code references()}。
 *       {@code @declares} / {@code @backref} を記号表として解決する文法（P4）はこれで足りる。</li>
 *   <li><b>capture site + lexical</b> — {@code @backref} の {@code sameRuleText} モード（UBNF 自身の文法）は
 *       「同じ規則の中で綴りが一致するか」しか見ないので {@code ScopeStore} に宣言が積まれない。
 *       その場合は IR から畳み込んだ {@link Meta#DECLARE_SITES} / {@link Meta#USE_SITES} と
 *       {@code lexical()} の {@code exprId} を突き合わせて宣言・参照を拾う。</li>
 * </ol>
 */
public final class Symbols {
    /** 宣言または参照 1 件。 */
    public record Symbol(String name, int offsetCp, int lengthCp, boolean declaration) {
        public int endCp() { return offsetCp + lengthCp; }
        public boolean covers(int offsetCp) { return offsetCp >= this.offsetCp && offsetCp <= endCp(); }
    }

    /** document symbols の 1 節点。 */
    public record Node(String name, String detail, int startCp, int endCp, int nameStartCp, int nameLengthCp,
                       boolean container, List<Node> children) {}

    private final Analysis analysis;
    private final List<Symbol> symbols;
    private final Map<String, List<Symbol>> byName;
    private final boolean fromLexical;

    private Symbols(Analysis analysis, List<Symbol> symbols, boolean fromLexical) {
        this.analysis = analysis;
        this.symbols = List.copyOf(symbols);
        this.fromLexical = fromLexical;
        var index = new LinkedHashMap<String, List<Symbol>>();
        for (Symbol symbol : this.symbols) index.computeIfAbsent(symbol.name(), key -> new ArrayList<>()).add(symbol);
        index.replaceAll((key, value) -> List.copyOf(value));
        this.byName = Map.copyOf(index);
    }

    static Symbols of(Analysis analysis) {
        var scope = analysis.scope();
        var out = new ArrayList<Symbol>();
        if (scope != null) {
            for (var declaration : scope.declarations())
                out.add(new Symbol(declaration.name(), declaration.offsetCp(), length(declaration.name()), true));
            for (var reference : scope.references())
                out.add(new Symbol(reference.name(), reference.offset(), reference.len(), false));
        }
        if (!out.isEmpty()) return new Symbols(analysis, out, false);
        // scope 効果が記号表を作らない文法（sameRuleText）。capture site で拾う。
        for (var item : analysis.lexical()) {
            if (item.token() == null) continue;
            boolean declaration = Meta.DECLARE_SITES.contains(item.exprId());
            if (!declaration && !Meta.USE_SITES.contains(item.exprId())) continue;
            String name = item.token().strip();
            if (name.isEmpty()) continue;
            out.add(new Symbol(name, item.span().start(), item.span().end() - item.span().start(), declaration));
        }
        return new Symbols(analysis, out, true);
    }

    private static int length(String name) { return name == null ? 0 : name.codePointCount(0, name.length()); }

    /** 索引が {@code lexical()} 由来（scope 観測が空だった）なら true。 */
    public boolean fromLexical() { return fromLexical; }

    public List<Symbol> all() { return symbols; }

    /** 位置を覆う記号。宣言と参照が同じ位置にあるときは参照を優先する（定義へ跳べるように）。 */
    public Symbol at(int offsetCp) {
        Symbol best = null;
        for (Symbol symbol : symbols) {
            if (!symbol.covers(offsetCp)) continue;
            if (best == null || best.declaration() && !symbol.declaration()
                || symbol.lengthCp() < best.lengthCp() && symbol.declaration() == best.declaration()) best = symbol;
        }
        return best;
    }

    public List<Symbol> named(String name) { return byName.getOrDefault(name, List.of()); }

    public List<Symbol> declarationsOf(String name) {
        return named(name).stream().filter(Symbol::declaration).toList();
    }

    /** 採用された scope event を順に再生する。use はその時点の lookup、同名更新は後勝ち。 */
    public List<Symbol> definitions(Symbol symbol) {
        if (symbol.declaration()) return List.of(symbol);
        if (fromLexical || analysis.scope() == null || analysis.scope().events().isEmpty())
            return declarationsOf(symbol.name());
        return replay(symbol.offsetCp(), symbol.name()).stream().filter(s -> s.name().equals(symbol.name())).toList();
    }

    public List<Symbol> visibleAt(int offsetCp) {
        if (fromLexical || analysis.scope() == null || analysis.scope().events().isEmpty())
            return symbols.stream().filter(s -> s.declaration() && s.endCp() <= offsetCp).toList();
        return replay(offsetCp, null);
    }

    private List<Symbol> replay(int offsetCp, String reference) {
        var stack = new ArrayList<LinkedHashMap<String, List<Symbol>>>();
        stack.add(new LinkedHashMap<>());
        int clock = 0;
        for (var event : analysis.scope().events()) {
            int time = event.action().equals("declare") ? registeredAt(event.offsetCp()) : event.offsetCp();
            clock = Math.max(clock, time);
            if (clock > offsetCp) break;
            if (event.action().equals("use") && event.offsetCp() == offsetCp && reference != null) break;
            switch (event.action()) {
                case "enter" -> stack.add(new LinkedHashMap<>());
                case "leave" -> {
                    boolean entryEnd = reference == null && event.name().equals(LanguageServerImpl.ENTRY)
                        && event.offsetCp() == analysis.positions().lengthCp();
                    if (!entryEnd && stack.size() > 1) stack.remove(stack.size() - 1);
                }
                case "declare" -> {
                    var declaration = new Symbol(event.name(), event.offsetCp(), event.lengthCp(), true);
                    var current = stack.get(stack.size() - 1);
                    // field context は型検査前の候補集合。型による一意解決とは区別する。
                    var previous = current.getOrDefault(event.name(), List.of());
                    var values = new ArrayList<Symbol>();
                    if (hasContext(declaration, "field"))
                        previous.stream().filter(s -> hasContext(s, "field")).forEach(values::add);
                    values.add(declaration);
                    current.put(event.name(), values);
                }
                default -> { }
            }
        }
        var visible = new LinkedHashMap<String, List<Symbol>>();
        for (var frame : stack) visible.putAll(frame);
        return visible.values().stream().flatMap(List::stream).toList();
    }

    private String declarationRule(int offsetCp) {
        for (var item : analysis.lexical())
            if (item.token() != null && item.span().start() <= offsetCp && offsetCp < item.span().end() && Meta.DECLARE_RULES.containsKey(item.exprId()))
                return Meta.DECLARE_RULES.get(item.exprId());
        return null;
    }

    private int registeredAt(int offsetCp) {
        String rule = declarationRule(offsetCp);
        int end = Integer.MAX_VALUE;
        for (var item : analysis.lexical())
            if (item.token() == null && java.util.Objects.equals(item.exprId(), Meta.RULE_BODIES.get(rule))
                && item.span().start() <= offsetCp && offsetCp < item.span().end())
                end = Math.min(end, item.span().end());
        // scanner が直接宣言する場合は event の座標を使用する。
        return end == Integer.MAX_VALUE ? offsetCp : end;
    }

    public boolean hasContext(Symbol symbol, String context) {
        String contexts = Meta.DECLARATION_CONTEXTS.get(declarationRule(symbol.offsetCp()));
        return contexts != null && List.of(contexts.split(",")).contains(context);
    }

    public boolean inContext(Symbol symbol, String context) {
        if (Meta.DECLARATION_CONTEXTS.isEmpty()) return true;
        return context != null && hasContext(symbol, context);
    }

    public List<Symbol> referencesOf(String name) {
        return named(name).stream().filter(symbol -> !symbol.declaration()).toList();
    }

    /** 宣言位置を含む終端の規則 id（hover の説明文を引くのに使う）。無ければ null。 */
    public String ruleAt(int offsetCp) {
        String best = null;
        int bestWidth = Integer.MAX_VALUE;
        for (var item : analysis.lexical()) {
            if (item.token() == null) continue;
            var span = item.span();
            if (offsetCp < span.start() || offsetCp > span.end()) continue;
            int width = span.end() - span.start();
            if (width < bestWidth) { bestWidth = width; best = item.ruleId(); }
        }
        return best;
    }

    /** 位置を覆う終端。hover が「何の上にいるか」を答えるのに使う。 */
    public ParseResult.Lexical terminalAt(int offsetCp) {
        ParseResult.Lexical best = null;
        for (var item : analysis.lexical()) {
            if (item.token() == null) continue;
            var span = item.span();
            if (offsetCp < span.start() || offsetCp > span.end()) continue;
            if (best == null || span.end() - span.start() < best.span().end() - best.span().start()) best = item;
        }
        return best;
    }

    /**
     * {@code scope().events()} の enter / leave / declare から木を組む。
     * 入れ子は<b>事象の順序</b>で決める（規則が成功した順。宣言が leave の後に来ることがあるのは
     * {@code successAfterLeave} の意味論どおり）。events を持たない文法は宣言の平らな一覧にする。
     */
    public List<Node> documentSymbols() {
        var scope = analysis.scope();
        var roots = new ArrayList<Node>();
        if (scope == null || scope.events().isEmpty()) {
            for (Symbol symbol : symbols)
                if (symbol.declaration())
                    roots.add(new Node(symbol.name(), Meta.localName(ruleAt(symbol.offsetCp())),
                        symbol.offsetCp(), symbol.endCp(), symbol.offsetCp(), symbol.lengthCp(), false, List.of()));
            return roots;
        }
        record Open(String name, int startCp, List<Node> children) {}
        var stack = new ArrayDeque<Open>();
        for (var event : scope.events()) {
            switch (event.action()) {
                case "enter" -> stack.push(new Open(event.name(), event.offsetCp(), new ArrayList<>()));
                case "leave" -> {
                    if (stack.isEmpty()) break;
                    Open open = stack.pop();
                    var node = new Node(open.name(), null, open.startCp(), event.offsetCp(),
                        open.startCp(), 0, true, List.copyOf(open.children()));
                    (stack.isEmpty() ? roots : stack.peek().children()).add(node);
                }
                case "declare" -> {
                    var node = new Node(event.name(), Meta.localName(ruleAt(event.offsetCp())), event.offsetCp(),
                        event.offsetCp() + event.lengthCp(), event.offsetCp(), event.lengthCp(), false, List.of());
                    (stack.isEmpty() ? roots : stack.peek().children()).add(node);
                }
                default -> { }
            }
        }
        while (!stack.isEmpty()) {
            Open open = stack.pop();
            var node = new Node(open.name(), null, open.startCp(), analysis.positions().lengthCp(),
                open.startCp(), 0, true, List.copyOf(open.children()));
            (stack.isEmpty() ? roots : stack.peek().children()).add(node);
        }
        return roots;
    }
}
