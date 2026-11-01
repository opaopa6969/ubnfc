//! 段階5。Forth の標準上の期待と、現行 stax の parse / 実行観測を混同しない。
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use ubnfc_stax::generated::{api::ParseOptions, parse_with_options};

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn fixed_syntax_and_macro_boundaries_match_the_report() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let cases: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("boundaries/cases.json")).unwrap())
            .unwrap();
    let temp = Command::new("mktemp").arg("-d").output().unwrap();
    assert!(temp.status.success());
    let dir = String::from_utf8(temp.stdout).unwrap();
    let temporary = Temporary(PathBuf::from(dir.trim()));
    let file = temporary.0.join("boundary.stax");
    for case in cases["stax"].as_array().unwrap() {
        let text = case["source"].as_str().unwrap();
        let result = parse_with_options(text, ParseOptions::default());
        assert_eq!(
            result.ok,
            case["parse"].as_bool().unwrap(),
            "{}",
            case["id"]
        );
        fs::write(&file, text).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_stax"))
            .arg(&file)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            case["exit"].as_i64().map(|v| v as i32),
            "{}: {}",
            case["id"],
            String::from_utf8_lossy(&output.stderr)
        );
        if let Some(want) = case["stdout"].as_str() {
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                want,
                "{}",
                case["id"]
            );
            assert!(output.stderr.is_empty());
        } else {
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(case["error"].as_str().unwrap()),
                "{}: {}",
                case["id"],
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
