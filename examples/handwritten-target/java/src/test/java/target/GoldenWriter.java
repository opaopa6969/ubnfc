package target;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import target.api.*;
import target.operators.OperatorsParser;
import target.power.PowerParser;

/** 明示実行だけで golden 候補を出す。通常 test は絶対に更新しない。 */
public final class GoldenWriter {
    @SuppressWarnings("unchecked")
    public static void main(String[] args) throws Exception {
        if (args.length != 1) throw new IllegalArgumentException("output directory is required");
        Path output = Path.of(args[0]);
        Files.createDirectories(output);
        for (String grammar : new String[]{"operators", "power"}) {
            var lines = new ArrayList<String>();
            var rows = (List<Map<String, Object>>) JsonReader.read(Files.readString(
                Path.of("src/test/resources/corpus", grammar + ".json")));
            for (var row : rows) {
                String input = (String) row.get("input");
                ParseResult<?> result;
                MappingException error = null;
                try { result = grammar.equals("operators") ? OperatorsParser.parse(input) : PowerParser.parse(input); }
                catch (MappingException e) { result = e.syntax(); error = e; }
                lines.add("  " + Json.write(Json.result(input, result, error)));
            }
            Files.writeString(output.resolve(grammar + ".json"), "[\n" + String.join(",\n", lines) + "\n]\n");
        }
    }
}
