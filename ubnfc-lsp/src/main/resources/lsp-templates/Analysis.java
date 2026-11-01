package __PACKAGE__;

import java.util.List;
import java.util.Map;
import __API_PACKAGE__.ParseOptions;
import __API_PACKAGE__.ParseResult;
import __API_PACKAGE__.TokenScanner;

/**
 * 1 文書ぶんの解析結果。増分解析は無いので、文書が変わるたびに丸ごと作り直す（D-060）。
 *
 * <p>2 つの解析を持つ。
 * <ul>
 *   <li><b>primary</b> — 全文を認識モード（D-023 の二段構え）で解析したもの。診断の出所。</li>
 *   <li><b>scoped</b> — scope 観測・lexical の出所。全文が通ればそれ自身。
 *       <b>失敗したときは主失敗位置で入力を切り、その前半だけを解析し直す</b>。</li>
 * </ul>
 *
 * <p><b>なぜ切って解析し直すのか</b>（段階 2 の唯一のパリティ欠落、D-062）。
 * 生成パーサの {@code ScopeStore} は失敗した枝を巻き戻すので、入口規則が失敗した文書では
 * {@code declarations()} / {@code references()} / {@code diagnostics()} が<b>全部空</b>になる。
 * 巻き戻しをやめれば観測契約（{@code docs/design/04-cursor-and-diagnostics-contract.md} と
 * D-025 の scope 観測）が変わってしまうので、<b>パーサ側は 1 行も変えず</b>、
 * 「主失敗位置までの前半を別の入力として解析し直す」ほうを採った。
 * 解析は決定的なので、この再解析の観測は「その前半だけを書いた文書」の観測と完全に一致する。
 * 失敗したときだけ 1 回余計に解析するので、通る文書の打鍵ごとの費用は段階 2 と同じ。
 *
 * <p>lexical（semantic tokens 用）は<b>要求されたときだけ</b>解析し直す。
 * {@code lexical=true} は Match 木を作らせるので、診断の経路には載せない。
 */
public final class Analysis {
    /** 生成パーサへ渡す解析要求。診断を要求しない認識モード（失敗時は生成パーサが自分で解析し直す）。 */
    private static ParseOptions options(Map<String, TokenScanner> scanners, boolean requireFullInput, boolean lexical) {
        return new ParseOptions(false, requireFullInput, lexical, true, scanners, ParseOptions.DEFAULT_MAX_DEPTH, false, false);
    }

    private final String uri;
    private final String text;
    private final Positions positions;
    private final Map<String, TokenScanner> scanners;
    private final ParseResult<?> primary;
    private final ParseResult<?> scoped;
    private final String scopedText;
    private final boolean scopedIsPrefix;
    private final int cutCp;
    private final RuntimeException failure;
    private volatile Symbols symbols;
    private volatile List<ParseResult.Lexical> lexical;
    private volatile Map<String, List<__API_PACKAGE__.Capture>> catalogs;

    private Analysis(String uri, String text, Positions positions, Map<String, TokenScanner> scanners,
                     ParseResult<?> primary, ParseResult<?> scoped, String scopedText, boolean scopedIsPrefix,
                     int cutCp, RuntimeException failure) {
        this.uri = uri; this.text = text; this.positions = positions; this.scanners = scanners;
        this.primary = primary; this.scoped = scoped; this.scopedText = scopedText;
        this.scopedIsPrefix = scopedIsPrefix; this.cutCp = cutCp; this.failure = failure;
    }

    public static Analysis of(String uri, String text, Map<String, TokenScanner> scanners) {
        var positions = new Positions(text);
        ParseResult<?> primary;
        try {
            primary = parse(text, options(scanners, true, false));
        } catch (RuntimeException error) {
            return new Analysis(uri, text, positions, scanners, null, null, text, false, -1, error);
        }
        if (primary.ok())
            return new Analysis(uri, text, positions, scanners, primary, primary, text, false, -1, null);
        int cut = primary.diagnostics() == null ? -1
            : primary.diagnostics().offset() >= 0 ? primary.diagnostics().offset() : primary.diagnostics().farthestCp();
        if (cut > 0 && cut < positions.lengthCp()) {
            String head = head(text, cut);
            try {
                var prefix = parse(head, options(scanners, false, false));
                if (prefix.ok())
                    return new Analysis(uri, text, positions, scanners, primary, prefix, head, true, cut, null);
            } catch (RuntimeException ignored) {
                // 前半の再解析は「あれば得」なので、落ちても診断は出す。
            }
        }
        return new Analysis(uri, text, positions, scanners, primary, null, text, false, -1, null);
    }

    private static ParseResult<?> parse(String source, ParseOptions options) {
        return __PARSER_CLASS__.parseEntry(LanguageServerImpl.GRAMMAR, LanguageServerImpl.ENTRY, source, options);
    }

    /** code point 単位で先頭 {@code cut} 個を取り出す。 */
    static String head(String text, int cut) {
        int index = 0;
        for (int remaining = cut; remaining > 0 && index < text.length(); remaining--)
            index += Character.charCount(text.codePointAt(index));
        return text.substring(0, index);
    }

    public String uri() { return uri; }
    public String text() { return text; }
    public Positions positions() { return positions; }
    public ParseResult<?> primary() { return primary; }
    /** 解析器自身が投げた例外（普通は null）。 */
    public RuntimeException failure() { return failure; }
    /** scope 観測・lexical の出所。無ければ null。 */
    public ParseResult<?> scoped() { return scoped; }
    /** {@link #scoped()} が「主失敗位置までの前半」なら true。診断にその旨を書く。 */
    public boolean scopedIsPrefix() { return scopedIsPrefix; }
    /** 前半を切った code point 位置（切っていなければ -1）。 */
    public int cutCp() { return cutCp; }

    public ParseResult.Scope scope() { return scoped == null ? null : scoped.scope(); }

    /** semantic tokens 用の終端一覧。要求されたときだけ解析し直して覚える。 */
    public List<ParseResult.Lexical> lexical() {
        if (lexical != null) return lexical;
        if (scoped == null) return lexical = List.of();
        try {
            lexical = parse(scopedText, options(scanners, !scopedIsPrefix, true)).lexical();
        } catch (RuntimeException error) {
            lexical = List.of();
        }
        return lexical;
    }

    public Symbols symbols() {
        if (symbols == null) symbols = Symbols.of(this);
        return symbols;
    }

    /** 外部カタログを指定したときだけ公開 capture 出現表を要求する。 */
    public Map<String, List<__API_PACKAGE__.Capture>> catalogs() {
        if (catalogs != null) return catalogs;
        if (scoped == null) return catalogs = Map.of();
        return catalogs = parse(scopedText, options(scanners, !scopedIsPrefix, false)
            .withObservations(true, false)).catalogs();
    }
}
