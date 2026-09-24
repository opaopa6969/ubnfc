//! 全 corpus の .ubnf → IR → Java/Rust → build → driver → 独立 oracle 比較。
//! loader が既存 Conformance テストと同じ形に合成した文法を再解釈しない。
#[cfg(test)]
mod rejection_tests;
mod report;
mod rust;
pub use crate::runner::rust::BatchStats;
pub use report::{Counts, Outcome, RefusalCounts, Report};

use crate::{
    canonical::Difference,
    divergence::Verdict,
    runner::{self, Actual, Backend, Error, Result, RunCase, Runner, Scratch},
    CorpusFile, Expectation, GrammarSource,
};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Debug)]
pub struct Config {
    pub backend: Backend,
    pub rust_generator: Option<PathBuf>,
    /// Rust の並列度。未指定時は Cargo の既定値・環境設定を使う。
    pub cargo_jobs: Option<usize>,
    pub front: PathBuf,
    pub java_generator: PathBuf,
    pub families: Vec<String>,
    pub jobs: usize,
}
impl Default for Config {
    fn default() -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        Self {
            backend: Backend::Java,
            rust_generator: None,
            cargo_jobs: None,
            front: std::env::var_os("UBNFC_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| workspace.join("target/debug/ubnfc")),
            java_generator: std::env::var_os("UBNFC_JAVA_GENERATOR")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    workspace.join("ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar")
                }),
            families: Vec::new(),
            jobs: 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    #[default]
    NotRun,
    Success,
    Failed,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct StageResult {
    pub status: StageStatus,
    pub reasons: Vec<String>,
    pub output: String,
}
impl StageResult {
    fn success() -> Self {
        Self {
            status: StageStatus::Success,
            ..Self::default()
        }
    }
    fn failed(stage: &str, output: String) -> Self {
        Self {
            status: StageStatus::Failed,
            reasons: reasons(stage, &output),
            output,
        }
    }
    fn command(stage: &str, output: std::io::Result<Output>) -> Self {
        match output {
            Err(error) => Self::failed(stage, format!("起動失敗: {error}")),
            Ok(output) => {
                let log = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stderr),
                    String::from_utf8_lossy(&output.stdout)
                );
                if output.status.success() {
                    Self {
                        output: log,
                        ..Self::success()
                    }
                } else {
                    Self::failed(stage, format!("終了 {}:\n{log}", output.status))
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildResult {
    pub id: String,
    pub source_hash: String,
    pub grammars: Vec<String>,
    /// 一時 .ubnf を削除しても再現できるように原文と import を保存する。
    pub source: String,
    pub imports: BTreeMap<String, String>,
    pub ir: StageResult,
    pub generation: StageResult,
    pub javac: StageResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo: Option<StageResult>,
}
#[derive(Clone, Debug, Serialize)]
pub struct CaseResult {
    pub corpus: String,
    pub case_id: String,
    pub family: String,
    pub kind: String,
    pub grammar: String,
    pub variant: String,
    pub build_id: Option<String>,
    pub input: String,
    pub expected: Expectation,
    pub extra_expectations: BTreeMap<String, Value>,
    pub known_divergences: Vec<crate::divergence::Observation>,
    pub outcome: Outcome,
    pub actual: Option<Value>,
    pub differences: Vec<Difference>,
    pub missing: Vec<String>,
}
impl CaseResult {
    fn new(corpus: &str, kind: &str, grammar: &str, case: &RunCase) -> Self {
        Self {
            corpus: corpus.into(),
            case_id: case.case.id.clone(),
            family: case.family.clone(),
            kind: kind.into(),
            grammar: grammar.into(),
            variant: case.variant.clone(),
            build_id: None,
            input: case.case.input.clone(),
            expected: case.case.expect.clone(),
            extra_expectations: case.extra_expectations.clone(),
            known_divergences: case.case.known_divergences.clone(),
            outcome: Outcome::Unobserved,
            actual: None,
            differences: Vec::new(),
            missing: Vec::new(),
        }
    }
    fn observe(&mut self, case: &RunCase, actual: Actual) {
        self.missing = runner::unobserved(case, &actual);
        self.outcome = if actual.response.get("error").is_some() {
            Outcome::DriverError
        } else {
            match runner::compare(case, &actual, &case.case.known_divergences) {
                Verdict::Pass if self.missing.is_empty() => Outcome::Pass,
                Verdict::Pass => Outcome::Unobserved,
                Verdict::KnownDivergence(diffs) => {
                    self.differences = diffs;
                    Outcome::KnownDivergence
                }
                Verdict::Fail(diffs) => {
                    self.differences = diffs;
                    Outcome::Fail
                }
            }
        };
        self.actual = Some(actual.response);
    }
}

struct Work {
    grammar: GrammarSource,
    build: BuildResult,
    cases: Vec<RunCase>,
    rows: Vec<CaseResult>,
}
struct Plan {
    files: usize,
    variants: usize,
    work: Vec<Work>,
    without_grammar: Vec<CaseResult>,
}

/// source hash は安定した FNV-1a/64。cache key の原文照合で衝突を区別する。
pub fn source_hash(source: &str) -> String {
    let hash = source.bytes().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    });
    format!("{hash:016x}")
}
fn plan(files: &[CorpusFile], filters: &[String], backend: Backend) -> Result<Plan> {
    // 未知の filter と参照切れを既存 select と同じ契約で拒否する。
    runner::select(files, filters, None)?;
    let mut plan = Plan {
        files: 0,
        variants: 0,
        work: Vec::new(),
        without_grammar: Vec::new(),
    };
    let mut cache: BTreeMap<(String, String), usize> = BTreeMap::new();
    for file in files {
        let family = file
            .path
            .parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                Error(format!(
                    "corpus の系統名がありません: {}",
                    file.path.display()
                ))
            })?;
        let kind = serde_json::to_value(file.family)
            .expect("Family")
            .as_str()
            .unwrap()
            .to_owned();
        if !filters.is_empty()
            && !filters
                .iter()
                .any(|filter| filter == family || filter == &kind)
        {
            continue;
        }
        plan.files += 1;
        let corpus = format!(
            "{family}/{}",
            file.path.file_name().unwrap().to_string_lossy()
        );
        let mut subset = file.clone();
        subset.cases.retain(|case| case.grammar_ref.is_empty());
        if !subset.cases.is_empty() {
            for case in runner::select(&[subset], &[], None)? {
                let mut row = CaseResult::new(&corpus, &kind, &format!("{corpus}:文法なし"), &case);
                row.missing = runner::unobserved(
                    &case,
                    &Actual {
                        backend,
                        variant: case.variant.clone(),
                        response: serde_json::json!({}),
                    },
                );
                row.missing
                    .push("/grammar (操作列の driver protocol 未対応)".into());
                plan.without_grammar.push(row);
            }
        }
        for (index, grammar) in file.grammars.iter().enumerate() {
            plan.variants += 1;
            let (name, source, imports) = source_parts(grammar)?;
            let label = format!("{corpus}:{index}[{name}]");
            // import 原文も key に含める。同一 main source の import-alias/two-level を混同しない。
            // 外部 .ubnf の相対 import は親ディレクトリが解決文脈になる。
            let context = match grammar {
                GrammarSource::File { path, .. } if source.contains("@import") => path
                    .canonicalize()?
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                _ => String::new(),
            };
            let identity = serde_json::to_string(&(&source, &imports, context)).unwrap();
            let hash = source_hash(&identity);
            let key = (hash.clone(), identity);
            let work_index = if let Some(&existing) = cache.get(&key) {
                existing
            } else {
                let next = plan.work.len();
                let id = format!("{hash}-{next:03}");
                plan.work.push(Work {
                    grammar: grammar.clone(),
                    build: BuildResult {
                        id,
                        source_hash: source_hash(&source),
                        grammars: Vec::new(),
                        source,
                        imports,
                        ir: StageResult::default(),
                        generation: StageResult::default(),
                        javac: StageResult::default(),
                        cargo: None,
                    },
                    cases: Vec::new(),
                    rows: Vec::new(),
                });
                cache.insert(key, next);
                next
            };
            let work = &mut plan.work[work_index];
            work.build.grammars.push(label.clone());
            let mut subset = file.clone();
            subset
                .cases
                .retain(|case| case.grammar_ref.contains(&index));
            for case in &mut subset.cases {
                case.grammar_ref = vec![index];
            }
            if !subset.cases.is_empty() {
                for mut case in runner::select(&[subset], &[], None)? {
                    expect_rejection(&corpus, &mut case, backend);
                    let mut row = CaseResult::new(&corpus, &kind, &label, &case);
                    row.build_id = Some(work.build.id.clone());
                    work.cases.push(case);
                    work.rows.push(row);
                }
            }
        }
    }
    Ok(plan)
}

