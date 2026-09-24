// ANTLR 4 の判定と計測を 1 本にしたドライバ（issue #26）。
//
//   AntlrRunner check <json|operators|power> <file>...
//       1 行 1 ファイルで TSV: path<TAB>accept|reject<TAB>理由
//   AntlrRunner bench <json|operators|power> <tree|recognize> <runs> <runSeconds> <file>...
//       CSV: impl,input,bytes,mode,iterations,warmup,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s,alloc_bytes_per_parse
//
// 判定の作法:
//   - parser は BailErrorStrategy（回復せず例外で落とす）。既定の DefaultErrorStrategy は
//     挿入・削除で回復して「受理」してしまうので、拒否の判定には使えない。
//   - lexer / parser の両方に、最初の syntaxError で例外を投げる ErrorListener を付ける
//     （既定の ConsoleErrorListener は標準エラーに出すだけで受理判定を変えない）。
//   - 入口規則は EOF を含むので、末尾の余りは構文エラーになる。
//   - 入力は byte 列を**厳密な** UTF-8 として復号する。復号できないものは
//     `not-utf8` として拒否する（RFC 8259 §8.1 は JSON text を UTF-8 と定める）。
//     ANTLR の CharStreams は既定で不正 byte を U+FFFD へ置換するので、置換に任せると
//     「不正 UTF-8 を受理」になる。この境界は報告に明記する。
//
// 計測の作法:
//   - 1 反復 = 新しい Lexer + CommonTokenStream + Parser（利用者が実際に書く形）。
//   - tree      … setBuildParseTree(true) で parse tree を作り、no-op listener で全体を walk する。
//   - recognize … setBuildParseTree(false)（認識のみ。parse tree を作らない）。
//   - warmup は反復数の半分（最大 20,000）。JIT の暖機は出力の warmup 列に出す。
import java.io.IOException;
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

import org.antlr.v4.runtime.BaseErrorListener;
import org.antlr.v4.runtime.BailErrorStrategy;
import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.antlr.v4.runtime.Lexer;
import org.antlr.v4.runtime.Parser;
import org.antlr.v4.runtime.RecognitionException;
import org.antlr.v4.runtime.Recognizer;
import org.antlr.v4.runtime.misc.ParseCancellationException;
import org.antlr.v4.runtime.tree.AbstractParseTreeVisitor;
import org.antlr.v4.runtime.tree.ErrorNode;
import org.antlr.v4.runtime.tree.ParseTree;
import org.antlr.v4.runtime.tree.ParseTreeListener;
import org.antlr.v4.runtime.tree.ParseTreeWalker;
import org.antlr.v4.runtime.tree.TerminalNode;
import org.antlr.v4.runtime.ParserRuleContext;

public final class AntlrRunner {

  /** 最初の字句・構文エラーで打ち切る listener。 */
  static final class Throwing extends BaseErrorListener {
    static final Throwing INSTANCE = new Throwing();
    @Override
    public void syntaxError(Recognizer<?, ?> recognizer, Object offendingSymbol, int line,
        int charPositionInLine, String msg, RecognitionException e) {
      throw new ParseCancellationException(line + ":" + charPositionInLine + " " + msg);
    }
  }

  /** 木の構築込みを測るための no-op listener（walk のコストだけを載せる）。 */
  static final class NoOp implements ParseTreeListener {
    static final NoOp INSTANCE = new NoOp();
    @Override public void visitTerminal(TerminalNode node) { }
    @Override public void visitErrorNode(ErrorNode node) { }
    @Override public void enterEveryRule(ParserRuleContext ctx) { }
    @Override public void exitEveryRule(ParserRuleContext ctx) { }
  }

  private interface Entry {
    /** 解析して parse tree を返す（buildTree=false なら null でよい）。 */
    ParseTree parse(CharStream input, boolean buildTree);
  }

  private static Entry entry(String grammar) {
    switch (grammar) {
      case "json":
        return (input, buildTree) -> {
          gen.json.JSONLexer lexer = new gen.json.JSONLexer(input);
          gen.json.JSONParser parser = prepare(lexer, new gen.json.JSONParser(new CommonTokenStream(lexer)), buildTree);
          return parser.json();
        };
      case "operators":
        return (input, buildTree) -> {
          gen.operators.OperatorsLexer lexer = new gen.operators.OperatorsLexer(input);
          gen.operators.OperatorsParser parser = prepare(lexer, new gen.operators.OperatorsParser(new CommonTokenStream(lexer)), buildTree);
          return parser.start();
        };
      case "power":
        return (input, buildTree) -> {
          gen.power.PowerLexer lexer = new gen.power.PowerLexer(input);
          gen.power.PowerParser parser = prepare(lexer, new gen.power.PowerParser(new CommonTokenStream(lexer)), buildTree);
          return parser.start();
        };
      default:
        throw new IllegalArgumentException("unknown grammar: " + grammar);
    }
  }

