package p4antlr;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.antlr.v4.runtime.BailErrorStrategy;
import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.antlr.v4.runtime.LexerNoViableAltException;
import org.antlr.v4.runtime.Token;
import org.antlr.v4.runtime.atn.DecisionInfo;
import org.antlr.v4.runtime.atn.PredictionMode;
import org.antlr.v4.runtime.misc.ParseCancellationException;
import org.antlr.v4.runtime.tree.ParseTree;

/**
 * Acceptance + speed driver for the ANTLR-idiomatic grammar
 * ({@code TinyExpressionP4Idiomatic.g4}): the same driver as {@link P4AntlrDriver},
 * against the grammar whose four type towers are one {@code expression} rule.
 * It is a separate class so that the 1:1 port's driver stays byte-identical to the
 * one PR #38 measured.
 *
 * <p>Two-stage parsing (the standard ANTLR idiom): SLL with {@link BailErrorStrategy}
 * first, and on a bail a retry with full LL, again with {@code BailErrorStrategy}.
 * A failure of the LL stage is a genuine rejection. No error recovery is ever used,
 * so "accepted" means "parsed with zero errors and consumed every token".
 *
 * <p>The grammar is syntax only: P4's scope annotations have no ANTLR counterpart,
 * so nothing here resolves names.
 *
 * <pre>
 *   java p4antlr.P4AntlrIdiomaticDriver accept &lt;fixtureDir&gt;
 *   java p4antlr.P4AntlrIdiomaticDriver bench  &lt;fixtureDir&gt; &lt;runs&gt; &lt;warmup&gt; &lt;name[:entry]&gt;...
 * </pre>
 */
public final class P4AntlrIdiomaticDriver {

    /** Outcome of one parse. */
    public record Outcome(boolean ok, boolean sllSufficed, int tokens, String message) {}

    private P4AntlrIdiomaticDriver() {}

    private static TinyExpressionP4IdiomaticParser newParser(String text) {
        CharStream input = CharStreams.fromString(text);
        TinyExpressionP4IdiomaticLexer lexer = new TinyExpressionP4IdiomaticLexer(input);
        lexer.removeErrorListeners();
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        TinyExpressionP4IdiomaticParser parser = new TinyExpressionP4IdiomaticParser(tokens);
        parser.removeErrorListeners();
        parser.setErrorHandler(new BailErrorStrategy());
        return parser;
    }

    private static ParseTree invoke(TinyExpressionP4IdiomaticParser parser, String entry) {
        // The idiomatic grammar has no booleanExpression rule; "BooleanExpression" and
        // "Expression" both mean the unified expression entry, so the two grammars can
        // be driven from the same fixture:entry specifications.
        return "Formula".equals(entry) ? parser.formula() : parser.expression();
    }

    /**
     * Prediction stages. {@code two-stage} (the default) is the standard ANTLR idiom:
     * SLL first, LL on a bail. For this grammar SLL alone suffices on all 16 fixtures,
     * so {@code sll} is a meaningful mode here (it is not for the 1:1 port).
     * Selected with {@code -Dp4antlr.mode=two-stage|ll|sll}.
     */
    private static final PredictionMode[] STAGES = switch (System.getProperty("p4antlr.mode", "two-stage")) {
        case "ll" -> new PredictionMode[] {PredictionMode.LL};
        case "sll" -> new PredictionMode[] {PredictionMode.SLL};
        default -> new PredictionMode[] {PredictionMode.SLL, PredictionMode.LL};
    };

