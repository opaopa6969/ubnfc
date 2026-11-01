//! 段階5。一般語列と語彙効果を分け、境界レポートの実測を CI で照合する。
use std::{path::Path, process::Command};

#[test]
fn stax_boundary_report_matches_both_generated_backends() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("bash")
        .arg("examples/stax/scripts/check-boundaries.sh")
        .current_dir(root)
        .env("RUSTUP_TOOLCHAIN", "stable")
        .output()
        .expect("stax の境界検証を開始する");
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
