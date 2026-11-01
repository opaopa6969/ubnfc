//! Classic の綴りの引数付き規則（`F(x) ::=` / `F(a)`、unlaxer-parser #701、ubnfc issue #11）。
//!
//! Classic の `spec-corpus/parameterized-rules/cases.json`（`fixtures/classic/` に写し）を oracle
//! にして、ubnfc の front で展開し Rust backend で生成した parser が、Classic と同じ入力を受理・
//! 拒否し、同じ AST を返すことを確かめる。拒否の例は、ubnfc も拒否することと、その code を
//! 一覧で固定する（ubnfc は instance 名を作らないので、一部は別の code になる）。

use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let out = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(out.status.success());
        Self(PathBuf::from(String::from_utf8(out.stdout).unwrap().trim()))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../../fixtures/classic/parameterized-rules/cases.json"
    ))
    .unwrap()
}

fn compile(
    grammar: &str,
) -> Result<ubnfc_front::pipeline::Compiled, ubnfc_front::pipeline::PipelineError> {
    let text = grammar.to_owned();
    ubnfc_front::pipeline::compile("/virtual/classic.ubnf", &move |_| Ok(text.clone()))
}

fn cargo(dir: &Path, args: &[&str]) {
    let out = Command::new(env!("CARGO"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn exchange(binary: &Path, lines: &str) -> Vec<Value> {
    let mut child = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let output = std::thread::scope(|scope| {
        scope.spawn(move || input.write_all(lines.as_bytes()).unwrap());
        child.wait_with_output().unwrap()
    });
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// 拒否の例ごとに ubnfc が返す code。Classic と違うものは理由を添える。
fn expected_rejection(name: &str) -> &'static str {
    match name {
        // 展開の後の既存検査（左再帰）で拒否する。Classic と同じ順序。
        // ubnfc は左再帰を脱糖できる形（D-039）以外を lower で拒否する（D-016）。
        "ce002-self-argument" | "nullable-prefix-after-expansion" => "E-LOWER-D016",
        "arity-too-many" | "arity-bare-reference" | "arity-trivia-before-parenthesis" => {
            "E-PARAM-ARITY"
        }
        // 定義本体の未定義の名前は、展開の後の resolve が未定義参照として拒否する。
        "undefined-parameter" => "E-RULE-UNDEFINED",
        "cycle-self" | "cycle-mutual-crlf" | "cycle-identity" => "E-PARAM-CYCLE",
        "annotation-mapping" | "annotation-root" => "E-PARAM-ANNOTATION",
        "capture-in-argument" | "capture-in-body" => "E-PARAM-CAPTURE",
        "parameter-shadows-rule" => "E-PARAM-SHADOW",
        "duplicate-parameter" | "duplicate-rule" => "E-PARAM-DUPLICATE",
        // 構文エラーとして位置付きで拒否する（本文の先頭が code）。
        "reserved-name" => "E-PARAM-RESERVED",
        // 要素数 65536 と、展開した本体の入れ子 128（構文解析と同じ数え方）の上限。
        "limit-elements" | "limit-chain-times-nesting" | "limit-nesting" => "E-PARAM-LIMIT",
        other => panic!("一覧に無い拒否の例: {other}"),
    }
}

/// ubnfc では拒否にならない Classic の拒否の例と、その理由。
fn not_rejected(name: &str) -> Option<&'static str> {
    match name {
        // ubnfc は instance に名前を付けない（その場で展開する、D-043）ので、
        // 展開名と既存の規則名の衝突は起こらない。
        "name-collision" => Some("instance 名を作らない"),
        // Classic の入れ子は instance の本体と呼び出しの連鎖も数えるが、ubnfc は instance を
        // 作らずその場で展開するので、展開後の木の入れ子は上限内に収まる（IR 生成と
        // backend の生成も通る）。使われない実引数も ubnfc は捨てた後の本体しか数えない。
        "limit-unused-argument-growth"
        | "limit-chain-flat"
        | "limit-chain-nested"
        | "limit-argument-nesting" => Some("ubnfc の数え方では上限内"),
        _ => None,
    }
}

fn rejection_codes(grammar: &str) -> Vec<String> {
    match compile(grammar) {
        Ok(_) => Vec::new(),
        Err(error) => {
            use ubnfc_front::pipeline::PipelineError as E;
            match error {
                E::Syntax { error, .. } => vec![error.message.clone()],
                E::Resolve(errors) => errors.into_iter().map(|e| e.code).collect(),
                E::Validation(issues) | E::Strict(issues) => issues
                    .into_iter()
                    .filter(|i| i.severity() == "ERROR")
                    .map(|i| i.code)
                    .collect(),
                other => vec![other.to_string()],
            }
        }
    }
}

#[test]
fn rejections_are_rejected_with_the_listed_codes() {
    let mut checked = 0;
    for case in cases().iter().filter(|c| c["status"] == "reject") {
        let name = case["name"].as_str().unwrap();
        let grammar = case["grammar"].as_str().unwrap();
        let codes = rejection_codes(grammar);
        if let Some(reason) = not_rejected(name) {
            // 一覧に載せた理由が今も当てはまる（ubnfc が受理するか、別の理由で拒否する）ことだけ記録する。
            eprintln!("{name}: Classic だけが拒否（{reason}）: ubnfc = {codes:?}");
            continue;
        }
        let want = expected_rejection(name);
        assert!(
            codes.iter().any(|c| c.starts_with(want)),
            "{name}: {want} を期待したが {codes:?}\n{grammar}"
        );
        checked += 1;
    }
    assert_eq!(checked, 20);
}

/// 受理だけを比べ、AST は比べない例（共通の部分集合の外）とその理由。
fn outside_common_subset(name: &str) -> Option<&'static str> {
    match name {
        // `@nestedCaptures`（unlaxer-parser #699）は引数付き規則とは別の Classic の機能で、
        // ubnfc には無い（W-ANNOTATION-UNKNOWN）。capture の値が list ではなく text になる。
        "nested-captures-through-instances" => Some("@nestedCaptures は ubnfc に無い"),
        _ => None,
    }
}

/// Classic の AST（`{type, span, fields}`、text は文字列、list は配列）と比べられる形へ、
/// generated driver の `ast` を写す。
fn project(value: &Value) -> Value {
    value.clone()
}

#[test]
fn accepted_grammars_accept_the_same_inputs_with_the_same_ast() {
    let accepted: Vec<_> = cases()
        .into_iter()
        .filter(|c| c["status"] == "accept")
        .collect();
    assert_eq!(accepted.len(), 10);
    let mut rows = 0;
    let mut mismatches = BTreeMap::new();
    for case in &accepted {
        let name = case["name"].as_str().unwrap();
        let compiled =
            compile(case["grammar"].as_str().unwrap()).unwrap_or_else(|e| panic!("{name}: {e}"));
        let temp = Temp::new();
        ubnfc_rust::emit(&compiled.ir, &Default::default())
            .unwrap()
            .write_to(&temp.0)
            .unwrap();
        cargo(&temp.0, &["build", "--offline", "--quiet"]);
        // driver は grammar の名前で入口を選ぶ。入口はファイルの最後の grammar。
        let text = case["grammar"].as_str().unwrap();
        let grammar = text
            .rsplit("grammar ")
            .next()
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap()
            .to_owned();
        let mut lines = String::new();
        let inputs = case["inputs"].as_array().unwrap();
        for (i, input) in inputs.iter().enumerate() {
            lines.push_str(&format!(
                "{}\n",
                json!({"id": i, "grammar": grammar, "entry": null, "input": input["input"],
                       "options": {"lexical": false, "scope": false}})
            ));
        }
        let results = exchange(&temp.0.join("target/debug/driver"), &lines);
        for (input, row) in inputs.iter().zip(&results) {
            rows += 1;
            let expected = &input["expected"];
            let accepted = row["accepted"] == true && row["allConsumed"] == true;
            let ok = expected["status"] == "ok";
            let compare_ast = ok && outside_common_subset(name).is_none();
            if accepted != ok || (compare_ast && project(&row["ast"]) != expected["value"]) {
                mismatches.insert(
                    format!("{name}: {}", input["input"]),
                    format!("expected {expected}\n  ubnfc {row}"),
                );
            }
        }
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
    assert_eq!(rows, 26);
}
