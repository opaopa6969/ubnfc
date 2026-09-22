package target.api;

/** 構文成功後だけ発生する。syntax() に両カーソルと採用 capture を残す。 */
public final class MappingException extends IllegalArgumentException {
    private final ParseResult<?> syntax;
    private final Span span;

    public MappingException(ParseResult<?> syntax, Span span, NumberFormatException cause) {
        super("NUMBER cannot be mapped to int at " + span, cause);
        this.syntax = syntax;
        this.span = span;
    }

    public ParseResult<?> syntax() { return syntax; }
    public Span span() { return span; }
}
