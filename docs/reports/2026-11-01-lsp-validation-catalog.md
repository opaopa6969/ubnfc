# LSP の文法意味検査と外部カタログ照合（issue #26）

## 完了範囲と設計判断

段階 3 §5.2 に残した GrammarValidator 相当の検査を、既存 front の
`ubnfc validate --format json` を呼ぶ生成 `FrontValidation.java` で提供する（D-094）。
外部 `.tecatalog` は生成 `CatalogDiagnostics.java` が `ParseResult.catalogs()` の capture と照合する（D-095）。
生成される LSP ソースは 12 → 14 ファイルになる。

**LSP / DAP は Java 側だけの opt-in feature である。Rust backend に LSP / DAP はない。**
Rust 製 front を検査プロセスとして使うことと、Rust backend で LSP を生成することは別である。
生成パーサ、Java の AST 構築、front の検査規則、外部 fixture の pin は変更しない。
本件は #26 の二項目を完了する。表示・編集 capability は #25、DAP の要求追加は #27、
実機 VS Code 確認は #28 / #48 の管理範囲である。

## GrammarValidator 相当の検査と front の対応

検査項目の正本は既存 `crates/ubnfc-front/src/validate/` と resolve / syntax である。
段階 3 で旧 LSP との差だった 34 件の分類も、この呼出し境界を通る。
下表は検査の担当箇所を示す。LSP はコード・severity・message・hint を変換して返し、
個々の規則を Java に複製しない。診断の英語本文は既存 front の契約を保持する。

| 検査項目 | 既存 front の担当 | 代表的なコード / 差の扱い |
|---|---|---|
| 未定義規則・token、重複宣言、import の解決 | `resolve/`、`pipeline.rs` | `E-RULE-UNDEFINED`、import 解決診断。失敗時も CLI の JSON 診断を返す |
| root 未指定、未解決 host token | `validate/general.rs` | `W-GENERAL-NO-ROOT`、`W-TOKEN-UNRESOLVED`。host クラスを LSP の JVM にロードして推測しない |
| global / rule の whitespace、memoSafeToken、token 名衝突・文字範囲、回数境界 | `validate/general.rs`、`annotations.rs`、`boundary.rs` | `E-WHITESPACE-*`、`E-MEMO-*`、`E-RULE-TOKEN-NAME-COLLISION`、`E-TOKEN-*`、`E-BOUNDED-*` |
| mapping の capture / params、commonField、enum、eval の型 | `validate/mapping.rs` | `E-MAPPING-*`、`E-COMMONFIELD-MISMATCH`、`E-ENUM-NON-TERMINAL`、`E-EVAL-SUM-TYPE` |
| scope、declares / backref、catalog の capture、typeof、重複・未知 annotation | `validate/annotations.rs` | `E-ANNOTATION-*`、`E-TYPEOF-*`、`W-ANNOTATION-*` |
| 結合性、precedence、predictive / longest choice、左再帰 | `validate/operators.rs` | `E-ASSOC-*`、`E-PRECEDENCE-*`、`E-RIGHTASSOC-*`、`E-PREDICTIVE-*`、`E-LONGEST-CHOICE`、`W-LEFT-RECURSION` |
| 負の precedence や不正な構文 | `syntax/` → CLI の `E-SYNTAX` | front は非負整数を構文として読むため、負数を独立の `E-PRECEDENCE-NEGATIVE` としては返さない（既存 front の契約） |

生成時の IR は「サーバが解析する言語」の IR であり、編集中の `.ubnf` の意味検査結果ではない。
IR の表引きでは未保存の文法を検査できず、Java への再実装は resolve / validate の二重管理になる。
そのため実行時に同じ front を呼ぶ、最小で取り消しやすい境界を選んだ。
front が今後追加する JSON 診断もコードごとの Java 修正なしで通る。
最新 Classic の全検査を再実装したという意味ではなく、ここでの対応範囲は既存 front の検査契約である。

### 起動・更新・位置・失敗

UBNF から生成したサーバを次のように起動する。`--validator` は実行ファイルの絶対パスで、
省略時にはプロセスも追加診断も発生しない。UBNF 以外のサーバでの指定は明示エラーにする。

