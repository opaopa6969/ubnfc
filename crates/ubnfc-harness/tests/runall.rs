use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use ubnfc_harness::{
    corpus::oracle,
    runall::{self, Config, Outcome},
    runner::{Backend, Scratch},
};

#[test]
fn associative_from_ubnf_has_24_pass_and_24_expected_rejection_pass() {
    associative("java");
}

#[test]
fn rust_associative_has_24_pass_and_builds_once() {
    associative("rust");
}

fn build_front() -> PathBuf {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let front = Command::new("cargo")
        .current_dir(&workspace)
        .args([
            "build",
            "--offline",
            "-q",
            "--bin",
            "ubnfc",
            "--bin",
            "ubnfc-rust",
        ])
        .output()
        .unwrap();
    assert!(
        front.status.success(),
        "{}",
        String::from_utf8_lossy(&front.stderr)
    );
    workspace
}

#[test]
fn tiny_fence_all_86_cases_pass_both_backends_with_raw_long_captures_and_diagnostics() {
    let workspace = build_front();
    let files: Vec<_> = ubnfc_harness::load_all(ubnfc_harness::default_root())
        .unwrap()
        .into_iter()
        .filter(|f| f.path.ends_with("tiny-code-fence/corpus.json"))
        .collect();
    assert_eq!(files.len(), 1);
    for backend in [Backend::Java, Backend::Rust] {
        let report = runall::run(
            &files,
            &Config {
                backend,
                front: workspace.join("target/debug/ubnfc"),
                rust_generator: Some(workspace.join("target/debug/ubnfc-rust")),
                cargo_jobs: Some(2),
                ..Config::default()
            },
        )
        .unwrap();
        // 元の 73 Pass と LONG の 13 件を、期待値・台帳の変更なしで全件照合する。
        assert_eq!(report.totals.total(), 86);
        assert_eq!(report.totals.pass, 86, "{}", report.markdown());
        let long: Vec<_> = report
            .cases
            .iter()
            .filter(|c| c.variant.starts_with("long-block"))
            .collect();
        assert_eq!(long.len(), 13);
        for case in long {
            let actual = case.actual.as_ref().unwrap();
            let ubnfc_harness::Expectation::Accept(expected) = &case.expected else {
                panic!("tiny fence の期待値は Accept");
            };
            if expected.accepted {
                // AST 値は終端改行なし。capture は LF/CRLF/CR・非 BMP を含む原文のまま。
                let start = if case.variant == "long-block-shifted" {
                    2
                } else {
                    0
                };
                let raw: String = case.input.chars().skip(start).collect();
                assert_eq!(
                    actual["captures"],
                    serde_json::json!([{
                        "name": "value", "span": [start, case.input.chars().count()], "text": raw
                    }]),
                    "{:?}: {}",
                    backend,
                    case.case_id
                );
                assert_eq!(actual["diagnostics"], serde_json::json!([]));
            } else {
                assert_eq!(actual["captures"], serde_json::json!([]));
                let diagnostics = actual["diagnostics"].as_array().unwrap();
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0]["kind"], "syntax");
                assert_eq!(diagnostics[0]["offset"], 0);
                assert_eq!(diagnostics[0]["farthestOffset"], 0);
                // Java は rule の候補も公開し、Rust の primary expected は terminal のみ。
                assert_eq!(
                    diagnostics[0]["expected"],
                    match backend {
                        Backend::Java => serde_json::json!(["RootParser", "long code block"]),
                        Backend::Rust => serde_json::json!(["long code block"]),
                    }
                );
                assert_eq!(
                    diagnostics[0]["farthestExpected"],
                    serde_json::json!(["RootParser", "long code block"])
                );
                assert_eq!(actual["consumed"], 0);
                assert_eq!(actual["matched"], 0);
            }
        }
    }
}