/// 元の採取値は保持し、case × variant に展開してから拒否契約を付ける。
/// backend 固有の契約（Rust の非正則 REGEX 拒否）は当該 backend にだけ登録する。
fn expect_rejection(corpus: &str, case: &mut RunCase, backend: Backend) {
    let (stage, code, evidence) = match (corpus, case.variant.as_str(), case.case.id.as_str()) {
        // docs/design/03-design-v1.md §3.2: Rust は正則部分集合のみ DFA 化し、後方参照等の
        // 非正則 REGEX は生成時エラー。Java は java.util.regex.Pattern へ委譲するので runtime 比較。
        ("oracle/regex-backref.json" | "oracle/reobserved-regex-backref.json", _, "regex-backref/1")
            if backend == Backend::Rust =>
        {
            (
                "generation",
                "regex",
                "docs/design/03-design-v1.md §3.2（Rust: 非正則 REGEX は生成時エラー）; crates/ubnfc-rust の regex 生成拒否「requires an analyzed DFA; Java Pattern fallback is unavailable」",
            )
        }
        ("associative/corpus.json", "reversed-metadata", _) => (
            "validate",
            "E-PRECEDENCE-ORDER",
            "docs/reports/2026-09-22-triage.md 末尾の訂正; GrammarValidator.java:917–945",
        ),
        // common-field-type/1 は D-074（token の直接 capture は数値 token でも text）で
        // 共通 field の型が揃い、生成できるようになった。拒否契約から外し、oracle の
        // 生成失敗との差は divergences.json の `new-contract:direct-token-capture-text`。
        ("oracle/common-field-missing.json", _, "common-field-missing/1") => (
            "ir",
            "E-LOWER-COMMON-FIELD",
            "docs/reports/2026-09-22-triage.md 末尾の訂正; corpus/oracle/common-field-missing.json compilation.diagnostics",
        ),
        ("oracle/eval-sum.json", _, "eval-sum/1") => (
            "ir",
            "E-LOWER-SUM-EVAL",
            "docs/reports/2026-09-22-triage.md 末尾の訂正（sum eval の早期拒否を維持）",
        ),
        // 循環 import は loader が採取済み apiError を確認して登録済み。
        _ => return,
    };
    // triage 末尾の訂正: 「同じ文法を既存 GrammarValidator.validate に直接かけると
    // E-PRECEDENCE-ORDER で拒否される」。commonField 2 件・sum eval 1 件も早期拒否を維持。
    // D-017 の既存バグに対する既知乖離ではなく、指定の段階・コードだけを Pass にする。
    case.case.expect = Expectation::ExpectedRejection {
        stage: stage.into(),
        code: code.into(),
        evidence: evidence.into(),
        original: Box::new(case.case.expect.clone()),
    };
}

