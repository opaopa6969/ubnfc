//! 生成物の内容で分離した、build_dir 寿命の Rust キャッシュ。
use super::{check_build, Error, Executable, Result, Scratch};
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Instant,
};

pub(crate) fn generator() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("UBNFC_RUST_GENERATOR") {
        return PathBuf::from(path).canonicalize().map_err(Into::into);
    }
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
    let entry = CacheEntry::read(&generated.0, build_dir)?;
    if let Some(executable) = entry.cached()? {
        return Ok(executable);
    }
    // 同じ hash で target を共有する。別文法の driver に上書きされない。
    let target = build_dir.join("rust-target").join(&entry.key);
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
    entry.publish(&executable, build_dir)
}

struct CacheEntry {
    key: String,
    files: BTreeMap<PathBuf, Vec<u8>>,
    cache: PathBuf,
}
impl CacheEntry {
    fn read(generated: &Path, build_dir: &Path) -> Result<Self> {
        let mut files = BTreeMap::new();
        sources(generated, generated, &mut files)?;
        let mut hash = DefaultHasher::new();
        files.hash(&mut hash);
        let key = format!("{:016x}", hash.finish());
        Ok(Self {
            cache: build_dir.join("rust-cache").join(&key),
            key,
            files,
        })
    }
    fn executable(&self) -> Executable {
        Executable {
            program: self
                .cache
                .join(format!("driver{}", std::env::consts::EXE_SUFFIX)),
            args: vec![],
            grammars: BTreeMap::new(),
        }
    }
    fn cached(&self) -> Result<Option<Executable>> {
        let executable = self.executable();
        if !executable.program.is_file() {
            return Ok(None);
        }
        let root = self.cache.join("sources");
        let mut saved = BTreeMap::new();
        sources(&root, &root, &mut saved)?;
        if saved != self.files {
            return Err(Error("Rust 生成物キャッシュの hash 衝突".into()));
        }
        Ok(Some(executable))
    }
    fn publish(&self, executable: &Path, build_dir: &Path) -> Result<Executable> {
        // 完成した cache だけを公開。単体 build と一括 build は同じ原文を key にする。
        let ready = Scratch::create(Some(build_dir))?;
        for (path, content) in &self.files {
            let dest = ready.0.join("sources").join(path);
            fs::create_dir_all(dest.parent().expect("source parent"))?;
            fs::write(dest, content)?;
        }
        let program = self.executable().program;
        fs::copy(
            executable,
            ready.0.join(program.file_name().expect("driver filename")),
        )?;
        fs::create_dir_all(self.cache.parent().expect("cache parent"))?;
        if let Err(error) = fs::rename(&ready.0, &self.cache) {
            if !program.is_file() {
                return Err(error.into());
            }
        }
        self.cached()?
            .ok_or_else(|| Error("Rust cache がありません".into()))
    }
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct BatchStats {
    /// None は Cargo の既定値・環境設定。
    pub jobs: Option<usize>,
    pub generated_crates: usize,
    pub unique_crates: usize,
    pub cache_hits: usize,
    pub cargo_invocations: usize,
    pub initial_build_seconds: f64,
    /// 失敗 crate の個別切り分けを含む cargo の実測時間。
    pub total_build_seconds: f64,
}

pub(crate) struct BatchBuild {
    pub results: Vec<Result<Executable>>,
    pub stats: BatchStats,
}

/// 生成器が出す固定形式の manifest のみを書き換える。ソースと lib 名は維持する。
fn member_manifest(files: &BTreeMap<PathBuf, Vec<u8>>, package: &str) -> Result<String> {
    let manifest = files
        .get(Path::new("Cargo.toml"))
        .ok_or_else(|| Error("生成物に Cargo.toml がありません".into()))?;
    let manifest = std::str::from_utf8(manifest).map_err(|e| Error(e.to_string()))?;
    for required in [
        "name = \"ubnfc-generated\"",
        "[workspace]",
        "[lib]",
        "name = \"driver\"",
    ] {
        if manifest.matches(required).count() != 1 {
            return Err(Error(format!("生成 manifest の形式が不正です: {required}")));
        }
    }
    Ok(manifest
        .replacen(
            "name = \"ubnfc-generated\"",
            &format!("name = {package:?}"),
            1,
        )
        .replacen("[workspace]", "", 1)
        .replacen("[lib]", "[lib]\nname = \"ubnfc_generated\"", 1)
        .replacen(
            "name = \"driver\"",
            &format!("name = \"{package}-driver\""),
            1,
        ))
}

fn artifacts(output: &Output) -> BTreeMap<String, PathBuf> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let row: serde_json::Value = serde_json::from_str(line).ok()?;
            if row["reason"] != "compiler-artifact" {
                return None;
            }
            Some((
                row["target"]["name"].as_str()?.to_owned(),
                PathBuf::from(row["executable"].as_str()?),
            ))
        })
        .collect()
}
fn cargo_log(output: &Output) -> String {
    let mut log = String::from_utf8_lossy(&output.stderr).into_owned();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(row) => {
                if let Some(rendered) = row["message"]["rendered"].as_str() {
                    log.push_str(rendered);
                }
            }
            Err(_) => {
                log.push_str(line);
                log.push('\n');
            }
        }
    }
    format!("cargo build 失敗 ({}):\n{log}", output.status)
}

