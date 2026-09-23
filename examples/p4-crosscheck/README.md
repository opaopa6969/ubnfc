# P4 4 実装クロスチェック

同じ `.tiny` 入力を 4 つの P4 実装へ流し、受理と canonical AST を突き合わせる比較用ドライバ。
既存のパーサ・生成器のソースは変更しない。計測はしない。

| 記号 | 実装 | 位置 |
|---|---|---|
| R1 | ubnfc Rust backend の生成物 | `examples/p4-rust` |
| J1 | ubnfc Java backend の生成物 | `examples/p4-java` |
| R2 | unlaxer Rust codegen の生成物 + `unlaxer-runtime` | `tinyexpression/rust/tinyexpression-rs`（`parse` CLI） |
| J2 | 旧 Java（生成 parser + `unlaxer-common` combinator） | `examples/p4-java/scripts/legacy.sh` 経由（`mapper` と `facade`） |

## 走らせ方

```bash
bash scripts/run.sh [OUT_DIR]      # 既定 OUT_DIR は ./out
```

必要なもの: cargo（offline 可）、JDK 21、Maven、`~/.m2` の `unlaxer-common` / `unlaxer-dsl`（既定 3.0.15）、
`tinyexpression/target/classes` がビルド済みであること。
`TINYEXPRESSION_DIR` で tinyexpression の場所を変えられる。

**旧 Java（J2）は 1 入力あたり最大 45 秒の JVM 起動を 16 fixture × 2 モードぶん行うので、
全体で 20 分以上かかる**。`complex-x64` のような大入力は timeout で落ちることがあり、
その場合は「拒否/未取得」として記録する（速度の指標として読まないこと）。

## 入力

`examples/p4-rust/tests/fixtures/*.tiny` の 16 件（tinyexpression の
`benchmarks/fixtures` と `rust/tinyexpression-rs/tests/fixtures` から byte を変えず収録したもの）。

## 比較の作り方

4 実装とも canonical AST は `{"type","span","fields"}` 形で出るが、mapper の設計が違う。
**正規化は一切しない**で差分を全部数え、`引用符` / `span` / `木の形` に分類して提示する。
分類はあくまで読みやすさのためで、差分を消してはいない。

## 結果

`evidence/report.md`（2026-09-23 実行）。要点は
`docs/reports/2026-09-23-parser-feature-correctness-comparison.md` の Part 3.2。

## 仕様由来ケース（issue #5）

`corpus/spec/p4/*.json` の 32 ケースは tinyexpression の `spec/SPEC.md` /
`specs/language.md` の規範文から期待値を導いたもので、実装の観測値を使っていない。

```bash
bash scripts/run-spec.sh [OUT_DIR]      # 既定 OUT_DIR は ./out-spec
```

駆動するのは R1（`examples/p4-rust` の `--example spec-probe`）と
R2（`tinyexpression-rs` の `parse` CLI）だけ。J1 / J2 は 1 入力あたり JVM 起動が要るので
上の 16 fixture 比較に任せている。

実行済みの結果は `evidence/spec-report.md`。仕様側の留保（自己申告で実装由来、
文書間の優先順位の矛盾、Formula の production が無い）は
`docs/reports/2026-09-24-spec-derived-oracle.md` §6。
