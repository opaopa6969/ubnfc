import java.lang.management.ManagementFactory;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Locale;

/**
 * Family C の旧 Java 実装: unlaxer-dsl の bootstrap {@code UBNFParsers}（combinator runtime）で
 * `.ubnf` を解析する。反射で呼ぶので unlaxer への compile-time 依存はない。
 *
 * <p>引数は {@code name=path} の並び。{@code -Dbench.memo=SAFE_FAILURES|OFF} で memo 方式を選ぶ。
 * この経路は CST（Parsed 木）を必ず作るので「認識のみ」は存在しない。AST への mapping は
 * bootstrap parser 側に無く、UBNF の AST 化は別コンポーネントなので測っていない。
 */
public final class UbnfLegacyBench {
    private static volatile Object sink;

    @SuppressWarnings({"unchecked", "rawtypes"})
    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        int warmup = Integer.getInteger("bench.warmup", 200);
        int runs = Integer.getInteger("bench.runs", 7);
        int iterations = Integer.getInteger("bench.iterations", 50);
        String memo = System.getProperty("bench.memo", "SAFE_FAILURES");

        Object root = Class.forName("org.unlaxer.dsl.bootstrap.UBNFParsers").getMethod("getRootParser").invoke(null);
        Class<?> parserClass = Class.forName("org.unlaxer.parser.Parser");
        Class<?> contextClass = Class.forName("org.unlaxer.context.ParseContext");
        Class<?> optionsClass = Class.forName("org.unlaxer.context.ParseOptions");
        Class<?> memoizationClass = Class.forName("org.unlaxer.context.Memoization");
        Class<?> sourceClass = Class.forName("org.unlaxer.StringSource");
        Object options = optionsClass.getMethod("withMemoization", memoizationClass)
            .invoke(null, Enum.valueOf((Class<Enum>) memoizationClass, memo));
        Method createRootSource = sourceClass.getMethod("createRootSource", String.class);
        Object noEffectors = java.lang.reflect.Array.newInstance(Class.forName("org.unlaxer.context.ParseContextEffector"), 0);
        Method withOptions = contextClass.getMethod("withOptions",
            Class.forName("org.unlaxer.Source"), optionsClass, noEffectors.getClass());
        Method parse = parserClass.getMethod("parse", contextClass);
        Method close = contextClass.getMethod("close");
        Method succeeded = Class.forName("org.unlaxer.Parsed").getMethod("isSucceeded");

        java.util.function.Function<String, Object> run = source -> {
            try {
                Object context = withOptions.invoke(null, createRootSource.invoke(null, source), options, noEffectors);
                try {
                    Object parsed = parse.invoke(root, context);
                    if (!Boolean.TRUE.equals(succeeded.invoke(parsed))) throw new IllegalStateException("UBNF parse failed");
                    return parsed;
                } finally { close.invoke(context); }
            } catch (InvocationTargetException failure) { throw new IllegalStateException(failure.getCause()); }
            catch (ReflectiveOperationException failure) { throw new IllegalStateException(failure); }
        };

        System.out.println("# java=" + System.getProperty("java.version") + " memo=" + memo);
        System.out.println("# jvm=" + ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("# root=" + root.getClass().getName()
            + " <- " + root.getClass().getProtectionDomain().getCodeSource().getLocation());
        System.out.println("family,impl,input,bytes,mode,iterations,runs,median_ms,min_ms,max_ms,us_per_byte,allocs_per_parse,alloc_bytes_per_parse");
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        long thread = Thread.currentThread().getId();
        for (String arg : args) {
            int split = arg.indexOf('=');
            String name = arg.substring(0, split);
            String source = Files.readString(Path.of(arg.substring(split + 1)));
            long first = System.nanoTime();
            try { sink = run.apply(source); }
            catch (RuntimeException failure) { System.out.println("# REJECTED " + name + " " + failure); continue; }
            System.err.println("first parse " + name + " " + (System.nanoTime() - first) / 1_000_000 + " ms");
            for (int i = 0; i < warmup; i++) sink = run.apply(source);
            double[] times = new double[runs]; long[] bytes = new long[runs];
            for (int r = 0; r < runs; r++) {
                long allocated = threads.getThreadAllocatedBytes(thread), start = System.nanoTime();
                for (int i = 0; i < iterations; i++) sink = run.apply(source);
                times[r] = (System.nanoTime() - start) / (1e6 * iterations);
                bytes[r] = (threads.getThreadAllocatedBytes(thread) - allocated) / iterations;
            }
            double[] sorted = times.clone(); Arrays.sort(sorted);
            long[] sortedBytes = bytes.clone(); Arrays.sort(sortedBytes);
            System.out.printf("C,unlaxer-dsl UBNFParsers(旧 Java/combinator),%s,%d,%s,%d,%d,%.6f,%.6f,%.6f,%.4f,,%d%n",
                name, source.getBytes(java.nio.charset.StandardCharsets.UTF_8).length, "CST(認識相当)",
                iterations, runs, sorted[runs / 2], sorted[0], sorted[runs - 1],
                sorted[runs / 2] * 1e3 / source.getBytes(java.nio.charset.StandardCharsets.UTF_8).length,
                sortedBytes[runs / 2]);
            System.out.flush();
        }
    }
}
