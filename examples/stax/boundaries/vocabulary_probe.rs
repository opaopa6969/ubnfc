use boundary::{ParseOptions, ScanEffect, ScanResult, State, TokenScanner};
use std::{env, fs};

// 内部可変な辞書は使わず、呼出し前に固定した snapshot の名前だけを返す。
struct Snapshot(String);
impl TokenScanner for Snapshot {
    fn scan(
        &mut self,
        id: &str,
        _: &str,
        state: State,
        match_only: bool,
        _: &boundary::rt::scope::ScopeStore,
    ) -> ScanResult {
        assert_eq!(id, "Vocabulary::SEED");
        let p = if match_only {
            state.matched
        } else {
            state.consumed
        };
        ScanResult {
            ok: true,
            consumed_end: state.consumed,
            matched_end: state.matched,
            value_span: [p, p],
            diagnostics: vec![],
            effects: vec![ScanEffect::Declare {
                name: self.0.clone(),
                offset_cp: 0,
            }],
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let source = fs::read_to_string(&args[1]).unwrap();
    let result = boundary::parse_with_scanner(
        &source,
        ParseOptions {
            build_ast: false,
            memo: args[3] == "true",
            lexical_cache: args[4] == "true",
            diagnostics: true,
            ..ParseOptions::default()
        },
        &mut Snapshot(args[2].clone()),
    );
    // この再現の語彙名は external / other の ASCII だけ。Debug 表示も JSON の文字列になる。
    let names: Vec<&str> = result
        .declarations
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    println!(
        "{{\"ok\":{},\"declarations\":{:?},\"warnings\":{},\"scopeDepth\":{}}}",
        result.ok,
        names,
        result.scope_diagnostics.len(),
        result.scope.depth
    );
}
