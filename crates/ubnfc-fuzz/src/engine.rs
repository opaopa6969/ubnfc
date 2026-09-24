//! 文法 1 つぶんの「IR → 両バックエンド生成 → build → driver 実行」。
//!
//! 生成・コンパイルは harness の `Artifact::FromIr` / `Runner::build` をそのまま使う
//! （scanner 同梱・Rust の内容 hash キャッシュ・javac 起動の作法を再実装しない）。
//! driver との JSON Lines 交換だけは自前に持つ。campaign では片側が停止し得るので、
//! 制限時間で子プロセスを落として「片側だけ応答しない」を観測として残す必要がある。

use crate::model::Grammar;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use ubnfc_harness::runner::{Artifact, Backend, Executable, Runner};

/// 文法が実行段階に到達しなかった理由。機能別の discard 率に使う。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discard {
    /// `E-...` / `W-...`、または `syntax` / `io` / `build:java` などの段階名。
    pub code: String,
    pub message: String,
}

pub struct Built {
    pub grammar_name: String,
    pub java: Executable,
    pub rust: Executable,
}

pub struct Engine {
    directory: PathBuf,
    java: Runner,
    rust: Runner,
    cache: BTreeMap<String, std::rc::Rc<Built>>,
    pub builds: usize,
    pub timeouts: usize,
    pub timeout: Duration,
}

impl Engine {
    pub fn new(directory: PathBuf) -> Self {
        std::fs::create_dir_all(&directory).expect("fuzz build dir");
        Self {
            java: Runner {
                backend: Backend::Java,
                build_dir: directory.join("java"),
            },
            rust: Runner {
                backend: Backend::Rust,
                build_dir: directory.join("rust"),
            },
            directory,
            cache: BTreeMap::new(),
            builds: 0,
            timeouts: 0,
            timeout: Duration::from_secs(90),
        }
    }

    /// 文法を front に通して IR を書き出す。build は行わない（生成器の triage 用）。
    pub fn compile_front(&mut self, grammar: &Grammar) -> Result<PathBuf, Discard> {
        self.write_and_compile(grammar).map(|(ir, _)| ir)
    }

    fn write_and_compile(&mut self, grammar: &Grammar) -> Result<(PathBuf, String), Discard> {
        let digest = grammar.digest();
        let directory = self.directory.join(format!("g{digest}"));
        std::fs::create_dir_all(&directory).map_err(|e| Discard {
            code: "io".into(),
            message: e.to_string(),
        })?;
        let main = directory.join("grammar.ubnf");
        std::fs::write(&main, grammar.render()).map_err(|e| Discard {
            code: "io".into(),
            message: e.to_string(),
        })?;
        if let Some((file, source)) = grammar.imported_source() {
            std::fs::write(directory.join(file), source).map_err(|e| Discard {
                code: "io".into(),
                message: e.to_string(),
            })?;
        }
        let compiled = ubnfc_front::pipeline::compile(&main, &|path| std::fs::read_to_string(path))
            .map_err(|error| discard_of(&error))?;
        let ir_path = directory.join("grammar.ir.json");
        std::fs::write(&ir_path, ubnfc_front::ir::to_canonical_json(&compiled.ir)).map_err(
            |e| Discard {
                code: "io".into(),
                message: e.to_string(),
            },
        )?;
        let grammar_name = compiled
            .ir
            .grammar_units
            .iter()
            .find(|unit| unit.id == compiled.ir.entry.grammar_id)
            .map(|unit| unit.name.clone())
            .ok_or_else(|| Discard {
                code: "ir-entry".into(),
                message: "IR に entry grammar がありません".into(),
            })?;
        Ok((ir_path, grammar_name))
    }

    /// 文法を IR にし、両バックエンドの driver を build する。内容 hash で使い回す。
    pub fn prepare(&mut self, grammar: &Grammar) -> Result<std::rc::Rc<Built>, Discard> {
        let digest = grammar.digest();
        if let Some(built) = self.cache.get(&digest) {
            return Ok(built.clone());
        }
        let (ir_path, grammar_name) = self.write_and_compile(grammar)?;
        self.builds += 1;
        let java = self
            .java
            .build(Artifact::FromIr {
                ir_path: ir_path.clone(),
                backend: Backend::Java,
            })
            .map_err(|error| Discard {
                code: "build:java".into(),
                message: error.to_string(),
            })?;
        let rust = self
            .rust
            .build(Artifact::FromIr {
                ir_path,
                backend: Backend::Rust,
            })
            .map_err(|error| Discard {
                code: "build:rust".into(),
                message: error.to_string(),
            })?;
        let built = std::rc::Rc::new(Built {
            grammar_name,
            java,
            rust,
        });
        self.cache.insert(digest, built.clone());
        Ok(built)
    }

    /// D-070 の in-backend A/B。Rust backend だけを `options.scan` 付きで走らせる。
    /// driver はこのキーがあるとき fast mode（診断・出現表なし）で解析するので、
    /// 一括走査が効く条件そのもので on / off を比べられる。
    pub fn run_scan(&mut self, built: &Built, inputs: &[String], scan: bool) -> Vec<Value> {
        self.run_with(built, Backend::Rust, inputs, Some(scan))
    }

    /// 全入力を 1 プロセスに流す。応答は入力順に返り、欠落は error 応答になる。
    pub fn run(&mut self, built: &Built, backend: Backend, inputs: &[String]) -> Vec<Value> {
        self.run_with(built, backend, inputs, None)
    }

