package target;

import java.util.Arrays;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfSystemProperty;
import target.api.*;
import target.operators.OperatorsParser;

/** -Dbench=true のときだけ実行。warmup/run 回数の短縮オプションは設けない。 */
@EnabledIfSystemProperty(named = "bench", matches = "true")
public class Bench {
    private static volatile Object sink;
    @Test void measure() {
        String[] names = {"1+2*3", "depth-8", "512-atoms"};
        String[] inputs = {"1+2*3", "(".repeat(8) + "1+2*3" + ")".repeat(8), "1+".repeat(511) + "1"};
        System.out.printf("Java %s; VM %s; OS %s/%s%n", System.getProperty("java.version"),
            System.getProperty("java.vm.name"), System.getProperty("os.name"), System.getProperty("os.arch"));
        for (int i = 0; i < inputs.length; i++) for (var options : new ParseOptions[]{ParseOptions.RECOGNIZE, ParseOptions.DEFAULT}) {
            String mode = options.buildAst() ? "ast" : "recognize";
            for (int n = 0; n < 20_000; n++) consume(inputs[i], options);
            double[] runs = new double[5];
            for (int run = 0; run < 5; run++) {
                long start = System.nanoTime();
                for (int n = 0; n < 200_000; n++) consume(inputs[i], options);
                runs[run] = (System.nanoTime() - start) / 200_000.0;
                System.out.printf("%s %s run=%d ns/op=%.1f%n", names[i], mode, run + 1, runs[run]);
            }
            System.out.printf("RUNS %s %s %s%n", names[i], mode, Arrays.toString(runs));
            Arrays.sort(runs);
            System.out.printf("MEDIAN %s %s %.1f ns/op%n", names[i], mode, runs[2]);
        }
    }
    private static void consume(String input, ParseOptions options) {
        ParseResult<?> result = OperatorsParser.parse(input, options);
        if (!result.ok() || result.consumedCp() != input.length()) throw new AssertionError(input);
        sink = result;
    }
}
