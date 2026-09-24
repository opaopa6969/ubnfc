package org.unlaxer.ubnfc.lsp;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import org.unlaxer.ubnfc.java.ir.GrammarIr;

/**
 * Grammar IR から、生成パーサを包む<b>文法デバッガ</b>（DAP サーバ）のソースを出す
 * （opt-in feature、issue #28 段階 4）。
 *
 * <p>{@link LspEmitter} と同じ約束で動く。
 * <ul>
 *   <li>生成パーサには一切手を入れない。出力は LSP 包みの<b>下位パッケージ</b>
 *       （既定 {@code <lsp-package>.dap}）に追加されるだけ</li>
 *   <li>{@code --emit-dap} を渡さなければ 1 byte も出ない。渡しても
 *       {@code --emit-lsp} の 11 ファイルは byte 単位で変わらない
 *       （{@code ParserUnchangedTest} が検査する）</li>
 * </ul>
 *
 * <p>DAP は LSP 包みの {@code Positions}（code point → UTF-16）・{@code Meta}・{@code Analysis} ・
 * {@code Diagnostics} を<b>そのまま使う</b>ので、{@code --emit-dap} は {@code --emit-lsp} を要求する。
 * 位置変換を 2 箇所に持たないための決定（段階 1 §5.1-1 の UTF-16 取り違えを繰り返さない）。
 *
 * <p>文法ごとに変わるのは {@code AstWalk.java} だけで、そこには IR の
 * {@code semantics.astTypes}（型・欄・宣言順）から畳み込んだ {@code switch} が入る。
 * <b>反射は使わない</b>——旧スタックの {@code DAPRuntimeEmitter} は {@code Class.getMethods()} を
 * 舐めていて子の順序が JVM 依存だった。
 */
public final class DapEmitter {
    /** 生成される DAP サーバが使う lsp4j.debug の版。VSIX の server pom と揃える。 */
    public static final String LSP4J_DEBUG_VERSION = LspEmitter.LSP4J_VERSION;

    private static final List<String> TEMPLATES =
        List.of("AstWalk", "Steps", "EvaluatorBridge", "DebugAdapter", "DapMain");

    private final GrammarIr ir;
    private final String pkg;
    private final String lspPackage;
    private final String parserPackage;
    private final String parserClass;
    private final String grammar;

    public DapEmitter(LspEmitter lsp, String packageName) {
        this.ir = lsp.ir();
        this.lspPackage = lsp.packageName();
        this.parserPackage = lsp.parserPackage();
        this.parserClass = lsp.parserClass();
        this.grammar = lsp.grammar();
        this.pkg = packageName != null ? packageName : this.lspPackage + ".dap";
        for (String part : this.pkg.split("\\.", -1)) org.unlaxer.ubnfc.java.ir.Validation.identifier(part);
    }

    public String packageName() { return pkg; }
    public String mainClass() { return pkg + ".DapMain"; }

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
            .replace("__LSP_PACKAGE__", lspPackage)
            .replace("__API_PACKAGE__", parserPackage + ".api")
            .replace("__PARSER_CLASS__", parserClass)
            .replace("__GRAMMAR__", grammar)
            .replace("__NODE_CHECK__", nodeCheck())
            .replace("__TYPE_NAME_CASES__", typeNameCases())
            .replace("__RULE_ID_CASES__", ruleIdCases())
            .replace("__FIELD_CASES__", fieldCases());
    }

    // ---- IR から畳み込む switch ---------------------------------------------

    /** union 型（{@code transparentUnion}）の Java 名。これに属するものが AST の節点。 */
    public List<String> unionTypes() {
        var out = new ArrayList<String>();
        for (var type : astTypes()) if ("transparentUnion".equals(type.kind())) out.add(java(type.id()));
        return out;
    }

    /** 記録型（{@code record}）の Java 名。ステップ列に並ぶのはこれ。 */
    public List<String> recordTypes() {
        var out = new ArrayList<String>();
        for (var type : astTypes()) if ("record".equals(type.kind())) out.add(java(type.id()));
        return out;
    }

    private List<GrammarIr.AstType> astTypes() {
        var types = ir.semantics().astTypes();
        return types == null ? List.of() : types;
    }

    private String java(String typeId) { return parserPackage + "." + typeId; }

    private String nodeCheck() {
        var checks = new ArrayList<String>();
        for (String type : unionTypes()) checks.add("value instanceof " + type);
        if (checks.isEmpty()) for (String type : recordTypes()) checks.add("value instanceof " + type);
        return checks.isEmpty() ? "false" : String.join("\n            || ", checks);
    }

    private String typeNameCases() {
        var lines = new ArrayList<String>();
        for (var type : astTypes())
            if ("record".equals(type.kind()))
                lines.add("            case " + java(type.id()) + " ignored -> " + LspEmitter.q(type.name()) + ";");
        return String.join("\n", lines);
    }

    /** AST 型 → その型を作る規則の id。{@code semantics.rules[].mappingTypeId} から引く。 */
    public Map<String, String> ruleIds() {
        var out = new LinkedHashMap<String, String>();
        for (var rule : ir.semantics().rules())
            if (rule.mappingTypeId() != null && rule.symbolId() != null)
                out.putIfAbsent(rule.mappingTypeId(), rule.symbolId());
        return out;
    }

    private String ruleIdCases() {
        var ruleIds = ruleIds();
        var lines = new ArrayList<String>();
        for (var type : astTypes()) {
            if (!"record".equals(type.kind())) continue;
            String ruleId = ruleIds.get(type.id());
            if (ruleId != null)
                lines.add("            case " + java(type.id()) + " ignored -> " + LspEmitter.q(ruleId) + ";");
        }
        return String.join("\n", lines);
    }

    private String fieldCases() {
        var lines = new ArrayList<String>();
        for (var type : astTypes()) {
            if (!"record".equals(type.kind())) continue;
            var fields = type.fields() == null ? List.<GrammarIr.AstField>of() : type.fields();
            if (fields.isEmpty()) {
                lines.add("            case " + java(type.id()) + " ignored -> { }");
                continue;
            }
            lines.add("            case " + java(type.id()) + " n -> {");
            for (var field : fields) {
                org.unlaxer.ubnfc.java.ir.Validation.identifier(field.name());
                lines.add("                out.add(new Field(" + LspEmitter.q(field.name()) + ", n." + field.name() + "()));");
            }
            lines.add("            }");
        }
        return String.join("\n", lines);
    }

    private static String resource(String name) throws IOException {
        try (var stream = DapEmitter.class.getResourceAsStream("/dap-templates/" + name + ".java")) {
            if (stream == null) throw new IOException("Missing bundled template " + name);
            return new String(stream.readAllBytes(), StandardCharsets.UTF_8);
        }
    }
}