fn associative(backend: &str) {
    let workspace = build_front();
    let jar = Config::default().java_generator;
    if backend == "java" && !jar.is_file() {
        let build = Command::new("mvn")
            .current_dir(workspace.join("ubnfc-java"))
            .args(["-o", "-q", "-DskipTests", "package"])
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
    }
    let scratch = Scratch::new().unwrap();
    let json = scratch.0.join("nested/results.json");
    let markdown = scratch.0.join("nested/results.md");
    let output = Command::new(env!("CARGO_BIN_EXE_ubnfc-harness"))
        .args([
            "run-all",
            "--backend",
            backend,
            "--families",
            "associative",
            "--report",
        ])
        .arg(&markdown)
        .arg("--json")
        .arg(&json)
        .args(if backend == "rust" {
            vec!["--jobs", "2"]
        } else {
            vec![]
        })
        .env("UBNFC_BIN", workspace.join("target/debug/ubnfc"))
        .env("UBNFC_JAVA_GENERATOR", &jar)
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    // triage 末尾の訂正: 反転 metadata は既存 GrammarValidator も拒否する。
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&std::fs::read(json).unwrap()).unwrap();
    assert_eq!(report["source_cases"], 24);
    assert_eq!(report["grammar_variants"], 2);
    assert_eq!(report["unique_builds"], 2);
    assert_eq!(
        report["jobs"],
        if backend == "rust" {
            2
        } else {
            Config::default().jobs
        }
    );
    assert_eq!(report["totals"]["pass"], 24);
    assert_eq!(report["totals"]["expected_rejection_pass"], 24);
    for key in [
        "ir_failure",
        "generation_rejected",
        "javac_failure",
        "build_failure",
        "driver_error",
        "known_divergence",
        "fail",
        "unobserved",
    ] {
        assert_eq!(report["totals"][key], 0, "{key}: {report}");
    }
    assert_eq!(report["backend"], backend);
    if backend == "rust" {
        assert_eq!(report["rust_build"]["cargo_invocations"], 1);
        assert_eq!(report["rust_build"]["jobs"], 2);
        assert_eq!(report["rust_build"]["generated_crates"], 1);
        assert_eq!(report["rust_build"]["unique_crates"], 1);
        assert!(report["generator_sha256"]
            .as_str()
            .is_some_and(|s| s.len() == 64));
    }
    assert_eq!(report["refusals"]["ir/E-PRECEDENCE-ORDER"]["builds"], 1);
    assert!(std::fs::read_to_string(markdown)
        .unwrap()
        .contains("| associative | 0 | 0 | 0 | 0 | 24 | 24 | 0 | 0 | 0 |"));
}

#[test]
fn oracle_front_rejections_are_three_passes_with_separate_counts() {
    let workspace = build_front();
    // D-074: common-field-type は共通 field の型が揃って生成できるので、拒否の集計から外した
    // （`a_common_field_that_only_differed_by_numeric_capture_type_now_runs`）。
    let files = [
        "common-field-missing.json",
        "eval-sum.json",
        "import-cycle.json",
    ]
    .into_iter()
    .map(|name| oracle::load(&oracle::default_root().join(name)).unwrap())
    .collect::<Vec<_>>();
    let report = runall::run(
        &files,
        &Config {
            backend: Backend::Rust,
            front: workspace.join("target/debug/ubnfc"),
            rust_generator: Some(workspace.join("target/debug/ubnfc-rust")),
            ..Config::default()
        },
    )
    .unwrap();
    assert_eq!(report.totals.expected_rejection_pass, 3);
    assert_eq!(report.totals.pass, 0);
    assert_eq!(report.totals.ir_failure, 0);
    assert_eq!(report.totals.total(), 3);
    assert!(report.cases.iter().all(|row| row.outcome == Outcome::Pass));
    for (code, executions) in [
        ("E-LOWER-COMMON-FIELD", 1),
        ("E-LOWER-SUM-EVAL", 1),
        ("E-IMPORT-CYCLE", 1),
    ] {
        assert_eq!(
            report.refusals[&format!("ir/{code}")].executions,
            executions
        );
    }
}

/// D-074: 旧 oracle は `name` の型が String と int に割れて javac で落ちた。token の直接
/// capture は数値 token でも text なので型が揃い、生成・実行できる。oracle（生成失敗）との差は
/// `new-contract:direct-token-capture-text` として既知乖離になる（Fail ではない）。
#[test]
fn a_common_field_that_only_differed_by_numeric_capture_type_now_runs() {
    let workspace = build_front();
    // 台帳（divergences.json）を当てた corpus 全体から、この 1 ファイルだけを取り出す。
    let files: Vec<_> = ubnfc_harness::load_all(ubnfc_harness::default_root())
        .unwrap()
        .into_iter()
        .filter(|f| {
            f.path
                .file_name()
                .is_some_and(|n| n == "common-field-type.json")
        })
        .collect();
    assert_eq!(files.len(), 1);
    let report = runall::run(
        &files,
        &Config {
            backend: Backend::Rust,
            front: workspace.join("target/debug/ubnfc"),
            rust_generator: Some(workspace.join("target/debug/ubnfc-rust")),
            ..Config::default()
        },
    )
    .unwrap();
    assert_eq!(report.totals.total(), 1);
    assert_eq!(report.totals.expected_rejection_pass, 0);
    assert_eq!(report.totals.ir_failure, 0);
    assert!(
        report
            .cases
            .iter()
            .all(|row| row.outcome == Outcome::KnownDivergence),
        "{:?}",
        report.cases
    );
}

#[test]
fn invalid_cli_options_are_rejected_before_execution() {
    let binary = env!("CARGO_BIN_EXE_ubnfc-harness");
    for args in [
        vec!["--backend", "unknown"],
        vec!["--backend", "java", "--jobs", "0"],
        vec!["--backend", "java", "--jobs", "-1"],
        vec!["--backend", "java", "--jobs", "abc"],
        vec!["--backend", "java", "--families", "token,"],
        vec!["--backend", "java", "--jobs", "1", "--jobs", "2"],
        vec!["--backend", "java", "--report", "x", "--json", "x"],
        vec!["--backend", "java", "--unknown", "x"],
        vec!["--backend"],
    ] {
        let output = Command::new(binary)
            .arg("run-all")
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(!output.stderr.is_empty(), "{args:?}");
    }
}
