import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import javax.tools.ToolProvider;
import org.unlaxer.StringSource;
import org.unlaxer.context.ParseContext;
import org.unlaxer.dsl.bootstrap.UBNFMapper;
import org.unlaxer.dsl.codegen.ASTGenerator;
import org.unlaxer.dsl.codegen.CodeGenerator;
import org.unlaxer.dsl.codegen.MapperGenerator;
import org.unlaxer.dsl.codegen.ParserGenerator;
import org.unlaxer.parser.Parser;

/**
 * 旧 Java 実測値を独立採取するドライバ。ubnfc の生成器・実行結果・期待値は使用しない。
 * 引数: unlaxer-parser checkout、ubnfc checkout、mktemp -d で作成した出力先。
 * Maven の test source 外に置き、旧 Java のクラスパスだけで実行する（README.md 参照）。
 */
public final class LegacyDiagnosticsDump {
    private record Spec(String slug, String grammar, String inputs) {}

    public static void main(String[] args) throws Exception {
        if (args.length != 3) throw new IllegalArgumentException("Expected legacy checkout, ubnfc checkout, scratch directory");
        Path legacy = Path.of(args[0]), ubnfc = Path.of(args[1]), scratch = Path.of(args[2]);
        var gson = new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create();
        for (var spec : List.of(
                new Spec("operators", "unlaxer-dsl/src/test/resources/associative/Operators.ubnf",
                    "examples/handwritten-target/java/src/test/resources/corpus/operators.json"),
                new Spec("power", "unlaxer-dsl/src/test/resources/right-associative/Power.ubnf",
                    "examples/handwritten-target/java/src/test/resources/corpus/power.json"),
                new Spec("tinycalc", "unlaxer-dsl/examples/tinycalc.ubnf",
                    "ubnfc-java/src/test/resources/golden/tinycalc.json"))) {
            String grammarSource = Files.readString(legacy.resolve(spec.grammar()));
            if (spec.slug().equals("tinycalc")) {
                // 旧 bootstrap の制限を避ける。javaStyle 自体が同じ行コメント処理を含む。
                grammarSource = grammarSource.substring(grammarSource.indexOf("grammar TinyCalc {"))
                    .replace("\r\n", "\n").replace("  @comment: { line: \"//\" }\n", "");
            }
            var grammar = UBNFMapper.parse(grammarSource).grammars().getFirst();
            Path dir = Files.createDirectories(scratch.resolve("legacy-" + spec.slug()));
            Files.writeString(dir.resolve(spec.slug() + ".ubnf"), grammarSource);
            var sources = new ArrayList<java.io.File>();
            String pkg = null;
            for (CodeGenerator generator : List.of(new ParserGenerator(), new ASTGenerator(), new MapperGenerator())) {
                var source = generator.generate(grammar);
                pkg = source.packageName();
                Path file = dir.resolve(source.className() + ".java");
                Files.writeString(file, source.source());
                sources.add(file.toFile());
            }
            var compiler = ToolProvider.getSystemJavaCompiler();
            try (var manager = compiler.getStandardFileManager(null, null, null)) {
                if (!compiler.getTask(null, manager, null,
                        List.of("-cp", System.getProperty("java.class.path"), "-d", dir.toString()), null,
                        manager.getJavaFileObjectsFromFiles(sources)).call()) {
                    throw new IllegalStateException("Legacy compilation failed: " + spec.slug());
                }
            }
            var result = new JsonArray();
            try (var loader = new URLClassLoader(new URL[]{dir.toUri().toURL()}, LegacyDiagnosticsDump.class.getClassLoader())) {
                Parser parser = (Parser) loader.loadClass(pkg + "." + grammar.name() + "Parsers")
                    .getMethod("getRootParser").invoke(null);
                var inputs = JsonParser.parseString(Files.readString(ubnfc.resolve(spec.inputs()))).getAsJsonArray();
                for (var row : inputs) {
                    // 入力だけを読む。元 fixture の expected / diagnostics / AST / value は使わない。
                    String input = row.getAsJsonObject().get("input").getAsString();
                    try (var context = new ParseContext(StringSource.createRootSource(input))) {
                        parser.parse(context);
                        var diagnostic = context.getParseFailureDiagnostics();
                        var out = new JsonObject();
                        out.addProperty("_comment", "旧 Java 実測値。LegacyDiagnosticsDump で " + grammar.name()
                            + " の root parser にこの行の input を渡し、ParseFailureDiagnostics を採取。"
                            + "expectedTokens はソートのみ（削除・追加なし）。");
                        out.addProperty("input", input);
                        out.addProperty("farthestCp", diagnostic.getFarthestOffset());
                        out.add("farthestExpected", gson.toJsonTree(diagnostic.getExpectedTokens().stream().sorted().toList()));
                        result.add(out);
                    }
                }
            }
            Files.writeString(scratch.resolve("legacy-" + spec.slug() + ".json"), gson.toJson(result) + "\n");
        }
    }
}
