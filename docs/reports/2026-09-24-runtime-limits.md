# ランタイムの資源制限を実測から見直す（issue #35）

2026-09-24。issue [#35](https://github.com/opaopa6969/ubnfc/issues/35)。
[`2026-09-24-antlr-treesitter-comparison.md`](2026-09-24-antlr-treesitter-comparison.md) §1 が見つけた 2 件の頑健性の欠陥を直す。

1. 生成 Rust ランタイムの native stack 番兵（256 KiB）が `ParseOptions.max_depth`（既定 10,000）と無関係な固定値で、
   JSONTestSuite の `i_structure_500_nested_arrays` のような正常入力を誤って拒否していた。
2. `ResourceLimits.arena_entries` の既定（2,000,000）が 1 MB の JSON を typed AST で解析するには小さすぎた。

D-070（stack 番兵）・D-071（arena 既定）として [`../design/03-design-v1.md`](../design/03-design-v1.md) に記録した。

---

## 1. native stack 番兵

### 1.1 フレームサイズの実測

`limit()` の native stack 番兵を一時的に大きくし（64 MiB）、`Session.depth`（`enter_rule` 呼出し回数）と
`anchor.abs_diff(address)`（呼出し元からの native stack 消費）を同時に記録して、深さ単位あたりの byte を測った。

| 文法 | 入力 | 条件 | 1 深さ単位（`enter_rule` 1 回）あたりの native stack |
|---|---|---|---:|
| JSON（生成 Rust） | `[[[…]]]`（n 重ネスト配列） | `build_ast: true`（既定） | **約 568 byte**（depth 5240→6000 の傾き 431,680/760） |
| JSON（生成 Rust） | 同上 | `build_ast: false`（認識のみ） | 256 KiB の番兵に約 313 重ネストで当たる（≈256*1024/313/2 depth-units ≈ 419 byte/unit 相当。境界計測のみ） |
| P4（`examples/p4-rust`） | `(…)`（n 重ネスト括弧） | `ParseOptions::default()` | **約 620–750 byte**（predict/factor の後戻りで depth が往復するため 2 系列で幅がある） |

JSON は `build_ast: true`（既定オプション）だと 256 KiB の番兵に **約 229 重ネスト**で当たる
（`262144 / 229 ≈ 1145 byte/レベル` ≒ 568 byte/深さ単位 × 2 単位/レベル、と整合）。
`i_structure_500_nested_arrays`（500 重ネスト）はこの約 2.2 倍で、`max_depth`（10,000。JSON は
1 レベルあたり 2 `enter_rule` なので論理上限は 5,000 レベル相当）にはまだ大きく余裕があるのに
番兵だけが先に落ちていた。

P4 は Java 側でも同じ入れ子（`"(".repeat(depth) + "1" + ")".repeat(depth)`）を `-Xss` を振って測ると、
1 レベルあたり約 5.6 KiB（後述 §1.3）で、Rust より深さ単位が細かい分 1 レベルの重みが大きい
（predict/factor で複数の expression を試す分、深さ単位の粒度が異なる）。

### 1.2 選んだ規則（D-070）

呼出し元 thread の実スタック量を生成ライブラリからは知りようがないため、
**番兵を単純に大きくする案は採らなかった**（呼出し元が明示的に小さい stack で呼んでいる場合、
本物のスタックオーバーフローに番兵より先に当たってしまう。
`crates/ubnfc-rust/tests/support/review_assertions.rs` の `recursion_aborts_optional_and_recovers_next_call`
後半が 1–2 MiB の専用 thread で安全性を確かめている契約）。

代わりに 2 段構えにした。

1. **既定呼出し**（`parse` / `parse_with_options` / `parse_entry_with_options`）は呼出し元 thread の上で
   旧来どおり保守的な `NATIVE_STACK_SENTINEL_BYTES = 256 * 1024` のまま解析する。安全性は不変。
2. 番兵に当たった時点で **`Session.depth` が `max_depth` 未満**（＝論理上限にはまだ余裕がある）なら、
   `max_depth` から見積もった十分な stack を持つ **専用 thread へ 1 度だけエスカレーション**する。
   - `escalated_stack_budget(max_depth) = max_depth * 2048`（実測の最大 750 byte に約 3 倍の安全率）。
   - 実際に確保する thread stack は `escalated_stack_budget + 2 MiB`（下限 8 MiB・上限 512 MiB）。
   - `std::thread::scope` + `Builder::spawn_scoped` を使い、`text: &str` は借用のままエスカレーション先へ渡す
     （1 MB 入力でも copy しない）。`RejectExtern` は状態を持たない zero-sized 型なので thread をまたいで
     再構築してよい。
   - エスカレーション後の thread でも `max_depth` に当たれば、同じ診断（`maximum parse depth exceeded`）で
     確定する。無限にはエスカレーションしない（`parse_entry_budget` はエスカレーション判定を持たない
     non-escalating 版）。

**外部 `TokenScanner` を受け取る `parse_with_scanner` は対象外**にした。任意の状態を持つ scanner は
`&mut dyn TokenScanner` で thread 境界を越えられる保証が無い（`Send` を要求すると既存 API を壊す）。
P4 は Extern token を持つのでこの経路になり、`-Xss`/stack size 非依存化の恩恵を受けない
（旧来どおり。既存の `deeply_nested_input_reports_a_depth_diagnostic_instead_of_crashing` は無変更で緑）。

Java は同じ契約を `Thread` の stack size 指定（`new Thread(ThreadGroup, Runnable, String, long)`）で揃えた。
`Session.stackOverflowTripped`（`DepthLimit` ではなく `StackOverflowError` を捕まえたときだけ true）を見て、
`options.scanners()` が空のときだけ `Session.escalatedStackBytes(maxDepth) = maxDepth * 8192 + 2 MiB`
（下限 8 MiB・上限 512 MiB）を持つ `Thread` で 1 度だけ解析し直す。extern scanner を持つ文法（P4）は
Rust と同じ理由で対象外にして両 backend の契約を揃えた。

### 1.3 両 backend の実測（500 / 1,000 / 10,000 段、既定 options、既定 -Xss）

JSON（`[[[…]]]`、n 重ネスト配列。既定 `ParseOptions`/`ParseOptions.DEFAULT`、Rust は呼出し元 thread が
既定サイズ、Java は `-Xss` を渡さない既定 JVM）:

| 深さ | ubnfc Rust | ubnfc Java |
|---:|---|---|
| 200 | 受理 | 受理 |
| 500 | **受理**（旧: 拒否） | **受理**（旧: 拒否） |
| 1,000 | **受理**（旧: 拒否） | **受理**（旧: 拒否） |
| 5,000 | 拒否・`maximum parse depth exceeded` | 拒否・`maximum parse depth exceeded (maxDepth=10000)` |
| 10,000 | 拒否・`maximum parse depth exceeded`（クラッシュしない） | 拒否・`maximum parse depth exceeded (maxDepth=10000)`（クラッシュしない） |

5,000/10,000 は JSON 1 レベルあたり `enter_rule` 2 回相当なので、`max_depth`（既定 10,000）の論理上限に
ちょうど当たる（500/1,000 段の native stack 番兵ではなく、こちらで確定する）。

P4（`(…)`、n 重ネスト括弧。extern scanner を使う経路なので **エスカレーション対象外** — Rust は
`parse_formula`/`parse_entry`、Java は `P4Scanners.options(ast)` が非空の scanners map を渡す）:

| 呼出し元 stack | 深さ | ubnfc Rust | ubnfc Java |
|---|---:|---|---|
| 256 KiB（`-Xss256k`） | 50 | 拒否・native stack 由来（旧来どおり。エスカレーション対象外） | 拒否・`JVM stack limit`（旧来どおり） |
| 十分大きい（`-Xss64m`） | 50 / 1,000 | 受理（純粋に論理上限の範囲内。エスカレーションとは無関係） | 受理 |
| 十分大きい | 4,000 / 5,000 | 拒否・`max_depth` | 拒否・`maxDepth=10000` |

P4 は extern scanner 経路なので、issue #35 の対象範囲外（`-Xss`/stack size 非依存化は今回も未対応のまま。
`docs/design/03-design-v1.md` の既存の記述どおり）。既存の depth テスト（Rust
`examples/p4-rust/tests/fixtures.rs::deeply_nested_input_reports_a_depth_diagnostic_instead_of_crashing`、
Java `examples/p4-java/scripts/depth.sh`）は無変更で緑。

---

## 2. `arena_entries` の既定値

### 2.1 entries/byte の実測

`ParseResult.statistics.recipes`（実際に使った arena entry 数）を、`arena_entries` を大きく上書きして
（超過による打ち切りを避けて）測った。

| 文法 | 入力 | entries | bytes | entries/byte |
|---|---|---:|---:|---:|
| JSON（typed AST） | `benchmarks/scripts/make-json-inputs.py` の 1 MB 入力 | 4,288,393 | 1,000,112 | **4.29** |
| P4（typed AST） | `examples/p4-rust/tests/fixtures/complex-x64.tiny` | 80,246 | 20,923 | **3.84** |
| Operators（typed AST） | `"1".repeat(512).join("+")` | 9,212 | 1,023 | **9.00**（実測の最大） |

既定の `arena_entries`（2,000,000）は JSON 1 MB の 4,288,393 entries に対して半分以下しかなく、
`arena budget exceeded` になる。

### 2.2 選んだ規則（D-071）

`resource_limits_fail_without_partial_success`（`crates/ubnfc-rust/tests/support/round2_assertions.rs`）が
`arena_entries` をわざと小さい値（`2`）に設定して budget 超過を確実に踏ませる契約テストなので、
**既定値を引き上げる対象は「明示的に設定しなかった呼出し」に限定**する必要がある。型を変えずにこれを
見分けるため、`Session::new` は `options.limits.arena_entries` が新しい公開定数
`DEFAULT_ARENA_ENTRIES`（2,000,000）と**一致するときだけ**

```
scaled_default_arena_entries(input_bytes) = max(2_000_000, input_bytes * 16).min(32_000_000)
```

で実効上限を置き換える（`16` は実測最大 9.00 entries/byte に約 1.8 倍の安全率、上限 32,000,000 は
約 2 MB 入力相当）。それ以外の値（契約テストの `2` や、既存ベンチマークが「実質無制限」の意味で
使っている `1 << 30` など）はそのまま使う。

**既知の割り切り**: 既定値と全く同じ 2,000,000 を明示した呼出しも「未指定」として扱われスケーリング
される。正確に 2,000,000 に固定したいだけの意図とは区別できない（別の値、例えば `2_000_001` を
使えば回避できる）。

**Java は対象外**: `ParseOptions` に arena 相当の上限が無く、heap がそのまま境界になる。1 MB JSON の
typed AST は既定で通ることを確認しただけで、コード変更は無い。

### 2.3 実測（1 MB JSON typed AST、既定 options）

| 実装 | 既定 `arena_entries`/相当 | 結果 |
|---|---|---|
| ubnfc Rust（旧） | 2,000,000（固定） | `arena budget exceeded` |
| ubnfc Rust（新） | `scaled_default_arena_entries(1,000,112) = 16,001,792` | **受理**（`ast.is_some()`） |
| ubnfc Java | heap（上限なし） | 受理（変更なし） |

明示的に小さい `arena_entries`（`resource_limits_fail_without_partial_success` の `2`）を設定した呼出しは
不変（`arena budget exceeded` のまま）。

---

## 3. JSONTestSuite 再検証

`i_structure_500_nested_arrays`（[`2026-09-24-antlr-treesitter-comparison.md`](2026-09-24-antlr-treesitter-comparison.md) §1.1 の表 B 行）を、
生成 Rust / Java の既定 `ParseOptions` で単独再実行した（`benchmarks/external/JSONTestSuite/test_parsing/i_structure_500_nested_arrays.json`、500 重ネスト、1000 byte）。

| file | suite の分類 | ubnfc Rust（新） | ubnfc Java（新） | ANTLR4 | tree-sitter |
|---|---|---|---|---|---|
| `i_structure_500_nested_arrays` | 実装依存 | **accept**（旧: reject） | **accept**（旧: reject） | accept | accept |

4 実装の割れは `i_structure_UTF-8_BOM_empty_object`（BOM、C — 規格内の実装差、対象外）の 1 件だけになった。

---

## 4. 検証

- `cargo build --offline --workspace` / `cargo test --offline --workspace --no-fail-fast` 緑（0 failed）。
- `cargo clippy --offline --workspace --all-targets -- -D warnings` 緑、`cargo fmt --all --check` 緑。
- `cd ubnfc-java && mvn -o -q -DskipTests package` の後、`mvn -o test` 緑（150/150）。
- `examples/p4-java`: `scripts/generate.sh --check` 緑（生成物は commit 済みのものと byte 一致）、`mvn -o test` 緑（31/31）。
- `examples/p4-rust`: `cargo test -p ubnfc-p4-rust` 緑、`tests/regenerate.rs` 緑（生成物 byte 一致）。
- `crates/ubnfc-selfhost`: `tests/regenerate.rs` 緑（自己ホスト固定点も byte 一致）。
- corpus（`ubnfc-harness run-all`）: **Java Pass 1,310 / Fail 0**、**Rust Pass 1,292 / Fail 0**（無退行）。
- 回帰テスト:
  - Rust: `crates/ubnfc-rust/tests/support/review_assertions.rs::recursion_aborts_optional_and_recovers_next_call`
    （2,000 段は受理・20,000 段は診断、1–2 MiB の専用 thread から検証）。
  - Java: `ubnfc-java/src/test/java/org/ubnfc/java/ReviewRegressionTest.java::recursionLimitAbortsWholeParseAndNeverLeaksStackOverflow`
    （同型。別 JVM プロセスを `-Xss256k`/`-Xss1m` で起こして 2,000 段受理・20,000 段診断を確認）。

## 5. D-* 行

D-070（native stack 番兵のエスカレーション）・D-071（`arena_entries` 既定のスケーリング）を
[`../design/03-design-v1.md`](../design/03-design-v1.md) に追加した（この報告の作成時点で最後の行は D-069）。