fn source_parts(grammar: &GrammarSource) -> Result<(String, String, BTreeMap<String, String>)> {
    let (name, source, metadata) = match grammar {
        GrammarSource::Inline {
            name,
            source,
            metadata,
        } => (name, source.clone(), metadata),
        GrammarSource::File {
            name,
            path,
            metadata,
        } => (name, fs::read_to_string(path)?, metadata),
    };
    let imports = match metadata.get("imports") {
        None => BTreeMap::new(),
        Some(value) => {
            serde_json::from_value(value.clone()).map_err(|e| Error(format!("imports: {e}")))?
        }
    };
    Ok((name.clone(), source, imports))
}

/// File は元の .ubnf を直接渡す。inline は専用 scratch に原文のまま保存する。
pub fn materialize(grammar: &GrammarSource, directory: &Path) -> Result<PathBuf> {
    if let GrammarSource::File { path, .. } = grammar {
        return Ok(path.canonicalize()?);
    }
    let (_, source, imports) = source_parts(grammar)?;
    fs::create_dir_all(directory)?;
    for (name, text) in imports {
        let path = Path::new(&name);
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || name == "__runall__.ubnf"
        {
            return Err(Error(format!("不正な import 相対パス: {name}")));
        }
        let target = directory.join(path);
        fs::create_dir_all(target.parent().unwrap())?;
        fs::write(target, text)?;
    }
    let path = directory.join("__runall__.ubnf");
    fs::write(&path, source)?;
    Ok(path)
}

