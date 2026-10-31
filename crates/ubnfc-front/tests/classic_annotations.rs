//! Classic の綴りのうち、引数付き規則（`classic_spelling.rs`）以外の残り（issue #8 の C 系統、D-085）。
//!
//! - plain form の注釈（`@ leftAssoc`、`(` の無い `@mapping`。unlaxer-parser #715）: 適用しない
//!   Simple 注釈にして `W-ANNOTATION-IGNORED`。
//! - ubnfc に無い意味の注釈（`@lexicalContext` / `@predicate` / `@namePredicate` /
//!   `@uniqueLongestChoice` / `@nestedCaptures`）: 位置付きの `E-ANNOTATION-UNSUPPORTED`。
//! - 版付き token adapter `ADAPTER('id', version=N)`（#158）: 位置付きの `E-TOKEN-ADAPTER-UNSUPPORTED`。
//! - 宣言的 token の `CHAR_RANGE` / `NEGATION` の `\uXXXX` / `\u{X…}`: code point へ展開する。

use serde_json::Value;
use ubnfc_front::pipeline::{validate_file, PipelineError};
use ubnfc_front::syntax::parse_file;

/// (code, 原文の切り出し) の一覧。警告も含める。
fn diagnostics(source: &str) -> Vec<(String, String)> {
    let text = source.to_owned();
    match validate_file("/virtual/main.ubnf", &move |_| Ok(text.clone())) {
        Ok(issues) => issues
            .into_iter()
            .map(|i| (i.code, String::new()))
            .collect(),
        Err(PipelineError::Resolve(errors)) => errors
            .into_iter()
            .map(|e| (e.code, source[e.span.start..e.span.end].to_owned()))
            .collect(),
        Err(PipelineError::Syntax { error, .. }) => {
            vec![(
                error.message,
                source[error.span.start..error.span.end].to_owned(),
            )]
        }
        Err(other) => vec![(other.to_string(), String::new())],
    }
}

fn codes(source: &str) -> Vec<String> {
    diagnostics(source).into_iter().map(|(c, _)| c).collect()
}

#[test]
fn plain_forms_are_simple_annotations_that_are_not_applied() {
    // `@ root` は適用されないので、root の無い grammar の警告も出る。
    let got = codes("grammar G { @ root R ::= 'x' ; }");
    assert!(got.contains(&"W-ANNOTATION-IGNORED".to_owned()), "{got:?}");
    assert!(got.contains(&"W-GENERAL-NO-ROOT".to_owned()), "{got:?}");
    // 引数が要る名前を `(` なしで書いた形も同じ。`(` の前の空白は構わない（適用される）。
    let got = codes("grammar G { @root @mapping R ::= 'x' ; }");
    assert!(got.contains(&"W-ANNOTATION-IGNORED".to_owned()), "{got:?}");
    let got = codes("grammar G { @root @mapping (R, params=[v]) R ::= 'x' @v ; }");
    assert!(got.is_empty(), "{got:?}");
    // 隣接した `@leftAssoc@skip` はどちらも通常の注釈。
    let file = parse_file("grammar G { @root @leftAssoc@skip R ::= 'x' ; }").unwrap();
    let names: Vec<_> = file.grammars[0].rules[0]
        .annotations
        .iter()
        .map(|a| serde_json::to_value(a).unwrap())
        .map(|v| v.as_object().unwrap().keys().next().unwrap().clone())
        .collect();
    assert_eq!(names, ["Root", "LeftAssoc", "Skip"]);
}

#[test]
fn classic_only_annotations_are_rejected_at_their_position() {
    let source = "grammar G { @ubnf: v2 token ID ::= 'a'; \
                  @root @lexicalContext(tokens=['ID'], literals=['>']) R ::= ID S T U V ; \
                  @predicate('octet') S ::= 's' ; \
                  @namePredicate(snapshot='cxx23', version='v1', name='name', kind='type') T ::= 't' @name ; \
                  @uniqueLongestChoice U ::= 'u' | 'uu' ; \
                  @nestedCaptures V ::= 'v' ; }";
    let mut got = diagnostics(source);
    got.sort();
    let mut want: Vec<(String, String)> = [
        "@lexicalContext(tokens=['ID'], literals=['>'])",
        "@namePredicate(snapshot='cxx23', version='v1', name='name', kind='type')",
        "@nestedCaptures",
        "@predicate('octet')",
        "@uniqueLongestChoice",
    ]
    .into_iter()
    .map(|s| ("E-ANNOTATION-UNSUPPORTED".to_owned(), s.to_owned()))
    .collect();
    want.sort();
    assert_eq!(got, want);
}

#[test]
fn token_adapters_are_rejected_at_their_position() {
    let source = "grammar G { token S = ADAPTER('tinyexpression.string', version=1) \
                  @root R ::= S ; }";
    assert_eq!(
        diagnostics(source),
        [(
            "E-TOKEN-ADAPTER-UNSUPPORTED".to_owned(),
            "token S = ADAPTER('tinyexpression.string', version=1)".to_owned()
        )]
    );
}

/// 宣言的 token の値を、位置を除いた JSON にする。
fn token_value(source: &str) -> Value {
    let file = parse_file(source).unwrap();
    let mut value = serde_json::to_value(&file.grammars[0].tokens[0].value).unwrap();
    fn strip(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.remove("span");
                map.values_mut().for_each(strip);
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    strip(&mut value);
    value
}

#[test]
fn declarative_tokens_expand_code_point_escapes_in_char_range_and_negation() {
    let escaped = token_value(
        r"grammar G { @ubnf: 2 token T ::= CHAR_RANGE('\u{1F600}','\u{1F601}') | NEGATION('\u0000\u0009'); }",
    );
    let literal = token_value(
        "grammar G { @ubnf: 2 token T ::= CHAR_RANGE('😀','😁') | NEGATION('\u{0}\t'); }",
    );
    assert_eq!(escaped, literal);
    for (source, code) in [
        (
            r"grammar G { @ubnf: 2 token T ::= CHAR_RANGE('\uZZZZ','a'); }",
            "E-TOKEN-ESCAPE-UNICODE",
        ),
        (
            r"grammar G { @ubnf: 2 token T ::= NEGATION('\uD800'); }",
            "E-TOKEN-ESCAPE-SURROGATE",
        ),
    ] {
        let error = parse_file(source).unwrap_err();
        assert!(
            error.message.starts_with(code),
            "{source}: {}",
            error.message
        );
    }
}
