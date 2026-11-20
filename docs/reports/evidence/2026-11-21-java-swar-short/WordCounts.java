package gen.json;

import java.nio.file.Files;
import java.nio.file.Path;
import gen.json.api.ParseOptions;

public final class WordCounts {
    public static void main(String[] args) throws Exception {
        String input = Files.readString(Path.of(args[0]));
        var result = JsonParser.parse(input, ParseOptions.RECOGNIZE);
        if (!result.ok() || result.consumedCp() != input.codePointCount(0, input.length())) {
            throw new IllegalStateException("入力全体を受理できない");
        }
        for (int i = 0; i < JsonParser.WORD_NAMES.length; i++) {
            System.out.print(JsonParser.WORD_NAMES[i]);
            for (long count : JsonParser.WORD_COUNTS[i]) System.out.print("," + count);
            System.out.println();
        }
    }
}
