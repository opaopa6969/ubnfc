//! mixed な選択（意味値を持つ候補と text だけの候補）の text 候補を、両バックエンドが
//! **一致範囲の字句 1 つの意味値**として外側の capture へ渡すことを確かめる（D-078）。
//!
//! issue #78 の回帰試験。直接 capture の帯の seed 1000002 で、根の
//! `( ',' @tail ( M1+ @v14 | '=' @v15 )? )* @tail` が縮小入力 `iあ,=0:1''` に対して
//! java=`tail: [",", "="]`、rust=`tail: [","]` を返した（`diff:/ast/fields/tail/*`）。
//! 旧実装は mixed な選択の text 候補に `#semantic:text` を束縛してその字句を値にしていた
//! （`CaptureBindingPlan.markSemanticValues`）。Java（`SemanticShape`）はそれと同じで、
//! Rust の生成器が list 形の選択で text 候補を落としていた。
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
fn leaf() -> Element {
    Element::new(Atom::Ref("Leaf".into()))
}
fn quantified(mut element: Element, quant: Quant) -> Element {
    element.quant = Some(quant);
    element
}
fn captured(mut element: Element, name: &str) -> Element {
    element.capture = Some(name.into());
    element
}
fn choice(alternatives: Vec<Seq>) -> Atom {
    Atom::Group(Body { alternatives })
}

/// `Leaf ::= T @v`（mapped node）と、`root` を本体に持つ `@root` の規則。
fn grammar(name: &str, params: &[&str], root: Seq) -> Grammar {
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
                vec!["@mapping(Leaf, params=[v])".to_owned()],
                "Leaf".into(),
                Body::single(vec![Element::captured(Atom::Ref("T".into()), "v")]),
            ),
            Rule::plain(
                vec![
                    "@root".to_owned(),
                    format!("@mapping(Top, params=[{}])", params.join(", ")),
                ],
                "Root".into(),
                Body::single(root),
            ),
        ],
        import: None,
        features: Default::default(),
    }
}

/// 両 backend の応答を突き合わせ、全入力受理した応答の AST を返す（Java 側）。
fn agree(engine: &mut Engine, grammar: &Grammar, inputs: &[&str]) -> Vec<Value> {
    let built = engine
        .prepare(grammar)
        .unwrap_or_else(|d| panic!("{}: {} {}", grammar.name, d.code, d.message));
    let inputs: Vec<String> = inputs.iter().map(|s| (*s).to_owned()).collect();
    let java = engine.run(&built, Backend::Java, &inputs);
    let rust = engine.run(&built, Backend::Rust, &inputs);
    let mut asts = Vec::new();
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
        asts.push(java[index]["ast"]["fields"].clone());
    }
    asts
}

#[test]
fn text_alternatives_of_mixed_choices_are_semantic_values_on_both_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let mut engine = Engine::new(scratch.0.clone());

    // issue #78 の形: 外側 `@tail` は各反復要素の意味値（`','` は区切りで値にならない）。
    // 内側 `','` の `@tail` が先、外側が後（完了順、D-032 の表）。
    let issue = grammar(
        "MixedTail",
        &["tail", "v14", "v15"],
        vec![captured(
            quantified(
                Element::new(choice(vec![vec![
                    captured(lit(","), "tail"),
                    quantified(
                        Element::new(choice(vec![
                            vec![captured(quantified(leaf(), Quant::Plus), "v14")],
                            vec![captured(lit("="), "v15")],
                        ])),
                        Quant::Question,
                    ),
                ]])),
                Quant::Star,
            ),
            "tail",
        )],
    );
    let asts = agree(&mut engine, &issue, &[",=", ",", ",=,x,,="]);
    let x = json!({"type": "Leaf", "span": [3, 4], "fields": {"v": "x"}});
    assert_eq!(
        asts[0],
        json!({"tail": [",", "="], "v14": [], "v15": ["="]})
    );
    assert_eq!(asts[1], json!({"tail": [","], "v14": [], "v15": []}));
    assert_eq!(
        asts[2],
        json!({"tail": [",", "=", ",", x, ",", ",", "="], "v14": [x], "v15": ["=", "="]})
    );

    // list 形の選択を直接 capture: text 候補（literal 1 つ / 連接）も 1 つの値。
    let many = grammar(
        "MixedMany",
        &["many"],
        vec![captured(
            quantified(
                Element::new(choice(vec![
                    vec![quantified(leaf(), Quant::Plus)],
                    vec![lit("=")],
                    vec![lit("<"), lit(">")],
                ])),
                Quant::Star,
            ),
            "many",
        )],
    );
    let asts = agree(&mut engine, &many, &["=", "x<>=", "<>"]);
    assert_eq!(asts[0], json!({"many": ["="]}));
    assert_eq!(asts[2], json!({"many": ["<>"]}));

    // scalar の mixed（従来どおり、corpus `conformance-nested-captures-text-node-and-mixed-values`）。
    let one = grammar(
        "MixedOne",
        &["one"],
        vec![captured(
            Element::new(choice(vec![vec![leaf()], vec![lit("=")]])),
            "one",
        )],
    );
    let asts = agree(&mut engine, &one, &["=", "x"]);
    assert_eq!(asts[0], json!({"one": "="}));

    // 先頭要素を共有する連接候補（factor.rs が 1 回だけ評価する経路）。
    let factored = grammar(
        "MixedFactored",
        &["fac"],
        vec![captured(
            quantified(
                Element::new(choice(vec![
                    vec![lit("-"), lit("-")],
                    vec![lit("-"), leaf()],
                ])),
                Quant::Star,
            ),
            "fac",
        )],
    );
    let asts = agree(&mut engine, &factored, &["--", "-x--"]);
    assert_eq!(asts[0], json!({"fac": ["--"]}));

    // 空の一致・反復を含む text 候補。候補の値は一致範囲の字句そのもので、空でも 1 つ
    // （`""`）。内側の反復が 0 回でも（`'='?` が無い `-`）字句を落とさない。
    let empty: Vec<(&str, Seq, &[&str], Vec<Value>)> = vec![
        (
            "EmptyToken",
            vec![Element::new(Atom::Ref("Z".into()))],
            &["[]"],
            vec![json!({"x": [""]})],
        ),
        (
            "EmptyStar",
            vec![quantified(lit("="), Quant::Star)],
            &["[]", "[==]"],
            vec![json!({"x": [""]}), json!({"x": ["=="]})],
        ),
        (
            "InnerOptional",
            vec![quantified(
                Element::new(choice(vec![vec![
                    lit("-"),
                    quantified(lit("="), Quant::Question),
                ]])),
                Quant::Star,
            )],
            &["[-]", "[-=-]"],
            vec![json!({"x": ["-"]}), json!({"x": ["-=-"]})],
        ),
    ];
    for (name, alternative, inputs, expected) in empty {
        let mut g = grammar(
            name,
            &["x"],
            vec![
                lit("["),
                captured(
                    Element::new(choice(vec![
                        vec![quantified(leaf(), Quant::Plus)],
                        alternative,
                    ])),
                    "x",
                ),
                lit("]"),
            ],
        );
        g.tokens.push(Token {
            name: "Z".into(),
            declaration: "NEGATIVE_LOOKAHEAD('#')".into(),
            samples: Vec::new(),
            feature: "token",
        });
        assert_eq!(agree(&mut engine, &g, inputs), expected, "{name}");
    }
}
