use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use ubnfc_harness::{default_root, load_all};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let output = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(output.status.success());
        Self(PathBuf::from(
            String::from_utf8(output.stdout).unwrap().trim(),
        ))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn list_uses_explicit_root_and_environment_and_rejects_missing_corpus() {
    let binary = env!("CARGO_BIN_EXE_ubnfc-harness");
    let root = default_root().canonicalize().unwrap();
    let scratch = Scratch::new();
    // --root は環境変数より優先し、cwd を変えても明示パスはそのまま使う。
    let output = Command::new(binary)
        .args(["list", "--root"])
        .arg(&root)
        .env("UNLAXER_PARSER_DIR", &scratch.0)
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("| oracle | 107 | 398 |"));
    // rule-trivia corpus が 48 件増えた（tests/corpus.rs）。
    assert!(stdout.contains("| 合計 | 127 | 1279 |"), "{stdout}");
    let configured = Command::new(binary)
        .arg("list")
        .env("UNLAXER_PARSER_DIR", &root)
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    assert!(configured.status.success());
    assert_eq!(String::from_utf8(configured.stdout).unwrap(), stdout);
    let default = Command::new(binary)
        .arg("list")
        .env_remove("UNLAXER_PARSER_DIR")
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    let default_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../unlaxer-parser");
    if default_root.is_dir() {
        assert!(
            default.status.success(),
            "{}",
            String::from_utf8_lossy(&default.stderr)
        );
        assert_eq!(String::from_utf8(default.stdout).unwrap(), stdout);
    } else {
        // 隔離 worktree では sibling が無い。既定パスの明示エラーを検証する。
        // 実 corpus の成功は上の --root / 環境変数の両経路で検証済み。
        assert!(!default.status.success());
        assert!(default.stdout.is_empty());
        let expected =
            default_root.join("unlaxer-dsl/src/test/resources/nested-captures/corpus.json");
        assert!(String::from_utf8_lossy(&default.stderr).contains(&expected.display().to_string()));
    }
    let missing = Command::new(binary)
        .arg("list")
        .env("UNLAXER_PARSER_DIR", &scratch.0)
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("nested-captures/corpus.json"));
    assert!(!Command::new(binary)
        .args(["list", "--root"])
        .output()
        .unwrap()
        .status
        .success());
}

#[test]
fn missing_invalid_and_wrongly_typed_files_fail_with_path_and_json_line() {
    let scratch = Scratch::new();
    let path = scratch
        .0
        .join("unlaxer-dsl/src/test/resources/nested-captures/corpus.json");
    let missing = load_all(&scratch.0).unwrap_err();
    assert_eq!(missing.path, path);
    assert_eq!(missing.line, None);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "[\n  invalid\n]").unwrap();
    let malformed = load_all(&scratch.0).unwrap_err();
    assert_eq!(malformed.path, path);
    assert_eq!(malformed.line, Some(2));
    assert!(malformed.to_string().contains("corpus.json:2:"));
    fs::write(
        &path,
        "[\n {\"name\": false, \"grammar\": \"\", \"cases\": []}\n]",
    )
    .unwrap();
    let wrong_type = load_all(&scratch.0).unwrap_err();
    assert_eq!(wrong_type.path, path);
    assert_eq!(wrong_type.line, Some(2));
    assert!(wrong_type.to_string().contains("corpus.json:2:"));
}
