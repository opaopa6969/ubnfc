// scripts/generate-shadow.py が UbnfcP4PreferredAstMapper から生成する。手で編集しない。
// classpath 上でこのクラスを tinyexpression の jar より前に置くと、
// FormulaInfo -> DslGeneratedAstJavaEmitter -> JavaCodeCalculatorV3 -> javac の経路が
// そのまま ubnfc 生成パーサで動く（tinyexpression は無変更）。
package org.unlaxer.tinyexpression.p4;

import java.util.List;
import java.util.Objects;
import org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4AST;
import org.unlaxer.tinyexpression.p4.ubnfc.UbnfcP4Parse;
import org.unlaxer.tinyexpression.parser.ExpressionType;

/**
 * {@code org.unlaxer.tinyexpression.p4.P4PreferredAstMapper} と同じ公開面を持つ、
 * ubnfc 生成パーサ版のファサード。戻り値の {@link TinyExpressionP4AST} と
 * {@link P4SourceText} は tinyexpression 側の型そのもので、tinyexpression は無変更。
 *
 * <p>差し込みの実際は同 FQN による classpath shadow で行う
 * （{@code src/shadow/java} に同じ公開面の {@code P4PreferredAstMapper} を置く）。
 * このクラスはその実体であり、単体でも呼べる。
 *
 * <p>旧実装との差は次の 2 点だけで、いずれも意図的:
 * <ul>
 *   <li>{@code P4SourceText} の span が全ての面で code point 半開区間になる。旧実装の
 *       published-generator 互換経路（{@code P4SourceMapping#normalizeLegacySpan}）は
 *       「code point の開始 + UTF-16 の長さ」という混在表現を後から正規化しており、
 *       BMP 外の文字を含む span で 1 文字ぶん長くなる。</li>
 *   <li>解析期限は解析の前後でだけ見る。ubnfc 生成パーサは packrat + 深さ上限で、
 *       旧実装のような指数バックトラック (issue #19, #20) を起こさない。</li>
 * </ul>
 */
public final class P4PreferredAstMapper {

    private P4PreferredAstMapper() {}

    public record ParsedAst(TinyExpressionP4AST ast, String selectionMode, P4SourceText sourceText) {
        public ParsedAst {
            Objects.requireNonNull(sourceText, "sourceText");
        }

        public ParsedAst(TinyExpressionP4AST ast, String selectionMode) {
            this(ast, selectionMode, P4SourceText.lexicalOnly());
        }
    }

    /** パース期限超過。生成 P4 専用バックエンドでは明示的なパース失敗として扱う。 */
    public static final class ParseDeadlineExceededException extends RuntimeException {
        private static final long serialVersionUID = 1L;

        ParseDeadlineExceededException(String message) {
            super(message);
        }
    }

    public static TinyExpressionP4AST parse(String formula) {
        return parseDetailed(formula, null).ast();
    }

    public static TinyExpressionP4AST parse(String formula, ExpressionType preferredResultType) {
        return parseDetailed(formula, preferredResultType).ast();
    }

    public static ParsedAst parseDetailed(String formula) {
        return parseDetailed(formula, null);
    }

    public static ParsedAst parseDetailed(String formula, ExpressionType preferredResultType) {
        return wrap(run(() -> UbnfcP4Parse.parseDetailed(formula, preferredResultType)));
    }

    public static TinyExpressionP4AST parseByAstSimpleName(
            String formula, String preferredAstSimpleName) {
        return parseByAstSimpleNameDetailed(
            formula, preferredAstSimpleName, UbnfcP4Parse.defaultParseDeadlineNanos()).ast();
    }

    public static TinyExpressionP4AST parseByAstSimpleName(
            String formula, String preferredAstSimpleName, long deadlineNanos) {
        return parseByAstSimpleNameDetailed(formula, preferredAstSimpleName, deadlineNanos).ast();
    }

    public static ParsedAst parseByAstSimpleNameDetailed(
            String formula, String preferredAstSimpleName, long deadlineNanos) {
        return wrap(run(() -> UbnfcP4Parse.parseByAstSimpleNameDetailed(
            formula, preferredAstSimpleName, deadlineNanos)));
    }

    public static TinyExpressionP4AST parseByAstSimpleNames(
            String formula, List<String> candidates, long deadlineNanos) {
        return parseByAstSimpleNamesDetailed(formula, candidates, deadlineNanos).ast();
    }

    public static ParsedAst parseByAstSimpleNamesDetailed(
            String formula, List<String> candidates, long deadlineNanos) {
        return wrap(run(() -> UbnfcP4Parse.parseByAstSimpleNamesDetailed(
            formula, candidates, deadlineNanos)));
    }

    /** 旧実装と同じく、生成パーサは式断片をそのまま受けるので書き換えない。 */
    public static String normalizeExpressionSnippetForParsing(String formula) {
        return formula == null ? "" : formula;
    }

    /** 括弧付きスライス受け手の書き換えを外した後も残る互換入口。 */
    public static String normalizeParenthesizedSliceReceivers(String formula) {
        return formula;
    }

    public static List<String> preferredAstSimpleNames(String formula) {
        return UbnfcP4Parse.preferredAstSimpleNames(formula, null);
    }

    public static List<String> preferredAstSimpleNames(
            String formula, ExpressionType preferredResultType) {
        return UbnfcP4Parse.preferredAstSimpleNames(formula, preferredResultType);
    }

    public static List<String> astEvaluatorCandidateAstSimpleNames(
            String formula, ExpressionType preferredResultType) {
        return UbnfcP4Parse.astEvaluatorCandidateAstSimpleNames(formula, preferredResultType);
    }

    public static List<String> generatedValueCandidateAstSimpleNames(
            String formula, ExpressionType preferredResultType) {
        return UbnfcP4Parse.generatedValueCandidateAstSimpleNames(formula, preferredResultType);
    }

    public static List<String> declarationCandidateAstSimpleNames(
            String formula, ExpressionType preferredResultType) {
        return UbnfcP4Parse.declarationCandidateAstSimpleNames(formula, preferredResultType);
    }

    private static ParsedAst wrap(UbnfcP4Parse.Result result) {
        return new ParsedAst(result.ast(), result.selectionMode(), result.sourceText());
    }

    /** 期限超過だけは、このクラスが公開している例外型で投げ直す。 */
    private static UbnfcP4Parse.Result run(java.util.function.Supplier<UbnfcP4Parse.Result> parse) {
        try {
            return parse.get();
        } catch (UbnfcP4Parse.ParseDeadlineExceededException exceeded) {
            throw new ParseDeadlineExceededException(exceeded.getMessage());
        }
    }
}
