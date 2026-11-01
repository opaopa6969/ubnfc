package org.unlaxer.ubnfc.lsp;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Map;
import javax.tools.ToolProvider;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.unlaxer.ubnfc.java.ir.IrReader;

/** 生成・コンパイル・JSON-RPC 往復を含む文法意味検査・外部カタログ の golden。更新後は必ず通常実行でも照合する。 */
class ValidationGoldenTest {
    @Test void generatedValidation(@TempDir Path work) throws Exception {
        Path sources = work.resolve("sources");
        Path classes = work.resolve("classes");
        Files.createDirectories(classes);
        Path fixture = Path.of("src/test/resources/validation");
        generate(fixture.resolve("catalog-demo.ir.json"), sources, "catalog");
        generate(Path.of("../ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json"), sources, "ubnf");
        Files.copy(fixture.resolve("ValidationScenario.java"), sources.resolve("ValidationScenario.java"));
        Path front;
        String configured = System.getProperty("ubnfc.validation.command");
        if (configured != null) front = Path.of(configured).toAbsolutePath();
        else {
            var build = new ProcessBuilder("cargo", "build", "--bin", "ubnfc").directory(Path.of("..").toFile()).inheritIO();
            build.environment().put("RUSTUP_TOOLCHAIN", "stable");
            assertEquals(0, build.start().waitFor(), "実際の front を組み立てる");
            front = Path.of("../target/debug/ubnfc").toAbsolutePath().normalize();
        }
        var compiler = ToolProvider.getSystemJavaCompiler();
        assertNotNull(compiler, "JDK が必要です");
        try (var manager = compiler.getStandardFileManager(null, null, null);
             var paths = Files.walk(sources)) {
            var files = paths.filter(p -> p.toString().endsWith(".java")).sorted().toList();
            var options = new ArrayList<>(java.util.List.of("--release", "21", "-encoding", "UTF-8",
                "-classpath", System.getProperty("surefire.test.class.path", System.getProperty("java.class.path")),
                "-d", classes.toString()));
            assertEquals(true, compiler.getTask(null, manager, null, options, null,
                manager.getJavaFileObjectsFromPaths(files)).call(), "生成物のコンパイル");
        }
        try (var loader = new URLClassLoader(new java.net.URL[]{classes.toUri().toURL()}, getClass().getClassLoader())) {
            Object result = loader.loadClass("ValidationScenario").getMethod("run", Path.class, Path.class, Path.class)
                .invoke(null, work, front, fixture.toAbsolutePath());
            assertNotNull(result);
            for (var entry : ((Map<?, ?>) result).entrySet()) {
                Path golden = Path.of("src/test/resources/validation", entry.getKey() + ".json");
                String actual = (String) entry.getValue();
                if (Boolean.getBoolean("ubnfc.golden.update")) Files.writeString(golden, actual);
                assertEquals(Files.readString(golden), actual, entry.getKey().toString());
            }
        }
    }

    private static void generate(Path ir, Path out, String name) throws Exception {
        String parser = "org.unlaxer.ubnfc.examples." + name + ".generated";
        org.unlaxer.ubnfc.java.Main.main(new String[]{"--ir", ir.toString(), "--out", out.toString(),
            "--package", parser, "--no-emit-driver"});
        var lsp = new LspEmitter(IrReader.read(ir), "org.unlaxer.ubnfc.examples." + name + ".lsp",
            parser, name, null, null, null);
        lsp.write(out);
    }
}
