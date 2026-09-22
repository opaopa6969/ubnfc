package target.api;

public record ParseOptions(boolean buildAst, boolean requireFullInput) {
    public static final ParseOptions DEFAULT = new ParseOptions(true, true);
    public static final ParseOptions RECOGNIZE = new ParseOptions(false, true);
    public static final ParseOptions PREFIX = new ParseOptions(true, false);
}