/// build を単位に並列化する。文法の重複は実行前に束ね、生成・javac・driver 起動は各一度。
pub fn run(files: &[CorpusFile], config: &Config) -> Result<Report> {
    if config.jobs == 0 || config.cargo_jobs == Some(0) {
        return Err(Error("--jobs は1以上が必要です".into()));
    }
    let mut config = config.clone();
    config.front = config.front.canonicalize().map_err(|e| {
        Error(format!(
            "ubnfc がありません: {} ({e}); cargo build --bin ubnfc または UBNFC_BIN を指定",
            config.front.display()
        ))
    })?;
    let generator = match config.backend {
        Backend::Java => {
            config.java_generator = config.java_generator.canonicalize().map_err(|e| Error(format!(
                "Java 生成器がありません: {} ({e}); mvn -o -q -DskipTests package または UBNFC_JAVA_GENERATOR を指定", config.java_generator.display())))?;
            config.java_generator.clone()
        }
        Backend::Rust => {
            let path = match &config.rust_generator {
                Some(path) => path.canonicalize()?,
                None => runner::rust::generator()?,
            };
            config.rust_generator = Some(path.clone());
            path
        }
    };
    let generator_sha256 = sha256(&generator)?;
    let front_sha256 = sha256(&config.front)?;
    let plan = plan(files, &config.families, config.backend)?;
    let scratch = Scratch::new()?;
    let mut report = match config.backend {
        Backend::Java => run_java(plan, &config, &scratch.0)?,
        Backend::Rust => rust::run(plan, &config, &scratch.0)?,
    };
    if sha256(&generator)? != generator_sha256 || sha256(&config.front)? != front_sha256 {
        return Err(Error(
            "実行中に generator/front が更新されました。固定したコピーで再実行してください".into(),
        ));
    }
    report.generator_sha256 = Some(generator_sha256);
    report.front_sha256 = Some(front_sha256);
    Ok(report)
}

fn run_java(plan: Plan, config: &Config, directory: &Path) -> Result<Report> {
    let next = AtomicUsize::new(0);
    let mut results = std::thread::scope(|scope| -> Result<Vec<_>> {
        let workers: Vec<_> = (0..config.jobs.min(plan.work.len()))
            .map(|_| {
                scope.spawn(|| {
                    let mut results = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(work) = plan.work.get(index) else {
                            break;
                        };
                        let result = execute(work, config, &directory.join(&work.build.id));
                        results.push((index, result));
                    }
                    results
                })
            })
            .collect();
        let mut results = Vec::new();
        for worker in workers {
            results.extend(
                worker
                    .join()
                    .map_err(|_| Error("run-all worker が panic しました".into()))?,
            );
        }
        Ok(results)
    })?;
    results.sort_by_key(|(index, _)| *index);
    let mut builds = Vec::new();
    let mut cases = plan.without_grammar;
    for (_, (build, rows)) in results {
        builds.push(build);
        cases.extend(rows);
    }
    Ok(Report::new(
        config.backend,
        config.jobs,
        plan.files,
        plan.variants,
        builds,
        cases,
    ))
}

fn sha256(path: &Path) -> Result<String> {
    let output = Command::new("sha256sum").arg(path).output()?;
    let digest = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_owned();
    if !output.status.success()
        || digest.len() != 64
        || !digest.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(Error(format!(
            "SHA-256 を取得できません: {}",
            path.display()
        )));
    }
    Ok(digest)
}
fn prepare_ir(
    work: &Work,
    config: &Config,
    directory: &Path,
) -> std::result::Result<(PathBuf, String), StageResult> {
    let grammar = materialize(&work.grammar, &directory.join("grammar"));
    let ir_path = directory.join("grammar.ir.json");
    let ir = match grammar {
        Ok(grammar) => {
            // File の場合も build の出力先だけは用意する。
            match fs::create_dir_all(directory) {
                Ok(()) => StageResult::command(
                    "ir",
                    Command::new(&config.front)
                        .args(["ir", "--grammar"])
                        .arg(grammar)
                        .arg("--out")
                        .arg(&ir_path)
                        .output(),
                ),
                Err(error) => StageResult::failed("ir", error.to_string()),
            }
        }
        Err(error) => StageResult::failed("ir", error.to_string()),
    };
    if ir.status == StageStatus::Failed {
        return Err(ir);
    }
    // wire の grammar は構文解析済み IR の entry から得る（コメント中の grammar と混同しない）。
    let ir = fs::read(&ir_path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string()));
    let grammar_name = ir.and_then(|ir| {
        ir["grammarUnits"]
            .as_array()
            .and_then(|units| {
                units
                    .iter()
                    .find(|unit| unit["id"] == ir["entry"]["grammarId"])
            })
            .and_then(|unit| unit["name"].as_str())
            .map(str::to_owned)
            .ok_or_else(|| "IR に entry grammar がありません".into())
    });
    let grammar_name = grammar_name.map_err(|error| StageResult::failed("ir", error))?;
    Ok((ir_path, grammar_name))
}

