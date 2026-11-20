import java.lang.management.ManagementFactory;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import jdk.jfr.Recording;
import org.unlaxer.ubnfc.examples.p4.P4Scanners;
import org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4Parser;

public final class ProfileP4 {
    private static volatile Object sink;

    public static void main(String[] args) throws Exception {
        var text = Files.readString(Path.of(args[0]));
        var options = P4Scanners.options(Boolean.parseBoolean(args[1])).withObservations(false, false);
        if (!TinyExpressionP4Parser.parse(text, options).ok()) throw new AssertionError();
        for (int i = 0; i < 1000; i++) sink = TinyExpressionP4Parser.parse(text, options);
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        long thread = Thread.currentThread().getId();
        try (var recording = new Recording()) {
            recording.enable("jdk.ExecutionSample").withPeriod(Duration.ofMillis(2));
            recording.enable("jdk.ObjectAllocationSample").withStackTrace();
            recording.enable("jdk.GarbageCollection");
            recording.enable("jdk.GCPhasePause");
            recording.start();
            long bytes = threads.getThreadAllocatedBytes(thread);
            long cpu = threads.getCurrentThreadCpuTime();
            long start = System.nanoTime();
            int parses = 0;
            while (System.nanoTime() - start < 8_000_000_000L) {
                sink = TinyExpressionP4Parser.parse(text, options);
                parses++;
            }
            long end = System.nanoTime(), cpuEnd = threads.getCurrentThreadCpuTime();
            long bytesEnd = threads.getThreadAllocatedBytes(thread);
            recording.stop();
            System.out.printf("%d %.6f %.6f %d%n", parses, (end - start) / 1e6,
                (cpuEnd - cpu) / 1e6, (bytesEnd - bytes) / parses);
            recording.dump(Path.of(args[2]));
        }
    }
}
