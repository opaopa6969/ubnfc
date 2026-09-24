//! `ubnfc` CLI。文法の診断は exit 1、strict 警告は 5、引数・入出力エラーは 2。

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;

use crate::ir;
use crate::pipeline::{self, PipelineError};
use crate::validate::ValidationIssue;

pub fn run(args: &[String]) -> Result<i32, String> {
    if args.is_empty() || matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        if args.len() > 1 {
            return Err("--help に追加の引数は指定できません".into());
        }
        stdout(USAGE)?;
        return Ok(0);
    }
    if matches!(args[0].as_str(), "--version" | "-v") {
        if args.len() != 1 {
            return Err("--version に追加の引数は指定できません".into());
        }
        stdout(&format!("ubnfc {}\n", env!("CARGO_PKG_VERSION")))?;
        return Ok(0);
    }
    let command = args[0].as_str();
    if !matches!(command, "ir" | "validate" | "gen") {
        return Err(format!("不明なサブコマンド: {command}\n{USAGE}"));
    }
    if args.len() == 2 && matches!(args[1].as_str(), "--help" | "-h") {
        stdout(USAGE)?;
        return Ok(0);
    }
    let options = Options::parse(command, &args[1..])?;
    let loader = |p: &Path| std::fs::read_to_string(p);
    if command == "validate" {
        let mut issues = match pipeline::validate_file(&options.grammar, &loader) {
            Ok(issues) => issues,
            Err(error) => diagnostic_issues(error, &options.grammar)?,
        };
        pipeline::sort_issues(&mut issues);
        stdout(&format_issues(&issues, options.format == "json"))?;
        return Ok(issue_exit_code(&issues, options.strict));
    }
    let mut compiled =
        match pipeline::compile_with_strict(&options.grammar, &loader, options.strict) {
            Ok(compiled) => compiled,
            Err(error) => {
                let code = match error {
                    PipelineError::Strict(_) => 5,
                    PipelineError::Io { .. } => 2,
                    _ => 1,
                };
                eprintln!("ubnfc: {error}");
                return Ok(code);
            }
        };
    for warning in &compiled.warnings {
        eprintln!("{}", warning.format());
    }
    if let Some(path) = &options.extern_first_chars {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let declarations =
            ir::parse_extern_first_chars(&text).map_err(|e| format!("{path}: {e}"))?;
        let (_, missing) = ir::apply_extern_first_chars(&mut compiled.ir, &declarations);
        // v2 は host 実装を明示する形式（D-041）なので、registry が提供しない extern を
        // **生成時に**落とす。v1 は凍結なので従来どおり stderr の注意だけ（D-042）。
        if compiled.ir.language_version >= 2 && !missing.is_empty() {
            for class in &missing {
                eprintln!(
                    "ubnfc: E-TOKEN-EXTERN-UNREGISTERED: extern token implementation \
                     {class} is not provided by {path} \
                     [hint: add {class} to the scanner registry (scanners/first-chars.json and \
                     scanners/{{java,rust}}), or fix the name in the extern declaration]"
                );
            }
            return Ok(1);
        }
        for class in missing {
            eprintln!(
                "ubnfc: {path}: Extern class {class} has no firstChars declaration (kept unknown)"
            );
        }
    }
    let output = ir::to_canonical_json(&compiled.ir);
    if command == "gen" {
        let dir = Path::new(options.out.as_deref().expect("gen --out"));
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join("grammar.ir.json");
        std::fs::write(&path, output).map_err(|e| format!("{}: {e}", path.display()))?;
        return generate_backend(options.target.as_deref().expect("gen --target"), &path, dir);
    }
    match options.out.as_deref().unwrap_or("-") {
        "-" => stdout(&output)?,
        path => std::fs::write(path, output).map_err(|e| format!("{path}: {e}"))?,
    }
    Ok(0)
}

