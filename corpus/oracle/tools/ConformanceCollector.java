package org.unlaxer.dsl;

import com.google.gson.*;
import java.lang.reflect.*;
import java.net.URLClassLoader;
import java.nio.file.*;
import java.util.*;
import org.unlaxer.StringSource;
import org.unlaxer.context.ParseContext;
import org.unlaxer.dsl.bootstrap.UBNFMapper;
import org.unlaxer.dsl.codegen.*;
import org.unlaxer.dsl.codegen.rust.RustGrammarLowering;
import org.unlaxer.dsl.runtime.ScopeStore.SymbolInfo;
import org.unlaxer.parser.Parser;

/** 既存テストの Java 観測メソッドをそのまま実行する追加採取器。テストの期待値は読まない。 */
public final class ConformanceCollector {
    static Path repo;
    static final String COLLECTION = "triage-2026-09-22";

    public static void main(String[] args) throws Exception {
        var specs = JsonParser.parseString(Files.readString(Path.of(args[0]))).getAsJsonArray();
        Path output = Path.of(args[1]);
        Files.createDirectories(output);
        OracleCollector.scratch = Path.of(args[2]);
        OracleCollector.revision = args[3];
        repo = Path.of(args[4]);
        // 既存ファイルは一切上書きしない。再採取は必ず別 output に行う。
        for (var item : specs) {
            Path path = output.resolve(item.getAsJsonObject().get("slug").getAsString() + ".json");
            if (Files.exists(path)) throw new IllegalStateException("出力が既に存在します: " + path);
        }
        for (var item : specs) {
            JsonObject spec = item.getAsJsonObject();
            String slug = spec.get("slug").getAsString();
            String kind = spec.get("observation").getAsString();
            JsonObject result;
            if (kind.equals("oracle")) {
                result = OracleCollector.collect(spec, Files.createDirectory(OracleCollector.scratch.resolve(slug)));
            } else {
                result = envelope(spec);
                switch (kind) {
                    case "catalog" -> catalog(spec, result);
                    case "scope-store" -> scopeStore(spec, result);
                    default -> parser(spec, result, kind);
                }
            }
            result.addProperty("collection", COLLECTION);
            result.add("origin", spec.get("origin"));
            result.add("variant", spec.get("variant"));
            result.addProperty("observation", kind);
            Files.writeString(output.resolve(slug + ".json"), OracleCollector.JSON.toJson(result) + "\n");
            System.out.println(slug + ": " + result.getAsJsonArray("cases").size());
        }
    }

    static JsonObject envelope(JsonObject spec) {
        var result = new JsonObject();
        result.add("feature", spec.get("feature"));
        if (spec.has("grammar")) result.add("grammar", spec.get("grammar"));
        if (spec.has("grammarPath")) result.add("grammarPath", spec.get("grammarPath"));
        result.addProperty("source", OracleCollector.revision);
        result.addProperty("stage", "executed");
        result.add("cases", new JsonArray());
        for (String key : List.of("queries", "captureNames", "valueSpanField", "valueSpanListField"))
            if (spec.has(key)) result.add(key, spec.get(key));
        return result;
    }

    static Object invoke(Object target, String name, Object... args) throws Exception {
        for (Method method : target.getClass().getDeclaredMethods()) {
            if (!method.getName().equals(name) || method.getParameterCount() != args.length) continue;
            Class<?>[] types = method.getParameterTypes();
            boolean matches = true;
            for (int i = 0; i < types.length; i++) {
                Class<?> type = types[i] == int.class ? Integer.class : types[i] == boolean.class ? Boolean.class : types[i];
                if (args[i] != null && !type.isInstance(args[i])) matches = false;
            }
            if (!matches) continue;
            method.setAccessible(true);
            try { return method.invoke(target, args); }
            catch (InvocationTargetException e) {
                if (e.getCause() instanceof Exception cause) throw cause;
                if (e.getCause() instanceof Error cause) throw cause;
                throw e;
            }
        }
        throw new NoSuchMethodException(target.getClass().getName() + "." + name);
    }

    static JsonObject identity(JsonObject row) {
        var result = new JsonObject();
        result.add("id", row.get("id"));
        result.add("input", row.get("input"));
        return result;
    }

