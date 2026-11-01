use std::path::{Path, PathBuf};
use ubnfc_front::ir::*;
use ubnfc_front::pipeline::{compile, PipelineError};

fn source(text: &str) -> GrammarIr {
    compile("/virtual/contract.ubnf", &|_| Ok(text.into()))
        .unwrap()
        .ir
}

fn fixture(variable: &str, default: &str, relative: &str) -> GrammarIr {
    let root = std::env::var_os(variable).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(default),
        PathBuf::from,
    );
    compile(root.join(relative), &|p| std::fs::read_to_string(p))
        .unwrap()
        .ir
}

fn field<'a>(ir: &'a GrammarIr, type_id: &str, name: &str) -> &'a AstField {
    ir.semantics
        .ast_types
        .iter()
        .find_map(|t| match t {
            AstType::Record { id, fields, .. } if id == type_id => {
                fields.iter().find(|f| f.name == name)
            }
            _ => None,
        })
        .unwrap()
}

fn recipe<'a>(ir: &'a GrammarIr, id: &str, name: &str) -> &'a RecipeField {
    ir.semantics
        .recipes
        .iter()
        .find(|r| r.id == id)
        .unwrap()
        .fields
        .iter()
        .find(|f| f.field == name)
        .unwrap()
}

#[test]
fn shared_constructor_joins_types_but_preserves_each_site_conversion_in_both_orders() {
    let first = "@mapping(Value,params=[value]) First ::= 'a' @value;";
    let second = "@mapping(Value,params=[value]) Second ::= Leaf @value;";
    let third = "@mapping(Value,params=[value]) Third ::= N @value;";
    for rules in [[first, second, third], [third, second, first]] {
        let ir = source(&format!(
            "grammar G {{ token N = NumberParser @root Root ::= First | Second | Third; {} @mapping(Leaf) Leaf ::= 'x'; }}",
            rules.join(" ")
        ));
        // D-074: 数値 token の直接 capture も text なので、join は text/node の 2 つ。
        let ValueType::Mixed { members } = &field(&ir, "GAST.Value", "value").value_type else {
            panic!("共有 constructor は text/node の join");
        };
        assert_eq!(members.len(), 2);
        assert!(members.contains(&ValueType::Text {}));
        assert!(members.contains(&ValueType::Node {
            type_id: "GAST.Leaf".into()
        }));
        assert!(!members
            .iter()
            .any(|m| matches!(m, ValueType::Number { .. })));
        for (rule, local) in [
            ("First", Conversion::Text),
            ("Second", Conversion::Node),
            ("Third", Conversion::Text),
        ] {
            let f = recipe(&ir, &format!("recipe:{rule}"), "value");
            assert_eq!(f.conversion, Conversion::Mixed);
            assert_eq!(f.cardinality, Cardinality::Scalar);
            let capture = ir
                .semantics
                .captures
                .iter()
                .find(|c| c.site_id == f.site_ids[0])
                .unwrap();
            assert_eq!(capture.conversion, local);
        }
    }
}

#[test]
fn skipped_mapping_has_no_constructor_fields() {
    let ir = source("grammar G { @root Root ::= Hidden; @skip @mapping(Hidden,params=[value]) Hidden ::= 'x' @value; }");
    let recipe = &ir.semantics.recipes[0];
    assert_eq!(recipe.kind, RecipeKind::Skip);
    assert!(recipe.result_type_id.is_none());
    assert!(recipe.fields.is_empty());
    assert!(ir.semantics.captures[0].skip);
}

#[test]
fn compound_text_and_mapped_dispatch_keep_their_value_types() {
    let ir = source("grammar G { token N = NumberParser @root @mapping(All,params=[text,number,choice,nodes,mixed]) Root ::= (N ':' 'x') @text N @number ('a' | 'b') @choice { Leaf | Other } @nodes (Leaf | '!') @mixed; @mapping(Leaf) Leaf ::= 'l'; @mapping(Other) Other ::= 'o'; }");
    for name in ["text", "choice"] {
        assert_eq!(field(&ir, "GAST.All", name).value_type, ValueType::Text {});
        assert_eq!(
            recipe(&ir, "recipe:Root", name).conversion,
            Conversion::Text
        );
    }
    // D-074: token の直接 capture は数値 token でも原文の文字列。
    assert_eq!(
        field(&ir, "GAST.All", "number").value_type,
        ValueType::Text {}
    );
    assert_eq!(
        recipe(&ir, "recipe:Root", "number").conversion,
        Conversion::Text
    );
    let nodes = field(&ir, "GAST.All", "nodes");
    assert_eq!(nodes.cardinality, Cardinality::List);
    let ValueType::Mixed { members } = &nodes.value_type else {
        panic!("typed choice");
    };
    assert_eq!(members.len(), 2);
    assert!(members.iter().all(|m| matches!(m, ValueType::Node { .. })));
    let ValueType::Mixed { members } = &field(&ir, "GAST.All", "mixed").value_type else {
        panic!("mixed choice");
    };
    assert!(members.contains(&ValueType::Text {}));
    assert_eq!(
        recipe(&ir, "recipe:Root", "mixed").conversion,
        Conversion::Mixed
    );
}

