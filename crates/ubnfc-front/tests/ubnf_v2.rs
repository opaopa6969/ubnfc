//! UBNF v2: バージョン門（D-038）と左再帰ルールの脱糖（D-039）。
//!
//! v1 の意味論が凍結されていることも、ここで押さえる（`@ubnf` が無い文法は
//! 何も変わらず、候補先頭注釈も左再帰も v1 のままの扱いになる）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ubnfc_front::ir::{ExprNode, OperatorAssociativity, OperatorCanonicalForm, RecipeKind};
use ubnfc_front::pipeline::{compile, Compiled, PipelineError};

fn source(text: &str) -> Result<Compiled, PipelineError> {
    let files: BTreeMap<PathBuf, String> =
        BTreeMap::from([("/virtual/main.ubnf".into(), text.into())]);
    compile("/virtual/main.ubnf", &|path: &Path| Ok(files[path].clone()))
}

fn codes(text: &str) -> Vec<String> {
    match source(text) {
        Ok(compiled) => compiled.warnings.iter().map(|i| i.code.clone()).collect(),
        Err(PipelineError::Validation(issues) | PipelineError::Strict(issues)) => {
            issues.iter().map(|i| i.code.clone()).collect()
        }
        Err(PipelineError::Lower(errors)) => errors
            .iter()
            .map(|e| {
                let (ubnfc_front::lower::LowerError::Warning { code, .. }
                | ubnfc_front::lower::LowerError::Error { code, .. }) = e;
                code.clone()
            })
            .collect(),
        Err(other) => panic!("想定外のエラー: {other}"),
    }
}

fn rule_names(compiled: &Compiled) -> Vec<String> {
    compiled
        .ir
        .semantics
        .rules
        .iter()
        .map(|r| r.symbol_id.rsplit("::").next().unwrap().to_owned())
        .collect()
}

/// 設定は token 宣言より前に置く（構文の順序）。
fn header(settings: &str) -> String {
    format!(
        "grammar G {{\n  @package: org.example\n{settings}  @whitespace: javaStyle\n  \
         token NUMBER = NumberParser\n"
    )
}

fn grammar(version: Option<u64>, body: &str) -> String {
    let settings = version.map_or(String::new(), |v| format!("  @ubnf: {v}\n"));
    format!("{}{body}}}", header(&settings))
}

// ── §0 バージョン門 ────────────────────────────────────────────────────────

#[test]
fn language_version_defaults_to_one_and_is_carried_in_the_ir() {
    let plain = source(&grammar(None, "  @root R ::= NUMBER;\n")).unwrap();
    assert_eq!(plain.ir.language_version, 1);
    let one = source(&grammar(Some(1), "  @root R ::= NUMBER;\n")).unwrap();
    assert_eq!(one.ir.language_version, 1);
    let two = source(&grammar(Some(2), "  @root R ::= NUMBER;\n")).unwrap();
    assert_eq!(two.ir.language_version, 2);
    // v2 を宣言しても v2 の機能を使わなければ意味論・解析・計画は 1 バイトも変わらない
    // （ソース由来の sources / sha256 だけが違う）。
    assert_eq!(
        serde_json::to_value((&one.ir.semantics, &one.ir.analysis, &one.ir.plan)).unwrap(),
        serde_json::to_value((&two.ir.semantics, &two.ir.analysis, &two.ir.plan)).unwrap()
    );
    assert_eq!(plain.ir.semantics.rules.len(), one.ir.semantics.rules.len());
}

