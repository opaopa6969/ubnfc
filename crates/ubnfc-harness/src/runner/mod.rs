//! オフライン build と JSON Lines driver。実行値を oracle から補完しない。
mod comparison;
mod report;
pub(crate) mod rust;
pub(crate) mod scanners;
pub use comparison::{compare, unobserved};
pub use report::Report;

use crate::{Case, CorpusFile, GrammarSource};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Rust,
    Java,
}

#[derive(Debug)]
pub struct Error(pub String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;

/// mktemp -d が確保した、この呼出し専用の一時ディレクトリ。
#[derive(Debug)]
pub struct Scratch(pub PathBuf);
impl Scratch {
    pub fn new() -> Result<Self> {
        Self::create(None)
    }
    fn persist(mut self) -> PathBuf {
        std::mem::take(&mut self.0)
    }
    fn create(parent: Option<&Path>) -> Result<Self> {
        let mut command = Command::new("mktemp");
        command.arg("-d");
        if let Some(parent) = parent {
            command.arg(parent.join("harness.XXXXXXXXXX"));
        }
        let output = command.output()?;
        if !output.status.success() {
            return Err(Error(String::from_utf8_lossy(&output.stderr).into_owned()));
        }
        Ok(Self(PathBuf::from(
            String::from_utf8_lossy(&output.stdout).trim(),
        )))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.0.as_os_str().is_empty() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Options {
    pub lexical: bool,
    pub scope: bool,
}

/// artifact は一つの variant を実装する。観測指定は文法の選択・変形には使わない。
#[derive(Clone, Debug)]
pub struct RunCase {
    pub case: Case,
    pub grammar: String,
    pub variant: String,
    pub family: String,
    pub options: Options,
    /// 元 fixture の観測指定。captureNames 等を期待値から逆算しない。
    pub fixture: Value,
    /// loader の型付き Oracle 以外も raw から保持する。
    pub extra_expectations: BTreeMap<String, Value>,
}
impl RunCase {
    pub fn label(&self) -> String {
        format!("{}:{}[{}]", self.family, self.case.id, self.variant)
    }
    fn request(&self, id: usize) -> Value {
        // probe が必要とする fixture metadata。期待値や評価結果は渡さない。
        // NestedCaptureConformanceTest.java:283-300、ScopeAnnotationConformanceTest.java:108-110。
        let mut observations = serde_json::Map::new();
        observations.insert("family".into(), json!(self.family));
        observations.insert("variant".into(), json!(self.variant));
        for key in [
            "captureNames",
            "queries",
            "valueSpanField",
            "valueSpanListField",
        ] {
            if let Some(value) = self.fixture.get(key) {
                observations.insert(key.into(), value.clone());
            }
        }
        if let crate::Expectation::Ast(expect) = &self.case.expect {
            observations.insert(
                "parentRollback".into(),
                json!(expect.parent_rollback.is_some()),
            );
        }
        json!({"id": id, "grammar": self.grammar, "entry": self.case.entry_rule,
            "input": self.case.input, "options": self.options, "observations": observations})
    }
}

/// fixture ディレクトリ名、または6系統の serde 名で選択する。
/// 既定は各ケースの先頭 grammar_ref。--variant は artifact の実装 variant を宣言する。
pub fn select(
    files: &[CorpusFile],
    families: &[String],
    variant: Option<&str>,
) -> Result<Vec<RunCase>> {
    let mut seen = vec![false; families.len()];
    let mut selected = Vec::new();
    for file in files {
        let family = file
            .path
            .parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            .unwrap_or("corpus");
        let family_kind = serde_json::to_value(file.family).expect("Family");
        let mut include = families.is_empty();
        for (i, filter) in families.iter().enumerate() {
            if filter == family || family_kind.as_str() == Some(filter) {
                seen[i] = true;
                include = true;
            }
        }
        if !include {
            continue;
        }
        for case in &file.cases {
            let mut extra_expectations = BTreeMap::new();
            if matches!(case.expect, crate::Expectation::Oracle { .. }) {
                if let Some(row) = file.raw["cases"]
                    .as_array()
                    .and_then(|rows| rows.iter().find(|row| row["id"] == case.id))
                {
                    for (key, value) in row.as_object().expect("oracle row") {
                        if ![
                            "id",
                            "input",
                            "accepted",
                            "allConsumed",
                            "consumed",
                            "matched",
                            "ast",
                            "captures",
                            "declarations",
                            "diagnostics",
                            "mappingError",
                        ]
                        .contains(&key.as_str())
                        {
                            extra_expectations.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            if case.grammar_ref.is_empty() {
                selected.push(RunCase {
                    case: case.clone(),
                    grammar: String::new(),
                    variant: String::new(),
                    family: family.into(),
                    options: Options::default(),
                    fixture: Value::Null,
                    extra_expectations,
                });
                continue;
            }
            let grammar = case
                .grammar_ref
                .iter()
                .filter_map(|&i| file.grammars.get(i))
                .find(|grammar| variant.is_none_or(|v| grammar_name(grammar) == v));
            let Some(grammar) = grammar else {
                return Err(Error(format!(
                    "{}: 実行可能な文法/variant がありません",
                    case.id
                )));
            };
            let source = match grammar {
                GrammarSource::Inline { source, .. } => source.clone(),
                GrammarSource::File { path, .. } => fs::read_to_string(path)?,
            };
            // 文法名は variant の表示名ではなく UBNF の grammar 宣言から取得する。
            let words: Vec<_> = source
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .filter(|s| !s.is_empty())
                .collect();
            let name = words
                .windows(2)
                .find(|pair| pair[0] == "grammar")
                .map(|pair| pair[1])
                .ok_or_else(|| Error(format!("{}: grammar 宣言がありません", case.id)))?;
            selected.push(RunCase {
                case: case.clone(),
                grammar: name.into(),
                variant: grammar_name(grammar).into(),
                family: family.into(),
                options: Options {
                    lexical: true,
                    scope: true,
                },
                fixture: match grammar {
                    GrammarSource::Inline { metadata, .. }
                    | GrammarSource::File { metadata, .. } => metadata.clone(),
                },
                extra_expectations,
            });
        }
    }
    if let Some((i, _)) = seen.iter().enumerate().find(|(_, found)| !**found) {
        return Err(Error(format!("未知の系統: {}", families[i])));
    }
    if selected.is_empty() {
        return Err(Error("実行ケースがありません".into()));
    }
    Ok(selected)
}
fn grammar_name(grammar: &GrammarSource) -> &str {
    match grammar {
        GrammarSource::File { name, .. } | GrammarSource::Inline { name, .. } => name,
    }
}

#[derive(Clone, Debug)]
pub struct Actual {
    pub backend: Backend,
    pub variant: String,
    /// 欠落キー（未観測）と明示 null を区別する。受信行をそのまま保持する。
    pub response: Value,
}
impl Actual {
    fn error(backend: Backend, case: &RunCase, error: impl ToString) -> Self {
        Self {
            backend,
            variant: case.variant.clone(),
            response: json!({"error": error.to_string()}),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Executable {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    /// Rust の <grammar>-driver。空なら全ケースを program に dispatch。
    pub grammars: BTreeMap<String, PathBuf>,
}

/// 既存ソース、または生成する Grammar IR。FromIr は Java / Rust の生成器を呼ぶ。
#[derive(Clone, Debug)]
pub enum Artifact {
    Prebuilt { dir: PathBuf },
    FromIr { ir_path: PathBuf, backend: Backend },
}
// 従来の runner.build(path) 呼出しも維持する。
impl<P: AsRef<Path>> From<P> for Artifact {
    fn from(dir: P) -> Self {
        Self::Prebuilt {
            dir: dir.as_ref().to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Runner {
    pub backend: Backend,
    pub build_dir: PathBuf,
}
impl Runner {
    pub fn build(&self, artifact: impl Into<Artifact>) -> Result<Executable> {
        self.build_impl(artifact.into(), None)
    }

    /// 生成器の jar/classes を明示する。テストでは target/classes を再利用する。
    pub fn build_with_java_generator(
        &self,
        artifact: impl Into<Artifact>,
        classpath: impl AsRef<Path>,
    ) -> Result<Executable> {
        self.build_impl(artifact.into(), Some(classpath.as_ref()))
    }

    fn build_impl(&self, artifact: Artifact, java_generator: Option<&Path>) -> Result<Executable> {
        fs::create_dir_all(&self.build_dir)?;
        let build_dir = self.build_dir.canonicalize()?;
        // 生成ソースは javac の終了まで保持し、失敗時も Scratch が削除する。
        let mut generated = None;
        let artifact = match artifact {
            Artifact::Prebuilt { dir } => dir.canonicalize()?,
            Artifact::FromIr { ir_path, backend } => {
                if backend != self.backend {
                    return Err(Error("Artifact と Runner の backend が一致しません".into()));
                }
                if backend == Backend::Rust {
                    return rust::build(&ir_path, &build_dir);
                }
                let ir_path = ir_path.canonicalize()?;
                let default_jar = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar");
                let generator = java_generator.unwrap_or(&default_jar);
                if !generator.exists() {
                    return Err(Error(format!(
                        "Java 生成器がありません: {} (cd ubnfc-java && mvn -o -q package を先に実行)",
                        generator.display()
                    )));
                }
                // D-033: 既定 jar は手で build する成果物なので、driver template や emitter を
                // 直したまま package し忘れると「古い契約の driver」が黙って動き、
                // 差分 fuzzing に存在しない乖離が出る（2026-09-23 の `diff:/catalogs` /
                // `diff:/diagnostics/*` の誤報）。ソースより古い jar は使わず、ここで落とす。
                if java_generator.is_none() {
                    check_java_generator_freshness(generator)?;
                }
                let sources = Scratch::create(Some(&build_dir))?;
                let output = Command::new("java")
                    .arg("-cp")
                    .arg(generator.canonicalize()?)
                    .args(["org.unlaxer.ubnfc.java.Main", "--ir"])
                    .arg(ir_path)
                    .arg("--out")
                    .arg(sources.0.join("src/main/java"))
                    .args(["--package", "target", "--emit-driver"])
                    .output()?;
                check_build("Java IR generation", &output)?;
                scanners::java(&sources.0)?;
                let path = sources.0.clone();
                generated = Some(sources);
                path
            }
        };
        let _generated = generated;
        match self.backend {
            Backend::Rust => {
                let output = Command::new("cargo")
                    .current_dir(&artifact)
                    .args([
                        "build",
                        "--release",
                        "--offline",
                        "--bins",
                        "--message-format=json",
                        "--manifest-path",
                    ])
                    .arg(artifact.join("Cargo.toml"))
                    .arg("--target-dir")
                    .arg(build_dir.join("rust"))
                    .output()?;
                check_build("cargo build", &output)?;
                let mut binaries = BTreeMap::new();
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    if let Ok(row) = serde_json::from_str::<Value>(line) {
                        if row["reason"] == "compiler-artifact" {
                            if let (Some(name), Some(path)) =
                                (row["target"]["name"].as_str(), row["executable"].as_str())
                            {
                                if name == "driver" || name.ends_with("-driver") {
                                    binaries.insert(name.to_owned(), PathBuf::from(path));
                                }
                            }
                        }
                    }
                }
                if let Some(program) = binaries.remove("driver") {
                    Ok(Executable {
                        program,
                        args: vec![],
                        grammars: BTreeMap::new(),
                    })
                } else {
                    let program = binaries.values().next().cloned().ok_or_else(|| {
                        Error("driver または <grammar>-driver bin が必要です".into())
                    })?;
                    Ok(Executable {
                        program,
                        args: vec![],
                        grammars: binaries
                            .into_iter()
                            .map(|(name, path)| {
                                (name.trim_end_matches("-driver").to_lowercase(), path)
                            })
                            .collect(),
                    })
                }
            }
            Backend::Java => {
                let mut sources = Vec::new();
                java_sources(&artifact.join("src/main/java"), &mut sources)?;
                sources.sort();
                if sources.is_empty() {
                    return Err(Error("Java ソースがありません".into()));
                }
                // 毎回新しい classes にし、前回の class を成功と誤認しない。
                let classes = Scratch::create(Some(&build_dir))?;
                let output = Command::new("javac")
                    .current_dir(&artifact)
                    .args(["-encoding", "UTF-8", "-d"])
                    .arg(&classes.0)
                    .args(sources)
                    .output()?;
                check_build("javac", &output)?;
                if !classes.0.join("target/driver/Driver.class").is_file() {
                    return Err(Error("target.driver.Driver がありません".into()));
                }
                let scanner_registry = artifact
                    .join("src/main/java/ubnfc/scanners/Registry.java")
                    .is_file();
                let classes = classes.persist();
                let mut args = vec![
                    OsString::from("-cp"),
                    classes.clone().into_os_string(),
                    OsString::from("target.driver.Driver"),
                ];
                if scanner_registry {
                    args.push("--scanners".into());
                    args.push(classes.into_os_string());
                }
                // classes の寿命は呼出元の build_dir と同じ。CLI/test の Scratch が削除する。
                Ok(Executable {
                    program: "java".into(),
                    args,
                    grammars: BTreeMap::new(),
                })
            }
        }
    }

    /// 各 executable に一度だけ起動し、全リクエストを送る。結果は入力順に戻す。
    pub fn run(&self, exec: &Executable, cases: &[RunCase]) -> Vec<Actual> {
        let mut actual: Vec<_> = cases
            .iter()
            .map(|case| Actual::error(self.backend, case, "未実行"))
            .collect();
        let mut groups = BTreeMap::<PathBuf, Vec<usize>>::new();
        for (i, case) in cases.iter().enumerate() {
            if case.grammar.is_empty() {
                actual[i] = Actual {
                    backend: self.backend,
                    variant: case.variant.clone(),
                    response: json!({}),
                };
                continue;
            }
            let program = if exec.grammars.is_empty() {
                Some(&exec.program)
            } else {
                exec.grammars.get(&case.grammar.to_lowercase())
            };
            if let Some(program) = program {
                groups.entry(program.clone()).or_default().push(i);
            } else {
                actual[i] = Actual::error(
                    self.backend,
                    case,
                    format!("{} の driver がありません", case.grammar),
                );
            }
        }
        for (program, indices) in groups {
            let requests: Vec<_> = indices.iter().map(|&i| cases[i].request(i)).collect();
            match exchange(&program, &exec.args, &requests) {
                Ok(mut responses) => {
                    for i in indices {
                        actual[i] = Actual {
                            backend: self.backend,
                            variant: cases[i].variant.clone(),
                            response: responses
                                .remove(&i)
                                .unwrap_or_else(|| json!({"error":"応答行がありません"})),
                        };
                    }
                }
                Err(error) => {
                    for i in indices {
                        actual[i] = Actual::error(self.backend, &cases[i], &error);
                    }
                }
            }
        }
        actual
    }
}
fn java_sources(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            java_sources(&entry.path(), out)?;
        } else if entry.path().extension().is_some_and(|ext| ext == "java") {
            out.push(entry.path());
        }
    }
    Ok(())
}
fn check_build(name: &str, output: &std::process::Output) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    Err(Error(format!(
        "{name} 失敗 ({}):\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    )))
}
/// 既定 jar が `ubnfc-java` のソースより古くないかを見る（D-033）。
/// driver template は jar の resource なので、package し忘れた jar は
/// 「古い観測契約の driver」を生成する。それを黙って使わない。
fn check_java_generator_freshness(jar: &Path) -> Result<()> {
    let Ok(built) = fs::metadata(jar).and_then(|meta| meta.modified()) else {
        return Ok(()); // mtime が読めない環境では検査しない
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ubnfc-java");
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    let mut stack = vec![root.join("src/main"), root.join("pom.xml")];
    while let Some(path) = stack.pop() {
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if meta.is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            stack.extend(entries.flatten().map(|entry| entry.path()));
            continue;
        }
        let Ok(modified) = meta.modified() else {
            continue;
        };
        if newest.as_ref().is_none_or(|(at, _)| modified > *at) {
            newest = Some((modified, path));
        }
    }
    match newest {
        Some((modified, path)) if modified > built => Err(Error(format!(
            "Java 生成器 jar がソースより古い: {} より {} が新しい \
             (cd ubnfc-java && mvn -o -q -DskipTests package を先に実行)",
            jar.display(),
            path.display()
        ))),
        _ => Ok(()),
    }
}

fn exchange(
    program: &Path,
    args: &[OsString],
    requests: &[Value],
) -> Result<BTreeMap<usize, Value>> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    // stdin 書込みと stdout/stderr の排出を並行し、pipe 容量を超えてもデッドロックしない。
    let (written, output) = std::thread::scope(|scope| {
        let writer = scope.spawn(move || -> std::io::Result<()> {
            for request in requests {
                writeln!(stdin, "{request}")?;
            }
            Ok(())
        });
        let output = child.wait_with_output();
        (writer.join().expect("stdin writer"), output)
    });
    let output = output?;
    if !output.status.success() {
        return Err(Error(format!(
            "driver 終了 {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    written?;
    let stdout = String::from_utf8(output.stdout).map_err(|error| Error(error.to_string()))?;
    let ids: Vec<_> = requests
        .iter()
        .map(|r| r["id"].as_u64().unwrap() as usize)
        .collect();
    let mut responses = BTreeMap::new();
    for (line, text) in stdout.lines().enumerate() {
        let row: Value = serde_json::from_str(text)
            .map_err(|error| Error(format!("driver 行 {}: {error}", line + 1)))?;
        let id = row
            .get("id")
            .and_then(Value::as_u64)
            .and_then(|id| usize::try_from(id).ok())
            .ok_or_else(|| Error(format!("driver 行 {}: id が不正です", line + 1)))?;
        if !ids.contains(&id) || responses.insert(id, row).is_some() {
            return Err(Error(format!("driver 行 {}: 未知/重複 id {id}", line + 1)));
        }
    }
    Ok(responses)
}
