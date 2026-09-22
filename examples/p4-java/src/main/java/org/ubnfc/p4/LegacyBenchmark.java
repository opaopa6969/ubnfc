package org.ubnfc.p4;

import java.lang.management.ManagementFactory;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Locale;

/**
 * 旧生成物（tinyexpression の TinyExpressionP4Mapper / P4PreferredAstMapper）を反射で同じ計測法にかける任意ツール。
 * 引数: mapper|facade FIXTURE_DIR [fixture,...]。プロパティ: bench.warmup / bench.runs / bench.iterations / bench.deadlineSec。
 * 1 parse が deadline を超えた fixture は計測せず REJECTED として記録する。
 */
public final class LegacyBenchmark {
    private static volatile Object sink;
    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        String kind = args[0]; Path fixtures = Path.of(args[1]);
        String[] names = (args.length > 2 ? args[2] : "complex,complex-x4,complex-x16,complex-x64").split(",");
        int warmup = Integer.getInteger("bench.warmup", 200), runs = Integer.getInteger("bench.runs", 5), fixed = Integer.getInteger("bench.iterations", 0);
        long deadline = Integer.getInteger("bench.deadlineSec", 20) * 1_000_000_000L;
        Method parse;
        if (kind.equals("mapper")) parse = Class.forName("org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper").getMethod("parse", String.class);
        else parse = Class.forName("org.unlaxer.tinyexpression.p4.P4PreferredAstMapper").getMethod("parseDetailed", String.class);
        for (String name : new String[]{"org.unlaxer.parser.AbstractParser", "org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper"})
            System.out.println("# " + name + " <- " + Class.forName(name).getProtectionDomain().getCodeSource().getLocation());
        System.out.println("# java=" + System.getProperty("java.version") + " memoize=" + System.getProperty("tinyexpression.p4.memoize"));
        System.out.println("# jvm=" + ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("fixture,mode,entry,warmup,iterations,run1_ms,run2_ms,run3_ms,run4_ms,run5_ms,median_ms,alloc_bytes_per_parse");
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        for (String fixture : names) {
            String source = Files.readString(fixtures.resolve(fixture + ".tiny"));
            long first = System.nanoTime();
            try { sink = parse.invoke(null, source); }
            catch (InvocationTargetException failure) { System.out.println("# REJECTED " + fixture + " " + failure.getCause()); continue; }
            long firstMs = (System.nanoTime() - first) / 1_000_000;
            if (firstMs * 1_000_000L > deadline) { System.out.println("# REJECTED " + fixture + " first parse " + firstMs + " ms exceeds deadline"); continue; }
            int count = fixed > 0 ? fixed : fixture.endsWith("x64") ? 50 : fixture.endsWith("x16") ? 125 : fixture.endsWith("x4") ? 500 : 2000;
            long budget = System.nanoTime() + deadline * 6;
            int done = 0;
            for (; done < warmup && System.nanoTime() < budget; done++) sink = parse.invoke(null, source);
            System.err.println("warmup " + fixture + " " + done + "/" + warmup + " (first parse " + firstMs + " ms)");
            double[] times = new double[runs]; long[] bytes = new long[runs]; long thread = Thread.currentThread().getId();
            for (int r = 0; r < runs; r++) {
                long allocated = threads.getThreadAllocatedBytes(thread), start = System.nanoTime();
                for (int i = 0; i < count; i++) sink = parse.invoke(null, source);
                times[r] = (System.nanoTime() - start) / (1e6 * count);
                bytes[r] = (threads.getThreadAllocatedBytes(thread) - allocated) / count;
            }
            double[] sorted = times.clone(); Arrays.sort(sorted); long[] sortedBytes = bytes.clone(); Arrays.sort(sortedBytes);
            var line = new StringBuilder(fixture + ",legacy-" + kind + ",Formula," + done + "," + count);
            for (int r = 0; r < 5; r++) line.append(',').append(r < runs ? String.format("%.6f", times[r]) : "");
            System.out.println(line.append(',').append(String.format("%.6f", sorted[runs / 2])).append(',').append(sortedBytes[runs / 2]));
            System.out.flush();
        }
    }
}
