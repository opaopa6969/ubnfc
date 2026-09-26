package org.unlaxer.ubnfc.java;

import java.io.IOException;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import javax.tools.ToolProvider;
import org.unlaxer.ubnfc.java.emit.JavaEmitter;
import org.unlaxer.ubnfc.java.ir.GrammarIr;
import org.unlaxer.ubnfc.java.ir.Json;
import static org.junit.jupiter.api.Assertions.*;

final class CompilerHarness implements AutoCloseable {
    final Path directory;
    final URLClassLoader loader;
    final Class<?> parser, options, json;
    final String pkg;
    CompilerHarness(GrammarIr ir, String pkg) throws Exception {
        this(ir, pkg, Map.of());
    }
    CompilerHarness(GrammarIr ir, String pkg, Map<String, String> additionalSources) throws Exception {
        this.pkg = pkg;
        var process = new ProcessBuilder("mktemp", "-d").start();
        directory = Path.of(new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8).strip());
        assertEquals(0, process.waitFor());
        var emitter = new JavaEmitter(ir, pkg);
        emitter.write(directory.resolve("src"));
        for (var source : additionalSources.entrySet()) {
            Path file = directory.resolve("src").resolve(source.getKey()); Files.createDirectories(file.getParent());
            Files.writeString(file, source.getValue(), StandardCharsets.UTF_8);
        }
        var compiler = ToolProvider.getSystemJavaCompiler(); assertNotNull(compiler);
        var messages = new javax.tools.DiagnosticCollector<javax.tools.JavaFileObject>();
        Path classes = Files.createDirectory(directory.resolve("classes"));
        try (var manager = compiler.getStandardFileManager(messages, null, StandardCharsets.UTF_8); var walk = Files.walk(directory.resolve("src"))) {
            var files = walk.filter(p -> p.toString().endsWith(".java")).sorted().map(Path::toFile).toList();
            boolean ok = compiler.getTask(null, manager, messages,
                List.of("--release", "17", "-Xlint:all", "-Werror", "-d", classes.toString()), null, manager.getJavaFileObjectsFromFiles(files)).call();
            assertTrue(ok, messages.getDiagnostics().toString()); assertTrue(messages.getDiagnostics().isEmpty(), messages.getDiagnostics().toString());
        }
        loader = new URLClassLoader(new java.net.URL[]{classes.toUri().toURL()}, ClassLoader.getPlatformClassLoader());
        String name = ir.grammarUnits().stream().filter(g -> g.id().equals(ir.entry().grammarId())).findFirst().orElseThrow().name();
        parser = loader.loadClass(pkg + "." + name + "Parser"); options = loader.loadClass(pkg + ".api.ParseOptions"); json = loader.loadClass(pkg + ".api.Json");
    }
    Object options(boolean ast, boolean full, boolean lexical, boolean memo, Map<String, Object> scanners) throws Exception {
        return options.getConstructor(boolean.class, boolean.class, boolean.class, boolean.class, Map.class).newInstance(ast, full, lexical, memo, scanners);
    }
    Object parse(String input) throws Exception { return parse(input, options.getField("DEFAULT").get(null)); }
    Object parse(String input, Object opts) throws Exception { return parser.getMethod("parse", CharSequence.class, options).invoke(null, input, opts); }
    Object canonical(Object result) throws Exception {
        return Json.read((String) json.getMethod("canonical", loader.loadClass(pkg + ".api.ParseResult")).invoke(null, result));
    }
    static Object get(Object value, String accessor) throws Exception { return value.getClass().getMethod(accessor).invoke(value); }
    static boolean ok(Object result) throws Exception { return (Boolean) get(result, "ok"); }
    @Override public void close() throws IOException {
        loader.close();
        try (var paths = Files.walk(directory)) { for (Path path : paths.sorted(Comparator.reverseOrder()).toList()) Files.delete(path); }
    }
}
