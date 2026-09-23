//! 仕様由来ケース（`corpus/spec/p4/*.json`）を P4 生成 Rust parser へ通す受理プローブ。
//!
//! 標準入力から `id<TAB>source` の TSV を読み（`\n` `\t` は 2 文字表記）、
//! 1 行につき `id<TAB>ok|reject<TAB>consumed_cp` を出す。AST は出さない。
//!
//! ```sh
//! cargo run -q --offline --release -p ubnfc-p4-rust --example spec-probe < cases.tsv
//! ```
use ubnfc_p4_rust::{parse_formula, ParseOptions};

fn main() {
    let mut text = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut text).unwrap();
    for line in text.lines() {
        let (id, source) = line.split_once('\t').unwrap_or((line, ""));
        let source = source.replace("\\t", "\t").replace("\\n", "\n");
        let options = ParseOptions {
            build_ast: false,
            require_eof: true,
            ..ParseOptions::default()
        };
        let result = parse_formula(&source, options);
        let verdict = if result.ok { "ok" } else { "reject" };
        println!("{id}\t{verdict}\t{}", result.consumed_cp);
    }
}
