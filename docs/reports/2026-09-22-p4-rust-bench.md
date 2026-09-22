# P4 Rust 生成・互換性・性能の検証（2026-09-22）

Java 側 [`2026-09-22-p4-java-bench.md`](2026-09-22-p4-java-bench.md) の Rust 版。
`examples/p4-rust`（crate `ubnfc-p4-rust`）に生成 P4 parser、Extern scanner 3種の登録、
fixture 16件、canonical AST golden、ベンチ、計数プロファイルを収録した。
**emitter（`crates/ubnfc-rust`）は変更していない。** 本報告は計測と診断であり、改善は Rust M5 に送る。

## 結論

- P4 IR（124 rules、1,043 expressions）から Rust の生成・build・実行が可能だった。生成拒否はない。
  生成物 13 ファイル 1.73 MB（`parser.rs` 1.24 MB）。release build は 2 複製で約 54 秒。
- fixture の受理は Java と同じ（Formula 入口 9 件受理、comparison-heavy 系 4 件と不正入力 3 件を拒否、
  BooleanExpression 入口で comparison-heavy 系 4 件を全消費）。**ただし既定の診断予算
  （`limits.diagnostics` = 2,000,000 node）では complex-x4 以上・large-match・comparison-heavy-x4 以上が
  `diagnostic budget exceeded` で全体失敗する。** 受理検証とベンチは予算を広げて実行した。
- canonical AST は Java golden と、文字列値の単引用符（Rust の scanner 集合は `'jp'` → `jp`、Java は字句のまま）
  以外は完全一致。数値葉は両者とも字句 String で、差はなかった。
- **性能は Java 生成物より遅い。** complex.tiny（332 byte）は認識のみ 24.9 ms、AST 付き 26.3 ms
  （Java 17.3 ms）。75〜150 µs/byte で、手書き目標（512 atom ≈ 0.4 ms ≈ 0.4 µs/byte）の約 200 倍。
  1 parse で **1.7 億 byte** を確保し、bench 全体の sys 時間（2 分 25 秒）が user 時間（57 秒）を上回った。
- 原因は計数で特定できた。complex 1 parse で rule 呼出し 2,455・expression 呼出し 7,205 に対し、
  **診断 DAG の node 追加 1,081,968・join 1,499,934・display frame の走査 4,327,682**。
  失敗 1 件ごとに全 active frame（最大 133）へ join する `DisplayDiagnostics::merge` と、
  expression ごとの frame push が支配的で、memo・choice の再走査はその次である。

## 使用物と再現

- 作業ブランチ `feat/p4-rust-e2e`（worktree）。入力は
  `ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json`。
- 生成: `ubnfc-rust --no-emit-driver`（同 checkout の `crates/ubnfc-rust`）。生成物は `src/generated/` に無変更で commit し、
  `tests/regenerate.rs` が同じ IR・同じ emitter からの byte 一致を検証する（Cargo.toml は比較対象外）。
- Extern scanner: `scanners/rust/scanners.rs` を `#[path]` で取り込み、IR の `(logicalId, originalClassName)` 3 組で
  `Registry::new` する（driver の `EXTERN_CLASSES` と同じ）。scanner の単体テスト（`scanners_tests.rs`）も同 crate で走る。
- 任意 rule 入口: 生成 `parse_entry_with_options` は `RejectExtern` 固定なので、`build.rs` が driver と同じ文字列置換で
  `parse_entry_with_scanner` を複製 parser に加える。`parse_entry(Some("BooleanExpression"), …)` はこの経路。
  複製と commit 済み生成物が同じ結果を返すことは `entry_selection_matches_root_parser` で確認した。
- rustc 1.98.1、AMD Ryzen 9 7950X、WSL2。共用ホストで CPU 固定なし。

```bash
cargo test -p ubnfc-p4-rust --offline                       # 12 tests（regenerate 1、fixture 9、scanner 2）
cargo clippy -p ubnfc-p4-rust --offline --all-targets -- -D warnings
cargo run -q --offline --release -p ubnfc-p4-rust --example bench
cargo run -q --offline --release -p ubnfc-p4-rust --features counters --example bench -- --counters complex complex-x4 comparison-heavy
```

ワークスペースの `[profile.dev.package.ubnfc-p4-rust] opt-level = 3` を追加した。debug のままだと x64 fixture の
1 parse に数十秒かかり、テストが数分になるため。

## fixture と canonical AST

