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
 * 引数: root|root-off|root-deferred|mapper|mapper-off|facade FIXTURE_DIR [fixture,...]。プロパティ: bench.warmup / bench.runs / bench.iterations / bench.deadlineSec。
 * 1 parse が deadline を超えた fixture は計測せず REJECTED として記録する。
 *
 * <p>{@code root} は tinyexpression の JMH {@code P4ParserBenchmark.parseFreshToken(memoized)} と同じ
 * 「生成 root parser を {@code Memoization.SAFE_FAILURES} で 1 回走らせる」経路で、ubnfc の認識モードに対応する。
 * {@code root-deferred} は加えて {@code Diagnostics.DETAILED_ON_FAILURE}（成功経路で失敗記録を作らない）。
 * {@code mapper} は AST 生成まで、{@code facade} は公開入口（候補選択を含む）。
 */
public final class LegacyBenchmark {
    private static volatile Object sink;

    /** 実行時にどの jar / ディレクトリから読まれたかを記録する主要クラス。 */
    private static final String[] PROBED_CLASSES = {
        "org.unlaxer.parser.AbstractParser",
        "org.unlaxer.parser.Parser",
        "org.unlaxer.parser.ParentHolderParser",
        "org.unlaxer.parser.combinator.Chain",
        "org.unlaxer.parser.combinator.Choice",
        "org.unlaxer.parser.combinator.Occurs",
        "org.unlaxer.context.ParseContext",
        "org.unlaxer.context.PackratMemoTable",
        "org.unlaxer.context.Memoization",
        "org.unlaxer.context.SafeFailureMemoizable",
        "org.unlaxer.context.SafeSuccessMemoizable",
        "org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Parsers",
        "org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper",
        "org.unlaxer.tinyexpression.p4.P4PreferredAstMapper",
    };

    private static java.util.function.Function<String, Object> reflective(Method method) {
        return source -> {
            try { return method.invoke(null, source); }
            catch (InvocationTargetException failure) { throw new IllegalStateException(failure.getCause()); }
            catch (IllegalAccessException failure) { throw new IllegalStateException(failure); }
        };
    }

    /** JMH P4ParserBenchmark.parseFreshToken(true) と同じ経路（root parser + SAFE_FAILURES）。 */
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static java.util.function.Function<String, Object> rootParser(boolean deferred, boolean memoized) throws Exception {
        Object root = Class.forName("org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Parsers").getMethod("getRootParser").invoke(null);
        Class<?> parserClass = Class.forName("org.unlaxer.parser.Parser");
        Class<?> contextClass = Class.forName("org.unlaxer.context.ParseContext");
        Class<?> optionsClass = Class.forName("org.unlaxer.context.ParseOptions");
        Class<?> memoizationClass = Class.forName("org.unlaxer.context.Memoization");
        Class<?> sourceClass = Class.forName("org.unlaxer.StringSource");
        Object policy = Enum.valueOf((Class<Enum>) memoizationClass, memoized ? "SAFE_FAILURES" : "OFF");
        Object options0 = optionsClass.getMethod("withMemoization", memoizationClass).invoke(null, policy);
        if (deferred) {
            Class<?> diagnosticsClass = Class.forName("org.unlaxer.context.ParseOptions$Diagnostics");
            options0 = optionsClass.getMethod("withDiagnostics", diagnosticsClass)
                .invoke(options0, Enum.valueOf((Class<Enum>) diagnosticsClass, "DETAILED_ON_FAILURE"));
        }
        final Object options = options0;
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
    /**
     * 生成 Mapper を {@code SAFE_FAILURES} で走らせる（JMH {@code parseAndMapSafe} 相当）。
     * 引数 1 つの {@code parse(String)} は {@code ParseOptions.DEFAULT}＝memo OFF なので、
     * AST モードの比較には使えない（それは {@code mapper-off}）。
     */
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static java.util.function.Function<String, Object> memoizedMapper() throws Exception {
        Class<?> optionsClass = Class.forName("org.unlaxer.context.ParseOptions");
        Class<?> memoizationClass = Class.forName("org.unlaxer.context.Memoization");
        Object options = optionsClass.getMethod("withMemoization", memoizationClass)
            .invoke(null, Enum.valueOf((Class<Enum>) memoizationClass, "SAFE_FAILURES"));
        Method parse = Class.forName("org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper")
            .getMethod("parseWithOptions", String.class, optionsClass);
        return source -> {
            try { return parse.invoke(null, source, options); }
            catch (InvocationTargetException failure) { throw new IllegalStateException(failure.getCause()); }
            catch (IllegalAccessException failure) { throw new IllegalStateException(failure); }
        };
    }

    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        String kind = args[0]; Path fixtures = Path.of(args[1]);
        String[] names = (args.length > 2 ? args[2] : "complex,complex-x4,complex-x16,complex-x64").split(",");
        int warmup = Integer.getInteger("bench.warmup", 200), runs = Integer.getInteger("bench.runs", 5), fixed = Integer.getInteger("bench.iterations", 0);
        long deadline = Integer.getInteger("bench.deadlineSec", 20) * 1_000_000_000L;
        java.util.function.Function<String, Object> parse;
        if (kind.startsWith("root")) parse = rootParser(kind.endsWith("deferred"), !kind.equals("root-off"));
        else if (kind.equals("mapper-off")) parse = reflective(Class.forName("org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper").getMethod("parse", String.class));
        else if (kind.equals("mapper")) parse = memoizedMapper();
        else parse = reflective(Class.forName("org.unlaxer.tinyexpression.p4.P4PreferredAstMapper").getMethod("parseDetailed", String.class));
        for (String name : PROBED_CLASSES) {
            String where;
            try {
                var source = Class.forName(name).getProtectionDomain().getCodeSource();
                where = source == null ? "<no code source>" : source.getLocation().toString();
            } catch (Throwable missing) { where = "ABSENT (" + missing.getClass().getSimpleName() + ")"; }
            System.out.println("# " + name + " <- " + where);
        }
        try {
            Object rootProbe = Class.forName("org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Parsers").getMethod("getRootParser").invoke(null);
            Class<?> safeSuccess = Class.forName("org.unlaxer.context.SafeSuccessMemoizable");
            Class<?> safeFailure = Class.forName("org.unlaxer.context.SafeFailureMemoizable");
            System.out.println("# root=" + rootProbe.getClass().getName()
                + " safeSuccess=" + safeSuccess.isInstance(rootProbe) + " safeFailure=" + safeFailure.isInstance(rootProbe));
        } catch (Throwable missing) { System.out.println("# marker probe unavailable: " + missing); }
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
