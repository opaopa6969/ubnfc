//! 生成物の内容で分離した、build_dir 寿命の Rust キャッシュ。
use super::{check_build, Error, Executable, Result, Scratch};
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
};

fn generator() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    for path in [
        std::env::var_os("CARGO_BIN_EXE_ubnfc-rust"),
        option_env!("CARGO_BIN_EXE_ubnfc-rust").map(Into::into),
    ]
    .into_iter()
    .flatten()
    {
        candidates.push(PathBuf::from(path));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(format!("ubnfc-rust{}", std::env::consts::EXE_SUFFIX)));
            if dir.file_name().is_some_and(|n| n == "deps") {
                candidates.push(
                    dir.parent()
                        .unwrap()
                        .join(format!("ubnfc-rust{}", std::env::consts::EXE_SUFFIX)),
                );
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    for profile in ["debug", "release"] {
        candidates.push(
            target
                .join(profile)
                .join(format!("ubnfc-rust{}", std::env::consts::EXE_SUFFIX)),
        );
    }
    candidates.iter().find(|p| p.is_file()).map(|p| p.canonicalize().map_err(Into::into)).unwrap_or_else(|| Err(Error(format!(
        "Rust 生成器がありません。cargo build -q --bin ubnfc-rust --offline を先に実行。探索パス: {}",
        candidates.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
    ))))
}
fn sources(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            sources(root, &path, out)?;
        } else {
            out.insert(
                path.strip_prefix(root)
                    .expect("generated source")
                    .to_owned(),
                fs::read(path)?,
            );
        }
    }
    Ok(())
}
pub(super) fn build(ir_path: &Path, build_dir: &Path) -> Result<Executable> {
    let generated = Scratch::create(Some(build_dir))?;
    let output = Command::new(generator()?)
        .arg("--ir")
        .arg(ir_path.canonicalize()?)
        .arg("--out")
        .arg(&generated.0)
        .arg("--emit-driver")
        .output()?;
    check_build("Rust IR generation", &output)?;
    let mut files = BTreeMap::new();
    sources(&generated.0, &generated.0, &mut files)?;
    let mut hash = DefaultHasher::new();
    files.hash(&mut hash);
    let key = format!("{:016x}", hash.finish());
    let cache = build_dir.join("rust-cache").join(&key);
    let program = cache.join(format!("driver{}", std::env::consts::EXE_SUFFIX));
    if program.is_file() {
        // hash 衝突でも別の生成物を実行しない。
        for (path, content) in &files {
            if fs::read(cache.join("sources").join(path))? != *content {
                return Err(Error("Rust 生成物キャッシュの hash 衝突".into()));
            }
        }
        return Ok(Executable {
            program,
            args: vec![],
            grammars: BTreeMap::new(),
        });
    }
    // 同じ hash で target を共有する。別文法の driver に上書きされない。
    let target = build_dir.join("rust-target").join(&key);
    let output = Command::new("cargo")
        .current_dir(&generated.0)
        .args([
            "build",
            "--release",
            "--offline",
            "--bin",
            "driver",
            "--message-format=json",
            "--target-dir",
        ])
        .arg(&target)
        .output()?;
    check_build("cargo build (Rust IR)", &output)?;
    let executable = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let row: serde_json::Value = serde_json::from_str(line).ok()?;
            (row["reason"] == "compiler-artifact" && row["target"]["name"] == "driver")
                .then(|| row["executable"].as_str().map(PathBuf::from))
                .flatten()
        })
        .next()
        .ok_or_else(|| Error("生成した driver executable がありません".into()))?;
    // 完成した cache だけを公開。並行した同一 hash の build は同じ内容になる。
    let ready = Scratch::create(Some(build_dir))?;
    for (path, content) in &files {
        let dest = ready.0.join("sources").join(path);
        fs::create_dir_all(dest.parent().expect("source parent"))?;
        fs::write(dest, content)?;
    }
    fs::copy(
        executable,
        ready.0.join(program.file_name().expect("driver filename")),
    )?;
    fs::create_dir_all(cache.parent().expect("cache parent"))?;
    if let Err(error) = fs::rename(&ready.0, &cache) {
        if !program.is_file() {
            return Err(error.into());
        }
    }
    Ok(Executable {
        program,
        args: vec![],
        grammars: BTreeMap::new(),
    })
}
