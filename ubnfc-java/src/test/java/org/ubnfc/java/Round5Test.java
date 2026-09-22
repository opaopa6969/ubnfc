package org.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.ubnfc.java.CompilerHarness.*;
import static org.ubnfc.java.IrFixture.*;
import java.lang.reflect.Proxy;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

class Round5Test {
    @Test void externCandidatesKeepScannerDisplaysWithoutClassOrCaptureNames() throws Exception {
        var f = new IrFixture();
        String token = f.token("Root", "EXTERNAL", obj("kind", "extern", "logicalId", "scan",
            "javaBinding", null, "rustBinding", null, "originalClassName", "oracle.ExternalCursorParser",
            "inputState", List.of(), "resultFields", List.of("ok", "consumedEnd", "matchedEnd", "valueSpan", "diagnostics", "effects")));
        f.capture(token, "value", "text", "scalar", obj("kind", "text"));
        f.rule("Root", token);
        try (var h = f.compile()) {
            Class<?> scannerType = h.loader.loadClass(h.pkg + ".api.TokenScanner");
            var diagnostic = h.loader.loadClass(h.pkg + ".api.TokenScanner$ScanDiagnostic").getConstructor(int.class, String.class);
            var result = h.loader.loadClass(h.pkg + ".api.TokenScanner$ScanResult")
                .getConstructor(boolean.class, int.class, int.class, int.class, int.class, List.class, List.class);
            // corpus/oracle/extern.json: input=a、ExternalCursorParser.java:8-10 の独立実測。
            Object scanner = Proxy.newProxyInstance(h.loader, new Class<?>[]{scannerType}, (proxy, method, args) ->
                result.newInstance(false, 0, 0, 0, 0, List.of(diagnostic.newInstance(0, "'😀'"), diagnostic.newInstance(0, "'x'")), List.of()));
            for (boolean memo : List.of(false, true)) {
                Object parsed = h.parse("a", h.options(false, true, true, memo, Map.of("scan", scanner)));
                assertFalse(ok(parsed));
                assertEquals(List.of("'x'", "'😀'", "RootParser"), get(get(parsed, "diagnostics"), "farthestExpected"));
            }
        }
    }
}
