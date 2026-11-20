import java.nio.file.Path;
import java.util.Set;
import java.util.TreeMap;
import jdk.jfr.consumer.RecordedFrame;
import jdk.jfr.consumer.RecordingFile;

public final class JfrCounts {
    private static final Set<String> REPEATS = Set.of("scan6", "scan58", "scan97", "scan101", "scan112");
    private static final String PARSER = "gen.json.JsonParser";

    public static void main(String[] args) throws Exception {
        System.out.println("variant,event,scope,method,count");
        for (String name : new String[]{"scalar", "before", "candidate"}) {
            var counts = new TreeMap<String, Integer>();
            try (var recording = new RecordingFile(Path.of(args[0], name + ".jfr"))) {
                while (recording.hasMoreEvents()) {
                    var event = recording.readEvent();
                    String type = event.getEventType().getName();
                    if ((!type.equals("jdk.ExecutionSample") && !type.equals("jdk.NativeMethodSample"))
                            || !event.getThread("sampledThread").getJavaName().equals("main")
                            || event.getStackTrace() == null) continue;
                    var frames = event.getStackTrace().getFrames();
                    if (frames.stream().noneMatch(frame -> owner(frame).equals(PARSER)
                            && frame.getMethod().getName().equals("parse"))) continue;
                    String leaf = owner(frames.getFirst()) + "." + frames.getFirst().getMethod().getName();
                    counts.merge(type + ",self," + leaf, 1, Integer::sum);
                    frames.stream().filter(frame -> owner(frame).equals(PARSER)
                            && REPEATS.contains(frame.getMethod().getName()))
                        .findFirst().ifPresent(frame -> counts.merge(
                            type + ",inclusive," + frame.getMethod().getName(), 1, Integer::sum));
                }
            }
            counts.forEach((key, count) -> System.out.println(name + "," + key + "," + count));
        }
    }

    private static String owner(RecordedFrame frame) {
        return frame.getMethod().getType().getName();
    }
}