```sh
java -jar ubnfc-ubnf-lsp-server.jar --validator /absolute/path/to/ubnfc
```

実行ファイルを文書や LSP 設定から選ばせず、起動引数または
`LanguageServerImpl(scanners, validatorPath)` の合成で指定する。
shell は介さず `ProcessBuilder` の引数配列で起動し、stdout は JSON 専用に読む。
検査ごとに専用の一時ディレクトリへ本文を書き、処理後に削除する。元文書には書き込まない。
相対 import は UBNF の lexical trace にある `ImportDecl` の STRING 終端だけを
元文書の親からの絶対パスへ写す。文字列リテラルやコメントを一括置換せず、
文書の周辺ディレクトリも複製しない。絶対 import は維持する。
`pkg:` は front の公開仮想パス `<元文書の親>/.ubnf-packages/ID` へ写し、
元文書の隣にある manifest / lock / cache の照合を front に委ねる。パッケージを取得し直さない。
import 先はディスク上の内容を読み、他の未保存バッファを重ねる機能は持たない。
非 file URI は一時文書として扱い、相対 import の基準ディレクトリは提供しない。

`didOpen` / `didChange` では現在の全文、`didSave` / 設定変更 / ファイル変更通知では
保持中の全文を検査する。実行は同期で、プロセスの待機上限は 10 秒、JSON 応答は 1 MiB 以下とする。
`textDocumentSync` は Full / openClose に加えて save（本文再送なし）を明示し、保存通知を要求する。
設定変更・ファイル変更通知では開いている全対象文書を URI 順で再通知する。
クライアントが監視通知を送らない場合も、文書の変更・保存で再検査できる。

front の公開 JSON は `grammar` / `rule` を持つが span を持たない。
同名規則や import 先の位置を推測せず、**意味診断は現在の文書の `0:0–0:0` に集約する**。
`source=ubnfc-front`、コードと Error / Warning、本文と hint を保持する。
生成パーサの構文診断は従来の UTF-16 範囲と `source=ubnfc` のまま同じ通知に並ぶ。
両パーサの受理範囲は同一ではない。例えば front は UBNF の block comment を拒否するため、
その場合の `E-SYNTAX` も隠さない。golden の UBNF fixture は front が受理する行コメントを使う。

| 失敗 | 診断コード | 処理 |
|---|---|---|
| 意味エラーによる終了値 1 | front が返したコード | 正常な JSON 応答として読む |
| 起動不能 / 不正 JSON | `front-unavailable` | 構文診断と一緒に Error を返す |
| I/O 等の異常終了 / 応答過大 | `front-failed` | 同上 |
| 10 秒を超過 | `front-timeout` | プロセスを終了し Error を返す |
| スレッド割込み | `front-cancelled` | 割込みを保持して Error を返す |

## 外部 `.tecatalog` の契約

照合元は `initializationOptions` または `workspace/didChangeConfiguration.settings` の
`catalogs` で指定する。相対パスや暗黙の workspace 探索、開発者 tree の既定値は採用しない。
設定変更は表全体を差し替え、空表または未指定なら無効にする。

```json
{
  "catalogs": {
    "variable": "/absolute/path/variables.tecatalog",
    "names": {"path": "/absolute/path/names.tecatalog", "context": "names"}
  }
}
```

キーは IR の `@catalog(context=...)` の用途名である。
ファイルの行にある context（FA / NIM などの分類）とは分ける。
パス文字列だけなら全分類を照合対象とし、`{path, context}` なら該当分類と分類なしの行を使う。
1 用途に明示した 1 ファイルを読む形に絞り、ディレクトリ列挙や文法にない業務上の既定値を足さない。

形式は UTF-8 の canonical `tinyexpression-catalog-v1`。空行・先頭 `#` の行は無視する。
照合方式は既存カタログにある `exact` と `prefixWithSuffix` を扱う。
フィールドの区切りは `|`、説明文は診断の判定には使わない。

```text
tinyexpression-catalog-v1
exact|alpha|名前の説明|names
prefixWithSuffix|item|_|1|接尾辞を持つ名前|names
```

