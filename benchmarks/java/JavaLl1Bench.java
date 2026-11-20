// run-java-ll1-ab.sh から呼ぶ認識専用の計測。各 fork で option と UTF-8 入力を一度だけ作る。
import gen.json.JsonParser;
import java.lang.management.ManagementFactory;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import org.unlaxer.ubnfc.examples.p4.P4Scanners;
import org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4Parser;

public final class JavaLl1Bench {
    @FunctionalInterface
    private interface Recognize { void parse(); }

    // 同じ計測器を LL(1) API の追加前の生成物にもコンパイルする。反射は計測外に置く。
    @SuppressWarnings("unchecked")
    private static <T> T options(T options, boolean ll1) throws ReflectiveOperationException {
        return ll1 ? (T) options.getClass().getMethod("withLl1", boolean.class).invoke(options, true) : options;
    }

    private static Recognize recognizer(String task, String text, boolean ll1) throws ReflectiveOperationException {
        int length = text.codePointCount(0, text.length());
        if (task.equals("json")) {
            var options = options(gen.json.api.ParseOptions.RECOGNIZE, ll1);
            return () -> {
                var result = JsonParser.parse(text, options);
                check(result.ok(), result.consumedCp(), result.matchedCp(), length);
            };
        }
        var options = options(P4Scanners.recognize(), ll1);
        return () -> {
            var result = TinyExpressionP4Parser.parse(text, options);
            check(result.ok(), result.consumedCp(), result.matchedCp(), length);
        };
    }

    private static void check(boolean ok, int consumed, int matched, int length) {
        if (!ok || consumed != length || matched != length) throw new IllegalStateException("入力全体を受理できない");
    }

    public static void main(String[] args) throws Exception {
        String task = args[0], variant = args[1];
        Path input = Path.of(args[2]);
        long bytes = Files.size(input);
        var parser = recognizer(task, Files.readString(input), variant.equals("on"));
        int warmup = Integer.getInteger("bench.warmupIterations", 200);
        for (int i = 0; i < warmup; i++) parser.parse();
        long pilot = System.nanoTime();
        for (int i = 0; i < 10; i++) parser.parse();
        int iterations = (int) Math.clamp(2_000_000_000L / Math.max(1, System.nanoTime() - pilot), 5, 100_000);
        double[] ms = new double[5];
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        long tid = Thread.currentThread().threadId();
        long allocated = threads.getThreadAllocatedBytes(tid);
        for (int sample = 0; sample < ms.length; sample++) {
            long start = System.nanoTime();
            for (int i = 0; i < iterations; i++) parser.parse();
            ms[sample] = (System.nanoTime() - start) / 1e6 / iterations;
        }
        allocated = (threads.getThreadAllocatedBytes(tid) - allocated) / (iterations * ms.length);
        Arrays.sort(ms);
        double median = ms[ms.length / 2];
        System.out.printf(java.util.Locale.ROOT, "%s,%s,%d,%d,%d,%.6f,%.8f,%d%n",
            task, variant, bytes, warmup, iterations, median, median * 1e3 / bytes, allocated);
    }
}
