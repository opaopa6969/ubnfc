//! 順序付き選択の到達不能候補（D-037）。警告だけで受理は変えない。
//!
//! 期待値は「証明できた候補だけを鳴らす」こと。偽陽性は実在文法で最も高くつくので、
//! 負例（鳴ってはいけない形）も同じ数だけ固定する。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ubnfc_front::pipeline::{compile, validate_file};

fn codes(text: &str) -> Vec<String> {
    let issues = validate_file("/virtual/main.ubnf", &|path| {
        assert_eq!(path, Path::new("/virtual/main.ubnf"));
        Ok(text.into())
    })
    .unwrap();
    assert!(
        issues.iter().all(|i| i.severity() != "ERROR"),
        "{text}: {issues:?}"
    );
    issues
        .iter()
        .filter(|i| i.code.starts_with("W-CHOICE-"))
        .map(|i| format!("{} {}", i.code, i.rule.clone().unwrap_or_default()))
        .collect()
}

const EOF: &str = "token EOF = org.unlaxer.parser.elementary.EndOfSourceParser";

fn grammar(body: &str) -> String {
    format!("grammar G {{ {EOF} @root Root ::= Pick EOF ; {body} }}")
}

#[test]
fn a_nullable_alternative_that_is_not_last_kills_every_alternative_after_it() {
    assert_eq!(
        codes(&grammar("Pick ::= Opt | 'x' | 'y' ; Opt ::= [ 'a' ] ;")),
        ["W-CHOICE-NULLABLE-NOT-LAST Pick"]
    );
    assert_eq!(
        codes(&grammar("Pick ::= { 'a' } | 'x' ;")),
        ["W-CHOICE-NULLABLE-NOT-LAST Pick"]
    );
    // 末尾なら死なない。ゼロ幅成功が条件付き（先読み・EOF）なら証明できない。
    assert!(codes(&grammar("Pick ::= 'x' | [ 'a' ] ;")).is_empty());
    assert!(codes(&format!(
        "grammar G {{ {EOF} token LA = LOOKAHEAD('a') @root Root ::= Pick EOF ; Pick ::= LA | 'x' ; }}"
    ))
    .is_empty());
    assert!(codes(&grammar("Pick ::= EOF | 'x' ;")).is_empty());
}

#[test]
fn an_earlier_literal_that_is_a_prefix_of_a_later_one_makes_the_later_one_dead() {
    assert_eq!(
        codes(&grammar("Pick ::= 'a' | 'ab' ;")),
        ["W-CHOICE-PREFIX Pick"]
    );
    // 接頭辞は後続要素まで連結して見る。
    assert_eq!(
        codes(&grammar("Pick ::= 'ab' | 'a' 'b' 'c' ;")),
        ["W-CHOICE-PREFIX Pick"]
    );
    // rule 越しでも見る。
    assert_eq!(
        codes(&grammar("Pick ::= 'a' | Long ; Long ::= 'abc' ;")),
        ["W-CHOICE-PREFIX Pick"]
    );
    // 逆順（長い方が先）は健全な書き方。case-insensitive は証明しない。
    assert!(codes(&grammar("Pick ::= 'ab' | 'a' ;")).is_empty());
    assert!(codes(&grammar("Pick ::= 'a' | 'b' ;")).is_empty());
    assert!(codes(&grammar("Pick ::= 'a' | ( 'ab' | 'b' ) ;")).is_empty());
}

#[test]
fn a_later_alternative_that_always_starts_with_an_earlier_one_is_unreachable() {
    assert_eq!(
        codes(&grammar("Pick ::= Num | Num '+' Num ; Num ::= '1' | '2' ;")),
        ["W-CHOICE-UNREACHABLE Pick"]
    );
    assert_eq!(
        codes(&grammar("Pick ::= Num | Num ; Num ::= '1' ;")),
        ["W-CHOICE-UNREACHABLE Pick"]
    );
    // 構造同一の候補（rule 呼出しでなくても）。
    assert_eq!(
        codes(&grammar("Pick ::= 'a' 'b' | 'x' | 'a' 'b' ;")),
        ["W-CHOICE-UNREACHABLE Pick"]
    );
    // 先行候補が後続の一部の導出にしか現れないなら unreachable ではない。
    assert!(codes(&grammar(
        "Pick ::= Num | Other ; Num ::= '1' ; Other ::= '2' ;"
    ))
    .is_empty());
}

#[test]
fn a_derivation_that_starts_with_an_earlier_alternative_and_consumes_more_is_shadowed() {
    // P4 の Expression と同じ形。Add は Num で始まる導出を持つが、Num でない導出もある。
    let issues = codes(&grammar(
        "Pick ::= Num | Add ; Add ::= Num '+' Num | 'z' ; Num ::= '1' | '2' ;",
    ));
    assert_eq!(issues, ["W-CHOICE-SHADOWED-PATH Pick"]);
    // 余分な消費が保証されないなら鳴らさない（`Num { }` は Num と同じ長さで終われる）。
    assert!(codes(&grammar(
        "Pick ::= Num | Add ; Add ::= Num [ '+' Num ] | 'z' ; Num ::= '1' ;"
    ))
    .is_empty());
}

