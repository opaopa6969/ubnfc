import java.lang.management.ManagementFactory;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Locale;

/** #4: 同一 JVM で direct / Recipe / 認識対照 / 手書きを交互に測る。 */
public final class JavaDirectAstBench {
    private static volatile Object sink;
    private record Cell(String fixture, String mode, String source, Method parse, Object options,
                        Field toggle, boolean direct, int warmup, int iterations, double[] ms, long[] bytes) {
        Object invoke() throws Exception { return parse.invoke(null, source, options); }
        void select() throws Exception { if (toggle != null) toggle.setBoolean(null, direct); }
    }
    private static final List<Cell> cells = new ArrayList<>();
    private static final int RUNS = Integer.getInteger("bench.runs", 7);
    private static void add(String fixture, String source, String pkg, String parser, Object options,
                            String mode, Field toggle, boolean direct, int warmup, int iterations) throws Exception {
        Method parse = Class.forName(pkg + "." + parser).getMethod("parse", CharSequence.class, options.getClass());
        cells.add(new Cell(fixture, mode, source, parse, options, toggle, direct, warmup, iterations,
            new double[RUNS], new long[RUNS]));
    }
    private static String load() throws Exception { return Files.readString(Path.of("/proc/loadavg")).strip(); }
    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        if (args[0].equals("p4")) {
            String pkg = "org.unlaxer.ubnfc.examples.p4.generated";
            Class<?> scanner = Class.forName("org.unlaxer.ubnfc.examples.p4.P4Scanners");
            Class<?> opts = Class.forName(pkg + ".api.ParseOptions");
            Field toggle = Class.forName(pkg + ".internal.Session").getField("directAst");
            for (String name : System.getProperty("bench.fixtures", "complex,complex-x64").split(",")) {
                String source = Files.readString(Path.of(args[1], name + ".tiny"));
                for (boolean ast : new boolean[] {true, false}) {
                    Object options = scanner.getMethod("options", boolean.class).invoke(null, ast);
                    options = opts.getMethod("withObservations", boolean.class, boolean.class).invoke(options, false, false);
                    for (boolean direct : new boolean[] {false, true})
                        add(name, source, pkg, "TinyExpressionP4Parser", options, (ast ? "ast" : "recognize") + (direct ? "-direct" : "-recipe"),
                            toggle, direct, Integer.getInteger("bench.warmup", 1000), name.endsWith("x64") ? 50 : 2000);
                }
            }
        } else if (args[0].equals("operators")) {
            String source = "1+".repeat(511) + "1";
            Class<?> opts = Class.forName("gen.operators.api.ParseOptions");
            Field toggle = Class.forName("gen.operators.internal.Session").getField("directAst");
            for (boolean ast : new boolean[] {true, false}) {
                Object options = opts.getConstructor(boolean.class, boolean.class).newInstance(ast, true);
                options = opts.getMethod("withObservations", boolean.class, boolean.class).invoke(options, false, false);
                for (boolean direct : new boolean[] {false, true})
                    add("atoms-512", source, "gen.operators", "OperatorsParser", options, (ast ? "ast" : "recognize") + (direct ? "-direct" : "-recipe"),
                        toggle, direct, 20000, 1000);
            }
            Class<?> handwritten = Class.forName("target.api.ParseOptions");
            add("atoms-512", source, "target.operators", "OperatorsParser", handwritten.getField("DEFAULT").get(null), "ast-handwritten", null, true, 20000, 1000);
        } else throw new IllegalArgumentException("p4 または operators");
        System.out.println("# java=" + System.getProperty("java.version") + " jvm=" + ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("# load=" + load());
        for (Cell cell : cells) {
            cell.select(); Object result = cell.invoke();
            if (!(Boolean) result.getClass().getMethod("ok").invoke(result)) throw new AssertionError("拒否: " + cell.mode());
            System.err.println("warmup " + cell.fixture() + " " + cell.mode());
            for (int i = 0; i < cell.warmup(); i++) sink = cell.invoke();
        }
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        long thread = Thread.currentThread().getId();
        for (int run = 0; run < RUNS; run++) {
            // run ごとに巡回の向きも変え、先行する経路を固定しない。
            for (int j = 0; j < cells.size(); j++) {
                Cell cell = cells.get(run % 2 == 0 ? j : cells.size() - j - 1);
                cell.select(); long before = threads.getThreadAllocatedBytes(thread), start = System.nanoTime();
                for (int i = 0; i < cell.iterations(); i++) sink = cell.invoke();
                cell.ms()[run] = (System.nanoTime() - start) / (1e6 * cell.iterations());
                cell.bytes()[run] = (threads.getThreadAllocatedBytes(thread) - before) / cell.iterations();
            }
            System.out.println("# run=" + (run + 1) + " load=" + load());
        }
        System.out.println("fixture,mode,warmup,iterations,runs,median_ms,min_ms,max_ms,bytes,run_ms");
        for (Cell cell : cells) {
            double[] ms = cell.ms().clone(); Arrays.sort(ms);
            long[] bytes = cell.bytes().clone(); Arrays.sort(bytes);
            System.out.printf("%s,%s,%d,%d,%d,%.6f,%.6f,%.6f,%d,\"%s\"%n", cell.fixture(), cell.mode(), cell.warmup(), cell.iterations(), RUNS,
                ms[RUNS / 2], ms[0], ms[RUNS - 1], bytes[RUNS / 2], Arrays.toString(cell.ms()));
        }
        System.out.println("# load=" + load());
    }
}
