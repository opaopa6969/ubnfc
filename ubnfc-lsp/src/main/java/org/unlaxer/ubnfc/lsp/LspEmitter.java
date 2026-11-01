package org.unlaxer.ubnfc.lsp;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import org.unlaxer.ubnfc.java.ir.GrammarIr;
import org.unlaxer.ubnfc.java.ir.Validation;

/**
 * Grammar IR から、生成パーサを包む LSP サーバのソースを出す（opt-in feature、D-060）。
 *
 * <p>生成パーサ側には一切手を入れない。出力は {@code --package} で指定した<b>別パッケージ</b>に
 * 追加されるだけなので、この emitter を動かしても
 * {@code org.unlaxer.ubnfc.java.Main} が書いたファイルは 1 byte も変わらない
 * （{@code ParserUnchangedTest} がそれを検査する）。
 *
 * <p>段階 3 の capability: 診断 + definition / references / document symbols /
 * semantic tokens / completion / hover / TextMate 文法。DAP は同じ opt-in 機構で段階 4 以降。
 *
 * <p>文法ごとに変わる表（規則の {@code @doc}、semantic token の種別、宣言・参照を生む
 * capture site、keyword の一覧）は {@code Meta.java} に<b>生成時に畳み込む</b>。
 * 実行時に IR を読まないので、VSIX に IR を同梱しなくてよい。
 */
public final class LspEmitter {
    /** 生成される server が使う lsp4j の版。{@code ubnfc-lsp/pom.xml} の {@code lsp4j.version} と揃える。 */
    public static final String LSP4J_VERSION = "0.23.1";

    /** semantic tokens の legend。控えめに 5 種だけ（旧スタックの p4 は AST 型で細かく分けるが、文法から出せるのはここまで）。 */
    public static final List<String> TOKEN_TYPES = List.of("keyword", "operator", "string", "number", "variable");

    private static final List<String> TEMPLATES = List.of(
        "Positions", "Meta", "Analysis", "Symbols", "Diagnostics", "DocumentStore",
        "Completion", "Hovers", "SemanticTokens", "FoldingRanges", "LanguageServerImpl", "ServerMain");

    private final GrammarIr ir;
    private final String pkg;
    private final String parserPackage;
    private final String grammar;
    private final String entry;
    private final String parserClass;
    private final String languageId;
    private final String scannerClass;
    private final String serverName;
    private final String serverVersion;

    public LspEmitter(GrammarIr ir, String packageName, String parserPackage, String languageId,
                      String scannerClass, String serverName, String serverVersion) {
        Validation.validate(ir);
        this.ir = ir;
        var unit = ir.grammarUnits().stream().filter(g -> g.id().equals(ir.entry().grammarId())).findFirst()
            .orElseThrow(() -> new IllegalArgumentException("entry の grammarUnit がありません"));
        this.grammar = unit.name();
        Validation.identifier(this.grammar);
        this.entry = localName(ir.entry().ruleId());
        Validation.identifier(this.entry);
        this.parserPackage = parserPackage != null ? parserPackage
            : unit.package_() != null ? unit.package_() : org.unlaxer.ubnfc.java.emit.JavaEmitter.defaultPackage(grammar);
        this.pkg = packageName != null ? packageName : this.parserPackage + ".lsp";
        for (String part : this.pkg.split("\\.", -1)) Validation.identifier(part);
        for (String part : this.parserPackage.split("\\.", -1)) Validation.identifier(part);
        this.parserClass = this.parserPackage + "." + grammar + "Parser";
        this.languageId = languageId != null ? languageId : grammar.toLowerCase(Locale.ROOT);
        this.scannerClass = scannerClass != null ? scannerClass : "";
        this.serverName = serverName != null ? serverName : grammar + " language server (ubnfc)";
        this.serverVersion = serverVersion != null ? serverVersion : "0.1.0";
    }

    private static String localName(String ruleId) {
        int index = ruleId.lastIndexOf("::");
        return index < 0 ? ruleId : ruleId.substring(index + 2);
    }

    public String packageName() { return pkg; }
    /** 生成パーサのパッケージ（{@code org.unlaxer.ubnfc.java.Main} の {@code --package}）。 */
    public String parserPackage() { return parserPackage; }
    /** 生成パーサの入口クラス（FQCN）。 */
    public String parserClass() { return parserClass; }
    public String languageId() { return languageId; }
    public String grammar() { return grammar; }
    public String entry() { return entry; }
    public String serverMainClass() { return pkg + ".ServerMain"; }
    public GrammarIr ir() { return ir; }

    /** 出力する相対パス → ソース。 */
    public Map<String, String> sources() throws IOException {
        var result = new TreeMap<String, String>();
        String prefix = pkg.replace('.', '/') + "/";
        for (String name : TEMPLATES) result.put(prefix + name + ".java", fill(resource(name)));
        return Collections.unmodifiableMap(result);
    }

