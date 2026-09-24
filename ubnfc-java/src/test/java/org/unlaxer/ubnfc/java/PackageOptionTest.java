package org.unlaxer.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.List;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;

/**
 * issue #82: the generated package. Order of precedence is {@code --package} &gt; the grammar's own package &gt;
 * {@code org.unlaxer.ubnfc.generated.<grammar>}. The default lives under {@code org.unlaxer}, the Maven namespace
 * the owner holds; {@code --package} lets a consumer (e.g. tinyexpression) vendor the parser under its own package.
 */
class PackageOptionTest {
    private static final Path TINY = Path.of("../ir/examples/tinycalc.ir.json");

    private static List<String> packageLines(Path root) throws Exception {
        try (Stream<Path> files = Files.walk(root)) {
            return files.filter(p -> p.toString().endsWith(".java")).sorted().map(p -> {
                try {
                    String pkg = Files.readAllLines(p).stream().filter(l -> l.startsWith("package ")).findFirst().orElse("<none>");
                    return pkg + " @" + root.relativize(p).toString().replace('\\', '/');
                }
                catch (java.io.IOException e) { throw new java.io.UncheckedIOException(e); }
            }).toList();
        }
    }

    private static void delete(Path root) throws Exception {
        try (Stream<Path> files = Files.walk(root)) {
            for (Path p : files.sorted(Comparator.reverseOrder()).toList()) Files.delete(p);
        }
    }

    private static Path generate(Path ir, String... extra) throws Exception {
        Path out = Files.createTempDirectory("ubnfc-pkg");
        var args = new java.util.ArrayList<>(List.of("--ir", ir.toString(), "--out", out.toString()));
        args.addAll(List.of(extra));
        Main.main(args.toArray(String[]::new));
        return out;
    }

    @Test void defaultPackageIsUnderOrgUnlaxerWhenGrammarDeclaresNone() throws Exception {
        Path dir = Files.createTempDirectory("ubnfc-ir");
        Path ir = dir.resolve("tinycalc-nopackage.ir.json");
        String json = Files.readString(TINY);
        String stripped = json.replace("\"package\": \"org.unlaxer.tinycalc.generated\"", "\"package\": null");
        assertNotEquals(json, stripped, "fixture must declare a package so that this test really strips it");
        Files.writeString(ir, stripped);
        Path out = generate(ir);
        try {
            assertEquals("org.unlaxer.ubnfc.generated.", org.unlaxer.ubnfc.java.emit.JavaEmitter.DEFAULT_PACKAGE_PREFIX);
            Path root = out.resolve("org/unlaxer/ubnfc/generated/tinycalc");
            assertTrue(Files.isRegularFile(root.resolve("TinyCalcParser.java")), "parser under the default package");
            assertTrue(Files.isRegularFile(root.resolve("api/Span.java")));
            try (Stream<Path> top = Files.list(out)) {
                assertEquals(List.of("org"), top.map(p -> p.getFileName().toString()).toList(), "nothing outside org/unlaxer");
            }
            for (String line : packageLines(out))
                assertTrue(line.startsWith("package org.unlaxer.ubnfc.generated.tinycalc"), line);
            assertEquals("package org.unlaxer.ubnfc.generated.tinycalc;",
                Files.readAllLines(root.resolve("TinyCalcParser.java")).get(0));
            assertTrue(Files.readAllLines(root.resolve("api/Span.java")).contains("package org.unlaxer.ubnfc.generated.tinycalc.api;"));
        } finally { delete(out); delete(dir); }
    }

    @Test void grammarPackageWinsOverDefault() throws Exception {
        Path out = generate(TINY, "--no-emit-driver");
        try {
            assertTrue(Files.isRegularFile(out.resolve("org/unlaxer/tinycalc/generated/TinyCalcParser.java")));
        } finally { delete(out); }
    }

    @Test void explicitPackageWinsOverEverything() throws Exception {
        Path out = generate(TINY, "--package", "com.example.vendored.tiny", "--no-emit-driver");
        try {
            Path parser = out.resolve("com/example/vendored/tiny/TinyCalcParser.java");
            assertEquals("package com.example.vendored.tiny;", Files.readAllLines(parser).get(0));
            assertTrue(Files.isRegularFile(out.resolve("com/example/vendored/tiny/rt/Cursor.java")), "runtime follows --package");
            for (String line : packageLines(out))
                assertTrue(line.startsWith("package com.example.vendored.tiny"), line);
        } finally { delete(out); }
    }
}
