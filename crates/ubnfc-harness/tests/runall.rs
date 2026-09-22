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
fn oracle_front_rejections_are_four_passes_with_separate_counts() {
    let workspace = build_front();
    let files = [
        "common-field-missing.json",
        "common-field-type.json",
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
    assert_eq!(report.totals.expected_rejection_pass, 4);
    assert_eq!(report.totals.pass, 0);
    assert_eq!(report.totals.ir_failure, 0);
    assert_eq!(report.totals.total(), 4);
    assert!(report.cases.iter().all(|row| row.outcome == Outcome::Pass));
    for (code, executions) in [
        ("E-LOWER-COMMON-FIELD", 2),
        ("E-LOWER-SUM-EVAL", 1),
        ("E-IMPORT-CYCLE", 1),
    ] {
        assert_eq!(
            report.refusals[&format!("ir/{code}")].executions,
            executions
        );
    }
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
