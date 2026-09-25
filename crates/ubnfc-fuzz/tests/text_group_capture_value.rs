//! node 変換の field で、**中身が text だけの group の capture**（`( P P ) @x`、`P ::= 'i' | 'j'` は写像なし）
//! は出現 1 つにつき一致範囲の字句 1 つを値にすることを両バックエンドで確かめる（issue #89、D-032）。
//!
//! `Root ::= '[' ( P P ) @x Leaf @x ']'` で入力 `[ijx]` に対し java=`["ij", Leaf]`、rust=`["i", "j", Leaf]` だった。
//! `x` は `Leaf` の site を持つので node 変換になり、Rust の `node_values` が capture の子を組み立てて
//! 写像なし規則 `P` の `Event::Rule` ごとに字句を足していた。`( P P ) @x` だけなら text 変換で差は出ない。
//! D-032 は「site より内側で text だけの領域は一致範囲全体で 1 つの値」。Java（旧実装も）は plain text の
//! 出現を `Value(start, end)` 1 つにする。出現が 2 つ（`P @x P @x`）なら値も 2 つ。
//!
//! 前提ツールは `campaign.rs` と同じ（`javac` / `java` / mvn 済みの ubnfc-java / `ubnfc-rust`）。

use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use ubnfc_fuzz::engine::Engine;
use ubnfc_fuzz::model::{Atom, Body, Element, Grammar, Quant, Rule, Seq, Token};
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

fn lit(text: &str) -> Element {
    Element::new(Atom::Literal(text.into()))
}
fn reference(name: &str) -> Element {
    Element::new(Atom::Ref(name.into()))
}
fn quantified(mut element: Element, quant: Quant) -> Element {
    element.quant = Some(quant);
    element
}
fn captured(mut element: Element, name: &str) -> Element {
    element.capture = Some(name.into());
    element
}
fn choice(alternatives: Vec<Seq>) -> Element {
    Element::new(Atom::Group(Body { alternatives }))
}
/// `'[' ( items ) @x ']'`
fn bracketed(items: Seq) -> Seq {
    vec![lit("["), captured(choice(vec![items]), "x"), lit("]")]
}

/// `@enum L ::= 'gamma'`、text だけの `P ::= 'i' | 'j'`、写像 `Leaf ::= T @v` と根。
fn grammar(name: &str, root: Seq) -> Grammar {
    Grammar {
        name: name.to_owned(),
        settings: vec!["@package: fuzz.generated".to_owned()],
        templates: Vec::new(),
        tokens: vec![Token {
            name: "T".into(),
            declaration: "org.unlaxer.parser.clang.IdentifierParser".into(),
            samples: Vec::new(),
            feature: "token",
        }],
        rules: vec![
            Rule::plain(
                vec!["@enum".to_owned()],
                "L".into(),
                Body::single(vec![lit("gamma")]),
            ),
            Rule::plain(
                Vec::new(),
                "P".into(),
                Body {
                    alternatives: vec![vec![lit("i")], vec![lit("j")]],
                },
            ),
            Rule::plain(
                vec!["@mapping(Leaf, params=[v])".to_owned()],
                "Leaf".into(),
                Body::single(vec![Element::captured(Atom::Ref("T".into()), "v")]),
            ),
            Rule::plain(
                vec!["@root".to_owned(), "@mapping(Top, params=[x])".to_owned()],
                "Root".into(),
                Body::single(root),
            ),
        ],
        import: None,
        features: Default::default(),
    }
}

/// 両 backend の応答を突き合わせ、全入力受理した応答の `x` を返す（Java 側）。
fn agree(engine: &mut Engine, grammar: &Grammar, inputs: &[&str]) -> Vec<Value> {
    let built = engine
        .prepare(grammar)
        .unwrap_or_else(|d| panic!("{}: {} {}", grammar.name, d.code, d.message));
    let inputs: Vec<String> = inputs.iter().map(|s| (*s).to_owned()).collect();
    let java = engine.run(&built, Backend::Java, &inputs);
    let rust = engine.run(&built, Backend::Rust, &inputs);
    let mut values = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
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
        assert_eq!(
            java[index]["allConsumed"],
            Value::Bool(true),
            "{} {input:?}: {}",
            grammar.name,
            java[index]
        );
        values.push(java[index]["ast"]["fields"]["x"].clone());
    }
    values
}

