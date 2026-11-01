//! stax 段階4。既存生成器の LSP をプロトコルと回復配置の実測で検証する。
use std::{path::Path, process::Command};

#[test]
fn generated_stax_lsp_matches_protocol_and_recovery_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("bash")
        .arg("examples/stax/scripts/check-lsp.sh")
        .current_dir(root)
        .env("RUSTUP_TOOLCHAIN", "stable")
        .output()
        .expect("stax の LSP 検証を開始する");
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
