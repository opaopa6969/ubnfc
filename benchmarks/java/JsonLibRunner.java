// JSON 専用ライブラリ（Jackson / Gson）の判定と計測を 1 本にしたドライバ（issue #68）。
//
//   JsonLibRunner check <variant> <file>...
//       1 行 1 ファイルで TSV: path<TAB>accept|reject<TAB>理由
//       variant: jackson-default | jackson-strict | jackson-lenient | gson-default | gson-strict
//                | jackson+<JsonReadFeature 名>（その 1 つだけを既定へ足す。帰属の切り分け用）
//   JsonLibRunner bench <mode> <runs> <runSeconds> <file>...
//       CSV: impl,input,bytes,mode,iterations,warmup,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s,alloc_bytes_per_parse
//       mode: jackson-stream | jackson-tree | jackson-map | gson-element | gson-map
//
// 計測の作法は benchmarks/java/AntlrRunner.java と揃えてある（同一セッションで交互に
// 回して並べるため）。違いは warmup の下限だけで、ここでは
// 「**500 反復以上 かつ 5 秒以上**」を満たすまで暖機する（`bench.warmupIterations` /
// `bench.warmupMinSeconds`、打ち切りは `bench.warmupSeconds` 秒）。JMH は使っていない。
// JMH を使うと ANTLR / ubnfc の既存の行（手書きループ）と同じ走行で並べられなくなり、
// 「同一セッション・交互実行」という #26 からの作法が崩れるためで、
// 代わりに暖機回数を出力の warmup 列に必ず出す。
//
// 判定の作法:
//   - 入力は byte 列を**厳密な** UTF-8 として復号する（AntlrRunner と同じ関数）。
//     復号できないものは `not-utf8` として拒否する。ライブラリ側の byte 入口を使うと
//     「復号の差」が比較に混ざるので、Java の 3 実装（ubnfc / ANTLR / Jackson・Gson）は
//     すべて同じ復号を通してから String を渡す。
//   - 受理は「**先頭から 1 つの JSON 値**を読み切り、その後に何も残っていない」こと。
//     Jackson は streaming の token を数えて top-level 値が 1 つであることを見る
//     （`ObjectMapper.readValue` の既定は末尾の余りを黙って捨てるので、判定には使わない）。
//     Gson は `JsonParser.parseString` / `fromJson(JsonReader)` のあと END_DOCUMENT を確かめる。
//
// 各 variant の設定:
//   jackson-default … `new JsonFactory()` の既定そのまま。
//   jackson-strict  … JsonReadFeature の ALLOW_* 14 個を**明示的に全て off**
//                     （既定でも off のものが多いので、既定との差がそのまま
//                       「Jackson の既定がどれだけ厳格か」を示す）。
//   jackson-lenient … 同じ 14 個を**全て on**（寛容モードが n_ を何件受理するかを見る）。
//   gson-default    … `JsonParser.parseString`（Gson は LEGACY_STRICT の reader を
//                     LENIENT に上げてから読む＝既定が寛容）。
//   gson-strict     … `new GsonBuilder().setStrictness(Strictness.STRICT)`（Gson 2.11+）。
import java.io.IOException;
import java.io.StringReader;
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
import java.util.Map;

import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonFactoryBuilder;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonToken;
import com.fasterxml.jackson.core.json.JsonReadFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.Strictness;
import com.google.gson.stream.JsonReader;

public final class JsonLibRunner {

  /** Jackson 2.x の JSON 固有の寛容 feature（全部）。strict では off、lenient では on にする。 */
  private static final JsonReadFeature[] LENIENCY = JsonReadFeature.values();

  private static JsonFactory jacksonFactory(String variant) {
    switch (variant) {
      case "jackson-default":
        return new JsonFactory();
      case "jackson-strict": {
        JsonFactoryBuilder b = (JsonFactoryBuilder) JsonFactory.builder();
        for (JsonReadFeature f : LENIENCY) { b.disable(f); }
        return b.build();
      }
      case "jackson-lenient": {
        JsonFactoryBuilder b = (JsonFactoryBuilder) JsonFactory.builder();
        for (JsonReadFeature f : LENIENCY) { b.enable(f); }
        return b.build();
      }
      default: {
        // `jackson+<JsonReadFeature 名>` … その 1 つだけを既定へ足す（帰属の切り分け用）。
        if (variant.startsWith("jackson+")) {
          JsonFactoryBuilder b = (JsonFactoryBuilder) JsonFactory.builder();
          b.enable(JsonReadFeature.valueOf(variant.substring("jackson+".length())));
          return b.build();
        }
        throw new IllegalArgumentException("unknown jackson variant: " + variant);
      }
    }
  }