    static void evidence(JsonObject result, String test, String methods) throws Exception {
        var evidence = new JsonObject();
        String path = "unlaxer-dsl/src/test/java/org/unlaxer/dsl/codegen/" + test + ".java";
        evidence.addProperty("path", path);
        evidence.addProperty("sha256", OracleCollector.sha(repo.resolve(path)));
        evidence.addProperty("methods", methods);
        result.add("observer", evidence);
    }

    static void parser(JsonObject spec, JsonObject result, String kind) throws Exception {
        var test = new RustConformanceTest();
        test.temporary.create();
        var scopeTest = new ScopeAnnotationConformanceTest();
        var nestedTest = new NestedCaptureConformanceTest();
        var grammar = UBNFMapper.parse(spec.get("grammar").getAsString()).grammars().get(0);
        boolean evolution = kind.equals("evolution");
        Object extra = evolution ? Integer.valueOf(spec.get("variant").getAsInt()) : List.of();
        try (var loader = (URLClassLoader) invoke(test, "compileJava", grammar, extra)) {
            // package は生成済み class から探し、文法原文の package/name を変更しない。
            Path classes = Path.of(loader.getURLs()[0].toURI());
            String mapperName;
            try (var paths = Files.walk(classes)) {
                Path mapper = paths.filter(p -> p.getFileName().toString().equals(grammar.name() + "Mapper.class"))
                    .findFirst().orElseThrow();
                mapperName = classes.relativize(mapper).toString().replace('/', '.').replace(".class", "");
            }
            Class<?> mapper = loader.loadClass(mapperName);
            Class<?> parsers = loader.loadClass(mapperName.replace("Mapper", "Parsers"));
            Parser parser = (Parser) parsers.getMethod("getRootParser").invoke(null);
            if (kind.equals("scope-annotations")) {
                result.add("metadata", (JsonElement) invoke(scopeTest, "javaMetadata", parsers, spec.getAsJsonArray("metadata")));
                evidence(result, "ScopeAnnotationConformanceTest", "parseJava, javaMetadata, canonical");
            } else if (kind.equals("nested-captures")) {
                evidence(result, "NestedCaptureConformanceTest", "javaValueSpans, declarations, canonical");
            } else {
                evidence(result, "RustConformanceTest", evolution ? "compileJava, javaResult, javaDiagnostic, canonical"
                    : "compileJava; cardinalityPreservesOptionalListsSpansAndEvaluation / rustMixedCorpus の Java 観測");
            }
            JsonArray rows = spec.getAsJsonArray("cases");
            for (int i = 0; i < rows.size(); i++) {
                JsonObject row = rows.get(i).getAsJsonObject();
                String input = row.get("input").getAsString();
                JsonObject observed = identity(row);
                if (evolution) {
                    observed = OracleCollector.emptyCase(observed);
                    // 要求された共通 schema の prefix cursor/capture/scope は従来採取器で実測。
                    OracleCollector.observe(observed, parsers, mapper, OracleCollector.bindings(grammar));
                    JsonObject java = (JsonObject) invoke(test, "javaResult", loader, input);
                    boolean full = observed.get("accepted").getAsBoolean() && observed.get("allConsumed").getAsBoolean();
                    if (full != java.get("ok").getAsBoolean()) throw new AssertionError("受理経路の不一致: " + row);
                    if (full && !observed.get("ast").equals(java.get("ast"))) throw new AssertionError("AST 経路の不一致");
                    for (String key : List.of("value", "evaluationError")) if (java.has(key)) observed.add(key, java.get(key));
                    // 成功時 evaluationError は既存 javaResult と同じくキーなし。false を補わない。
                } else {
                    Optional<?> diagnostic = (Optional<?>) mapper.getMethod("diagnose", String.class).invoke(null, input);
                    boolean accepted = diagnostic.isEmpty();
                    observed.addProperty("accepted", accepted);
                    if (kind.equals("mixed-values")) {
                        try (var context = new ParseContext(StringSource.createRootSource(input))) {
                            observed.add("prefix", OracleCollector.JSON.toJsonTree(new Object[]{parser.parse(context).isSucceeded(),
                                context.getConsumedPosition().value(), context.getMatchedPosition().value()}));
                        }
                    }
                    if (kind.equals("scope-annotations")) {
                        Object parsed = invoke(scopeTest, "parseJava", parser, input, spec.getAsJsonArray("queries"), row.has("parentRollback"));
                        if (row.has("parentRollback")) observed.add("parentRollback", row.get("parentRollback"));
                        observed.add("prefix", (JsonElement) invoke(parsed, "prefix"));
                        observed.add("scope", (JsonElement) invoke(parsed, "scope"));
                        observed.add("rollback", (JsonElement) invoke(parsed, "rollback"));
                    }
                    if (accepted) {
                        Object mapped = mapper.getMethod("parseWithSourceMap", String.class).invoke(null, input);
                        Object ast = OracleCollector.call(mapped, "ast");
                        observed.add("ast", (JsonElement) invoke(test, "canonical", ast, mapped));
                        if (kind.equals("nested-captures")) {
                            observed.add("declarations", (JsonElement) invoke(nestedTest, "declarations", parser, input));
                            observed.add("valueSpans", (JsonElement) invoke(nestedTest, "javaValueSpans", spec, ast, mapped));
                            if (spec.has("valueSpanField")) {
                                String next = rows.get((i + 1) % rows.size()).getAsJsonObject().get("input").getAsString();
                                mapper.getMethod("parse", String.class).invoke(null, next);
                            }
                            observed.add("retainedValueSpans", (JsonElement) invoke(nestedTest, "javaValueSpans", spec, ast, mapped));
                        }
                    }
                }
                result.getAsJsonArray("cases").add(observed);
            }
        } finally { test.temporary.delete(); }
    }

