# 差分 fuzzing の所見 `diff:/catalogs` / `diff:/diagnostics/*` の追跡（2026-09-23）

対象: 固定 seed campaign（`cargo test -p ubnfc-fuzz --offline --test campaign`）が出したとされる
2 つの未登録 signature。

```
"diff:/catalogs (diff-catalogs-dd4854a9-seed4002)"
"diff:/diagnostics/* (diff-diagnostics-988b1c0e-seed4001)"
```

結論を先に書く。**どちらも生成器のバグではなく、`ubnfc-java` の jar を package し直さずに
campaign を回したときにだけ出る誤報**だった。契約はすでに D-025 / D-026 で決まっていて、
両 driver の実装もそれに従っている。再発防止としてハーネスが jar の鮮度を検査するようにした（D-033）。

## 1. 現状の確認（master 65c4a3f、jar を package し直した状態）

| 実行 | 結果 |
|---|---|
| `cargo test -p ubnfc-fuzz --offline --test campaign` | **green**（2 passed） |
| seed 4001..4004 / inputs 10（固定 seed と同条件） | 所見 2 件、いずれも `KNOWN`（`diff:/captures/*`、`diff:/diagnostics/*/farthestExpected/*`） |
| seed 4001..4040 / inputs 16（640 比較） | `diff:/catalogs`・`diff:/diagnostics/*` は **0 件** |
| seed 1..80 / inputs 16（1,280 比較） | `diff:/catalogs`・`diff:/diagnostics/*` は **0 件** |
| seed 1 / inputs 24 / shrink 10（`findings/README.md` の元 recipe） | 残るのは `diff-captures-ccc0d8bb-seed1`（A-7、既知）だけ |

`crates/ubnfc-fuzz/findings/diff-catalogs-dd4854a9-seed1/` と
`diff-diagnostics-988b1c0e-seed1/` は seed 1 の同一文法・同一入力（空文字列）から採られた
**D-024 〜 D-029 以前の記録**で、いまは再現しない。`finding.json` が同時に挙げている
`/mappingError/present`・`/recoveries`・`/scope/*`・`/value` も同じ commit で閉じている。
slug の hash（`dd4854a9` 等）は **signature だけの hash** なので、seed 番号が違っても
同じ signature なら同じ hash が付く。報告にあった `-seed4002` / `-seed4001` は
「その signature が最初に出た seed」であって、別の所見ではない。

## 2. 根本原因（実走で確定）

Java backend の driver は **jar の resource**
（`ubnfc-java/src/main/resources/templates/driver/Driver.java`）から生成される。
`catalogs` キーと D-026 の成功時診断の抑止は、どちらも 3fb7cfd（D-024 〜 D-029）で
**このテンプレートに**入った。したがって 3fb7cfd より前に package した jar が
`ubnfc-java/target/` に残っていると、front・Rust 側は新しい契約、Java 側だけ古い契約という
組合せで campaign が走り、「Java が契約を破っている」ようにしか見えない差分が出る。

再現実験（この branch で実施し、あとで元に戻した）: 現在のテンプレートから

- `"catalogs", CATALOGS` の 1 ペアを削る（D-025 (c) を外す）
- `boolean speculative = ...` を `false` に固定する（D-026 を外す）

の 2 点だけを取り除いて package し直し、固定 seed campaign を回すと:

```
未登録の差分: [
    "diff:/catalogs (diff-catalogs-dd4854a9-seed4001)",
    "diff:/diagnostics/* (diff-diagnostics-988b1c0e-seed4001)",
]
```

報告と同じ signature・同じ hash が、**この 2 点だけ**で再現した。
テンプレートを戻して package し直すと campaign は再び green になる。

（3fb7cfd^ の jar をそのまま持ってくると `Missing bundled IR schema` /
IR 読み込み失敗で `build:java` discard になる。front の IR が D-032 で先に進んでいるため、
「かなり古い jar」は差分ではなく discard として落ちる。誤報になるのは
**IR は読めるがテンプレートだけ古い**、つまり 3fb7cfd 周辺の jar である。）

## 3. 所見ごとの規範

### `diff:/catalogs` — `expected {"<unobserved>": "catalogs"}` vs `actual []`

正本は **D-025 (c)**（`docs/design/03-design-v1.md` §8）。
`catalogs` は `@catalog` の**静的表** `{rule, context, captures}` で、入力に依らない metadata である。
Rust だけが返していたのを「corpus の Catalog 系統が要求する観測なので Java も返す」と決めた。
静的表なので `@catalog` の無い文法でも**キーは常に在り、中身が空配列になる**。
`harness-driver-protocol.md` の「観測済みの `null` とキーの欠落を区別する」規則（D-028 (c) と同じ立場）から、
`<unobserved>`（キーごと無い）と `[]`（空と観測した）は別物であり、
キーを落としていた側（古い Java driver）が違反。現在の `Driver.java:145` は
`"catalogs", CATALOGS` を無条件に置くので違反はない。**修正不要。**

### `diff:/diagnostics/*` — Java に 1 件、Rust に 0 件