#[test]
fn the_check_is_deterministic_and_does_not_change_acceptance() {
    let text = grammar("Pick ::= Num | Add ; Add ::= Num '+' Num | 'z' ; Num ::= '1' | '2' ;");
    let first = codes(&text);
    assert_eq!(first, codes(&text));
    let compiled = compile("/virtual/main.ubnf", &|_| Ok(text.clone())).unwrap();
    assert_eq!(
        compiled
            .warnings
            .iter()
            .filter(|i| i.code.starts_with("W-CHOICE-"))
            .count(),
        1
    );
    // 警告は IR に載らない。
    assert!(!ubnfc_front::ir::to_canonical_json(&compiled.ir).contains("W-CHOICE"));
}

fn fixture_root(variable: &str, default: &str) -> PathBuf {
    let root = std::env::var_os(variable).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(default),
        PathBuf::from,
    );
    assert!(root.is_dir(), "{} ({variable})", root.display());
    root
}

/// 24 fixture + P4 を実測した結果。`docs/reports/2026-09-23-choice-reachability.md` の表と同じ。
const FILES: &[(&str, usize)] = &[
    ("story/code/calc-var.ubnf", 0),
    ("story/code/calc.ubnf", 0),
    ("story/code/calc-leftrec.ubnf", 0),
    (
        "rust/unlaxer-ubnf/tests/fixtures/positive/boundaries.ubnf",
        0,
    ),
    // 解決/検証でエラーになる 3 件は lower まで進まないので 0 のまま。
    (
        "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
        0,
    ),
    ("rust/unlaxer-ubnf/tests/fixtures/positive/elements.ubnf", 0),
    (
        "rust/unlaxer-ubnf/tests/fixtures/known/eval-default.ubnf",
        0,
    ),
    (
        "rust/unlaxer-ubnf/tests/fixtures/known/keyword-boundary.ubnf",
        0,
    ),
    (
        "rust/unlaxer-ubnf/tests/fixtures/known/common-field-space.ubnf",
        0,
    ),
    (
        "unlaxer-dsl/src/test/resources/right-associative/Power.ubnf",
        0,
    ),
    (
        "unlaxer-dsl/src/test/resources/mixed-values/Delimited.ubnf",
        0,
    ),
    ("unlaxer-dsl/src/test/resources/mixed-values/Mixed.ubnf", 0),
    ("unlaxer-dsl/src/test/resources/mixed-values/Shared.ubnf", 0),
    (
        "unlaxer-dsl/src/test/resources/evolution/0/Evolution.ubnf",
        0,
    ),
    (
        "unlaxer-dsl/src/test/resources/evolution/1/Evolution.ubnf",
        0,
    ),
    (
        "unlaxer-dsl/src/test/resources/evolution/2/Evolution.ubnf",
        0,
    ),
    (
        "unlaxer-dsl/src/test/resources/evolution/3/Evolution.ubnf",
        0,
    ),
    (
        "unlaxer-dsl/src/test/resources/associative/Operators.ubnf",
        0,
    ),
    (
        "unlaxer-dsl/src/test/resources/cardinality/Cardinality.ubnf",
        0,
    ),
    ("unlaxer-dsl/grammar/ubnf.ubnf", 0),
    ("unlaxer-dsl/tinycalc-vscode/grammar/tinycalc.ubnf", 1),
    ("unlaxer-dsl/examples/tinycalc.ubnf", 1),
];

#[test]
fn every_repository_grammar_reports_exactly_the_recorded_choice_warnings() {
    let unlaxer = fixture_root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser");
    let tiny = fixture_root("TINYEXPRESSION_DIR", "../../../tinyexpression");
    let mut total = BTreeMap::new();
    // all-tokens / namespace は resolve でエラーになるので validate_file が Err を返す。
    for (relative, expected) in FILES {
        let issues = validate_file(unlaxer.join(relative), &|p| std::fs::read_to_string(p))
            .unwrap_or_else(|e| panic!("{relative}: {e}"));
        let choice: Vec<_> = issues
            .iter()
            .filter(|i| i.code.starts_with("W-CHOICE-"))
            .collect();
        assert_eq!(choice.len(), *expected, "{relative}: {choice:?}");
        for issue in choice {
            *total.entry(issue.code.clone()).or_insert(0usize) += 1;
        }
    }
    let p4 = tiny.join("tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf");
    let issues = validate_file(&p4, &|p| std::fs::read_to_string(p)).unwrap();
    let choice: Vec<_> = issues
        .iter()
        .filter(|i| i.code.starts_with("W-CHOICE-"))
        .collect();
    assert_eq!(choice.len(), 9, "P4: {choice:?}");
    // 報告の見出しになった実例: Expression の NumberExpression が BooleanExpression を覆う。
    assert!(choice
        .iter()
        .any(|i| i.rule.as_deref() == Some("Expression")
            && i.code == "W-CHOICE-SHADOWED-PATH"
            && i.message.contains("alternative 2")
            && i.message.contains("NumberExpression")));
    for issue in choice {
        *total.entry(issue.code.clone()).or_insert(0usize) += 1;
    }
    assert_eq!(
        total,
        BTreeMap::from([
            ("W-CHOICE-PREFIX".to_owned(), 2),
            ("W-CHOICE-SHADOWED-PATH".to_owned(), 9),
        ])
    );
}
