package org.unlaxer.dsl;

import com.google.gson.*;
import java.io.*;
import java.lang.reflect.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.stream.*;
import javax.tools.*;
import org.unlaxer.*;
import org.unlaxer.context.ParseContext;
import org.unlaxer.dsl.bootstrap.UBNFAST.*;
import org.unlaxer.dsl.bootstrap.UBNFMapper;
import org.unlaxer.dsl.codegen.*;
import org.unlaxer.dsl.runtime.ScopeStore;
import org.unlaxer.parser.Parser;

/** 既存 Java 実装の公開観測値を JSON に固定する独立ドライバ。 */
public final class OracleCollector {
    static final Gson JSON = new GsonBuilder().setPrettyPrinting().serializeNulls().disableHtmlEscaping().create();
    static final String PACKAGE = "org.example.oracle.";
    static Path scratch;
    static String revision;

    public static void main(String[] args) throws Exception {
        var specs = JsonParser.parseString(Files.readString(Path.of(args[0]))).getAsJsonArray();
        Path output = Path.of(args[1]);
        scratch = Path.of(args[2]).toAbsolutePath();
        revision = args[3];
        Files.createDirectories(output);
        for (var spec : specs) {
            var row = spec.getAsJsonObject();
            String slug = row.get("slug").getAsString();
            JsonObject result = collect(row, Files.createDirectory(scratch.resolve(slug)));
            Files.writeString(output.resolve(slug + ".json"), JSON.toJson(result) + "\n");
            System.out.println(slug + ": " + result.getAsJsonArray("cases").size() + " cases; " + result.get("stage"));
        }
    }