#[test]
fn p4_shared_and_selfhost_recipes_agree_with_ast_fields() {
    for (variable, root, path) in [
        (
            "TINYEXPRESSION_DIR",
            "../../../tinyexpression",
            "tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf",
        ),
        (
            "UNLAXER_PARSER_DIR",
            "../../../unlaxer-parser",
            "unlaxer-dsl/src/test/resources/mixed-values/Shared.ubnf",
        ),
        (
            "UNLAXER_PARSER_DIR",
            "../../../unlaxer-parser",
            "unlaxer-dsl/grammar/ubnf.ubnf",
        ),
    ] {
        let ir = fixture(variable, root, path);
        for r in &ir.semantics.recipes {
            for f in &r.fields {
                let schema = field(&ir, r.result_type_id.as_ref().unwrap(), &f.field);
                assert_eq!(f.cardinality, schema.cardinality, "{} {}", r.id, f.field);
                match &schema.value_type {
                    ValueType::Text {} => assert_eq!(f.conversion, Conversion::Text),
                    ValueType::Number { .. } => assert_eq!(f.conversion, Conversion::Number),
                    ValueType::Node { .. } => assert_eq!(f.conversion, Conversion::Node),
                    ValueType::Mixed { members } => {
                        if members
                            .iter()
                            .any(|m| !matches!(m, ValueType::Node { .. } | ValueType::Null {}))
                        {
                            assert_eq!(f.conversion, Conversion::Mixed, "{} {}", r.id, f.field);
                        } else {
                            assert!(matches!(f.conversion, Conversion::Mixed | Conversion::Node));
                        }
                    }
                    other => panic!("front の field は要素型と cardinality を分ける: {other:?}"),
                }
            }
        }
        if path.ends_with("tinyexpression-p4.ubnf") {
            assert_eq!(
                recipe(&ir, "recipe:SliceNestedExpression", "value").conversion,
                Conversion::Mixed
            );
            let externs: Vec<_> = ir
                .semantics
                .tokens
                .iter()
                .filter_map(|t| {
                    if let TokenSpec::Extern { input_state, .. } = &t.spec {
                        Some((t.symbol_id.as_str(), input_state))
                    } else {
                        None
                    }
                })
                .collect();
            // 2026-10 に P4 の token は lexical module の宣言的 token へ移り、host 実装
            // （extern）は無くなった。
            assert!(externs.is_empty(), "{externs:?}");
        } else if path.ends_with("Shared.ubnf") {
            assert_eq!(
                recipe(&ir, "recipe:Second", "value").conversion,
                Conversion::Mixed
            );
        } else {
            let annotations = field(&ir, "UBNFAST.RuleDecl", "annotations");
            assert_eq!(annotations.cardinality, Cardinality::List);
            let ValueType::Mixed { members } = &annotations.value_type else {
                panic!("annotation variants");
            };
            // 2026-10 に @lexicalContext / @uniqueLongestChoice / @namePredicate / @predicate が増えた。
            assert_eq!(members.len(), 24);
            assert!(members.iter().all(|m| matches!(m, ValueType::Node { .. })));
            assert_eq!(
                recipe(&ir, "recipe:RuleDecl", "annotations").conversion,
                Conversion::Mixed
            );
        }
    }
}

