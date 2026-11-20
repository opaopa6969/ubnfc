import java.lang.management.ManagementFactory;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.Locale;
import java.util.function.ToIntFunction;
import jdk.jfr.Recording;
import gen.json.JsonParser;
import gen.operators.OperatorsParser;
import org.unlaxer.ubnfc.examples.p4.P4Scanners;
import org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4Parser;

public final class ShortScanBench {
    private record Task(String name, Path file, int iterations, ToIntFunction<String> parse) {}

    private static final int WARMUP = 200;
    private static final int SAMPLES = 3;
    private static volatile int sink;

    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        Path inputs = Path.of(args[0]);
        Path fixtures = Path.of(args[1]);
        Task json = new Task("json-1m", inputs.resolve("json-1m.json"), 32, input -> {
            var result = JsonParser.parse(input, gen.json.api.ParseOptions.RECOGNIZE);
            return result.ok() ? result.consumedCp() : -1;
        });
        if (args.length == 3 && args[2].equals("json")) {
            time(json);
            return;
        }
        if (args.length == 3) {
            profile(json, Path.of(args[2]));
            return;
        }
        time(json);
        time(new Task("operators-atoms-512", inputs.resolve("operators-atoms-512.txt"), 2000, input -> {
            var result = OperatorsParser.parse(input, gen.operators.api.ParseOptions.RECOGNIZE);
            return result.ok() ? result.consumedCp() : -1;
        }));
        var options = P4Scanners.recognize();
        String[] names = {"complex", "complex-x4", "complex-x16", "complex-x64",
                          "comparison-heavy", "comparison-heavy-x64"};
        int[] iterations = {2000, 500, 125, 50, 2000, 50};
        for (int i = 0; i < names.length; i++) {
            boolean comparison = names[i].startsWith("comparison-heavy");
            time(new Task(names[i], fixtures.resolve(names[i] + ".tiny"), iterations[i], input -> {
                var result = comparison
                    ? TinyExpressionP4Parser.parseEntry("TinyExpressionP4", "BooleanExpression", input, options)
                    : TinyExpressionP4Parser.parse(input, options);
                return result.ok() ? result.consumedCp() : -1;
            }));
        }
    }

    private static String warm(Task task) throws Exception {
        String input = Files.readString(task.file());
        if (task.parse().applyAsInt(input) != input.codePointCount(0, input.length())) {
            throw new IllegalStateException("入力全体を受理できない " + task.file());
        }
        run(task, input, WARMUP);
        return input;
    }

    private static void run(Task task, String input, int iterations) {
        for (int i = 0; i < iterations; i++) sink = task.parse().applyAsInt(input);
    }

    private static void time(Task task) throws Exception {
        String input = warm(task);
        var thread = ManagementFactory.getThreadMXBean();
        for (int sample = 1; sample <= SAMPLES; sample++) {
            long cpu = thread.getCurrentThreadCpuTime();
            long wall = System.nanoTime();
            run(task, input, task.iterations());
            long elapsed = System.nanoTime() - wall;
            long consumed = thread.getCurrentThreadCpuTime() - cpu;
            System.out.printf("%s,%d,%d,%d,%.6f,%.6f%n", task.name(), sample, WARMUP,
                task.iterations(), elapsed / (1e6 * task.iterations()), consumed / (1e6 * task.iterations()));
        }
    }

    private static void profile(Task json, Path destination) throws Exception {
        String input = warm(json);
        try (Recording recording = new Recording()) {
            recording.enable("jdk.ExecutionSample").withPeriod(Duration.ofMillis(2)).withStackTrace();
            recording.enable("jdk.NativeMethodSample").withPeriod(Duration.ofMillis(2)).withStackTrace();
            recording.start();
            long stop = System.nanoTime() + Duration.ofSeconds(20).toNanos();
            while (System.nanoTime() < stop) run(json, input, json.iterations());
            recording.stop();
            recording.dump(destination);
        }
    }
}
