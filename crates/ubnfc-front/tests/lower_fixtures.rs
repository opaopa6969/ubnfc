use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use ubnfc_front::{
    ir,
    lower::{lower, SemanticsIr, SourceInput},
    resolve::resolve,
    syntax::parse_file,
    validate::validate,
};

fn fixture_root(variable: &str, default: &str) -> PathBuf {
    let root = std::env::var_os(variable).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(default),
        PathBuf::from,
    );
    assert!(
        root.is_dir(),
        "fixture ディレクトリがありません: {} ({variable})",
        root.display()
    );
    root.canonicalize().unwrap()
}
fn run(root: &Path, relative: &str) -> SemanticsIr {
    let path = root.join(relative);
    let text = std::fs::read_to_string(&path).unwrap();
    let ast = parse_file(&text).unwrap();
    let inputs = RefCell::new(BTreeMap::from([(path.clone(), text)]));
    let grammar = resolve(&[(path, ast)], &|p| {
        let text = ubnfc_front::packages::read_to_string(p)?;
        inputs.borrow_mut().insert(p.to_path_buf(), text.clone());
        Ok(text)
    })
    .unwrap();
    let issues = validate(&grammar);
    assert!(
        issues.iter().all(|i| i.severity() != "ERROR"),
        "{relative}: {issues:?}"
    );
    let inputs: Vec<_> = inputs
        .into_inner()
        .into_iter()
        .map(|(p, t)| {
            SourceInput::new(
                p.clone(),
                p.strip_prefix(root).unwrap().to_str().unwrap(),
                t,
            )
        })
        .collect();
    let out = lower(&grammar, &inputs).unwrap_or_else(|e| panic!("{relative}: {e:#?}"));
    assert_eq!(out, lower(&grammar, &inputs).unwrap(), "決定性: {relative}");
    assert_eq!(
        out.semantics.rules.len(),
        grammar
            .grammars
            .iter()
            .map(|g| g.rules.len())
            .sum::<usize>()
    );
    assert_semantics_references(&out, relative);
    out
}
const FILES: &[&str] = &[
    "story/code/calc-var.ubnf",
    "story/code/calc.ubnf",
    "story/code/calc-leftrec.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/boundaries.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/elements.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/eval-default.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/keyword-boundary.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/namespace.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/common-field-space.ubnf",
    "unlaxer-dsl/src/test/resources/right-associative/Power.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Delimited.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Mixed.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Shared.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/0/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/1/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/2/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/3/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/associative/Operators.ubnf",
    "unlaxer-dsl/src/test/resources/cardinality/Cardinality.ubnf",
    "unlaxer-dsl/grammar/ubnf.ubnf",
    "unlaxer-dsl/tinycalc-vscode/grammar/tinycalc.ubnf",
    "unlaxer-dsl/examples/tinycalc.ubnf",
];

#[test]
fn valid_external_fixtures() {
    let root = fixture_root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser");
    assert_eq!(FILES.len(), 24);
    let mut count = 0;
    for file in FILES {
        if [
            "/all-tokens.ubnf",
            "/namespace.ubnf",
            "/positive/annotations.ubnf",
        ]
        .iter()
        .any(|s| file.ends_with(s))
        {
            continue;
        }
        let out = run(&root, file);
        if file.ends_with("calc-leftrec.ubnf") {
            assert_eq!(out.semantics.normalizations.len(), 2);
        }
        count += 1;
    }
    let tiny = fixture_root("TINYEXPRESSION_DIR", "../../../tinyexpression");
    let p4 = run(
        &tiny,
        "tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf",
    );
    assert_eq!(p4.semantics.rules.len(), 124);
    assert_eq!(p4.semantics.captures.len(), 246);
    assert_eq!(count, 21);
}
#[test]
fn tinycalc_example() {
    let root = fixture_root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser");
    let out = run(&root, "unlaxer-dsl/examples/tinycalc.ubnf");
    let example = ir::from_json(include_str!("../../../ir/examples/tinycalc.ir.json")).unwrap();
    let mut replaced = example.clone();
    replaced.semantics = out.semantics.clone();
    // 同一の analysis/plan に載せて canonical writer の byte 列で比較する。
    assert_eq!(
        ir::to_canonical_json(&replaced),
        ir::to_canonical_json(&example)
    );
    assert_eq!(out.sources, example.sources);
    assert_eq!(out.grammar_units, example.grammar_units);
    let actual = serde_json::to_value(&out.semantics).unwrap();
    let expected = serde_json::to_value(&example.semantics).unwrap();
    let mut differences = vec![];
    diff("", &expected, &actual, &mut differences);
    assert!(
        differences.is_empty(),
        "tinycalc semantics differences:\n{}",
        differences.join("\n")
    );
}
fn diff(path: &str, a: &serde_json::Value, b: &serde_json::Value, out: &mut Vec<String>) {
    if a == b {
        return;
    }
    match (a, b) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) if a.keys().eq(b.keys()) => {
            for (k, a) in a {
                diff(&format!("{path}/{k}"), a, &b[k], out);
            }
        }
        (serde_json::Value::Array(a), serde_json::Value::Array(b)) if a.len() == b.len() => {
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                diff(&format!("{path}/{i}"), a, b, out);
            }
        }
        _ => out.push(format!("{path}: {a} -> {b}")),
    }
}

// analysis/plan の未計算エラーのみを除外し、semantics の全参照を既存 validator
// で検査する。製品 API はこの仮の GrammarIr を生成・実行しない。
fn assert_semantics_references(out: &SemanticsIr, name: &str) {
    let mut ir = ir::from_json(include_str!("../../../ir/examples/tinycalc.ir.json")).unwrap();
    ir.semantics = out.semantics.clone();
    ir.sources = out.sources.clone();
    ir.grammar_units = out.grammar_units.clone();
    ir.import_edges = out.import_edges.clone();
    ir.entry = out.entries[0].clone();
    ir.analysis.expressions.clear();
    ir.analysis.rules.clear();
    ir.analysis.sccs.clear();
    ir.analysis.prefix_tries.clear();
    ir.plan.choices.clear();
    ir.plan.lexical.clear();
    ir.plan.memo.entries.clear();
    ir.plan.operators.clear();
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../ir/schema/grammar-ir.v1.schema.json")).unwrap();
    ir.required_features =
        serde_json::from_value(schema["properties"]["requiredFeatures"]["items"]["enum"].clone())
            .unwrap();
    ir.required_features
        .sort_by_key(|f| serde_json::to_string(f).unwrap());
    let errors = ir::validate_references(&ir);
    let semantic_errors: Vec<_> =
        errors
            .into_iter()
            .filter(|e| match e {
                ir::IrError::Validation { path, message } => !path.starts_with("/analysis/")
                    // import した symbol の sourceId は pipeline の finalize_origins が宣言元へ
                    // 付け替える（lower 単体の出力は統合先を指す）。import を持つ fixture
                    // （tinyexpression P4 の lexical module / std/layout）の完成した IR は
                    // pipeline_golden が検査する。
                    && !(!out.import_edges.is_empty()
                        && path.starts_with("/semantics/symbols/")
                        && message == "symbol source differs from its grammar source")
                    && !(path.starts_with("/semantics/recovery/")
                        && message
                            == "AUTO recovery requires analyzed FOLLOW; null is not an empty set"),
                _ => true,
            })
            .collect();
    assert!(semantic_errors.is_empty(), "{name}: {semantic_errors:#?}");
}
