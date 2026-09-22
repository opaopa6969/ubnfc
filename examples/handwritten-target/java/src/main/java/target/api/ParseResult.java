package target.api;

import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

public record ParseResult<N>(boolean ok, int consumedCp, int matchedCp, Optional<N> ast,
                             Map<N, Span> nodeSpans, List<Capture> captures,
                             Diagnostic diagnostics) {
    public ParseResult {
        // 等値な Number(1) でも別出現の span を失わない。
        nodeSpans = Collections.unmodifiableMap(new IdentityHashMap<>(nodeSpans));
        captures = List.copyOf(captures);
    }
}