    fn run_with(
        &mut self,
        built: &Built,
        backend: Backend,
        inputs: &[String],
        scan: Option<bool>,
    ) -> Vec<Value> {
        let executable = match backend {
            Backend::Java => &built.java,
            Backend::Rust => &built.rust,
        };
        let requests: Vec<Value> = inputs
            .iter()
            .enumerate()
            .map(|(id, input)| request(id, &built.grammar_name, input, scan))
            .collect();
        match exchange(executable, &requests, self.timeout) {
            Ok(mut rows) => (0..inputs.len())
                .map(|id| {
                    rows.remove(&id)
                        .unwrap_or_else(|| json!({"error": "応答行がありません"}))
                })
                .collect(),
            Err(error) => {
                if error.starts_with("timeout") {
                    self.timeouts += 1;
                }
                inputs
                    .iter()
                    .map(|_| json!({ "error": error.clone() }))
                    .collect()
            }
        }
    }
}

/// `docs/harness-driver-protocol.md` のリクエスト 1 行。
fn request(id: usize, grammar: &str, input: &str, scan: Option<bool>) -> Value {
    let mut options = json!({"lexical": true, "scope": true});
    if let Some(scan) = scan {
        options["scan"] = json!(scan);
    }
    json!({"id": id, "grammar": grammar, "entry": Value::Null, "input": input,
        "options": options,
        "observations": {"family": "fuzz", "variant": "fuzz"}})
}

fn discard_of(error: &ubnfc_front::pipeline::PipelineError) -> Discard {
    use ubnfc_front::pipeline::PipelineError as E;
    let message = error.to_string();
    let stage = match error {
        E::Io { .. } => "io",
        E::Syntax { .. } => "syntax",
        E::Resolve(_) => "resolve",
        E::Validation(_) | E::Strict(_) => "validate",
        E::Lower(_) => "lower",
        E::Analysis(_) => "analysis",
        E::Internal(_) => "internal",
    };
    Discard {
        code: code_in(&message).unwrap_or_else(|| stage.to_owned()),
        message,
    }
}

/// メッセージ中の最初の `E-...` / `W-...` コードを取り出す。
fn code_in(message: &str) -> Option<String> {
    let bytes: Vec<char> = message.chars().collect();
    for start in 0..bytes.len().saturating_sub(2) {
        if !matches!(bytes[start], 'E' | 'W') || bytes[start + 1] != '-' {
            continue;
        }
        if start > 0 && (bytes[start - 1].is_alphanumeric() || bytes[start - 1] == '-') {
            continue;
        }
        let mut end = start + 2;
        while end < bytes.len() && (bytes[end].is_ascii_uppercase() || bytes[end] == '-') {
            end += 1;
        }
        if end > start + 2 {
            return Some(
                bytes[start..end]
                    .iter()
                    .collect::<String>()
                    .trim_end_matches('-')
                    .to_owned(),
            );
        }
    }
    None
}

fn drain(mut source: impl Read + Send + 'static) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut text = String::new();
        let mut raw = Vec::new();
        let _ = source.read_to_end(&mut raw);
        text.push_str(&String::from_utf8_lossy(&raw));
        text
    })
}

fn wait_or_kill(child: &mut Child, timeout: Duration) -> Result<bool, String> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Err(error) => return Err(error.to_string()),
            Ok(Some(status)) => return Ok(status.success()),
            Ok(None) => {}
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("timeout: {}s を超えました", timeout.as_secs()));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// 制限時間つきの JSON Lines 交換。停止した driver は kill して error 観測にする。
fn exchange(
    executable: &Executable,
    requests: &[Value],
    timeout: Duration,
) -> Result<BTreeMap<usize, Value>, String> {
    let mut child = Command::new(&executable.program)
        .args(&executable.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let lines: String = requests
        .iter()
        .map(|request| format!("{request}\n"))
        .collect();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(lines.as_bytes());
        let _ = stdin.flush();
    });
    let stdout = drain(child.stdout.take().expect("piped stdout"));
    let stderr = drain(child.stderr.take().expect("piped stderr"));
    let status = wait_or_kill(&mut child, timeout);
    let _ = writer.join();
    let out = stdout.join().unwrap_or_default();
    let err = stderr.join().unwrap_or_default();
    match status {
        Err(error) => return Err(error),
        Ok(false) => {
            return Err(format!(
                "driver 異常終了: {}",
                err.lines()
                    .next_back()
                    .unwrap_or("")
                    .chars()
                    .take(200)
                    .collect::<String>()
            ))
        }
        Ok(true) => {}
    }
    let mut rows = BTreeMap::new();
    for (index, line) in out.lines().enumerate() {
        let row: Value = serde_json::from_str(line)
            .map_err(|error| format!("driver 行 {}: {error}", index + 1))?;
        let id = row
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("driver 行 {}: id が不正です", index + 1))?;
        rows.insert(id as usize, row);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::code_in;

    #[test]
    fn error_codes_are_extracted_from_pipeline_messages() {
        assert_eq!(
            code_in("rule R has no mapping [code: E-ASSOC-NO-MAPPING] [hint: x]").as_deref(),
            Some("E-ASSOC-NO-MAPPING")
        );
        assert_eq!(
            code_in("E-LOWER-INDIRECT-LEFT-RECURSION: A -> B").as_deref(),
            Some("E-LOWER-INDIRECT-LEFT-RECURSION")
        );
        assert_eq!(code_in("no code here").as_deref(), None);
        assert_eq!(
            code_in("XE-NOPE and W-LEFT-RECURSION").as_deref(),
            Some("W-LEFT-RECURSION")
        );
    }
}
