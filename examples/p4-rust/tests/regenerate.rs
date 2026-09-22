//! commit 済み `src/generated` が、同じ IR から同じ emitter で byte 一致で再生成できることを検証する。
//! crates/ubnfc-selfhost/tests/regenerate.rs と同じ方式（driver は生成しない、Cargo.toml は比較対象外）。
use std::path::{Path, PathBuf};
use std::process::Command;

struct Temporary(PathBuf);
impl Temporary {
    fn new() -> Self {
        let output = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(output.status.success());
        Self(PathBuf::from(
            String::from_utf8(output.stdout).unwrap().trim(),
        ))
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn files(root: &Path, current: &Path, result: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(current).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(root, &path, result);
        } else {
            result.push(path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
}

#[test]
fn p4_ir_regenerates_every_committed_byte() {
    let ir = ubnfc_front::ir::from_json(include_str!(
        "../../../ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json"
    ))
    .unwrap();
    let options = ubnfc_rust::EmitOptions {
        emit_driver: false,
        ..Default::default()
    };
    let generated = ubnfc_rust::emit(&ir, &options).unwrap();
    let temp = Temporary::new();
    generated.write_to(&temp.0).unwrap();
    let committed = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/generated");
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    files(&temp.0, &temp.0, &mut actual);
    files(&committed, &committed, &mut expected);
    // 生成 crate の Cargo.toml は単体 crate 用。ここでは module として埋め込むので比較対象外。
    actual.retain(|p| p != Path::new("Cargo.toml"));
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected, "余分な生成物・欠落も検出する");
    for file in actual {
        assert_eq!(
            std::fs::read(committed.join(&file)).unwrap(),
            std::fs::read(temp.0.join(&file)).unwrap(),
            "生成物が不一致: {}",
            file.display()
        );
    }
}
