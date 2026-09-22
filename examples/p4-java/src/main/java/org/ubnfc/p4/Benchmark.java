package org.ubnfc.p4;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Locale;
import org.ubnfc.p4.generated.TinyExpressionP4Parser;

/** 1 スレッド、nanoTime、入力ごとに warmup 2,000 回、5 run の中央値。 */
public final class Benchmark {
    private static volatile Object sink;
    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        Path fixtures = Path.of(args[0]);
        System.out.println("# java=" + System.getProperty("java.version") + " vm=" + System.getProperty("java.vm.name"));
        System.out.println("# jvm=" + java.lang.management.ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("fixture,mode,warmup,iterations,run1_ms,run2_ms,run3_ms,run4_ms,run5_ms,median_ms");
        for (String fixture : new String[]{"complex", "complex-x4", "complex-x16", "complex-x64", "comparison-heavy", "comparison-heavy-x64"}) {
            String source = Files.readString(fixtures.resolve(fixture + ".tiny"));
            for (boolean ast : new boolean[]{false, true}) {
                String mode = ast ? "ast" : "recognize";
                var options = P4Scanners.options(ast);
                var check = TinyExpressionP4Parser.parse(source, options);
                if (!check.ok() || check.consumed() != source.codePointCount(0, source.length()) || ast && check.ast().isEmpty()) {
                    System.out.println("# REJECTED " + fixture + " " + mode + " " + check.diagnostics());
                    continue;
                }
                int count = fixture.endsWith("x64") ? 50 : fixture.endsWith("x16") ? 125 : fixture.endsWith("x4") ? 500 : 2000;
                System.err.println("warmup " + fixture + " " + mode);
                for (int i = 0; i < 2000; i++) {
                    sink = TinyExpressionP4Parser.parse(source, options);
                    if ((i + 1) % 250 == 0) System.err.println("warmup " + fixture + " " + mode + " " + (i + 1));
                }
                double[] runs = new double[5];
                for (int r = 0; r < runs.length; r++) {
                    long start = System.nanoTime();
                    for (int i = 0; i < count; i++) sink = TinyExpressionP4Parser.parse(source, options);
                    runs[r] = (System.nanoTime() - start) / (1e6 * count);
                }
                double[] sorted = runs.clone(); Arrays.sort(sorted);
                System.out.printf("%s,%s,2000,%d,%.6f,%.6f,%.6f,%.6f,%.6f,%.6f%n",
                    fixture, mode, count, runs[0], runs[1], runs[2], runs[3], runs[4], sorted[2]);
                System.out.flush();
            }
        }
    }
}
