import java.nio.file.Path;
import java.util.List;
import jdk.jfr.consumer.RecordedFrame;
import jdk.jfr.consumer.RecordingFile;

public final class ReadProfile {
    private static String method(RecordedFrame frame) {
        return frame.getMethod().getType().getName() + "." + frame.getMethod().getName();
    }

    public static void main(String[] args) throws Exception {
        System.out.println("event\tweight\tleaf\tstack");
        try (var file = new RecordingFile(Path.of(args[0]))) {
            while (file.hasMoreEvents()) {
                var event = file.readEvent();
                switch (event.getEventType().getName()) {
                    case "jdk.ExecutionSample", "jdk.ObjectAllocationSample" -> {
                        boolean execution = event.getEventType().getName().equals("jdk.ExecutionSample");
                        var thread = execution ? event.getThread("sampledThread") : event.getThread();
                        if (thread == null || !"main".equals(thread.getJavaName()) || event.getStackTrace() == null) continue;
                        List<RecordedFrame> frames = event.getStackTrace().getFrames();
                        if (frames.isEmpty() || frames.stream().noneMatch(frame -> method(frame).contains("TinyExpressionP4Parser."))) continue;
                        var leaf = execution ? method(frames.get(0)) : event.getClass("objectClass").getName();
                        long weight = execution ? 1 : event.getLong("weight");
                        var stack = String.join(";", frames.stream().map(ReadProfile::method).toList());
                        System.out.printf("%s\t%d\t%s\t%s%n", execution ? "cpu" : "alloc", weight, leaf, stack);
                    }
                    case "jdk.GarbageCollection", "jdk.GCPhasePause" -> System.out.printf(
                        "%s\t%d\t%s%n", event.getEventType().getName(), event.getDuration().toNanos(), event.getString("name"));
                    default -> {}
                }
            }
        }
    }
}
