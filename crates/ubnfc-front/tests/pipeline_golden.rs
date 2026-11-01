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
/// golden は同梱 scanner registry の先頭文字宣言（`scanners/first-chars.json`）を写した IR。
fn first_chars() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scanners/first-chars.json")
        .canonicalize()
        .unwrap()
}
fn run(root: &Path, command: &str, relative: &str) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ubnfc"));
    cmd.current_dir(root).args([command, "--grammar", relative]);
    if command != "validate" {
        cmd.arg("--extern-first-chars").arg(first_chars());
    }
    cmd.output().unwrap()
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
        .args(["--out", "-", "--extern-first-chars"])
        .arg(first_chars())
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
    assert_eq!(typed.semantics.rules.len(), 130);
}

#[test]
fn three_invalid_fixtures_match_validation_text_goldens_and_json_counts() {
    let root = fixture_root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser");
    // (ファイル, ERROR 件数, WARNING 件数)。annotations.ubnf の 1 件は D-045 の
    // `@custom`（未知注釈）で、12 件の ERROR は Java 実装と同じまま。
    let cases = [
        (
            "rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf",
            1,
            0,
        ),
        (
            "rust/unlaxer-ubnf/tests/fixtures/known/namespace.ubnf",
            1,
            0,
        ),
        (
            "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
            12,
            1,
        ),
    ];
    for (file, errors, warnings) in cases {
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
        assert_eq!(value["severityCounts"]["WARNING"], warnings);
        assert_eq!(
            value["issues"].as_array().unwrap().len(),
            (errors + warnings) as usize
        );
    }
}

/// D-063: この repository の中にある v2 文法の golden。sibling repository に依存しない
/// 唯一の golden で、code point クラス（`\uXXXX` / `\u{X…}` の境界、補助面）が
/// IR の `charRange` としてどう落ちるかをここで固定する。
///
/// 再生成:
/// ```sh
/// cargo run -p ubnfc-front --bin ubnfc -- ir --grammar examples/json/json.ubnf \
///   --extern-first-chars scanners/first-chars.json --out ir/examples/json-v2.ir.json
/// ```
#[test]
fn the_json_v2_grammar_matches_its_committed_ir_golden() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let golden = root.join("ir/examples/json-v2.ir.json");
    let expected = std::fs::read(&golden).unwrap_or_else(|e| panic!("{}: {e}", golden.display()));
    let produced = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
        .current_dir(root.canonicalize().unwrap())
        .args(["ir", "--grammar", "examples/json/json.ubnf", "--out", "-"])
        .args(["--extern-first-chars"])
        .arg(first_chars())
        .output()
        .unwrap();
    assert!(
        produced.status.success(),
        "{}",
        String::from_utf8_lossy(&produced.stderr)
    );
    assert_eq!(
        produced.stdout, expected,
        "golden: ir/examples/json-v2.ir.json"
    );
    let typed = ir::from_json(std::str::from_utf8(&produced.stdout).unwrap()).unwrap();
    assert_eq!(typed.language_version, 2);
    assert!(ir::validate_references(&typed).is_empty());
    // RFC 8259 §7 の「生で置ける 1 文字」を 4 つの code point 範囲で覆う。
    let mut ranges: Vec<(u64, u64)> = typed
        .semantics
        .tokens
        .iter()
        .filter(|t| t.symbol_id.contains("::SAFE_"))
        .filter_map(|t| match t.spec {
            ir::TokenSpec::CharRange { min, max } => Some((min, max)),
            _ => None,
        })
        .collect();
    ranges.sort_unstable();
    assert_eq!(
        ranges,
        vec![
            (0x20, 0x21),
            (0x23, 0x5b),
            (0x5d, 0xd7ff),
            (0xe000, 0x10ffff)
        ]
    );
}

/// `examples/stax/stax.ubnf`（型の付いた連接言語、issue #66 stage 1）の golden。
/// sibling repository に依存しない 2 本目の v2 golden で、**scope 注釈**（`@scopeTree` /
/// `@declares` / `@symbolRef`）と `@catalog` / `@doc` が IR へどう落ちるかをここで固定する。
///
/// 再生成:
/// ```sh
/// cargo run -p ubnfc-front --bin ubnfc -- ir --grammar examples/stax/stax.ubnf \
///   --out ir/examples/stax.ir.json
/// ```
#[test]
fn the_stax_grammar_matches_its_committed_ir_golden() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let golden = root.join("ir/examples/stax.ir.json");
    let expected = std::fs::read(&golden).unwrap_or_else(|e| panic!("{}: {e}", golden.display()));
    let produced = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
        .current_dir(root.canonicalize().unwrap())
        .args(["ir", "--grammar", "examples/stax/stax.ubnf", "--out", "-"])
        .output()
        .unwrap();
    assert!(
        produced.status.success(),
        "{}",
        String::from_utf8_lossy(&produced.stderr)
    );
    assert_eq!(
        produced.stdout, expected,
        "golden: ir/examples/stax.ir.json"
    );
    let typed = ir::from_json(std::str::from_utf8(&produced.stdout).unwrap()).unwrap();
    assert_eq!(typed.language_version, 2);
    assert!(ir::validate_references(&typed).is_empty());
    // 語・型・struct フィールド・局所束縛の 4 種類の宣言と、語・型・フィールドの参照が
    // scope 効果として載っていること（これが「文法だけで記号表が出る」ことの根拠）。
    let declares = typed
        .semantics
        .scope
        .effects
        .iter()
        .flat_map(|effect| effect.events.iter())
        .filter(|event| matches!(event.action, ir::ScopeEffectEventsAction::Declare))
        .count();
    let uses = typed
        .semantics
        .scope
        .effects
        .iter()
        .flat_map(|effect| effect.events.iter())
        .filter(|event| matches!(event.action, ir::ScopeEffectEventsAction::Use))
        .count();
    assert!(declares >= 7, "宣言の scope 効果が少なすぎる: {declares}");
    assert!(uses >= 4, "参照の scope 効果が少なすぎる: {uses}");
}