    static JsonObject collect(JsonObject spec, Path dir) throws Exception {
        var result = new JsonObject();
        String grammarSource = spec.get("grammar").getAsString();
        result.add("feature", spec.get("feature"));
        result.addProperty("grammar", grammarSource);
        result.addProperty("source", revision);
        if (spec.has("imports")) result.add("imports", spec.get("imports"));
        var cases = new JsonArray();
        result.add("cases", cases);
        for (var c : spec.getAsJsonArray("cases")) cases.add(emptyCase(c.getAsJsonObject()));
        Path grammarFile = dir.resolve("Probe.ubnf");
        Files.writeString(grammarFile, grammarSource);
        if (spec.has("imports")) for (var e : spec.getAsJsonObject("imports").entrySet())
            Files.writeString(dir.resolve(e.getKey()), e.getValue().getAsString());
        Path generated = Files.createDirectory(dir.resolve("generated"));
        var out = new ByteArrayOutputStream();
        var err = new ByteArrayOutputStream();
        int exit = CodegenMain.run(new String[]{"--grammar", grammarFile.toString(), "--output", generated.toString(),
            "--generators", "parser,ast,mapper"}, new PrintStream(out, true, StandardCharsets.UTF_8), new PrintStream(err, true, StandardCharsets.UTF_8));
        var generation = new JsonObject();
        if (exit == 2 && err.toString(StandardCharsets.UTF_8).contains("Unknown generator: parser")) {
            generation.addProperty("lowercaseCliExit", exit);
            generation.addProperty("lowercaseCliError", normalize(err.toString(StandardCharsets.UTF_8)));
            out.reset();
            err.reset();
            exit = CodegenMain.run(new String[]{"--grammar", grammarFile.toString(), "--output", generated.toString(),
                "--generators", "Parser,AST,Mapper"}, new PrintStream(out, true, StandardCharsets.UTF_8), new PrintStream(err, true, StandardCharsets.UTF_8));
        }
        generation.addProperty("cliGenerators", "Parser,AST,Mapper");
        generation.addProperty("cliExit", exit);
        generation.addProperty("cliError", normalize(err.toString(StandardCharsets.UTF_8)));
        result.add("generation", generation);
        GrammarDecl grammar;
        try {
            // CLI は parseWithImports を呼ばないため、import は API 経路も別途実行する。
            if (spec.has("imports")) {
                grammar = UBNFMapper.parseWithImports(grammarSource, path -> {
                    try { return Files.readString(dir.resolve(path)); }
                    catch (IOException e) { return null; }
                }).grammars().get(0);
                generation.addProperty("api", "UBNFMapper.parseWithImports + ParserGenerator/ASTGenerator/MapperGenerator");
                var resolved = new JsonArray();
                for (RuleDecl rule : grammar.rules()) {
                    var entry = new JsonObject();
                    entry.addProperty("name", rule.name());
                    entry.addProperty("body", rule.body().toString());
                    resolved.add(entry);
                }
                generation.add("resolvedRules", resolved);
                generation.add("resolvedTokens", JSON.toJsonTree(grammar.tokens().stream().map(TokenDecl::name).toList()));
                for (CodeGenerator generator : List.of(new ParserGenerator(), new ASTGenerator(), new MapperGenerator())) {
                    var source = generator.generate(grammar);
                    Path target = generated.resolve(source.packageName().replace('.', '/')).resolve(source.className() + ".java");
                    Files.createDirectories(target.getParent());
                    Files.writeString(target, source.source());
                }
            } else {
                if (exit != 0) { result.addProperty("stage", "generation_failed"); return result; }
                grammar = UBNFMapper.parse(grammarSource).grammars().get(0);
            }
        } catch (RuntimeException e) {
            generation.addProperty("apiError", error(e));
            result.addProperty("stage", "generation_failed");
            return result;
        }
        Path classes = Files.createDirectory(dir.resolve("classes"));
        List<Path> sources;
        try (var paths = Files.walk(generated)) { sources = paths.filter(p -> p.toString().endsWith(".java")).sorted().toList(); }
        var hashes = new JsonObject();
        for (Path source : sources) hashes.addProperty(generated.relativize(source).toString(), sha(source));
        result.add("generatedSourceSha256", hashes);
        JsonObject compilation = compile(sources, classes, System.getProperty("java.class.path"));
        result.add("compilation", compilation);
        if (!compilation.get("success").getAsBoolean()) {
            result.addProperty("stage", "compilation_failed");
            return result;
        }
        try (var loader = new URLClassLoader(new URL[]{classes.toUri().toURL()}, OracleCollector.class.getClassLoader())) {
            Class<?> parsers = loader.loadClass(PACKAGE + "ProbeParsers");
            Class<?> mapper = loader.loadClass(PACKAGE + "ProbeMapper");
            Class<?> ast = loader.loadClass(PACKAGE + "ProbeAST");
            result.add("astTypes", types(ast));
            if (spec.has("evaluator")) result.add("evaluator", evaluator(grammar, generated, classes, loader));
            if (spec.get("feature").getAsString().equals("@catalog")) {
                // Java frontend を通じた既存 IR metadata の観測。runtime parser は上で別途生成・コンパイル済み。
                var catalogs = new JsonArray();
                for (var rule : org.unlaxer.dsl.codegen.rust.RustGrammarLowering.lower(grammar).rules()) {
                    Object catalog = call(rule, "catalog");
                    if (catalog != null) {
                        var item = record(catalog);
                        item.addProperty("rule", rule.name());
                        catalogs.add(item);
                    }
                }
                result.add("catalogs", catalogs);
            }
            Map<String, String> bindings = bindings(grammar);
            for (var c : cases) {
                JsonObject observation = c.getAsJsonObject();
                observe(observation, parsers, mapper, bindings);
                if (spec.get("feature").getAsString().equals("@enum")) {
                    try {
                        Object value = loader.loadClass(PACKAGE + "ProbeAST$Mode").getMethod("fromText", String.class)
                            .invoke(null, observation.get("input").getAsString());
                        observation.addProperty("enumValue", ((Enum<?>) value).name());
                    } catch (InvocationTargetException e) { observation.addProperty("enumError", error(e)); }
                }
            }
            result.addProperty("stage", "executed");
        }
        return result;
    }

