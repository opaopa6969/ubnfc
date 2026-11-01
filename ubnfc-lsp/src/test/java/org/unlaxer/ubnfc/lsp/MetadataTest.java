package org.unlaxer.ubnfc.lsp;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Path;
import org.junit.jupiter.api.Test;
import org.unlaxer.ubnfc.java.ir.IrReader;

/**
 * IR から畳み込む表（{@code Meta.java} の中身）の検査。
 *
 * <p>{@code @doc} は <b>front が既に IR へ載せている</b>（{@code semantics.rules[].annotations} の
 * {@code {name: "doc", arguments: {text: …}}}）。
 * {@code docs/design/07-annotation-inventory.md} が「無視」と書いているのは
 * <b>両 backend の codegen</b> のことで、IR には最初から入っている。
 * そのため段階 3 では <b>IR スキーマにも front にも手を入れていない</b>（D-062）。
 * <p>{@code doc-demo.ubnf} / {@code doc-demo.ir.json} はそのための最小文法とその IR（{@code ubnfc ir} の実出力）。
 * VSIX を出す 2 文法（P4 と UBNF 自身）はどちらも読み取り専用の sibling repo にあり {@code @doc} を
 * 使っていないので、そちらの hover は {@code @declares(description=…)} と規則名に落ちる。
 * 表示の経路は {@code Meta.documentation()} で共通。
 */
class MetadataTest {
    private static final Path P4 = Path.of("..", "ir", "frozen",
        "tinyexpression-p4-f86ce8a5.ir.json");
    private static final Path UBNF = Path.of("..", "ir", "fixtures", "unlaxer-dsl__grammar__ubnf.ubnf.ir.json");
    private static final Path DOCS = Path.of("src", "test", "resources", "doc-demo.ir.json");

    private static LspEmitter emitter(Path ir) throws Exception {
        return new LspEmitter(IrReader.read(ir), null, null, null, null, null, null);
    }

    @Test void docAnnotationReachesTheIrAndTheGeneratedTable() throws Exception {
        var emitter = emitter(DOCS);
        var docs = emitter.ruleDocs();
        assertFalse(docs.isEmpty(), "@doc が IR から読めていない");
        assertEquals("式の入口。項を + でつないだもの。", docs.get("DocDemo::Expr"));
        assertEquals("項。識別子か数のどちらか。", docs.get("DocDemo::Term"));
        String meta = emitter.sources().get(emitter.packageName().replace('.', '/') + "/Meta.java");
        assertTrue(meta.contains("式の入口。項を + でつないだもの。"), "生成された Meta.java に @doc の文が入っていない");
        assertTrue(emitter.sources().get(emitter.packageName().replace('.', '/') + "/Hovers.java")
            .contains("Meta.documentation"), "hover が @doc を読んでいない");
    }

    @Test void p4HasNoDocButHasDeclaresDescriptions() throws Exception {
        var emitter = emitter(P4);
        assertTrue(emitter.ruleDocs().isEmpty(), "P4 の文法は @doc を使っていない（hover は @declares の説明に落ちる）");
        assertEquals("desc", emitter.ruleDescriptions().get("TinyExpressionP4::NumberVariableDeclaration"));
    }

    @Test void semanticTokenTypesComeFromTheGrammar() throws Exception {
        var types = emitter(P4).tokenTypes();
        assertEquals("keyword", types.get("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1767:1775:body/0/literal"));
        assertEquals("number", types.get("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11347:11353:body/9/tokenRef"));
        assertTrue(types.values().stream().allMatch(LspEmitter.TOKEN_TYPES::contains), types.toString());
        assertTrue(emitter(P4).keywords().contains("import"));
        assertFalse(emitter(P4).keywords().contains("("), "記号は keyword ではない");
        assertTrue(emitter(P4).operators().contains("("));
    }

    /** UBNF 自身の文法は {@code @backref} が {@code sameRuleText} なので ScopeStore に宣言が積まれない。 */
    @Test void ubnfDeclarationsComeFromCaptureSites() throws Exception {
        var emitter = emitter(UBNF);
        assertEquals(java.util.Set.of("expr:unlaxer-dsl/grammar/ubnf.ubnf:4148:4158:body/1/tokenRef"), emitter.declareSites());
        assertEquals(java.util.Set.of("expr:unlaxer-dsl/grammar/ubnf.ubnf:9381:9391:body/1/tokenRef"), emitter.useSites());
    }

    @Test void triviaDelimitersAreExcludedFromCompletion() throws Exception {
        var trivia = emitter(P4).triviaLiterals();
        assertTrue(trivia.containsAll(java.util.List.of("//", "/*", "*/")), trivia.toString());
    }

    @Test void textMateGrammarIsGeneratedFromTheIr() throws Exception {
        var ir = IrReader.read(P4);
        String json = new TmLanguageEmitter(ir, "tinyexpression-p4", "TinyExpressionP4").json();
        assertTrue(json.contains("\"scopeName\": \"source.tinyexpression-p4\""), json.substring(0, 200));
        assertTrue(json.contains("comment.line.tinyexpression-p4"), "行コメントが出ていない");
        assertTrue(json.contains("comment.block.tinyexpression-p4"), "ブロックコメントが出ていない");
        assertTrue(json.contains("string.quoted.single.tinyexpression-p4"), "文字列が出ていない");
        assertTrue(json.contains("constant.numeric.tinyexpression-p4"), "数値が出ていない");
        assertTrue(json.contains("keyword.control.tinyexpression-p4"), "keyword が出ていない");
        assertTrue(json.contains("keyword.operator.tinyexpression-p4"), "operator が出ていない");
        assertTrue(json.contains("information_for_contributors"), "生成物である印が無い");
        // 語境界つきの keyword は他の語の一部に当たらない。
        assertTrue(json.contains("\\\\b(?:"), json);
    }

    @Test void textMateEscapesRegexMetacharacters() {
        assertEquals("\\(", TmLanguageEmitter.regex("("));
        assertEquals("\\*/", TmLanguageEmitter.regex("*/"));
        assertEquals("if", TmLanguageEmitter.regex("if"));
    }
}
