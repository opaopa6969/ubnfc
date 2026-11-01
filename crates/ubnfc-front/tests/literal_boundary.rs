//! リテラルの語境界（D-040、`06-ubnf-v2.md` §3）。
//!
//! v1 は凍結なので `wordBoundary` は常に false。v2 は既定で「末尾が識別子構成文字の
//! リテラルにだけ」境界が付く。`@literalBoundary: word|none` が両方向で上書きする。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ubnfc_front::ir::{ExprNode, TokenSpec};
use ubnfc_front::pipeline::{compile, Compiled, PipelineError};

fn source(text: &str) -> Result<Compiled, PipelineError> {
    let files: BTreeMap<PathBuf, String> =
        BTreeMap::from([("/virtual/main.ubnf".into(), text.into())]);
    compile("/virtual/main.ubnf", &|path: &Path| Ok(files[path].clone()))
}

fn codes(text: &str) -> Vec<String> {
    match source(text) {
        Ok(compiled) => compiled.warnings.iter().map(|i| i.code.clone()).collect(),
        Err(PipelineError::Validation(issues) | PipelineError::Strict(issues)) => {
            issues.iter().map(|i| i.code.clone()).collect()
        }
        Err(other) => panic!("想定外のエラー: {other}"),
    }
}

/// リテラル値 → 語境界。同じ値が複数あれば OR ではなく最後の出現を採る（同一文法なら同値）。
fn boundaries(compiled: &Compiled) -> BTreeMap<String, bool> {
    compiled
        .ir
        .semantics
        .expressions
        .iter()
        .filter_map(|e| match &e.node {
            ExprNode::Literal {
                value,
                word_boundary,
                ..
            } => Some((value.clone(), *word_boundary)),
            _ => None,
        })
        .collect()
}

fn grammar(settings: &str, body: &str) -> String {
    format!(
        "grammar G {{\n  @package: org.example\n{settings}  @whitespace: javaStyle\n  \
         token IDENTIFIER = IdentifierParser\n{body}}}"
    )
}

/// 末尾が識別子構成文字・記号・非 ASCII を混ぜた本体。
const BODY: &str = "  @root R ::= ( 'let' | '+=' | '(' | 'x9' | '_' | 'あ' | 'aあ' ) IDENTIFIER;\n";

#[test]
fn v1_never_puts_a_boundary_on_any_literal() {
    for settings in ["", "  @ubnf: 1\n"] {
        let compiled = source(&grammar(settings, BODY)).unwrap();
        assert!(
            boundaries(&compiled).values().all(|b| !b),
            "v1 は凍結: {settings:?} {:?}",
            boundaries(&compiled)
        );
    }
}

#[test]
fn v2_bounds_only_the_literals_that_end_in_an_identifier_character() {
    let compiled = source(&grammar("  @ubnf: 2\n", BODY)).unwrap();
    let map = boundaries(&compiled);
    // 末尾が ASCII [A-Za-z0-9_] のものだけ。記号終わりと非 ASCII 終わりは対象外。
    let bounded: Vec<&str> = map
        .iter()
        .filter(|(_, b)| **b)
        .map(|(v, _)| v.as_str())
        .collect();
    assert_eq!(bounded, ["_", "let", "x9"], "{map:?}");
}

#[test]
fn the_setting_overrides_the_version_default_in_both_directions() {
    let off = source(&grammar("  @ubnf: 2\n  @literalBoundary: none\n", BODY)).unwrap();
    assert!(boundaries(&off).values().all(|b| !b));
    let on = source(&grammar("  @literalBoundary: word\n", BODY)).unwrap();
    assert!(boundaries(&on)["let"]);
    // 版そのものは設定で動かない。
    assert_eq!(on.ir.language_version, 1);
}

#[test]
fn a_bad_or_duplicated_setting_is_rejected_by_code() {
    assert!(codes(&grammar("  @literalBoundary: yes\n", BODY))
        .contains(&"E-LITERAL-BOUNDARY-VALUE".to_owned()));
    assert!(codes(&grammar(
        "  @literalBoundary: word\n  @literalBoundary: none\n",
        BODY
    ))
    .contains(&"E-LITERAL-BOUNDARY-DUPLICATE".to_owned()));
}

#[test]
fn case_insensitive_tokens_follow_the_same_rule() {
    let body = "  @root R ::= KW IDENTIFIER;\n";
    let with_ci = |settings: &str| {
        let text = format!(
            "grammar G {{\n  @package: org.example\n{settings}  @whitespace: javaStyle\n  \
             token IDENTIFIER = IdentifierParser\n  token KW = CI('select')\n{body}}}"
        );
        let compiled = source(&text).unwrap();
        compiled
            .ir
            .semantics
            .tokens
            .iter()
            .find_map(|t| match &t.spec {
                TokenSpec::CaseInsensitive { word_boundary, .. } => Some(*word_boundary),
                _ => None,
            })
            .unwrap()
    };
    assert!(!with_ci(""), "v1 は凍結");
    assert!(with_ci("  @ubnf: 2\n"));
    assert!(!with_ci("  @ubnf: 2\n  @literalBoundary: none\n"));
}

// ── W-CHOICE-PREFIX の健全性（D-037 の証明は境界に依存する） ─────────────────

fn choice_codes(settings: &str, alternatives: &str) -> Vec<String> {
    codes(&grammar(
        settings,
        &format!("  @root R ::= ( {alternatives} ) IDENTIFIER;\n"),
    ))
}

#[test]
fn the_prefix_check_is_silent_once_the_shorter_literal_has_a_boundary() {
    // 境界なし（v1）: 'var' が 'variable' を殺す。
    assert!(choice_codes("", "'var' | 'variable'").contains(&"W-CHOICE-PREFIX".to_owned()));
    // 境界あり（v2）: 'var' は `variable` に一致しないので 'variable' は生きている。
    assert!(
        !choice_codes("  @ubnf: 2\n", "'var' | 'variable'").contains(&"W-CHOICE-PREFIX".to_owned())
    );
}

#[test]
fn the_prefix_check_still_fires_when_the_next_character_cannot_be_an_identifier() {
    // 'x' の直後が '+' なので境界は満たされる。`'x'` は必ず成功し `'x+y'` は死ぬ。
    for settings in ["", "  @ubnf: 2\n"] {
        assert!(
            choice_codes(settings, "'x' | 'x+y'").contains(&"W-CHOICE-PREFIX".to_owned()),
            "{settings:?}"
        );
    }
    // 記号で終わるリテラルは境界が付かないので、v2 でも従来どおり。
    for settings in ["", "  @ubnf: 2\n"] {
        assert!(
            choice_codes(settings, "'+' | '+='").contains(&"W-CHOICE-PREFIX".to_owned()),
            "{settings:?}"
        );
    }
}

#[test]
fn classic_v2_preserves_prefix_literals_even_with_a_reserved_none_token() {
    let body = "token none ::= '#'; @root Root ::= 'a' 'b';";
    let classic = source(&grammar("@ubnf: v2", body)).unwrap();
    for expr in &classic.ir.semantics.expressions {
        if let ExprNode::Literal { word_boundary, .. } = &expr.node {
            assert!(!word_boundary);
        }
    }
    let explicit = source(&grammar("@ubnf: v2 @literalBoundary: word", body)).unwrap();
    assert!(explicit.ir.semantics.expressions.iter().any(|e| matches!(
        e.node,
        ExprNode::Literal {
            word_boundary: true,
            ..
        }
    )));
}
