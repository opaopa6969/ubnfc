package org.unlaxer.tinyexpression.p4.ubnfc;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;
import org.unlaxer.tinyexpression.p4.P4PreferredAstMapper;
import org.unlaxer.tinyexpression.parser.ExpressionTypes;

class SmokeTest {
    @Test
    void sameAstForSimpleFormula() {
        String formula = "if($age >= 20){100}else{0}";
        P4PreferredAstMapper.ParsedAst old =
            P4PreferredAstMapper.parseDetailed(formula, ExpressionTypes._float);
        UbnfcP4PreferredAstMapper.ParsedAst fresh =
            UbnfcP4PreferredAstMapper.parseDetailed(formula, ExpressionTypes._float);
        assertEquals(old.selectionMode(), fresh.selectionMode());
        assertEquals(CanonicalAst.json(old.ast(), old.sourceText()),
            CanonicalAst.json(fresh.ast(), fresh.sourceText()));
    }
}