#[test]
fn typeof_and_default_eval_keep_mapping_contract_metadata() {
    let ir = fixture(
        "UNLAXER_PARSER_DIR",
        "../../../unlaxer-parser",
        "rust/unlaxer-ubnf/tests/fixtures/positive/elements.ubnf",
    );
    assert!(ir.required_features.contains(&RequiredFeature::Typeof));
    assert_eq!(ir.semantics.typeof_constraints.len(), 1);
    let constraint = &ir.semantics.typeof_constraints[0];
    let own = ir
        .semantics
        .captures
        .iter()
        .find(|c| c.site_id == constraint.own_capture)
        .unwrap();
    let referenced = ir
        .semantics
        .captures
        .iter()
        .find(|c| c.site_id == constraint.referenced_capture)
        .unwrap();
    assert_eq!((&*own.name, &*referenced.name), ("k", "a"));
    assert_eq!(constraint.site_id, format!("{}/typeof/0", own.expr_id));
    assert_eq!(constraint.phase, TypeofPhase);
    assert_eq!(constraint.comparison_kind, TypeofComparisonKind);
    assert_eq!(constraint.mismatch, TypeofMismatch);

    let ir = fixture(
        "UNLAXER_PARSER_DIR",
        "../../../unlaxer-parser",
        "rust/unlaxer-ubnf/tests/fixtures/known/eval-default.ubnf",
    );
    assert!(ir.required_features.contains(&RequiredFeature::Eval));
    let eval = ir.semantics.rules[0].eval.as_ref().unwrap();
    assert_eq!(eval.kind, "variable_ref");
    assert_eq!(eval.strategy, EvalStrategy::Auto);
    assert_eq!(
        serde_json::to_value(&eval.params).unwrap(),
        serde_json::json!({"strip_prefix":"$"})
    );
}

#[test]
fn automatic_sum_eval_fails_validation_and_manual_or_unknown_kind_uses_callback() {
    let grammar = "grammar G { @root @mapping(Node) @eval(kind='literal',strategy='auto') Root ::= A | B; @mapping(A) A ::= 'a'; @mapping(B) B ::= 'b'; }";
    let Err(PipelineError::Validation(issues)) =
        compile("/virtual/sum.ubnf", &|_| Ok(grammar.into()))
    else {
        panic!("和型の自動 eval は validate で拒否する");
    };
    assert!(issues.iter().any(|i| i.code == "E-EVAL-SUM-TYPE"));
    let manual = source(&grammar.replace("strategy='auto'", "strategy='manual'"));
    assert_eq!(
        manual.semantics.rules[0].eval.as_ref().unwrap().strategy,
        EvalStrategy::AbstractCallback
    );
    let unknown = source(
        "grammar G { @root @mapping(Value) @eval(kind='custom',arg='value') Root ::= 'x'; }",
    );
    let eval = unknown.semantics.rules[0].eval.as_ref().unwrap();
    assert_eq!(eval.kind, "custom");
    assert_eq!(eval.strategy, EvalStrategy::AbstractCallback);
    assert_eq!(
        serde_json::to_value(&eval.params).unwrap(),
        serde_json::json!({"arg":"value"})
    );
}

#[test]
fn rewritten_left_recursion_is_metadata_not_a_backend_requirement() {
    let ir = fixture(
        "UNLAXER_PARSER_DIR",
        "../../../unlaxer-parser",
        "story/code/calc-leftrec.ubnf",
    );
    assert!(!ir.semantics.normalizations.is_empty());
    assert!(!ir
        .required_features
        .contains(&RequiredFeature::LeftRecursionRewrite));
    for rewrite in &ir.semantics.normalizations {
        let rule = ir
            .semantics
            .rules
            .iter()
            .find(|r| r.symbol_id == rewrite.rule_id)
            .unwrap();
        assert_eq!(rule.body, rewrite.rewritten_body);
        assert_ne!(rule.body, rewrite.original_body);
        let body = ir
            .semantics
            .expressions
            .iter()
            .find(|e| e.id == rule.body)
            .unwrap();
        let ExprNode::Seq { items } = &body.node else {
            panic!("Base Repeat");
        };
        assert!(items.iter().any(|id| ir
            .semantics
            .expressions
            .iter()
            .any(|e| e.id == *id && matches!(e.node, ExprNode::Repeat { .. }))));
        assert_eq!(
            ir.analysis
                .rules
                .iter()
                .find(|r| r.rule_id == rule.symbol_id)
                .unwrap()
                .left_recursion,
            RuleAnalysisLeftRecursion::DirectRewritten
        );
    }
    assert!(validate_references(&ir).is_empty());
    ubnfc_front::analysis::analyze(&ir.semantics).unwrap();
}