fn execute(work: &Work, config: &Config, directory: &Path) -> (BuildResult, Vec<CaseResult>) {
    let mut build = work.build.clone();
    let mut rows = work.rows.clone();
    let (ir_path, grammar_name) = match prepare_ir(work, config, directory) {
        Ok(prepared) => {
            build.ir = StageResult::success();
            prepared
        }
        Err(error) => {
            build.ir = error;
            return stopped(build, rows, Outcome::IrFailure, directory);
        }
    };
    let artifact = directory.join("generated");
    build.generation = StageResult::command(
        "generation",
        Command::new("java")
            .arg("-cp")
            .arg(&config.java_generator)
            .args(["org.ubnfc.java.Main", "--ir"])
            .arg(&ir_path)
            .arg("--out")
            .arg(artifact.join("src/main/java"))
            .args(["--package", "target", "--emit-driver"])
            .output(),
    );
    if build.generation.status == StageStatus::Failed {
        return stopped(build, rows, Outcome::GenerationRejected, directory);
    }
    if let Err(error) = runner::scanners::java(&artifact) {
        build.javac = StageResult::failed("javac", error.to_string());
        return stopped(build, rows, Outcome::JavacFailure, directory);
    }
    let runner = Runner {
        backend: Backend::Java,
        build_dir: directory.join("build"),
    };
    match runner.build(&artifact) {
        Err(error) => {
            build.javac = StageResult::failed("javac", error.to_string());
            stopped(build, rows, Outcome::JavacFailure, directory)
        }
        Ok(executable) => {
            build.javac = StageResult::success();
            let mut cases = work.cases.clone();
            for case in &mut cases {
                case.grammar.clone_from(&grammar_name);
            }
            for ((row, case), actual) in rows
                .iter_mut()
                .zip(&cases)
                .zip(runner.run(&executable, &cases))
            {
                row.observe(case, actual);
            }
            normalize_logs(&mut build, directory);
            (build, rows)
        }
    }
}
fn stopped(
    mut build: BuildResult,
    mut rows: Vec<CaseResult>,
    outcome: Outcome,
    directory: &Path,
) -> (BuildResult, Vec<CaseResult>) {
    for row in &mut rows {
        row.outcome = outcome;
        if let Expectation::ExpectedRejection { stage, code, .. } = &row.expected {
            let (actual_stage, result) = match outcome {
                // ubnfc ir は validate も実行する。このコードは検証器の拒否。
                Outcome::IrFailure
                    if build.ir.reasons.iter().any(|c| c == "E-PRECEDENCE-ORDER") =>
                {
                    ("validate", &build.ir)
                }
                Outcome::IrFailure => ("ir", &build.ir),
                Outcome::GenerationRejected => ("generation", &build.generation),
                Outcome::JavacFailure => ("javac", &build.javac),
                Outcome::BuildFailure => ("build", build.cargo.as_ref().expect("cargo failure")),
                _ => unreachable!("stopped は build 段階の失敗のみ"),
            };
            match rejection_verdict(stage, code, actual_stage, result) {
                Verdict::Pass => row.outcome = Outcome::Pass,
                Verdict::Fail(differences) => {
                    row.outcome = Outcome::Fail;
                    row.differences = differences;
                }
                Verdict::KnownDivergence(_) => unreachable!("期待拒否は既知乖離で免除しない"),
            }
        }
    }
    normalize_logs(&mut build, directory);
    (build, rows)
}

