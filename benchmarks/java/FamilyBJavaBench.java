import java.lang.management.ManagementFactory;
import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.Locale;

/**
 * Family B（Operators / Power）の Java 実装 2 本を 1 JVM で交互に測る。
 *
 * <p>生成側は ubnfc の Java backend が同じ IR から出したもの（package gen.operators / gen.power）、
 * 手書き側は examples/handwritten-target/java（package target）。生成クラスは反射で呼ぶので、
 * このドライバ自体は生成コードに compile-time 依存しない。
 *
 * <p>計時は nanoTime、単一 thread。warmup と反復数と run 数はシステムプロパティで指定する。
 * 割当は ThreadMXBean の thread allocated bytes（計時 run と同じループで採る）。
 */
public final class FamilyBJavaBench {
    private static volatile Object sink;

    private interface Parse { Object apply(String source, boolean ast); }

    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        int runs = Integer.getInteger("bench.runs", 7);
        // 1 run の長さを揃える（既定 1 秒）。JIT warmup は反復数の半分、ただし 20,000 回以上 200,000 回以下。
        double runSeconds = Double.parseDouble(System.getProperty("bench.runSeconds", "1.0"));
        int fixedIterations = Integer.getInteger("bench.iterations", 0);

        String[][] operators = {
            {"1+2*3", "1+2*3"},
            {"parentheses-8", "(".repeat(8) + "1+2*3" + ")".repeat(8)},
            {"atoms-512", "1+".repeat(511) + "1"},
        };
        String[][] powers = {
            {"2^3^2", "2^3^2"},
            {"power-64", "2^".repeat(63) + "2"},
        };

        System.out.println("# java=" + System.getProperty("java.version") + " vm=" + System.getProperty("java.vm.name"));
        System.out.println("# jvm=" + ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("family,impl,input,bytes,mode,iterations,runs,median_ms,min_ms,max_ms,us_per_byte,allocs_per_parse,alloc_bytes_per_parse");

        Parse genOperators = generated("gen.operators.OperatorsParser", "gen.operators.api.ParseOptions");
        Parse genPower = generated("gen.power.PowerParser", "gen.power.api.ParseOptions");
        Parse handwrittenOperators = handwritten("target.operators.OperatorsParser");
        Parse handwrittenPower = handwritten("target.power.PowerParser");

        for (String[] input : operators) {
            measure("B/Operators", "ubnfc-java(生成)", input[0], input[1], genOperators, runSeconds, runs, fixedIterations);
            measure("B/Operators", "手書き Java(目標形)", input[0], input[1], handwrittenOperators, runSeconds, runs, fixedIterations);
        }
        for (String[] input : powers) {
            measure("B/Power", "ubnfc-java(生成)", input[0], input[1], genPower, runSeconds, runs, fixedIterations);
            measure("B/Power", "手書き Java(目標形)", input[0], input[1], handwrittenPower, runSeconds, runs, fixedIterations);
        }
    }

    /** 生成 parser: parse(CharSequence, ParseOptions)。ParseOptions(buildAst, requireFullInput)。 */
    private static Parse generated(String parserClass, String optionsClass) throws Exception {
        Class<?> options = Class.forName(optionsClass);
        Method parse = Class.forName(parserClass).getMethod("parse", CharSequence.class, options);
        Object ast = options.getConstructor(boolean.class, boolean.class).newInstance(true, true);
        Object recognize = options.getConstructor(boolean.class, boolean.class).newInstance(false, true);
        return (source, build) -> invoke(parse, source, build ? ast : recognize);
    }

    private static Parse handwritten(String parserClass) throws Exception {
        Class<?> options = Class.forName("target.api.ParseOptions");
        Method parse = Class.forName(parserClass).getMethod("parse", CharSequence.class, options);
        Object ast = options.getField("DEFAULT").get(null);
        Object recognize = options.getField("RECOGNIZE").get(null);
        return (source, build) -> invoke(parse, source, build ? ast : recognize);
    }

    private static Object invoke(Method parse, String source, Object options) {
        try { return parse.invoke(null, source, options); }
        catch (ReflectiveOperationException failure) { throw new IllegalStateException(failure); }
    }

    private static void measure(String family, String name, String input, String source, Parse parse,
                                double runSeconds, int runs, int fixedIterations) throws Exception {
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        long thread = Thread.currentThread().getId();
        for (boolean ast : new boolean[] {false, true}) {
            Object check = parse.apply(source, ast);
            boolean ok = (Boolean) check.getClass().getMethod("ok").invoke(check);
            if (!ok) { System.out.println("# REJECTED " + name + " " + input + " ast=" + ast); continue; }
            // 反復数は「JIT が効いたあとの 1 parse」から決める（冷えた初回で決めない）。
            for (int i = 0; i < 20000; i++) sink = parse.apply(source, ast);
            long probe = System.nanoTime();
            for (int i = 0; i < 200; i++) sink = parse.apply(source, ast);
            double per = Math.max((System.nanoTime() - probe) / 200.0, 1.0);
            int iterations = fixedIterations > 0 ? fixedIterations
                : (int) Math.max(50, Math.min(500_000, runSeconds * 1e9 / per));
            int warmup = Math.max(20000, Math.min(200_000, iterations / 2));
            System.err.println("warmup " + name + " " + input + " ast=" + ast + " iterations=" + iterations + " warmup=" + warmup);
            for (int i = 0; i < warmup; i++) sink = parse.apply(source, ast);
            double[] times = new double[runs];
            long[] bytes = new long[runs];
            for (int r = 0; r < runs; r++) {
                long allocated = threads.getThreadAllocatedBytes(thread), start = System.nanoTime();
                for (int i = 0; i < iterations; i++) sink = parse.apply(source, ast);
                times[r] = (System.nanoTime() - start) / (1e6 * iterations);
                bytes[r] = (threads.getThreadAllocatedBytes(thread) - allocated) / iterations;
            }
            double[] sorted = times.clone(); Arrays.sort(sorted);
            long[] sortedBytes = bytes.clone(); Arrays.sort(sortedBytes);
            System.out.printf("%s,%s,%s,%d,%s,%d,%d,%.6f,%.6f,%.6f,%.4f,,%d%n",
                family, name, input, source.length(), ast ? "AST" : "認識", iterations, runs,
                sorted[runs / 2], sorted[0], sorted[runs - 1],
                sorted[runs / 2] * 1e3 / source.length(), sortedBytes[runs / 2]);
            System.out.flush();
        }
    }
}
