package org.ubnfc.p4.lsp;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Stream;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

/**
 * 段階 3 の capability を台本クライアント（実際の JSON-RPC）で叩き、応答を golden と突き合わせる。
 *
 * <p>fixture ごとに document symbols と semantic tokens を丸ごと、
 * {@code src/test/resources/probes/<fixture>.txt} に書いた位置ごとに
 * definition / references / hover / completion を記録する。
 *
 * <p>golden の更新は {@code mvn test -Dubnfc.golden.update=true}。
 * {@code non-bmp-nav.tiny} は 1 行目に BMP 外文字 2 個を置き、その<b>後ろ</b>で記号を宣言する。
 * code point を UTF-16 index として渡す実装だと definition の桁が 2 ずれる。
 */
class GoldenCapabilitiesTest {
    private static final Path FIXTURES = Path.of("src/test/resources/fixtures");
    private static final Path PROBES = Path.of("src/test/resources/probes");
    private static final Path GOLDEN = Path.of("src/test/resources/golden");
    private static final boolean UPDATE = Boolean.getBoolean("ubnfc.golden.update");

    record Probe(int line, int character) {}

    @TestFactory Stream<DynamicTest> fixtures() throws Exception {
        var paths = Files.list(FIXTURES).filter(p -> p.toString().endsWith(".tiny")).sorted().toList();
        var client = new ScriptedClient();
        var legend = client.initialized().getCapabilities().getSemanticTokensProvider().getLegend().getTokenTypes();
        var tests = new ArrayList<DynamicTest>();
        tests.add(DynamicTest.dynamicTest("capabilities", () -> {
            var capabilities = client.initialized().getCapabilities();
            assertTrue(capabilities.getDefinitionProvider().getLeft(), "definition");
            assertTrue(capabilities.getReferencesProvider().getLeft(), "references");
            assertTrue(capabilities.getDocumentSymbolProvider().getLeft(), "documentSymbol");
            assertTrue(capabilities.getHoverProvider().getLeft(), "hover");
            assertEquals(List.of("keyword", "operator", "string", "number", "variable"), legend);
        }));
        for (Path path : paths)
            tests.add(DynamicTest.dynamicTest(path.getFileName().toString(), () -> {
                String name = path.getFileName().toString();
                String text = Files.readString(path, StandardCharsets.UTF_8);
                String uri = path.toUri().toString();
                client.diagnosticsOf(uri, text);
                String actual = render(client, uri, probes(name), legend);
                Path golden = GOLDEN.resolve(name + ".caps.json");
                if (UPDATE) {
                    Files.createDirectories(GOLDEN);
                    Files.writeString(golden, actual, StandardCharsets.UTF_8);
                }
                assertEquals(Files.readString(golden, StandardCharsets.UTF_8), actual, name);
            }));
        tests.add(DynamicTest.dynamicTest("shutdown", client::close));
        return tests.stream();
    }

    /** {@code probes/<fixture>.txt} の「行 桁」。無ければ先頭だけ。 */
    static List<Probe> probes(String fixture) throws Exception {
        Path path = PROBES.resolve(fixture + ".txt");
        if (!Files.exists(path)) return List.of(new Probe(0, 0));
        var out = new ArrayList<Probe>();
        for (String line : Files.readAllLines(path, StandardCharsets.UTF_8)) {
            String trimmed = line.trim();
            if (trimmed.isEmpty() || trimmed.startsWith("#")) continue;
            String[] parts = trimmed.split("\\s+");
            out.add(new Probe(Integer.parseInt(parts[0]), Integer.parseInt(parts[1])));
        }
        return out;
    }

    private static String render(ScriptedClient client, String uri, List<Probe> probes, List<String> legend)
            throws Exception {
        var out = new StringBuilder("{\n");
        out.append("  \"documentSymbols\": ").append(Goldens.symbols(client.documentSymbols(uri), "  ")).append(",\n");
        var tokens = client.semanticTokens(uri);
        out.append("  \"semanticTokens\": ")
           .append(Goldens.semanticTokens(tokens == null ? List.of() : tokens.getData(), legend, "  ")).append(",\n");
        var items = new ArrayList<String>();
        for (Probe probe : probes) {
            var completion = new ArrayList<String>();
            for (var item : client.completion(uri, probe.line(), probe.character())) completion.add(item.getLabel());
            items.add("{\"at\": [" + probe.line() + ", " + probe.character() + "]"
                + ", \"definition\": " + Goldens.locations(client.definition(uri, probe.line(), probe.character()))
                + ", \"references\": " + Goldens.locations(client.references(uri, probe.line(), probe.character(), true))
                + ", \"referencesOnly\": " + Goldens.locations(client.references(uri, probe.line(), probe.character(), false))
                + ", \"hover\": " + Goldens.hover(client.hover(uri, probe.line(), probe.character()))
                + ", \"completion\": " + Goldens.strings(completion.size() > 20 ? completion.subList(0, 20) : completion)
                + ", \"completionCount\": " + completion.size() + "}");
        }
        out.append("  \"probes\": ").append(Goldens.list(items, "  ")).append('\n');
        return out.append("}\n").toString();
    }
}
