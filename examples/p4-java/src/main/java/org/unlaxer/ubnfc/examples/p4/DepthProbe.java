package org.unlaxer.ubnfc.examples.p4;
import org.unlaxer.ubnfc.examples.p4.generated.TinyExpressionP4Parser;
/** 別 JVM の -Xss と構文深度の関係を記録する。 */
public final class DepthProbe {
    public static void main(String[] args) {
        System.out.println("JVM " + java.lang.management.ManagementFactory.getRuntimeMXBean().getInputArguments());
        for (int depth : args.length == 0 ? new int[]{8, 16, 32, 64, 128} : new int[]{Integer.parseInt(args[0])}) {
            String source = "(".repeat(depth) + "1" + ")".repeat(depth);
            for (boolean ast : new boolean[]{false, true}) {
                var result = TinyExpressionP4Parser.parse(source, P4Scanners.options(ast));
                System.out.println("depth=" + depth + " ast=" + ast + " ok=" + result.ok()
                    + " consumed=" + result.consumed() + " diagnostic=" + (result.ok() ? "なし" : result.diagnostics().expected()));
            }
        }
    }
}
