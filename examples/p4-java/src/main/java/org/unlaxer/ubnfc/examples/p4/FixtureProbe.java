package org.unlaxer.ubnfc.examples.p4;

import java.nio.file.Files;
import java.nio.file.Path;
import org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4Parser;
import org.unlaxer.ubnfc.examples.p4.generated.api.Json;

/** 調査用。golden 更新は出力先を明示した場合のみ行う。 */
public final class FixtureProbe {
    public static void main(String[] args) throws Exception {
        Path input = Path.of(args[0]);
        Path output = args.length > 1 ? Path.of(args[1]) : null;
        String entry = args.length > 2 ? args[2] : "Formula";
        if (output != null) Files.createDirectories(output);
        try (var paths = Files.list(input)) {
            for (Path path : paths.filter(p -> p.toString().endsWith(".tiny")).sorted().toList()) {
                String source = Files.readString(path);
                long start = System.nanoTime();
                try {
                    var result = TinyExpressionP4Parser.parseEntry("TinyExpressionP4", entry, source, P4Scanners.options(true));
                    System.out.printf("%s\tok=%s\tconsumed=%d/%d\tast=%s\tms=%.3f%s%n",
                        path.getFileName(), result.ok(), result.consumed(), source.codePointCount(0, source.length()),
                        result.ast().map(a -> a.getClass().getSimpleName()).orElse("null"),
                        (System.nanoTime() - start) / 1e6, result.ok() ? "" : "\t" + result.diagnostics());
                    if (output != null && result.ok())
                        Files.writeString(output.resolve(path.getFileName() + ".json"), Json.canonical(result) + "\n");
                } catch (RuntimeException | StackOverflowError error) {
                    System.out.println(path.getFileName() + "\t" + error);
                    error.printStackTrace(System.err);
                }
            }
        }
    }
}