  /**
   * Jackson の streaming で「先頭から 1 つの JSON 値を読み切り、その後に何も無い」ことを見る。
   * 値の中身（数値の実体・文字列の実体）は作らない。例外はそのまま呼び出し元へ。
   */
  private static void jacksonRecognizeOne(JsonFactory factory, String text) throws IOException {
    try (JsonParser p = factory.createParser(text)) {
      int depth = 0;
      int topLevelValues = 0;
      JsonToken t;
      while ((t = p.nextToken()) != null) {
        if (t == JsonToken.START_OBJECT || t == JsonToken.START_ARRAY) {
          if (depth == 0) { topLevelValues++; }
          depth++;
        } else if (t == JsonToken.END_OBJECT || t == JsonToken.END_ARRAY) {
          depth--;
        } else if (t != JsonToken.FIELD_NAME && depth == 0) {
          topLevelValues++;
        }
        if (topLevelValues > 1) { throw new IOException("trailing top-level value"); }
      }
      if (topLevelValues != 1) { throw new IOException("no complete top-level value"); }
      if (depth != 0) { throw new IOException("unbalanced"); }
    }
  }

  /** 厳密な UTF-8 復号。置換せず、不正なら例外（AntlrRunner と同じ）。 */
  private static String decodeStrictUtf8(byte[] bytes) throws CharacterCodingException {
    CharsetDecoder decoder = StandardCharsets.UTF_8.newDecoder()
        .onMalformedInput(CodingErrorAction.REPORT)
        .onUnmappableCharacter(CodingErrorAction.REPORT);
    CharBuffer out = decoder.decode(ByteBuffer.wrap(bytes));
    return out.toString();
  }

  public static void main(String[] args) throws Exception {
    if (args.length < 3) {
      System.err.println("usage: JsonLibRunner check <variant> <file>... "
          + "| JsonLibRunner bench <mode> <runs> <sec> <file>...");
      System.exit(2);
    }
    if (args[0].equals("check")) { check(args[1], Arrays.copyOfRange(args, 2, args.length)); return; }
    if (args[0].equals("bench")) {
      bench(args[1], Integer.parseInt(args[2]), Double.parseDouble(args[3]),
          Arrays.copyOfRange(args, 4, args.length));
      return;
    }
    System.err.println("unknown mode: " + args[0]);
    System.exit(2);
  }

  private interface Judge { void parse(String text) throws Exception; }

  private static Judge judge(String variant) {
    switch (variant) {
      case "gson-default":
        // JsonParser.parseString は reader を LENIENT へ上げて読み、末尾の余りも見る。
        return text -> {
          JsonElement e = com.google.gson.JsonParser.parseString(text);
          if (e == null) { throw new IOException("empty"); }
        };
      case "gson-strict": {
        Gson gson = new GsonBuilder().setStrictness(Strictness.STRICT).create();
        return text -> {
          JsonReader reader = new JsonReader(new StringReader(text));
          reader.setStrictness(Strictness.STRICT);
          JsonElement e = gson.fromJson(reader, JsonElement.class);
          if (e == null) { throw new IOException("empty"); }
          if (reader.peek() != com.google.gson.stream.JsonToken.END_DOCUMENT) {
            throw new IOException("trailing content");
          }
        };
      }
      default: {
        if (variant.startsWith("jackson")) {
          JsonFactory factory = jacksonFactory(variant);
          return text -> jacksonRecognizeOne(factory, text);
        }
        throw new IllegalArgumentException("unknown variant: " + variant);
      }
    }
  }