fn rejection_verdict(stage: &str, code: &str, actual_stage: &str, result: &StageResult) -> Verdict {
    if result.status == StageStatus::Failed
        && stage == actual_stage
        && result.reasons.len() == 1
        && result.reasons[0] == code
    {
        Verdict::Pass
    } else {
        Verdict::Fail(vec![Difference {
            path: "/rejection".into(),
            expected: Some(serde_json::json!({"stage":stage,"code":code})),
            actual: Some(serde_json::json!({"stage":actual_stage,"codes":result.reasons})),
        }])
    }
}
fn normalize_logs(build: &mut BuildResult, directory: &Path) {
    // ランダムな mktemp パスをレポートの差分に持ち込まない。診断内容は保持する。
    for (name, stage) in [
        ("ir", &mut build.ir),
        ("generation", &mut build.generation),
        ("javac", &mut build.javac),
    ]
    .into_iter()
    .chain(build.cargo.as_mut().map(|stage| ("build", stage)))
    {
        stage.output = stage
            .output
            .replace(directory.to_string_lossy().as_ref(), "<build>");
        if stage.status == StageStatus::Failed {
            stage.reasons = reasons(name, &stage.output);
        }
    }
}
fn reasons(stage: &str, output: &str) -> Vec<String> {
    let mut reasons = BTreeSet::new();
    if stage == "ir" {
        for kind in [
            "InvalidSemantics",
            "DirectLeftRecursion",
            "IndirectLeftRecursion",
            "CaptureCardinality",
            "InvalidOperator",
        ] {
            if output.contains(&format!("{kind} {{")) {
                reasons.insert(format!("analysis.{kind}"));
            }
        }
        for word in output.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
            if word.starts_with("E-") {
                reasons.insert(word.to_owned());
            }
        }
    } else if stage == "generation" {
        for line in output.lines() {
            if let Some((_, reason)) = line.split_once("unsupported IR feature ") {
                reasons.insert(
                    reason
                        .split_once(": ")
                        .map_or(reason, |(feature, _)| feature)
                        .trim()
                        .to_owned(),
                );
            }
            for prefix in ["Unsupported IR feature: ", "Unsupported requiredFeatures: "] {
                if let Some((_, reason)) = line.split_once(prefix) {
                    reasons.insert(generation_feature(reason.trim()).to_owned());
                }
            }
        }
    }
    if reasons.is_empty() {
        // 未分類例外も消さず原文の先頭の理由を作業リストに残す。
        let line = output
            .lines()
            .map(str::trim)
            .find(|line| {
                !line.is_empty() && !line.starts_with("終了 ") && !line.starts_with("javac 失敗")
            })
            .unwrap_or("理由不明");
        let line = line
            .split_once("java.lang.IllegalArgumentException: ")
            .map_or(line, |(_, r)| r);
        reasons.insert(
            if stage == "generation" {
                generation_feature(line)
            } else {
                line
            }
            .to_owned(),
        );
    }
    reasons.into_iter().collect()
}

