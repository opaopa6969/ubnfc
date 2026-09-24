import java.lang.management.ManagementFactory;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * Family B（Operators / Power）の Java 実装を 1 JVM で交互に測る。
 *
 * <p>生成側は ubnfc の Java backend が同じ IR から出したもの（package gen.operators / gen.power）、
 * 手書き側は examples/handwritten-target/java（package target）。生成クラスは反射で呼ぶので、
 * このドライバ自体は生成コードに compile-time 依存しない。
 *
 * <p>計時は nanoTime、単一 thread。全ての cell（実装 × 入力 × mode）を先に warmup してから、
 * <b>1 run ごとに全 cell を巡回する</b>ので、機械の負荷変動は全 cell に同じように乗る。
 * 割当は ThreadMXBean の thread allocated bytes（計時 run と同じループで採る）。
 *
 * <p>mode（{@code bench.modes}、カンマ区切り）:
 * <ul>
 *   <li>{@code ast} — AST を作る（buildAst=true）</li>
 *   <li>{@code recognize} — 認識、観測は従来どおり全て要求（出現表・診断）</li>
 *   <li>{@code recognize-notree} — 認識、出現表を要求しない（step A: Match 木・capture・Recipe を作らない）</li>
 *   <li>{@code recognize-fast} — 認識、観測を何も要求しない（step A+B: 診断 DAG も作らない。D-023 で失敗時のみ再解析）</li>
 *   <li>上記に {@code -nomemo} を付けると memo を切る</li>
 * </ul>
 */
public final class FamilyBJavaBench {
    private static volatile Object sink;

    private record Cell(String family, String impl, String input, String source, String mode, Method parse, Object options,
                        double[] times, long[] bytes) {}

    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        int runs = Integer.getInteger("bench.runs", 7);
        double runSeconds = Double.parseDouble(System.getProperty("bench.runSeconds", "1.0"));
        int fixedIterations = Integer.getInteger("bench.iterations", 0);
        String[] modes = System.getProperty("bench.modes", "ast,recognize,recognize-notree,recognize-fast").split(",");

        String[][] operators = {
            {"1+2*3", "1+2*3"},
            {"parentheses-8", "(".repeat(8) + "1+2*3" + ")".repeat(8)},
            {"atoms-512", "1+".repeat(511) + "1"},
        };
        String[][] powers = {
            {"2^3^2", "2^3^2"},
            {"power-64", "2^".repeat(63) + "2"},
        };