#[test]
fn unknown_or_duplicated_ubnf_versions_are_rejected() {
    assert!(codes(&grammar(Some(3), "  @root R ::= NUMBER;\n"))
        .contains(&"E-UBNF-VERSION-UNSUPPORTED".to_owned()));
    assert!(codes(&grammar(Some(0), "  @root R ::= NUMBER;\n"))
        .contains(&"E-UBNF-VERSION-UNSUPPORTED".to_owned()));
    let text = format!(
        "{}  @root R ::= NUMBER;\n}}",
        header("  @ubnf: 2\n  @ubnf: 2\n")
    );
    assert!(codes(&text).contains(&"E-UBNF-VERSION-DUPLICATE".to_owned()));
    let text = format!("{}  @root R ::= NUMBER;\n}}", header("  @ubnf: two\n"));
    assert!(codes(&text).contains(&"E-UBNF-VERSION-UNSUPPORTED".to_owned()));
}

// ── §1 左再帰の脱糖 ──────────────────────────────────────────────────────

const EXPR_V2: &str = "  @root @mapping(Binary, params=[left, op, right])\n  \
    Expr ::=\n      Expr @left MulOp @op Expr @right\n    \
    | Expr @left AddOp @op Expr @right\n    | NUMBER\n    | '(' Expr ')' ;\n  \
    AddOp ::= '+' | '-';\n  MulOp ::= '*' | '/';\n";

#[test]
fn a_left_recursive_rule_becomes_the_v1_precedence_tower() {
    let compiled = source(&grammar(Some(2), EXPR_V2)).unwrap();
    assert_eq!(
        rule_names(&compiled),
        ["Expr", "Expr__lr1", "Expr__lrBase", "AddOp", "MulOp"]
    );
    // 入口（元の名前）が最低優先＝候補の最後の演算子、最深が最初の演算子。
    let by_rule: BTreeMap<_, _> = compiled
        .ir
        .semantics
        .rules
        .iter()
        .map(|r| (r.symbol_id.rsplit("::").next().unwrap(), r))
        .collect();
    for (name, level, base) in [("Expr", 10, "Expr__lr1"), ("Expr__lr1", 20, "Expr__lrBase")] {
        let operator = by_rule[name].operator.as_ref().expect("operator metadata");
        assert_eq!(operator.associativity, OperatorAssociativity::Left);
        assert_eq!(
            operator.canonical_form,
            OperatorCanonicalForm::LeftOpRightLists
        );
        assert_eq!(operator.precedence, level);
        assert!(operator.base_rule_id.ends_with(base), "{name}");
    }
    assert!(by_rule["Expr__lrBase"].operator.is_none());
    // 各段が `Base @left { Op @op Base @right }` の正規形になっている。
    let expressions: BTreeMap<_, _> = compiled
        .ir
        .semantics
        .expressions
        .iter()
        .map(|e| (&e.id, &e.node))
        .collect();
    for name in ["Expr", "Expr__lr1"] {
        let ExprNode::Seq { items } = expressions[&by_rule[name].body] else {
            panic!("{name}: Seq ではない");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(
            expressions[&items[1]],
            ExprNode::Repeat {
                min: 0,
                max: None,
                ..
            }
        ));
    }
    // 演算子の段だけが mapping を持ち、基底の段は持たない。
    let recipes: BTreeMap<_, _> = compiled
        .ir
        .semantics
        .recipes
        .iter()
        .map(|r| (r.rule_id.rsplit("::").next().unwrap(), r.kind))
        .collect();
    assert_eq!(recipes.get("Expr"), Some(&RecipeKind::AssocLists));
    assert_eq!(recipes.get("Expr__lr1"), Some(&RecipeKind::AssocLists));
    assert_eq!(recipes.get("Expr__lrBase"), None);
}

