//! 意味値の領域の中にある **text だけの規則**（`P ::= 'i' | 'j'`、写像なし）への参照は、
//! 区切りとして値を作らないことを両バックエンドで確かめる（issue #86、D-032）。
//!
//! 直接 capture の帯の seed 1000189 で、`M0 ::= L1? @v3 imp.Piece+ % '\n' @v4 | 'c' @v5 '😀' @v6`
//! （`imp.Piece ::= 'i' | 'j'`）を要素に持つ `@tail` が、入力 `c😀,c😀i)2:1''` に対して
//! java=`[",", "c😀"]`、rust=`[",", "c😀", "i"]` を返した（`diff:/ast/fields/tail/*`）。
//! `L1?` は省略されて値を作らない。値の `"i"` は `L1?` の不在から来たのではなく、
//! `imp.Piece` の規則が「値の無い写像なし規則は一致範囲の字句を値にする」経路で作っていた。
//! Java の `combine` は null（plain text）の項目を値の列に数えないので、連接の中の text 規則は
//! 値にならない。Rust は `L1` があるときも `["GAMMA", "i"]` と余分な値を作り、scalar の field では
//! `mapped value type mismatch` になっていた。
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
fn optional(name: &str) -> Element {
    quantified(reference(name), Quant::Question)
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
fn text_rules_inside_semantic_regions_are_delimiters_on_both_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let mut engine = Engine::new(scratch.0.clone());
    let leaf =
        |start: usize| json!({"type": "Leaf", "span": [start, start + 1], "fields": {"v": "x"}});

    // issue #86 の形: 省略可能な値の後ろに text 規則の反復、mixed な選択の text 候補、その反復。
    // `L?` の不在は値を作らず、`P+` も値を作らない。text 候補 `'c' 'd'` は D-078 の字句 1 つ。
    let issue = grammar(
        "TextRuleIssue",
        vec![
            lit("["),
            captured(
                quantified(
                    choice(vec![
                        vec![optional("L"), quantified(reference("P"), Quant::Plus)],
                        vec![lit("c"), lit("d")],
                    ]),
                    Quant::Plus,
                ),
                "x",
            ),
            lit("]"),
        ],
    );
    assert_eq!(
        agree(&mut engine, &issue, &["[i]", "[cdi]", "[cdgammaij]"]),
        vec![json!([]), json!(["cd"]), json!(["cd", "GAMMA"])]
    );

    // scalar の選択（候補が採用されても省略の不在は値なし）。
    let scalar = grammar(
        "TextRuleScalarChoice",
        vec![
            lit("["),
            captured(
                choice(vec![
                    vec![optional("L"), quantified(reference("P"), Quant::Plus)],
                    vec![lit("c"), lit("d")],
                ]),
                "x",
            ),
            lit("]"),
        ],
    );
    assert_eq!(
        agree(&mut engine, &scalar, &["[i]", "[cd]", "[gammai]"]),
        vec![json!(null), json!("cd"), json!("GAMMA")]
    );

    // 連接の中の text 規則は、隣の値が有っても無くても値にならない（Rust は余分な字句で
    // `mapped value type mismatch` になっていた）。
    let cases: Vec<(&str, Seq, &[&str], Vec<Value>)> = vec![
        (
            "TextRuleOptionalEnum",
            bracketed(vec![optional("L"), reference("P")]),
            &["[i]", "[gammai]"],
            vec![json!(null), json!("GAMMA")],
        ),
        (
            "TextRuleEnum",
            bracketed(vec![reference("L"), reference("P")]),
            &["[gammaj]"],
            vec![json!("GAMMA")],
        ),
        (
            "TextRuleBeforeNode",
            bracketed(vec![reference("P"), reference("Leaf")]),
            &["[ix]"],
            vec![leaf(2)],
        ),
        (
            "TextRuleAfterDelimiter",
            bracketed(vec![optional("Leaf"), lit("-"), reference("P")]),
            &["[-i]", "[x-i]"],
            vec![json!(null), leaf(1)],
        ),
        // 値の領域の外では従来どおり: 規則そのものの capture は字句、text だけの連接は全体で 1 つ、
        // 反復の capture は要素ごと。
        (
            "TextRuleAlone",
            bracketed(vec![reference("P")]),
            &["[i]"],
            vec![json!("i")],
        ),
        (
            "TextRulePair",
            bracketed(vec![reference("P"), reference("P")]),
            &["[ij]"],
            vec![json!("ij")],
        ),
        (
            "TextRuleRepeat",
            vec![
                lit("["),
                captured(quantified(reference("P"), Quant::Star), "x"),
                lit("]"),
            ],
            &["[]", "[ij]"],
            vec![json!([]), json!(["i", "j"])],
        ),
    ];
    for (name, root, inputs, expected) in cases {
        let g = grammar(name, root);
        assert_eq!(agree(&mut engine, &g, inputs), expected, "{name}");
    }
}
