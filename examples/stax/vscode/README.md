# stax の生成 LSP と VSIX（段階4）

`.stax` の構文・未定義参照の診断、定義ジャンプ、補完、hover を使う拡張。
サーバは既存の `--emit-lsp` だけで生成する。Rust の型検査器との統合は含まない。
名前が未定義なら警告を出すが、スタック効果・型の誤りは `stax --check` で検査する。

## 生成・build・起動

repository ルートから、stable Rust、SDKMAN の JDK 21 / Maven を使う。
VSIX 作成には Node.js と npm も必要。ここでは JDK 21.0.9、Maven 3.9.9、
Node.js 20.20.0、npm 10.8.2 で検証した。

```sh
# 初回の Maven 依存取得を許す。取得済みなら MAVEN_FLAGS を省略して offline build。
MAVEN_FLAGS= bash examples/stax/scripts/build-lsp.sh

# stdio LSP。stdout は JSON-RPC 用なので、通常の端末から入力して使うものではない。
java -jar examples/stax/vscode/server-dist/ubnfc-stax-lsp-server.jar

# protocol・TextMate・回復配置の golden を照合する通常 CI の入口。
RUSTUP_TOOLCHAIN=stable cargo test --offline -p ubnfc-harness --test stax_lsp

# npm ci（package-lock.json の版）→ TypeScript → ローカル VSIX。publish は行わない。
MAVEN_FLAGS= bash examples/stax/scripts/build-lsp.sh --vsix
```

VSIX は `target/ubnfc-stax-0.1.0.vsix`、サーバは
`server-dist/ubnfc-stax-lsp-server.jar`。いずれもこのディレクトリ以下。
CI の `cargo test --workspace` は VS Code と npm を使わず、生成した fat jar に
Content-Length 付き JSON-RPC を送って検証する。

## 生成元と生成物

| 入力・処理 | 出力（このディレクトリからの相対パス） |
|---|---|
| `../stax.ubnf` → `ubnfc ir` | 一時ディレクトリの `stax.ir.json` |
| 同じ IR → `ubnfc-java --no-emit-driver` | `server/target/generated-sources/ubnfc/org/example/stax/` の parser・typed AST・runtime |
| 同じ IR → `ubnfc-lsp --emit-lsp` | 上記の `lsp/` 以下の12クラス。パーサの既存ファイルは変更しない |
| `--emit-tmlanguage` | `syntaxes/stax.tmLanguage.json`（`../lsp/golden/` と照合） |
| Maven shade | `server-dist/ubnfc-stax-lsp-server.jar` |
| `src/extension.ts` + `package.json` +生成サーバ | `target/ubnfc-stax-0.1.0.vsix` |

`server-dist/generation.json` は文法・IR・各生成 Java ファイル・TextMate の SHA-256 と
パッケージの対応を持つ。絶対パスと時刻を入れない。build は作業ディレクトリを root に揃え、
LSP 生成の前後でパーサの全ファイルが byte 一致することも検証する。
生成ソースは手編集・commit しない。追跡するのは文法、build 手順、クライアント、
pom、lockfile、観測結果の golden。jar / VSIX の zip 時刻を含む byte 再現性までは要求しない。

## 自動検証の実測

`../scripts/lsp-protocol.py` は `../lsp/fixtures/` を開き、全応答を
`../lsp/golden/protocol.json` と照合する。絶対 URI の接頭辞だけを `fixture:///` に置換し、
候補・range・メッセージは加工しない。型宣言と局所宣言の同じ行の前に `😀` を置き、
code point から UTF-16 への変換も期待位置と照合する。

| 検証 | 結果 |
|---|---|
| 語・型・局所束縛・明示 import・フィールドの definition | 一意の名前では各1か所。URI と宣言開始位置も照合 |
| 未定義の `missing` | Warning 1件。位置と綴りを照合 |
| 語の宣言・primitive・語の参照の hover | 各規則の `@doc` を表示。利用者の語ごとの説明生成ではない |
| 型位置 / 本体位置の completion | 型位置に `Int` があり `dup` は無い。本体位置はその逆 |
| `.` の直後の不完全入力 | 候補は空。フィールド専用候補を文法注釈だけで保証しない |
| 壊れた文書 → 修正した全文の didChange | Error 診断を返し、修正後は診断ゼロ・definition 復帰 |
| didClose | 診断を空にし、索引を破棄。古い定義を返さない |