        System.out.println("# java=" + System.getProperty("java.version") + " vm=" + System.getProperty("java.vm.name"));
        System.out.println("# jvm=" + ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("# loadavg=" + loadAverage());
        System.out.println("family,impl,input,bytes,mode,iterations,runs,median_ms,min_ms,max_ms,us_per_byte,allocs_per_parse,alloc_bytes_per_parse");

        var cells = new ArrayList<Cell>();
        for (String[] input : operators) {
            add(cells, "B/Operators", "ubnfc-java(生成)", input, modes, "gen.operators.OperatorsParser", "gen.operators.api.ParseOptions");
            add(cells, "B/Operators", "手書き Java(目標形)", input, modes, "target.operators.OperatorsParser", null);
        }
        for (String[] input : powers) {
            add(cells, "B/Power", "ubnfc-java(生成)", input, modes, "gen.power.PowerParser", "gen.power.api.ParseOptions");
            add(cells, "B/Power", "手書き Java(目標形)", input, modes, "target.power.PowerParser", null);
        }

        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        long thread = Thread.currentThread().getId();
        var iterations = new int[cells.size()];
        for (int i = 0; i < cells.size(); i++) {
            Cell cell = cells.get(i);
            Object check = cell.parse().invoke(null, cell.source(), cell.options());
            boolean ok = (Boolean) check.getClass().getMethod("ok").invoke(check);
            if (!ok) { System.out.println("# REJECTED " + cell.impl() + " " + cell.input() + " " + cell.mode()); iterations[i] = -1; continue; }
            for (int k = 0; k < 20000; k++) sink = cell.parse().invoke(null, cell.source(), cell.options());
            long probe = System.nanoTime();
            for (int k = 0; k < 200; k++) sink = cell.parse().invoke(null, cell.source(), cell.options());
            double per = Math.max((System.nanoTime() - probe) / 200.0, 1.0);
            iterations[i] = fixedIterations > 0 ? fixedIterations : (int) Math.max(50, Math.min(500_000, runSeconds * 1e9 / per));
            int warmup = Math.max(20000, Math.min(200_000, iterations[i] / 2));
            System.err.println("warmup " + cell.impl() + " " + cell.input() + " " + cell.mode() + " iterations=" + iterations[i]);
            for (int k = 0; k < warmup; k++) sink = cell.parse().invoke(null, cell.source(), cell.options());
        }
        // 1 run ごとに全 cell を巡回する（同一プロセス交互）。
        for (int r = 0; r < runs; r++) {
            for (int i = 0; i < cells.size(); i++) {
                if (iterations[i] < 0) continue;
                Cell cell = cells.get(i);
                long allocated = threads.getThreadAllocatedBytes(thread), start = System.nanoTime();
                for (int k = 0; k < iterations[i]; k++) sink = cell.parse().invoke(null, cell.source(), cell.options());
                cell.times()[r] = (System.nanoTime() - start) / (1e6 * iterations[i]);
                cell.bytes()[r] = (threads.getThreadAllocatedBytes(thread) - allocated) / iterations[i];
            }
            System.err.println("run " + (r + 1) + "/" + runs + " loadavg=" + loadAverage());
        }
        for (int i = 0; i < cells.size(); i++) {
            if (iterations[i] < 0) continue;
            Cell cell = cells.get(i);
            double[] sorted = Arrays.copyOf(cell.times(), runs); Arrays.sort(sorted);
            long[] sortedBytes = Arrays.copyOf(cell.bytes(), runs); Arrays.sort(sortedBytes);
            System.out.printf("%s,%s,%s,%d,%s,%d,%d,%.6f,%.6f,%.6f,%.4f,,%d%n",
                cell.family(), cell.impl(), cell.input(), cell.source().length(), cell.mode(), iterations[i], runs,
                sorted[runs / 2], sorted[0], sorted[runs - 1], sorted[runs / 2] * 1e3 / cell.source().length(), sortedBytes[runs / 2]);
        }
        System.out.println("# loadavg=" + loadAverage());
    }

    private static String loadAverage() {
        try { return java.nio.file.Files.readString(java.nio.file.Path.of("/proc/loadavg")).strip(); }
        catch (Exception ignored) { return "?"; }
    }

    private static void add(List<Cell> cells, String family, String impl, String[] input, String[] modes,
                            String parserClass, String optionsClass) throws Exception {
        Class<?> options = Class.forName(optionsClass == null ? "target.api.ParseOptions" : optionsClass);
        Method parse = Class.forName(parserClass).getMethod("parse", CharSequence.class, options);
        int runs = Integer.getInteger("bench.runs", 7);
        for (String mode : modes) {
            Object value = optionsClass == null ? handwritten(options, mode) : generated(options, mode);
            if (value == null) continue;
            cells.add(new Cell(family, impl, input[0], input[1], mode, parse, value, new double[runs], new long[runs]));
        }
    }

    /** 手書き目標形は AST / 認識の 2 つしか持たないので、mode 集合の代表 1 つずつだけ測る。 */
    private static Object handwritten(Class<?> options, String mode) throws Exception {
        if (mode.equals("ast")) return options.getField("DEFAULT").get(null);
        if (mode.equals("recognize")) return options.getField("RECOGNIZE").get(null);
        return null;
    }

    private static Object generated(Class<?> options, String mode) throws Exception {
        boolean memo = !mode.endsWith("-nomemo");
        String base = memo ? mode : mode.substring(0, mode.length() - "-nomemo".length());
        Object value = switch (base) {
            case "ast" -> options.getConstructor(boolean.class, boolean.class).newInstance(true, true);
            // 観測を全て要求する認識（PR #14 までの振る舞い）。
            case "recognize" -> options.getConstructor(boolean.class, boolean.class, boolean.class, boolean.class, Map.class)
                .newInstance(false, true, false, true, Map.of());
            case "recognize-notree" -> options.getMethod("withObservations", boolean.class, boolean.class)
                .invoke(options.getField("RECOGNIZE").get(null), false, true);
            case "recognize-fast" -> options.getField("RECOGNIZE").get(null);
            default -> throw new IllegalArgumentException("Unsupported mode: " + mode);
        };
        return memo ? value : options.getMethod("withMemo", boolean.class).invoke(value, false);
    }
}
