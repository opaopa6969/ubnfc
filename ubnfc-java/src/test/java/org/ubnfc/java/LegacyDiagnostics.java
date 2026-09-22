package org.ubnfc.java;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import org.ubnfc.java.ir.Json;

/**
 * 旧 Java 実測値の読み込み専用 fixture。LegacyDiagnosticsDump が旧 root parser を各行の input で
 * 実行して採取した値を使う。実測手順・文法・入力一覧は src/test/legacy/README.md を参照。
 */
final class LegacyDiagnostics {
    private LegacyDiagnostics() {}

    static Map<String, Object> forInput(String grammar, String input) throws IOException {
        var rows = Json.array(Json.read(Files.readString(
            Path.of("src/test/resources/legacy-diagnostics/" + grammar + ".json"))));
        var matches = rows.stream().map(Json::object).filter(row -> input.equals(row.get("input"))).toList();
        if (matches.size() != 1) throw new AssertionError("Expected one legacy measurement for " + grammar + ": " + input);
        return matches.getFirst();
    }
}
