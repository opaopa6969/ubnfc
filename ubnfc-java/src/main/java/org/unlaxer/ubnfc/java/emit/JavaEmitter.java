package org.unlaxer.ubnfc.java.emit;

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
import org.unlaxer.ubnfc.java.ir.GrammarIr;
import org.unlaxer.ubnfc.java.ir.GrammarIr.*;
import org.unlaxer.ubnfc.java.ir.Validation;

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
    /**
     * D-078: 意味値を持つ選択の、text だけの候補（{@code @skip} 規則への参照を除く）。採用されたら
     * 一致範囲の字句 1 つを意味値にする（旧実装の {@code #semantic:text}）。内側の空反復が残す
     * 「値なし」（空の values）で字句を消さないよう、包みで values を plain text に戻す（{@code Session.textAlternative}）。
     * Rust は {@code Generator.text_alternatives}。
     */
    private final Set<String> textAlternatives = new java.util.HashSet<>();
    private final DiagnosticPlan diagnosticPlan;
    private final Prediction prediction;
    private final Factoring factoring;
    /** D-080: {@code plan.scans} の根ごとの走査関数（D-072 の Java 版）。 */
    private final Scanning scanning;
    private final Map<String, Integer> exprIds = new LinkedHashMap<>(), ruleIds = new LinkedHashMap<>();
    /** Generated constants (arrays of boundaries, fields, hints, ...) hoisted to static finals: expression text -> field name. */
    private final Map<String, String> constants = new LinkedHashMap<>();
    /**
     * Package of generated code when neither {@code --package} nor the grammar's own package is given
     * (issue #82 / D-*): under {@code org.unlaxer}, the Maven namespace the owner holds on Central, never a
     * namespace nobody can publish. Consumers that vendor the parser pass {@code --package} for their own.
     */
    public static final String DEFAULT_PACKAGE_PREFIX = "org.unlaxer.ubnfc.generated.";
    /** {@link #DEFAULT_PACKAGE_PREFIX} + the grammar name in lower case. */
    public static String defaultPackage(String grammar) {
        return DEFAULT_PACKAGE_PREFIX + grammar.toLowerCase(java.util.Locale.ROOT);
    }
    public JavaEmitter(GrammarIr ir, String packageName) { this(ir, packageName, true); }
    public JavaEmitter(GrammarIr ir, String packageName, boolean emitDriver) {
        Validation.validate(ir); this.ir = ir; this.emitDriver = emitDriver;
        var unit = ir.grammarUnits().stream().filter(g -> g.id().equals(ir.entry().grammarId())).findFirst().orElseThrow();
        grammar = unit.name(); Validation.identifier(grammar); ast = grammar + "AST"; parser = grammar + "Parser";
        pkg = packageName != null ? packageName : unit.package_() != null ? unit.package_() : defaultPackage(grammar);
        for (String part : pkg.split("\\.", -1)) Validation.identifier(part);
        expressions = Validation.index(ir.semantics().expressions(), Expr::id);
        rules = Validation.index(ir.semantics().rules(), Rule::symbolId);
        types = Validation.index(ir.semantics().astTypes(), AstType::id);
        captures = Validation.index(ir.semantics().captures(), Capture::siteId);
        tokens = Validation.index(ir.semantics().tokens(), Token::symbolId);
        recipes = Validation.index(ir.semantics().recipes(), Recipe::ruleId);
        semanticShapes = new SemanticShape(expressions, rules);
        for (Expr expr : expressions.values()) {
            if (!expr.node().kind().equals("choice") || semanticShapes.of(expr.id()).kind().equals("text")) continue;
            for (String alternative : expr.node().alternatives()) {
                var node = expressions.get(alternative).node();
                boolean skipped = node.kind().equals("ruleRef") && Boolean.TRUE.equals(rules.get(node.symbolId()).skip());
                if (!skipped && semanticShapes.of(alternative).kind().equals("text")) textAlternatives.add(alternative);
            }
        }
        diagnosticPlan = new DiagnosticPlan(ir);
        for (String id : expressions.keySet()) exprIds.put(id, exprIds.size());
        for (String id : rules.keySet()) ruleIds.put(id, ruleIds.size());
        prediction = new Prediction(ir, expressions, rules, tokens, ruleIds, diagnosticPlan, this::labelText);
        factoring = new Factoring(ir, expressions, rules, captures, semanticShapes, diagnosticPlan, this::labelText, id -> scopeSites().contains(id));
        scanning = new Scanning(ir, expressions, rules, tokens, recipes, exprIds, this::constant);
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
    static String q(String value) {
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
            // issue #35 / D-070: 既定呼出しは呼出し元 Thread の既定 -Xss のまま解析する。
            // Session.parse が DepthLimit ではなく StackOverflowError を捕まえた（＝maxDepth
            // にはまだ達していない）ときだけ、maxDepth から見積もった十分な stack を持つ
            // 専用 Thread で 1 度だけ解析し直す（Rust 版 crates/ubnfc-rust の native stack
            // 番兵エスカレーションと同じ契約。docs/reports/2026-09-24-runtime-limits.md）。
            .append("    private static ParseResult<").append(ast).append("> parseEntryInternal(String grammar, String entry, CharSequence source, ParseOptions options, boolean rollback) {\n")
            .append("        boolean[] stackFlag = new boolean[1];\n")
            .append("        ParseResult<").append(ast).append("> result = parseEntryAttempt(grammar, entry, source, options, rollback, stackFlag);\n")
            // issue #35 / D-070: extern scanner はエスカレーション先 Thread へ安全に渡せない
            // 保証が無い（任意の実装が状態を持ち得る）ので、scanner を伴う呼出しは対象外にする
            // （Rust 版 crates/ubnfc-rust の parse_with_scanner と同じ既知の制約）。
            .append("        if (result.ok() || !stackFlag[0] || !options.scanners().isEmpty()) return result;\n")
            .append("        return parseEntryEscalated(grammar, entry, source, options, rollback);\n    }\n")
            .append("    private static ParseResult<").append(ast).append("> parseEntryEscalated(String grammar, String entry, CharSequence source, ParseOptions options, boolean rollback) {\n")
            .append("        long stackBytes = Session.escalatedStackBytes(options.maxDepth());\n")
            .append("        var box = new java.util.concurrent.atomic.AtomicReference<ParseResult<").append(ast).append(">>();\n")
            .append("        var error = new java.util.concurrent.atomic.AtomicReference<Throwable>();\n")
            .append("        Thread thread = new Thread(null, () -> {\n")
            .append("            try { box.set(parseEntryAttempt(grammar, entry, source, options, rollback, new boolean[1])); }\n")
            .append("            catch (Throwable t) { error.set(t); }\n")
            .append("        }, \"ubnfc-parse-escalated\", stackBytes);\n")
            .append("        thread.start();\n")
            .append("        try { thread.join(); } catch (InterruptedException e) { Thread.currentThread().interrupt(); throw new RuntimeException(e); }\n")
            .append("        Throwable failure = error.get();\n")
            .append("        if (failure instanceof RuntimeException re) throw re;\n")
            .append("        if (failure instanceof Error er) throw er;\n")
            .append("        if (failure != null) throw new RuntimeException(failure);\n")
            .append("        return box.get();\n    }\n")
            .append("    private static ParseResult<").append(ast).append("> parseEntryAttempt(String grammar, String entry, CharSequence source, ParseOptions options, boolean rollback, boolean[] stackFlag) {\n")
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
            .append("        var s = new Session(source, options, ").append(kStrings(rules.keySet().stream().map(id -> ir.semantics().symbols().stream().filter(t -> t.id().equals(id)).map(Symbol::name).findFirst().orElse(id)).toList())).append(", ").append(twoMode()).append(", ").append(plainRecognition()).append(");\n")
            // D-080: rollback の結果は再解析されずに fast session の到達位置（reached）を診断位置として返すので、
            // 走査（reached を更新しない）を切って参照経路と同じ位置を返す。
            .append(scanning.roots.isEmpty() ? "" : "        if (rollback) s.scan = false;\n")
            .append("        var f = new Frame(); Match match = s.parse(entryPoint, f);\n")
            .append("        stackFlag[0] = stackFlag[0] || s.stackOverflowTripped;\n")
            .append("        if (rollback) { s.rollback(f); return s.result(f, null, Map.of()); }\n")
            .append("        ParseResult<").append(ast).append("> syntax = s.result(f, match, ").append(catalogs()).append(");\n")
            .append("        // D-023: 認識専用の fast path は観測を記録しない。診断を要する結果だけ決定的に再解析する。\n")
            .append("        if (!syntax.ok() && !s.diag) return parseEntryAttempt(grammar, entry, source, options.withDiagnostics(), rollback, stackFlag);\n")
            .append("        if (!syntax.ok() || !options.buildAst()) return syntax;\n")
            .append("        if (match.mappingFailure()) throw s.mapping(match.start(), match.end(), \"Recovered input cannot be mapped\", null);\n")
            .append("        if (match.nodes().isEmpty()) return syntax;\n")
            .append("        return syntax.withAst((").append(ast).append(") build(match.nodes().getFirst(), s), s.spans());\n    }\n");
        emitBuild(out);
        for (Rule rule : rules.values()) emitRule(out, rule);
        for (Expr expr : expressions.values()) emitExpr(out, expr);
        out.append(scanning.functions());
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
            .append(", ").append(constant("Fields", fields.isEmpty() ? "Fields.NONE" : "new Fields(" + fields + ")")).append(", ").append(constant("Effect[]", "new Effect[]{" + effects + "}")).append(", ").append(rule.skip()).append(", ")
            .append(kLabels(ir.semantics().recovery().stream().filter(r -> r.ruleId().equals(rule.symbolId())).flatMap(r -> r.resolvedSyncPatterns().stream()).distinct().map(hint -> "'" + hint + "'").toList())).append(", ").append(rightAssociativeSpan(rule)).append(", ").append(kLabels(diagnosticPlan.rule(rule.symbolId()))).append(");\n    }\n");
        Recovery recovery = entryRecovery(rule.symbolId());
        if (recovery != null) out.append("    private static Match ").append(ruleMethod(rule.symbolId())).append("$entry(Session s, Frame f) {\n        ")
            .append(recoveryCall(recovery, parser + "::" + ruleMethod(rule.symbolId()))).append("\n    }\n");
    }
    // `@rightAssoc` の span 補正は「rule 固有の trivia 方針がある」ときだけ掛ける。
    // 規則に同居する trivia 注釈が無い（＝ grammar 既定の trivia しかない）場合、旧実装の
    // 右再帰ノードは末尾 trivia を含む消費範囲をそのまま持つ
    // （unlaxer-dsl RightAssociativeMappingRuntimeTest: " 2 ^ 3 ^ 2 " で [5,11] / [9,11]）。
    // 規則局所の trivia があるときだけ significant end に切り替える
    // （rule-trivia/corpus.json:14 RightScoped: [11,12]）。Rust emit.rs と同一条件。
    private static boolean rightAssociativeSpan(Rule rule) {
        boolean rightAssoc = rule.annotations().stream().anyMatch(a -> a.name().equals("rightAssoc"));
        boolean localTrivia = rule.annotations().stream().anyMatch(a ->
            a.name().equals("interleave") || a.name().equals("whitespace") || a.name().equals("comment"));
        return rightAssoc && localTrivia;
    }
    private Set<String> scopeSiteIds;
    /** scope 効果が読む capture site（site が null の効果は出現表を読まない）。 */
    private Set<String> scopeSites() {
        if (scopeSiteIds == null) scopeSiteIds = ir.semantics().scope().effects().stream()
            .flatMap(e -> e.events().stream()).map(GrammarIr.ScopeEffectEventsItem::captureSiteId)
            .filter(java.util.Objects::nonNull).collect(Collectors.toSet());
        return scopeSiteIds;
    }
    /**
     * D-056: 認識専用の二段構えを許す文法か。@recover は RecoveryEvent と Recipe marker を
     * 認識結果にも載せるので対象外。scope 効果が反復下（quantified）の capture site を読む
     * 文法も、出現表を落とすと scope 観測が変わるので対象外にする。
     */
    private boolean twoMode() {
        if (!ir.semantics().recovery().isEmpty()) return false;
        var scoped = scopeSites();
        if (scoped.isEmpty()) return true;
        return expressions.values().stream().noneMatch(e ->
            Set.of("repeat", "optional", "separated").contains(e.node().kind())
                && e.captureSiteIds().stream().anyMatch(scoped::contains));
    }
    /**
     * D-075: 認識専用経路（{@code tree=false}）の成功値が情報を持たなくてよい文法か。
     * その経路で {@code Match} の区間・text・出現列を読むのは {@code Session.applyEffects}
     * （scope 効果）と {@code Session.wrap} の scoped site だけで、どちらも {@code @scope} 効果が
     * 1 つも無ければ存在しない（{@link #scopeSites} が空なら {@code capture} の包みは
     * {@code bypassCapture} で飛ぶ）。@mapping の field を読むのは AST 経路だけである。
     */
    private boolean plainRecognition() { return ir.semantics().scope().effects().isEmpty(); }
    private boolean safe(String id, Set<String> visited) {
        if (!visited.add(id)) return true;
        Expr expr = expressions.get(id);
        if (expr.node().kind().equals("tokenRef")) { var token = tokens.get(expr.node().symbolId()); return token.memoSafety().success() && token.memoSafety().failure(); }
        if (expr.node().kind().equals("ruleRef")) return safe(rules.get(expr.node().symbolId()).body(), visited);
        return Validation.children(expr.node()).stream().allMatch(c -> safe(c, visited));
    }
    private void emitExpr(StringBuilder out, Expr expr) {
        String method = method(expr.id());
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
        // 認識専用経路（Match 木も診断も作らない）で capture の包みが恒等になる式は、包みを飛ばして
        // 中身を直に呼ぶ。到達位置の更新だけ同じ位置で行う（包みの中の progress と同じ）。
        // D-068: AST を作る経路でも、読まれる capture site を 1 つも持たない式は包みが
        // 「意味値の射影」だけになる。射影を生成側の定数に畳んで包みを飛ばす。
        if (treeBypassable(expr))
            out.append("        if (s.bypassWrap) { Match r = f.matched ? ").append(method).append("M(s, f) : ").append(method)
                .append("C(s, f); s.progress(f); ").append(projectTail(expr)).append(" }\n");
        else if (bypassable(expr))
            out.append("        if (s.bypassCapture) { Match r = f.matched ? ").append(method).append("M(s, f) : ").append(method)
                .append("C(s, f); s.progress(f); return r; }\n");
        out.append("        return s.capture(f, f.matched ? ").append(parser).append("::").append(method).append("M : ").append(parser).append("::").append(method).append("C, ")
            .append(capArgs(expr)).append(");\n    }\n");
        String body = expression(expr);
        // D-080: 走査は consumed mode（C）だけ。matchOnly mode（M）は参照経路のまま。
        for (String mode : List.of("C", "M")) out.append("    private static Match ").append(method).append(mode).append("(Session s, Frame f) {\n        ")
            .append(mode.equals("C") ? scanning.fastPath(expr.id()) : "").append(body).append("\n    }\n");
    }
    /** {@code Session.capture} / {@code Session.Cap} が共有する 12 引数（site 表・span 規則・意味形・失敗 hint）。 */
    private String capArgs(Expr expr) {
        var sites = expr.captureSiteIds().stream().map(captures::get).toList();
        return kStrings(expr.captureSiteIds()) + ", " + kStrings(sites.stream().map(Capture::name).toList()) + ", " + kStrings(sites.stream().map(Capture::spanRule).toList())
            + ", " + constant("boolean[]", "new boolean[]{" + sites.stream().map(c -> c.skip().toString()).collect(Collectors.joining(",")) + "}")
            + ", " + constant("boolean[]", "new boolean[]{" + sites.stream().map(c -> Boolean.toString(scopeSites().contains(c.siteId()))).collect(Collectors.joining(",")) + "}")
            + ", " + q(expr.ruleId()) + ", " + q(expr.id())
            + ", " + Set.of("repeat", "optional", "separated").contains(expr.node().kind())
            + ", " + Set.of("literal", "tokenRef", "any", "eof", "empty").contains(expr.node().kind())
            + ", " + q(textAlternatives.contains(expr.id()) ? "textAlternative" : semanticShapes.of(expr.id()).kind())
            + ", " + (semanticShapes.of(expr.id()).many() || expr.id().contains("/lower/separated/"))
            + ", " + kLabels(diagnosticPlan.expression(expr.id()));
    }
    /** 認識専用経路（{@code Session.bypassCapture}）で capture の包みが恒等になる式か（D-059）。 */
    private boolean bypassable(Expr expr) {
        return expr.captureSiteIds().stream().map(captures::get).noneMatch(c -> scopeSites().contains(c.siteId()) && !c.skip());
    }
    /**
     * D-068: 公開出現表を要求しない AST 経路（{@code Session.bypassWrap}）で、capture の包みが
     * 「意味値の射影」だけになる式か。出現を 1 件も残さなくてよい条件は
     * 「この式が持つ capture site を、@mapping の field も scope 効果も読まない」こと。
     * 反復下の site は包みが項目ごとの Match を組み直すので、site を持つ反復は対象外にする。
     */
    private boolean treeBypassable(Expr expr) {
        var sites = expr.captureSiteIds();
        if (!sites.isEmpty() && Set.of("repeat", "optional", "separated").contains(expr.node().kind())) return false;
        return sites.stream().map(captures::get).noneMatch(c -> readSites().contains(c.siteId()) && !c.skip());
    }
    /** {@link #treeBypassable} の式が包みの代わりに行う射影（{@code Session.project}）。 */
    private String projectTail(Expr expr) {
        var shape = semanticShapes.of(expr.id());
        boolean many = shape.many() || expr.id().contains("/lower/separated/");
        // 認識専用経路（tree=false）では射影そのものが無いので、呼出しを出さずに畳む。
        if (textAlternatives.contains(expr.id())) return "return s.tree ? s.textAlternative(r) : r;";
        return shape.kind().equals("text") ? "return r;"
            : "return s.tree ? s.project(r, " + (shape.kind().equals("mixed") && !many) + ") : r;";
    }
    private Set<String> readSiteIds;
    /**
     * 実行時に capture 出現を読む site。@mapping の recipe field（{@code Session.rule} が
     * 局所出現から field 値を集める）と scope 効果（{@code Session.applyEffects}）の 2 つだけが読む。
     * {@code sameRuleText} は site ではなく名前で引くので、その mode があれば名前一致も読む側に数える。
     */
    private Set<String> readSites() {
        if (readSiteIds == null) {
            var read = new java.util.HashSet<>(scopeSites());
            for (Recipe recipe : recipes.values()) for (var field : recipe.fields()) read.addAll(field.siteIds());
            var byName = ir.semantics().scope().effects().stream().filter(e -> "sameRuleText".equals(e.mode()))
                .flatMap(e -> e.events().stream()).map(GrammarIr.ScopeEffectEventsItem::name)
                .filter(java.util.Objects::nonNull).collect(Collectors.toSet());
            if (!byName.isEmpty()) captures.values().stream().filter(c -> byName.contains(c.name())).forEach(c -> read.add(c.siteId()));
            readSiteIds = read;
        }
        return readSiteIds;
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
            case "literal" -> "return s.literal(f, " + q(n.value()) + ", " + n.caseSensitive() + ", " + Boolean.TRUE.equals(n.wordBoundary()) + ", " + label(label) + ");";
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
            case "caseInsensitive" -> "s.literal(f," + q(spec.word()) + ",false," + Boolean.TRUE.equals(spec.wordBoundary()) + "," + label(label) + ")";
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
        // D-061: 先頭要素を共有する連続候補の左因数分解。候補除外の条件が一致する群だけを作る。
        String groups = "null";
        if (!predict && !longest && prediction.ordered(expr)) {
            var keys = expr.node().alternatives().stream().map(id -> guardKey(prediction.guard(id))).toList();
            var factored = factoring.plan(expr.node().alternatives(), keys);
            if (!factored.isEmpty()) groups = constant("Group[]", "new Group[]{" + factored.stream()
                .map(g -> group(expr.node().alternatives(), g)).collect(Collectors.joining(",")) + "}");
        }
        String tail = groups.equals("null") ? (guards.equals("null") ? "" : "," + guards) : "," + guards + "," + groups;
        return eofGuard + exclusiveMiss(expr, plan) + "return s.choice(f," + calls(expr.node().alternatives()) + "," + longest + "," + eligible + "," + (predict && (plan == null || plan.fallback().equals("fullChoice"))) + tail + ");";
    }
    /**
     * D-066: front が {@code exclusiveCandidates} を証明した選択（{@code strategy=predict} かつ
     * {@code fallback=none}）で、呼出し位置の文字がどの候補の先頭でもないときに候補を 1 つも呼ばずに
     * 失敗する短絡。Rust backend の {@code expressions.rs::predict_literals} と同じ条件で、
     * 診断モードでは {@code Session#exclusiveReady} が偽になるので従来どおり全候補を走らせる。
     */
    private String exclusiveMiss(Expr expr, ChoicePlan plan) {
        if (plan == null || !plan.strategy().equals("predict") || !plan.fallback().equals("none")) return "";
        if (plan.proof() == null || !plan.proof().kind().equals("exclusiveCandidates")) return "";
        var firsts = new java.util.TreeSet<Integer>();
        for (String id : expr.node().alternatives()) {
            ExprNode alternative = expressions.get(id).node();
            // 証明の内容（単一 code point・case sensitive）を生成時にもう一度確かめる。
            if (!alternative.kind().equals("literal") || !Boolean.TRUE.equals(alternative.caseSensitive())) return "";
            String value = alternative.value();
            if (value == null || value.codePointCount(0, value.length()) != 1) return "";
            if (!firsts.add(value.codePointAt(0))) return "";
        }
        if (firsts.isEmpty()) return "";
        return "if (s.exclusiveReady(f) && switch (s.cp(f.c)) { case "
            + firsts.stream().map(String::valueOf).collect(Collectors.joining(", "))
            + " -> false; default -> true; }) return s.exclusiveFail(f); ";
    }
    /** 候補除外の条件の同一性（群にできるのは条件が一致する候補だけ）。 */
    private static String guardKey(Prediction.Guard guard) {
        if (guard == null) return "";
        var out = new StringBuilder();
        for (int[] range : guard.allowed().items()) out.append(range[0]).append(':').append(range[1]).append(',');
        return out.append('|').append(guard.labels()).append('|').append(guard.triviaPolicyId())
            .append('|').append(guard.suffix()).append('|').append(guard.maxRuleDepth()).toString();
    }
    private String group(List<String> alternatives, Factoring.Group g) {
        Expr first = expressions.get(alternatives.get(g.start()));
        int width = 0;
        for (int k = g.start(); k < g.end(); k++) width = Math.max(width, expressions.get(alternatives.get(k)).node().items().size());
        return "new Group(" + g.start() + "," + g.end() + "," + first.execution().entryFlag().equals("setFalse") + ","
            + boundaries(first.id()) + "," + width + "," + level(alternatives, g, 0, new int[0]) + ")";
    }
    /** 群の 1 段（index {@code depth} の要素を共有し、次の段で分岐する）。 */
    private String level(List<String> alternatives, Factoring.Group g, int depth, int[] evaluated) {
        Expr head = expressions.get(alternatives.get(g.start()));
        int[] next = java.util.Arrays.copyOf(evaluated, depth + 1); next[depth] = g.start();
        String branches = factoring.level(alternatives, g, depth + 1).stream()
            .map(sub -> sub.end() - sub.start() >= 2 ? "new Branch(" + level(alternatives, sub, depth + 1, next) + ")"
                : branch(alternatives, sub.start(), depth + 1, next))
            .collect(Collectors.joining(","));
        return "new Level(" + call(head.node().items().get(depth)) + "," + boundaries(head.id()) + "," + depth + "," + branches + ")";
    }
    /** 単独候補になった枝（残りの要素・出口 trivia・capture の包み・共有要素の付け替え）。 */
    private String branch(List<String> alternatives, int k, int from, int[] evaluated) {
        Expr candidate = expressions.get(alternatives.get(k));
        List<String> items = candidate.node().items();
        var retags = new ArrayList<String>();
        boolean any = false;
        for (int d = 0; d < from; d++) {
            Expr element = expressions.get(items.get(d));
            var site = element.captureSiteIds().stream().map(captures::get).filter(c -> !c.skip()).findFirst().orElse(null);
            if (evaluated[d] == k || site == null) { retags.add("null"); retags.add("null"); continue; }
            any = true; retags.add(q(site.siteId())); retags.add(q(site.name()));
        }
        return "new Branch(" + calls(items.subList(from, items.size())) + "," + boundaries(candidate.id()) + "," + from + ","
            + constant("Cap", "new Cap(" + capArgs(candidate) + "," + bypassable(candidate) + "," + treeBypassable(candidate) + ")") + ","
            + (any ? constant("String[]", "new String[]{" + String.join(",", retags) + "}") : "null") + ")";
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
        // D-079: recipe ごとに 1 つの static method を出し、field の取り出しと変換を直に書く。
        // 以前は s.scalar(r, i, name, v -> ...) に r / s を捕捉する lambda を渡していたので、
        // 1 field ごとに lambda 実体を確保し、Session 側の呼出しは recipe 数ぶんの型を見る
        // megamorphic な interface 呼出しになっていた（inline されない）。
        out.append("    private static Object build(Recipe r, Session s) {\n        if (s.hasBuilt(r)) return s.built(r);\n        if (r.id().equals(\"#recovery\")) return s.remember(r, null);\n        try {\n        Object node = switch (r.id()) {\n");
        var fallbacks = new LinkedHashSet<String>();
        var methods = new StringBuilder();
        int methodIndex = 0;
        for (Recipe recipe : recipes.values()) {
            if (recipe.resultTypeId() == null || Set.of("transparent", "text", "skip").contains(recipe.kind())) continue;
            AstType type = types.get(recipe.resultTypeId());
            if (recipe.leafFallback() != null) fallbacks.add(recipe.leafFallback().typeId());
            out.append("            case ").append(q(recipe.id())).append(" -> ");
            if (type.kind().equals("enum")) {
                out.append("switch (s.source.substring(r.start(), r.end()).strip()) {\n");
                for (var variant : type.variants()) out.append("                case ").append(q(variant.literal())).append(" -> ").append(typeName(type.id())).append('.').append(enumNames(type).get(variant.literal())).append(";\n");
                out.append("                default -> throw s.mapping(r.start(), r.end(), \"Unknown enum literal\", null);\n            };\n");
                continue;
            }
            String method = "b" + methodIndex++;
            out.append(method).append("(r, s);\n");
            methods.append("    private static Object ").append(method).append("(Recipe r, Session s) {\n");
            var locals = new LinkedHashMap<String, String>();
            for (AstField field : type.fields()) {
                int n = locals.size();
                String local = "field" + n; locals.put(field.name(), local);
                var mapping = recipe.fields().stream().filter(f -> f.field().equals(field.name())).findFirst().orElseThrow();
                // Session.rule は recipe.fields() の順で値列を詰めるので、添字は生成時に確定する。
                int index = recipe.fields().indexOf(mapping);
                String javaType = fieldType(field);
                switch (mapping.cardinality()) {
                    case "scalar" -> methods.append("        Recipe.Value v").append(n).append(" = s.scalarValue(r,").append(index).append(',').append(q(field.name())).append(");\n")
                        .append("        ").append(javaType).append(' ').append(local).append(" = s.present(")
                        .append(convert(field.valueType(), mapping.conversion(), "v" + n)).append(", v").append(n).append(");\n");
                    case "optional" -> methods.append("        Recipe.Value v").append(n).append(" = s.firstValue(r,").append(index).append(");\n")
                        .append("        ").append(javaType).append(' ').append(local).append(" = v").append(n).append(" == null ? java.util.Optional.empty() : java.util.Optional.ofNullable(")
                        .append(convert(field.valueType(), mapping.conversion(), "v" + n)).append(");\n");
                    case "list" -> methods.append("        List<Recipe.Value> l").append(n).append(" = r.fieldValues(").append(index).append(");\n")
                        .append("        Object[] a").append(n).append(" = l").append(n).append(".isEmpty() ? null : new Object[l").append(n).append(".size()]; int k").append(n).append(" = 0;\n")
                        .append("        for (int i = 0, n = l").append(n).append(".size(); i < n; i++) { Recipe.Value v = l").append(n).append(".get(i); Object x = ")
                        .append(convert(field.valueType(), mapping.conversion(), "v")).append("; if (x != null) a").append(n).append("[k").append(n).append("++] = x; }\n")
                        .append("        ").append(javaType).append(' ').append(local).append(" = s.listOf(a").append(n).append(", k").append(n).append(");\n");
                    default -> throw new IllegalArgumentException(mapping.cardinality());
                }
            }
            for (var constraint : ir.semantics().typeofConstraints()) {
                Capture own = captures.get(constraint.ownCapture()), referenced = captures.get(constraint.referencedCapture());
                if (own.ruleId().equals(recipe.ruleId())) methods.append("        s.typeof(r, ")
                    .append(q(own.name())).append(',').append(locals.get(own.name())).append(',')
                    .append(q(referenced.name())).append(',').append(locals.get(referenced.name())).append(");\n");
            }
            methods.append("        return new ").append(typeName(type.id())).append('(')
                .append(String.join(", ", locals.values())).append(");\n    }\n");
        }
        for (String fallback : fallbacks) out.append("            case ").append(q("#leaf:" + fallback)).append(" -> new ").append(typeName(fallback)).append("(null, List.of(s.source.substring(r.start(), r.end()).strip()), List.of());\n");
        out.append("            default -> throw s.mapping(r.start(), r.end(), \"Cannot build recipe: \" + r.id(), null);\n        };\n        return s.remember(r, node);\n        } catch (Session.RecoveredNodeMissing missing) { return s.remember(r, null); }\n    }\n");
        out.append(methods);
    }
    private String convert(ValueType type, String conversion, String v) {
        String java = javaType(type, true);
        return switch (conversion) {
            case "text" -> "s.text(" + v + ")"; case "number" -> "s.number(" + v + ")";
            case "node" -> (java.equals("java.lang.Object") ? "" : "(" + java + ") ") + "build(s.node(" + v + ", r), s)";
            case "mixed" -> {
                boolean text = type.members().stream().anyMatch(t -> t.kind().equals("text"));
                boolean number = type.members().stream().anyMatch(t -> t.kind().equals("number"));
                String leaf = text ? "s.text(" + v + ")" : number ? "s.number(" + v + ")" : "null";
                yield (java.equals("java.lang.Object") ? "" : "(" + java + ") ") + "(" + v + ".nodes().isEmpty() ? " + leaf + " : build(s.node(" + v + ", r), s))";
            }
            default -> throw new IllegalArgumentException(conversion);
        };
    }
}