    public void write(Path output) throws IOException {
        for (var file : sources().entrySet()) {
            Path path = output.resolve(file.getKey());
            Files.createDirectories(path.getParent());
            Files.writeString(path, file.getValue(), StandardCharsets.UTF_8);
        }
    }

    private String fill(String template) {
        return template
            .replace("__PACKAGE__", pkg)
            .replace("__API_PACKAGE__", parserPackage + ".api")
            .replace("__RT_PACKAGE__", parserPackage + ".rt")
            .replace("__PARSER_CLASS__", parserClass)
            .replace("__GRAMMAR__", grammar)
            .replace("__ENTRY__", entry)
            .replace("__LANGUAGE_ID__", languageId)
            .replace("__SCANNER_CLASS__", scannerClass)
            .replace("__SERVER_NAME__", serverName)
            .replace("__SERVER_VERSION__", serverVersion)
            .replace("__SOURCE__", "ubnfc")
            .replace("__TOKEN_TYPES__", list(TOKEN_TYPES))
            .replace("__RULE_DOC__", pairs(ruleDocs()))
            .replace("__RULE_DESCRIPTION__", pairs(ruleDescriptions()))
            .replace("__RULE_KIND__", pairs(ruleKinds()))
            .replace("__TOKEN_TYPE__", pairs(tokenTypes()))
            .replace("__DECLARE_SITES__", list(declareSites()))
            .replace("__USE_SITES__", list(useSites()))
            .replace("__KEYWORDS__", list(keywords()))
            .replace("__TRIVIA_LITERALS__", list(triviaLiterals()))
            .replace("__COMMENT_DELIMITERS__", commentDelimiters());
    }

    // ---- IR から畳み込む表 -------------------------------------------------

    /** 規則 → {@code @doc('…')} の文。IR の {@code semantics.rules[].annotations} に既に載っている。 */
    public Map<String, String> ruleDocs() {
        var result = new LinkedHashMap<String, String>();
        for (var rule : ir.semantics().rules())
            for (var annotation : rule.annotations())
                if (annotation.name().equals("doc") && annotation.arguments().get("text") instanceof String text && !text.isBlank())
                    result.put(rule.symbolId(), text);
        return result;
    }

    /** 規則 → {@code @declares(symbol, description)} の説明文。hover の第 2 の出所。 */
    public Map<String, String> ruleDescriptions() {
        var result = new LinkedHashMap<String, String>();
        for (var rule : ir.semantics().rules())
            for (var annotation : rule.annotations())
                if (annotation.name().equals("declares")
                    && annotation.arguments().get("description") instanceof String text && !text.isBlank())
                    result.put(rule.symbolId(), text);
        return result;
    }

    /** 規則 → 宣言される記号の種類（hover と document symbol の表示に使う）。 */
    public Map<String, String> ruleKinds() {
        var result = new LinkedHashMap<String, String>();
        for (var effect : ir.semantics().scope().effects())
            for (var event : effect.events())
                if (event.action().equals("declare")) result.putIfAbsent(effect.ruleId(), localName(effect.ruleId()));
        return result;
    }

    /**
     * 式 → semantic token の種別。{@code ParseResult.lexical()} は終端ごとに {@code exprId} を返すので、
     * 生成時にこの写像を畳み込んでおけば実行時は表引きだけで済む。
     */
    public Map<String, String> tokenTypes() {
        var tokens = new LinkedHashMap<String, GrammarIr.TokenSpec>();
        for (var token : ir.semantics().tokens()) tokens.put(token.symbolId(), token.spec());
        var result = new LinkedHashMap<String, String>();
        for (var expression : ir.semantics().expressions()) {
            var node = expression.node();
            String type = switch (node.kind()) {
                case "literal" -> isWord(node.value()) ? "keyword" : "operator";
                case "tokenRef" -> tokenType(tokens.get(node.symbolId()));
                default -> null;
            };
            if (type != null) result.put(expression.id(), type);
        }
        return result;
    }

    private static String tokenType(GrammarIr.TokenSpec spec) {
        if (spec == null) return "variable";
        if ("until".equals(spec.kind())) return "string";
        if ("builtin".equals(spec.kind())) {
            return switch (String.valueOf(spec.which())) {
                case "Number", "Digit", "UnsignedInteger" -> "number";
                case "SingleQuoted", "DoubleQuoted", "Quoted" -> "string";
                default -> "variable";
            };
        }
        // extern / regex / word: 先頭文字が引用符だけなら文字列リテラル扱い。
        if (spec.firstChars() != null && !spec.firstChars().isEmpty()
            && spec.firstChars().stream().allMatch(range -> quote(range.start()) && quote(range.end()))) return "string";
        return "variable";
    }