`exact|名前|説明[|分類]` は capture の本文を完全一致で照合する。
`prefixWithSuffix|名前|区切り|最小接尾辞長|説明|分類` は名前と区切りで始まることと接尾辞長を検査する。
接尾辞長は既存 Java カタログの契約と同じ UTF-16 長で数える。負数・非整数は不正とする。
capture の外側の空白は除き、`$` の除去・引用符の解除・大文字小文字の同一視は推測で加えない。

照合が設定されたときだけ `Analysis.catalogs()` が capture 出現表を要求して再解析する。
通常の診断経路は従来の認識モードを保つ。構文不正では D-062 の成功した前半だけを使い、
`catalog-prefix` と「構文エラーより前の範囲だけを見た結果」の注記を付ける。
前半も解析できなければ値の診断を出さない。

| 状態 | 診断コード / 範囲 | 振る舞い |
|---|---|---|
| 既知の値 | なし | 構文・scope 診断は従来どおり |
| 未知の値 | `catalog-unknown` / capture の UTF-16 範囲 | Warning。前半なら `catalog-prefix` |
| ファイル未発見 | `catalog-missing` / `0:0` | Warning。空カタログと見なして大量の未知値を出さない |
| 不正 header・行・照合方式・過大ファイル | `catalog-invalid` / `0:0` | Warning。同上 |
| 入出力・UTF-8 読込失敗 | `catalog-io` / `0:0` | Warning。同上 |
| 不正な設定 | `catalog-config` / `0:0` | Warning。古い設定を使い続けない |
| 更新・削除 | 次の検査で再読込 | 内容のキャッシュや古い成功結果への fallback はない |

全カタログ診断の `source` は `ubnfc-catalog`。ファイルごとに 1 MiB を上限とし、
不正行が一つでもあれば部分読込結果を採用しない。
fixture は `ubnfc-lsp/src/test/resources/validation/` の自作文法・IR・カタログに閉じる。
外部 repo のカタログをテスト時に読む必要はない。

## 検証と再現

`ValidationGoldenTest` は生成した UBNF / 小文法の parser と LSP を JDK 21 でコンパイルし、
`ValidationScenario` の実 lsp4j launcher 間で JSON-RPC を往復する。
front の正常系は模擬応答ではなく実際の Rust CLI を使う。既定は stable で CLI をビルドし、
既に組んだ CLI は `-Dubnfc.validation.command=/absolute/path/to/ubnfc` で指定できる。

| golden | 固定する観測 |
|---|---|
| `front.json` | 正常・root 警告・未定義参照・mapping・左再帰・BMP 外文字・構文診断との共存・文書変更後の解消・close |
| `imports.json` | `../` import の解決と import 先の更新通知による意味診断 |
| `package.json` | `pkg:std/layout` の manifest / lock を元文書の隣から解決 |
| `front-unavailable.json` | 実行ファイル未発見の明示エラー |
| `front-errors.json` | 不正 JSON・異常終了・10 秒 timeout の明示エラー |
| `catalog.json` | 完全一致・prefix・分類フィルタ・未知値・UTF-16 座標・正常更新・不正内容・未発見・無効化・不正設定・構文不正の前半・修復・close |

一時文書の URI だけを固定 URI に正規化する。診断コード・文言・severity・位置は実応答を比較する。
未保存文書が元ファイルを書き換えないこと、無指定で追加検査しないこと、
UBNF 以外の validator 指定の拒否も assert する。
`ParserUnchangedTest` は LSP 追加前後の全パーサ、DAP 追加前後の全パーサ / LSP の
byte 一致、feature off で出力しないこと、lsp4j 非依存、全テンプレート置換を引き続き検査する。
既存の段階 5 の四つの golden も変更しない。

再現手順（Java は sdkman の JDK / Maven、Rust は stable）:

```sh
cd ubnfc-java && mvn -q -DskipTests package
cd ..
mvn -q -Plsp -pl ubnfc-lsp -am test '-Dtest=org.unlaxer.ubnfc.lsp.*Test' -Dsurefire.failIfNoSpecifiedTests=false
d=$(mktemp -d)
eval "$(scripts/fetch-external-fixtures.sh "$d" | sed 's/^/export /')"
RUSTUP_TOOLCHAIN=stable cargo test --workspace --no-fail-fast
```

