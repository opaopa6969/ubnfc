# P4 の独立 Rust 実行例

`examples/p4-java` の Rust 版。P4 IR（124 rules）から `ubnfc-rust` が生成した parser を無変更で commit し、
`scanners/rust/scanners.rs` の Extern scanner 3種（STRING / CODE_START / CODE_END）を登録する。
通常依存はゼロ（`json` feature のときだけ serde / serde_json）。生成器・tinyexpression・unlaxer は変更していない。

**性能・互換性の検証例であり、P4 の完全互換を達成したものではない。**
16 fixture 中、Formula 入口で 9 件を全消費、意図的な不正入力 3 件と comparison-heavy 系 4 件を拒否する
（Java 側と同じ）。BooleanExpression 入口なら comparison-heavy 系 4 件も全消費する。
詳細と計測は [計測報告](../../docs/reports/2026-09-22-p4-rust-bench.md)。

> **診断予算について（2026-09-23 訂正）**: 本 README は以前「既定の診断予算（`limits.diagnostics`
> = 200 万 node）では complex-x4 以上・large-match・comparison-heavy-x4 以上が
> `diagnostic budget exceeded` で失敗する」と書いていたが、**現在は当てはまらない**。
> 失敗記録が O(1) 件になり、さらに既定が fast mode（`ParseOptions::diagnostics = false`）に
> なって予算を消費しなくなったため、既定 2,000,000 で全 fixture が通る（D-023、
> [`2026-09-23-p4-rust-diag-two-mode.md`](../../docs/reports/2026-09-23-p4-rust-diag-two-mode.md)）。
> 契約は `tests/fixtures.rs` の
> `default_diagnostic_budget_accepts_every_fixture_and_small_budget_fails_as_resource` が固定する。

## 再現

```bash
cargo test -p ubnfc-p4-rust --offline
cargo clippy -p ubnfc-p4-rust --offline --all-targets -- -D warnings
cargo run -q --offline --release -p ubnfc-p4-rust --example bench
cargo run -q --offline --release -p ubnfc-p4-rust --features counters --example bench -- --counters complex comparison-heavy
bash scripts/generate.sh --check   # または --write
```

- `src/generated/` は `ubnfc-rust --no-emit-driver` の出力（Cargo.toml を除く）。`tests/regenerate.rs` が
  同じ IR・同じ emitter からの byte 一致を検証する。rustfmt の対象外。
- `src/lib.rs` の `parse_formula` は commit 済み生成物の `parse_with_scanner`、`parse_entry` は
  `build.rs` が複製 parser に加えた `parse_entry_with_scanner`（driver と同じ置換）で任意 rule を入口にする。
- `counters` feature では build.rs が複製 parser / runtime / rt/diag に計数フックを挿入する
  （rule 関数の呼出しと包含・自己時間、expression 関数の呼出し、memo lookup/store、arena event、
  診断 DAG の node 追加、display frame の走査回数）。計測値は複製経路（`parse_entry`）だけに付く。
- `tests/fixtures/` は tinyexpression の `benchmarks/fixtures/*.tiny` と
  `rust/tinyexpression-rs/tests/fixtures/*.tiny` を byte を変えずに収録した（16 件）。
- `tests/golden/` は生成 Rust の canonical AST。Java golden（`../p4-java`）とは文字列値の単引用符だけが異なる
  （Rust の scanner 集合は単引用符を value から除く。`tests/fixtures.rs` で個数まで固定）。
- `examples/bench.rs` は std のみ。warmup 後 5 run の中央値、確保回数は GlobalAlloc の別 parse で数える。
- `benchmarks/` に計測結果を保存する。
