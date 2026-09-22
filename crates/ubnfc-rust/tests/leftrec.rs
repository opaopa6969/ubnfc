use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};
use ubnfc_front::ir::*;
use ubnfc_rust::{emit, EmitOptions};

fn fixture(leftrec: bool) -> GrammarIr {
    from_json(if leftrec {
        include_str!("../../../ir/fixtures/story__code__calc-leftrec.ubnf.ir.json")
    } else {
        include_str!("../../../ir/fixtures/story__code__calc.ubnf.ir.json")
    })
    .unwrap()
}

fn without_history(mut ir: GrammarIr) -> GrammarIr {
    ir.semantics.normalizations.clear();
    for rule in &mut ir.analysis.rules {
        rule.left_recursion = RuleAnalysisLeftRecursion::None;
    }
    ir
}

// D-016 intentionally excludes mappings from the source left-recursive grammar.
// Attach calc's mapping only AFTER lowering, without changing executable nodes.
// Both fixtures lack @eval; values below are folds of the generated canonical AST.
fn mapped_leftrec() -> GrammarIr {
    let mut ir = fixture(true);
    let calc = fixture(false);
    ir.semantics.ast_types = calc.semantics.ast_types.clone();
    ir.required_features = calc.required_features.clone();
    for original in &calc.semantics.rules {
        if original.mapping_type_id.is_none() {
            continue;
        }
        let rule_id = original.symbol_id.replace("TinyCalc::", "TinyCalcLR::");
        let rule = ir
            .semantics
            .rules
            .iter_mut()
            .find(|r| r.symbol_id == rule_id)
            .unwrap();
        rule.mapping_type_id = original.mapping_type_id.clone();
        rule.mapping_params = original.mapping_params.clone();
        let expressions: BTreeMap<_, _> = ir
            .semantics
            .expressions
            .iter()
            .map(|e| (&e.id, &e.node))
            .collect();
        let ExprNode::Seq { items } = expressions[&rule.body] else {
            panic!("lowered body must be Base (Op Base)*");
        };
        let ExprNode::Repeat { child, .. } = expressions[&items[1]] else {
            panic!("lowered tail must repeat");
        };
        let ExprNode::Seq { items: pair } = expressions[child] else {
            panic!("lowered pair must be Op Base");
        };
        let sites = [items[0].clone(), pair[0].clone(), pair[1].clone()];
        let mut recipe = calc
            .semantics
            .recipes
            .iter()
            .find(|r| r.rule_id == original.symbol_id)
            .unwrap()
            .clone();
        recipe.rule_id = rule_id.clone();
        for (name, expr_id) in ["left", "op", "right"].into_iter().zip(sites) {
            let mut capture = calc
                .semantics
                .captures
                .iter()
                .find(|c| c.rule_id == original.symbol_id && c.name == name)
                .unwrap()
                .clone();
            capture.rule_id = rule_id.clone();
            capture.site_id = format!("{expr_id}/capture/0");
            capture.expr_id = expr_id.clone();
            let site_ids = vec![capture.site_id.clone()];
            ir.semantics
                .expressions
                .iter_mut()
                .find(|e| e.id == expr_id)
                .unwrap()
                .capture_site_ids = site_ids.clone();
            recipe
                .fields
                .iter_mut()
                .find(|f| f.field == name)
                .unwrap()
                .site_ids = site_ids;
            ir.semantics.captures.push(capture);
        }
        ir.semantics.recipes.push(recipe);
    }
    ir
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let output = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(output.status.success());
        Self(PathBuf::from(
            String::from_utf8(output.stdout).unwrap().trim(),
        ))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn node(ty: &str, span: [usize; 2], left: Value, op: &[&str], right: Vec<Value>) -> Value {
    json!({"type":ty,"span":span,"fields":{"left":left,"op":op,"right":right}})
}
fn number(text: &str, start: usize) -> Value {
    node("MulExpr", [start, start + 1], json!(text), &[], vec![])
}
fn golden() -> Vec<Value> {
    vec![
        node(
            "AddExpr",
            [0, 5],
            number("1", 0),
            &["+"],
            vec![node(
                "MulExpr",
                [2, 5],
                json!("2"),
                &["*"],
                vec![json!("3")],
            )],
        ),
        node(
            "AddExpr",
            [0, 7],
            node(
                "MulExpr",
                [0, 7],
                node(
                    "AddExpr",
                    [1, 4],
                    number("1", 1),
                    &["+"],
                    vec![number("2", 3)],
                ),
                &["*"],
                vec![json!("3")],
            ),
            &[],
            vec![],
        ),
        node(
            "AddExpr",
            [0, 5],
            number("1", 0),
            &["-", "-"],
            vec![number("2", 2), number("3", 4)],
        ),
        Value::Null,
    ]
}
fn evaluate(ast: &Value) -> f64 {
    if let Some(text) = ast.as_str() {
        return text.parse().unwrap();
    }
    let fields = &ast["fields"];
    let mut value = evaluate(&fields["left"]);
    let ops = fields["op"].as_array().unwrap();
    let rights = fields["right"].as_array().unwrap();
    assert_eq!(ops.len(), rights.len());
    for (op, right) in ops.iter().zip(rights) {
        let right = evaluate(right);
        value = match op.as_str().unwrap() {
            "+" => value + right,
            "-" => value - right,
            "*" => value * right,
            "/" => value / right,
            other => panic!("unknown operator: {other}"),
        };
    }
    value
}

#[test]
fn lowered_calc_compiles_and_matches_canonical_ast_spans_and_values() {
    let temp = Temp::new();
    let grammars = [
        ("raw", fixture(true)),
        ("lowered", mapped_leftrec()),
        ("no_history", without_history(mapped_leftrec())),
        ("ordinary", fixture(false)),
    ];
    let mut main = String::from("#![allow(dead_code)]\n");
    for (name, ir) in &grammars {
        emit(
            ir,
            &EmitOptions {
                emit_driver: false,
                ..Default::default()
            },
        )
        .unwrap()
        .write_to(temp.0.join("src").join(name))
        .unwrap();
        main.push_str(&format!("mod {name};\n"));
    }
    main.push_str("fn main() {for memo in [false,true] {for input in [\"1+2*3\",\"(1+2)*3\",\"1-2-3\",\"1+\"] {\n");
    for (name, _) in &grammars {
        main.push_str(&format!(
            "let r={name}::parse_with_options(input,{name}::ParseOptions{{memo,..Default::default()}});println!(\"{{}}\",serde_json::json!({{\"ok\":r.ok,\"consumed\":r.consumed_cp,\"matched\":r.matched_cp,\"ast\":r.ast.as_ref().map(|a|a.canonical_value())}}));\n"
        ));
    }
    main.push_str("}}}\n");
    fs::write(temp.0.join("src/main.rs"), main).unwrap();
    fs::write(temp.0.join("Cargo.toml"), "[package]\nname=\"leftrec-check\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[features]\njson=[]\n[dependencies]\nserde={version=\"1\",features=[\"derive\"]}\nserde_json=\"1\"\n").unwrap();
    let output = Command::new(env!("CARGO"))
        .args(["run", "--offline", "--quiet", "--features", "json"])
        .current_dir(&temp.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 32);
    let golden = golden();
    for (index, results) in rows.chunks_exact(4).enumerate() {
        let case = index % 4;
        for result in results {
            assert_eq!(result["ok"], case < 3, "case {case}: {result}");
            for key in ["consumed", "matched"] {
                assert_eq!(result[key], results[3][key], "case {case}: {key}");
            }
        }
        for result in &results[1..] {
            assert_eq!(result["ast"], golden[case], "case {case}");
            assert_eq!(result, &results[3], "lowering equivalence: case {case}");
            if case < 3 {
                assert_eq!(evaluate(&result["ast"]), [7.0, 9.0, -4.0][case]);
                assert_eq!(result["consumed"], [5, 7, 5][case]);
                assert_eq!(result["matched"], [5, 7, 5][case]);
            }
        }
        if case == 3 {
            assert!(results[0]["ast"].is_null());
        }
    }
}

#[test]
fn unlowered_direct_and_indirect_cycles_are_rejected_without_trusting_history() {
    for history in [false, true] {
        let mut ir = fixture(true);
        let rewrite = ir.semantics.normalizations[0].clone();
        ir.semantics
            .rules
            .iter_mut()
            .find(|r| r.symbol_id == rewrite.rule_id)
            .unwrap()
            .body = rewrite.original_body.clone();
        // Keep normalization references valid while making the active body recursive.
        ir.semantics.normalizations[0].rewritten_body = rewrite.original_body;
        if !history {
            ir = without_history(ir);
        }
        let error = emit(&ir, &Default::default()).unwrap_err().to_string();
        assert!(error.contains("unloweredLeftRecursion"), "{error}");
    }
    let mut ir = without_history(fixture(true));
    // Expression -> Term -> Factor -> Expression, before consuming any input.
    let factor = ir
        .semantics
        .expressions
        .iter_mut()
        .find(|e| e.rule_id == "TinyCalcLR::Factor" && matches!(e.node, ExprNode::TokenRef { .. }))
        .unwrap();
    factor.node = ExprNode::RuleRef {
        symbol_id: "TinyCalcLR::Expression".into(),
    };
    let error = emit(&ir, &Default::default()).unwrap_err().to_string();
    assert!(error.contains("unloweredLeftRecursion"), "{error}");
}

#[test]
fn nullable_prefix_cycles_are_rejected_but_consuming_recursion_is_allowed() {
    let temp = Temp::new();
    let path = temp.0.join("cycle.ubnf");
    for prefix_nullable in [true, false] {
        let source = if prefix_nullable {
            "grammar Probe { @root Root ::= 'a'? 'b'; }"
        } else {
            "grammar Probe { @root Root ::= 'a' 'b'; }"
        };
        let mut ir = ubnfc_front::pipeline::compile(&path, &|_| Ok(source.into()))
            .unwrap()
            .ir;
        let last = ir
            .semantics
            .expressions
            .iter_mut()
            .find(|e| matches!(&e.node, ExprNode::Literal { value, .. } if value == "b"))
            .unwrap();
        last.node = ExprNode::RuleRef {
            symbol_id: ir.entry.rule_id.clone(),
        };
        last.execution.transaction = ExecutionTransaction::Delegate;
        // Conservative/stale nullable metadata must neither invent nor hide cycles.
        for analysis in &mut ir.analysis.expressions {
            analysis.nullable = !prefix_nullable;
        }
        let result = emit(&ir, &Default::default());
        if prefix_nullable {
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("unloweredLeftRecursion"));
        } else {
            result.unwrap();
        }
    }
}
