package generated.api;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import generated.rt.ScopeStore;
public record ParseResult<N>(boolean ok, int consumedCp, int matchedCp, Optional<N> ast,
    Map<Object, Span> nodeSpans, List<Capture> captures, List<Lexical> lexical,
    Scope scope, Diagnostic diagnostics, Map<String, List<Capture>> catalogs, List<Recovery> recoveries) {
    /** Adopted recovery regions; all positions are Unicode code points. */
    public record Recovery(String ruleId, String mode, Span span, String syncPattern, Span skippedSpan,
                           Span syncSpan, List<Long> captureOccurrences, Diagnostic diagnostic) {
        public Recovery { captureOccurrences = List.copyOf(captureOccurrences); }
    }
    public record Lexical(long occurrenceId, long parentOccurrenceId, String ruleId, String exprId,
                          Span span, long completionOrder) {}
    public record Scope(List<ScopeStore.Decl> declarations, List<ScopeStore.Reference> references,
                        List<ScopeStore.Diagnostic> diagnostics, List<ScopeEvent> events) {
        public Scope { declarations = List.copyOf(declarations); references = List.copyOf(references); diagnostics = List.copyOf(diagnostics); events = List.copyOf(events); }
        public Scope(List<ScopeStore.Decl> declarations, List<ScopeStore.Reference> references, List<ScopeStore.Diagnostic> diagnostics) {
            this(declarations, references, diagnostics, List.of());
        }
    }
    public record ScopeEvent(long order, String action, String mode, String name, int offsetCp, int lengthCp) {}
    public ParseResult {
        nodeSpans = Collections.unmodifiableMap(new IdentityHashMap<>(nodeSpans));
        captures = List.copyOf(captures); lexical = List.copyOf(lexical); catalogs = Map.copyOf(catalogs);
        recoveries = List.copyOf(recoveries);
    }
    public ParseResult(boolean ok, int consumedCp, int matchedCp, Optional<N> ast, Map<Object, Span> nodeSpans,
                       List<Capture> captures, List<Lexical> lexical, Scope scope, Diagnostic diagnostics,
                       Map<String, List<Capture>> catalogs) {
        this(ok, consumedCp, matchedCp, ast, nodeSpans, captures, lexical, scope, diagnostics, catalogs, List.of());
    }
    public int consumed() { return consumedCp; }
    public int matched() { return matchedCp; }
    public <T> ParseResult<T> withAst(T node, Map<Object, Span> spans) {
        return new ParseResult<>(ok, consumedCp, matchedCp, Optional.ofNullable(node), spans,
            captures, lexical, scope, diagnostics, catalogs, recoveries);
    }
}
