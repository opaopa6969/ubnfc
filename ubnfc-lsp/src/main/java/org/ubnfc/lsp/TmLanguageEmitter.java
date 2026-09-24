package org.ubnfc.lsp;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import org.ubnfc.java.ir.GrammarIr;

/**
 * Grammar IR から TextMate 文法（{@code *.tmLanguage.json}）を出す。
 *
 * <p>旧スタックは {@code org.unlaxer.dsl.init.TmLanguageEmitter} が
 * {@code unlaxer-dsl init} の雛形として<b>1 度だけ</b>出し、以後は手書きで維持していた。
 * ここでは毎ビルド再生成する（先頭に「生成物・手で編集しない」と書く）。
 *
 * <p>取れるのは文法から機械的に決まるぶんだけ。
 * <ul>
 *   <li><b>keyword</b> — 語として読める literal（{@code @keyword} のような注釈は要らない）</li>
 *   <li><b>operator</b> — それ以外の literal（長いものから順に並べる）</li>
 *   <li><b>comment</b> — trivia 方針の {@code lineComment} / {@code blockComment} 区切り</li>
 *   <li><b>string</b> / <b>number</b> — token の種別（builtin の SingleQuoted / Number、引用符始まりの extern）</li>
 * </ul>
 * 意味づけ（型名・関数名・埋め込み言語）は文法から出ないので入れない。そこは LSP の
 * semantic tokens が担う（TextMate 文法はサーバが起動するまでの下地）。
 */
public final class TmLanguageEmitter {
    private final GrammarIr ir;
    private final String languageId;
    private final String name;

    public TmLanguageEmitter(GrammarIr ir, String languageId, String name) {
        this.ir = ir;
        this.languageId = languageId;
        this.name = name;
    }

    public String scopeName() { return "source." + languageId; }

    public String json() {
        var emitter = new LspEmitter(ir, null, null, languageId, null, null, null);
        var sections = new ArrayList<String>();
        var includes = new ArrayList<String>();
        String comments = comments();
        if (comments != null) { includes.add("#comments"); sections.add(q("comments") + ": " + comments); }
        String strings = strings();
        if (strings != null) { includes.add("#strings"); sections.add(q("strings") + ": " + strings); }
        String numbers = numbers();
        if (numbers != null) { includes.add("#numbers"); sections.add(q("numbers") + ": " + numbers); }
        String keywords = keywords(emitter.keywords());
        if (keywords != null) { includes.add("#keywords"); sections.add(q("keywords") + ": " + keywords); }
        String operators = operators(emitter.operators());
        if (operators != null) { includes.add("#operators"); sections.add(q("operators") + ": " + operators); }

        var out = new StringBuilder("{\n");
        out.append("  ").append(q("information_for_contributors")).append(": [")
           .append(q("ubnfc が Grammar IR から生成した TextMate 文法。手で編集しない。")).append(", ")
           .append(q("再生成: scripts/build-vsix.sh（ubnfc-lsp --emit-tmlanguage）")).append("],\n");
        out.append("  ").append(q("name")).append(": ").append(q(name)).append(",\n");
        out.append("  ").append(q("scopeName")).append(": ").append(q(scopeName())).append(",\n");
        out.append("  ").append(q("patterns")).append(": [\n");
        for (int i = 0; i < includes.size(); i++)
            out.append("    { ").append(q("include")).append(": ").append(q(includes.get(i))).append(" }")
               .append(i + 1 < includes.size() ? "," : "").append('\n');
        out.append("  ],\n");
        out.append("  ").append(q("repository")).append(": {\n");
        for (int i = 0; i < sections.size(); i++)
            out.append("    ").append(sections.get(i)).append(i + 1 < sections.size() ? "," : "").append('\n');
        out.append("  }\n}\n");
        return out.toString();
    }