| fixture | Formula 入口（予算拡大） | 既定予算 | code point |
|---|---|---|---:|
| complex | 受理 | 受理 | 325 |
| complex-x4 / x16 / x64 | 受理 | **diagnostic budget exceeded** | 1,293 / 5,179 / 20,923 |
| flat-arithmetic | 受理 | 受理 | 1,426 |
| large-match | 受理 | **diagnostic budget exceeded** | 1,380 |
| valid-basic / valid-multiline / valid-unicode | 受理 | 受理 | 6 / 40 / 9 |
| comparison-heavy 系 4 件 | 拒否（主診断 offset 5、Java と同じ） | 拒否 | — |
| complex-half / complex-tail / invalid-syntax | 拒否 | 拒否 | — |
| comparison-heavy 系（BooleanExpression 入口） | 受理 | x1 のみ受理、x4 以上は予算超過 | 179 / 716 / 2,864 / 11,456 |

memo 無効では complex でも既定予算に達する（rule 評価 168,262 回、予算拡大で 4.0 秒）。
以上は `tests/fixtures.rs` の `default_diagnostic_budget_is_exceeded` で固定した。
認識のみ（`build_ast=false`）は同じ受理・消費で AST と capture を省略し、memo 無効（小さい fixture）でも同じ AST。

Java golden との構造比較（`serde_json::Value`、key 順は無視）:

| fixture | 生の差分 | 内容 |
|---|---:|---|
| complex / x4 / x16 / x64 | 3 / 12 / 48 / 192 | すべて単引用符付き文字列値（`'jp'` → `jp`） |
| valid-unicode | 1 | 同上（`'こんにちは😀'`） |
| flat-arithmetic / large-match / valid-basic / valid-multiline | 0 | 完全一致 |
| comparison-heavy 系 4 件（boolean-entry） | 0 | 完全一致 |

単引用符の除去は `scanners/rust/scanners.rs` の `quoted`（QuotedParser 由来、single だけ value から除く）による。
二重引用符は残る（`"jp"` → `"jp"`）。旧 Mapper の `stripQuotes` 結果とは偶然一致するが、Java 生成物の
「字句のまま」契約とは非対称なので、Java 報告の提案 2（span と値変換の分離）を Rust 側でも同じ論点として引き継ぐ。
数値葉 `BinaryExpr(left=null, op=["10"], right=[])` は Java golden も文字列で、期待していた lexeme/int の差は出なかった。
span は両者とも code point。

## ベンチマーク

`examples/bench.rs`。std のみ、`Instant`、release、warmup 後 5 run の中央値。1 run の反復数は 1 parse の所要から
0.3 秒を満たすよう決めた（3〜5000）。確保回数・byte は GlobalAlloc の alloc/realloc を数えた別 parse（realloc 後のサイズも合算、
peak RSS ではない）。予算は `diagnostics / arena_entries / memo_entries = 2^30`（事前確保ではないので計時に影響しない）。
全列は [`benchmarks/timing.md`](../../examples/p4-rust/benchmarks/timing.md)。

| 入力 | bytes | 入口 | 認識 ms/parse | AST ms/parse | µs/byte | allocs/parse 認識 / AST | alloc bytes/parse | rule_evaluations | memo_hits / entries |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| complex | 332 | Formula | 24.95 | 26.33 | 75 / 79 | 1,301 / 3,793 | 170 MB | 2,455 | 816 / 6,292 |
| complex-x4 | 1,293 | Formula | 108.07 | 107.00 | 84 / 83 | 4,867 / 14,573 | 679 MB | 9,418 | 3,141 / 24,118 |
| complex-x16 | 5,179 | Formula | 479.72 | 441.62 | 93 / 85 | 19,086 / 57,592 | 2.72 GB | 37,270 | 12,441 / 95,422 |
| complex-x64 | 20,923 | Formula | 1,852.98 | 1,905.86 | 89 / 91 | 75,918 / 229,522 | 10.9 GB | 148,678 | 49,641 / 380,638 |
| flat-arithmetic | 1,426 | Formula | 34.59 | 36.27 | 24 / 25 | 119 / 12,302 | 175 MB | 7,188 | 0 / 21,062 |
| large-match | 1,380 | Formula | 107.78 | 114.69 | 78 / 83 | 8,808 / 28,373 | 686 MB | 12,811 | 2,539 / 36,323 |
| comparison-heavy | 179 | BooleanExpression | 22.10 | 23.57 | 123 / 132 | 134 / 4,399 | 87 MB | 3,015 | 624 / 7,887 |
| comparison-heavy-x4 | 716 | BooleanExpression | 89.32 | 90.34 | 125 / 126 | 235 / 17,054 | 347 MB | 12,054 | 2,496 / 31,527 |
| comparison-heavy-x16 | 2,864 | BooleanExpression | 387.19 | 396.91 | 135 / 139 | 624 / 67,617 | 1.39 GB | 48,210 | 9,984 / 126,087 |
| comparison-heavy-x64 | 11,456 | BooleanExpression | 1,671.98 | 1,716.20 | 146 / 150 | 2,170 / 269,781 | 5.5 GB | 192,834 | 39,936 / 504,327 |

