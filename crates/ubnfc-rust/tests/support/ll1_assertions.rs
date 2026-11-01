#![allow(dead_code, clippy::while_let_loop)]
mod nested;
mod json;
fn nested_equal(text: &str, max_depth: usize) {
    let opt = nested::ParseOptions { build_ast: false, max_depth, ..Default::default() };
    let mut on = nested::parse_with_options(text, opt);
    let mut off = nested::parse_with_options(text, nested::ParseOptions { ll1: false, ..opt });
    on.statistics = Default::default();
    off.statistics = Default::default();
    assert_eq!(on, off, "{text:?}, depth={max_depth}");
}
fn json_equal(text: &str) {
    for scan in [false, true] {
        for require_eof in [false, true] {
            for mode in 0..5 {
                let opt = json::ParseOptions {
                    build_ast: mode == 1, ast_tree: mode == 2, occurrences: mode == 3,
                    diagnostics: mode == 4, require_eof, scan, ..Default::default()
                };
                let mut on = json::parse_with_options(text, opt);
                let mut off = json::parse_with_options(text, json::ParseOptions { ll1: false, ..opt });
                on.statistics = Default::default(); off.statistics = Default::default();
                assert_eq!(on, off, "{text:?}, scan={scan}, mode={mode}, eof={require_eof}");
            }
        }
    }
}
fn main() {
    let on = nested::parse_with_options("((1))", nested::ParseOptions { build_ast: false, ..Default::default() });
    let off = nested::parse_with_options("((1))", nested::ParseOptions { build_ast: false, ll1: false, ..Default::default() });
    assert!(on.ok);
    assert!(on.statistics.rule_evaluations < off.statistics.rule_evaluations, "表の実行を確認");
    for n in 0..40 {
        let text = format!("{}1{}", "(".repeat(n), ")".repeat(n));
        for depth in [0, 1, 2, 4, 32, 100] { nested_equal(&text, depth); }
        nested_equal(&text[..text.len().saturating_sub(1)], 100);
    }
    for text in ["", "[]", "{}", "[ 1, 2 , 3 ]", "{\"あ😀\": [true, false, null, -12.34e+2]}",
        "\"\\u1234\\n\\\\\"", "[0,]", "[1. ]", "[1e+]", "{\"x\":}", "truex", "null_", "\"\u{1f}\"", "[] trailing"] {
        json_equal(text);
        for (end, _) in text.char_indices() { json_equal(&text[..end]); }
    }
    // 小さい記号集合の全文字列。選択・反復の局所失敗と巻戻しを集中的に通す。
    let alphabet = ['[', ']', '0', ',', ' '];
    for mut code in 0..625 {
        let mut text = String::new();
        for _ in 0..4 { text.push(alphabet[code % alphabet.len()]); code /= alphabet.len(); }
        json_equal(&text);
    }
}