/// 全生成 crate を一時 workspace に置き、cargo に並列度を委ねる。
/// 成功 artifact は失敗した workspace build からも回収し、残りを -p で個別診断する。
pub(crate) fn build_batch(
    generated: &[PathBuf],
    build_dir: &Path,
    jobs: Option<usize>,
) -> Result<BatchBuild> {
    fs::create_dir_all(build_dir)?;
    let build_dir = build_dir.canonicalize()?;
    let workspace = Scratch::create(Some(&build_dir))?;
    let mut stats = BatchStats {
        jobs,
        generated_crates: generated.len(),
        ..BatchStats::default()
    };
    let mut results: Vec<Option<Result<Executable>>> = (0..generated.len()).map(|_| None).collect();
    let mut entries: BTreeMap<String, (CacheEntry, Vec<usize>)> = BTreeMap::new();
    for (index, directory) in generated.iter().enumerate() {
        let entry = match CacheEntry::read(directory, &build_dir) {
            Ok(entry) => entry,
            Err(error) => {
                results[index] = Some(Err(error));
                continue;
            }
        };
        if let Some((existing, indices)) = entries.get_mut(&entry.key) {
            if existing.files != entry.files {
                return Err(Error("Rust 生成物キャッシュの hash 衝突".into()));
            }
            indices.push(index);
            continue;
        }
        entries.insert(entry.key.clone(), (entry, vec![index]));
    }
    stats.unique_crates = entries.len();
    let mut pending = Vec::new();
    for (entry, indices) in entries.into_values() {
        let prepared = (|| -> Result<Option<Executable>> {
            if let Some(executable) = entry.cached()? {
                return Ok(Some(executable));
            }
            let package = format!("ubnfc-runall-{}", entry.key);
            let manifest = member_manifest(&entry.files, &package)?;
            let member = workspace.0.join(&package);
            for (path, content) in &entry.files {
                let dest = member.join(path);
                fs::create_dir_all(dest.parent().expect("source parent"))?;
                fs::write(dest, content)?;
            }
            fs::write(member.join("Cargo.toml"), manifest)?;
            Ok(None)
        })();
        match prepared {
            Ok(None) => pending.push((entry, indices)),
            Ok(Some(executable)) => {
                stats.cache_hits += indices.len();
                for index in indices {
                    results[index] = Some(Ok(executable.clone()));
                }
            }
            Err(error) => {
                for index in indices {
                    results[index] = Some(Err(Error(error.to_string())));
                }
            }
        }
    }
    if !pending.is_empty() {
        let members: Vec<_> = pending
            .iter()
            .map(|(e, _)| format!("ubnfc-runall-{}", e.key))
            .collect();
        fs::write(
            workspace.0.join("Cargo.toml"),
            format!(
                "[workspace]\nresolver = \"2\"\nmembers = {}\n",
                serde_json::to_string(&members).unwrap()
            ),
        )?;
        let cargo = |package: Option<&str>, stats: &mut BatchStats| {
            let mut cmd = Command::new("cargo");
            cmd.current_dir(&workspace.0)
                .args([
                    "build",
                    "--release",
                    "--offline",
                    "--bins",
                    "--keep-going",
                    "--message-format=json",
                    "--target-dir",
                ])
                .arg(build_dir.join("rust-batch-target"));
            if let Some(package) = package {
                cmd.args(["-p", package]);
            } else {
                cmd.arg("--workspace");
            }
            if let Some(jobs) = jobs {
                cmd.arg("--jobs").arg(jobs.to_string());
            }
            let start = Instant::now();
            let output = cmd.output();
            let elapsed = start.elapsed().as_secs_f64();
            stats.cargo_invocations += 1;
            stats.total_build_seconds += elapsed;
            if package.is_none() {
                stats.initial_build_seconds = elapsed;
            }
            output
        };
        let output = cargo(None, &mut stats);
        let binaries = output.as_ref().map(artifacts).unwrap_or_default();
        for (entry, indices) in pending {
            let package = format!("ubnfc-runall-{}", entry.key);
            let binary = format!("{package}-driver");
            let result = if let Some(program) = binaries.get(&binary) {
                entry.publish(program, &build_dir)
            } else {
                match cargo(Some(&package), &mut stats) {
                    Err(error) => Err(error.into()),
                    Ok(output) if !output.status.success() => Err(Error(
                        cargo_log(&output)
                            .replace(workspace.0.to_string_lossy().as_ref(), "<workspace>"),
                    )),
                    Ok(output) => match artifacts(&output).get(&binary) {
                        Some(program) => entry.publish(program, &build_dir),
                        None => Err(Error("生成した driver executable がありません".into())),
                    },
                }
            };
            for index in indices {
                results[index] = Some(match &result {
                    Ok(executable) => Ok(executable.clone()),
                    Err(error) => Err(Error(error.to_string())),
                });
            }
        }
    }
    Ok(BatchBuild {
        results: results
            .into_iter()
            .map(|r| r.expect("all builds recorded"))
            .collect(),
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated(root: &Path, name: &str, value: &str) -> PathBuf {
        let path = root.join(name);
        fs::create_dir_all(path.join("src/bin")).unwrap();
        fs::write(path.join("Cargo.toml"), "[package]\nname = \"ubnfc-generated\"\nversion = \"0.0.0\"\nedition = \"2021\"\nautobins = false\n[workspace]\n[lib]\npath = \"mod.rs\"\n[[bin]]\nname = \"driver\"\npath = \"src/bin/driver.rs\"\n").unwrap();
        fs::write(
            path.join("mod.rs"),
            format!("pub fn value() -> &'static str {{ {value:?} }}"),
        )
        .unwrap();
        fs::write(
            path.join("src/bin/driver.rs"),
            "fn main() { println!(\"{}\", ubnfc_generated::value()); }",
        )
        .unwrap();
        path
    }

    fn stdout(executable: &Executable) -> String {
        let output = Command::new(&executable.program).output().unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    }

    #[test]
    fn batch_isolates_failed_crates_and_keeps_hash_cache_and_driver_identity() {
        let scratch = Scratch::new().unwrap();
        let a = generated(&scratch.0, "a", "alpha");
        let b = generated(&scratch.0, "b", "beta");
        let broken = generated(&scratch.0, "broken", "broken");
        fs::write(
            broken.join("mod.rs"),
            "compile_error!(\"intentional batch failure\");",
        )
        .unwrap();
        let malformed = generated(&scratch.0, "malformed", "malformed");
        fs::write(malformed.join("Cargo.toml"), "invalid manifest").unwrap();
        let manifest = fs::read(a.join("Cargo.toml")).unwrap();
        let build_dir = scratch.0.join("build");
        let batch = build_batch(
            &[a.clone(), b.clone(), broken, a.clone(), malformed],
            &build_dir,
            Some(2),
        )
        .unwrap();
        assert_eq!(batch.stats.generated_crates, 5);
        assert_eq!(batch.stats.unique_crates, 4);
        assert_eq!(
            batch.stats.cargo_invocations, 2,
            "workspace + failed package only"
        );
        assert_eq!(stdout(batch.results[0].as_ref().unwrap()), "alpha\n");
        assert_eq!(stdout(batch.results[1].as_ref().unwrap()), "beta\n");
        assert!(batch.results[2]
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("intentional batch failure"));
        assert_eq!(
            batch.results[0].as_ref().unwrap().program,
            batch.results[3].as_ref().unwrap().program
        );
        assert!(batch.results[4]
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("manifest"));
        assert_eq!(fs::read(a.join("Cargo.toml")).unwrap(), manifest);
        let cached = build_batch(&[a.clone(), b, a.clone()], &build_dir, None).unwrap();
        assert_eq!(cached.stats.cache_hits, 3);
        assert_eq!(cached.stats.cargo_invocations, 0);
        assert_eq!(stdout(cached.results[0].as_ref().unwrap()), "alpha\n");
        fs::write(
            a.join("mod.rs"),
            "pub fn value() -> &'static str { \"changed\" }",
        )
        .unwrap();
        let changed = build_batch(&[a], &build_dir, None).unwrap();
        assert_eq!(changed.stats.cache_hits, 0);
        assert_eq!(changed.stats.cargo_invocations, 1);
        assert_eq!(stdout(changed.results[0].as_ref().unwrap()), "changed\n");
        assert_eq!(stdout(batch.results[0].as_ref().unwrap()), "alpha\n");
    }
}