worktree では sibling を既定値で探す Rust テストもあるため、ローカル全体検証は
専用一時ディレクトリへ現作業ソースと Java jar を複製し、その中だけに CI と同じ
固定 fixture の sibling link を置いて行う。front の source ID は Git ルートの有無で変わるため、
コピーにも `.git` のルート印を置く。Java の手書き fixture の `src/main/java/target/` は
ビルド成果物ではなく追跡済みソースなので、コピーの除外対象にしない。
共有ディレクトリにはリンクを作らない。
CI の既存 LSP ステップはパッケージの全テストを選ぶため、本テストも自動的に実行する。
VSIX の実機確認はこのマシンに `code` がなく実施しない。

ローカルの結果:

- CI と同じ Java jar package: 成功。Java backend 全11,885テストも失敗・エラー・skip 0。
- CI と同じ LSP パッケージ全テスト: 18テスト、失敗・エラー・skip 0。golden 更新後の通常照合で確認。
- 既存 P4 サーバの LSP / DAP: 30テスト成功。UBNF サーバ: 下記の既存 golden 補正後に通常照合で21テスト成功。
- CI と同じ JSON driver / P4 library / selfhost library の WASM release build: すべて成功。
- 帰属レポートの shell 構文検査と Python 3テスト: 成功。

既存 UBNF golden にはコミット済み IR とのずれがあった。変更前の `origin/master`（`1a51aad`）を
専用一時領域へ `git archive` で展開し、旧生成器でも同じ6 fixture ファイルの比較が失敗することを確認した。
新旧の全生成パーサ・DAP ソース・`Diagnostics.java` は byte 一致した。
baseline の実応答から golden を採取し、補完候補と期待集合の11行だけを補正したうえで、
本変更の生成サーバの通常 golden がすべて一致することを確認した。
本機能による既存観測の変更として採用したものではない。

`cargo test --workspace --no-fail-fast` の初回には、一時コピーの Git ルート印の不足で
`pipeline_golden` の3件、追跡済み `src/main/java/target/` を除外したことで `runner` の2件が失敗した。
配置を直し、この5件を含む2テスト群を同じ pin で再実行して計21テスト成功を確認した。
Rust のソースや期待値は変更していない。CI での workspace 全体検証の最終結果は
本 PR の checks と issue #26 の完了コメントに記録する。

配置を揃えて Java package から同じ順序で全体を再実行し、602成功・失敗0・skip0・終了値0を確認した。
最初の head の native CI も23分35秒で成功した。その待機中に stax 段階4（PR #51、
master `bdd87bd`）が merge されたため取り込んだ。stax の protocol 検査は `textDocumentSync` の
整数 `1` を前提としていたので、Full 同期を検査しつつ openClose / save の明示通知も検査する形へ追従した。
既存の protocol golden も同期の通知形式だけを更新し、言語の振る舞いは変更しない。
その後、増分再解析設計の PR #18（master `06fa27b`）も取り込んだ。master の D-093 を保持し、今回の追加行だけを D-094 / D-095 へ振り直した。
統合後は LSP 全18テスト、stax の protocol / 回復検査と Cargo 側の統合テスト1件が成功した。

最新 head の native CI では fuzz campaign の4件が `build:rust` の discard で失敗した。
同じ head のローカル再実行は8件すべて成功し、生成 Rust と campaign の実装に差分はない。
CI は seed と Cargo / rustc のエラー本文を保持しておらず、正確な原因は確定できない。
観測情報を保持する改善は受け入れ条件付きの [issue #54](https://github.com/opaopa6969/ubnfc/issues/54)
へ分離した。診断条件を弱めず、最新 master を含む head の CI を通す。最終結果は PR checks と
issue #26 の完了コメントを参照する。

## 引継ぎと取り消し

#25 では `LanguageServerImpl` の capability と要求ハンドラを追加する際、
診断の合流点 `diagnostics(Analysis)` と保存・設定・監視通知の再検査を保持する。
front は span を返さず、catalog は capture の範囲を返すという違いも維持する。
#27 の DAP は `Diagnostics.of(Analysis)` を従来どおり直接使い、
この LSP 起動時設定を DAP の意味診断や評価器へ暗黙に引き継がない。
取り消す場合は本 PR の merge commit を revert する。
