package org.ubnfc.java.emit;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.stream.Collectors;
import org.ubnfc.java.ir.GrammarIr;
import org.ubnfc.java.ir.GrammarIr.*;
import org.ubnfc.java.ir.Validation;

/** Deterministic direct source emission. Generated code has no dependency on this generator. */
public final class JavaEmitter {
    private final GrammarIr ir;
    private final String pkg, grammar, ast, parser;
    private final Map<String, Expr> expressions;
    private final Map<String, Rule> rules;
    private final Map<String, AstType> types;
    private final Map<String, Capture> captures;
    private final Map<String, Token> tokens;
    private final Map<String, Recipe> recipes;
    private final Map<String, Integer> exprIds = new LinkedHashMap<>(), ruleIds = new LinkedHashMap<>();
    public JavaEmitter(GrammarIr ir, String packageName) {
        Validation.validate(ir); this.ir = ir;
        var unit = ir.grammarUnits().stream().filter(g -> g.id().equals(ir.entry().grammarId())).findFirst().orElseThrow();
        grammar = unit.name(); Validation.identifier(grammar); ast = grammar + "AST"; parser = grammar + "Parser";
        pkg = packageName != null ? packageName : unit.package_() != null ? unit.package_() : "generated." + grammar.toLowerCase(java.util.Locale.ROOT);
        for (String part : pkg.split("\\.", -1)) Validation.identifier(part);
        expressions = Validation.index(ir.semantics().expressions(), Expr::id);
        rules = Validation.index(ir.semantics().rules(), Rule::symbolId);
        types = Validation.index(ir.semantics().astTypes(), AstType::id);
        captures = Validation.index(ir.semantics().captures(), Capture::siteId);
        tokens = Validation.index(ir.semantics().tokens(), Token::symbolId);
        recipes = Validation.index(ir.semantics().recipes(), Recipe::ruleId);
        for (String id : expressions.keySet()) exprIds.put(id, exprIds.size());
        for (String id : rules.keySet()) ruleIds.put(id, ruleIds.size());
    }
    public Map<String, String> sources() throws IOException {
        var result = new TreeMap<String, String>(); String prefix = pkg.replace('.', '/') + "/";
        result.put(prefix + ast + ".java", astSource());
        result.put(prefix + parser + ".java", parserSource());
        result.put(prefix + grammar + "Result.java", "package " + pkg + ";\nimport " + pkg + ".api.ParseResult;\npublic record " + grammar + "Result(ParseResult<" + ast + "> result) {}\n");
        for (String name : List.of("Span", "Capture", "Diagnostic", "MappingException", "ParseOptions", "ParseResult", "TokenScanner", "Json"))
            result.put(prefix + "api/" + name + ".java", resource("/templates/api/" + name + ".java").replace("generated.", pkg + "."));
        for (String name : List.of("Recipe", "Session")) result.put(prefix + "internal/" + name + ".java", resource("/templates/internal/" + name + ".java").replace("generated.", pkg + "."));
        for (String name : List.of("Input", "Cursor", "MemoTable", "ScopeStore", "Diagnostics", "Severity"))
            result.put(prefix + "rt/" + name + ".java", resource("/runtime/ubnfc/rt/" + name + ".java").replace("package ubnfc.rt;", "package " + pkg + ".rt;"));
        return java.util.Collections.unmodifiableMap(result);
    }
    public void write(Path output) throws IOException {
        Map<String, String> sources = sources();
        for (var file : sources.entrySet()) { Path path = output.resolve(file.getKey()); Files.createDirectories(path.getParent()); Files.writeString(path, file.getValue(), StandardCharsets.UTF_8); }
    }
    private static String resource(String path) throws IOException {
        try (var stream = JavaEmitter.class.getResourceAsStream(path)) {
            if (stream == null) throw new IOException("Missing bundled template " + path);
            return new String(stream.readAllBytes(), StandardCharsets.UTF_8);
        }
    }
    private static String q(String value) {
        if (value == null) return "null";
        // Java source strings: avoid raw control characters and source-level Unicode escapes.
        var out = new StringBuilder("\"");
        for (int i = 0; i < value.length(); i++) {
            char ch = value.charAt(i);
            switch (ch) {
                case '"' -> out.append("\\\""); case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n"); case '\r' -> out.append("\\r"); case '\t' -> out.append("\\t");
                case '\b' -> out.append("\\b"); case '\f' -> out.append("\\f");
                default -> { if (ch < 32 || ch == 127) out.append(String.format("\\%03o", (int) ch)); else out.append(ch); }
            }
        }
        return out.append('"').toString();
    }
    private static String strings(List<String> list) { return "new String[]{" + list.stream().map(JavaEmitter::q).collect(Collectors.joining(",")) + "}"; }
    private static String ints(List<Integer> list) { return "new int[]{" + list.stream().map(Object::toString).collect(Collectors.joining(",")) + "}"; }
    private String method(String id) { return "e" + exprIds.get(id); }
    private String call(String id) { return parser + "::" + method(id); }
    private String calls(List<String> ids) { return "new Call[]{" + ids.stream().map(this::call).collect(Collectors.joining(",")) + "}"; }
    private String ruleMethod(String id) {
        String name = ir.semantics().symbols().stream().filter(s -> s.id().equals(id)).map(Symbol::name).findFirst().orElse("Rule" + ruleIds.get(id));
        return "parse" + name.replaceAll("[^A-Za-z0-9_]", "_") + "_" + ruleIds.get(id);
    }
    private boolean root(AstType type) { return type.kind().equals("transparentUnion") && type.name().equals(ast); }
    private String typeName(String id) {
        AstType type = types.get(id);
        if (root(type)) return ast;
        return (type.enclosingTypeId() == null ? ast : typeName(type.enclosingTypeId())) + "." + type.name();
    }
    private Set<String> parents(AstType type) {
        var parents = new LinkedHashSet<>(type.implements_());
        for (AstType union : types.values()) if (union.kind().equals("transparentUnion") && union.variants().stream().anyMatch(v -> v.variant().equals(type.id()))) parents.add(union.id());
        return parents;
    }
    private String javaType(ValueType value, boolean boxed) {
        return switch (value.kind()) {
            case "text" -> "java.lang.String"; case "number" -> boxed ? "java.lang.Integer" : "int";
            case "node" -> typeName(value.typeId()); case "null" -> "java.lang.Object";
            case "optional" -> "java.util.Optional<" + javaType(value.element(), true) + ">";
            case "list" -> "java.util.List<" + javaType(value.element(), true) + ">";
            case "mixed" -> { var members = value.members().stream().filter(v -> !v.kind().equals("null")).toList(); yield members.size() == 1 ? javaType(members.getFirst(), true) : "java.lang.Object"; }
            default -> throw new IllegalArgumentException(value.kind());
        };
    }
    private String fieldType(AstField field) {
        String type = javaType(field.valueType(), !field.cardinality().equals("scalar"));
        return switch (field.cardinality()) { case "optional" -> "java.util.Optional<" + type + ">"; case "list" -> "java.util.List<" + type + ">"; default -> type; };
    }
    private String astSource() {
        var out = new StringBuilder("package " + pkg + ";\nimport java.util.List;\nimport java.util.Optional;\n\npublic sealed interface " + ast);
        var direct = types.values().stream().filter(t -> !root(t) && (parents(t).isEmpty() || parents(t).stream().anyMatch(p -> root(types.get(p))))).map(t -> typeName(t.id())).toList();
        if (direct.isEmpty()) direct = List.of(ast + ".Text");
        out.append(" permits ").append(String.join(", ", direct)).append(" {\n");
        if (types.isEmpty()) out.append("    record Text(String value) implements ").append(ast).append(" {}\n");
        for (AstType type : types.values()) if (!root(type) && (type.enclosingTypeId() == null || root(types.get(type.enclosingTypeId())))) astType(out, type, "    ");
        for (var common : ir.semantics().commonFields()) if (root(types.get(common.interfaceTypeId()))) out.append("    ").append(fieldType(common.field())).append(' ').append(common.field().name()).append("();\n");
        return out.append("}\n").toString();
    }
    private void astType(StringBuilder out, AstType type, String indent) {
        Set<String> parents = parents(type);
        String supers = parents.isEmpty() ? ast : parents.stream().map(this::typeName).collect(Collectors.joining(", "));
        out.append(indent);
        switch (type.kind()) {
            case "record" -> {
                out.append("record ").append(type.name()).append('(').append(type.fields().stream().map(f -> fieldType(f) + " " + f.name()).collect(Collectors.joining(", ")))
                    .append(") implements ").append(supers).append(" {\n");
                if (type.fields().stream().anyMatch(f -> f.cardinality().equals("list") || f.cardinality().equals("optional"))) {
                    out.append(indent).append("    public ").append(type.name()).append(" {\n");
                    for (var f : type.fields()) {
                        if (f.cardinality().equals("list")) out.append(indent).append("        ").append(f.name()).append(" = java.util.List.copyOf(").append(f.name()).append(");\n");
                        if (f.cardinality().equals("optional")) out.append(indent).append("        java.util.Objects.requireNonNull(").append(f.name()).append(");\n");
                    }
                    out.append(indent).append("    }\n");
                }
            }
            case "transparentUnion" -> {
                out.append("sealed interface ").append(type.name()).append(" extends ").append(supers).append(" permits ")
                    .append(type.variants().stream().map(v -> typeName(v.variant())).collect(Collectors.joining(", "))).append(" {\n");
                for (var common : ir.semantics().commonFields()) if (common.interfaceTypeId().equals(type.id())) out.append(indent).append("    ").append(fieldType(common.field())).append(' ').append(common.field().name()).append("();\n");
            }
            case "enum" -> out.append("enum ").append(type.name()).append(" implements ").append(supers).append(" { ")
                .append(type.variants().stream().map(AstTypeVariantsItem::javaName).collect(Collectors.joining(", "))).append(";\n");
            default -> throw new IllegalArgumentException(type.kind());
        }
        for (AstType child : types.values()) if (type.id().equals(child.enclosingTypeId())) astType(out, child, indent + "    ");
        out.append(indent).append("}\n");
    }
    private String parserSource() {
        var out = new StringBuilder("package " + pkg + ";\nimport java.util.List;\nimport java.util.Map;\nimport java.util.Optional;\nimport java.util.regex.Pattern;\nimport " + pkg + ".api.*;\nimport " + pkg + ".internal.Recipe;\nimport " + pkg + ".internal.Session;\nimport " + pkg + ".internal.Session.*;\n\npublic final class " + parser + " {\n    private " + parser + "() {}\n");
        out.append("    public static ParseResult<").append(ast).append("> parse(CharSequence source) { return parse(source, ParseOptions.DEFAULT); }\n")
            .append("    public static ParseResult<").append(ast).append("> parse(CharSequence source, ParseOptions options) {\n")
            .append("        var s = new Session(source, options, ").append(strings(rules.keySet().stream().map(id -> ir.semantics().symbols().stream().filter(t -> t.id().equals(id)).map(Symbol::name).findFirst().orElse(id)).toList())).append(");\n")
            .append("        var f = new Frame(); Match match = ").append(ruleMethod(ir.entry().ruleId())).append("(s, f);\n")
            .append("        ParseResult<").append(ast).append("> syntax = s.result(f, match, ").append(catalogs()).append(");\n")
            .append("        if (!syntax.ok() || !options.buildAst()) return syntax;\n")
            .append("        if (match.mappingFailure()) throw s.mapping(match.start(), match.end(), \"Recovered input cannot be mapped\", null);\n")
            .append("        if (match.nodes().isEmpty()) return syntax;\n")
            .append("        return syntax.withAst((").append(ast).append(") build(match.nodes().getFirst(), s), s.spans);\n    }\n");
        emitBuild(out);
        for (Rule rule : rules.values()) emitRule(out, rule);
        for (Expr expr : expressions.values()) emitExpr(out, expr);
        return out.append("}\n").toString();
    }
    private String catalogs() {
        return "Map.ofEntries(" + ir.semantics().catalogs().stream().map(c -> "Map.entry(" + q(c.captureSiteId()) + "," + q(c.context()) + ")").collect(Collectors.joining(",")) + ")";
    }
    private void emitRule(StringBuilder out, Rule rule) {
        Recipe recipe = recipes.get(rule.symbolId());
        var memo = ir.plan().memo().entries().stream().filter(e -> e.exprId().equals(rule.body())).findFirst().orElse(null);
        boolean safe = safe(rule.body(), new java.util.HashSet<>());
        boolean success = memo == null ? safe : memo.cacheSuccess(); boolean failure = memo == null ? safe : memo.cacheFailure();
        var scope = ir.semantics().scope().effects().stream().filter(e -> e.ruleId().equals(rule.symbolId())).toList();
        String effects = scope.stream().flatMap(e -> e.events().stream().map(v -> "new Effect(" + q(v.when()) + "," + q(v.action()) + "," + q(v.captureSiteId()) + "," + q(v.name()) + "," + q(e.mode()) + ")")).collect(Collectors.joining(","));
        String fields = recipe == null ? "" : recipe.fields().stream().map(f -> "new Field(" + q(f.field()) + "," + strings(f.siteIds()) + "," + q(f.conversion()) + "," + q(f.cardinality()) + ")").collect(Collectors.joining(","));
        out.append("    private static Match ").append(ruleMethod(rule.symbolId())).append("(Session s, Frame f) {\n        return s.rule(f, ").append(ruleIds.get(rule.symbolId())).append(", ").append(success).append(", ").append(failure)
            .append(", ").append(call(rule.body())).append(", ").append(q(recipe == null ? null : recipe.id())).append(", ").append(q(recipe == null || recipe.leafFallback() == null ? null : recipe.leafFallback().typeId())).append(", ").append(q(recipe == null ? null : recipe.kind()))
            .append(", new Field[]{").append(fields).append("}, new Effect[]{").append(effects).append("}, ").append(rule.skip()).append(");\n    }\n");
    }
    private boolean safe(String id, Set<String> visited) {
        if (!visited.add(id)) return true;
        Expr expr = expressions.get(id);
        if (expr.node().kind().equals("tokenRef")) { var token = tokens.get(expr.node().symbolId()); return token.memoSafety().success() && token.memoSafety().failure(); }
        if (expr.node().kind().equals("ruleRef")) return safe(rules.get(expr.node().symbolId()).body(), visited);
        return Validation.children(expr.node()).stream().allMatch(c -> safe(c, visited));
    }
    private void emitExpr(StringBuilder out, Expr expr) {
        String method = method(expr.id());
        var sites = expr.captureSiteIds().stream().map(captures::get).toList();
        out.append("    private static Match ").append(method).append("(Session s, Frame f) {\n");
        if (expr.execution().entryFlag().equals("setFalse")) out.append("        f.reset = false;\n");
        out.append("        return s.capture(f, f.matched ? ").append(parser).append("::").append(method).append("M : ").append(parser).append("::").append(method).append("C, ")
            .append(strings(expr.captureSiteIds())).append(", ").append(strings(sites.stream().map(Capture::name).toList())).append(", ").append(strings(sites.stream().map(Capture::spanRule).toList()))
            .append(", new boolean[]{").append(sites.stream().map(c -> c.skip().toString()).collect(Collectors.joining(","))).append("}, ").append(q(expr.ruleId())).append(", ").append(q(expr.id())).append(");\n    }\n");
        String body = expression(expr);
        for (String mode : List.of("C", "M")) out.append("    private static Match ").append(method).append(mode).append("(Session s, Frame f) {\n        ").append(body).append("\n    }\n");
    }
    private String expression(Expr expr) {
        ExprNode n = expr.node(); String boundary = boundaries(expr.id());
        String label = expr.diagnosticLabel();
        var display = ir.semantics().diagnostics().expectedDisplay().stream().filter(d -> d.exprId().equals(expr.id())).findFirst().orElse(null);
        if (display != null) label = display.java_();
        if (label == null) label = n.kind().equals("literal") ? "'" + n.value() + "'" : n.kind();
        return switch (n.kind()) {
            case "seq" -> "return s.sequence(f, " + calls(n.items()) + ", " + boundary + ");";
            case "group" -> expr.execution().transaction().equals("delegate") ? "return " + method(n.child()) + "(s, f);" : "return s.sequence(f, " + calls(List.of(n.child())) + ", " + boundary + ");";
            case "optional" -> "return s.optional(f, " + call(n.child()) + ");";
            case "repeat" -> "return s.repeat(f, " + call(n.child()) + ", " + n.min() + ", " + (n.max() == null ? "Integer.MAX_VALUE" : n.max()) + ", " + (n.terminator() == null ? "null" : call(n.terminator())) + ", " + boundary + ");";
            case "separated" -> "return s.separated(f, " + call(n.element()) + ", " + call(n.separator()) + ", " + n.min() + ", " + boundary + ");";
            case "choice" -> choice(expr);
            case "literal" -> "return s.literal(f, " + q(n.value()) + ", " + n.caseSensitive() + ", " + q(label) + ");";
            case "ruleRef" -> {
                String call = parser + "::" + ruleMethod(n.symbolId());
                var recovery = ir.semantics().recovery().stream().filter(r -> r.applicationSites().contains(expr.id())).findFirst().orElse(null);
                yield recovery == null ? "return " + ruleMethod(n.symbolId()) + "(s, f);" : "return s.recover(f, " + call + ", " + strings(recovery.resolvedSyncPatterns()) + ", " + recovery.consumeSync() + ", " + q(recovery.noSyncAction()) + ", " + q(recovery.astProjection()) + ", " + q(recovery.recipeId()) + ");";
            }
            case "tokenRef" -> "return " + token(tokens.get(n.symbolId()).spec()) + ";";
            case "lookahead", "negativeLookahead" -> "return s.look(f, " + call(n.child()) + ", " + n.kind().equals("negativeLookahead") + ", " + q(label) + ");";
            case "any" -> "return s.any(f, " + q(label) + ");";
            case "eof" -> "return s.eof(f);"; case "empty" -> "return s.empty(f);";
            case "error" -> "return s.error(f, " + q(n.message()) + ");";
            default -> throw new IllegalArgumentException(n.kind());
        };
    }
    private String token(TokenSpec spec) {
        return switch (spec.kind()) {
            case "builtin" -> "s.builtin(f," + q(spec.which()) + "," + ints(spec.codePoints()) + ")";
            case "until" -> "s.until(f," + q(spec.terminator()) + ")";
            case "lookahead", "negativeLookahead" -> "s.look(f,(ss,ff)->ss.literal(ff," + q(spec.pattern()) + ",true," + q(spec.pattern()) + ")," + spec.kind().equals("negativeLookahead") + "," + q(spec.pattern()) + ")";
            case "negation" -> "s.range(f,0,1114111," + q(spec.excluded()) + ",\"NEGATION\")";
            case "charRange" -> "s.range(f," + spec.min() + "," + spec.max() + ",null,\"CHAR_RANGE\")";
            case "caseInsensitive" -> "s.literal(f," + q(spec.word()) + ",false," + q(spec.word()) + ")";
            case "regex" -> "s.regex(f,Pattern.compile(" + q(spec.pattern()) + "))";
            case "extern" -> "s.external(f," + q(spec.logicalId()) + ")";
            case "any" -> "s.any(f,\"ANY\")"; case "eof" -> "s.eof(f)"; case "empty" -> "s.empty(f)";
            default -> throw new IllegalArgumentException(spec.kind());
        };
    }
    private String boundaries(String id) {
        var policies = Validation.index(ir.semantics().trivia().policies(), TriviaPolicy::id);
        return "new Boundary[]{" + ir.semantics().trivia().insertionPlan().stream().filter(p -> p.boundary().ownerExprId().equals(id)).map(p -> {
            Boundary b = p.boundary();
            String delimiters = policies.get(p.policyId()).effectiveDelimiterSet().stream().map(d -> "new Delimiter(" + q(d.kind()) + "," + q(d.open()) + "," + q(d.close()) + "," + ints(d.codePoints()) + ")").collect(Collectors.joining(","));
            return "new Boundary(" + q(b.at()) + "," + (b.childIndex() == null ? -1 : b.childIndex()) + ",new Delimiter[]{" + delimiters + "})";
        }).collect(Collectors.joining(",")) + "}";
    }
    private String choice(Expr expr) {
        var plan = ir.plan().choices().stream().filter(p -> p.exprId().equals(expr.id())).findFirst().orElse(null);
        boolean longest = expr.node().selection().equals("longest");
        boolean predict = expr.node().selection().equals("predictive") || plan != null && plan.strategy().equals("predict");
        String eligible = "null";
        if (predict) {
            var analyses = Validation.index(ir.analysis().expressions(), ExprAnalysis::exprId);
            eligible = "new boolean[]{" + expr.node().alternatives().stream().map(id -> {
                MayStart start = analyses.get(id).mayStart();
                if (start.certainty().equals("unknown") || !start.includesTrivia() || !start.zeroConsumeCondition().equals("never")) return "true";
                var points = new ArrayList<Integer>();
                for (var range : start.ranges()) { if (range.end() - range.start() > 256) return "true"; for (int i = range.start(); i <= range.end(); i++) points.add(i); }
                if (start.eof()) points.add(-1);
                if (points.isEmpty()) return "false";
                return "(switch (s.cp(f.reset ? f.c : f.position())) { case " + points.stream().map(Object::toString).collect(Collectors.joining(",")) + " -> true; default -> false; })";
            }).collect(Collectors.joining(",")) + "}";
        }
        return "return s.choice(f," + calls(expr.node().alternatives()) + "," + longest + "," + eligible + "," + (predict && (plan == null || plan.fallback().equals("fullChoice"))) + ");";
    }
    private void emitBuild(StringBuilder out) {
        if (recipes.values().stream().noneMatch(r -> r.resultTypeId() != null && !Set.of("transparent", "text", "skip").contains(r.kind()))) {
            out.append("    private static Object build(Recipe r, Session s) { throw s.mapping(r.start(), r.end(), \"No AST recipe\", null); }\n");
            return;
        }
        out.append("    private static Object build(Recipe r, Session s) {\n        Object cached = s.built(r); if (cached != null) return cached;\n        Object node = switch (r.id()) {\n");
        var fallbacks = new LinkedHashSet<String>();
        for (Recipe recipe : recipes.values()) {
            if (recipe.resultTypeId() == null || Set.of("transparent", "text", "skip").contains(recipe.kind())) continue;
            AstType type = types.get(recipe.resultTypeId());
            if (recipe.leafFallback() != null) fallbacks.add(recipe.leafFallback().typeId());
            out.append("            case ").append(q(recipe.id())).append(" -> ");
            if (type.kind().equals("enum")) {
                out.append("switch (s.source.substring(r.start(), r.end()).strip()) {\n");
                for (var variant : type.variants()) out.append("                case ").append(q(variant.literal())).append(" -> ").append(typeName(type.id())).append('.').append(variant.javaName()).append(";\n");
                out.append("                default -> throw s.mapping(r.start(), r.end(), \"Unknown enum literal\", null);\n            };\n");
            } else {
                out.append("new ").append(typeName(type.id())).append('(');
                out.append(type.fields().stream().map(field -> {
                    var mapping = recipe.fields().stream().filter(f -> f.field().equals(field.name())).findFirst().orElseThrow(() -> new IllegalArgumentException("Missing recipe field: " + recipe.id() + "." + field.name()));
                    return "s." + mapping.cardinality() + "(r," + q(field.name()) + ",v -> " + convert(field.valueType(), mapping.conversion()) + ")";
                }).collect(Collectors.joining(", "))).append(");\n");
            }
        }
        for (String fallback : fallbacks) out.append("            case ").append(q("#leaf:" + fallback)).append(" -> new ").append(typeName(fallback)).append("(null, List.of(s.source.substring(r.start(), r.end()).strip()), List.of());\n");
        out.append("            default -> throw s.mapping(r.start(), r.end(), \"Cannot build recipe: \" + r.id(), null);\n        };\n        return s.remember(r, node);\n    }\n");
    }
    private String convert(ValueType type, String conversion) {
        String java = javaType(type, true);
        return switch (conversion) {
            case "text" -> "s.text(v)"; case "number" -> "s.number(v)";
            case "node" -> (java.equals("java.lang.Object") ? "" : "(" + java + ") ") + "build(s.node(v, r), s)";
            case "mixed" -> {
                boolean text = type.members().stream().anyMatch(t -> t.kind().equals("text"));
                boolean number = type.members().stream().anyMatch(t -> t.kind().equals("number"));
                String leaf = text ? "s.text(v)" : number ? "s.number(v)" : "null";
                yield (java.equals("java.lang.Object") ? "" : "(" + java + ") ") + "(v.nodes().isEmpty() ? " + leaf + " : build(s.node(v, r), s))";
            }
            default -> throw new IllegalArgumentException(conversion);
        };
    }
}
