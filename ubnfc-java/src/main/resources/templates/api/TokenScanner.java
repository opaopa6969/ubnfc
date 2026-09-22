package generated.api;
import java.util.List;
import generated.rt.Input;
/** Scanner positions and value spans use UTF-16; effects are replayable, never direct mutations. */
@FunctionalInterface
public interface TokenScanner {
    ScanResult scan(Input input, int consumed, int matched);
    record Effect(String action, String name, int offsetCp, int lengthCp) {}
    record ScanDiagnostic(int offsetUtf16, String expected) {}
    record ScanResult(boolean ok, int consumedEnd, int matchedEnd, int valueStart, int valueEnd,
                      List<ScanDiagnostic> diagnostics, List<Effect> effects) {
        public ScanResult { diagnostics = List.copyOf(diagnostics); effects = List.copyOf(effects); }
    }
}