  private static <P extends Parser> P prepare(Lexer lexer, P parser, boolean buildTree) {
    lexer.removeErrorListeners();
    lexer.addErrorListener(Throwing.INSTANCE);
    parser.removeErrorListeners();
    parser.addErrorListener(Throwing.INSTANCE);
    parser.setErrorHandler(new BailErrorStrategy());
    parser.setBuildParseTree(buildTree);
    return parser;
  }

  /** 厳密な UTF-8 復号。置換せず、不正なら例外。 */
  private static String decodeStrictUtf8(byte[] bytes) throws CharacterCodingException {
    CharsetDecoder decoder = StandardCharsets.UTF_8.newDecoder()
        .onMalformedInput(CodingErrorAction.REPORT)
        .onUnmappableCharacter(CodingErrorAction.REPORT);
    CharBuffer out = decoder.decode(ByteBuffer.wrap(bytes));
    return out.toString();
  }

  public static void main(String[] args) throws Exception {
    if (args.length < 3) {
      System.err.println("usage: AntlrRunner check <grammar> <file>... | AntlrRunner bench <grammar> <tree|recognize> <runs> <sec> <file>...");
      System.exit(2);
    }
    if (args[0].equals("check")) { check(args[1], Arrays.copyOfRange(args, 2, args.length)); return; }
    if (args[0].equals("bench")) {
      bench(args[1], args[2], Integer.parseInt(args[3]), Double.parseDouble(args[4]),
          Arrays.copyOfRange(args, 5, args.length));
      return;
    }
    System.err.println("unknown mode: " + args[0]);
    System.exit(2);
  }

  private static void check(String grammar, String[] files) throws IOException {
    Entry entry = entry(grammar);
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
          entry.parse(CharStreams.fromString(text), true);
        } catch (ParseCancellationException e) {
          verdict = "reject";
          reason = shorten(describe(e));
        } catch (RuntimeException | StackOverflowError e) {
          verdict = "reject";
          reason = shorten(e.getClass().getSimpleName() + ":" + e.getMessage());
        }
      }
      out.append(file).append('\t').append(verdict).append('\t').append(reason).append('\n');
    }
    System.out.print(out);
  }

  /** BailErrorStrategy は RecognitionException を包むので、原因側の位置を取り出す。 */
  private static String describe(ParseCancellationException e) {
    if (e.getMessage() != null) { return e.getMessage(); }
    Throwable cause = e.getCause();
    if (cause instanceof RecognitionException re && re.getOffendingToken() != null) {
      return "bail at " + re.getOffendingToken().getLine() + ":"
          + re.getOffendingToken().getCharPositionInLine() + " token '"
          + re.getOffendingToken().getText() + "'";
    }
    return cause == null ? e.getClass().getSimpleName() : cause.getClass().getSimpleName();
  }

  private static String shorten(String s) {
    String one = s.replace('\n', ' ').replace('\t', ' ').replace('\r', ' ');
    return one.length() > 90 ? one.substring(0, 90) : one;
  }

  private static void bench(String grammar, String mode, int runs, double runSeconds, String[] files)
      throws Exception {
    Entry entry = entry(grammar);
    boolean buildTree = mode.equals("tree");
    com.sun.management.ThreadMXBean threads =
        (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
    long self = Thread.currentThread().threadId();
    System.out.println("impl,input,bytes,mode,iterations,warmup,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s,alloc_bytes_per_parse");
    for (String file : files) {
      byte[] bytes = Files.readAllBytes(Path.of(file));
      String text = decodeStrictUtf8(bytes);
      Runnable once = () -> {
        ParseTree tree = entry.parse(CharStreams.fromString(text), buildTree);
        if (buildTree) {
          ParseTreeWalker.DEFAULT.walk(NoOp.INSTANCE, tree);
        }
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
      String name = Path.of(file).getFileName().toString();
      System.out.printf("ANTLR4,%s,%d,%s,%d,%d,%d,%.6f,%.6f,%.6f,%.4f,%.2f,%d%n",
          name, bytes.length, buildTree ? "parse tree+walk" : "認識のみ", iterations, warmup, runs,
          median, samples[0], samples[runs - 1],
          median * 1e3 / Math.max(1, bytes.length),
          bytes.length / (median * 1e-3) / 1e6, allocated);
    }
  }
}
