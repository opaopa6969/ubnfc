package generated.api;
import java.util.Map;
public record ParseOptions(boolean buildAst, boolean requireFullInput, boolean lexical,
                           boolean memo, Map<String, TokenScanner> scanners) {
    public ParseOptions { scanners = Map.copyOf(scanners); }
    public ParseOptions(boolean buildAst, boolean requireFullInput) { this(buildAst, requireFullInput, false, true, Map.of()); }
    public static final ParseOptions DEFAULT = new ParseOptions(true, true);
    public static final ParseOptions RECOGNIZE = new ParseOptions(false, true);
    public static final ParseOptions PREFIX = new ParseOptions(true, false);
}