    /** One parse at {@code entry} ("Formula" or "BooleanExpression"). */
    public static Outcome parse(String text, String entry, boolean buildTree) {
        PredictionMode last = STAGES[STAGES.length - 1];
        for (PredictionMode mode : STAGES) {
            TinyExpressionP4IdiomaticParser parser = newParser(text);
            parser.getInterpreter().setPredictionMode(mode);
            parser.setBuildParseTree(buildTree);
            try {
                invoke(parser, entry);
                // formula ends in EOF; the expression entry does not, so check it here.
                Token la = parser.getInputStream().LT(1);
                if (la != null && la.getType() != Token.EOF) {
                    if (mode != last) {
                        continue;
                    }
                    return new Outcome(false, false, parser.getInputStream().size(),
                            "trailing input at token " + la.getTokenIndex());
                }
                return new Outcome(true, mode == PredictionMode.SLL,
                        parser.getInputStream().size(), null);
            } catch (ParseCancellationException | LexerNoViableAltException e) {
                if (mode != last) {
                    continue;
                }
                return new Outcome(false, false, -1, String.valueOf(e));
            }
        }
        throw new IllegalStateException("unreachable");
    }

    private static String read(Path dir, String name) throws IOException {
        return Files.readString(dir.resolve(name + ".tiny"), StandardCharsets.UTF_8);
    }

    private static final String[] FIXTURES = {
        "complex", "complex-x4", "complex-x16", "complex-x64",
        "flat-arithmetic", "large-match",
        "valid-basic", "valid-multiline", "valid-unicode",
        "comparison-heavy", "comparison-heavy-x4", "comparison-heavy-x16", "comparison-heavy-x64",
        "complex-half", "complex-tail", "invalid-syntax",
    };

    private static void accept(Path dir, List<String> names) throws IOException {
        System.out.println("fixture\tbytes\tentry\tantlr_ok\tsll_sufficed\tdetail");
        for (String name : names.isEmpty() ? Arrays.asList(FIXTURES) : names) {
            String text = read(dir, name);
            int bytes = text.getBytes(StandardCharsets.UTF_8).length;
            for (String entry : name.startsWith("comparison-heavy")
                    ? new String[] {"Formula", "BooleanExpression"}
                    : new String[] {"Formula"}) {
                Outcome o = parse(text, entry, true);
                System.out.printf("%s\t%d\t%s\t%s\t%s\t%s%n",
                        name, bytes, entry, o.ok(), o.sllSufficed(),
                        o.message() == null ? "" : o.message().replace('\n', ' '));
            }
        }
    }

    /** Upper bound on the warmup loop's wall time; -Dp4antlr.warmupSeconds overrides. */
    private static final long WARMUP_NANOS =
            (long) (Double.parseDouble(System.getProperty("p4antlr.warmupSeconds", "10")) * 1e9);

    private static void bench(Path dir, int runs, int warmup, List<String> cases) throws IOException {
        System.out.println("tool\tfixture\tbytes\tentry\tmode\truns\twarmup\titers\tmedian_ms\tmin_ms\tload1");
        for (String spec : cases) {
            int colon = spec.indexOf(':');
            String name = colon < 0 ? spec : spec.substring(0, colon);
            String entry = colon < 0 ? "Formula" : spec.substring(colon + 1);
            String text = read(dir, name);
            int bytes = text.getBytes(StandardCharsets.UTF_8).length;
            for (boolean tree : new boolean[] {false, true}) {
                Outcome first = parse(text, entry, tree);
                if (!first.ok()) {
                    throw new IllegalStateException(name + ": rejected");
                }
                // JIT warmup first; only then size the run. Sizing from the cold parse
                // (ATN construction + interpreted bytecode) would give every run 3
                // iterations, which is far too few once the parser is warm.
                // The warmup is bounded in wall time as well as in count: a single
                // parse of this grammar can cost seconds at the Formula entry.
                long warmStart = System.nanoTime();
                int warmed = 0;
                while (warmed < warmup && System.nanoTime() - warmStart < WARMUP_NANOS) {
                    parse(text, entry, tree);
                    warmed++;
                }
                long t0 = System.nanoTime();
                parse(text, entry, tree);
                double per = Math.max((System.nanoTime() - t0) / 1e9, 1e-9);
                int iters = (int) Math.max(3, Math.min(20000, 0.5 / per));
                double[] samples = new double[runs];
                for (int r = 0; r < runs; r++) {
                    long start = System.nanoTime();
                    for (int i = 0; i < iters; i++) {
                        parse(text, entry, tree);
                    }
                    samples[r] = (System.nanoTime() - start) / 1e6 / iters;
                }
                Arrays.sort(samples);
                System.out.printf("antlr-idiomatic(%s)\t%s\t%d\t%s\t%s\t%d\t%d\t%d\t%.4f\t%.4f\t%s%n",
                        System.getProperty("p4antlr.mode", "two-stage"),
                        name, bytes, entry, tree ? "tree" : "recognition",
                        runs, warmed, iters, samples[runs / 2], samples[0], loadAverage());
                System.out.flush();
            }
        }
    }

