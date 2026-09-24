package org.ubnfc.p4;

import java.lang.management.ManagementFactory;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Locale;
import org.ubnfc.p4.generated.TinyExpressionP4Parser;
import org.ubnfc.p4.generated.api.ParseOptions;

/**
 * 1 スレッド、nanoTime、run の中央値。割り当て量は ThreadMXBean の thread allocated bytes。
 * 全ての cell（fixture × mode）を先に warmup してから、<b>1 run ごとに全 cell を巡回する</b>ので、
 * 機械の負荷変動は全 cell に同じように乗る（同一プロセス交互 A/B）。
 *
 * <p>システムプロパティ: bench.fixtures（カンマ区切り、既定は全 6 件）、bench.modes、
 * bench.warmup（既定 2000）、bench.runs（既定 5）、bench.iterations（既定は fixture ごとの規定値）、
 * bench.entry（既定 null=Formula。comparison-heavy 系は BooleanExpression を指定する）。
 *
 * <p>mode: {@code ast} / {@code recognize}（観測を全て要求）/ {@code recognize-notree}（出現表を
 * 要求しない = Match 木・capture・Recipe を作らない）/ {@code recognize-fast}（観測を何も要求しない
 * = 診断 DAG も作らず、D-023 で失敗時だけ再解析）。末尾に {@code -nomemo} で memo を切る。
 */
public final class Benchmark {
    private static volatile Object sink;

    private record Cell(String fixture, String source, String mode, ParseOptions options, int count,
                        double[] times, long[] bytes) {}

    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        Path fixtures = Path.of(args[0]);
        String[] names = System.getProperty("bench.fixtures", "complex,complex-x4,complex-x16,complex-x64,comparison-heavy,comparison-heavy-x64").split(",");
        String[] modes = System.getProperty("bench.modes", "recognize,ast").split(",");
        int warmup = Integer.getInteger("bench.warmup", 2000);
        int runs = Integer.getInteger("bench.runs", 5);
        int fixedIterations = Integer.getInteger("bench.iterations", 0);
        String entry = System.getProperty("bench.entry");
        var threads = (com.sun.management.ThreadMXBean) ManagementFactory.getThreadMXBean();
        System.out.println("# java=" + System.getProperty("java.version") + " vm=" + System.getProperty("java.vm.name"));
        System.out.println("# jvm=" + ManagementFactory.getRuntimeMXBean().getInputArguments());
        System.out.println("# loadavg=" + loadAverage());
        System.out.println("fixture,mode,entry,warmup,iterations,run1_ms,run2_ms,run3_ms,run4_ms,run5_ms,median_ms,min_ms,alloc_bytes_per_parse");

        var cells = new ArrayList<Cell>();
        for (String fixture : names) {
            String source = Files.readString(fixtures.resolve(fixture + ".tiny"));
            for (String mode : modes) {
                ParseOptions options = options(mode);
                var check = parse(entry, source, options);
                if (!check.ok() || check.consumedCp() != source.codePointCount(0, source.length())
                    || options.buildAst() && check.ast().isEmpty()) {
                    System.out.println("# REJECTED " + fixture + " " + mode + " " + check.diagnostics());
                    continue;
                }
                int count = fixedIterations > 0 ? fixedIterations : fixture.endsWith("x64") ? 50 : fixture.endsWith("x16") ? 125 : fixture.endsWith("x4") ? 500 : 2000;
                cells.add(new Cell(fixture, source, mode, options, count, new double[runs], new long[runs]));
            }
        }
        for (Cell cell : cells) {
            System.err.println("warmup " + cell.fixture() + " " + cell.mode());
            for (int i = 0; i < warmup; i++) sink = parse(entry, cell.source(), cell.options());
        }
        long thread = Thread.currentThread().getId();
        for (int r = 0; r < runs; r++) {
            for (Cell cell : cells) {
                long allocated = threads.getThreadAllocatedBytes(thread);
                long start = System.nanoTime();
                for (int i = 0; i < cell.count(); i++) sink = parse(entry, cell.source(), cell.options());
                cell.times()[r] = (System.nanoTime() - start) / (1e6 * cell.count());
                cell.bytes()[r] = (threads.getThreadAllocatedBytes(thread) - allocated) / cell.count();
            }
            System.err.println("run " + (r + 1) + "/" + runs + " loadavg=" + loadAverage());
        }
        for (Cell cell : cells) {
            double[] sorted = cell.times().clone(); Arrays.sort(sorted);
            long[] sortedBytes = cell.bytes().clone(); Arrays.sort(sortedBytes);
            var line = new StringBuilder(cell.fixture() + "," + cell.mode() + "," + (entry == null ? "Formula" : entry) + "," + warmup + "," + cell.count());
            for (int r = 0; r < 5; r++) line.append(',').append(r < runs ? String.format("%.6f", cell.times()[r]) : "");
            line.append(',').append(String.format("%.6f", sorted[runs / 2]))
                .append(',').append(String.format("%.6f", sorted[0]))
                .append(',').append(sortedBytes[runs / 2]);
            System.out.println(line);
            System.out.flush();
        }
        System.out.println("# loadavg=" + loadAverage());
    }

    private static String loadAverage() {
        try { return Files.readString(Path.of("/proc/loadavg")).strip(); } catch (Exception ignored) { return "?"; }
    }

    static ParseOptions options(String mode) {
        boolean memo = !mode.endsWith("-nomemo");
        String base = memo ? mode : mode.substring(0, mode.length() - "-nomemo".length());
        ParseOptions options = switch (base) {
            case "ast" -> P4Scanners.options(true);
            case "recognize" -> P4Scanners.options(false);
            case "recognize-notree" -> P4Scanners.recognize().withObservations(false, true);
            case "recognize-fast" -> P4Scanners.recognize();
            default -> throw new IllegalArgumentException("Unsupported mode: " + mode);
        };
        return memo ? options : options.withMemo(false);
    }

    private static org.ubnfc.p4.generated.api.ParseResult<org.ubnfc.p4.generated.TinyExpressionP4AST> parse(String entry, String source, ParseOptions options) {
        return entry == null ? TinyExpressionP4Parser.parse(source, options) : TinyExpressionP4Parser.parseEntry("TinyExpressionP4", entry, source, options);
    }
}
