package org.ubnfc.p4.generated.api;
import java.util.Map;
public record ParseOptions(boolean buildAst, boolean requireFullInput, boolean lexical,
                           boolean memo, Map<String, TokenScanner> scanners, int maxDepth) {
    public static final int DEFAULT_MAX_DEPTH = 10_000;
    public ParseOptions {
        scanners = Map.copyOf(scanners);
        if (maxDepth < 1) throw new IllegalArgumentException("maxDepth must be positive");
    }
    public ParseOptions(boolean buildAst, boolean requireFullInput, boolean lexical, boolean memo, Map<String, TokenScanner> scanners) {
        this(buildAst, requireFullInput, lexical, memo, scanners, DEFAULT_MAX_DEPTH);
    }
    public ParseOptions(boolean buildAst, boolean requireFullInput) { this(buildAst, requireFullInput, false, true, Map.of()); }
    public static final ParseOptions DEFAULT = new ParseOptions(true, true);
    public static final ParseOptions RECOGNIZE = new ParseOptions(false, true);
    public static final ParseOptions PREFIX = new ParseOptions(true, false);
}