    static JsonObject emptyCase(JsonObject input) {
        JsonObject c = input.deepCopy();
        for (String key : List.of("accepted", "allConsumed", "consumed", "matched", "ast", "mappingError")) c.add(key, JsonNull.INSTANCE);
        for (String key : List.of("captures", "declarations", "references", "diagnostics")) c.add(key, new JsonArray());
        return c;
    }

    static void observe(JsonObject c, Class<?> parsers, Class<?> mapper, Map<String,String> names) throws Exception {
        String input = c.get("input").getAsString();
        Parser parser = (Parser) parsers.getMethod("getRootParser").invoke(null);
        try (var ctx = new ParseContext(StringSource.createRootSource(input))) {
            try {
                Parsed parsed = parser.parse(ctx);
                c.addProperty("accepted", parsed.isSucceeded());
                c.addProperty("parsedStatus", parsed.status.name());
                if (parsed.isSucceeded()) captures(parsed.getRootToken(false), names, c.getAsJsonArray("captures"));
            } catch (RuntimeException e) {
                c.addProperty("parseError", error(e));
            }
            c.addProperty("allConsumed", ctx.allConsumed());
            c.addProperty("consumed", ctx.getConsumedPosition().value());
            c.addProperty("matched", ctx.getMatchedPosition().value());
            var nativeFailure = ctx.getParseFailureDiagnostics();
            var farthest = new JsonObject();
            farthest.addProperty("offset", nativeFailure.getFarthestOffset());
            farthest.addProperty("consumed", nativeFailure.getFarthestConsumedOffset());
            farthest.addProperty("matched", nativeFailure.getFarthestMatchedOffset());
            farthest.add("expected", JSON.toJsonTree(nativeFailure.getExpectedTokens().stream().sorted().toList()));
            c.add("nativeFailure", farthest);
            c.add("declarations", JSON.toJsonTree(ScopeStore.getAllDeclarations(ctx)));
            c.add("references", JSON.toJsonTree(ScopeStore.getAllReferences(ctx)));
            c.addProperty("scopeDepth", ScopeStore.currentScopeDepth(ctx));
            for (var diagnostic : ScopeStore.getDiagnostics(ctx)) {
                JsonObject item = JSON.toJsonTree(diagnostic).getAsJsonObject();
                item.addProperty("kind", diagnostic.severity().name());
                // ScopeStore には expected API が存在しない。
                item.add("expected", new JsonArray());
                item.addProperty("origin", "ScopeStore");
                c.getAsJsonArray("diagnostics").add(item);
            }
        }
        try {
            Optional<?> diagnostic = (Optional<?>) mapper.getMethod("diagnose", String.class).invoke(null, input);
            if (diagnostic.isPresent()) {
                JsonObject item = record(diagnostic.get());
                item.addProperty("origin", "Mapper.diagnose");
                c.getAsJsonArray("diagnostics").add(item);
            }
        } catch (InvocationTargetException e) { c.addProperty("diagnosticError", error(e)); }
        // parseWithSourceMap は全入力受理の場合だけ呼ぶ。構文失敗を mapping 例外と混同しない。
        if (c.get("accepted").equals(new JsonPrimitive(true)) && c.get("allConsumed").getAsBoolean()) {
            try {
                Object mapped = mapper.getMethod("parseWithSourceMap", String.class).invoke(null, input);
                Object value = call(mapped, "ast");
                try { c.add("ast", canonical(value, mapped)); }
                catch (ReflectiveOperationException | IllegalStateException e) { c.addProperty("canonicalError", error(e)); }
            } catch (InvocationTargetException e) { c.addProperty("mappingError", error(e)); }
        }
    }

