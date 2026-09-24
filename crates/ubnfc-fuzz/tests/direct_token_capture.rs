//! token の**直接** capture（`NUMBER @x`）を両バックエンドで突き合わせる、的を絞った差分テスト。
//!
//! issue #75 の回帰試験。直接 capture の seed 帯（`gen::DIRECT_CAPTURE_SEED_BASE`）を
//! 旧 front で回すと、seed 1000002 で `/ast/fields/direct/fields/dn`（java=`0` rust=`"0"`）と
//! `/ast/fields/direct/fields/dl/0`（java=`2` rust=`"2"`）が出た。縮小後の入力は `iあ0:1''`。
//! ここでは文法を `Direct` 規則だけに縮めて固定し、比較は `observe::differences`
//! （campaign と同じ、射影なし）で行う。加えて canonical JSON の**値の型**を直接確かめる:
//! D-074 の正本は「token の直接 capture は原文の文字列」なので、Java も Rust も文字列を返す。
//!
//! 前提ツールは `campaign.rs` と同じ（`javac` / `java` / mvn 済みの ubnfc-java / `ubnfc-rust`）。

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use ubnfc_fuzz::engine::Engine;
use ubnfc_fuzz::model::{Atom, Body, Element, Grammar, Rule, Token};
use ubnfc_fuzz::observe::differences;
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

fn token(name: &str, declaration: &str) -> Token {
    Token {
        name: name.into(),
        declaration: declaration.into(),
        samples: Vec::new(),
        feature: "capture:directToken",
    }
}

/// 生成器の `Direct` 規則と同じ形。根は `@mapping` で `Direct` の node を field に持つ。
fn grammar(name: &str, v2: bool, tokens: [&str; 3]) -> Grammar {
    let reference = |name: &str| Atom::Ref(name.to_owned());
    let mut settings = vec!["@package: fuzz.generated".to_owned()];
    if v2 {
        settings.insert(0, "@ubnf: 2".to_owned());
    }
    Grammar {
        name: name.to_owned(),
        settings,
        templates: Vec::new(),
        tokens: vec![
            token("DN", tokens[0]),
            token("DI", tokens[1]),
            token("DS", tokens[2]),
        ],
        rules: vec![
            Rule::plain(
                vec!["@mapping(Direct, params=[dn, dg, di, dl, ds])".to_owned()],
                "Direct".into(),
                Body::single(vec![
                    Element::captured(reference("DN"), "dn"),
                    Element::new(Atom::Literal(":".into())),
                    Element::captured(
                        Atom::Group(Body::single(vec![Element::new(reference("DN"))])),
                        "dg",
                    ),
                    Element::new(Atom::Optional(Body::single(vec![Element::captured(
                        reference("DI"),
                        "di",
                    )]))),
                    Element::new(Atom::Repeat(Body::single(vec![
                        Element::new(Atom::Literal(",".into())),
                        Element::captured(reference("DN"), "dl"),
                    ]))),
                    Element::captured(reference("DS"), "ds"),
                ]),
            ),
            Rule::plain(
                vec![
                    "@root".to_owned(),
                    "@mapping(Top, params=[direct])".to_owned(),
                ],
                "Root".into(),
                Body::single(vec![Element::captured(reference("Direct"), "direct")]),
            ),
        ],
        import: None,
        features: Default::default(),
    }
}

/// `Direct` node の field がすべて文字列（リストなら文字列の列）であること。
fn assert_text_fields(backend: Backend, input: &str, response: &Value) {
    let direct = &response["ast"]["fields"]["direct"];
    assert_eq!(
        direct["type"], "Direct",
        "{backend:?} {input:?}: {response}"
    );
    for (field, value) in direct["fields"].as_object().unwrap() {
        let ok = match value {
            Value::String(_) | Value::Null => true,
            Value::Array(items) => items.iter().all(Value::is_string),
            _ => false,
        };
        assert!(
            ok,
            "{backend:?} {input:?}: field {field} は文字列でなければならない（D-074）: {value}"
        );
    }
}

fn assert_backends_agree(engine: &mut Engine, grammar: &Grammar, inputs: &[&str]) {
    let built = engine
        .prepare(grammar)
        .unwrap_or_else(|d| panic!("{}: {} {}", grammar.name, d.code, d.message));
    let inputs: Vec<String> = inputs.iter().map(|s| (*s).to_owned()).collect();
    let java = engine.run(&built, Backend::Java, &inputs);
    let rust = engine.run(&built, Backend::Rust, &inputs);
    let mut accepted = 0usize;
    for (index, input) in inputs.iter().enumerate() {
        // campaign.rs の KNOWN（A-6: 最遠位置の候補表示、D-027）だけは除く。
        let found: Vec<_> = differences(&java[index], &rust[index])
            .into_iter()
            .filter(|d| {
                !(d.path.starts_with("/diagnostics/") && d.path.contains("/farthestExpected"))
            })
            .collect();
        assert!(
            found.is_empty(),
            "{} {input:?}: {found:#?}\njava={}\nrust={}",
            grammar.name,
            java[index],
            rust[index]
        );
        if java[index]["allConsumed"] == Value::Bool(true) {
            accepted += 1;
            assert_text_fields(Backend::Java, input, &java[index]);
            assert_text_fields(Backend::Rust, input, &rust[index]);
        }
    }
    // 受理が 1 件も無ければ、この文法・入力は直接 capture を観測していない。
    assert!(accepted >= 3, "{}: 受理 {accepted} 件", grammar.name);
}

/// 縮小した seed 1000002 の入力（`0:1''`）と、符号・先頭ゼロ・小数・`int` 範囲外・
/// 反復の中の数値。旧 Java はここで int へ変換するか NumberFormatException を出していた。
const INPUTS: &[&str] = &[
    "0:1''",
    "1:1,2''",
    "+0007:-17ab,2147483648,1.5'😀'",
    "1e2:.5x1,0'q'",
    "7:7",
];

#[test]
fn direct_token_captures_are_lexemes_on_both_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let mut engine = Engine::new(scratch.0.clone());

    // v1: host クラス名の綴り。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "DirectV1",
            false,
            [
                "NumberParser",
                "org.unlaxer.parser.clang.IdentifierParser",
                "org.unlaxer.parser.elementary.SingleQuotedParser",
            ],
        ),
        INPUTS,
    );
    // v2: 中立語彙。
    assert_backends_agree(
        &mut engine,
        &grammar("DirectV2", true, ["NUMBER", "IDENTIFIER", "SINGLE_QUOTED"]),
        INPUTS,
    );
    // v2: extern scanner の直接 capture。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "DirectV2Extern",
            true,
            ["NUMBER", "IDENTIFIER", "extern 'SingleQuotedParser'"],
        ),
        INPUTS,
    );
}
