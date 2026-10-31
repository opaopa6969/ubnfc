#!/usr/bin/env bash
# P4 LSP の全文解析費用を測る。先に MAVEN_FLAGS='' SKIP_VSIX=1 scripts/build-vsix.sh p4 を実行する。
# WARMUP / SAMPLES は各入力の反復数。CPU を指定すると JVM を taskset で固定する。
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
jar="$repo/examples/p4-vscode/server/target/ubnfc-p4-lsp-server.jar"
if [[ ! -f "$jar" ]]; then
  echo "先に P4 LSP サーバ jar をビルドしてください: $jar" >&2
  exit 1
fi
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
cat > "$scratch/ReparseCost.java" <<'JAVA'
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.LinkedHashMap;
import org.unlaxer.ubnfc.examples.p4.lsp.LanguageServerImpl;

public final class ReparseCost {
    private static volatile Object sink;

    // 宣言、式、関数をそれぞれ束ねて、受理される長い P4 文書を作る。
    private static String scaled(String base, int count) {
        int annotation = base.indexOf("@bench(");
        int body = base.indexOf('\n', annotation) + 1;
        int function = base.indexOf("float bonus(", body);
        var declarations = new StringBuilder();
        var expressions = new StringBuilder("@bench(name='scaling')\n");
        var functions = new StringBuilder();
        for (int i = 1; i <= count; i++) {
            String suffix = Integer.toString(i);
            declarations.append(rename(base.substring(0, annotation), suffix));
            if (i > 1) expressions.append("+ ");
            expressions.append(rename(base.substring(body, function), suffix));
            functions.append(rename(base.substring(function), suffix));
        }
        return declarations.toString() + expressions + functions;
    }

    private static String rename(String text, String suffix) {
        return text.replace("$price", "$price" + suffix)
            .replace("$country", "$country" + suffix).replace("bonus", "bonus" + suffix);
    }

    public static void main(String[] args) throws Exception {
        Path fixtures = Path.of(args[0]);
        int warmup = Integer.parseInt(args[1]), samples = Integer.parseInt(args[2]);
        if (warmup < 1 || samples < 2) throw new IllegalArgumentException("WARMUP>=1、SAMPLES>=2 が必要です");
        var inputs = new LinkedHashMap<String, String>();
        String base = Files.readString(fixtures.resolve("complex.tiny"), StandardCharsets.UTF_8);
        inputs.put("complex", base);
        for (int n : new int[] {3, 32}) inputs.put("合成x" + n, scaled(base, n));
        inputs.put("complex-x64", Files.readString(fixtures.resolve("complex-x64.tiny"), StandardCharsets.UTF_8));
        for (int n : new int[] {128, 256, 320, 512}) inputs.put("合成x" + n, scaled(base, n));
        var server = new LanguageServerImpl();
        System.out.println("入力,UTF-8_bytes,code_points,構文成功,診断数,min_ms,p50_ms,p90_ms");
        for (var input : inputs.entrySet()) {
            String uri = "file:///計測/" + input.getKey() + ".tiny", text = input.getValue();
            var analysis = server.analysis(uri, text);
            if (analysis.failure() != null) throw analysis.failure();
            if (analysis.primary() == null || !analysis.primary().ok())
                throw new IllegalStateException("構文解析失敗: " + input.getKey());
            int diagnostics = server.analyze(uri, text).size();
            for (int i = 0; i < warmup; i++) sink = server.analyze(uri, text);
            long[] times = new long[samples];
            for (int i = 0; i < samples; i++) {
                long start = System.nanoTime();
                sink = server.analyze(uri, text);
                times[i] = System.nanoTime() - start;
            }
            Arrays.sort(times);
            System.out.printf(java.util.Locale.ROOT, "%s,%d,%d,true,%d,%.3f,%.3f,%.3f%n",
                input.getKey(), text.getBytes(StandardCharsets.UTF_8).length,
                text.codePointCount(0, text.length()), diagnostics,
                times[0] / 1e6, times[samples / 2] / 1e6,
                times[(int) Math.ceil(samples * 0.9) - 1] / 1e6);
        }
    }
}
JAVA
javac -encoding UTF-8 -cp "$jar" -d "$scratch" "$scratch/ReparseCost.java"
pin=()
if [[ -n "${CPU:-}" ]]; then pin=(taskset -c "$CPU"); fi
"${pin[@]}" java -Xms512m -Xmx2g -Xss16m -cp "$scratch:$jar" ReparseCost \
  "$repo/examples/p4-java/src/test/resources/fixtures" "${WARMUP:-200}" "${SAMPLES:-201}"
