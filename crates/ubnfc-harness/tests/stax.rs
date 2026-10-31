//! issue #7 段階 1。既存の JSON Lines driver と canonical AST 照合を CI でも必須にする。
use std::{path::Path, process::Command};

#[test]
fn stax_samples_have_equal_canonical_asts_in_rust_and_java() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("bash")
        .arg("examples/stax/scripts/crosscheck.sh")
        .current_dir(root)
        .env("RUSTUP_TOOLCHAIN", "stable")
        .output()
        .expect("stax の両 backend 検証を開始する");
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
