# 高負荷下の driver タイムアウトを負荷率で伸ばし、未観測として扱う（issue #73）

> 2026-09-24。対象は `crates/ubnfc-fuzz`（driver 交換とタイムアウト）と
> `docs/harness-driver-protocol.md`（契約の追記）。`crates/ubnfc-harness` のコードは変更していない
> （driver 起動と壁時計タイムアウトは `ubnfc-fuzz` 側にしかないことを確認した。§1）。

## 0. 結論

- 壁時計タイムアウトを掛けているのは `crates/ubnfc-fuzz/src/engine.rs` の `exchange` /
  `wait_or_kill` だけ。`crates/ubnfc-harness/src/runner/**` には timeout も Duration も無い
  （grep で確認済み。build には元々タイムアウトが無い）。
- `crates/ubnfc-fuzz/src/load.rs` を新設し、`/proc/loadavg` の 1 分平均 ÷ オンライン CPU 数を
  倍率にして基準タイムアウトへ掛ける。倍率は `[1.0, 4.0]` にクランプし、無負荷（1 CPU あたり
  1 未満）では 1 倍のまま（既定動作を変えない）。
- `driver_timeout` になった応答には機械可読な印を付け（`is_driver_timeout`）、
  `campaign::run_pair_with_retries` が該当 backend だけを最大 2 回（`Config::timeout_retries`
  既定値）再試行する。使い切ってもなお timeout の比較は `Summary.unobserved_timeouts` に数え、
  `driver_errors` にも差分（`findings`）にも入れない。
- 再現実験（§1）は 2 つの有力候補バイナリを人為的な CPU 高負荷（1 CPU あたり負荷 1.5〜2 倍を
  数分間）で走らせたが、失敗を再現できなかった。したがって本対応は issue #64 / #73 の報告
  （`docs/reports/2026-09-23-differential-fuzzing.md` の `driver_timeout` discard、
  2026-09-24 master 2217625 での 1 バイナリ 23 秒 `0 passed; 1 failed`）を一次証拠として設計し、
  再現できなかったことを正直に記録する。

## 1. どのバイナリが 23 秒で落ちたか（再現の試み）

`cargo test --workspace --offline -- --list` で、1 バイナリに 1 テストだけを持つものを
洗い出した（cargo は単数のとき `1 test,` と表示するため、複数形 `tests,` だけを見る素朴な
grep では見つからない。要注意）。

| クレート | テストファイル | テスト名 |
|---|---|---|
| ubnfc-front | `tests/resolve_validate_fixtures.rs` | `external_fixture_diagnostics` |
| ubnfc-front | `tests/selfhost_pipeline.rs` | `parsed_main_and_imports_produce_the_existing_compile_result` |
| **ubnfc-fuzz** | `tests/recovery_diagnostics.rs` | `recovery_diagnostics_and_capture_occurrences_agree_between_backends` |
| **ubnfc-fuzz** | `tests/right_assoc_span.rs` | `right_associative_node_spans_agree_between_backends` |
| ubnfc-harness | `tests/scanners.rs` | `tiny_string_from_ir_registers_both_backends_without_oracle_rewriting` |
| ubnfc-p4-rust | `tests/regenerate.rs` | `p4_ir_regenerates_every_committed_byte` |
| ubnfc-rust | `tests/review.rs` | `java_review_regressions_in_generated_rust` |
| ubnfc-rust | `tests/semantics.rs` | `generated_semantic_contracts` |
| ubnfc-selfhost | `unittests src/lib.rs` | `tests::capture_site_metadata_matches_unmodified_golden` |

太字の 2 本（`ubnfc-fuzz`）が最有力候補: 両方とも `Engine::run` で driver を直接呼び、
`docs/harness-driver-protocol.md` の壁時計タイムアウトを経由する唯一のテストだからである
（他の 7 本は front/self-host の golden 比較か、harness 経由でも build のみでタイムアウトを
経由しない）。

### 再現手順と結果

