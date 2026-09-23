//! ubnfc-front: UBNF の字句・構文解析、検証、解析、Grammar IR 出力。
//!
//! モジュール構成（設計 v0 §2）:
//! - `syntax`   : UBNF ソース → `GrammarAst`（手書き lexer + 再帰下降）
//! - `resolve`  : @import / 名前空間 / token クラス解決
//! - `validate` : 既存 E-*/W-* コード互換の検証
//! - `ir`       : Grammar IR（serde）
//! - `analysis` : nullable / FIRST / 左再帰 / 優先順位 / choice 戦略 / capture cardinality
//! - `cli`      : `ubnfc` コマンド
//! - `pipeline` : ソース読込から参照検証済み Grammar IR までの結線
//! - `vocabulary`: UBNF v2 の言語中立な組込み token 語彙（D-040）

pub mod analysis;
pub mod boundary;
pub mod cli;
pub mod desugar;
pub mod ir;
pub mod lower;
pub mod pipeline;
pub mod resolve;
pub mod selfhost;
pub mod syntax;
pub mod validate;
pub mod vocabulary;

pub const IR_VERSION: u32 = 1;
