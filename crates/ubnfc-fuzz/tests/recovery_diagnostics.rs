//! `@recovery` を通った解析の**観測**（回復診断の候補と `captureOccurrences`）を
//! 両バックエンドで突き合わせる、的を絞った差分テスト。
//!
//! issue #41（v2 seed 100276）の回帰試験。乱択 campaign は `@recovery` を生成しているが、
//! 固定 seed の帯は「回復が実際に起きたうえで回復診断が観測に出る入力」に当たっていなかった。
//! ここでは文法と入力を固定する。
//!
//! 正解はどちらも Java 側（D-025 の wire 正本、D-027 の最遠候補の蓄積）。
//! - 回復診断の `expected` / `farthestExpected` は**公開（表示）語彙**で、範囲は
//!   回復した規則の frame（Java `Session.recover` の `enterExpression` / `leaveRule`）。
//!   Rust は主診断 DAG の内部 label をそのまま公開していた（`'a'` ではなく `a`）。
//! - 規則が自分の開始位置で失敗したときは、同期文字列も候補に載る
//!   （Java `Session.rule` の recoveryHints。旧実装の観測は `corpus/oracle/recovery-auto.json`）。
//! - `captureOccurrences` は「回復領域を**自分の値としてそのまま持つ**出現」だけ。
//!   間に AST ノードを作る規則が入ると、その外側の capture は数えない。
//!
//! 前提ツールは `campaign.rs` と同じ（`javac` / `java` / mvn 済みの ubnfc-java）。

use std::path::PathBuf;
use std::process::Command;
use ubnfc_fuzz::engine::Engine;
use ubnfc_fuzz::model::{Atom, Body, Element, Grammar, Rule};
use ubnfc_harness::runner::Backend;

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn scratch() -> Scratch {
    let output = Command::new("mktemp").arg("-d").output().unwrap();
    assert!(output.status.success());
    Scratch(PathBuf::from(
        String::from_utf8(output.stdout).unwrap().trim(),
    ))
}

fn rule(annotations: &[&str], name: &str, body: Body) -> Rule {
    Rule::plain(
        annotations.iter().map(|a| (*a).to_owned()).collect(),
        name.to_owned(),
        body,
    )
}

fn grammar(name: &str, rules: Vec<Rule>) -> Grammar {
    Grammar {
        name: name.to_owned(),
        settings: vec!["@package: fuzz.generated".to_owned()],
        templates: Vec::new(),
        tokens: Vec::new(),
        rules,
        import: None,
        features: Default::default(),
    }
}

/// 比較するのは `recoveries` と **回復診断**だけ。回復領域を値に持つ field の AST 投影は
/// 別の未決事項（`docs/decisions/D-018-undecided-divergences.md` の `recovery-projection`、
/// 差分 fuzzing 報告の A-1 系）なので、ここでは巻き込まない。
fn recovery_view(response: &serde_json::Value) -> serde_json::Value {
    let diagnostics: Vec<_> = response
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .map(|rows| {
            rows.iter()
                .filter(|row| row.get("kind").and_then(|k| k.as_str()) == Some("recovery"))
                .map(|row| {
                    let mut out = serde_json::Map::new();
                    for key in [
                        "kind",
                        "offset",
                        "length",
                        "severity",
                        "origin",
                        "ruleId",
                        "expected",
                        "farthestOffset",
                        "farthestExpected",
                    ] {
                        out.insert(key.into(), row.get(key).cloned().unwrap_or_default());
                    }
                    serde_json::Value::Object(out)
                })
                .collect()
        })
        .unwrap_or_default();
    serde_json::json!({
        "recoveries": response.get("recoveries").cloned().unwrap_or_default(),
        "recoveryDiagnostics": diagnostics,
    })
}

