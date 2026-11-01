import java.nio.file.Files;
import java.nio.file.Path;
import jdk.jfr.Recording;
import org.unlaxer.ubnfc.examples.p4.P4Scanners;
import org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4Parser;

/** 暖機後の main thread だけを JFR の集計対象にする。 */
public final class Attribution {
    private static volatile Object sink;
    public static void main(String[] args) throws Exception {
        if (args.length != 6) throw new IllegalArgumentException("Attribution <p4|json> <recognize|ast> <file> <warmup> <seconds> <jfr>");
        boolean ast = switch (args[1]) { case "ast" -> true; case "recognize" -> false; default -> throw new IllegalArgumentException("不明な mode"); };
        String text = Files.readString(Path.of(args[2]));
        Runnable parse = switch (args[0]) {
            case "p4" -> () -> {
                var options = ast ? P4Scanners.options(true).withObservations(false, false) : P4Scanners.recognize();
                var r = TinyExpressionP4Parser.parse(text, options);
                if (!r.ok() || (ast && r.ast().isEmpty())) throw new IllegalStateException("受理失敗");
                sink = r;
            };
            case "json" -> () -> {
                var options = ast ? gen.json.api.ParseOptions.DEFAULT.withObservations(false, false) : gen.json.api.ParseOptions.RECOGNIZE;
                var r = gen.json.JsonParser.parse(text, options);
                if (!r.ok() || (ast && r.ast().isEmpty())) throw new IllegalStateException("受理失敗");
                sink = r;
            };
            default -> throw new IllegalArgumentException("不明な文法");
        };
        int warmup = Integer.parseInt(args[3]);
        double seconds = Double.parseDouble(args[4]);
        if (warmup <= 0 || seconds <= 0) throw new IllegalArgumentException("暖機と秒数は正数");
        for (int i = 0; i < warmup; i++) parse.run();
        try (var recording = new Recording(jdk.jfr.Configuration.getConfiguration("profile"))) {
            recording.start();
            long start = System.nanoTime();
            long count = 0;
            do { parse.run(); count++; } while ((System.nanoTime() - start) / 1e9 < seconds);
            long elapsed = System.nanoTime() - start;
            recording.stop();
            recording.dump(Path.of(args[5]));
            System.out.println("文法,mode,bytes,暖機,反復,ns/parse\n" + args[0] + "," + args[1] + "," + text.getBytes(java.nio.charset.StandardCharsets.UTF_8).length + "," + warmup + "," + count + "," + elapsed / count);
        }
    }
}
