//! workspace 外へ生成し、同じプロセスで手書き目標と測定する。
use std::{fs, path::PathBuf, process::Command};
fn main() {
    let out = Command::new("mktemp")
        .args(["-d", "/tmp/ubnfc-bench.XXXXXX"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let dir = PathBuf::from(String::from_utf8(out.stdout).unwrap().trim());
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ir = ubnfc_front::ir::from_json(
        &fs::read_to_string(root.join("tests/ir/operators.ir.json")).unwrap(),
    )
    .unwrap();
    ubnfc_rust::emit(&ir, &Default::default())
        .unwrap()
        .write_to(dir.join("src/generated"))
        .unwrap();
    fs::write(dir.join("Cargo.toml"),format!("[package]\nname=\"generated-bench\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[features]\njson=[]\n[dependencies]\nubnfc-target-rust={{path={:?}}}\n",root.join("../../examples/handwritten-target/rust").canonicalize().unwrap())).unwrap();
    fs::write(
        dir.join("src/main.rs"),
        if std::env::var_os("UBNFC_BENCH_ALLOCATIONS").is_some() {
            include_str!("../templates/allocations.rs")
        } else {
            include_str!("../templates/bench.rs")
        },
    )
    .unwrap();
    eprintln!("benchmark crate: {}", dir.display());
    let status = Command::new(env!("CARGO"))
        .args(["run", "--offline", "--release", "--quiet"])
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());
    fs::remove_dir_all(dir).unwrap();
}
