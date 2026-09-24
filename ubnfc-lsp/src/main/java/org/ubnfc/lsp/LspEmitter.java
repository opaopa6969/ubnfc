package org.ubnfc.lsp;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Collections;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TreeMap;
import org.ubnfc.java.ir.GrammarIr;
import org.ubnfc.java.ir.Validation;

/**
 * Grammar IR から、生成パーサを包む LSP サーバのソースを出す（opt-in feature、D-059）。
 *
 * <p>生成パーサ側には一切手を入れない。出力は {@code --package} で指定した<b>別パッケージ</b>に
 * 追加されるだけなので、この emitter を動かしても
 * {@code org.ubnfc.java.Main} が書いたファイルは 1 byte も変わらない
 * （{@code ParserUnchangedTest} がそれを検査する）。
 *
 * <p>段階 2 の capability は診断だけ。completion / hover / definition は段階 3。
 * DAP は同じ opt-in 機構で段階 4 以降（{@code Main} の {@code --emit-dap}）。
 */
public final class LspEmitter {
    /** 生成される server が使う lsp4j の版。{@code ubnfc-lsp/pom.xml} の {@code lsp4j.version} と揃える。 */
    public static final String LSP4J_VERSION = "0.23.1";

    private static final List<String> TEMPLATES =
        List.of("Positions", "Diagnostics", "DocumentStore", "LanguageServerImpl", "ServerMain");

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
        var unit = ir.grammarUnits().stream().filter(g -> g.id().equals(ir.entry().grammarId())).findFirst()
            .orElseThrow(() -> new IllegalArgumentException("entry の grammarUnit がありません"));
        this.grammar = unit.name();
        Validation.identifier(this.grammar);
        this.entry = localName(ir.entry().ruleId());
        Validation.identifier(this.entry);
        this.parserPackage = parserPackage != null ? parserPackage
            : unit.package_() != null ? unit.package_() : "generated." + grammar.toLowerCase(Locale.ROOT);
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
    public String languageId() { return languageId; }
    public String grammar() { return grammar; }
    public String entry() { return entry; }
    public String serverMainClass() { return pkg + ".ServerMain"; }

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
            .replace("__SOURCE__", "ubnfc");
    }

    private static String resource(String name) throws IOException {
        try (var stream = LspEmitter.class.getResourceAsStream("/lsp-templates/" + name + ".java")) {
            if (stream == null) throw new IOException("Missing bundled template " + name);
            return new String(stream.readAllBytes(), StandardCharsets.UTF_8);
        }
    }
}