- スケーリングは入力量にほぼ線形（complex 系 75→84→93→89 µs/byte、comparison-heavy 系 123→125→135→146 µs/byte）。
  超線形ではないが、定数が手書き目標の 200 倍前後。
- 認識のみと AST 付きの差は 1〜7 %。AST 構築は問題ではない。確保回数の差（complex 1,301 → 3,793）は AST の Vec/String。
- 確保 byte はほぼ全部が診断 arena の Vec 倍々拡張（下記）。complex-x64 では 1 parse あたり 10.9 GB を要求し、
  bench の sys 時間が user 時間の 2.5 倍になった（page fault / 巨大 realloc）。
- 参考値: Java 生成 P4 complex 17.3 ms（AST、同日報告）、旧 Rust complex 1.426 ms（criterion、別 revision）、
  手書き目標 512 atom ≈ 0.4 ms。Rust 生成 P4 はこのどれよりも遅い。
- 反復数が 3 の行（x16 以上）は run 間の分散が大きく、順位の議論はしない。

## プロファイル（計数）

`perf` / `valgrind` / `gdb` はこの環境に無く sudo も使えないため、`counters` feature で複製 parser / runtime / rt/diag に
計数フックを挿入した（`build.rs`）。rule 関数は `Instant` で包含・自己時間、expression 関数と runtime 呼出しは回数。
計測込みの所要は非計測の 1.1 倍程度（complex 27.8 ms 対 24.9 ms）。全表は
[`benchmarks/counters.md`](../../examples/p4-rust/benchmarks/counters.md)。

| 1 parse あたり | complex（332 B） | complex-x4（1,293 B） | comparison-heavy（179 B、Boolean 入口） |
|---|---:|---:|---:|
| rule 関数呼出し（`rN_c`） | 2,455 | 9,418 | 3,015 |
| expression 関数呼出し（`eN_c`） | 7,205 | 27,644 | 8,700 |
| `limit()`（stack 検査） | 9,660 | 37,062 | 11,715 |
| `mark()` / `restore()` | 10,190 / 10,364 | 39,083 / 39,725 | 12,287 / 13,473 |
| memo `lookup` / hit / `store` | 7,108 / 816 / 6,292 | 27,259 / 3,141 / 24,118 | 8,511 / 624 / 7,887 |
| arena `event` | 1,462 | 5,640 | 1,190 |
| 失敗（`fail` + `display.failures`） | 1,482 + 7,535 | 5,697 + 28,865 | 1,968 + 10,103 |
| `display.enter/leave`（frame push） | 11,175 | 42,864 | 13,870 |
| **診断 DAG node 追加（`push`）** | **1,081,968** | **4,313,145** | **948,651** |
| **診断 `join` 呼出し** | **1,499,934** | **5,981,307** | **938,349** |
| **display frame 走査回数** | **4,327,682** | **17,272,034** | **2,652,068** |
| display frame 最大深さ | 133 | 135 | 52 |

読み方:

1. 失敗 1 件あたり約 145 node・200 join・480 frame 走査。`DisplayDiagnostics::merge`（`rt/diag.rs:186-192`）が
   失敗のたびに **全 active frame** へ `arena.join` し、`Diagnostics::join`（`rt/diag.rs:39-55`）は位置が等しいと
   新しい `Join` node を push する。frame は expression 関数ごとに push される（`display.enter`、`parser.rs` の各
   `eN_c`、例 `e369_c` 8474 行）ので深さが 133 に達し、失敗数 × 深さの node/join が生まれる。
   `reach`（`rt/diag.rs:171-176`）も enter/reach ごとに全 frame を走査する。
2. node は `(usize, Node)` 32 byte。complex で 1.08M node ≈ 35 MB、Vec の倍々拡張の合計が確保 170 MB の大半。
   join は `nodes[a].0` / `nodes[b].0` を読むため 35 MB の配列へのランダム参照になり、1.5M 回で約 20 ms、
   これが complex の 25 ms のほぼ全部を説明する。既定予算 2M node に 700 byte で達するのも同じ理由。
3. rule 別自己時間の上位は SliceBaseExpression（189 回、10 %）、SliceBaseReceiver（168 回）、SliceNestedExpression、
   VariableRef、IfExpression。これは失敗の多い choice（文字列メソッド候補を毎 primary で試す）に diag コストが
   付いた結果で、rule 自体の処理ではない。
