use std::path::{Path, PathBuf};
use ubnfc_front::syntax;

fn root(variable: &str, default: &str) -> PathBuf {
    std::env::var_os(variable).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(default),
        PathBuf::from,
    )
}

fn same(source: &str) {
    let expected = syntax::parse_file(source).unwrap();
    let actual = ubnfc_selfhost::parse_file(source).unwrap();
    // 失敗時は最初の差分のみ示し、巨大な AST dump を避ける。
    if actual != expected {
        let expected = serde_json::to_value(expected).unwrap();
        let actual = serde_json::to_value(actual).unwrap();
        panic!("{}", difference(&actual, &expected, "").unwrap());
    }
}

fn difference(a: &serde_json::Value, b: &serde_json::Value, path: &str) -> Option<String> {
    if a == b {
        return None;
    }
    match (a, b) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) if a.keys().eq(b.keys()) => a
            .iter()
            .find_map(|(key, value)| difference(value, &b[key], &format!("{path}/{key}"))),
        (serde_json::Value::Array(a), serde_json::Value::Array(b)) if a.len() == b.len() => a
            .iter()
            .zip(b)
            .enumerate()
            .find_map(|(i, (a, b))| difference(a, b, &format!("{path}/{i}"))),
        _ => Some(format!("{path}: selfhost={a}, handwritten={b}")),
    }
}

#[test]
fn self_description_ast_matches_including_every_span() {
    let source = std::fs::read_to_string(
        root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser").join("unlaxer-dsl/grammar/ubnf.ubnf"),
    )
    .unwrap();
    same(&source);
}

#[test]
fn suffixes_escapes_comments_and_empty_sequences_match() {
    for source in [
        "", "// 文法😀\r\n", "grammar G {}",
        "grammar G { R ::= A+{2,3}% (B*) @xs; }",
        "grammar G { R ::= ('a' | )+ {2,} % ',' @xs; }",
        "grammar G { R ::= @typeof(xs) [ A+ ] {B} ('😀\\n\\t\\r\\'\\\\\\q')*{2} % (C? | [D+]) @ys; }",
        "grammar G { R ::= A // +{2,3}% B\r\n @xs; } // 終端",
        "grammar G { R ::= [ ] | { } | ( ); E ::= ; }",
        "grammar G { R ::= ' // \\' ' // 😀\n ; }",
        "grammar G { R ::= a // コメント\n . B {2,} % ','; }",
    ] { same(source); }
}

const FILES: &[&str] = &[
    "story/code/calc-var.ubnf",
    "story/code/calc.ubnf",
    "story/code/calc-leftrec.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/boundaries.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/elements.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/eval-default.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/keyword-boundary.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/namespace.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/common-field-space.ubnf",
    "unlaxer-dsl/src/test/resources/right-associative/Power.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Delimited.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Mixed.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Shared.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/0/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/1/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/2/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/3/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/associative/Operators.ubnf",
    "unlaxer-dsl/src/test/resources/cardinality/Cardinality.ubnf",
    "unlaxer-dsl/grammar/ubnf.ubnf",
    "unlaxer-dsl/tinycalc-vscode/grammar/tinycalc.ubnf",
    "unlaxer-dsl/examples/tinycalc.ubnf",
];

const P4: &str = "tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf";

const KNOWN_REJECTIONS: &[(&str, usize)] = &[
    (
        "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
        43,
    ),
    (
        "rust/unlaxer-ubnf/tests/fixtures/known/eval-default.ubnf",
        32,
    ),
    (
        "rust/unlaxer-ubnf/tests/fixtures/known/keyword-boundary.ubnf",
        24,
    ),
    ("rust/unlaxer-ubnf/tests/fixtures/known/namespace.ubnf", 28),
    ("unlaxer-dsl/examples/tinycalc.ubnf", 276),
    (P4, 2595),
];

#[test]
fn fixture_acceptance_and_ast_parity() {
    let unlaxer = root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser");
    let tinyexpression = root("TINYEXPRESSION_DIR", "../../../tinyexpression");
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    for (root, file) in FILES
        .iter()
        .map(|file| (&unlaxer, *file))
        .chain([(&tinyexpression, P4)])
    {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        let manual = syntax::parse_file(&source)
            .unwrap_or_else(|error| panic!("手書き frontend が {file} を拒否しました: {error}"));
        if let Some((_, expected)) = KNOWN_REJECTIONS.iter().find(|(path, _)| *path == file) {
            let message = format!("既知差分 {file} が解消したため一覧の更新が必要です");
            let error = ubnfc_selfhost::parse_file(&source).expect_err(&message);
            let span = error.span().expect("構文差分には位置が必要です");
            assert_eq!(
                span.start, *expected,
                "既知差分の位置が変わりました: {file}"
            );
            rejected.push(file);
        } else {
            let actual = ubnfc_selfhost::parse_file(&source)
                .unwrap_or_else(|error| panic!("未記録の拒否: {file}: {error}"));
            assert_eq!(actual, manual, "AST が不一致: {file}");
            accepted.push(file);
        }
    }
    assert_eq!(accepted.len(), 19);
    assert_eq!(rejected.len(), KNOWN_REJECTIONS.len());
}

#[test]
fn syntax_error_positions() {
    for source in [
        "grammar G { R ::= 'x' }",
        "grammar G { R ::= ( 'x' ; }",
        "// 文法😀\n grammar G { R ::= 'unclosed",
    ] {
        let manual = syntax::parse_file(source).unwrap_err();
        let generated = ubnfc_selfhost::parse_file(source).unwrap_err();
        assert_eq!(generated.span(), Some(manual.span), "{source:?}");
    }
}

#[test]
fn three_validation_error_golden_sources_have_fixed_parse_outcomes() {
    let root = root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser");
    let cases = [
        (
            "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
            Some(43),
        ),
        (
            "rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf",
            None,
        ),
        (
            "rust/unlaxer-ubnf/tests/fixtures/known/namespace.ubnf",
            Some(28),
        ),
    ];
    for (file, rejected_at) in cases {
        let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../ir/fixtures/errors")
            .join(format!("{}.txt", file.replace('/', "__")));
        assert!(
            golden.is_file(),
            "error golden がありません: {}",
            golden.display()
        );
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        let manual = syntax::parse_file(&source).unwrap();
        match rejected_at {
            Some(offset) => assert_eq!(
                ubnfc_selfhost::parse_file(&source).unwrap_err().span(),
                Some(syntax::Span {
                    start: offset,
                    end: offset + 1,
                }),
                "{file}"
            ),
            None => assert_eq!(ubnfc_selfhost::parse_file(&source).unwrap(), manual),
        }
    }
}
