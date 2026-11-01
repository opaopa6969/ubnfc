//! 外部開発者 tree を使わず、子プロセス内だけで生成器 / javac を差し替える。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use ubnfc_fuzz::gen;
use ubnfc_harness::runner::Scratch;

fn script(path: &Path, source: &str) {
    std::fs::write(path, source).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

fn check_failure(backend: &str, message: &str) {
    let scratch = Scratch::new().unwrap();
    let json = scratch.0.join("report.json");
    let markdown = scratch.0.join("report.md");
    let generator = scratch.0.join("generator");
    script(
        &generator,
        include_str!("fixtures/failing-rust-generator.sh"),
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_ubnfc-fuzz"));
    command
        .args(["--start-seed", "1", "--seeds", "1", "--jobs", "1"])
        .arg("--findings")
        .arg(scratch.0.join("findings"))
        .arg("--json")
        .arg(&json)
        .arg("--report")
        .arg(&markdown)
        .env("UBNFC_RUST_GENERATOR", &generator)
        .env_remove("UNLAXER_PARSER_DIR")
        .env_remove("UNLAXER_PARSER_ROOT")
        .env_remove("TINYEXPRESSION_DIR");
    if backend == "java" {
        script(
            &scratch.0.join("javac"),
            "#!/bin/sh\nprintf 'issue54: 合成 javac の標準エラー\\n' >&2\nexit 1\n",
        );
        command.env(
            "PATH",
            std::env::join_paths(
                std::iter::once(scratch.0.clone())
                    .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
            )
            .unwrap(),
        );
    }
    let output = command.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    // CLI の既存の終了規則は変えず、discard / compiled / comparisons で未観測を検証する。
    assert!(output.status.success(), "{stderr}");
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(json).unwrap()).unwrap();
    assert_eq!(report["discarded"], 1);
    assert_eq!(report["discards"][format!("build:{backend}")], 1);
    assert_eq!(report["compiled"], 0);
    assert_eq!(report["comparisons"], 0);
    assert_eq!(report["findings"], serde_json::json!([]));
    let details = report["discard_details"].as_array().unwrap();
    assert_eq!(details.len(), 1);
    let detail = &details[0];
    let grammar = gen::generate(1);
    assert_eq!(detail["seed"], 1);
    assert_eq!(detail["grammar_name"], grammar.name);
    assert_eq!(detail["grammar_digest"], grammar.digest());
    assert_eq!(detail["grammar"], grammar.render());
    assert_eq!(detail["backend"], backend);
    let error = detail["message"].as_str().unwrap();
    assert!(error.contains(message), "{error}");
    if backend == "rust" {
        // Cargo の stderr と、stdout に含まれる rustc の JSON 診断の両方を保持する。
        assert!(error.contains("could not compile"), "{error}");
        assert!(error.contains("compiler-message"), "{error}");
    }
    let logged = stderr
        .lines()
        .find_map(|line| line.strip_prefix("ビルド失敗の詳細: "))
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(logged).unwrap(),
        *detail
    );
    let markdown = std::fs::read_to_string(markdown).unwrap();
    assert!(markdown.contains("seed 1"));
    assert!(markdown.contains(&grammar.digest()));
    assert!(markdown.contains(&format!("backend: `{backend}`")));
    assert!(markdown.contains(message));
    // CLI 終了時の一時ディレクトリ削除後も、ログと保存済み report だけで追跡できる。
    let workdir = stderr
        .lines()
        .find_map(|line| line.split_once("workdir=").map(|(_, path)| path))
        .unwrap();
    assert!(!Path::new(workdir).exists());
}

#[test]
fn rust_build_failure_keeps_seed_and_real_cargo_and_rustc_errors() {
    check_failure("rust", "issue54: 合成 fixture の意図したビルド失敗");
}

#[test]
fn java_build_failure_keeps_seed_and_javac_stderr() {
    check_failure("java", "issue54: 合成 javac の標準エラー");
}
