// 外部比較（issue #26）の ubnfc Java 側ドライバ。AntlrRunner と同じ入出力にしてある。
//
//   UbnfcRunner check <json|operators|power> <file>...
//   UbnfcRunner bench <json|operators|power> <認識|AST> <runs> <runSeconds> <file>...
//
// 判定の作法:
//   - 入力は byte 列を**厳密な** UTF-8 として復号する（不正なら not-utf8 で拒否）。
//     ANTLR 側と同じ境界を使うことで、比較が「復号の差」にならないようにしている。
//   - `requireFullInput` は既定 true。末尾の余りは ok=false。
//   - 認識のみ = ParseOptions.RECOGNIZE、typed AST = ParseOptions.DEFAULT.withObservations(false,false)
//     （出現表と診断を作らない最速の AST 経路。docs/reports/2026-09-24-java-ast-path.md）。
import java.lang.management.ManagementFactory;
import java.nio.ByteBuffer;
import java.nio.CharBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CharsetDecoder;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public final class UbnfcRunner {

  /** 文法ごとの入口。戻り値は (ok, consumedCp)。 */
  private interface Entry {
    long parse(String text, boolean buildAst);
  }

  private static long pack(boolean ok, int consumed) {
    return (ok ? 1L : 0L) << 32 | (consumed & 0xffffffffL);
  }

  private static Entry entry(String grammar) {
    switch (grammar) {
      case "json":
        return (text, buildAst) -> {
          var options = buildAst
              ? gen.json.api.ParseOptions.DEFAULT.withObservations(false, false)
              : gen.json.api.ParseOptions.RECOGNIZE;
          var r = gen.json.JsonParser.parse(text, options);
          return pack(r.ok() && (!buildAst || r.ast().isPresent()), r.consumedCp());
        };
      case "operators":
        return (text, buildAst) -> {
          var options = buildAst
              ? gen.operators.api.ParseOptions.DEFAULT.withObservations(false, false)
              : gen.operators.api.ParseOptions.RECOGNIZE;
          var r = gen.operators.OperatorsParser.parse(text, options);
          return pack(r.ok() && (!buildAst || r.ast().isPresent()), r.consumedCp());
        };
      case "power":
        return (text, buildAst) -> {
          var options = buildAst
              ? gen.power.api.ParseOptions.DEFAULT.withObservations(false, false)
              : gen.power.api.ParseOptions.RECOGNIZE;
          var r = gen.power.PowerParser.parse(text, options);
          return pack(r.ok() && (!buildAst || r.ast().isPresent()), r.consumedCp());
        };
      default:
        throw new IllegalArgumentException("unknown grammar: " + grammar);
    }
  }

  private static String decodeStrictUtf8(byte[] bytes) throws CharacterCodingException {
    CharsetDecoder decoder = StandardCharsets.UTF_8.newDecoder()
        .onMalformedInput(CodingErrorAction.REPORT)
        .onUnmappableCharacter(CodingErrorAction.REPORT);
    CharBuffer out = decoder.decode(ByteBuffer.wrap(bytes));
    return out.toString();
  }

  public static void main(String[] args) throws Exception {
    if (args.length < 3) {
      System.err.println("usage: UbnfcRunner check <grammar> <file>... | UbnfcRunner bench <grammar> <認識|AST> <runs> <sec> <file>...");
      System.exit(2);
    }
    Entry entry = entry(args[1]);
    if (args[0].equals("check")) {
      StringBuilder out = new StringBuilder();
      for (String file : Arrays.copyOfRange(args, 2, args.length)) {
        String verdict = "accept";
        String reason = "";
        byte[] bytes = Files.readAllBytes(Path.of(file));
        String text = null;
        try {
          text = decodeStrictUtf8(bytes);
        } catch (CharacterCodingException e) {
          verdict = "reject";
          reason = "not-utf8";
        }
        if (text != null) {
          try {
            long packed = entry.parse(text, false);
            if ((packed >>> 32) == 0) {
              verdict = "reject";
              reason = "failed at cp " + (int) packed;
            }
          } catch (RuntimeException | StackOverflowError e) {
            verdict = "reject";
            reason = e.getClass().getSimpleName();
          }
        }
        out.append(file).append('\t').append(verdict).append('\t').append(reason).append('\n');
      }
      System.out.print(out);
      return;
    }
    if (!args[0].equals("bench") || args.length < 6) {
      System.err.println("bad arguments");
      System.exit(2);
    }
    boolean buildAst = args[2].equals("AST");
    int runs = Integer.parseInt(args[3]);
    double runSeconds = Double.parseDouble(args[4]);
    com.sun.management.ThreadMXBean threads =
        (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
    long self = Thread.currentThread().threadId();
    System.out.println("impl,input,bytes,mode,iterations,warmup,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s,alloc_bytes_per_parse");
    for (String file : Arrays.copyOfRange(args, 5, args.length)) {
      byte[] bytes = Files.readAllBytes(Path.of(file));
      String text = decodeStrictUtf8(bytes);
      Runnable once = () -> {
        long packed = entry.parse(text, buildAst);
        if ((packed >>> 32) == 0) { throw new IllegalStateException("受理できない: " + file); }
      };
      // JIT 暖機を先に済ませる。冷えた 1 反復から反復数を決めると、解釈実行の所要で
      // 反復数が 3 になり「暖機なしの値」を測ってしまう（実測で 10 倍ずれた）。
      // 暖機は**反復数**で切る（時間で切ると、遅い解釈実行のせいで数回で終わってしまう）。
      // 既定 200 反復、ただし最長 20 秒で打ち切る。-Dbench.warmupIterations / -Dbench.warmupSeconds。
      int warmupTarget = Integer.parseInt(System.getProperty("bench.warmupIterations", "200"));
      double warmupCap = Double.parseDouble(System.getProperty("bench.warmupSeconds", "20.0"));
      long warmStart = System.nanoTime();
      int warmup = 0;
      while (true) {
        once.run();
        warmup++;
        double elapsed = (System.nanoTime() - warmStart) / 1e9;
        if (warmup >= warmupTarget || (elapsed >= warmupCap && warmup >= 5)) { break; }
      }
      long t0 = System.nanoTime();
      once.run();
      double per = Math.max((System.nanoTime() - t0) / 1e9, 1e-9);
      int iterations = (int) Math.min(200_000, Math.max(3, runSeconds / per));
      double[] samples = new double[runs];
      for (int r = 0; r < runs; r++) {
        long start = System.nanoTime();
        for (int i = 0; i < iterations; i++) { once.run(); }
        samples[r] = (System.nanoTime() - start) / 1e6 / iterations;
      }
      Arrays.sort(samples);
      long before = threads.getThreadAllocatedBytes(self);
      once.run();
      long allocated = threads.getThreadAllocatedBytes(self) - before;
      double median = samples[runs / 2];
      System.out.printf("ubnfc-java(生成),%s,%d,%s,%d,%d,%d,%.6f,%.6f,%.6f,%.4f,%.2f,%d%n",
          Path.of(file).getFileName(), bytes.length, buildAst ? "typed AST" : "認識のみ",
          iterations, warmup, runs, median, samples[0], samples[runs - 1],
          median * 1e3 / Math.max(1, bytes.length),
          bytes.length / (median * 1e-3) / 1e6, allocated);
    }
  }
}