`sh -c 'while true; do :; done'` を CPU コア数の 2〜3 倍立てて `/proc/loadavg` を
1 CPU あたり 1.1〜2.2 まで押し上げ（このホストは他 agent の並行ビルドも乗っている共有機で、
既定の load average も 12〜18 ある）、`ubnfc-java` の jar を先にビルドしたうえで
`cargo test -p ubnfc-fuzz --offline --test recovery_diagnostics --test right_assoc_span` を
未パッチのコード（`git stash` で今回の変更を退避した状態）で実行した。

| 実行 | 1 CPU あたり負荷（概算） | 結果 |
|---|---:|---|
| `recovery_diagnostics_and_capture_occurrences_agree_between_backends` | 1.1〜2.2 | ok（**284.79 秒**、通常は数秒〜十数秒） |
| `right_associative_node_spans_agree_between_backends` | 0.8〜1.8 | ok（**160.68 秒**） |

いずれも**失敗しなかった**。所要時間は通常の 10 倍以上に伸びたが、原因は `cargo build
--release` / `javac` がホストの他プロセスと CPU を奪い合って遅くなったことで、build 自体には
タイムアウトが無い（§0）ため、driver 交換（`Engine::timeout` 既定 90 秒）に達する前に
build が終われば通ってしまう。今回の負荷では 90 秒の壁に当たらなかった。

**結論**: 数分規模の CPU 高負荷では 23 秒フレークを再現できなかった。23 秒という短さは
「90 秒タイムアウトに触れて失敗した」よりも、**高負荷下で driver プロセス（`java` / 生成
Rust バイナリ）が短時間で異常終了した**（メモリ圧迫・fork 失敗など、`wait_or_kill` が
"driver 異常終了" として即座に返す経路）可能性の方が高い。この経路は `driver_timeout` の
印を付けない**本物のエラー**なので、今回の再試行の対象にはならない。当面は
`docs/reports/2026-09-23-differential-fuzzing.md` の `driver_timeout` discard（PR #64
作業中の 2 件、負荷低下後の再実行では 5/5 緑）を一次証拠として設計を進める。
異常終了系のフレークは監視を続け、再現できたら別 issue で追う。

## 2. 負荷率によるタイムアウトの伸長

`crates/ubnfc-fuzz/src/load.rs`:

```rust
pub fn factor() -> f64 {
    // /proc/loadavg の 1 分平均 ÷ available_parallelism()、[1.0, 4.0] にクランプ
}
pub fn scale(base: Duration) -> Duration {
    base.mul_f64(factor())
}
```

- 上限 4 倍は「恒常的な高負荷ホストでも driver 停止の検知が無限に遅れない」ための天井。
- `/proc/loadavg` が読めない環境（Linux 以外）は倍率 1（既定動作を変えない）。
- `Engine::run_with` が driver を起動するたびにその時点の負荷から掛け直す。
  `Engine.max_load_factor` に観測した最大値を残し、`campaign::Summary.max_load_factor` へ集約する。

## 3. `driver_timeout` の分類と再試行

`Engine::run_with` はタイムアウトで打ち切った応答に `{"error": "...", "kind":
"driver_timeout"}` を付ける（`engine::is_driver_timeout`）。`campaign::run_pair_with_retries`
はこの印が付いた比較だけを、**該当 backend だけ**新しいプロセスで
`Config::timeout_retries`（既定 2）回まで再試行する。同じバッチの他の入力は巻き込まない
（タイムアウトはプロセス単位の停止なので、入力を絞ったサブセットだけを新しいプロセスに
渡せば良い）。

`campaign::run_seed` は、再試行後もなお（片側だけでも）`driver_timeout` の比較を
`Summary.unobserved_timeouts` に数え、`differences` / `invariants` を呼ばずに素通りさせる
（`driver_errors` にも `findings` にも入れない）。片側だけ本物のエラー（異常終了・不正 JSON
など）が混ざっている場合はこれまで通り差分・`driver_errors` の対象のままにする。

`Summary` の新規フィールド（JSON レポートにそのまま残る）:

| フィールド | 意味 |
|---|---|
| `unobserved_timeouts` | 再試行を使い切ってもなお `driver_timeout` だった比較数 |
| `retried_comparisons` | 1 回以上再試行した比較の数 |
| `retry_attempts` | 実際に発行した再試行バッチの回数 |
| `max_load_factor` | campaign 中に観測した負荷率の最大値（1.0〜4.0） |

固定 seed campaign（`crates/ubnfc-fuzz/tests/campaign.rs`）の `driver_errors == 0` という
主張は変えていない（意味は「本物の driver エラーが無い」のまま）。無負荷なら
`unobserved_timeouts` も 0 になるはずなので、この変更は無負荷時の green を壊さない。

## 4. タイムアウトを模した確定的テスト

実ホスト負荷を必要とせず、`Built` を `Engine::prepare` を経由せず直接組み立てて（driver に
`sh -c '...'` の stub を挟み）driver_timeout を確定的に起こすテストを
`crates/ubnfc-fuzz/src/campaign.rs` の `mod tests` に 4 本追加した:

- `a_timed_out_comparison_is_retried_and_stops_being_unobserved_once_it_recovers`:
  最初の起動だけ眠り続け、2 回目（再試行）以降は正常応答する stub。再試行 1 回で
  `still_timeout == 0` になり、両者とも `accepted: true` を確認する。
- `a_comparison_that_never_recovers_stays_unobserved_after_retries_are_exhausted`:
  常に眠り続ける stub。再試行 1 回を使い切っても `still_timeout == 1`、
  もう片方の backend は正常なままであることを確認する。
- `zero_retries_reports_the_timeout_without_attempting_any_retry`:
  `max_retries=0` では再試行そのものをしない（`attempts == 0`）。
- `a_lingering_timeout_is_not_treated_as_a_hard_driver_error`:
  `run_seed` が使う分類条件（片側だけの `driver_timeout` は「未観測」であって
  `driver_errors` ではない）そのものを検証する。

## 5. 検証

origin/master（e1547ee、#93 の `h_text_group_captures_agree_across_backends` 追加を含む）を
merge した状態で、無負荷（他 agent の並行ビルドはあるがこのホストとしては通常時）に確認した。

- `cargo test --workspace --offline --no-fail-fast`: 全 crate green（0 failed）。
  `ubnfc-fuzz` の unittests は 33 passed（本 issue の 4 本 `campaign::tests::
  a_timed_out_comparison_is_retried_and_stops_being_unobserved_once_it_recovers` /
  `a_comparison_that_never_recovers_stays_unobserved_after_retries_are_exhausted` /
  `zero_retries_reports_the_timeout_without_attempting_any_retry` /
  `a_lingering_timeout_is_not_treated_as_a_hard_driver_error` を含む）。
  merge で新たに来た `tests/campaign.rs` の
  `h_text_group_captures_agree_across_backends`（#93）は `Config` リテラルに
  `timeout_retries` フィールドが無く非 fast-forward merge のまま conflict なく
  取り込まれていたので、`timeout_retries: 2` を足して他の統合テストと揃えた。
- `cargo clippy --workspace --offline --all-targets -- -D warnings`: warning 0。
- `cargo fmt --check`: 差分なし。
- 固定 seed campaign（無負荷、`tests/campaign.rs` 全 8 テスト）: green
  （`a_fixed_seed_campaign_reaches_both_backends_and_finds_no_unregistered_divergence` ほか）。
  無負荷時は `unobserved_timeouts` も 0 になるはずという §3 の想定どおり、
  既存の `driver_errors == 0` の assertion は変更なしで通っている。
- corpus 無退行（`cd ubnfc-java && mvn -o -q -DskipTests package` 後、
  `ubnfc-harness run-all --backend java|rust`）: **Java Pass 1310 / Fail 0**、
  **Rust Pass 1292 / Fail 0**（未観測 59 / 75、Known 29/29 も含め master 実測値と一致、
  差分なし）。