#[test]
fn the_desugared_tower_matches_a_hand_written_v1_tower() {
    let v2 = source(&grammar(Some(2), EXPR_V2)).unwrap();
    let v1 = source(&grammar(
        None,
        "  @root @mapping(Binary, params=[left, op, right]) @leftAssoc @precedence(level=10)\n  \
         Expr ::= Term @left { AddOp @op Term @right };\n  \
         @mapping(Binary, params=[left, op, right]) @leftAssoc @precedence(level=20)\n  \
         Term ::= Factor @left { MulOp @op Factor @right };\n  \
         Factor ::= NUMBER | '(' Expr ')';\n  AddOp ::= '+' | '-';\n  MulOp ::= '*' | '/';\n",
    ))
    .unwrap();
    // AST 型（生成される record とその field の型）は完全に一致する。
    assert_eq!(v1.ir.semantics.ast_types, v2.ir.semantics.ast_types);
    // recipe は規則名だけが違い、種別・結果型・field の基数/変換は同じ。
    let strip = |ir: &ubnfc_front::ir::GrammarIr| {
        ir.semantics
            .recipes
            .iter()
            .map(|r| {
                (
                    r.kind,
                    r.result_type_id.clone(),
                    r.fields
                        .iter()
                        .map(|f| (f.field.clone(), f.cardinality, f.conversion))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(strip(&v1.ir), strip(&v2.ir));
}

#[test]
fn right_associative_alternatives_use_the_base_op_self_form() {
    let compiled = source(&grammar(
        Some(2),
        "  @root @mapping(Power, params=[left, op, right])\n  \
         Expr ::=\n      @rightAssoc Expr @left '^' @op Expr @right\n    | NUMBER ;\n",
    ))
    .unwrap();
    let rule = compiled
        .ir
        .semantics
        .rules
        .iter()
        .find(|r| r.symbol_id.ends_with("::Expr"))
        .unwrap();
    // 右結合は反復の右辺が自分自身を指す（Power.ubnf と同じ正規形）。
    let expressions: BTreeMap<_, _> = compiled
        .ir
        .semantics
        .expressions
        .iter()
        .map(|e| (&e.id, e))
        .collect();
    let ExprNode::Seq { items } = &expressions[&rule.body].node else {
        panic!("Seq ではない");
    };
    let ExprNode::Repeat { child, .. } = &expressions[&items[1]].node else {
        panic!("Repeat ではない");
    };
    let ExprNode::Seq { items: pair } = &expressions[child].node else {
        panic!("Seq ではない");
    };
    assert!(matches!(&expressions[&pair[1]].node,
        ExprNode::RuleRef { symbol_id } if symbol_id == &rule.symbol_id));
    assert_eq!(
        compiled
            .ir
            .semantics
            .recipes
            .iter()
            .find(|r| r.rule_id == rule.symbol_id)
            .map(|r| r.kind),
        Some(RecipeKind::AssocLists)
    );
    // 演算子が規則参照の右結合は第一段では拒否する（v1 の正規形の食い違い）。
    assert!(codes(&grammar(
        Some(2),
        "  @root @mapping(Power, params=[left, op, right])\n  \
         Expr ::=\n      @rightAssoc Expr @left PowOp @op Expr @right\n    | NUMBER ;\n  \
         PowOp ::= '^';\n",
    ))
    .contains(&"E-LEFTREC-RIGHTASSOC-OP-REF".to_owned()));
}

#[test]
fn per_alternative_mapping_gives_each_precedence_level_its_own_ast_class() {
    let compiled = source(&grammar(
        Some(2),
        "  @root\n  Expr ::=\n      @mapping(XorExpr, params=[left, op, right]) Expr @left '^' @op Expr @right\n    \
         | @mapping(OrExpr, params=[left, op, right]) Expr @left '|' @op Expr @right\n    | NUMBER ;\n",
    ))
    .unwrap();
    let types: BTreeMap<_, _> = compiled
        .ir
        .semantics
        .recipes
        .iter()
        .map(|r| {
            (
                r.rule_id.rsplit("::").next().unwrap(),
                r.result_type_id.clone().unwrap_or_default(),
            )
        })
        .collect();
    assert!(types["Expr"].ends_with(".OrExpr"), "{types:?}");
    assert!(types["Expr__lr1"].ends_with(".XorExpr"), "{types:?}");
}

// ── 第一段で拒否するもの ──────────────────────────────────────────────────

#[test]
fn v1_grammars_are_untouched_by_the_desugaring() {
    // 候補先頭の結合性注釈は v2 の左再帰ルール専用。
    assert!(codes(&grammar(
        None,
        "  @root Expr ::= @rightAssoc NUMBER | '(' Expr ')';\n"
    ))
    .contains(&"E-ALT-ASSOC-UNSUPPORTED".to_owned()));
    // v1 の直接左再帰は従来どおり D-016 の対象形だけが書き換わる。
    assert!(codes(&grammar(
        None,
        "  @root @mapping(Binary, params=[left, op, right])\n  \
         Expr ::= Expr @left AddOp @op Expr @right | NUMBER;\n  AddOp ::= '+' | '-';\n"
    ))
    .contains(&"E-LOWER-D016".to_owned()));
}

#[test]
fn non_binary_recursive_alternatives_are_rejected_with_a_clear_code() {
    for body in [
        // 前置（自身が末尾だけ）は演算子候補として扱わないので基底候補になり、警告が出る。
        "  @root Expr ::= Expr @left '+' @op Expr @right Expr | NUMBER;\n",
        "  @root Expr ::= Expr '?' Expr ':' Expr | NUMBER;\n",
        "  @root Expr ::= Expr | NUMBER;\n",
    ] {
        assert!(
            codes(&grammar(Some(2), body)).contains(&"E-LEFTREC-ALT-SHAPE".to_owned()),
            "{body}"
        );
    }
}

#[test]
fn unsupported_annotations_on_a_left_recursive_rule_are_rejected() {
    for annotation in [
        "@scopeTree(mode=lexical)",
        "@declares(symbol=name)",
        "@backref(name=name)",
        "@recovery(SYNC)",
        "@catalog(context='x')",
        "@leftAssoc",
        "@precedence(level=10)",
        "@predictiveChoice",
    ] {
        let body = format!("  @root {annotation}\n  Expr ::= Expr '+' Expr | NUMBER;\n");
        assert!(
            codes(&grammar(Some(2), &body)).contains(&"E-LEFTREC-ANNOTATION".to_owned()),
            "{annotation}"
        );
    }
}

#[test]
fn a_left_recursive_rule_needs_a_base_alternative_and_free_names() {
    assert!(
        codes(&grammar(Some(2), "  @root Expr ::= Expr '+' Expr;\n"))
            .contains(&"E-LEFTREC-NO-BASE".to_owned())
    );
    assert!(codes(&grammar(
        Some(2),
        "  @root Expr ::= Expr '+' Expr | NUMBER;\n  Expr__lrBase ::= NUMBER;\n"
    ))
    .contains(&"E-LEFTREC-NAME-COLLISION".to_owned()));
    assert!(codes(&grammar(
        Some(2),
        "  @root Expr ::= Expr Op? Expr | NUMBER;\n  Op ::= '+';\n"
    ))
    .contains(&"E-LEFTREC-NULLABLE-OP".to_owned()));
}

#[test]
fn indirect_left_recursion_stays_rejected_under_v2() {
    assert!(codes(&grammar(
        Some(2),
        "  @root A ::= B '+' NUMBER | NUMBER;\n  B ::= A '-' NUMBER | NUMBER;\n"
    ))
    .contains(&"E-LOWER-INDIRECT-LEFT-RECURSION".to_owned()));
}

#[test]
fn a_base_alternative_ending_in_the_rule_itself_warns_about_its_binding() {
    let issues = codes(&grammar(
        Some(2),
        "  @root Expr ::= Expr '+' Expr | '-' Expr | NUMBER;\n",
    ));
    assert!(
        issues.contains(&"W-LEFTREC-TRAILING-SELF-REF".to_owned()),
        "{issues:?}"
    );
}

// ── P4-v2（examples/p4-v2）が commit 済み P4 の IR と構造同値であること ──────

/// `examples/p4-v2/tinyexpression-p4-v2.ubnf` は tinyexpression の P4 を
/// v2 の左再帰ルールで書き直したもの（式の塔以外は 1 文字も同じ）。生成した parser の
/// 観測比較は `examples/p4-v2/scripts/compare.sh`（報告
/// `docs/reports/2026-09-23-ubnf-v2-left-recursion.md`）。ここでは codegen 無しで
/// **AST 型と recipe が塔と完全に一致する**ことだけを固定する。
#[test]
fn p4_v2_produces_the_same_ast_types_and_recipes_as_the_committed_p4_ir() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let grammar = root.join("examples/p4-v2/tinyexpression-p4-v2.ubnf");
    let declarations = ubnfc_front::ir::parse_extern_first_chars(
        &std::fs::read_to_string(root.join("scanners/first-chars.json")).unwrap(),
    )
    .unwrap();
    let mut compiled = compile(&grammar, &|path: &Path| std::fs::read_to_string(path)).unwrap();
    let (_, missing) = ubnfc_front::ir::apply_extern_first_chars(&mut compiled.ir, &declarations);
    assert!(missing.is_empty(), "{missing:?}");
    assert_eq!(compiled.ir.language_version, 2);

    let v1: ubnfc_front::ir::GrammarIr = ubnfc_front::ir::from_json(
        &std::fs::read_to_string(root.join(
            "ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(v1.language_version, 1);
    assert_eq!(v1.semantics.rules.len(), 124);
    // 塔 8 本 -> 左再帰ルール 3 本。透過的な BooleanExpression__lrBase が 1 本増える。
    assert_eq!(compiled.ir.semantics.rules.len(), 125);
    assert_eq!(v1.semantics.ast_types, compiled.ir.semantics.ast_types);

    let shape = |ir: &ubnfc_front::ir::GrammarIr, names: &[&str]| {
        names
            .iter()
            .map(|name| {
                let recipe = ir
                    .semantics
                    .recipes
                    .iter()
                    .find(|r| r.rule_id.ends_with(&format!("::{name}")))
                    .unwrap_or_else(|| panic!("recipe がありません: {name}"));
                (
                    recipe.kind,
                    recipe.result_type_id.clone(),
                    recipe
                        .fields
                        .iter()
                        .map(|f| (f.field.clone(), f.cardinality, f.conversion))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        shape(
            &v1,
            &[
                "NumberExpression",
                "NumberTerm",
                "StringExpression",
                "BooleanExpression",
                "BooleanAndExpression",
                "BooleanXorExpression",
            ]
        ),
        shape(
            &compiled.ir,
            &[
                "NumberExpression",
                "NumberExpression__lr1",
                "StringExpression",
                "BooleanExpression",
                "BooleanExpression__lr1",
                "BooleanExpression__lr2",
            ]
        )
    );
}

// ── §5 言語中立な token 語彙（D-041 / D-042） ──────────────────────────────

/// token 宣言だけが違う最小の文法。
fn with_token(version: Option<u64>, declaration: &str) -> String {
    let settings = version.map_or(String::new(), |v| format!("  @ubnf: {v}\n"));
    format!(
        "grammar G {{\n  @package: org.example\n{settings}  @whitespace: none\n  \
         token T = {declaration}\n  @root\n  S ::= T ;\n}}"
    )
}

fn token_spec(compiled: &Compiled) -> String {
    let token = &compiled.ir.semantics.tokens[0];
    // spec / 診断プロファイル / memo 安全性の「意味のある部分」だけを比べる。
    // 綴りそのものが写る originalDeclaration と reason は外す。
    format!(
        "{:?}|{:?}|{}/{}",
        token.spec, token.diagnostic_profiles, token.memo_safety.success, token.memo_safety.failure
    )
}

/// 中立名は「同じ FQN を書いたとき」と**同じ述語・同じ IR** を指す（D-041）。
/// これが語彙の契約そのもの（02-facts §A の受理範囲を 1 つも変えない）。
#[test]
fn every_neutral_name_produces_the_same_token_spec_as_its_host_class_spelling() {
    for (neutral, class) in ubnfc_front::vocabulary::BUILTIN_VOCABULARY {
        let by_class = source(&with_token(Some(2), class)).unwrap();
        let by_name = source(&with_token(Some(2), neutral)).unwrap();
        assert_eq!(
            token_spec(&by_class),
            token_spec(&by_name),
            "{neutral} と {class} の IR が違う"
        );
    }
}

/// 中立語彙は**純粋な追加**なので v1 でも書ける（06 §0 の規則）。
/// v1 の既存文法は 1 文字も変わらないし、v1 で中立名を書いても v2 と同じ IR になる。
#[test]
fn the_neutral_vocabulary_is_additive_and_also_works_under_v1() {
    for version in [None, Some(1), Some(2)] {
        let compiled = source(&with_token(version, "NUMBER")).unwrap();
        assert_eq!(
            token_spec(&compiled),
            token_spec(&source(&with_token(version, "NumberParser")).unwrap())
        );
    }
    // v1 で中立名を書いても警告は出ない。
    assert!(codes(&with_token(Some(1), "IDENTIFIER")).is_empty());
}

/// `extern '<id>'` は host 実装の明示宣言。名前は registry の ID なので FQN でなくてよく、
/// 綴り間違いを拾うための W-TOKEN-UNRESOLVED は出さない（registry の有無は ir/gen が見る）。
#[test]
fn extern_declares_a_host_implementation_without_a_java_package() {
    let compiled = source(&with_token(Some(2), "extern 'StringLiteralParser'")).unwrap();
    let spec = format!("{:?}", compiled.ir.semantics.tokens[0].spec);
    assert!(spec.contains("StringLiteralParser"), "{spec}");
    assert!(!spec.contains("org.unlaxer"), "{spec}");
    assert!(codes(&with_token(Some(2), "extern 'StringLiteralParser'")).is_empty());
    // 単純名の ClassRef は「未知のクラス」として警告が出る形。extern はそれを消す。
    assert_eq!(
        codes(&with_token(Some(2), "StringLiteralParser")),
        ["W-TOKEN-HOST-CLASS", "W-TOKEN-UNRESOLVED"]
    );
    // extern は綴りをそのまま残すので、中立語彙と綴りが重なる host クラスも書ける。
    let escaped = source(&with_token(Some(2), "extern 'NUMBER'")).unwrap();
    assert!(format!("{:?}", escaped.ir.semantics.tokens[0].spec).contains("Extern"));
}

/// v2 は旧綴りを**警告する**（拒否しない、D-042）。v1 では旧綴りが唯一の綴りなので黙る。
#[test]
fn v2_warns_about_host_class_token_declarations_but_still_accepts_them() {
    let fqn = "org.unlaxer.parser.elementary.NumberParser";
    assert_eq!(codes(&with_token(Some(2), fqn)), ["W-TOKEN-HOST-CLASS"]);
    assert!(codes(&with_token(None, fqn)).is_empty());
    assert!(codes(&with_token(Some(1), fqn)).is_empty());
    // 警告どまりなので IR は作られ、中立名の IR と一致する。
    assert_eq!(
        token_spec(&source(&with_token(Some(2), fqn)).unwrap()),
        token_spec(&source(&with_token(Some(2), "NUMBER")).unwrap())
    );
}

/// v2 では registry が提供しない extern を **生成時に** 落とす（D-042）。
/// v1 は凍結なので従来どおり stderr の注意だけで IR を出す。
#[test]
fn v2_rejects_an_extern_that_no_scanner_registry_provides() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let first_chars = root.join("scanners/first-chars.json");
    let dir = std::env::temp_dir().join(format!("ubnfc-extern-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let run = |version: Option<u64>, declaration: &str| {
        let path = dir.join("g.ubnf");
        std::fs::write(&path, with_token(version, declaration)).unwrap();
        std::process::Command::new(env!("CARGO_BIN_EXE_ubnfc"))
            .args(["ir", "--grammar"])
            .arg(&path)
            .args(["--out", "-", "--extern-first-chars"])
            .arg(&first_chars)
            .output()
            .unwrap()
    };

    let missing = run(Some(2), "extern 'NoSuchScanner'");
    assert_eq!(missing.status.code(), Some(1));
    let stderr = String::from_utf8(missing.stderr).unwrap();
    assert!(stderr.contains("E-TOKEN-EXTERN-UNREGISTERED"), "{stderr}");
    assert!(stderr.contains("NoSuchScanner"), "{stderr}");

    let registered = run(Some(2), "extern 'StringLiteralParser'");
    assert_eq!(registered.status.code(), Some(0));

    // v1 は凍結: 同じ形が従来どおり注意だけで通る。
    let v1 = run(None, "NoSuchScanner");
    assert_eq!(v1.status.code(), Some(0));
    std::fs::remove_dir_all(&dir).unwrap();
}

// ── §6 注釈の統廃合（D-045 〜 D-049）─────────────────────────────────────

/// `@symbolRef` / `@sameText` は `scope.effects[].mode` を明示するだけで、
/// IR の他の部分は旧 `@backref` と同じ（D-046）。
#[test]
fn symbol_ref_and_same_text_only_pin_the_scope_mode() {
    let body = |annotation: &str| {
        format!(
            "  @root @scopeTree(mode=lexical) @mapping(Root, params=[decl, ref])\n  \
             Root ::= Decl @decl Ref @ref;\n  \
             @declares(symbol=name) @mapping(Decl, params=[name]) Decl ::= NUMBER @name ';';\n  \
             {annotation} @mapping(Ref, params=[name]) Ref ::= '->' NUMBER @name;\n"
        )
    };
    let modes = |text: &str| {
        let compiled = source(text).unwrap();
        compiled
            .ir
            .semantics
            .scope
            .effects
            .iter()
            .map(|e| {
                (
                    e.rule_id.rsplit("::").next().unwrap().to_owned(),
                    serde_json::to_value(e.mode)
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned(),
                )
            })
            .collect::<Vec<_>>()
    };
    // 文法に `@scopeTree` があるので旧 `@backref` は scope 検索になる。
    let legacy = grammar(None, &body("@backref(name=name)"));
    let explicit = grammar(None, &body("@symbolRef(name=name)"));
    assert_eq!(modes(&legacy), modes(&explicit));
    assert!(modes(&explicit).iter().all(|(_, mode)| mode == "lexical"));
    // `@sameText` は `@scopeTree` があってもテキスト一致に固定する（非局所判定より優先）。
    let same_text = source(&grammar(None, &body("@sameText(name=name)"))).unwrap();
    let ref_mode = same_text
        .ir
        .semantics
        .scope
        .effects
        .iter()
        .find(|e| e.rule_id.ends_with("::Ref"))
        .map(|e| serde_json::to_value(e.mode).unwrap());
    assert_eq!(ref_mode.unwrap(), serde_json::json!("sameRuleText"));
    // 旧綴りは v2 でだけ非推奨警告が出る（v1 は 1 件も増やさない）。
    assert!(!codes(&legacy).contains(&"W-ANNOTATION-BACKREF-AMBIGUOUS".to_owned()));
    let v2 = grammar(Some(2), &body("@backref(name=name)"));
    assert!(codes(&v2).contains(&"W-ANNOTATION-BACKREF-AMBIGUOUS".to_owned()));
    assert!(!codes(&grammar(Some(2), &body("@symbolRef(name=name)")))
        .contains(&"W-ANNOTATION-BACKREF-AMBIGUOUS".to_owned()));
}

/// 未知注釈は版に関係なく警告する（D-045）。未知の grammar 設定は v2 だけ。
#[test]
fn unknown_annotations_warn_in_every_version_and_unknown_settings_only_in_v2() {
    for version in [None, Some(1), Some(2)] {
        let text = grammar(version, "  @root @rooted R ::= NUMBER;\n");
        assert!(
            codes(&text).contains(&"W-ANNOTATION-UNKNOWN".to_owned()),
            "{version:?}"
        );
        assert!(!codes(&grammar(version, "  @root R ::= NUMBER;\n"))
            .contains(&"W-ANNOTATION-UNKNOWN".to_owned()));
    }
    // 未知設定は旧 Java 検証器と揃えた前方互換契約があるので v1 では黙る。
    let setting = |version: Option<u64>| {
        let settings = version.map_or(String::new(), |v| format!("  @ubnf: {v}\n"));
        format!(
            "grammar G {{\n  @package: org.example\n{settings}  @futureSetting: value\n  \
             @whitespace: javaStyle\n  token NUMBER = NumberParser\n  @root R ::= NUMBER;\n}}"
        )
    };
    assert!(!codes(&setting(None)).contains(&"W-ANNOTATION-UNKNOWN".to_owned()));
    assert!(!codes(&setting(Some(1))).contains(&"W-ANNOTATION-UNKNOWN".to_owned()));
    assert!(codes(&setting(Some(2))).contains(&"W-ANNOTATION-UNKNOWN".to_owned()));
}

/// `@interleave` と `@predictiveChoice` は v2 でだけ非推奨（D-048 / D-049）。
/// `@longestChoice` は意味論の選択なのでどの版でも黙る。
#[test]
fn interleave_and_predictive_choice_are_deprecated_only_under_v2() {
    let interleave = "  @root @interleave(profile=javaStyle) R ::= NUMBER;\n";
    let predictive = "  @root @predictiveChoice R ::= NUMBER | '-' NUMBER;\n";
    let longest = "  @root @longestChoice R ::= NUMBER | '-' NUMBER;\n";
    for (body, code) in [
        (interleave, "W-ANNOTATION-INTERLEAVE-DEPRECATED"),
        (predictive, "W-ANNOTATION-PREDICTIVE-REDUNDANT"),
    ] {
        assert!(
            !codes(&grammar(None, body)).contains(&code.to_owned()),
            "{code} v1"
        );
        assert!(
            codes(&grammar(Some(2), body)).contains(&code.to_owned()),
            "{code} v2"
        );
    }
    for version in [None, Some(2)] {
        assert!(!codes(&grammar(version, longest))
            .iter()
            .any(|c| c.starts_with("W-ANNOTATION-")));
    }
}

/// D-047: `@rightAssoc` + 規則参照演算子は `InvalidOperator` で落ちていた。
/// リテラル演算子と同じく通り、`operator` metadata は作らない。
#[test]
fn right_associative_rules_with_a_rule_reference_operator_compile() {
    let body =
        "  @root @mapping(Bin, params=[left, op, right]) @rightAssoc @precedence(level=20)\n  \
                R ::= Atom @left { PowOp @op R @right };\n  \
                PowOp ::= '^';\n  Atom ::= NUMBER;\n";
    let compiled = source(&grammar(None, body)).unwrap();
    let rule = compiled
        .ir
        .semantics
        .rules
        .iter()
        .find(|r| r.symbol_id.ends_with("::R"))
        .unwrap();
    assert!(rule.operator.is_none(), "{:?}", rule.operator);
    // リテラル演算子の右結合（Power.ubnf と同形）も同じく operator を作らない。
    let literal = source(&grammar(
        None,
        "  @root @mapping(Bin, params=[left, op, right]) @rightAssoc @precedence(level=20)\n  \
         R ::= Atom @left { '^' @op R @right };\n  Atom ::= NUMBER;\n",
    ))
    .unwrap();
    assert!(literal
        .ir
        .semantics
        .rules
        .iter()
        .find(|r| r.symbol_id.ends_with("::R"))
        .unwrap()
        .operator
        .is_none());
}
