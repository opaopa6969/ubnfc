package org.ubnfc.java.emit;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import org.ubnfc.java.ir.GrammarIr;
import org.ubnfc.java.ir.GrammarIr.*;
import org.ubnfc.java.ir.Validation;

/** Legacy display candidates: terminal hints within two parser edges, otherwise the parser name. */
final class DiagnosticPlan {
    private static final class Node {
        final String name;
        final List<String> terminal;
        List<Node> children = List.of();
        Node(String name, String... terminal) { this.name = name; this.terminal = List.of(terminal); }
        Node with(Node... nodes) { children = List.of(nodes); return this; }
    }
    private final Map<String, Expr> expressions;
    private final Map<String, Token> tokens;
    private final Map<String, String> names = new HashMap<>();
    private final Map<String, Node> rules = new LinkedHashMap<>(), nodes = new HashMap<>(), bases = new HashMap<>();
    private final Map<String, String> helpers = new HashMap<>();
    private final java.util.Set<String> recoverySites = new java.util.HashSet<>();
    private final java.util.Set<String> assigned = new java.util.HashSet<>();
    private final Map<String, String> displays = new HashMap<>();
    private final Map<String, String> bodies = new HashMap<>();
    DiagnosticPlan(GrammarIr ir) {
        expressions = Validation.index(ir.semantics().expressions(), Expr::id);
        tokens = Validation.index(ir.semantics().tokens(), Token::symbolId);
        ir.semantics().diagnostics().expectedDisplay().forEach(d -> displays.put(d.exprId(), d.java_()));
        ir.semantics().recovery().forEach(r -> recoverySites.addAll(r.applicationSites()));
        ir.semantics().symbols().forEach(s -> names.put(s.id(), s.name()));
        for (Rule r : ir.semantics().rules()) {
            ExprNode body = expressions.get(r.body()).node();
            rules.put(r.symbolId(), r.body().contains("/lower/operator/") && body.kind().equals("literal")
                ? new Node("WordParser", "'" + body.value() + "'") : new Node(names.get(r.symbolId()) + "Parser"));
            bodies.put(r.body(), r.symbolId());
            assign(r.body(), new int[4]);
        }
        for (Rule r : ir.semantics().rules()) rules.get(r.symbolId()).children = children(r.body());
        expressions.keySet().forEach(this::node);
    }
    private void assign(String id, int[] counters) {
        if (!assigned.add(id)) return;
        Expr e = expressions.get(id); ExprNode n = e.node(); String name = names.get(e.ruleId());
        if (id.endsWith("/lower/separated/0")) {
            String helper = name + "Sep" + counters[3]++ ;
            helpers.put(id, helper + "Parser");
            helpers.put(id.substring(0, id.length() - 1) + "2", helper + "BodyParser");
        }
        if (n.kind().equals("group")) helpers.put(n.child(), name + "Group" + counters[0]++ + "Parser");
        if (n.kind().equals("repeat") && !java.util.Set.of("ruleRef", "tokenRef").contains(expressions.get(n.child()).node().kind()) && !id.contains("/lower/separated/")) {
            String kind = n.min() == 0 ? "Repeat" : n.min() == 1 ? "OneOrMore" : "Bounded";
            helpers.put(n.child(), name + kind + counters[1]++ + "Parser");
        }
        if (n.kind().equals("optional") && n.child().contains("/lower/helper/"))
            helpers.put(n.child(), name + "Opt" + counters[2]++ + "Parser");
        if (n.kind().equals("separated")) helpers.put(id, name + "Sep" + counters[3]++ + "Parser");
        for (String child : Validation.children(n)) assign(child, counters);
    }
    private List<Node> children(String id) {
        ExprNode n = expressions.get(id).node();
        if (n.kind().equals("seq")) return n.items().stream().map(this::node).toList();
        if (n.kind().equals("choice")) return n.alternatives().stream().map(this::node).toList();
        return List.of(node(id));
    }
    private Node node(String id) {
        if (nodes.containsKey(id)) return nodes.get(id);
        Expr e = expressions.get(id); ExprNode n = e.node();
        Node base;
        base = switch (n.kind()) {
            case "literal" -> new Node("WordParser", "'" + n.value() + "'");
            case "ruleRef" -> rules.get(n.symbolId());
            case "tokenRef" -> token(tokens.get(n.symbolId()));
            case "group" -> {
                Node helper = new Node(helpers.get(n.child())); helper.children = children(n.child()); yield helper;
            }
            case "seq", "choice" -> {
                Node container = new Node(helpers.getOrDefault(id, ""));
                container.children = children(id); yield container;
            }
            case "repeat", "optional" -> new Node(n.kind().equals("optional") ? "Optional" : "Repeat").with(node(n.child()));
            case "separated" -> new Node(helpers.get(id)).with(node(n.element()), new Node("ZeroOrMore").with(
                new Node(helpers.get(id).replace("Parser", "BodyParser")).with(node(n.separator()), node(n.element()))));
            case "eof" -> new Node("EndOfSourceParser").with(new Node("WildCardCharacterParser"));
            case "empty" -> new Node("EmptyParser").with(new Node("MatchOnly").with(new Node("WildCardCharacterParser")));
            case "any" -> new Node("WildCardCharacterParser");
            case "lookahead", "negativeLookahead" -> new Node("MatchOnly").with(node(n.child()));
            case "error" -> new Node("ErrorMessageParser", n.message());
            default -> new Node(n.kind());
        };
        if (displays.get(id) != null) base = new Node(base.name, displays.get(id));
        if (recoverySites.contains(id)) base = new Node("RecoveryParser").with(base);
        bases.put(id, base);
        Node result = e.captureSiteIds().isEmpty() ? base : new Node("__CaptureSite").with(base);
        nodes.put(id, result); return result;
    }
    private Node token(Token token) {
        TokenSpec s = token.spec();
        return switch (s.kind()) {
            case "caseInsensitive" -> new Node("IgnoreCaseWordParser", "'" + s.word() + "'");
            case "regex" -> new Node("RegexTokenParser", "REGEX('" + s.pattern() + "')");
            case "lookahead", "negativeLookahead" -> new Node("MatchOnly").with(new Node("WordParser", "'" + s.pattern() + "'"));
            case "until" -> new Node("WildCardStringTerminatorParser").with(new Node("WildCardCharacterParser"));
            case "eof" -> new Node("EndOfSourceParser").with(new Node("WildCardCharacterParser"));
            case "empty" -> new Node("EmptyParser").with(new Node("MatchOnly").with(new Node("WildCardCharacterParser")));
            case "any" -> new Node("WildCardCharacterParser");
            case "builtin" -> switch (s.which()) {
                case "Number" -> new Node("NumberParser").with(new Node("Optional").with(new Node("SignParser")),
                    new Node("Choice").with(new Node("Chain").with(new Node("OneOrMore").with(new Node("DigitParser")), new Node("PointParser", "'.'"))),
                    new Node("Optional").with(new Node("ExponentParser")));
                case "Identifier" -> new Node("IdentifierParser").with(new Node("AlphabetUnderScoreParser"),
                    new Node("ZeroOrMore").with(new Node("AlphabetNumericUnderScoreParser")));
                case "EndOfSource" -> new Node("EndOfSourceParser");
                default -> new Node(names.get(token.symbolId()) + "Parser");
            };
            case "extern" -> new Node(s.originalClassName().substring(s.originalClassName().lastIndexOf('.') + 1));
            default -> new Node(names.get(token.symbolId()) + "Parser");
        };
    }
    private static void hints(Node node, int depth, LinkedHashSet<String> result) {
        result.addAll(node.terminal);
        if (depth > 0 && node.terminal.isEmpty()) for (Node child : node.children) hints(child, depth - 1, result);
    }
    private static List<String> hints(Node node) {
        var result = new LinkedHashSet<String>(); hints(node, 2, result);
        if (result.isEmpty() && !node.name.isBlank()) result.add(node.name);
        return List.copyOf(result);
    }
    List<String> rule(String id) { return hints(rules.get(id)); }
    List<String> expression(String id) {
        Expr e = expressions.get(id);
        // D-014: Extern の候補は scanner の表示文字列。Java クラス名や
        // capture wrapper は候補ではなく、scanner の診断に追加してはいけない。
        if (e.node().kind().equals("tokenRef") && tokens.get(e.node().symbolId()).spec().kind().equals("extern"))
            return List.of();
        var result = new ArrayList<String>();
        if (!bodies.containsKey(id) && !e.node().kind().equals("ruleRef") ) result.addAll(hints(bases.get(id)));
        if (!e.captureSiteIds().isEmpty()) result.addAll(hints(nodes.get(id)));
        return result.stream().distinct().toList();
    }
    String tokenLabel(String id) {
        Expr e = expressions.get(id);
        if (e.node().kind().equals("tokenRef") && tokens.get(e.node().symbolId()).spec().kind().equals("extern"))
            return names.get(e.node().symbolId());
        return hints(bases.get(id)).getFirst();
    }
}