fn generation_feature(reason: &str) -> &str {
    for (prefix, feature) in [
        ("externTokens.inputState:", "externTokens.inputState"),
        ("Recipe conversion mismatch:", "recipe.conversion"),
        ("Duplicate enum name:", "enum.duplicateName"),
        ("indirectLeftRecursion:", "indirectLeftRecursion"),
        ("unloweredLeftRecursion:", "unloweredLeftRecursion"),
    ] {
        if reason.starts_with(prefix) {
            return feature;
        }
    }
    reason
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{rule, token};
    use serde_json::json;

    #[test]
    fn token_ubnf_matches_rust_conformance_token_corpus() {
        let scratch = Scratch::new().unwrap();
        let path = scratch.0.join("corpus.json");
        fs::write(&path, r#"[{"token":"DIGIT","body":"T @value END","whitespace":false,"extra":"Extra ::= 'x';","cases":[{"input":"7","value":"7"}]}]"#).unwrap();
        let file = token::load(&path).unwrap();
        let path = materialize(&file.grammars[0], &scratch.0.join("generated")).unwrap();
        assert_eq!(path.extension().unwrap(), "ubnf");
        // RustConformanceTest.tokenCorpus は whitespace の値でなく存在を見る。
        let expected = "grammar Primitive { @package: org.example.primitive @whitespace: javaStyle token T = DIGIT\n token END = EOF\n Extra ::= 'x';\n @root @mapping(Value, params=[value]) Root ::= T @value END; }";
        let normalize = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(
            normalize(&fs::read_to_string(path).unwrap()),
            normalize(expected)
        );
    }

    #[test]
    fn rule_ubnf_matches_semantic_cardinality_conformance() {
        let scratch = Scratch::new().unwrap();
        let path = scratch.0.join("corpus.json");
        fs::write(&path, r#"[{"name":"helper","kind":"text","cardinality":"one","root":"Helper @values END","body":"'x'","extra":"token END = EOF","cases":[{"input":"x","value":"x"}]}]"#).unwrap();
        // SemanticCardinalityConformance の Grammar.ubnf.txt と同じテンプレート。
        fs::write(scratch.0.join("Grammar.ubnf.txt"), "grammar Semantic {\n  @package: org.example.semantic\n  @whitespace: javaStyle\n  @root @mapping(Box, params=[values]) Document ::= ROOT_BODY;\n  Helper ::= HELPER_BODY;\n  EXTRA_RULES\n  Factor ::= 'a' | '😀' | Leaf;\n  @mapping(Leaf, params=[text]) Leaf ::= ('x' | 'y') @text;\n}\n").unwrap();
        let file = rule::semantic(&path).unwrap();
        let path = materialize(&file.grammars[0], &scratch.0.join("generated")).unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), "grammar Semantic {\n  @package: org.example.semantic\n  @whitespace: javaStyle\n  @root @mapping(Box, params=[values]) Document ::= Helper @values END;\n  Helper ::= 'x';\n  token END = EOF\n  Factor ::= 'a' | '😀' | Leaf;\n  @mapping(Leaf, params=[text]) Leaf ::= ('x' | 'y') @text;\n}\n");
    }

    #[test]
    fn cache_deduplicates_sources_but_keeps_import_contents_and_all_variants() {
        let files = crate::load_all(crate::default_root()).unwrap();
        let plan = plan(&files, &[], Backend::Java).unwrap();
        let executions: usize = files
            .iter()
            .flat_map(|f| &f.cases)
            .map(|case| case.grammar_ref.len().max(1))
            .sum();
        assert_eq!(
            plan.work.iter().map(|w| w.rows.len()).sum::<usize>() + plan.without_grammar.len(),
            executions
        );
        assert!(plan.work.len() < plan.variants);
        let imports: Vec<_> = plan
            .work
            .iter()
            .filter(|w| !w.build.imports.is_empty())
            .collect();
        assert_eq!(imports.len(), 3);
        let alias = imports
            .iter()
            .find(|w| w.build.grammars[0].contains("import-alias.json"))
            .unwrap();
        let nested = imports
            .iter()
            .find(|w| w.build.grammars[0].contains("import-two-level.json"))
            .unwrap();
        assert_eq!(alias.build.source_hash, nested.build.source_hash);
        assert_ne!(alias.build.id, nested.build.id);
        let scratch = Scratch::new().unwrap();
        materialize(&nested.grammar, &scratch.0).unwrap();
        assert_eq!(
            fs::read_to_string(scratch.0.join("C.ubnf")).unwrap(),
            nested.build.imports["C.ubnf"]
        );
        let external = files
            .iter()
            .find(|f| f.path.ends_with("associative/corpus.json"))
            .unwrap();
        assert_eq!(
            materialize(&external.grammars[0], &scratch.0).unwrap(),
            match &external.grammars[0] {
                GrammarSource::File { path, .. } => path.canonicalize().unwrap(),
                _ => panic!(),
            }
        );
    }

    #[test]
    fn import_paths_cannot_escape_scratch() {
        let scratch = Scratch::new().unwrap();
        for name in ["../outside.ubnf", "/outside.ubnf", "__runall__.ubnf"] {
            let grammar = GrammarSource::Inline {
                name: "unsafe".into(),
                source: "grammar G {}".into(),
                metadata: json!({"imports": {name: "grammar B {}"}}),
            };
            assert!(materialize(&grammar, &scratch.0).is_err());
        }
    }

    #[test]
    fn stage_reasons_preserve_features_and_front_codes() {
        assert_eq!(reasons("ir", "ubnfc: E-LOWER-EXTERN: unsupported\nE-LOWER-EXTERN: duplicate\nE-PRECEDENCE-ORDER: invalid"),
            ["E-LOWER-EXTERN", "E-PRECEDENCE-ORDER"]);
        assert_eq!(reasons("generation", "Exception in thread \"main\" java.lang.IllegalArgumentException: Unsupported IR feature: nestedValueType:list\n\tat x"),
            ["nestedValueType:list"]);
        assert_eq!(
            reasons(
                "generation",
                "unsupported IR feature externTokens.inputState: G::T"
            ),
            ["externTokens.inputState"]
        );
        assert_eq!(
            reasons("generation", "Unsupported requiredFeatures: recovery"),
            ["recovery"]
        );
        assert_eq!(
            reasons(
                "generation",
                "Unsupported IR feature: externTokens.inputState:G::T"
            ),
            ["externTokens.inputState"]
        );
        assert_eq!(
            reasons(
                "ir",
                "ubnfc: CaptureCardinality { site_id: \"x\", declared: List, inferred: Scalar }"
            ),
            ["analysis.CaptureCardinality"]
        );
        assert_eq!(
            reasons(
                "generation",
                "java.lang.IllegalArgumentException: Invalid Java identifier: class"
            ),
            ["Invalid Java identifier: class"]
        );
    }

    #[test]
    fn aggregate_all_stage_results_and_keep_missing_separate_from_fail() {
        let files = crate::load_all(crate::default_root()).unwrap();
        let mut plan = plan(&files, &["associative".into()], Backend::Java).unwrap();
        let work = &mut plan.work[0];
        let base = work.rows[0].clone();
        let mut rows = Vec::new();
        for (i, outcome) in [
            Outcome::IrFailure,
            Outcome::GenerationRejected,
            Outcome::JavacFailure,
            Outcome::BuildFailure,
            Outcome::DriverError,
            Outcome::Pass,
            Outcome::KnownDivergence,
            Outcome::Fail,
            Outcome::Unobserved,
        ]
        .into_iter()
        .enumerate()
        {
            let mut row = base.clone();
            row.case_id = i.to_string();
            row.outcome = outcome;
            rows.push(row);
        }
        let mut missing = base.clone();
        missing.case_id = "9".into();
        missing.observe(
            &work.cases[0],
            Actual {
                backend: Backend::Java,
                variant: work.cases[0].variant.clone(),
                response: json!({}),
            },
        );
        assert_eq!(missing.outcome, Outcome::Unobserved);
        assert!(!missing.missing.is_empty());
        rows.push(missing);
        work.build.generation =
            StageResult::failed("generation", "Unsupported IR feature: eval".into());
        let report = Report::new(Backend::Java, 4, 1, 1, vec![work.build.clone()], rows);
        assert_eq!(
            report.totals,
            Counts {
                ir_failure: 1,
                generation_rejected: 1,
                javac_failure: 1,
                build_failure: 1,
                driver_error: 1,
                pass: 1,
                expected_rejection_pass: 0,
                known_divergence: 1,
                fail: 1,
                unobserved: 2
            }
        );
        assert_eq!(report.families["associative"], report.totals);
        assert_eq!(report.refusals["generation/eval"].builds, 1);
        assert!(report.markdown().contains("生成拒否"));
        assert_eq!(
            serde_json::from_str::<Value>(&report.json()).unwrap()["source_cases"],
            10
        );
    }
}
#[test]
fn expected_rejection_requires_the_exact_stage_and_code_and_keeps_failure_evidence() {
    // resolve.rs:143-155 の意図的な拒否。無関係な IR/生成失敗は Pass にしない。
    let files = crate::load_all(crate::default_root()).unwrap();
    let plan = plan(&files, &["oracle".into()], Backend::Java).unwrap();
    let work = plan
        .work
        .iter()
        .find(|w| w.rows.iter().any(|r| r.case_id == "import-cycle/1"))
        .unwrap();
    let directory = Path::new("/unused");
    let mut build = work.build.clone();
    build.ir = StageResult::failed("ir", "[code: E-IMPORT-CYCLE]".into());
    let (kept, rows) = stopped(
        build.clone(),
        work.rows.clone(),
        Outcome::IrFailure,
        directory,
    );
    assert_eq!(rows[0].outcome, Outcome::Pass);
    assert_eq!(kept.ir.status, StageStatus::Failed);
    build.ir = StageResult::failed("ir", "[code: E-IMPORT-UNRESOLVED]".into());
    let (_, rows) = stopped(
        build.clone(),
        work.rows.clone(),
        Outcome::IrFailure,
        directory,
    );
    assert_eq!(rows[0].outcome, Outcome::Fail);
    assert_eq!(rows[0].differences[0].path, "/rejection");
    build.generation = StageResult::failed("generation", "E-IMPORT-CYCLE".into());
    let (_, rows) = stopped(
        build,
        work.rows.clone(),
        Outcome::GenerationRejected,
        directory,
    );
    assert_eq!(rows[0].outcome, Outcome::Fail);
}