    static void scopeStore(JsonObject spec, JsonObject result) throws Exception {
        var test = new ScopeStoreConformanceTest();
        evidence(result, "ScopeStoreConformanceTest", "executeJava, javaState");
        for (var item : spec.getAsJsonArray("cases")) {
            var row = item.getAsJsonObject();
            JsonObject observed = identity(row);
            var actual = new ArrayList<JsonObject>();
            var retained = new TreeMap<String, List<SymbolInfo>>();
            String[] id = row.get("id").getAsString().split("/", 2);
            JsonArray operations = row.getAsJsonArray("operations").deepCopy();
            try (var context = new ParseContext(StringSource.createRootSource(row.get("input").getAsString()))) {
                invoke(test, "executeJava", id[0], id[1], operations, context, retained, new ArrayList<JsonObject>(), actual);
            }
            Iterator<JsonObject> snapshots = actual.iterator();
            attachStates(operations, snapshots);
            if (snapshots.hasNext()) throw new AssertionError("余分な snapshot");
            observed.add("operations", operations);
            result.getAsJsonArray("cases").add(observed);
        }
    }

    static void attachStates(JsonArray operations, Iterator<JsonObject> snapshots) {
        for (var item : operations) {
            JsonObject op = item.getAsJsonObject();
            if (op.get("op").getAsString().equals("observe")) {
                var snapshot = snapshots.next();
                if (!op.get("id").equals(snapshot.get("observe"))) throw new AssertionError("snapshot 順序");
                op.add("expected", snapshot.get("state"));
            }
            if (op.has("operations")) attachStates(op.getAsJsonArray("operations"), snapshots);
        }
    }

    static void catalog(JsonObject spec, JsonObject result) throws Exception {
        JsonArray fixtures = JsonParser.parseString(Files.readString(repo.resolve(
            "unlaxer-dsl/src/test/resources/catalog/corpus.json"))).getAsJsonArray();
        JsonObject fixture = fixtures.asList().stream().map(JsonElement::getAsJsonObject)
            .filter(f -> f.get("name").equals(spec.get("fixtureName"))).findFirst().orElseThrow();
        result.add("grammar", fixture.get("grammar"));
        var grammar = UBNFMapper.parse(fixture.get("grammar").getAsString()).grammars().get(0);
        GrammarValidator.validateOrThrow(grammar);
        var catalogs = new JsonArray();
        for (var rule : RustGrammarLowering.lower(grammar).rules()) {
            Object catalog = OracleCollector.call(rule, "catalog");
            if (catalog == null) continue;
            var item = new JsonObject();
            item.addProperty("rule", rule.name());
            item.addProperty("context", (String) OracleCollector.call(catalog, "context"));
            item.add("captures", OracleCollector.JSON.toJsonTree(OracleCollector.call(catalog, "captures")));
            catalogs.add(item);
        }
        JsonObject row = identity(spec.getAsJsonArray("cases").get(0).getAsJsonObject());
        row.add("catalogs", catalogs);
        result.getAsJsonArray("cases").add(row);
        evidence(result, "RustCatalogConformanceTest", "javaFrontendPreservesContextAndSortedLocalCapturesInIr の Java API 経路");
    }
}