fn assert_backends_agree(engine: &mut Engine, grammar: &Grammar, inputs: &[&str]) {
    let built = engine
        .prepare(grammar)
        .unwrap_or_else(|d| panic!("{}: {} {}", grammar.name, d.code, d.message));
    let inputs: Vec<String> = inputs.iter().map(|s| (*s).to_owned()).collect();
    let java = engine.run(&built, Backend::Java, &inputs);
    let rust = engine.run(&built, Backend::Rust, &inputs);
    let mut recovered = 0usize;
    for (index, input) in inputs.iter().enumerate() {
        let expected = recovery_view(&java[index]);
        let actual = recovery_view(&rust[index]);
        assert_eq!(
            expected, actual,
            "{} {input:?}\njava={}\nrust={}",
            grammar.name, java[index], rust[index]
        );
        if expected["recoveries"]
            .as_array()
            .is_some_and(|r| !r.is_empty())
        {
            recovered += 1;
        }
    }
    // 回復が 1 件も起きていなければ、この文法・入力は回帰試験になっていない。
    assert!(recovered > 0, "{}: 回復が 1 件も起きていない", grammar.name);
}

#[test]
fn recovery_diagnostics_and_capture_occurrences_agree_between_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let mut engine = Engine::new(scratch.0.clone());

    // 1. 回復した規則の後続に別のリテラルがある形。Java の回復診断は
    //    「規則の frame の最遠候補」なので `'a'` だけを載せる（`'zz'` は回復が
    //    位置を進めた**後**の失敗なので入らない）。Rust は内部 label（`a`）を
    //    公開していた。`@recovery(SKIP)` の同期文字列は FOLLOW から `zz` になる。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "RecoveryHints",
            vec![
                rule(
                    &["@recovery(SKIP)"],
                    "M0",
                    Body::single(vec![Element::new(Atom::Literal("a".into()))]),
                ),
                rule(
                    &["@root", "@mapping(Top, params=[head])"],
                    "Root",
                    Body::single(vec![
                        Element::captured(Atom::Ref("M0".into()), "head"),
                        Element::new(Atom::Optional(Body::single(vec![Element::new(
                            Atom::Literal("zz".into()),
                        )]))),
                    ]),
                ),
            ],
        ),
        &["+", "a", "azz", "+zz", ""],
    );

    // 2. 回復した規則を包む capture が 2 段あり、**間に AST ノードを作る規則**が入る。
    //    `captureOccurrences` は内側の 1 件だけ（Java は回復 marker を直接値に持つ
    //    出現だけを数える）。Rust は祖先の capture をすべて数えていた。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "RecoveryCaptureOccurrencesThroughNode",
            vec![
                rule(
                    &["@recovery(SKIP)"],
                    "M0",
                    Body::single(vec![Element::new(Atom::Literal("a".into()))]),
                ),
                rule(
                    &["@mapping(W, params=[head])"],
                    "W",
                    Body::single(vec![Element::captured(Atom::Ref("M0".into()), "head")]),
                ),
                rule(
                    &["@root", "@mapping(Top, params=[head, tail])"],
                    "Root",
                    Body::single(vec![
                        Element::captured(Atom::Ref("W".into()), "head"),
                        Element::captured(Atom::Literal("z".into()), "tail"),
                    ]),
                ),
            ],
        ),
        &["+z", "az", "z", "+"],
    );

    // 3. 間に AST ノードを作る規則が**無い**（`W` が transparent）ときは、
    //    外側の capture も回復領域をそのまま値に持つので両方数える。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "RecoveryCaptureOccurrencesTransparent",
            vec![
                rule(
                    &["@recovery(SKIP)"],
                    "M0",
                    Body::single(vec![Element::new(Atom::Literal("a".into()))]),
                ),
                rule(
                    &[],
                    "W",
                    Body::single(vec![Element::captured(Atom::Ref("M0".into()), "head")]),
                ),
                rule(
                    &["@root", "@mapping(Top, params=[head])"],
                    "Root",
                    Body::single(vec![Element::captured(Atom::Ref("W".into()), "head")]),
                ),
            ],
        ),
        &["+", "a"],
    );
}
