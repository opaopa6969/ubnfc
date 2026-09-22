//! tinyexpression P4 IR から生成した Rust parser の独立実行例。
//!
//! `generated` は `ubnfc-rust` の出力を無変更で commit したもの（tests/regenerate.rs が byte 一致を検証）。
//! Extern token 3種（STRING / CODE_START / CODE_END）は `scanners/rust/scanners.rs` の移植集合を
//! そのまま登録する。`parse_entry` は build.rs が複製した parser に加えた
//! `parse_entry_with_scanner`（driver と同じ置換）で任意の rule を入口にする。

// scanners/rust/scanners.rs は生成 crate 名 `ubnfc_generated` を参照する。
extern crate self as ubnfc_generated;

#[rustfmt::skip]
pub mod generated;
pub use generated::*;

#[cfg(feature = "counters")]
pub mod counters;

#[allow(dead_code)]
#[path = "../../../scanners/rust/scanners.rs"]
pub mod scanners;

#[allow(dead_code, unused_imports, clippy::all)]
mod patched {
    include!(concat!(env!("OUT_DIR"), "/patched/mod.rs"));
}

/// 文法名。`parse_entry` の grammar 引数に使う。
pub const GRAMMAR: &str = "TinyExpressionP4";

/// IR の Extern token（logicalId, originalClassName）。driver の EXTERN_CLASSES と同じ組。
pub const EXTERN_CLASSES: &[(&str, &str)] = &[
    (
        "TinyExpressionP4::STRING",
        "org.unlaxer.tinyexpression.parser.StringLiteralParser",
    ),
    (
        "TinyExpressionP4::CODE_START",
        "org.unlaxer.tinyexpression.parser.javalang.CodeStartParser",
    ),
    (
        "TinyExpressionP4::CODE_END",
        "org.unlaxer.tinyexpression.parser.javalang.CodeEndParser",
    ),
];

/// 3 Extern token を束ねた scanner registry。状態を持たないので毎回作ってよい。
pub fn registry() -> scanners::Registry {
    scanners::Registry::new(EXTERN_CLASSES).expect("P4 の Extern クラスは全て移植済み")
}

/// root rule（Formula）を入口に、commit 済み生成物で解析する。
pub fn parse_formula(text: &str, options: ParseOptions) -> ParseResult {
    let mut scanner = registry();
    generated::parse_with_scanner(text, options, &mut scanner)
}

/// 任意の rule を入口に解析する（`None` は root）。build.rs の複製 parser を使う。
/// `counters` feature ではこの経路だけが計測される。
pub fn parse_entry(
    entry: Option<&str>,
    text: &str,
    options: ParseOptions,
) -> Result<ParseResult, &'static str> {
    let mut scanner = registry();
    patched::parse_entry_with_scanner(GRAMMAR, entry, text, options, &mut scanner)
}
