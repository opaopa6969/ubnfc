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
    private final boolean emitDriver;
    private final String pkg, grammar, ast, parser;
    private final Map<String, Expr> expressions;
    private final Map<String, Rule> rules;
    private final Map<String, AstType> types;
    private final Map<String, Capture> captures;
    private final Map<String, Token> tokens;
    private final Map<String, Recipe> recipes;
    private final SemanticShape semanticShapes;
    private final DiagnosticPlan diagnosticPlan;
    private final Prediction prediction;
    private final Map<String, Integer> exprIds = new LinkedHashMap<>(), ruleIds = new LinkedHashMap<>();
    /** Generated constants (arrays of boundaries, fields, hints, ...) hoisted to static finals: expression text -> field name. */
    private final Map<String, String> constants = new LinkedHashMap<>();
    public JavaEmitter(GrammarIr ir, String packageName) { this(ir, packageName, true); }
    public JavaEmitter(GrammarIr ir, String packageName, boolean emitDriver) {
        Validation.validate(ir); this.ir = ir; this.emitDriver = emitDriver;
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
        semanticShapes = new SemanticShape(expressions, rules);
        diagnosticPlan = new DiagnosticPlan(ir);
        for (String id : expressions.keySet()) exprIds.put(id, exprIds.size());
        for (String id : rules.keySet()) ruleIds.put(id, ruleIds.size());
        prediction = new Prediction(ir, expressions, rules, tokens, ruleIds, diagnosticPlan, this::labelText);
    }
    public Map<String, String> sources() throws IOException {
        var result = new TreeMap<String, String>(); String prefix = pkg.replace('.', '/') + "/";
        result.put(prefix + ast + ".java", astSource());
        result.put(prefix + parser + ".java", parserSource());
        result.put(prefix + grammar + "Evaluator.java", evaluatorSource());
        result.put(prefix + grammar + "Result.java", "package " + pkg + ";\nimport " + pkg + ".api.ParseResult;\npublic record " + grammar + "Result(ParseResult<" + ast + "> result) {}\n");
        for (String name : List.of("Span", "Capture", "Diagnostic", "MappingException", "ParseOptions", "ParseResult", "TokenScanner", "Json", "SourceName"))
            result.put(prefix + "api/" + name + ".java", resource("/templates/api/" + name + ".java").replace("generated.", pkg + "."));
        for (String name : List.of("Recipe", "Session")) result.put(prefix + "internal/" + name + ".java", resource("/templates/internal/" + name + ".java").replace("generated.", pkg + "."));
        for (String name : List.of("Input", "Cursor", "MemoTable", "ScopeStore", "Diagnostics", "Severity"))
            result.put(prefix + "rt/" + name + ".java", resource("/runtime/ubnfc/rt/" + name + ".java").replace("package ubnfc.rt;", "package " + pkg + ".rt;"));
        if (emitDriver) result.put(prefix + "driver/Driver.java", driverSource());
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
    /** Hoist a side-effect-free constructor expression into a shared static final field (allocated once per class, not per call). */
    private String constant(String type, String expression) {
        return constants.computeIfAbsent(type + " " + expression, key -> "K" + constants.size());
    }
    private String kStrings(List<String> list) { return constant("String[]", strings(list)); }
    private String label(String text) { return constant("Label", "Label.of(" + q(text) + ")"); }
    private String kLabels(List<String> list) { return constant("Label[]", "new Label[]{" + list.stream().map(t -> "Label.of(" + q(t) + ")").collect(Collectors.joining(",")) + "}"); }
    private String method(String id) { return "e" + exprIds.get(id); }
    private String call(String id) { return parser + "::" + method(id); }
    private String calls(List<String> ids) { return constant("Call[]", "new Call[]{" + ids.stream().map(this::call).collect(Collectors.joining(",")) + "}"); }
    private String ruleMethod(String id) {
        String name = ir.semantics().symbols().stream().filter(s -> s.id().equals(id)).map(Symbol::name).findFirst().orElse("Rule" + ruleIds.get(id));
        return "parse" + name.replaceAll("[^A-Za-z0-9_]", "_") + "_" + ruleIds.get(id);
    }
    private boolean root(AstType type) { return type.kind().equals("transparentUnion") && type.name().equals(ast); }
    /** Injective escaping: java -> java_, java_ -> java__, etc. */
    private static String name(String value) { return value.matches("(?:java|javax|org|util)_*") ? value + "_" : value; }
    private String named(String value) { return name(value).equals(value) ? "" : "@" + pkg + ".api.SourceName(" + q(value) + ") "; }
    private String typeName(String id) {
        AstType type = types.get(id);
        if (root(type)) return ast;
        return (type.enclosingTypeId() == null ? ast : typeName(type.enclosingTypeId())) + "." + name(type.name());
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
    /** Keep original names, reserving them before assigning suffixes to collisions. */
    private Map<String, String> enumNames(AstType type) {
        var reserved = type.variants().stream().map(AstTypeVariantsItem::javaName).collect(Collectors.toSet());
        var used = new LinkedHashSet<String>(); var result = new LinkedHashMap<String, String>();
        for (var variant : type.variants()) {
            String base = variant.javaName(), candidate = base; int suffix = 2;
            while (used.contains(candidate)) {
                do { candidate = base + "_" + suffix++; } while (reserved.contains(candidate));
            }
            used.add(candidate); result.put(variant.literal(), candidate);
        }
        return result;
    }
    private String astSource() {
        var out = new StringBuilder("package " + pkg + ";\n\npublic sealed interface " + ast);
        var direct = types.values().stream().filter(t -> !root(t) && (parents(t).isEmpty() || parents(t).stream().anyMatch(p -> root(types.get(p))))).map(t -> typeName(t.id())).toList();
        if (direct.isEmpty()) direct = List.of(ast + ".Text");
        out.append(" permits ").append(String.join(", ", direct)).append(" {\n");
        if (types.isEmpty()) out.append("    record Text(java.lang.String value) implements ").append(ast).append(" {}\n");
        for (AstType type : types.values()) if (!root(type) && (type.enclosingTypeId() == null || root(types.get(type.enclosingTypeId())))) astType(out, type, "    ");
        for (var common : ir.semantics().commonFields()) if (root(types.get(common.interfaceTypeId()))) out.append("    ").append(fieldType(common.field())).append(' ').append(name(common.field().name())).append("();\n");
        return out.append("}\n").toString();
    }
    private void astType(StringBuilder out, AstType type, String indent) {
        Set<String> parents = parents(type);
        String supers = parents.isEmpty() ? ast : parents.stream().map(this::typeName).collect(Collectors.joining(", "));
        out.append(indent).append(named(type.name()));
        switch (type.kind()) {
            case "record" -> {
                out.append("record ").append(name(type.name())).append('(').append(type.fields().stream().map(f -> named(f.name()) + fieldType(f) + " " + name(f.name())).collect(Collectors.joining(", ")))
                    .append(") implements ").append(supers).append(" {\n");
                if (type.fields().stream().anyMatch(f -> f.cardinality().equals("list") || f.cardinality().equals("optional"))) {
                    out.append(indent).append("    public ").append(name(type.name())).append(" {\n");
                    for (var f : type.fields()) {
                        if (f.cardinality().equals("list")) out.append(indent).append("        ").append(name(f.name())).append(" = java.util.List.copyOf(").append(name(f.name())).append(");\n");
                        if (f.cardinality().equals("optional")) out.append(indent).append("        java.util.Objects.requireNonNull(").append(name(f.name())).append(");\n");
                    }
                    out.append(indent).append("    }\n");
                }
            }
            case "transparentUnion" -> {
                out.append("sealed interface ").append(name(type.name())).append(" extends ").append(supers).append(" permits ")
                    .append(type.variants().stream().map(v -> typeName(v.variant())).collect(Collectors.joining(", "))).append(" {\n");
                for (var common : ir.semantics().commonFields()) if (common.interfaceTypeId().equals(type.id())) out.append(indent).append("    ").append(fieldType(common.field())).append(' ').append(name(common.field().name())).append("();\n");
            }
            case "enum" -> {
                var names = enumNames(type);
                out.append("enum ").append(name(type.name())).append(" implements ").append(supers).append(" { ")
                    .append(String.join(", ", names.values())).append(";\n")
                    .append(indent).append("    public static ").append(name(type.name())).append(" fromText(java.lang.String text) {\n")
                    .append(indent).append("        return switch (text) {\n");
                for (var variant : names.entrySet()) out.append(indent).append("            case ").append(q(variant.getKey())).append(" -> ").append(variant.getValue()).append(";\n");
                out.append(indent).append("            default -> throw new java.lang.IllegalArgumentException(").append(q("Unknown " + type.name() + ": ")).append(" + text);\n")
                    .append(indent).append("        };\n").append(indent).append("    }\n");
            }
            default -> throw new IllegalArgumentException(type.kind());
        }
        for (AstType child : types.values()) if (type.id().equals(child.enclosingTypeId())) astType(out, child, indent + "    ");
        out.append(indent).append("}\n");
    }
    private String evalMethod(AstType type) {
        return "eval" + typeName(type.id()).substring(ast.length() + 1).replace(".", "");
    }
    private String evaluatorSource() {
        var out = new StringBuilder("package " + pkg + ";\n\npublic abstract class " + grammar + "Evaluator<T> {\n");
        out.append("    private DebugStrategy debugStrategy = DebugStrategy.NOOP;\n")
            .append("    public void setDebugStrategy(DebugStrategy strategy) { debugStrategy = java.util.Objects.requireNonNull(strategy); }\n")
            .append("    public T eval(").append(ast).append(" node) {\n")
            .append("        debugStrategy.onEnter(node); T result = evalInternal(node); debugStrategy.onExit(node, result); return result;\n    }\n")
            .append("    private T evalInternal(").append(ast).append(" node) {\n        return switch (node) {\n");
        if (types.isEmpty()) out.append("            case ").append(ast).append(".Text n -> evalText(n);\n");
        for (AstType type : types.values()) if (!type.kind().equals("transparentUnion"))
            out.append("            case ").append(typeName(type.id())).append(" n -> ").append(evalMethod(type)).append("(n);\n");
        out.append("        };\n    }\n");
        if (types.isEmpty()) out.append("    protected abstract T evalText(").append(ast).append(".Text node);\n");
        boolean binary = false, variable = false;
        for (AstType type : types.values()) {
            if (root(type)) continue;
            Eval eval = rules.values().stream().filter(r -> type.id().equals(r.mappingTypeId()) && r.eval() != null)
                .map(Rule::eval).findFirst().orElse(null);
            String kind = eval != null && eval.strategy().equals("auto") ? eval.kind() : "manual";
            String signature = "T " + evalMethod(type) + "(" + typeName(type.id()) + " node)";
            String body;
            if (type.kind().equals("transparentUnion")) body = "return evalInternal(node);";
            else switch (kind) {
                case "binary_arithmetic" -> {
                    binary = true;
                    body = "if (node.left() == null && node.right().isEmpty() && node.op().size() == 1) return evalLeaf(node.op().getFirst());\n"
                        + "        if (node.left() != null && node.op().isEmpty() && node.right().isEmpty()) return eval(node.left());\n"
                        + "        T current = eval(node.left());\n"
                        + "        for (int i = 0; i < Math.min(node.op().size(), node.right().size()); i++) current = applyBinary(node.op().get(i), current, eval(node.right().get(i)));\n"
                        + "        return current;";
                }
                case "variable_ref" -> {
                    variable = true; String prefix = (String) eval.params().getOrDefault("strip_prefix", "");
                    body = "String name = node.name();\n"
                        + (prefix.isEmpty() ? "" : "        if (name != null && name.startsWith(" + q(prefix) + ")) name = name.substring(" + prefix.length() + ");\n")
                        + "        return resolveVariable(name);";
                }
                case "conditional" -> body = "Object condition = eval(node.condition());\n"
                    + "        boolean truth = Boolean.TRUE.equals(condition) || \"true\".equalsIgnoreCase(String.valueOf(condition));\n"
                    + "        return truth ? eval(node.thenExpr()) : eval(node.elseExpr());";
                case "passthrough" -> body = "Object value = node.value();\n        if (value instanceof " + ast + " child) return eval(child);\n        return (T) value;";
                case "literal" -> body = "return (T) (Object) node.value();";
                default -> { out.append("    protected abstract ").append(signature).append(";\n"); continue; }
            }
            if (kind.equals("literal") || kind.equals("passthrough")) out.append("    @SuppressWarnings(\"unchecked\")\n");
            out.append("    protected ").append(signature).append(" {\n        ").append(body).append("\n    }\n");
        }
        if (binary) out.append("    protected abstract T evalLeaf(String literal);\n    protected abstract T applyBinary(String op, T left, T right);\n");
        if (variable) out.append("    protected abstract T resolveVariable(String name);\n");
        return out.append("    public interface DebugStrategy {\n        void onEnter(").append(ast).append(" node);\n        void onExit(").append(ast).append(" node, Object result);\n")
            .append("        DebugStrategy NOOP = new DebugStrategy() {\n            public void onEnter(").append(ast).append(" node) {}\n            public void onExit(").append(ast).append(" node, Object result) {}\n        };\n    }\n")
            .append("    public static class StepCounterStrategy implements DebugStrategy {\n        private int step;\n        private final java.util.function.BiConsumer<Integer, ").append(ast).append("> onStep;\n")
            .append("        public StepCounterStrategy(java.util.function.BiConsumer<Integer, ").append(ast).append("> onStep) { this.onStep = onStep; }\n")
            .append("        public void onEnter(").append(ast).append(" node) { onStep.accept(step++, node); }\n        public void onExit(").append(ast).append(" node, Object result) {}\n    }\n}\n").toString();
    }
    private String parserSource() {
        String defaultEntry = ir.semantics().symbols().stream().filter(s -> s.id().equals(ir.entry().ruleId())).findFirst().orElseThrow().name();
        var out = new StringBuilder("package " + pkg + ";\nimport java.util.List;\nimport java.util.Map;\nimport java.util.Optional;\nimport java.util.regex.Pattern;\nimport " + pkg + ".api.*;\nimport " + pkg + ".internal.Recipe;\nimport " + pkg + ".internal.Session;\nimport " + pkg + ".internal.Session.*;\n\npublic final class " + parser + " {\n    private " + parser + "() {}\n");
        out.append("    public static ParseResult<").append(ast).append("> parse(CharSequence source) { return parse(source, ParseOptions.DEFAULT); }\n")
            .append("    public static ParseResult<").append(ast).append("> parse(CharSequence source, ParseOptions options) {\n")
            .append("        return parseEntry(").append(q(grammar)).append(", ").append(q(defaultEntry)).append(", source, options);\n    }\n")
            .append("    public static ParseResult<").append(ast).append("> parseEntry(String grammar, String entry, CharSequence source, ParseOptions options) {\n")
            .append("        return parseEntryInternal(grammar, entry, source, options, false);\n    }\n")
            .append("    public static ParseResult<").append(ast).append("> parseEntryRollback(String grammar, String entry, CharSequence source, ParseOptions options) {\n")
            .append("        return parseEntryInternal(grammar, entry, source, options, true);\n    }\n")
            .append("    private static ParseResult<").append(ast).append("> parseEntryInternal(String grammar, String entry, CharSequence source, ParseOptions options, boolean rollback) {\n")
            .append("        Call entryPoint = switch (grammar) {\n");
        for (var unit : ir.grammarUnits()) {
            var symbols = ir.semantics().symbols().stream().filter(s -> s.grammarId().equals(unit.id()) && rules.containsKey(s.id())).toList();
            if (symbols.isEmpty()) {
                out.append("            case ").append(q(unit.name())).append(" -> throw new IllegalArgumentException(\"Grammar has no parser entry\");\n");
                continue;
            }
            var roots = symbols.stream().filter(s -> rules.get(s.id()).annotations().stream().anyMatch(a -> a.name().equals("root"))).toList();
            String rootId = roots.size() == 1 ? roots.getFirst().id() : roots.isEmpty() && unit.id().equals(ir.entry().grammarId()) ? ir.entry().ruleId() : null;
            out.append("            case ").append(q(unit.name())).append(" -> switch (entry) {\n");
            if (rootId != null) out.append("                case null -> ").append(entryCall(rootId)).append(";\n");
            else out.append("                case null -> throw new IllegalArgumentException(\"Grammar has no unique root\");\n");
            for (var symbol : symbols) out.append("                case ").append(q(symbol.name())).append(" -> ").append(entryCall(symbol.id())).append(";\n");
            out.append("                default -> throw new IllegalArgumentException(\"Unsupported entry: \" + entry);\n            };\n");
        }
        out.append("            default -> throw new IllegalArgumentException(\"Unsupported grammar: \" + grammar);\n        };\n")
            .append("        var s = new Session(source, options, ").append(kStrings(rules.keySet().stream().map(id -> ir.semantics().symbols().stream().filter(t -> t.id().equals(id)).map(Symbol::name).findFirst().orElse(id)).toList())).append(");\n")
            .append("        var f = new Frame(); Match match = s.parse(entryPoint, f);\n")
            .append("        if (rollback) { s.rollback(f); return s.result(f, null, Map.of()); }\n")
            .append("        ParseResult<").append(ast).append("> syntax = s.result(f, match, ").append(catalogs()).append(");\n")
            .append("        if (!syntax.ok() || !options.buildAst()) return syntax;\n")
            .append("        if (match.mappingFailure()) throw s.mapping(match.start(), match.end(), \"Recovered input cannot be mapped\", null);\n")
            .append("        if (match.nodes().isEmpty()) return syntax;\n")
            .append("        return syntax.withAst((").append(ast).append(") build(match.nodes().getFirst(), s), s.spans);\n    }\n");
        emitBuild(out);
        for (Rule rule : rules.values()) emitRule(out, rule);
        for (Expr expr : expressions.values()) emitExpr(out, expr);
        for (var constant : constants.entrySet()) {
            int split = constant.getKey().indexOf(' ');
            out.append("    private static final ").append(constant.getKey(), 0, split).append(' ').append(constant.getValue()).append(" = ").append(constant.getKey().substring(split + 1)).append(";\n");
        }
        return out.append("}\n").toString();
    }
    private String driverSource() throws IOException {
        String operatorTypes = rules.values().stream().filter(r -> r.operator() != null && recipes.containsKey(r.symbolId()))
            .map(r -> typeName(recipes.get(r.symbolId()).resultTypeId()) + ".class")
            .distinct().collect(Collectors.joining(", "));
        return resource("/templates/driver/Driver.java").replace("generated.", pkg + ".")
            .replace("__PARSER__", parser).replace("__AST__", ast).replace("__OPERATOR_TYPES__", operatorTypes).replace("__BINDINGS__", bindings()).replace("__EXTERN_CLASSES__", externClasses())
            .replace("__CATALOGS__", catalogMetadata()).replace("__SCOPE_MODES__", scopeModes());
    }
    /** D-025: `@catalog` の静的表。規則宣言順、context は初出順、capture 名は重複除去・ソート。 */
    private String catalogMetadata() {
        var rows = new java.util.ArrayList<String>();
        for (Rule rule : rules.values()) {
            var contexts = new java.util.LinkedHashSet<String>();
            for (var catalog : ir.semantics().catalogs())
                if (catalog.ruleId().equals(rule.symbolId())) contexts.add(catalog.context());
            for (String context : contexts) {
                var sites = ir.semantics().catalogs().stream()
                    .filter(c -> c.ruleId().equals(rule.symbolId()) && c.context().equals(context))
                    .map(GrammarIr.Catalog::captureSiteId).toList();
                var names = ir.semantics().captures().stream().filter(c -> sites.contains(c.siteId()))
                    .map(GrammarIr.Capture::name).distinct().sorted().map(JavaEmitter::q).collect(Collectors.joining(", "));
                String name = ir.semantics().symbols().stream().filter(symbol -> symbol.id().equals(rule.symbolId()))
                    .findFirst().orElseThrow().name();
                rows.add("object(\"rule\", " + q(name) + ", \"context\", " + q(context) + ", \"captures\", List.of(" + names + "))");
            }
        }
        return "List.of(" + String.join(", ", rows) + ")";
    }
    /** D-025: 文法が宣言した scope mode（宣言順・重複除去）。 */
    private String scopeModes() {
        return "List.of(" + ir.semantics().scope().effects().stream().map(GrammarIr.ScopeEffect::mode)
            .distinct().map(JavaEmitter::q).collect(Collectors.joining(", ")) + ")";
    }
    private String externClasses() {
        return "Map.ofEntries(" + tokens.values().stream().map(Token::spec)
            .filter(spec -> "extern".equals(spec.kind()))
            .map(spec -> "Map.entry(" + q(spec.logicalId()) + "," + q(spec.originalClassName()) + ")")
            .distinct().collect(Collectors.joining(",")) + ")";
    }
    private String bindings() {
        var counts = new java.util.HashMap<String, Integer>();
        return "Map.ofEntries(" + ir.semantics().captures().stream().map(c -> {
            String rule = ir.semantics().symbols().stream().filter(symbol -> symbol.id().equals(c.ruleId())).findFirst().orElseThrow().name();
            int index = counts.merge(c.ruleId(), 1, Integer::sum) - 1;
            return "Map.entry(" + q(c.siteId()) + "," + q(rule + ":" + index) + ")";
        }).collect(Collectors.joining(",")) + ")";
    }
    private String catalogs() {
        return constant("Map<String,String>", "Map.ofEntries(" + ir.semantics().catalogs().stream().map(c -> "Map.entry(" + q(c.captureSiteId()) + "," + q(c.context()) + ")").collect(Collectors.joining(",")) + ")");
    }
    private void emitRule(StringBuilder out, Rule rule) {
        Recipe recipe = recipes.get(rule.symbolId());
        var memo = ir.plan().memo().entries().stream().filter(e -> e.exprId().equals(rule.body())).findFirst().orElse(null);
        boolean safe = safe(rule.body(), new java.util.HashSet<>());
        boolean success = memo == null ? safe : memo.cacheSuccess(); boolean failure = memo == null ? safe : memo.cacheFailure();
        var scope = ir.semantics().scope().effects().stream().filter(e -> e.ruleId().equals(rule.symbolId())).toList();
        String effects = scope.stream().flatMap(e -> e.events().stream().map(v -> "new Effect(" + q(v.when()) + "," + q(v.action()) + "," + q(v.captureSiteId()) + "," + q(v.name()) + "," + q(e.mode()) + ")")).collect(Collectors.joining(","));
        String fields = recipe == null ? "" : recipe.fields().stream().map(f -> "new Field(" + q(f.field()) + "," + strings(f.siteIds()) + "," + q(f.conversion()) + "," + q(f.cardinality()) + ")").collect(Collectors.joining(","));
        out.append("    private static Match ").append(ruleMethod(rule.symbolId())).append("(Session s, Frame f) {\n        return s.rule(f, ").append(ruleIds.get(rule.symbolId())).append(", ").append(exprIds.size() + ruleIds.get(rule.symbolId())).append(", ").append(success).append(", ").append(failure)
            .append(", ").append(call(rule.body())).append(", ").append(q(recipe == null ? null : recipe.id())).append(", ").append(q(recipe == null || recipe.leafFallback() == null ? null : recipe.leafFallback().typeId())).append(", ").append(q(recipe == null ? null : recipe.kind()))
            .append(", ").append(constant("Field[]", "new Field[]{" + fields + "}")).append(", ").append(constant("Effect[]", "new Effect[]{" + effects + "}")).append(", ").append(rule.skip()).append(", ")
            .append(kLabels(ir.semantics().recovery().stream().filter(r -> r.ruleId().equals(rule.symbolId())).flatMap(r -> r.resolvedSyncPatterns().stream()).distinct().map(hint -> "'" + hint + "'").toList())).append(", ").append(rule.annotations().stream().anyMatch(a -> a.name().equals("rightAssoc"))).append(", ").append(kLabels(diagnosticPlan.rule(rule.symbolId()))).append(");\n    }\n");
        Recovery recovery = entryRecovery(rule.symbolId());
        if (recovery != null) out.append("    private static Match ").append(ruleMethod(rule.symbolId())).append("$entry(Session s, Frame f) {\n        ")
            .append(recoveryCall(recovery, parser + "::" + ruleMethod(rule.symbolId()))).append("\n    }\n");
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
        var memo = ir.plan().memo().entries().stream().filter(e -> e.exprId().equals(expr.id())).findFirst().orElse(null);
        // A rule body is memoized by Session.rule with the same key and flags; wrapping it again only doubles the table.
        boolean ruleBody = rules.values().stream().anyMatch(r -> r.body().equals(expr.id()));
        if (memo != null && (memo.cacheSuccess() || memo.cacheFailure()) && !ruleBody) {
            out.append("        return s.expression(f, ").append(exprIds.get(expr.id())).append(", ").append(memo.cacheSuccess()).append(", ").append(memo.cacheFailure())
                .append(", ").append(parser).append("::").append(method).append("Body);\n    }\n")
                .append("    private static Match ").append(method).append("Body(Session s, Frame f) {\n");
        }
        if (expr.execution().entryFlag().equals("setFalse")) out.append("        f.reset = false;\n");
        out.append("        return s.capture(f, f.matched ? ").append(parser).append("::").append(method).append("M : ").append(parser).append("::").append(method).append("C, ")
            .append(kStrings(expr.captureSiteIds())).append(", ").append(kStrings(sites.stream().map(Capture::name).toList())).append(", ").append(kStrings(sites.stream().map(Capture::spanRule).toList()))
            .append(", ").append(constant("boolean[]", "new boolean[]{" + sites.stream().map(c -> c.skip().toString()).collect(Collectors.joining(",")) + "}")).append(", ").append(q(expr.ruleId())).append(", ").append(q(expr.id())).append(", ").append(Set.of("repeat", "optional", "separated").contains(expr.node().kind())).append(", ").append(Set.of("literal", "tokenRef", "any", "eof", "empty").contains(expr.node().kind())).append(", ").append(q(semanticShapes.of(expr.id()).kind())).append(", ").append(semanticShapes.of(expr.id()).many() || expr.id().contains("/lower/separated/")).append(", ").append(kLabels(diagnosticPlan.expression(expr.id()))).append(");\n    }\n");
        String body = expression(expr);
        for (String mode : List.of("C", "M")) out.append("    private static Match ").append(method).append(mode).append("(Session s, Frame f) {\n        ").append(body).append("\n    }\n");
    }
    /** The label a primitive expression registers on failure (shared with Prediction). */
    String labelText(Expr expr) {
        ExprNode n = expr.node();
        String label = expr.diagnosticLabel();
        var display = ir.semantics().diagnostics().expectedDisplay().stream().filter(d -> d.exprId().equals(expr.id())).findFirst().orElse(null);
        if (display != null) label = display.java_();
        if (label == null) label = n.kind().equals("literal") ? "'" + n.value() + "'" : n.kind();
        return label;
    }
    private String expression(Expr expr) {
        ExprNode n = expr.node(); String boundary = boundaries(expr.id());
        String label = labelText(expr);
        return switch (n.kind()) {
            case "seq" -> "return s.sequence(f, " + calls(n.items()) + ", " + boundary + ");";
            case "group" -> expr.execution().transaction().equals("delegate") ? "return " + method(n.child()) + "(s, f);" : "return s.sequence(f, " + calls(List.of(n.child())) + ", " + boundary + ");";
            case "optional" -> "return s.optional(f, " + call(n.child()) + ");";
            case "repeat" -> "return s.repeat(f, " + call(n.child()) + ", " + n.min() + ", " + (n.max() == null ? "Integer.MAX_VALUE" : n.max()) + ", " + (n.terminator() == null ? "null" : call(n.terminator())) + ", " + boundary + (Integer.valueOf(0).equals(n.max()) && repeatedLiteral(n.child()) != null ? "," + label("'" + repeatedLiteral(n.child()) + "'") : "") + ");";
            case "separated" -> "return s.separated(f, " + call(n.element()) + ", " + call(n.separator()) + ", " + n.min() + ", " + boundary + ");";
            case "choice" -> choice(expr);
            case "literal" -> "return s.literal(f, " + q(n.value()) + ", " + n.caseSensitive() + ", " + label(label) + ");";
            case "ruleRef" -> {
                String call = parser + "::" + ruleMethod(n.symbolId());
                var recovery = ir.semantics().recovery().stream().filter(r -> r.applicationSites().contains(expr.id())).findFirst().orElse(null);
                yield recovery == null ? "return " + ruleMethod(n.symbolId()) + "(s, f);" : recoveryCall(recovery, call);
            }
            case "tokenRef" -> "return " + token(tokens.get(n.symbolId()).spec(), diagnosticPlan.tokenLabel(expr.id())) + ";";
            case "lookahead", "negativeLookahead" -> "return s.look(f, " + call(n.child()) + ", " + n.kind().equals("negativeLookahead") + ", " + label(label) + ");";
            case "any" -> "return s.any(f, " + label(label) + ");";
            case "eof" -> "return s.eof(f);"; case "empty" -> "return s.empty(f);";
            case "error" -> "return s.error(" + (expr.execution().transaction().equals("once") ? "f.copy(true)" : "f") + ", " + label(n.message()) + ");";
            default -> throw new IllegalArgumentException(n.kind());
        };
    }
    // D-021 / D-018 recovery-projection 2: @recovery は @root（および直接の entry）にも適用する。
    // IR の applicationSites は rule 参照だけを持つので、entry 用の wrapper で同じ recover を通す。
    private Recovery entryRecovery(String ruleId) {
        return ir.semantics().recovery().stream().filter(r -> r.ruleId().equals(ruleId)).findFirst().orElse(null);
    }
    private String entryCall(String ruleId) {
        return parser + "::" + (entryRecovery(ruleId) == null ? ruleMethod(ruleId) : ruleMethod(ruleId) + "$entry");
    }
    // design v1 §3.10: mode / 同期集合 / 同期文字列を消費するか / 未検出時の動作は
    // すべて IR が正本。ここで mode を推定し直すと backend ごとに受理が変わる。
    private String recoveryCall(Recovery recovery, String call) {
        List<String> patterns = recovery.resolvedSyncPatterns(); boolean consume = recovery.consumeSync();
        // §3.10: AUTO の FOLLOW が空なら既定の `;`。front は IR に載せて渡すが、
        // 手書き IR でも同じ規則にする。既定文字列は FOLLOW ではないので消費する。
        if (recovery.mode().equals("AUTO") && patterns.isEmpty()) { patterns = List.of(";"); consume = true; }
        return "return s.recover(f, " + call + ", " + q(recovery.ruleId()) + ", " + q(recovery.mode()) + ", "
            + strings(patterns) + ", " + consume + ", " + q(recovery.noSyncAction())
            + ", " + q(recovery.astProjection()) + ", " + q(recovery.recipeId()) + ");";
    }
    private boolean nullable(String id) {
        return ir.analysis().expressions().stream().filter(a -> a.exprId().equals(id)).findFirst().orElseThrow().nullable();
    }
    private String token(TokenSpec spec, String label) {
        return switch (spec.kind()) {
            case "builtin" -> "s.builtin(f," + q(spec.which()) + "," + constant("int[]", ints(spec.codePoints())) + "," + label(label) + ")";
            case "until" -> "s.until(f," + q(spec.terminator()) + ")";
            case "lookahead", "negativeLookahead" -> "s.look(f,(ss,ff)->ss.literal(ff," + q(spec.pattern()) + ",true," + label(label) + ")," + spec.kind().equals("negativeLookahead") + "," + label(label) + ")";
            case "negation" -> "s.range(f,0,1114111," + q(spec.excluded()) + "," + label(label) + ")";
            case "charRange" -> "s.range(f," + spec.min() + "," + spec.max() + ",null," + label(label) + ")";
            case "caseInsensitive" -> "s.literal(f," + q(spec.word()) + ",false," + label(label) + ")";
            case "regex" -> regex(spec, label);
            case "extern" -> "s.external(f," + q(spec.logicalId()) + "," + externIndex(spec.logicalId()) + ")";
            // D-027: 失敗候補の表示名は旧実装のクラス単純名（`WildCardCharacterParser`）。
            // UBNF の綴り `ANY` は表示名ではないので、診断 plan が出す label を使う。
            case "any" -> "s.any(f," + q(label) + ")"; case "eof" -> "s.eof(f)"; case "empty" -> "s.empty(f)";
            default -> throw new IllegalArgumentException(spec.kind());
        };
    }
    private final Map<String, Integer> externIndices = new LinkedHashMap<>();
    private int externIndex(String logicalId) { return externIndices.computeIfAbsent(logicalId, key -> externIndices.size()); }
    private String repeatedLiteral(String id) {
        ExprNode node = expressions.get(id).node();
        if (node.kind().equals("literal")) return node.value();
        if (node.kind().equals("group")) return repeatedLiteral(node.child());
        if (node.kind().equals("seq") && node.items().size() == 1) return repeatedLiteral(node.items().getFirst());
        return null;
    }
    private String regex(TokenSpec spec, String label) {
        if (spec.dfa() != null) {
            var dfa = spec.dfa();
            String transitions = dfa.transitions().stream().flatMap(t -> t.ranges().stream()
                .map(r -> "new int[]{" + t.from() + "," + r.get(0) + "," + r.get(1) + "," + t.to() + "}"))
                .collect(Collectors.joining(","));
            return "s.dfa(f," + dfa.start() + "," + constant("int[]", ints(dfa.accepting())) + "," + constant("int[][]", "new int[][]{" + transitions + "}") + "," + label(label) + ")";
        }
        var token = tokens.values().stream().filter(t -> t.spec() == spec).findFirst().orElseThrow();
        var plan = ir.plan().lexical().stream().filter(p -> p.tokenId().equals(token.symbolId()) && p.dfa() != null).findFirst().orElse(null);
        if (plan != null) {
            String transitions = plan.dfa().states().stream().flatMap(state -> state.transitions().stream()
                .map(t -> "new int[]{" + state.id() + "," + t.range().start() + "," + t.range().end() + "," + t.target() + "}"))
                .collect(Collectors.joining(","));
            return "s.dfa(f," + plan.dfa().startState() + "," + constant("int[]", ints(plan.dfa().states().stream().filter(s -> s.accepting()).map(s -> s.id()).toList()))
                + "," + constant("int[][]", "new int[][]{" + transitions + "}") + "," + label(label) + ")";
        }
        return "s.regex(f," + constant("Pattern", "Pattern.compile(" + q(spec.pattern()) + ")") + "," + label(label) + ")";
    }
    private String boundaries(String id) {
        var policies = Validation.index(ir.semantics().trivia().policies(), TriviaPolicy::id);
        var plan = ir.semantics().trivia().insertionPlan().stream().filter(p -> p.boundary().ownerExprId().equals(id)).toList();
        if (plan.isEmpty()) return "Trivia.NONE";
        ExprNode node = expressions.get(id).node();
        int children = node.kind().equals("seq") ? node.items().size() : 1;
        var afterChild = new String[children]; java.util.Arrays.fill(afterChild, "null");
        var phases = new java.util.HashMap<String, String>();
        for (var p : plan) {
            Boundary b = p.boundary();
            String delimiters = delimiters(p.policyId());
            if (b.at().equals("afterChild")) afterChild[b.childIndex()] = delimiters;
            else if (phases.put(b.at(), delimiters) != null) throw new IllegalStateException("Duplicate trivia boundary " + b.at() + " at " + id);
        }
        boolean anyAfter = java.util.Arrays.stream(afterChild).anyMatch(v -> !v.equals("null"));
        return constant("Trivia", "new Trivia(" + phases.getOrDefault("entry", "null") + "," + phases.getOrDefault("helperEntry", "null") + ","
            + (anyAfter ? constant("Delimiters[]", "new Delimiters[]{" + String.join(",", afterChild) + "}") : "null") + ","
            + phases.getOrDefault("helperExit", "null") + "," + phases.getOrDefault("exit", "null") + ")");
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
        String eofGuard = expr.node().selection().equals("predictive") && expr.node().alternatives().stream().allMatch(id -> !nullable(id))
            ? "if (s.cp(f.position()) < 0) return null; " : "";
        String guards = "null";
        if (!predict && !longest && prediction.ordered(expr)) {
            var plans = expr.node().alternatives().stream().map(prediction::guard).toList();
            if (plans.stream().anyMatch(java.util.Objects::nonNull))
                guards = constant("Guard[]", "new Guard[]{" + plans.stream().map(this::guard).collect(Collectors.joining(",")) + "}");
        }
        return eofGuard + "return s.choice(f," + calls(expr.node().alternatives()) + "," + longest + "," + eligible + "," + (predict && (plan == null || plan.fallback().equals("fullChoice"))) + (guards.equals("null") ? "" : "," + guards) + ");";
    }
    /** Static replay plan of one excluded alternative; see Prediction. */
    private String guard(Prediction.Guard plan) {
        if (plan == null) return "null";
        var bounds = new ArrayList<Integer>();
        for (int[] range : plan.allowed().items()) { bounds.add(range[0]); bounds.add(range[1]); }
        String labels = "new Label[]{" + plan.labels().stream()
            .map(text -> Prediction.TRIVIA_LABEL.equals(text) ? "null" : "Label.of(" + q(text) + ")").collect(Collectors.joining(",")) + "}";
        String trivia = plan.triviaPolicyId() == null ? "null" : delimiters(plan.triviaPolicyId());
        return constant("Guard", "new Guard(" + constant("int[]", ints(bounds)) + "," + constant("Label[]", labels) + ","
            + trivia + "," + constant("int[]", ints(plan.suffix())) + "," + plan.maxRuleDepth() + ")");
    }
    private String delimiters(String policyId) {
        TriviaPolicy policy = Validation.index(ir.semantics().trivia().policies(), TriviaPolicy::id).get(policyId);
        return constant("Delimiters", "new Delimiters(" + policy.effectiveDelimiterSet().stream()
            .map(d -> "new Delimiter(" + q(d.kind()) + "," + q(d.open()) + "," + q(d.close()) + "," + ints(d.codePoints()) + ")").collect(Collectors.joining(",")) + ")");
    }
    private void emitBuild(StringBuilder out) {
        if (recipes.values().stream().noneMatch(r -> r.resultTypeId() != null && !Set.of("transparent", "text", "skip").contains(r.kind()))) {
            out.append("    private static Object build(Recipe r, Session s) { if (r.id().equals(\"#recovery\")) return null; throw s.mapping(r.start(), r.end(), \"No AST recipe\", null); }\n");
            return;
        }
        out.append("    private static Object build(Recipe r, Session s) {\n        if (s.hasBuilt(r)) return s.built(r);\n        if (r.id().equals(\"#recovery\")) return s.remember(r, null);\n        try {\n        Object node = switch (r.id()) {\n");
        var fallbacks = new LinkedHashSet<String>();
        for (Recipe recipe : recipes.values()) {
            if (recipe.resultTypeId() == null || Set.of("transparent", "text", "skip").contains(recipe.kind())) continue;
            AstType type = types.get(recipe.resultTypeId());
            if (recipe.leafFallback() != null) fallbacks.add(recipe.leafFallback().typeId());
            out.append("            case ").append(q(recipe.id())).append(" -> ");
            if (type.kind().equals("enum")) {
                out.append("switch (s.source.substring(r.start(), r.end()).strip()) {\n");
                for (var variant : type.variants()) out.append("                case ").append(q(variant.literal())).append(" -> ").append(typeName(type.id())).append('.').append(enumNames(type).get(variant.literal())).append(";\n");
                out.append("                default -> throw s.mapping(r.start(), r.end(), \"Unknown enum literal\", null);\n            };\n");
            } else {
                out.append("{\n");
                var locals = new LinkedHashMap<String, String>();
                for (AstField field : type.fields()) {
                    String local = "field" + locals.size(); locals.put(field.name(), local);
                    var mapping = recipe.fields().stream().filter(f -> f.field().equals(field.name())).findFirst().orElseThrow();
                    out.append("                ").append(fieldType(field)).append(' ').append(local).append(" = s.")
                        .append(mapping.cardinality()).append("(r,").append(q(field.name())).append(",v -> ")
                        .append(convert(field.valueType(), mapping.conversion())).append(");\n");
                }
                for (var constraint : ir.semantics().typeofConstraints()) {
                    Capture own = captures.get(constraint.ownCapture()), referenced = captures.get(constraint.referencedCapture());
                    if (own.ruleId().equals(recipe.ruleId())) out.append("                s.typeof(r, ")
                        .append(q(own.name())).append(',').append(locals.get(own.name())).append(',')
                        .append(q(referenced.name())).append(',').append(locals.get(referenced.name())).append(");\n");
                }
                out.append("                yield new ").append(typeName(type.id())).append('(')
                    .append(String.join(", ", locals.values())).append(");\n            }\n");
            }
        }
        for (String fallback : fallbacks) out.append("            case ").append(q("#leaf:" + fallback)).append(" -> new ").append(typeName(fallback)).append("(null, List.of(s.source.substring(r.start(), r.end()).strip()), List.of());\n");
        out.append("            default -> throw s.mapping(r.start(), r.end(), \"Cannot build recipe: \" + r.id(), null);\n        };\n        return s.remember(r, node);\n        } catch (Session.RecoveredNodeMissing missing) { return s.remember(r, null); }\n    }\n");
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
