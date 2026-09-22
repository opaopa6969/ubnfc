use serde_json::Value;
use std::{path::Path, process::Command};
use ubnfc_harness::{runall::Config, runner::Scratch};

#[test]
fn associative_from_ubnf_has_24_pass_and_reports_rejected_metadata_variant() {
    associative("java");
}

#[test]
fn rust_associative_has_24_pass_and_builds_once() {
    associative("rust");
}

fn associative(backend: &str) {
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
    // RustConformanceTest.java:391-455 は反転 metadata でも正常生成・実行を要求する。
    // front の E-PRECEDENCE-ORDER を隠す、期待値を変える、variant を除外することはしない。
    assert_eq!(
        output.status.code(),
        Some(1),
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
    assert_eq!(report["totals"]["ir_failure"], 24);
    for key in [
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
        .contains("| associative | 24 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |"));
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
