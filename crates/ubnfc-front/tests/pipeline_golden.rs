use std::path::{Path, PathBuf};
use std::process::Command;

use ubnfc_front::ir;

fn fixture_root(variable: &str, default: &str) -> PathBuf {
    let root = std::env::var_os(variable).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(default),
        PathBuf::from,
    );
    assert!(
        root.is_dir(),
        "fixture がありません: {} ({variable})",
        root.display()
    );
    root.canonicalize().unwrap()
}
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ir/fixtures")
}
fn run(root: &Path, command: &str, relative: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ubnfc"))
        .current_dir(root)
        .args([command, "--grammar", relative])
        .output()
        .unwrap()
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

const P4: &str = "tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf";

fn check_ir(root: &Path, relative: &str) -> Vec<u8> {
    let golden = golden_dir().join(format!("{}.ir.json", relative.replace('/', "__")));
    let expected = std::fs::read(&golden).unwrap_or_else(|e| panic!("{}: {e}", golden.display()));
    let first = run(root, "ir", relative);
    assert!(
        first.status.success(),
        "{relative}: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, expected, "golden: {relative}");
    // 別プロセス・絶対パス・異なる cwd でも同一。hash seed/配置に依存しない。
    let second = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["ir", "--grammar"])
        .arg(root.join(relative))
        .args(["--out", "-"])
        .output()
        .unwrap();
    assert!(
        second.status.success(),
        "{relative}: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(first.stdout, second.stdout, "determinism: {relative}");
    let typed = ir::from_json(std::str::from_utf8(&first.stdout).unwrap()).unwrap();
    assert!(ir::validate_references(&typed).is_empty(), "{relative}");
    assert_eq!(
        ir::to_canonical_json(&typed).as_bytes(),
        first.stdout,
        "typed round trip: {relative}"
    );
    first.stdout
}

#[test]
fn all_21_valid_fixtures_match_cli_goldens_and_tinycalc_example() {
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
        let bytes = check_ir(&root, file);
        if *file == "unlaxer-dsl/examples/tinycalc.ubnf" {
            assert_eq!(
                bytes,
                include_bytes!("../../../ir/examples/tinycalc.ir.json")
            );
        }
        count += 1;
    }
    assert_eq!(count, 21);
}

#[test]
fn p4_matches_cli_golden_and_typed_round_trip() {
    let root = fixture_root("TINYEXPRESSION_DIR", "../../../tinyexpression");
    let bytes = check_ir(&root, P4);
    let typed = ir::from_json(std::str::from_utf8(&bytes).unwrap()).unwrap();
    assert_eq!(typed.semantics.rules.len(), 124);
}

#[test]
fn three_invalid_fixtures_match_validation_text_goldens_and_json_counts() {
    let root = fixture_root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser");
    let cases = [
        (
            "rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf",
            1,
        ),
        ("rust/unlaxer-ubnf/tests/fixtures/known/namespace.ubnf", 1),
        (
            "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
            12,
        ),
    ];
    for (file, errors) in cases {
        let out = run(&root, "validate", file);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stderr.is_empty());
        let expected = std::fs::read(
            golden_dir()
                .join("errors")
                .join(format!("{}.txt", file.replace('/', "__"))),
        )
        .unwrap();
        assert_eq!(out.stdout, expected, "{file}");
        assert_eq!(run(&root, "validate", file).stdout, out.stdout);
        let json = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
            .current_dir(&root)
            .args([
                "validate",
                "--grammar",
                file,
                "--format",
                "json",
                "--strict",
            ])
            .output()
            .unwrap();
        assert_eq!(json.status.code(), Some(1));
        let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
        assert_eq!(value["severityCounts"]["ERROR"], errors);
        assert_eq!(value["severityCounts"]["WARNING"], 0);
        assert_eq!(value["issues"].as_array().unwrap().len(), errors as usize);
    }
}
