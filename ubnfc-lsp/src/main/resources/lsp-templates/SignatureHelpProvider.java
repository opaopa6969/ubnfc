package __PACKAGE__;

import java.util.List;

/** 呼び出しのシグネチャを言語側から供給する。 */
@FunctionalInterface
public interface SignatureHelpProvider {
    /** 現在のスナップショットでカーソルを含む最内側の呼び出しを返す。対象外・未知なら null。 */
    Call at(Analysis document, int cursorCp);

    /** クライアントが自動で signature help を要求する文字。 */
    default List<String> triggerCharacters() { return List.of(); }

    /**
     * startCp と endCp はカーソルが入れる両端の code point 位置。
     * parameters は宣言順の表示ラベル。activeParameter は 0 起点、該当なしは -1。
     */
    record Call(int startCp, int endCp, String label, List<String> parameters, int activeParameter) {
        public Call { parameters = List.copyOf(parameters); }
    }
}