現行生成器の限界もテストで固定する:

- `use "library.stax" ( imported ) ;` の参照は、**同じ文書の ImportName** へ跳ぶ。
  実際の library を開いても別ファイルの定義へは跳ばない。ファイルの実在・明示名の
  実在・循環は段階3の checker が検査する。
- 同じ文書の同じ綴りを集める索引なので、別の語にある `let x` が2か所あれば、
  `x` の definition は2候補を返す。別の語の本体の completion にも `x` が残る。
  自分の語名が本体で提案される例もある。parser の語彙 scope の規則とは区別する。
- 2つの struct に同名 `x` があると `.x` の definition は2候補。
  文法だけではスタック上の型から1候補を決められない。重複名でもこの例の scope 診断は空。
- 補完は前半の期待終端と文書内の宣言を使う。現行 LSP は `@catalog(context=…)` を
  文脈ごとの記号絞り込みには使わない。IR の catalog があることと補完精度を同一視しない。
- `: before ( -- ) ;\n: broken ( ` の不完全入力では正常前半の定義索引も空になる。
  全文修正後の復帰は検証済み。失敗位置で切る既存の前半再解析では、この例を保持できない。

局所 scope・catalog context に沿った解決精度は [#49](https://github.com/opaopa6969/ubnfc/issues/49)
へ切り出した。実ファイルへの横断解決や型によるフィールド解決は、その issue の範囲にも含めない。

## 回復配置の評価

`../scripts/lsp-recovery.py` は `mktemp -d` に評価用文法を作り、原文・WordDef 回復・
TopLevel 回復を生成する。本番 `stax.ubnf` は変更しない。結果は `../lsp/golden/recovery.json`。

| 配置 | 正常な struct と main | 壊れた引用の後の after | strict |
|---|---|---|---|
| 回復無し（原文） | 宣言 x / Point / main を保持 | 全文失敗、宣言は rollback | 0 |
| `@recovery(sync=';') WordDef` | 回復が後続候補を横取り。宣言が全て消える | after は保持 | 5、W-RECOVERY-IN-CHOICE |
| `@recovery(sync=';') TopLevel` | 原文と同じ観測を保持、回復ゼロ | after を保持し回復領域を返す | 0 |

壊れた入力は `: broken ( -- ) [ ;\n: after ( -- ) ;\n`。
TopLevel 案では API が ok=true、回復 span=[0,19] / skippedSpan=[0,18] を返す。
しかし同じ IR から生成した LSP は **publishDiagnostics=[]** を返す。
現行 `Diagnostics` が primary 成功時の recoveries を列挙しないためで、回復の無言の成功を
実用上のエラー耐性と数えない。本番で採用する前に、回復領域からエラー診断を出し、
修正・close で消す契約が必要。選択全体の回復自体には現時点で front 拡張は不要である。
回復診断と正常前半の索引維持は [#50](https://github.com/opaopa6969/ubnfc/issues/50) で扱う。

## 実機で試す手順と未確認部分

このマシンでは `which code` が終了コード1、出力無しだった。
生成・build・stdio protocol・TextMate golden・VSIX 内容検査は成功したが、
実機 VS Code の画面・拡張ホストの確認は未実施。手動確認は #48 に切り出した。
README と既存 LICENSE / NOTICE を含む最終 VSIX は、vsce 表示で213ファイル・約1.38 MB。
生成サーバの同梱と設定は `check-vsix.py` で照合した。

```sh
stax_code_work=$(mktemp -d)
code --password-store=basic --user-data-dir "$stax_code_work/user" \
  --extensions-dir "$stax_code_work/extensions" --install-extension \
  examples/stax/vscode/target/ubnfc-stax-0.1.0.vsix
code --password-store=basic --user-data-dir "$stax_code_work/user" \
  --extensions-dir "$stax_code_work/extensions" examples/stax/lsp/fixtures/navigation.stax
```

問題パネルで missing の警告、F12 で語・型・局所名の移動、hover、型/本体位置の補完、
壊した文書を修正後の復帰を確認する。Java は JDK 21 以上。
必要なら `ubnfcStax.java.path` / `ubnfcStax.server.jarPath` / `ubnfcStax.server.jvmArgs` を設定する。
終了後に専用ディレクトリを削除し、版・OS・結果をこの文書へ記録する。
Marketplace 等への公開は含めない。
