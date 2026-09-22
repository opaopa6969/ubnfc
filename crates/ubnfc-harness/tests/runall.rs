use serde_json::Value;
use std::{path::Path, process::Command};
use ubnfc_harness::{runall::Config, runner::Scratch};

#[test]
fn associative_from_ubnf_has_24_pass_and_reports_rejected_metadata_variant() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let front = Command::new("cargo")
        .current_dir(&workspace)
        .args(["build", "--offline", "-q", "--bin", "ubnfc"])
        .output()
        .unwrap();
    assert!(
        front.status.success(),
        "{}",
        String::from_utf8_lossy(&front.stderr)
    );
    let jar = workspace.join("ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar");
    if !jar.is_file() {
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
            "java",
            "--families",
            "associative",
            "--report",
        ])
        .arg(&markdown)
        .arg("--json")
        .arg(&json)
        .env("UBNFC_BIN", workspace.join("target/debug/ubnfc"))
        .env("UBNFC_JAVA_GENERATOR", &jar)
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    // 全文法を実行するため、正規文法に加えて precedence 順を反転する負の variant も含む。
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
    assert_eq!(report["jobs"], Config::default().jobs);
    assert_eq!(report["totals"]["pass"], 24);
    assert_eq!(report["totals"]["ir_failure"], 24);
    for key in [
        "generation_rejected",
        "javac_failure",
        "driver_error",
        "known_divergence",
        "fail",
        "unobserved",
    ] {
        assert_eq!(report["totals"][key], 0, "{key}: {report}");
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
        vec!["--backend", "rust"],
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
