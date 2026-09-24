package org.ubnfc.ubnf.lsp.dap;

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
 * 台本クライアントを生成 DAP サーバ（文法デバッガ）にぶつけ、1 セッション分を golden と突き合わせる。
 *
 * <p>台本は fixture ごとに同じ:
 * {@code initialize → launch → setBreakpoints → configurationDone → threads →
 * stackTrace/scopes/variables → continue → next ×3 → stepIn → stepOut → evaluate → setVariable → disconnect}。
 *
 * <p>ブレークポイントの行は {@code src/test/resources/dap/<fixture>.txt}（1 起点、1 行 1 個）。
 * golden の更新は {@code mvn test -Pdap -Dubnfc.golden.update=true}。
 *
 * <p>{@code non-bmp.ubnf} は BMP 外文字を含む。code point 位置を UTF-16 index として渡す実装だと
 * stackFrame の column がずれる（旧スタックの {@code stackTrace} は実際にそうなっている）。
 */
class GoldenDapTest {
    private static final Path FIXTURES = Path.of("src/test/resources/fixtures");
    private static final Path BREAKPOINTS = Path.of("src/test/resources/dap");
    private static final Path GOLDEN = Path.of("src/test/resources/golden");
    private static final boolean UPDATE = Boolean.getBoolean("ubnfc.golden.update");
    private static final String SUFFIX = ".ubnf";

    @TestFactory Stream<DynamicTest> sessions() throws Exception {
        var paths = Files.list(FIXTURES).filter(p -> p.toString().endsWith(SUFFIX)).sorted().toList();
        var tests = new ArrayList<DynamicTest>();
        for (Path path : paths) {
            Path lines = BREAKPOINTS.resolve(path.getFileName() + ".txt");
            if (!Files.exists(lines)) continue;
            tests.add(DynamicTest.dynamicTest(path.getFileName().toString(), () -> {
                String actual = new DapSession(path, breakpoints(lines)).run();
                Path golden = GOLDEN.resolve(path.getFileName() + ".dap.json");
                if (UPDATE) {
                    Files.createDirectories(GOLDEN);
                    Files.writeString(golden, actual, StandardCharsets.UTF_8);
                }
                assertEquals(Files.readString(golden, StandardCharsets.UTF_8), actual, path.getFileName().toString());
            }));
        }
        assertTrue(tests.size() >= 3, "DAP の fixture が足りない: " + tests.size());
        return tests.stream();
    }

    private static List<Integer> breakpoints(Path path) throws Exception {
        var out = new ArrayList<Integer>();
        for (String line : Files.readAllLines(path, StandardCharsets.UTF_8))
            if (!line.isBlank() && !line.startsWith("#")) out.add(Integer.parseInt(line.strip()));
        return out;
    }
}
