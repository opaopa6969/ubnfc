# examples/p4-vscode — TinyExpression P4 の VS Code 拡張（ubnfc 生成）

`ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json` から
**パーサと LSP サーバの両方を生成**し、VSIX に梱包する例。issue #28 の段階 2 と 3。
設計判断は D-060 / D-061、調査と実測は
[`docs/reports/2026-09-24-lsp-vsix-stage1.md`](../../docs/reports/2026-09-24-lsp-vsix-stage1.md) と
[`docs/reports/2026-09-24-lsp-vsix-stage3.md`](../../docs/reports/2026-09-24-lsp-vsix-stage3.md)。

## 組み方

```sh
scripts/build-vsix.sh p4         # リポジトリ root から。p4 / ubnf / all（既定 all）
```

6 段階を通す。

1. 生成器を組む（`mvn -Plsp package`。**LSP は opt-in なので profile 無しでは入らない**）
2. `ubnfc-java --ir … --package org.ubnfc.p4.generated --no-emit-driver` でパーサ生成
3. `ubnfc-lsp --emit-lsp --ir … --package org.ubnfc.p4.lsp --parser-package org.ubnfc.p4.generated
   --emit-tmlanguage syntaxes/tinyexpression-p4.tmLanguage.json`
   で LSP サーバと TextMate 文法を生成（**別パッケージへ追加するだけ**。2 の出力は 1 byte も変わらない）
4. extern token scanner（`P4Scanners`）を `examples/p4-java` から複製（読むだけ）
5. `mvn package`（`server/pom.xml`。golden テストが走り、shade で fat jar）
6. `npm ci` + `vsce package`

環境変数: `MAVEN_FLAGS=`（依存が未取得のとき `-o` を外す）、`SKIP_VSIX=1`（5 まででやめる）。

出力:

- `examples/p4-vscode/server/target/ubnfc-p4-lsp-server.jar`（fat jar、`java -jar` で stdio LSP）
- `examples/p4-vscode/server-dist/ubnfc-p4-lsp-server.jar`（拡張に同梱するコピー）
- `examples/p4-vscode/syntaxes/tinyexpression-p4.tmLanguage.json`（**生成物**。毎ビルド上書き）
- `examples/p4-vscode/target/ubnfc-tinyexpression-p4-0.1.0.vsix`

**リポジトリに入れないもの**: `node_modules/`、`out/`、`target/`（生成ソースを含む）、
`server-dist/`、`syntaxes/`、`*.vsix`。すべて `.gitignore` 済み。

## 入れ方

```sh
code --install-extension examples/p4-vscode/target/ubnfc-tinyexpression-p4-0.1.0.vsix
```

`.tiny` を開くと、構文エラー（Error）と未定義シンボル参照（Warning）が波線で出る。
定義へ移動・参照の一覧・アウトライン・hover・補完・semantic tokens も効く
（capability の一覧は報告書 §1）。構文エラーのある文書でも、
**エラーより前の範囲**については宣言・参照・scope 警告（`code: scope-prefix`）が出る（D-061）。

## 中身

| ファイル | 役目 |
|---|---|
| `package.json` | 言語 id `tinyexpression-p4`（`.tiny`）、設定（java パス / jar パス / JVM 引数） |
| `src/extension.ts` | `vscode-languageclient` で `java -jar <server.jar>` を stdio 起動 |
| `server/pom.xml` | **生成ソースだけ**をコンパイルする（`sourceDirectory` は `target/generated-sources/ubnfc`）。依存は lsp4j 0.23.1 のみ |
| `server/src/test/**` | 台本 LSP クライアント（`ScriptedClient`）、診断 golden、capability golden（`probes/<fixture>.txt` の位置ごとに definition / references / hover / completion）、`Positions` の単体テスト |
| `../../scripts/lsp-probe.py` | 新旧どちらのサーバにも同じ手順で話す台本クライアント（パリティ計測に使う） |
| `evidence/parity-{old,new}.json` | 16 fixture × 新旧サーバの診断（段階 1 §5・段階 3 §3.4 の元データ） |

## パリティの取り直し

```sh
python3 scripts/lsp-probe.py \
  --jar examples/p4-vscode/server-dist/ubnfc-p4-lsp-server.jar \
  --language-id tinyexpression-p4 \
  --fixtures examples/p4-java/src/test/resources/fixtures --out evidence/parity-new.json

# 旧スタックは既存のビルド成果物を scratch へ複製して使う（兄弟リポジトリには書かない）
cp ~/work/unlaxer-workspace/tinyexpression/tools/tinyexpression-p4-lsp-vscode/server-dist/tinyexpression-p4-lsp-server.jar "$scratch/legacy.jar"
python3 scripts/lsp-probe.py --jar "$scratch/legacy.jar" \
  --language-id tinyexpressionP4 --jvm-arg=--enable-preview --jvm-arg=-Xss16m \
  --fixtures examples/p4-java/src/test/resources/fixtures --out evidence/parity-old.json
```