#[test]
fn text_only_group_captures_yield_one_value_per_occurrence_on_both_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let mut engine = Engine::new(scratch.0.clone());
    let leaf =
        |start: usize| json!({"type": "Leaf", "span": [start, start + 1], "fields": {"v": "x"}});
    let pair = || choice(vec![vec![reference("P"), reference("P")]]);
    let cases: Vec<(&str, Seq, &[&str], Vec<Value>)> = vec![
        // issue #89 の形: node 変換の field でも group の出現は字句 1 つ。
        (
            "TextGroupThenNode",
            vec![
                lit("["),
                captured(pair(), "x"),
                captured(reference("Leaf"), "x"),
                lit("]"),
            ],
            &["[ijx]"],
            vec![json!(["ij", leaf(3)])],
        ),
        // text 変換（site が text だけ）の group は従来どおり字句 1 つ。
        (
            "TextGroupAlone",
            bracketed(vec![reference("P"), reference("P")]),
            &["[ij]"],
            vec![json!("ij")],
        ),
        // 出現が 2 つなら値も 2 つ（text 変換 / node 変換の両方）。
        (
            "TextRuleTwoSites",
            vec![
                lit("["),
                captured(reference("P"), "x"),
                captured(reference("P"), "x"),
                lit("]"),
            ],
            &["[ij]"],
            vec![json!(["i", "j"])],
        ),
        (
            "TextRuleTwoSitesThenNode",
            vec![
                lit("["),
                captured(reference("P"), "x"),
                captured(reference("P"), "x"),
                captured(reference("Leaf"), "x"),
                lit("]"),
            ],
            &["[ijx]"],
            vec![json!(["i", "j", leaf(3)])],
        ),
        // 反復の capture は要素ごとに出現し、各要素（text だけの group）が字句 1 つ。
        (
            "TextGroupRepeatThenNode",
            vec![
                lit("["),
                captured(quantified(pair(), Quant::Star), "x"),
                captured(reference("Leaf"), "x"),
                lit("]"),
            ],
            &["[x]", "[ijjix]"],
            vec![json!([leaf(1)]), json!(["ij", "ji", leaf(5)])],
        ),
        // 省略可能な group: 不在は値なし、有れば字句 1 つ。
        (
            "TextGroupOptionalThenNode",
            vec![
                lit("["),
                captured(quantified(pair(), Quant::Question), "x"),
                captured(reference("Leaf"), "x"),
                lit("]"),
            ],
            &["[x]", "[ijx]"],
            vec![json!([leaf(1)]), json!(["ij", leaf(3)])],
        ),
        // D-078: 意味値を持つ選択の text 候補は字句 1 つ（site は text だけではない）。
        // `T` は IdentifierParser で貪欲なので、隣の `Leaf` とは `-` で区切る。
        (
            "MixedChoiceThenNode",
            vec![
                lit("["),
                captured(
                    choice(vec![
                        vec![reference("P"), reference("P")],
                        vec![reference("Leaf")],
                    ]),
                    "x",
                ),
                lit("-"),
                captured(reference("Leaf"), "x"),
                lit("]"),
            ],
            &["[ij-x]", "[x-x]"],
            vec![json!(["ij", leaf(4)]), json!([leaf(1), leaf(3)])],
        ),
        // 意味値の領域の中の text 規則は区切り（#86）のまま。
        (
            "TextRuleInsideNodeGroupThenNode",
            vec![
                lit("["),
                captured(choice(vec![vec![reference("P"), reference("Leaf")]]), "x"),
                lit("-"),
                captured(reference("Leaf"), "x"),
                lit("]"),
            ],
            &["[ix-x]"],
            vec![json!([leaf(2), leaf(4)])],
        ),
    ];
    for (name, root, inputs, expected) in cases {
        let g = grammar(name, root);
        assert_eq!(agree(&mut engine, &g, inputs), expected, "{name}");
    }
}