4. memo は expression 単位で 1,039 箇所（`parser.rs` の `Key{expression:N, …, version:self.scope.state_version()}`）。
   complex の hit 率は 816 / 7,108 = 11 %。key に scope の `state_version` が入るため、宣言（`effect_at` 155 回）の
   たびに既存 entry が使えなくなる。entry は 332 byte で 6,292（19 / byte）。
5. `limit()`（`runtime.rs:184`、`#[inline(never)]`、stack address 検査）は expression ごとに呼ばれ 9,660 回。
   `mark/restore` も expression ごと。rule 単位で十分。
6. choice の再走査: NumberExpression は complex で 38 回、NumberTerm 30 回、SliceBaseExpression 189 回、
   ExternalNumberInvocation（comparison-heavy）63 回。`Expression ::= NumberExpression | StringExpression | …`
   の順序選択と、rule 参照で始まる alternative には予測分岐が付かない（README: 非空 literal の FIRST だけ）。
   ただし rule 評価は 7.4 / byte で、これ自体は 1 ms 級。診断コストを取り除いた後の次の壁。

## Rust M5 への優先順位付き提案（emitter 変更、本作業では未実施）

1. **診断 DAG の O(depth) merge を止める**（`templates/diag.rs` → 生成 `rt/diag.rs:153-192`）。
   失敗は最上位 frame にだけ merge し、`leave` 時に親へ 1 回だけ渡す。`reach` も同様に最上位のみ更新し、
   leave で max を伝える。これで node/join は失敗数のオーダー（complex で ~9,000）になり、確保 170 MB と
   既定予算超過が消える。期待効果は complex で 20 ms 以上（8 割超）。
2. **frame を rule 単位にする**（`expressions.rs` が各 `eN_c` に出す `display.enter/reach/leave`、`parser.rs` 例 8474 行）。
   memo が保存する snapshot は式単位のままでよいなら、`store` 時に現在 frame の snapshot を取るだけで足りる。
   深さ 133 → 30 程度。
3. **`Diagnostics::join` の node 生成を抑える**（`rt/diag.rs:39-55`）。同位置の join は label 集合の和なので、
   `(a,b)` の順序対で dedupe するか、位置ごとの label bitset へ置き換える。1 と併用で最終出力の `summary` も軽くなる。
4. **memo の範囲と key**（`expressions.rs` の memo 出力、`parser.rs` の `Key{…version…}` 1,039 箇所、`runtime.rs:341-366`）。
   scope 効果に依存しない expression（analysis の effects が空）は `version` を key から外す。
   memo を rule と choice の alternative 先頭に限定し、Seq の内部 expression は memo しない（hit 率 11 %、entry 19 / byte）。
   `Saved` に display snapshot と effects span を持つのも 1・2 の後に見直す。
5. **`limit()` と `mark/restore` を rule 単位に**（`runtime.rs:184-211`、各 `eN_c` の先頭）。stack 検査は rule 入口だけで
   256 KiB 予算の趣旨を満たす。expression 単位の `mark`（scope checkpoint + effects len）は Seq の失敗 rollback に必要な
   箇所だけに絞る。
6. **rule 参照で始まる choice の予測分岐**（`expressions.rs` の Choice plan）。front の `mayStart` / FIRST を rule 参照越しに
   使い、`Expression` の 4 alternative や SliceBaseExpression の文字列メソッド候補を先頭 code point で絞る。
   comparison-heavy が Formula で拒否される順序問題（Java 報告の提案 1）とは別で、受理を変えない範囲の最適化。
7. **既定 `limits.diagnostics` の意味を固定する**（`templates/api.rs` の `ResourceLimits`）。1〜3 の後は 2M で十分になるはずだが、
   予算超過が「認識は正しいのに全体失敗」になる現状を、少なくとも fixture で回帰テストにする。
8. AST 経路の Vec/String 一時確保（`runtime.rs` の `field_nodes` / `build_values_inner`）は認識比 +5 % 程度で後回し。

## 検証記録

- `cargo test -p ubnfc-p4-rust --offline`: 12 tests / 0 failures（regenerate 1、fixtures 9、scanner unit 2）。
- `cargo clippy -p ubnfc-p4-rust --offline --all-targets -- -D warnings`: 成功（`--features counters` も成功）。
- `cargo fmt -p ubnfc-p4-rust -- --check`: 成功（`src/generated` は rustfmt 対象外）。
- push / PR / merge は依頼に従い実行しない。`crates/ubnfc-rust` と unlaxer-parser / tinyexpression は無変更。
