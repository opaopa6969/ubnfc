//! `ubnfc` CLI。文法の診断は exit 1、strict 警告は 5、引数・入出力エラーは 2。

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;

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
    let compiled = match pipeline::compile_with_strict(&options.grammar, &loader, options.strict) {
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
    let output = ir::to_canonical_json(&compiled.ir);
    if command == "gen" {
        let dir = Path::new(options.out.as_deref().expect("gen --out"));
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join("grammar.ir.json");
        std::fs::write(&path, output).map_err(|e| format!("{}: {e}", path.display()))?;
        eprintln!(
            "ubnfc: {} backend 未接続（IR: {}）",
            options.target.as_deref().expect("gen --target"),
            path.display()
        );
        return Ok(3);
    }
    match options.out.as_deref().unwrap_or("-") {
        "-" => stdout(&output)?,
        path => std::fs::write(path, output).map_err(|e| format!("{path}: {e}"))?,
    }
    Ok(0)
}

struct Options {
    grammar: String,
    out: Option<String>,
    target: Option<String>,
    strict: bool,
    format: String,
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
  ubnfc ir --grammar X.ubnf [--out FILE|-] [--strict]\n\
  ubnfc validate --grammar X.ubnf [--strict] [--format text|json]\n\
  ubnfc gen --grammar X.ubnf --target java|rust --out DIR\n\
  ubnfc --version | --help\n\n\
ir の既定出力は標準出力。--strict は WARNING を exit 5 にします。\n\
validate は syntax / resolve / validate の診断を出力します。\n\
gen は DIR/grammar.ir.json を書き、backend 未接続として exit 3 を返します。\n\
exit: 0 成功、1 文法エラー、2 CLI/入出力エラー、3 backend 未接続、5 strict 警告。\n";