    /**
     * ANTLR's own decision profiler ({@link org.antlr.v4.runtime.Parser#setProfile}).
     * Profiling itself costs about an order of magnitude, so only the *attribution*
     * between decisions is meaningful, not the absolute milliseconds.
     */
    private static void profile(Path dir, List<String> cases) throws IOException {
        System.out.println("fixture\tdecision\trule\tinvocations\tll_fallback\tpredict_ms\tll_maxlook\tambiguities\tctx_sensitivities");
        for (String spec : cases) {
            int colon = spec.indexOf(':');
            String name = colon < 0 ? spec : spec.substring(0, colon);
            String entry = colon < 0 ? "Formula" : spec.substring(colon + 1);
            String text = read(dir, name);
            TinyExpressionP4IdiomaticParser parser = newParser(text);
            parser.getInterpreter().setPredictionMode(PredictionMode.LL);
            parser.setProfile(true);
            long start = System.nanoTime();
            invoke(parser, entry);
            double total = (System.nanoTime() - start) / 1e6;
            DecisionInfo[] info = parser.getParseInfo().getDecisionInfo();
            DecisionInfo[] sorted = info.clone();
            Arrays.sort(sorted, (a, b) -> Long.compare(b.timeInPrediction, a.timeInPrediction));
            double predicted = 0;
            for (DecisionInfo d : info) {
                predicted += d.timeInPrediction / 1e6;
            }
            System.out.printf("# %s: profiled parse %.1f ms, prediction %.1f ms (%.0f%%)%n",
                    name, total, predicted, 100 * predicted / total);
            for (int i = 0; i < Math.min(8, sorted.length); i++) {
                DecisionInfo d = sorted[i];
                if (d.invocations == 0) {
                    break;
                }
                String rule = parser.getRuleNames()[
                        parser.getATN().decisionToState.get(d.decision).ruleIndex];
                System.out.printf("%s\t%d\t%s\t%d\t%d\t%.1f\t%d\t%d\t%d%n",
                        name, d.decision, rule, d.invocations, d.LL_Fallback,
                        d.timeInPrediction / 1e6, d.LL_MaxLook,
                        d.ambiguities.size(), d.contextSensitivities.size());
            }
            System.out.flush();
        }
    }

    private static String loadAverage() {
        try {
            return Files.readString(Path.of("/proc/loadavg")).trim().split("\\s+")[0];
        } catch (IOException e) {
            return "n/a";
        }
    }

    public static void main(String[] args) throws IOException {
        if (args.length < 2) {
            System.err.println(
                    "usage: accept <dir> [name...] | profile <dir> <name[:entry]>... "
                    + "| bench <dir> <runs> <warmup> <name[:entry]>...");
            System.exit(2);
        }
        Path dir = Path.of(args[1]);
        if ("accept".equals(args[0])) {
            accept(dir, new ArrayList<>(Arrays.asList(args).subList(2, args.length)));
        } else if ("profile".equals(args[0])) {
            profile(dir, new ArrayList<>(Arrays.asList(args).subList(2, args.length)));
        } else {
            bench(dir, Integer.parseInt(args[2]), Integer.parseInt(args[3]),
                    new ArrayList<>(Arrays.asList(args).subList(4, args.length)));
        }
    }
}
