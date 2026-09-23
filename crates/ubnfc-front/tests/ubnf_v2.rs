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
