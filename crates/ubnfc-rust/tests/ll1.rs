//! 表の判定境界と実行時の観測同値を検証する。
use std::{fs, path::PathBuf, process::Command};
use ubnfc_front::{ir::*, pipeline::compile};

fn grammar(body: &str) -> GrammarIr {
    let text = format!("grammar Table {{ @ubnf: 2\n @package: test.table\n @whitespace: none\n token D = DIGIT\n {body} }}");
    compile("table.ubnf", &|_| Ok(text.clone())).unwrap().ir
}
fn root_covered(ir: &GrammarIr) -> bool {
    let root = ir
        .semantics
        .rules
        .iter()
        .find(|r| r.symbol_id == ir.entry.rule_id)
        .unwrap();
    ir.plan.ll1.iter().any(|s| s.expr_id == root.body && s.root)
}

#[test]
fn exclusive_first_accepts_recursion_but_overlapping_nullable_and_effectful_regions_stay_out() {
    assert!(root_covered(&grammar(
        "@root Root ::= '(' [ Root ] ')' | D ;"
    )));
    for body in [
        "@root Root ::= 'ab' | 'ac' ;",
        "@root Root ::= [ 'a' ] | 'b' ;",
        "token L = LOOKAHEAD('a')
 @root Root ::= L ;",
    ] {
        assert!(!root_covered(&grammar(body)), "{body}");
    }
    let json = compile("../../examples/json/json.ubnf", &|p| fs::read_to_string(p))
        .unwrap()
        .ir;
    assert!(root_covered(&json));
    assert_eq!(
        json.plan.ll1.len(),
        json.semantics.expressions.len(),
        "JSON 全式が表の領域に入る"
    );
}

#[test]
fn scope_grammars_keep_the_reference_path_even_when_static_regions_exist() {
    let ir = from_json(include_str!("../../../ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json")).unwrap();
    assert!(!ir.plan.ll1.is_empty());
    let generated = ubnfc_rust::emit(
        &ir,
        &ubnfc_rust::EmitOptions {
            emit_driver: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!generated.files["parser.rs"].contains("self.options.ll1"));
}

#[test]
fn a_tampered_table_is_not_generated_or_executed() {
    let mut ir = grammar("@root Root ::= D+ ;");
    ir.plan.ll1[0].operation = Ll1Operation::Call { target: u64::MAX };
    let generated = ubnfc_rust::emit(&ir, &Default::default()).unwrap();
    assert!(!generated.files["parser.rs"].contains("self.options.ll1"));
    assert!(generated.files["runtime.rs"].contains("static LL1_TABLE:&[Ll1Op]=&[\n];"));
}

#[test]
fn a_reordered_table_retains_its_proof_by_expression_id() {
    let mut ir = grammar("@root Root ::= '(' D+ ')' ;");
    let size = ir.plan.ll1.len() as u64;
    ir.plan.ll1.reverse();
    for state in &mut ir.plan.ll1 {
        let remap = |target: &mut u64| *target = size - 1 - *target;
        match &mut state.operation {
            Ll1Operation::Sequence { targets } => {
                for target in targets {
                    remap(target);
                }
            }
            Ll1Operation::Choice { transitions } => {
                for row in transitions {
                    remap(&mut row.target);
                }
            }
            Ll1Operation::Optional { target }
            | Ll1Operation::Repeat { target, .. }
            | Ll1Operation::Call { target } => remap(target),
            _ => {}
        }
    }
    let generated =
        ubnfc_rust::emit(&ir, &Default::default()).expect("配列順ではなく命令を検証する");
    assert!(generated.files["parser.rs"].contains("self.options.ll1"));
}

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn generated_table_matches_the_reference_including_partial_failure_unicode_and_limits() {
    let output = Command::new("mktemp").arg("-d").output().unwrap();
    assert!(output.status.success());
    let temp = Temp(PathBuf::from(
        String::from_utf8(output.stdout).unwrap().trim(),
    ));
    let ir = grammar("@root Root ::= '(' [ Root ] ')' | D ;");
    let json = compile("../../examples/json/json.ubnf", &|p| fs::read_to_string(p))
        .unwrap()
        .ir;
    fs::create_dir(temp.0.join("src")).unwrap();
    for (name, ir) in [("nested", &ir), ("json", &json)] {
        ubnfc_rust::emit(
            ir,
            &ubnfc_rust::EmitOptions {
                emit_driver: false,
                ..Default::default()
            },
        )
        .unwrap()
        .write_to(temp.0.join("src").join(name))
        .unwrap();
    }
    fs::write(temp.0.join("Cargo.toml"), "[workspace]\n[package]\nname=\"ll1-check\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[features]\njson=[]\n").unwrap();
    fs::write(
        temp.0.join("src/main.rs"),
        include_str!("support/ll1_assertions.rs"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO"))
        .args(["run", "--offline", "--quiet"])
        .current_dir(&temp.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(env!("CARGO"))
        .args(["clippy", "--offline", "--", "-D", "warnings"])
        .current_dir(&temp.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