正本は **D-026**。全入力解析が構文成功した応答は `syntax` / `trailing_input` 診断を持たない。
成功経路の中の投機的失敗は D-023 の `hints` の領分で、`hints` は言語別（Verdict に使わない）。
古い Java driver は成功時にも合成 `syntax` 診断を 1 件足していたため、
`diagnostics` の添字が全部ずれて配列比較が丸ごと差分になっていた。
現在のテンプレートは `speculative` で抑止済み。**修正不要。**

なお同じ `finding.json` が並記している `/diagnostics/0` の `farthestExpected` 差
（`RootRepeat0Parser` 等）は別件で、**D-027** の「文法に現れない合成 parser の表示名は
D-014 の言語別 hints に含める」に当たる。これは `tests/campaign.rs` の
`diff:/diagnostics/*/farthestExpected/*` として既に登録済みで、蓄積規則そのものは共有契約のまま。

## 4. やったこと

1. **D-033**: `Runner::build` が既定 jar を使う経路で、`ubnfc-java/src/main` と `pom.xml` の
   最新 mtime が jar より新しければ build をエラーにする
   （`crates/ubnfc-harness/src/runner/mod.rs` の `check_java_generator_freshness`）。
   固定 seed campaign は `discarded == 0` を主張しているので、古い jar は
   「存在しない乖離」ではなく `discard [build:java] Java 生成器 jar がソースより古い …` として即座に赤くなる。
   classpath を明示する経路（テストの `target/classes`）は maven が鮮度を保つので検査しない。
   mtime が読めない環境では検査を飛ばす。
   実測: テンプレートを `touch` しただけで
   `-> discard [build:java] Java 生成器 jar がソースより古い: …/ubnfc-java-0.1.0-SNAPSHOT.jar より
   …/templates/driver/Driver.java が新しい (cd ubnfc-java && mvn -o -q -DskipTests package を先に実行)`。
2. **`crates/ubnfc-fuzz/findings/README.md`**: 「修正済みの所見」表に D-024 〜 D-029 の行を足し、
   seed 1 で再現しなくなった 9 slug（`diff-catalogs-…`・`diff-diagnostics-…`・`diff-recoveries-…`・
   `diff-scope-{declarations,depth,diagnostics,references}-…`・`diff-mappingerror-present-…`・`diff-value-…`）を記録した。
   `diff:/value` と `diff:/mappingError/present` は**クラスとしては seed 6 に残る**（原因は A-4）ので、
   消えたのは seed 1 の事例だけであることも書いた。

**既知 signature は増やしていない。** 言語別の正当な観測差は見つからなかったので、
`tests/campaign.rs` の `KNOWN` は 2 件（`diff:/captures/*`、`diff:/diagnostics/*/farthestExpected/*`）のまま。
corpus 期待値・oracle JSON（`corpus/oracle/divergences.json` を含む）・emitter・driver template は変更していない。

## 5. 検証

| 項目 | 結果 |
|---|---|
| `cargo test --workspace --offline` | green（campaign・selfhost `unmodified_golden_regenerates_every_committed_byte`・p4-rust `generated_crates_build_offline_and_match_all_golden` を含む） |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `cd ubnfc-java && mvn -o -q test` | green |
| `cd examples/p4-java && mvn -o -q test` | green |
| Java corpus `docs/reports/2026-09-23-java-corpus-fuzzfix2.md` | **Pass 1310 / 期待拒否 28 / Known 28 / Fail 0 / 未観測 59** |
| Rust corpus `docs/reports/2026-09-23-rust-corpus-fuzzfix2.md` | **Pass 1292 / 期待拒否 30 / Known 28 / Fail 0 / 未観測 75** |

corpus は `2026-09-23-{java,rust}-corpus-contract.md` と**差分が SHA-256 行と Rust の build 秒数だけ**で、
系統別の表は 1 行も動いていない（Java は 2 行、Rust は 3 行の差分のみ）。

fuzzer の before / after:

| | 固定 seed campaign | 未登録 signature |
|---|---|---|
| before（`catalogs` と D-026 を外したテンプレートで package した jar） | FAILED | `diff:/catalogs`、`diff:/diagnostics/*` |
| after（master のテンプレートで package した jar） | ok（2 passed） | なし |
| 広げた campaign（seed 1..80 / 1,280 比較、seed 4001..4040 / 640 比較） | — | `diff:/catalogs`・`diff:/diagnostics/*` は 0 件。driver error 0 / timeout 0 / discard 0 |

広げた campaign で出る他の signature（`diff:/ast/**/span/*` の入れ子 span、`diff:/value`、
`diff:/mappingError/present`、`diff:/diagnostics/*/expected/*`、`diff:/diagnostics/*/farthestOffset`、
`diff:/recoveries/*/captureOccurrences/*`）は
`docs/reports/2026-09-23-differential-fuzzing.md` の A-2 / A-3 / A-4 系統の既存の所見で、
固定 seed の範囲には入っていない。今回の変更で新しく出たものは無い。
