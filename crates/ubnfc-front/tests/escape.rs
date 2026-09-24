//! v2 の code point エスケープ（D-062）。
//!
//! 一番大事な不変量は「`\u` を含まない原文では、v2 の展開が v1 の字句値と 1 文字も
//! 違わない」こと。これが成り立つ限り、v2 を宣言しただけの文法は v1 と同じ IR になる。

use ubnfc_front::escape::{expand, has_unicode_escape, EscapeErrorKind};
use ubnfc_front::syntax::ast::TokenValue;
use ubnfc_front::syntax::parse_file;

/// `NEGATION('…')` を 1 つ書いた文法を通し、v1 の字句値と原文（引用符の内側）を返す。
fn literal(inside_quotes: &str) -> (String, String) {
    let source = format!("grammar G {{ token T = NEGATION('{inside_quotes}') @root R ::= T; }}");
    let file = parse_file(&source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
    match &file.grammars[0].tokens[0].value {
        TokenValue::Negation {
            chars, chars_raw, ..
        } => (chars.clone(), chars_raw.clone()),
        other => panic!("NEGATION ではない: {other:?}"),
    }
}

/// `\u` が現れない原文では、v2 の展開 == v1 の字句値。
#[test]
fn expansion_agrees_with_the_v1_lexer_when_no_unicode_escape_is_present() {
    for raw in [
        "",
        "abc",
        "ab\\n",
        "\\t",
        "\\r",
        "\\\\",
        "\\'",
        "a\\qb",
        "😀あ",
        "\\\\u0041",
        "x\\\\",
    ] {
        let (value, inner) = literal(raw);
        assert!(!has_unicode_escape(&inner), "{raw}");
        assert_eq!(expand(&inner).unwrap(), value, "{raw}");
    }
}

/// `\uXXXX` と `\u{X…}` は code point 1 個になる。
#[test]
fn unicode_escapes_expand_to_one_code_point() {
    for (raw, expected) in [
        ("\\u0041", "A"),
        ("\\u0000", "\u{0}"),
        ("\\u001F", "\u{1f}"),
        ("\\u{41}", "A"),
        ("\\u{1F600}", "😀"),
        ("\\u{10FFFF}", "\u{10ffff}"),
        ("a\\u0062c", "abc"),
        ("\\u0041\\u{42}", "AB"),
    ] {
        let (_, inner) = literal(raw);
        assert!(has_unicode_escape(&inner), "{raw}");
        assert_eq!(expand(&inner).unwrap(), expected, "{raw}");
    }
}

/// `\\` の直後の `u` はエスケープではない（原文から展開する理由そのもの）。
#[test]
fn a_backslash_escape_before_u_is_not_a_unicode_escape() {
    let (value, inner) = literal("\\\\u0041");
    assert_eq!(value, "\\u0041");
    assert!(!has_unicode_escape(&inner));
    assert_eq!(expand(&inner).unwrap(), "\\u0041");
    // 一方、エスケープとして書いたほうは 1 文字になる。
    let (_, escaped) = literal("\\u0041");
    assert!(has_unicode_escape(&escaped));
    assert_eq!(expand(&escaped).unwrap(), "A");
}

/// 綴りの誤り・surrogate・範囲外は展開に失敗する。
#[test]
fn malformed_surrogate_and_out_of_range_escapes_fail() {
    for (raw, kind) in [
        ("\\u00", EscapeErrorKind::Malformed),
        ("\\u00G1", EscapeErrorKind::Malformed),
        ("\\u{}", EscapeErrorKind::Malformed),
        ("\\u{110000}", EscapeErrorKind::Malformed),
        ("\\u{1234567}", EscapeErrorKind::Malformed),
        ("\\u{41", EscapeErrorKind::Malformed),
        ("\\ud800", EscapeErrorKind::Surrogate),
        ("\\u{DFFF}", EscapeErrorKind::Surrogate),
    ] {
        let (_, inner) = literal(raw);
        let error = expand(&inner).unwrap_err();
        assert_eq!(error.code, kind, "{raw}");
        assert!(has_unicode_escape(&inner), "{raw}");
    }
}