    private static boolean quote(Integer codePoint) { return codePoint != null && (codePoint == '"' || codePoint == '\''); }

    /** {@code @declares} が読む capture site を持つ式。{@code lexical()} の {@code exprId} と直接照合できる。 */
    public Set<String> declareSites() { return sites("declare"); }

    /** {@code @backref} / {@code use} が読む capture site を持つ式。 */
    public Set<String> useSites() { return sites("use"); }

    private Set<String> sites(String action) {
        var result = new LinkedHashSet<String>();
        for (var effect : ir.semantics().scope().effects())
            for (var event : effect.events())
                if (event.action().equals(action) && event.captureSiteId() != null)
                    result.add(event.captureSiteId().replaceAll("/capture/\\d+$", ""));
        return result;
    }

    /** completion と TextMate 文法で使う keyword（語として読める literal）。 */
    public List<String> keywords() {
        var result = new java.util.TreeSet<String>();
        for (var expression : ir.semantics().expressions())
            if (expression.node().kind().equals("literal") && isWord(expression.node().value()))
                result.add(expression.node().value());
        return List.copyOf(result);
    }

    /** trivia（コメント）の区切り。文法上はここに書けるが、補完候補としては雑音なので外す。 */
    public Set<String> triviaLiterals() {
        var result = new LinkedHashSet<String>();
        for (var policy : ir.semantics().trivia().policies())
            for (var delimiter : policy.effectiveDelimiterSet()) {
                if (delimiter.open() != null) result.add(delimiter.open());
                if (delimiter.close() != null) result.add(delimiter.close());
            }
        return result;
    }

    /** trivia の順序付きコメント区切り。パーサと同じく最初の close で閉じる。 */
    private String commentDelimiters() {
        var result = new LinkedHashSet<String>();
        for (var policy : ir.semantics().trivia().policies())
            for (var delimiter : policy.effectiveDelimiterSet())
                if (("lineComment".equals(delimiter.kind()) || "blockComment".equals(delimiter.kind()))
                    && delimiter.open() != null && !delimiter.open().isEmpty())
                    result.add("new CommentDelimiter(" + q(delimiter.open()) + ", "
                        + (delimiter.close() == null ? "null" : q(delimiter.close())) + ")");
        return String.join(", ", result);
    }

    /** 記号でない literal（演算子・区切り）。TextMate 文法の operator パターンに使う。 */
    public List<String> operators() {
        var result = new java.util.TreeSet<String>(java.util.Comparator
            .comparingInt(String::length).reversed().thenComparing(java.util.Comparator.naturalOrder()));
        for (var expression : ir.semantics().expressions())
            if (expression.node().kind().equals("literal") && !isWord(expression.node().value())
                && expression.node().value() != null && !expression.node().value().isBlank())
                result.add(expression.node().value());
        return List.copyOf(result);
    }

    static boolean isWord(String value) {
        if (value == null || value.isEmpty()) return false;
        if (!Character.isLetter(value.codePointAt(0)) && value.codePointAt(0) != '_') return false;
        return value.codePoints().allMatch(point -> Character.isLetterOrDigit(point) || point == '_');
    }

    // ---- Java ソースへの埋め込み -------------------------------------------

    private static String pairs(Map<String, String> table) {
        var out = new ArrayList<String>();
        table.forEach((key, value) -> out.add(q(key) + ", " + q(value)));
        return join(out);
    }

    private static String list(java.util.Collection<String> values) {
        var out = new ArrayList<String>();
        for (String value : values) out.add(q(value));
        return join(out);
    }

    private static String join(List<String> items) {
        if (items.isEmpty()) return "";
        var out = new StringBuilder();
        for (int i = 0; i < items.size(); i++) {
            out.append(i == 0 ? "\n        " : ",\n        ").append(items.get(i));
        }
        return out.toString();
    }

    /** Java の文字列リテラル。placeholder 検査に引っかからないよう {@code __} は分割して書く。 */
    static String q(String value) {
        if (value == null) return "null";
        var out = new StringBuilder("\"");
        value.codePoints().forEach(point -> {
            switch (point) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                default -> {
                    if (point < 0x20 || point == 0x7f) out.append(String.format("\\u%04x", point));
                    else if (point == '_' && out.length() > 0 && out.charAt(out.length() - 1) == '_')
                        out.append("\\u005f");
                    else out.appendCodePoint(point);
                }
            }
        });
        return out.append('"').toString();
    }

    private static String resource(String name) throws IOException {
        try (var stream = LspEmitter.class.getResourceAsStream("/lsp-templates/" + name + ".java")) {
            if (stream == null) throw new IOException("Missing bundled template " + name);
            return new String(stream.readAllBytes(), StandardCharsets.UTF_8);
        }
    }
}
