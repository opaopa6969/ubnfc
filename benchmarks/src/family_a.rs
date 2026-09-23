// Family A: tinyexpression P4 文法（124 rule）の Rust 実装 2 本を 1 プロセスで交互に測る。
//
// - `ubnfc-p4-rust` … ubnfc の Rust backend が P4 IR から生成した静的 parser
// - `tinyexpression-rs` … unlaxer の Rust codegen が同じ P4 文法から生成し、
//   `unlaxer-runtime` の Expr グラフを解釈する parser
//
// 認識 / AST の対応:
// - ubnfc: `build_ast=false` が認識、`true` が AST。
// - tinyexpression-rs: `parse_tree_detailed_with_options` は常に CST を作るので
//   「認識のみ」は存在しない。CST までを「認識(CST)」、`mapper::map` まで含めたものを AST とする。
include!("bench_util.rs");

use unlaxer_runtime::{Memoization, ParseOptions as RtOptions};

#[global_allocator]
static ALLOC: Counting = Counting;

fn ubnfc_options(build_ast: bool) -> ubnfc_p4_rust::ParseOptions {
    let mut options = ubnfc_p4_rust::ParseOptions {
        build_ast,
        ..Default::default()
    };
    options.limits.diagnostics = 1 << 30;
    options.limits.arena_entries = 1 << 30;
    options.limits.memo_entries = 1 << 30;
    options
}

fn main() {
    let dir = std::env::args().nth(1).expect("fixture dir");
    let names: Vec<String> = std::env::args().skip(2).collect();
    let safe = RtOptions::with_memoization(Memoization::SafeFailures);
    header();
    for name in &names {
        let path = format!("{dir}/{name}.tiny");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let bytes = text.len();
        let entry = if name.starts_with("comparison-heavy") {
            Some("BooleanExpression")
        } else {
            None
        };

        for build_ast in [false, true] {
            let options = ubnfc_options(build_ast);
            let mode = if build_ast { "AST" } else { "認識" };
            row("A", "ubnfc-rust(生成)", name, bytes, mode, || {
                let r = match entry {
                    None => ubnfc_p4_rust::parse_formula(std::hint::black_box(text.as_str()), options),
                    Some(e) => ubnfc_p4_rust::parse_entry(Some(e), std::hint::black_box(text.as_str()), options).unwrap(),
                };
                assert!(r.ok);
                std::hint::black_box(r);
            });
        }

        // tinyexpression-rs は Formula 入口だけを持つ（entry 指定の API がない）。
        use tinyexpression_rs::generated::{mapper, parser};
        if parser::parse_tree_detailed_with_options(&text, safe).is_ok() {
            row("A", "tinyexpression-rs(生成/Expr解釈)", name, bytes, "認識(CST)", || {
                let t = parser::parse_tree_detailed_with_options(std::hint::black_box(text.as_str()), safe)
                    .expect("fixture");
                std::hint::black_box(t);
            });
            row("A", "tinyexpression-rs(生成/Expr解釈)", name, bytes, "AST", || {
                let t = parser::parse_tree_detailed_with_options(std::hint::black_box(text.as_str()), safe)
                    .expect("fixture");
                let a = mapper::map(&t).expect("fixture");
                std::hint::black_box(a);
            });
            row("A", "tinyexpression-rs(公開 facade)", name, bytes, "AST+候補選択", || {
                let a = tinyexpression_rs::parse(std::hint::black_box(text.as_str())).expect("fixture");
                std::hint::black_box(a);
            });
        } else {
            eprintln!("# tinyexpression-rs は {name} を Formula 入口で受理しない");
        }
    }
}
