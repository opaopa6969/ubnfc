package org.unlaxer.tinyexpression.p4.ubnfc;

import static org.junit.platform.engine.discovery.DiscoverySelectors.selectClass;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import org.junit.platform.launcher.Launcher;
import org.junit.platform.launcher.LauncherDiscoveryRequest;
import org.junit.platform.launcher.core.LauncherDiscoveryRequestBuilder;
import org.junit.platform.launcher.core.LauncherFactory;
import org.junit.platform.launcher.listeners.SummaryGeneratingListener;
import org.junit.platform.launcher.listeners.TestExecutionSummary;

/**
 * tinyexpression 自身のテストを、こちらが用意した classpath 順で走らせる。
 *
 * <p>引数はテストクラス名を並べたファイル。classpath の先頭に
 * {@code target/shadow-classes} を置けば ubnfc 版 facade が、置かなければ
 * tinyexpression 本来の facade が使われる。テスト側は無変更。
 */
public final class PathProofRunner {

    public static void main(String[] args) throws Exception {
        if (args.length < 1) {
            System.err.println("使用法: PathProofRunner <テストクラス一覧ファイル> [出力 TSV]");
            System.exit(2);
        }
        List<String> classNames = Files.readAllLines(Path.of(args[0])).stream()
            .map(String::strip)
            .filter(line -> !line.isEmpty() && !line.startsWith("#"))
            .toList();
        System.out.println("facade の読み込み元: " + Class
            .forName("org.unlaxer.tinyexpression.p4.P4PreferredAstMapper")
            .getProtectionDomain().getCodeSource().getLocation());

        var rows = new ArrayList<String>();
        rows.add("test_class\ttests\tsucceeded\tfailed\taborted\tskipped\tmillis");
        long totalTests = 0;
        long totalFailed = 0;
        for (String className : classNames) {
            Launcher launcher = LauncherFactory.create();
            SummaryGeneratingListener listener = new SummaryGeneratingListener();
            LauncherDiscoveryRequest request = LauncherDiscoveryRequestBuilder.request()
                .selectors(selectClass(className)).build();
            long start = System.nanoTime();
            launcher.execute(request, listener);
            long millis = (System.nanoTime() - start) / 1_000_000L;
            TestExecutionSummary summary = listener.getSummary();
            rows.add(className + "\t" + summary.getTestsStartedCount() + "\t"
                + summary.getTestsSucceededCount() + "\t" + summary.getTestsFailedCount() + "\t"
                + summary.getTestsAbortedCount() + "\t" + summary.getTestsSkippedCount() + "\t"
                + millis);
            totalTests += summary.getTestsStartedCount();
            totalFailed += summary.getTestsFailedCount();
            System.out.printf("%-90s 実行 %3d 成功 %3d 失敗 %3d (%d ms)%n", className,
                summary.getTestsStartedCount(), summary.getTestsSucceededCount(),
                summary.getTestsFailedCount(), millis);
            summary.getFailures().forEach(failure -> System.out.println("    失敗: "
                + failure.getTestIdentifier().getDisplayName() + " -> " + failure.getException()));
        }
        System.out.println("合計 実行 " + totalTests + " 失敗 " + totalFailed);
        if (args.length > 1) {
            Path out = Path.of(args[1]);
            Files.createDirectories(out.getParent());
            Files.writeString(out, String.join("\n", rows) + "\n");
        }
    }
}
