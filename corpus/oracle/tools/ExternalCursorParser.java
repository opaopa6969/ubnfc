package oracle;

import org.unlaxer.parser.combinator.Chain;
import org.unlaxer.parser.combinator.MatchOnly;
import org.unlaxer.parser.elementary.WordParser;

/** 外部 token のテスト用実装。😀 を消費し、続く x を先読みする。 */
public class ExternalCursorParser extends Chain {
    public ExternalCursorParser() {
        super(new WordParser("😀"), new MatchOnly(new WordParser("x")));
    }
}