    private String comments() {
        var patterns = new ArrayList<String>();
        var lines = new LinkedHashSet<String>();
        var blocks = new LinkedHashSet<List<String>>();
        for (var policy : ir.semantics().trivia().policies())
            for (var delimiter : policy.effectiveDelimiterSet()) {
                if ("lineComment".equals(delimiter.kind()) && delimiter.open() != null) lines.add(delimiter.open());
                if ("blockComment".equals(delimiter.kind()) && delimiter.open() != null && delimiter.close() != null)
                    blocks.add(List.of(delimiter.open(), delimiter.close()));
            }
        for (String open : lines)
            patterns.add("{ " + q("name") + ": " + q("comment.line." + languageId) + ", "
                + q("match") + ": " + q(regex(open) + ".*$") + " }");
        for (var block : blocks)
            patterns.add("{ " + q("name") + ": " + q("comment.block." + languageId) + ", "
                + q("begin") + ": " + q(regex(block.get(0))) + ", " + q("end") + ": " + q(regex(block.get(1))) + " }");
        return patterns.isEmpty() ? null : patterns(patterns);
    }

    private String strings() {
        var patterns = new ArrayList<String>();
        var quotes = new LinkedHashSet<Character>();
        for (var token : ir.semantics().tokens()) {
            var spec = token.spec();
            if ("builtin".equals(spec.kind())) {
                if ("SingleQuoted".equals(spec.which())) quotes.add('\'');
                if ("DoubleQuoted".equals(spec.which())) quotes.add('"');
            } else if (spec.firstChars() != null)
                for (var range : spec.firstChars())
                    if (range.start() != null && range.start().equals(range.end())
                        && (range.start() == '\'' || range.start() == '"')) quotes.add((char) range.start().intValue());
        }
        for (char quote : quotes)
            patterns.add("{ " + q("name") + ": " + q("string.quoted." + (quote == '\'' ? "single." : "double.") + languageId)
                + ", " + q("begin") + ": " + q(String.valueOf(quote)) + ", " + q("end") + ": " + q(String.valueOf(quote))
                + ", " + q("patterns") + ": [{ " + q("name") + ": " + q("constant.character.escape." + languageId)
                + ", " + q("match") + ": " + q("\\\\.") + " }] }");
        return patterns.isEmpty() ? null : patterns(patterns);
    }

    private String numbers() {
        boolean numeric = ir.semantics().tokens().stream().anyMatch(token -> "builtin".equals(token.spec().kind())
            && List.of("Number", "Digit", "UnsignedInteger").contains(String.valueOf(token.spec().which())));
        if (!numeric) return null;
        return patterns(List.of("{ " + q("name") + ": " + q("constant.numeric." + languageId) + ", "
            + q("match") + ": " + q("\\b[0-9]+(?:\\.[0-9]+)?\\b") + " }"));
    }

    private String keywords(List<String> words) {
        if (words.isEmpty()) return null;
        var alternatives = new ArrayList<String>();
        for (String word : words) alternatives.add(regex(word));
        return patterns(List.of("{ " + q("name") + ": " + q("keyword.control." + languageId) + ", "
            + q("match") + ": " + q("\\b(?:" + String.join("|", alternatives) + ")\\b") + " }"));
    }

    private String operators(List<String> tokens) {
        var alternatives = new ArrayList<String>();
        for (String token : tokens) alternatives.add(regex(token));
        if (alternatives.isEmpty()) return null;
        return patterns(List.of("{ " + q("name") + ": " + q("keyword.operator." + languageId) + ", "
            + q("match") + ": " + q("(?:" + String.join("|", alternatives) + ")") + " }"));
    }

    private static String patterns(List<String> items) {
        var out = new StringBuilder("{ " + q("patterns") + ": [\n");
        for (int i = 0; i < items.size(); i++)
            out.append("      ").append(items.get(i)).append(i + 1 < items.size() ? "," : "").append('\n');
        return out.append("    ] }").toString();
    }

    /** 正規表現のメタ文字を落とす。 */
    static String regex(String literal) {
        var out = new StringBuilder();
        literal.codePoints().forEach(point -> {
            if ("\\^$.|?*+()[]{}".indexOf(point) >= 0) out.append('\\');
            out.appendCodePoint(point);
        });
        return out.toString();
    }

    /** JSON の文字列。 */
    static String q(String value) {
        var out = new StringBuilder("\"");
        value.codePoints().forEach(point -> {
            switch (point) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                default -> {
                    if (point < 0x20) out.append(String.format("\\u%04x", point));
                    else out.appendCodePoint(point);
                }
            }
        });
        return out.append('"').toString();
    }

    /** VS Code の {@code contributes.grammars} に書く相対パス。 */
    public String fileName() { return languageId + ".tmLanguage.json"; }
}