fn generate_backend(target: &str, ir: &Path, out: &Path) -> Result<i32, String> {
    let variable = if target == "rust" {
        "UBNFC_RUST_BIN"
    } else {
        "UBNFC_JAVA_JAR"
    };
    let mut searched = Vec::new();
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os(variable) {
        // 明示指定を優先し、typo を他の生成器への fallback で隠さない。
        candidates.push(PathBuf::from(path));
    } else {
        let mut roots = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")];
        if let Ok(cwd) = std::env::current_dir() {
            roots.extend(cwd.ancestors().map(Path::to_owned));
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                if target == "rust" {
                    candidates
                        .push(dir.join(format!("ubnfc-rust{}", std::env::consts::EXE_SUFFIX)));
                }
                roots.extend(dir.ancestors().map(Path::to_owned));
            }
        }
        if target == "rust" {
            let mut targets = roots.iter().map(|p| p.join("target")).collect::<Vec<_>>();
            if let Some(dir) = std::env::var_os("CARGO_TARGET_DIR") {
                targets.insert(0, dir.into());
            }
            for dir in targets {
                for profile in ["debug", "release"] {
                    candidates.push(
                        dir.join(profile)
                            .join(format!("ubnfc-rust{}", std::env::consts::EXE_SUFFIX)),
                    );
                }
            }
        } else {
            for root in roots {
                let dir = root.join("ubnfc-java/target");
                searched.push(dir.join("ubnfc-java-*.jar"));
                if let Ok(entries) = std::fs::read_dir(dir) {
                    let mut jars = entries
                        .filter_map(Result::ok)
                        .map(|e| e.path())
                        .filter(|p| {
                            p.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                                n.starts_with("ubnfc-java-")
                                    && n.ends_with(".jar")
                                    && !n.ends_with("-sources.jar")
                                    && !n.ends_with("-javadoc.jar")
                            })
                        })
                        .collect::<Vec<_>>();
                    jars.sort();
                    candidates.extend(jars);
                }
            }
        }
    }
    searched.extend(candidates.iter().cloned());
    let Some(generator) = candidates.iter().find(|p| p.is_file()) else {
        eprintln!("ubnfc: {target} backend が見つかりません（{variable} で指定可能、IR: {}）。探索パス:\n{}", ir.display(), searched.iter().map(|p| format!("  {}",p.display())).collect::<Vec<_>>().join("\n"));
        return Ok(3);
    };
    let generator = generator
        .canonicalize()
        .map_err(|e| format!("{}: {e}", generator.display()))?;
    let mut command = if target == "rust" {
        Command::new(&generator)
    } else {
        let mut command = Command::new("java");
        command
            .arg("-cp")
            .arg(&generator)
            .arg("org.unlaxer.ubnfc.java.Main");
        command
    };
    command.arg("--ir").arg(ir).arg("--out");
    if target == "rust" {
        command.arg(out);
    } else {
        command
            .arg(out.join("src/main/java"))
            .args(["--package", "target"]);
    }
    let status = command
        .arg("--emit-driver")
        .status()
        .map_err(|e| format!("{target} backend {}: {e}", generator.display()))?;
    Ok(status.code().unwrap_or(1))
}

struct Options {
    grammar: String,
    out: Option<String>,
    target: Option<String>,
    strict: bool,
    format: String,
    /// Extern token の先頭文字宣言ファイル（`scanners/first-chars.json`）。ir / gen だけ。
    extern_first_chars: Option<String>,
}

impl Options {
    fn parse(command: &str, args: &[String]) -> Result<Self, String> {
        let mut values = BTreeMap::new();
        let mut strict = false;
        let mut args = args.iter();
        while let Some(flag) = args.next() {
            if flag == "--strict" && command != "gen" {
                if strict {
                    return Err("--strict が重複しています".into());
                }
                strict = true;
                continue;
            }
            let allowed = flag == "--grammar"
                || (flag == "--out" && command != "validate")
                || (flag == "--extern-first-chars" && command != "validate")
                || (flag == "--target" && command == "gen")
                || (flag == "--format" && command == "validate");
            if !allowed {
                return Err(format!("{command}: 不明なオプション: {flag}"));
            }
            let value = args
                .next()
                .ok_or_else(|| format!("{flag} の値が必要です"))?;
            if value.is_empty() || (value.starts_with('-') && !(flag == "--out" && value == "-")) {
                return Err(format!("{flag} の値が不正です: {value}"));
            }
            if values.insert(flag.as_str(), value.clone()).is_some() {
                return Err(format!("{flag} が重複しています"));
            }
        }
        let grammar = values.remove("--grammar").ok_or("--grammar が必要です")?;
        let out = values.remove("--out");
        let target = values.remove("--target");
        let format = values.remove("--format").unwrap_or_else(|| "text".into());
        let extern_first_chars = values.remove("--extern-first-chars");
        if !matches!(format.as_str(), "text" | "json") {
            return Err(format!("--format は text または json: {format}"));
        }
        if command == "gen" {
            if !matches!(target.as_deref(), Some("java" | "rust")) {
                return Err("--target java|rust が必要です".into());
            }
            if out.as_deref().is_none_or(|p| p == "-") {
                return Err("gen には --out DIR が必要です".into());
            }
        }
        Ok(Self {
            grammar,
            out,
            target,
            strict,
            format,
            extern_first_chars,
        })
    }
}

