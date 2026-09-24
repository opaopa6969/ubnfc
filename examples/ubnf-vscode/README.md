# examples/ubnf-vscode — UBNF 自身の VS Code 拡張（ubnfc 生成）

`ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json`（`unlaxer-dsl/grammar/ubnf.ubnf` の IR）から
**パーサと LSP サーバの両方を生成**し、VSIX に梱包する。`examples/p4-vscode` と**同じ経路**で、
違うのは IR・パッケージ名・言語 id だけ。issue #28 の段階 4 の前半。
設計判断は D-060 / D-061、実測とパリティは
[`docs/reports/2026-09-24-lsp-vsix-stage3.md`](../../docs/reports/2026-09-24-lsp-vsix-stage3.md)。

`ubnf.ubnf` は extern token を使わないので、P4 と違って scanner の複製が要らない。

## 組み方

```sh
scripts/build-vsix.sh ubnf     # リポジトリ root から。p4 / ubnf / all（既定 all）
```

環境変数: `MAVEN_FLAGS=`（依存が未取得のとき `-o` を外す）、`SKIP_VSIX=1`（VSIX を組まない）。

出力:

- `examples/ubnf-vscode/server-dist/ubnfc-ubnf-lsp-server.jar`（fat jar、`java -jar` で stdio LSP）
- `examples/ubnf-vscode/syntaxes/ubnf.tmLanguage.json`（**生成物**。毎ビルド上書き）
- `examples/ubnf-vscode/target/ubnfc-ubnf-0.1.0.vsix`

## 何が出るか

`.ubnf` を開くと:

- 構文エラー（Error）。`ubnf.ubnf` が書いている文法**そのまま**の受理範囲
- 規則名の **定義へ移動 / 参照の一覧 / アウトライン / hover**
  （`@declares(symbol=name)` は `sameRuleText` モードなので `ScopeStore` に載らない。
  IR の capture site と `lexical()` を突き合わせて拾っている。`ubnf.ubnf` 自身で 59 宣言 / 110 参照）
- semantic tokens（keyword / operator / string / number / variable）と TextMate 文法の下地
- 補完（その位置に書ける終端 + それより前で宣言済みの規則名）

**出ないもの**: `GrammarValidator` 相当の意味検査（`W-TOKEN-UNRESOLVED` / `E-RULE-UNDEFINED` /
`W-LEFT-RECURSION` など）。旧 `unlaxer-dsl/ubnf-vscode` はこれを出す。ubnfc では front
（`ubnfc validate`）が同じ検査を持つが、LSP サーバには載せていない。

## 旧 `unlaxer-dsl/ubnf-vscode` との突き合わせ

```sh
scripts/ubnf-parity.sh
```

24 文法（`crates/ubnfc-front/tests/pipeline_golden.rs` の `FILES`）を新旧サーバに同じ手順で流す。
旧サーバの jar は scratch へ複製して動かす（兄弟リポジトリには書かない）。
結果は `evidence/parity-{old,new}.json` と `evidence/fixtures.tsv`、読み方は報告書 §5.2。

## 中身

| ファイル | 役目 |
|---|---|
| `package.json` | 言語 id `ubnf`（`.ubnf`）、TextMate 文法、設定（java パス / jar パス / JVM 引数） |
| `src/extension.ts` | `vscode-languageclient` で `java -jar <server.jar>` を stdio 起動 |
| `server/pom.xml` | **生成ソースだけ**をコンパイルする。依存は lsp4j 0.23.1 のみ |
| `server/src/test/**` | 台本 LSP クライアント、診断 golden、capability golden、`Positions` の単体テスト |
| `evidence/` | 24 文法 × 新旧サーバの診断（報告書 §5.2 の元データ） |

**リポジトリに入れないもの**: `node_modules/`、`out/`、`target/`、`server-dist/`、`syntaxes/`、`*.vsix`。
