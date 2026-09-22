package org.ubnfc.p4;

import java.lang.management.ManagementFactory;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Locale;

/**
 * 旧生成物（tinyexpression の生成 parser / Mapper / facade）を反射で同じ計測法にかける任意ツール。
 * 引数: root|root-deferred|mapper|facade FIXTURE_DIR [fixture,...]。プロパティ: bench.warmup / bench.runs / bench.iterations / bench.deadlineSec。
 * 1 parse が deadline を超えた fixture は計測せず REJECTED として記録する。
 *
 * <p>{@code root} は tinyexpression の JMH {@code P4ParserBenchmark.parseFreshToken(memoized)} と同じ
 * 「生成 root parser を {@code Memoization.SAFE_FAILURES} で 1 回走らせる」経路で、ubnfc の認識モードに対応する。
 * {@code root-deferred} は加えて {@code Diagnostics.DETAILED_ON_FAILURE}（成功経路で失敗記録を作らない）。
 * {@code mapper} は AST 生成まで、{@code facade} は公開入口（候補選択を含む）。
 */
public final class LegacyBenchmark {
    private static volatile Object sink;

    private static java.util.function.Function<String, Object> reflective(Method method) {
        return source -> {
            try { return method.invoke(null, source); }
            catch (InvocationTargetException failure) { throw new IllegalStateException(failure.getCause()); }
            catch (IllegalAccessException failure) { throw new IllegalStateException(failure); }
        };
    }

    /** JMH P4ParserBenchmark.parseFreshToken(true) と同じ経路（root parser + SAFE_FAILURES）。 */
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static java.util.function.Function<String, Object> rootParser(boolean deferred) throws Exception {
        Object root = Class.forName("org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Parsers").getMethod("getRootParser").invoke(null);
        Class<?> parserClass = Class.forName("org.unlaxer.parser.Parser");
        Class<?> contextClass = Class.forName("org.unlaxer.context.ParseContext");
        Class<?> optionsClass = Class.forName("org.unlaxer.context.ParseOptions");
        Class<?> memoizationClass = Class.forName("org.unlaxer.context.Memoization");
        Class<?> sourceClass = Class.forName("org.unlaxer.StringSource");
        Object safeFailures = Enum.valueOf((Class<Enum>) memoizationClass, "SAFE_FAILURES");
        Object memoized = optionsClass.getMethod("withMemoization", memoizationClass).invoke(null, safeFailures);
        if (deferred) {
            Class<?> diagnosticsClass = Class.forName("org.unlaxer.context.ParseOptions$Diagnostics");
            memoized = optionsClass.getMethod("withDiagnostics", diagnosticsClass)
                .invoke(memoized, Enum.valueOf((Class<Enum>) diagnosticsClass, "DETAILED_ON_FAILURE"));
        }
        final Object options = memoized;
        Method createRootSource = sourceClass.getMethod("createRootSource", String.class);
        Class<?> effectorArray = java.lang.reflect.Array.newInstance(Class.forName("org.unlaxer.context.ParseContextEffector"), 0).getClass();
        Object noEffectors = java.lang.reflect.Array.newInstance(Class.forName("org.unlaxer.context.ParseContextEffector"), 0);
        Method withOptions = contextClass.getMethod("withOptions", Class.forName("org.unlaxer.Source"), optionsClass, effectorArray);
        Method parse = parserClass.getMethod("parse", contextClass);
        Method close = contextClass.getMethod("close");
        Method succeeded = Class.forName("org.unlaxer.Parsed").getMethod("isSucceeded");
        return source -> {
            try {
                Object context = withOptions.invoke(null, createRootSource.invoke(null, source), options, noEffectors);
                try {
                    Object parsed = parse.invoke(root, context);
                    if (!Boolean.TRUE.equals(succeeded.invoke(parsed))) throw new IllegalStateException("P4 parse failed");
                    return parsed;
                } finally { close.invoke(context); }
            } catch (InvocationTargetException failure) { throw new IllegalStateException(failure.getCause()); }
            catch (ReflectiveOperationException failure) { throw new IllegalStateException(failure); }
        };
    }
    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        String kind = args[0]; Path fixtures = Path.of(args[1]);
        String[] names = (args.length > 2 ? args[2] : "complex,complex-x4,complex-x16,complex-x64").split(",");
        int warmup = Integer.getInteger("bench.warmup", 200), runs = Integer.getInteger("bench.runs", 5), fixed = Integer.getInteger("bench.iterations", 0);
        long deadline = Integer.getInteger("bench.deadlineSec", 20) * 1_000_000_000L;
        java.util.function.Function<String, Object> parse;
        if (kind.equals("root") || kind.equals("root-deferred")) parse = rootParser(kind.endsWith("deferred"));
        else if (kind.equals("mapper")) parse = reflective(Class.forName("org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper").getMethod("parse", String.class));
        else parse = reflective(Class.forName("org.unlaxer.tinyexpression.p4.P4PreferredAstMapper").getMethod("parseDetailed", String.class));
        for (String name : new String[]{"org.unlaxer.parser.AbstractParser", "org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper"})
            System.out.println("# " + name + " <- " + Class.forName(name).getProtectionDomain().getCodeSource().getLocation());
        System.out.println("# java=" + System.getProperty("java.version") + " memoize=" + System.getProperty("tinyexpression.p4.memoize"));
        System.out.println("# jvm=" + ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("fixture,mode,entry,warmup,iterations,run1_ms,run2_ms,run3_ms,run4_ms,run5_ms,median_ms,alloc_bytes_per_parse");
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        for (String fixture : names) {
            String source = Files.readString(fixtures.resolve(fixture + ".tiny"));
            long first = System.nanoTime();
            try { sink = parse.apply(source); }
            catch (RuntimeException failure) { System.out.println("# REJECTED " + fixture + " " + failure); continue; }
            long firstMs = (System.nanoTime() - first) / 1_000_000;
            if (firstMs * 1_000_000L > deadline) { System.out.println("# REJECTED " + fixture + " first parse " + firstMs + " ms exceeds deadline"); continue; }
            int count = fixed > 0 ? fixed : fixture.endsWith("x64") ? 50 : fixture.endsWith("x16") ? 125 : fixture.endsWith("x4") ? 500 : 2000;
            long budget = System.nanoTime() + deadline * 6;
            int done = 0;
            for (; done < warmup && System.nanoTime() < budget; done++) sink = parse.apply(source);
            System.err.println("warmup " + fixture + " " + done + "/" + warmup + " (first parse " + firstMs + " ms)");
            double[] times = new double[runs]; long[] bytes = new long[runs]; long thread = Thread.currentThread().getId();
            for (int r = 0; r < runs; r++) {
                long allocated = threads.getThreadAllocatedBytes(thread), start = System.nanoTime();
                for (int i = 0; i < count; i++) sink = parse.apply(source);
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
