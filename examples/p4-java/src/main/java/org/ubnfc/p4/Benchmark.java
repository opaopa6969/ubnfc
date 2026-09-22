package org.ubnfc.p4;

import java.lang.management.ManagementFactory;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Locale;
import org.ubnfc.p4.generated.TinyExpressionP4Parser;
import org.ubnfc.p4.generated.api.ParseOptions;

/**
 * 1 スレッド、nanoTime、run の中央値。割り当て量は ThreadMXBean の thread allocated bytes。
 * システムプロパティ: bench.fixtures（カンマ区切り、既定は全 6 件）、bench.modes（recognize,ast）、
 * bench.warmup（既定 2000）、bench.runs（既定 5）、bench.iterations（既定は fixture ごとの規定値）、
 * bench.entry（既定 null=Formula。comparison-heavy 系は BooleanExpression を指定する）。
 */
public final class Benchmark {
    private static volatile Object sink;
    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        Path fixtures = Path.of(args[0]);
        String[] names = System.getProperty("bench.fixtures", "complex,complex-x4,complex-x16,complex-x64,comparison-heavy,comparison-heavy-x64").split(",");
        String[] modes = System.getProperty("bench.modes", "recognize,ast").split(",");
        int warmup = Integer.getInteger("bench.warmup", 2000);
        int runs = Integer.getInteger("bench.runs", 5);
        int fixedIterations = Integer.getInteger("bench.iterations", 0);
        String entry = System.getProperty("bench.entry");
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        System.out.println("# java=" + System.getProperty("java.version") + " vm=" + System.getProperty("java.vm.name"));
        System.out.println("# jvm=" + ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("fixture,mode,entry,warmup,iterations,run1_ms,run2_ms,run3_ms,run4_ms,run5_ms,median_ms,alloc_bytes_per_parse");
        for (String fixture : names) {
            String source = Files.readString(fixtures.resolve(fixture + ".tiny"));
            for (String mode : modes) {
                boolean ast = mode.equals("ast");
                ParseOptions options = P4Scanners.options(ast);
                var check = parse(entry, source, options);
                if (!check.ok() || check.consumed() != source.codePointCount(0, source.length()) || ast && check.ast().isEmpty()) {
                    System.out.println("# REJECTED " + fixture + " " + mode + " " + check.diagnostics());
                    continue;
                }
                int count = fixedIterations > 0 ? fixedIterations : fixture.endsWith("x64") ? 50 : fixture.endsWith("x16") ? 125 : fixture.endsWith("x4") ? 500 : 2000;
                System.err.println("warmup " + fixture + " " + mode);
                for (int i = 0; i < warmup; i++) {
                    sink = parse(entry, source, options);
                    if ((i + 1) % 250 == 0) System.err.println("warmup " + fixture + " " + mode + " " + (i + 1));
                }
                double[] times = new double[runs]; long[] bytes = new long[runs];
                long thread = Thread.currentThread().getId();
                for (int r = 0; r < runs; r++) {
                    long allocated = threads.getThreadAllocatedBytes(thread);
                    long start = System.nanoTime();
                    for (int i = 0; i < count; i++) sink = parse(entry, source, options);
                    times[r] = (System.nanoTime() - start) / (1e6 * count);
                    bytes[r] = (threads.getThreadAllocatedBytes(thread) - allocated) / count;
                }
                double[] sorted = times.clone(); Arrays.sort(sorted);
                long[] sortedBytes = bytes.clone(); Arrays.sort(sortedBytes);
                var line = new StringBuilder(fixture + "," + mode + "," + (entry == null ? "Formula" : entry) + "," + warmup + "," + count);
                for (int r = 0; r < 5; r++) line.append(',').append(r < runs ? String.format("%.6f", times[r]) : "");
                line.append(',').append(String.format("%.6f", sorted[runs / 2])).append(',').append(sortedBytes[runs / 2]);
                System.out.println(line);
                System.out.flush();
            }
        }
    }
    private static org.ubnfc.p4.generated.api.ParseResult<org.ubnfc.p4.generated.TinyExpressionP4AST> parse(String entry, String source, ParseOptions options) {
        return entry == null ? TinyExpressionP4Parser.parse(source, options) : TinyExpressionP4Parser.parseEntry("TinyExpressionP4", entry, source, options);
    }
}