/// stax の文法は `--strict` で警告ゼロでなければならない。順序付き選択の到達不能検査
/// （D-037、`W-CHOICE-*`）は、組込み語をリテラルで並べるこの文法の生命線である。
#[test]
fn the_stax_grammar_validates_strictly_clean() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
        .current_dir(&root)
        .args([
            "validate",
            "--grammar",
            "examples/stax/stax.ubnf",
            "--strict",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `examples/stax/samples/` の 20 本と `samples/expected.json` が 1 対 1 で対応すること。
/// 受理・AST の突き合わせ自体は backend の生成を伴うので `scripts/crosscheck.sh` が行う。
#[test]
fn the_stax_samples_all_have_an_expected_verdict() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/stax/samples");
    let expected = std::fs::read_to_string(dir.join("expected.json")).unwrap();
    let mut listed: Vec<String> = expected
        .lines()
        .filter_map(|line| line.split("\"file\": \"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_owned)
        .collect();
    let mut found: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".stax"))
        .collect();
    listed.sort();
    found.sort();
    assert_eq!(listed, found, "expected.json と samples/*.stax が食い違う");
    assert_eq!(found.len(), 20, "サンプルは 20 本");
}

/// `examples/formula-info/formula-info.ubnf`（行指向の FormulaInfo 文書、issue #83、
/// 出典は tinyexpression issue #180 / PR #191）の golden。sibling repository に依存しない
/// 3 本目の v2 golden で、`@whitespace: none` の行指向文法・規則レベルの否定先読みが無い
/// UBNF で「補集合」を書くパターン（`NEGATION` / `NEGATIVE_LOOKAHEAD` / `LOOKAHEAD`）が
/// IR へどう落ちるかをここで固定する。
///
/// 再生成:
/// ```sh
/// cargo run -p ubnfc-front --bin ubnfc -- ir --grammar examples/formula-info/formula-info.ubnf \
///   --out ir/examples/formula-info.ir.json
/// ```
#[test]
fn the_formula_info_grammar_matches_its_committed_ir_golden() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let golden = root.join("ir/examples/formula-info.ir.json");
    let expected = std::fs::read(&golden).unwrap_or_else(|e| panic!("{}: {e}", golden.display()));
    let produced = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
        .current_dir(root.canonicalize().unwrap())
        .args([
            "ir",
            "--grammar",
            "examples/formula-info/formula-info.ubnf",
            "--out",
            "-",
        ])
        .output()
        .unwrap();
    assert!(
        produced.status.success(),
        "{}",
        String::from_utf8_lossy(&produced.stderr)
    );
    assert_eq!(
        produced.stdout, expected,
        "golden: ir/examples/formula-info.ir.json"
    );
    let typed = ir::from_json(std::str::from_utf8(&produced.stdout).unwrap()).unwrap();
    assert_eq!(typed.language_version, 2);
    assert!(ir::validate_references(&typed).is_empty());
}

/// formula-info の文法は `--strict` で警告ゼロでなければならない
/// （issue #83、tinyexpression PR #191 で ubnfc 2 commit にわたって確認済み）。
#[test]
fn the_formula_info_grammar_validates_strictly_clean() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
        .current_dir(&root)
        .args([
            "validate",
            "--grammar",
            "examples/formula-info/formula-info.ubnf",
            "--strict",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `examples/formula-info/samples/` の 53 本と `samples/expected.json` が 1 対 1 で
/// 対応すること。受理・canonical AST の突き合わせ自体は backend の生成を伴うので
/// `scripts/crosscheck.sh` が行う。
#[test]
fn the_formula_info_samples_all_have_an_expected_verdict() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/formula-info/samples");
    let expected = std::fs::read_to_string(dir.join("expected.json")).unwrap();
    let mut listed: Vec<String> = expected
        .lines()
        .filter_map(|line| line.split("\"file\": \"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_owned)
        .collect();
    let mut found: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".fi"))
        .collect();
    listed.sort();
    found.sort();
    assert_eq!(listed, found, "expected.json と samples/*.fi が食い違う");
    assert_eq!(found.len(), 53, "サンプルは 53 本");
}
