//! `@rightAssoc` の node span を両バックエンドで突き合わせる的を絞った差分テスト。
//!
//! 棚卸し（`docs/design/07-annotation-inventory.md` P7-1）で見つけた非対称の回帰テスト。
//! 乱択 campaign は `@rightAssoc` を生成しているが、固定 seed では
//! 「右再帰が実際に 2 段以上起きる入力」と「末尾 trivia がある入力」の組合せに
//! 当たっていなかったので、ここでは文法と入力を固定する。
//!
//! 正解は旧実装の 2 つの oracle が決めている。
//! - grammar 既定の trivia だけ（規則に trivia 注釈が無い）: 右再帰ノードの span は
//!   **消費範囲そのまま**（`unlaxer-dsl` `RightAssociativeMappingRuntimeTest`:
//!   `" 2 ^ 3 ^ 2 "` で root [0,11] / 右子 [5,11] / 葉 [9,11]）。
//! - 規則局所の trivia 注釈がある: 入れ子ノードの span は **significant end** で切る
//!   （`rule-trivia/corpus.json:14` RightScoped: `" x /*r*/ ^ x "` で外側 [0,13]・右子 [11,12]）。
//!
//! 前提ツールは `campaign.rs` と同じ（`javac` / `java` / mvn 済みの ubnfc-java）。

use std::path::PathBuf;
use std::process::Command;
use ubnfc_fuzz::engine::Engine;
use ubnfc_fuzz::model::{Atom, Body, Element, Grammar, Rule};
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

fn rule(annotations: &[&str], name: &str, body: Body) -> Rule {
    Rule::plain(
        annotations.iter().map(|a| (*a).to_owned()).collect(),
        name.to_owned(),
        body,
    )
}

/// `Base @left { Op @op Self @right }`（v1 の右結合正規形）。
fn right_assoc_body(base: Atom, operator: &str, self_name: &str) -> Body {
    Body::single(vec![
        Element::captured(base, "left"),
        Element::new(Atom::Repeat(Body::single(vec![
            Element::captured(Atom::Literal(operator.to_owned()), "op"),
            Element::captured(Atom::Ref(self_name.to_owned()), "right"),
        ]))),
    ])
}

fn grammar(name: &str, settings: &[&str], rules: Vec<Rule>) -> Grammar {
    Grammar {
        name: name.to_owned(),
        settings: settings.iter().map(|s| (*s).to_owned()).collect(),
        templates: Vec::new(),
        tokens: Vec::new(),
        rules,
        import: None,
        features: Default::default(),
    }
}

fn assert_backends_agree(engine: &mut Engine, grammar: &Grammar, inputs: &[&str]) {
    let built = engine
        .prepare(grammar)
        .unwrap_or_else(|d| panic!("{}: {} {}", grammar.name, d.code, d.message));
    let inputs: Vec<String> = inputs.iter().map(|s| (*s).to_owned()).collect();
    let java = engine.run(&built, Backend::Java, &inputs);
    let rust = engine.run(&built, Backend::Rust, &inputs);
    for (index, input) in inputs.iter().enumerate() {
        let found = differences(&java[index], &rust[index]);
        assert!(
            found.is_empty(),
            "{} {input:?}: {found:?}\njava={}\nrust={}",
            grammar.name,
            java[index],
            rust[index]
        );
    }
}

#[test]
fn right_associative_node_spans_agree_between_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let mut engine = Engine::new(scratch.0.clone());

    // 1. grammar 既定の trivia だけ（Power.ubnf と同じ形）。規則に trivia 注釈が無いので
    //    どちらのバックエンドも span 補正を掛けてはならない。Java はここを無条件に
    //    補正していた（右子の末尾空白ぶん span が縮む）。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "RightAssocGlobalTrivia",
            &["@package: fuzz.generated", "@whitespace: javaStyle"],
            vec![
                rule(
                    &[
                        "@root",
                        "@mapping(Pow, params=[left, op, right])",
                        "@rightAssoc",
                        "@precedence(level=10)",
                    ],
                    "Expr",
                    right_assoc_body(Atom::Ref("Atom".into()), "^", "Expr"),
                ),
                rule(
                    &[],
                    "Atom",
                    Body::single(vec![Element::new(Atom::Literal("x".into()))]),
                ),
            ],
        ),
        &[" x ^ x ^ x ", " x ^ x ", "x^x^x", "x", " x "],
    );

    // 1b. 同じく grammar 既定の trivia だけだが、`@left` が **リテラル**（値 span が
    //     末尾 trivia を含まない）。ここで Java の無条件補正が実際に効いてしまい、
    //     右再帰ノードの末尾が Rust より短くなっていた。1 の `@left` は規則参照で、
    //     その値 span が末尾 trivia を含むため補正が no-op になり差が出なかった。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "RightAssocGlobalTriviaLiteralLeft",
            &["@package: fuzz.generated", "@whitespace: javaStyle"],
            vec![rule(
                &[
                    "@root",
                    "@mapping(Pow, params=[left, op, right])",
                    "@rightAssoc",
                    "@precedence(level=10)",
                ],
                "Expr",
                right_assoc_body(Atom::Literal("x".into()), "^", "Expr"),
            )],
        ),
        &[" x ^ x ^ x ", " x ^ x ", "x^x^x", " x "],
    );

    // 2. 規則局所の trivia 注釈がある（rule-trivia corpus の RightScoped と同じ形）。
    //    入れ子ノードは significant end で切る。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "RightAssocLocalTrivia",
            &["@package: fuzz.generated"],
            vec![rule(
                &[
                    "@root",
                    "@mapping(Pow, params=[left, op, right])",
                    "@rightAssoc",
                    "@precedence(level=20)",
                    "@interleave(profile=commentsAndSpaces)",
                ],
                "Expr",
                right_assoc_body(Atom::Literal("x".into()), "^", "Expr"),
            )],
        ),
        &[" x /*r*/ ^ x ", " x ^ x ^ x ", "x^x"],
    );

    // 3. 規則局所の trivia を持つ右結合規則が entry でない場合。Rust は「入れ子呼出しか」
    //    （`depth > 1`）で、Java は「同じ規則の再帰呼出しか」で判定していたので、
    //    いちばん外側の Expr ノードの span が食い違っていた。
    assert_backends_agree(
        &mut engine,
        &grammar(
            "RightAssocNested",
            &["@package: fuzz.generated"],
            vec![
                rule(
                    &["@root", "@mapping(Root, params=[inner])"],
                    "Root",
                    Body::single(vec![Element::captured(Atom::Ref("Expr".into()), "inner")]),
                ),
                rule(
                    &[
                        "@mapping(Pow, params=[left, op, right])",
                        "@rightAssoc",
                        "@precedence(level=20)",
                        "@whitespace",
                    ],
                    "Expr",
                    right_assoc_body(Atom::Literal("x".into()), "^", "Expr"),
                ),
            ],
        ),
        &["x ^ x ^ x", "x^x", "x"],
    );
}