    static void captures(Token token, Map<String,String> names, JsonArray result) throws Exception {
        if (token == null) return;
        for (Token child : token.getOriginalChildren()) captures(child, names, result);
        Method method;
        try { method = token.parser.getClass().getMethod("captureBindings"); }
        catch (NoSuchMethodException e) { return; }
        for (Object binding : (List<?>) method.invoke(token.parser)) {
            if (!names.containsKey(binding.toString())) continue; // 内部 semantic binding は capture ではない。
            var item = new JsonObject();
            item.addProperty("name", names.get(binding.toString()));
            item.addProperty("binding", binding.toString());
            int start = token.source.offsetFromRoot().value();
            int end = start + token.source.codePointLength().value();
            item.add("span", JSON.toJsonTree(new int[]{start, end}));
            item.addProperty("text", token.source.sourceAsString());
            result.add(item);
        }
    }

    static Map<String,String> bindings(GrammarDecl grammar) throws Exception {
        // 名前と生成 site ID の対応だけを既存計画から読む。出現は実行後の token tree から読む。
        Class<?> planType = Class.forName("org.unlaxer.dsl.codegen.CaptureBindingPlan");
        Constructor<?> ctor = planType.getDeclaredConstructor(RuleDecl.class);
        ctor.setAccessible(true);
        Field field = planType.getDeclaredField("sites");
        field.setAccessible(true);
        Map<String,String> result = new LinkedHashMap<>();
        for (RuleDecl rule : grammar.rules()) {
            Map<?,?> sites = (Map<?,?>) field.get(ctor.newInstance(rule));
            for (var entry : sites.entrySet()) for (Object site : (List<?>) entry.getValue()) {
                Method id = site.getClass().getDeclaredMethod("id");
                id.setAccessible(true);
                result.put((String) id.invoke(site), (String) entry.getKey());
            }
        }
        return result;
    }

    static JsonElement canonical(Object value, Object mapped) throws Exception {
        if (value == null) return JsonNull.INSTANCE;
        if (value instanceof String text) return new JsonPrimitive(text);
        if (value instanceof Number number) return new JsonPrimitive(number);
        if (value instanceof Enum<?> e) return new JsonPrimitive(e.name());
        if (value instanceof Optional<?> optional) return canonical(optional.orElse(null), mapped);
        if (value instanceof List<?> list) {
            var result = new JsonArray();
            for (var element : list) result.add(canonical(element, mapped));
            return result;
        }
        var result = new JsonObject();
        result.addProperty("type", value.getClass().getSimpleName());
        int[] span = (int[]) ((Optional<?>) mapped.getClass().getMethod("sourceSpanOf", Object.class).invoke(mapped, value)).orElseThrow();
        result.add("span", JSON.toJsonTree(span));
        var fields = new JsonObject();
        for (var component : value.getClass().getRecordComponents()) fields.add(component.getName(), canonical(component.getAccessor().invoke(value), mapped));
        result.add("fields", fields);
        return result;
    }

    static JsonArray types(Class<?> type) throws Exception {
        var result = new JsonArray();
        for (Class<?> nested : Arrays.stream(type.getDeclaredClasses()).sorted(Comparator.comparing(Class::getName)).toList()) {
            var item = new JsonObject();
            item.addProperty("name", nested.getName().substring((PACKAGE + "ProbeAST$").length()));
            item.addProperty("kind", nested.isEnum() ? "enum" : nested.isRecord() ? "record" : "interface");
            var methods = new JsonArray();
            for (Method m : Arrays.stream(nested.getDeclaredMethods()).sorted(Comparator.comparing(Method::toString)).toList()) {
                if (m.isSynthetic() || List.of("equals", "hashCode", "toString").contains(m.getName())) continue;
                var method = new JsonObject();
                method.addProperty("name", m.getName());
                method.addProperty("returnType", m.getGenericReturnType().getTypeName());
                method.addProperty("abstract", Modifier.isAbstract(m.getModifiers()));
                methods.add(method);
            }
            item.add("methods", methods);
            if (nested.isEnum()) item.add("constants", JSON.toJsonTree(Arrays.stream(nested.getEnumConstants()).map(Object::toString).toList()));
            result.add(item);
            result.addAll(types(nested));
        }
        return result;
    }

