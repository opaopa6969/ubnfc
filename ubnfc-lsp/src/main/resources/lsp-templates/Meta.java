package __PACKAGE__;

import java.util.List;
import java.util.Map;
import java.util.Set;

/**
 * 文法から生成時に畳み込んだ表。実行時に IR を読まないので VSIX に IR を同梱しなくてよい。
 *
 * <p>ここにある値はすべて {@code ubnfc-lsp} の {@code LspEmitter} が Grammar IR から作る。
 * 手で編集しない（{@code scripts/build-vsix.sh} が毎回上書きする）。
 */
public final class Meta {
    private Meta() {}

    /** semantic tokens の legend（順序がそのまま LSP の token type 番号になる）。 */
    public static final List<String> TOKEN_TYPES = List.of(__TOKEN_TYPES__);

    /** 規則 → {@code @doc('…')} の文。hover の第 1 の出所。 */
    public static final Map<String, String> RULE_DOC = map(__RULE_DOC__);

    /** 規則 → {@code @declares(description='…')} の文。hover の第 2 の出所。 */
    public static final Map<String, String> RULE_DESCRIPTION = map(__RULE_DESCRIPTION__);

    /** 宣言を生む規則 → 表示名（document symbol / hover の「種類」）。 */
    public static final Map<String, String> RULE_KIND = map(__RULE_KIND__);

    /** 式 → semantic token 種別。{@code ParseResult.Lexical.exprId()} で引く。 */
    public static final Map<String, String> TOKEN_TYPE = map(__TOKEN_TYPE__);

    /** 宣言を読む capture site を持つ式。scope 効果が記号表を作らない文法（{@code sameRuleText}）の逃げ道。 */
    public static final Set<String> DECLARE_SITES = Set.of(__DECLARE_SITES__);

    /** 参照を読む capture site を持つ式。 */
    public static final Set<String> USE_SITES = Set.of(__USE_SITES__);

    /** close が null のものは行コメント。文法の trivia policy から生成する。 */
    public record CommentDelimiter(String open, String close) {}
    public static final List<CommentDelimiter> COMMENT_DELIMITERS = List.of(__COMMENT_DELIMITERS__);

    /** 語として読める literal。completion の候補と TextMate 文法の keyword。 */
    public static final List<String> KEYWORDS = List.of(__KEYWORDS__);

    /** trivia（コメント）の区切り。補完候補からは外す。 */
    public static final Set<String> TRIVIA_LITERALS = Set.of(__TRIVIA_LITERALS__);

    /** {@code Grammar::Rule} から {@code Rule} だけを取り出す。 */
    public static String localName(String symbolId) {
        if (symbolId == null) return null;
        int index = symbolId.lastIndexOf("::");
        return index < 0 ? symbolId : symbolId.substring(index + 2);
    }

    /** 規則の説明（{@code @doc} → {@code @declares} の順）。無ければ null。 */
    public static String documentation(String ruleId) {
        String doc = RULE_DOC.get(ruleId);
        return doc != null ? doc : RULE_DESCRIPTION.get(ruleId);
    }

    private static Map<String, String> map(String... pairs) {
        var result = new java.util.LinkedHashMap<String, String>();
        for (int i = 0; i + 1 < pairs.length; i += 2) result.put(pairs[i], pairs[i + 1]);
        return java.util.Collections.unmodifiableMap(result);
    }
}
