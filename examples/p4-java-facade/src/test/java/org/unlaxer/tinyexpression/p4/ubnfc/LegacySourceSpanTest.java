package org.unlaxer.tinyexpression.p4.ubnfc;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;

import java.util.Optional;
import org.junit.jupiter.api.Test;
import org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4AST;
import org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper;
import org.unlaxer.tinyexpression.p4.P4PreferredAstMapper;

/**
 * span の単位を固定する。
 *
 * <p>旧実装は「code point の開始 + UTF-16 の長さ」という混在表現を扱う経路を持ち、
 * {@code P4SourceMapping#normalizeLegacySpan} がそれを code point 半開区間へ直している
 * （公開 3.0.15 の generator 互換経路）。ubnfc 版は最初から code point しか持たない。
 *
 * <p>2026-09-23 の実装比較では非 BMP を含む入力で旧側の span 終端が 1 多かった
 * （{@code examples/p4-java/evidence/ast-differences.json} の 4 件）。
 * <b>作業ツリーの現行ビルドでは再現しない</b>。生成 mapper の大域表も snapshot も
 * code point を返し、ubnfc 版と一致する。ここはその事実を固定するためのテストで、
 * 差が戻ったら落ちる。
 */
class LegacySourceSpanTest {

    @Test
    void everySpanSourceAgreesOnCodePointsForNonBmpInput() {
        String formula = "'こんにちは😀'";
        int codePoints = formula.codePointCount(0, formula.length());
        // UTF-16 長は 9、code point 数は 8。混在表現なら終端が 9 になる。
        int[] expected = {0, codePoints};

        P4PreferredAstMapper.ParsedAst legacy = P4PreferredAstMapper.parseDetailed(formula);
        assertArrayEquals(expected, legacy.sourceText().spanOf(legacy.ast()).orElseThrow(),
            "旧 facade の snapshot");

        TinyExpressionP4AST.FormulaExpr direct = TinyExpressionP4Mapper.parse(formula);
        Optional<int[]> global = TinyExpressionP4Mapper.sourceSpanOf(direct);
        assertArrayEquals(expected, global.orElseThrow(), "生成 mapper の大域表");

        UbnfcP4PreferredAstMapper.ParsedAst fresh =
            UbnfcP4PreferredAstMapper.parseDetailed(formula);
        assertArrayEquals(expected, fresh.sourceText().spanOf(fresh.ast()).orElseThrow(),
            "ubnfc 版の snapshot");
    }
}