fn stdout(output: &str) -> Result<(), String> {
    io::stdout()
        .lock()
        .write_all(output.as_bytes())
        .map_err(|e| format!("標準出力: {e}"))
}

fn issue_exit_code(issues: &[ValidationIssue], strict: bool) -> i32 {
    if issues.iter().any(|i| i.severity() == "ERROR") {
        1
    } else if strict && !issues.is_empty() {
        5
    } else {
        0
    }
}

fn format_issues(issues: &[ValidationIssue], as_json: bool) -> String {
    if !as_json {
        return issues.iter().map(|i| format!("{}\n", i.format())).collect();
    }
    let mut severity_counts = BTreeMap::from([("ERROR", 0usize), ("WARNING", 0)]);
    let mut category_counts = BTreeMap::new();
    let issues: Vec<_> = issues
        .iter()
        .map(|i| {
            *severity_counts.entry(i.severity()).or_default() += 1;
            *category_counts.entry(i.category()).or_insert(0usize) += 1;
            json!({ "code": i.code, "message": i.message, "hint": i.hint,
            "grammar": i.grammar, "rule": i.rule,
            "severity": i.severity(), "category": i.category() })
        })
        .collect();
    let report = json!({ "issues": issues, "severityCounts": severity_counts,
        "categoryCounts": category_counts });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&report).expect("diagnostic serialization")
    )
}

fn diagnostic_issues(
    error: PipelineError,
    main_path: &str,
) -> Result<Vec<ValidationIssue>, String> {
    // resolver の import 診断にも同じ format()/JSON 契約を適用する。
    // 絶対配置を診断 golden に埋め込まない。
    let path = pipeline::absolute_path(Path::new(main_path)).map_err(|e| e.to_string())?;
    let prefix = format!("{}/", path.parent().unwrap_or(Path::new("")).display());
    match error {
        PipelineError::Resolve(errors) => Ok(errors
            .into_iter()
            .map(|e| ValidationIssue {
                code: e.code,
                message: e.message.replace(&prefix, ""),
                hint: e.hint.replace(&prefix, ""),
                grammar: e.grammar,
                rule: e.rule,
            })
            .collect()),
        PipelineError::Syntax { error, .. } => Ok(vec![ValidationIssue {
            code: "E-SYNTAX".into(),
            message: error.to_string(),
            hint: "UBNF の構文を修正してください。".into(),
            grammar: String::new(),
            rule: None,
        }]),
        // ファイル入出力失敗は CLI エラー。JSON 診断と混在させない。
        other => Err(other.to_string()),
    }
}

const USAGE: &str = "usage: ubnfc <ir|validate|gen> [options]\n\n\
  ubnfc ir --grammar X.ubnf [--out FILE|-] [--strict] [--extern-first-chars FILE]\n\
  ubnfc validate --grammar X.ubnf [--strict] [--format text|json]\n\
  ubnfc gen --grammar X.ubnf --target java|rust --out DIR [--extern-first-chars FILE]\n\
  ubnfc --version | --help\n\n\
ir の既定出力は標準出力。--strict は WARNING を exit 5 にします。\n\
validate は syntax / resolve / validate の診断を出力します。\n\
gen は DIR/grammar.ir.json を書き、backend で driver 付きソースを生成します。\n\
--extern-first-chars は scanner registry の先頭文字宣言（scanners/first-chars.json）を\n\
Extern token の firstChars に写します（宣言の無いクラスは unknown のまま）。\n\
exit: 0 成功、1 文法エラー、2 CLI/入出力エラー、3 backend が見つからない、5 strict 警告。\n";