  private static void check(String variant, String[] files) throws IOException {
    Judge judge = judge(variant);
    StringBuilder out = new StringBuilder();
    for (String file : files) {
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
          judge.parse(text);
        } catch (Exception | StackOverflowError e) {
          verdict = "reject";
          reason = shorten(e.getClass().getSimpleName() + ":" + e.getMessage());
        }
      }
      out.append(file).append('\t').append(verdict).append('\t').append(reason).append('\n');
    }
    System.out.print(out);
  }

  private static String shorten(String s) {
    String one = s.replace('\n', ' ').replace('\t', ' ').replace('\r', ' ');
    return one.length() > 90 ? one.substring(0, 90) : one;
  }

  /** mode ごとの「何を作るか」。報告の列見出しにそのまま載せる。 */
  private static String label(String mode) {
    switch (mode) {
      case "jackson-stream":  return "streaming(何も作らない)";
      case "jackson-tree":    return "readTree(JsonNode 木・span なし)";
      case "jackson-map":     return "readValue(Map)";
      case "gson-element":    return "parseString(JsonElement 木・span なし)";
      case "gson-map":        return "fromJson(Map)";
      default: throw new IllegalArgumentException("unknown mode: " + mode);
    }
  }

  private static String implOf(String mode) {
    return mode.startsWith("jackson") ? "Jackson" : "Gson";
  }

  private static void bench(String mode, int runs, double runSeconds, String[] files) throws Exception {
    final JsonFactory factory = new JsonFactory();           // 速度は既定設定で測る
    final ObjectMapper mapper = new ObjectMapper();
    final Gson gson = new Gson();
    com.sun.management.ThreadMXBean threads =
        (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
    long self = Thread.currentThread().threadId();
    System.out.println("impl,input,bytes,mode,iterations,warmup,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s,alloc_bytes_per_parse");
    for (String file : files) {
      byte[] bytes = Files.readAllBytes(Path.of(file));
      final String text = decodeStrictUtf8(bytes);
      Runnable once;
      switch (mode) {
        case "jackson-stream":
          once = () -> {
            try (JsonParser p = factory.createParser(text)) {
              while (p.nextToken() != null) { /* 何も作らない */ }
            } catch (IOException e) { throw new RuntimeException(e); }
          };
          break;
        case "jackson-tree":
          once = () -> {
            try {
              JsonNode node = mapper.readTree(text);
              if (node == null) { throw new IllegalStateException("null tree"); }
            } catch (IOException e) { throw new RuntimeException(e); }
          };
          break;
        case "jackson-map":
          once = () -> {
            try {
              Object value = mapper.readValue(text, Object.class);
              if (value == null) { throw new IllegalStateException("null value"); }
            } catch (IOException e) { throw new RuntimeException(e); }
          };
          break;
        case "gson-element":
          once = () -> {
            JsonElement e = com.google.gson.JsonParser.parseString(text);
            if (e == null) { throw new IllegalStateException("null element"); }
          };
          break;
        case "gson-map":
          once = () -> {
            Object value = gson.fromJson(text, Object.class);
            if (value == null) { throw new IllegalStateException("null value"); }
          };
          break;
        default:
          throw new IllegalArgumentException("unknown mode: " + mode);
      }
      // JIT 暖機。**500 反復以上 かつ 5 秒以上**を満たすまで回す（打ち切りは 30 秒）。
      int warmupTarget = Integer.parseInt(System.getProperty("bench.warmupIterations", "500"));
      double warmupMin = Double.parseDouble(System.getProperty("bench.warmupMinSeconds", "5.0"));
      double warmupCap = Double.parseDouble(System.getProperty("bench.warmupSeconds", "30.0"));
      long warmStart = System.nanoTime();
      int warmup = 0;
      while (true) {
        once.run();
        warmup++;
        double elapsed = (System.nanoTime() - warmStart) / 1e9;
        if (warmup >= warmupTarget && elapsed >= warmupMin) { break; }
        if (elapsed >= warmupCap && warmup >= 5) { break; }
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
      String name = Path.of(file).getFileName().toString();
      System.out.printf("%s,%s,%d,%s,%d,%d,%d,%.6f,%.6f,%.6f,%.4f,%.2f,%d%n",
          implOf(mode), name, bytes.length, label(mode), iterations, warmup, runs,
          median, samples[0], samples[runs - 1],
          median * 1e3 / Math.max(1, bytes.length),
          bytes.length / (median * 1e-3) / 1e6, allocated);
    }
  }
}
