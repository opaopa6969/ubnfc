import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import org.example.boundary.VocabularyParser;
import org.example.boundary.api.Json;
import org.example.boundary.api.ParseOptions;
import org.example.boundary.api.TokenScanner;

/** 解析全体で不変の語彙を、既存 scanner の宣言効果として返す。位置0は仮の出所。 */
public final class VocabularyProbe {
    public static void main(String[] args) throws Exception {
        String text = Files.readString(Path.of(args[0]));
        String vocabulary = args[1];
        TokenScanner scanner = (input, state) -> {
            int position = state.mode() == TokenScanner.Mode.matchOnly ? state.matched() : state.consumed();
            return new TokenScanner.ScanResult(true, state.consumed(), state.matched(),
                new TokenScanner.ValueSpan(position, position), List.of(),
                List.of(new TokenScanner.Effect("declare", vocabulary, 0, 0)));
        };
        var options = new ParseOptions(false, true, false, Boolean.parseBoolean(args[2]), Map.of(args[3], scanner));
        var result = VocabularyParser.parseEntry("Vocabulary", "Root", text, options);
        System.out.println(Json.write(Map.of("ok", result.ok(),
            "declarations", result.scope().declarations().stream().map(d -> d.name()).toList(),
            "warnings", result.scope().diagnostics().size(), "scopeDepth", result.scope().depth())));
    }
}
