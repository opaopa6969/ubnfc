# UBNF 4 実装クロスチェック

同じ UBNF 入力を 4 つのパーサへ流し、受理・拒否位置・正準 AST を突き合わせる比較用ドライバ。
既存のパーサ・生成器のソースは変更しない。計測はしない（速度は `docs/reports/` の速度報告）。

| 記号 | 実装 | 位置 |
|---|---|---|
| A | ubnfc 手書き frontend | `crates/ubnfc-front/src/syntax` |
| B | ubnfc 自己ホスト生成 frontend | `crates/ubnfc-selfhost` |
| C | unlaxer-ubnf（手書き Rust、依存ゼロ） | `unlaxer-parser/rust/unlaxer-ubnf`（`examples/inspect`） |
| D | Java bootstrap | `unlaxer-dsl` の `org.unlaxer.dsl.bootstrap.UBNFMapper` |

既存のテストは **A×B**（`crates/ubnfc-selfhost/tests/parity.rs`）と
**C×D**（`unlaxer-dsl/.../RustUbnfFrontendConformanceTest.java`、既定スキップ）の 2 組しか比べていない。
ここはその 4 つを同時に並べる。

## 走らせ方

```bash
bash scripts/run.sh [OUT_DIR]      # 既定 OUT_DIR は ./out
```

必要なもの: rustc / cargo（offline 可）、JDK 21、`~/.m2` の `unlaxer-common` と `unlaxer-dsl`（既定 3.0.15）。
環境変数 `UNLAXER_PARSER_DIR` / `TINYEXPRESSION_DIR` / `MAVEN_REPO` / `UNLAXER_VERSION` で場所を変えられる。
参照リポジトリは読むだけで、その `target/` にも書かない（`unlaxer-ubnf` は `rustc` で mktemp へビルドする）。

出力は `OUT_DIR/` に `ubnfc.jsonl` / `unlaxer-ubnf.jsonl` / `java-bootstrap.jsonl` / `corpus.tsv` / `report.md`。

## 入力

- `.ubnf` ファイル 25 件（unlaxer-parser 配下の全 24 件 + tinyexpression の P4 LSP 文法 1 件。
  `tinyexpression/docs/ubnf/` の草稿 3 件は最終版ではないので入れていない）
- `unlaxer-parser/rust/unlaxer-ubnf/tests/fixtures/negative.tsv` の不正入力 23 件

計 48 件。`evidence/corpus.tsv` が実際に使った一覧。

## 比較の作り方

3 系統すべてを `unlaxer-parser/rust/unlaxer-ubnf/tests/support/canonical.rs` の
**span 抜き正準形**へ射影して比べる。ubnfc の構文木は量指定子を要素の属性として持ち、
unlaxer 系は要素種別として持つので、そこだけ機械的に写す
（`X?`→`OptionalElement`、`X*`→`RepeatElement`、`X+`→`OneOrMoreElement`、
`X{n,m}`→`BoundedRepeatElement`、`X % s`→`SeparatedElement`）。
**差を消す目的の正規化はこれ以外に入れていない。**

`@eval` の AST 表現だけは両系統で設計が違う（ubnfc は `key=value` の並び、unlaxer は
`(kind, strategy, params)`）。ここは射影で埋めず、差分としてそのまま出る。

Java 側の正準化は `RustUbnfFrontendConformanceTest.canonical()` と同じ反射アルゴリズムを
`java/UbnfProbe.java` へ写したもの。忠実さの裏付けは「C と D が、記録済みの Java 欠陥 2 件を除く
全入力で完全一致した」こと。

## 結果

`evidence/report.md`（2026-09-23 実行）。要点は
`docs/reports/2026-09-23-parser-feature-correctness-comparison.md` の Part 3.1。
