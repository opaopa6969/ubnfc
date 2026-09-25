//! ubnfc-fuzz: Java / Rust の 2 バックエンドに対する差分 fuzzing。
//!
//! 両バックエンドは front（`crates/ubnfc-front`）を共有するが、生成器と runtime は
//! 完全に独立している。同じ (文法, 入力) に対する観測差は、少なくとも片側のバグか、
//! 契約の不足である。corpus（旧実装の観測の固定）では捕まらない種類の欠陥を探す。
//!
//! 比較を弱める箇所は corpus と同じ契約に限る（`observe` の冒頭に根拠を書いた）。

pub mod campaign;
pub mod engine;
pub mod gen;
pub mod input;
pub mod load;
pub mod model;
pub mod observe;
pub mod rng;

/// 生成 crate の `cargo build --release` を最適化なしにして campaign を回せる速さにする。
///
/// harness は生成物を必ず `--release` で build する（`docs/harness-driver-protocol.md`）。
/// fuzzing では 1 文法あたり 1 回の build が支配的なので、プロファイルの最適化レベルだけを
/// 環境変数で下げる。`debug-assertions` / `overflow-checks` は release の既定（いずれも off）
/// のままなので、観測される振る舞いは変わらない。呼出側が明示した値は上書きしない。
///
/// スレッドを起こす前に一度だけ呼ぶこと（`std::env::set_var` の制約）。
pub fn tune_generated_builds() {
    for (key, value) in [
        ("CARGO_PROFILE_RELEASE_OPT_LEVEL", "0"),
        ("CARGO_PROFILE_RELEASE_CODEGEN_UNITS", "16"),
        ("CARGO_PROFILE_RELEASE_DEBUG", "false"),
    ] {
        if std::env::var_os(key).is_none() {
            std::env::set_var(key, value);
        }
    }
}