    static JsonObject evaluator(GrammarDecl grammar, Path generated, Path classes, ClassLoader loader) throws Exception {
        var result = new JsonObject();
        CodeGenerator.GeneratedSource source;
        try { source = new EvaluatorGenerator().generate(grammar); }
        catch (RuntimeException e) { result.addProperty("generationError", error(e)); return result; }
        Path file = generated.resolve("ProbeEvaluator.java");
        Files.writeString(file, source.source());
        result.addProperty("sourceSha256", sha(file));
        String cp = classes + File.pathSeparator + System.getProperty("java.class.path");
        var compilation = compile(List.of(file), classes, cp);
        result.add("compilation", compilation);
        if (compilation.get("success").getAsBoolean()) {
            Class<?> evaluator = loader.loadClass(PACKAGE + "ProbeEvaluator");
            var methods = new JsonArray();
            for (var method : Arrays.stream(evaluator.getDeclaredMethods()).sorted(Comparator.comparing(Method::toString)).toList()) {
                var item = new JsonObject();
                item.addProperty("name", method.getName());
                item.addProperty("abstract", Modifier.isAbstract(method.getModifiers()));
                item.add("parameters", JSON.toJsonTree(Arrays.stream(method.getParameterTypes()).map(Class::getName).toList()));
                methods.add(item);
            }
            result.add("methods", methods);
            Path concrete = generated.resolve("EmptySemantics.java");
            Files.writeString(concrete, "package org.example.oracle; public class EmptySemantics extends ProbeEvaluator<Object> {}\n");
            result.add("emptyConcreteSubclass", compile(List.of(concrete), classes, cp));
        }
        return result;
    }

    static JsonObject compile(List<Path> sources, Path classes, String cp) throws Exception {
        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        var diagnostics = new DiagnosticCollector<JavaFileObject>();
        boolean success;
        try (var fm = compiler.getStandardFileManager(diagnostics, Locale.ROOT, StandardCharsets.UTF_8)) {
            success = compiler.getTask(new StringWriter(), fm, diagnostics,
                List.of("--enable-preview", "--release", "21", "-encoding", "UTF-8", "-proc:none", "-classpath", cp, "-d", classes.toString()),
                null, fm.getJavaFileObjectsFromPaths(sources)).call();
        }
        var result = new JsonObject();
        result.addProperty("success", success);
        var messages = new JsonArray();
        for (var d : diagnostics.getDiagnostics()) {
            var item = new JsonObject();
            item.addProperty("kind", d.getKind().name());
            item.addProperty("code", d.getCode());
            item.addProperty("file", d.getSource() == null ? null : Path.of(d.getSource().toUri()).getFileName().toString());
            item.addProperty("line", d.getLineNumber());
            item.addProperty("message", normalize(d.getMessage(Locale.ROOT)));
            messages.add(item);
        }
        result.add("diagnostics", messages);
        return result;
    }

    static Object call(Object object, String method) throws Exception { return object.getClass().getMethod(method).invoke(object); }
    static JsonObject record(Object object) throws Exception {
        var result = new JsonObject();
        for (var component : object.getClass().getRecordComponents()) result.add(component.getName(), JSON.toJsonTree(component.getAccessor().invoke(object)));
        return result;
    }
    static String normalize(String s) { return s.replace(scratch.toString(), "<scratch>"); }
    static String error(Throwable e) {
        while (e instanceof InvocationTargetException && e.getCause() != null) e = e.getCause();
        return normalize(e.getClass().getName() + ": " + e.getMessage());
    }
    static String sha(Path p) throws Exception { return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(p))); }
}
