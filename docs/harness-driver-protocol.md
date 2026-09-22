# Harness driver protocol

UBNF バックエンドの生成物と手書き目標が共有する、UTF-8 JSON Lines の契約。
runner は生成器を呼ばず、指定済み artifact のコンパイル・実行・比較を行う。
プロトコルの通常応答は、観測済みの `null` と未観測のキー欠落を区別する。

## 起動とビルド

- Rust: artifact 直下の `Cargo.toml` に `driver`、または `<grammar>-driver` bin を提供する。
  `cargo build --release --offline --bins --manifest-path ... --target-dir ... --message-format=json`
  でコンパイルする。依存は事前にローカルキャッシュへ用意する。
  `compiler-artifact` の executable を直接実行するので、ケースごとの cargo 起動はない。
  手動では `cargo run --release --offline --bin driver` も使える。
  `driver` があれば全 grammar を dispatch し、なければ grammar 名を小文字にした
  `<grammar>-driver` に振り分ける。
- Java: `src/main/java/**/*.java` を `javac -encoding UTF-8 -d <classes>` でコンパイルする。
  Maven とネットワークは不要。`java -cp <classes> target.driver.Driver` で実行する。
  手書き目標には JDK 21 以上が必要。`javac`/`java` が PATH に無ければ失敗する。
- 一つの executable につき一プロセスに全ケースを流し、stdin の EOF で終了する。
  stdout は応答 JSON のみ。ログは stderr に出す。各応答行の後に flush する。
  runner は stdin 書込みと stdout/stderr の排出を並行し、pipe 容量による停止を防ぐ。
- CLI/test の一時ファイルは `mktemp -d` の専用ディレクトリに置き、終了時に削除する。
  ライブラリ利用者は `Runner.build_dir` の寿命・削除を管理する。
  Java の classes は毎 build 別ディレクトリで、古い class を再利用しない。

## リクエスト

一行に一 object。改行を含む input は JSON の `\n` escape で渡す。

```json
{"id":0,"grammar":"Operators","entry":"Expression","input":"2/*😀*/+3\n*4","options":{"lexical":true,"scope":true}}
```

- `id`: JSON 値を変更せず返す。runner は同一実行内で一意の非負整数を使う。
  応答順序には依存せず id で対応づける。
- `grammar`: UBNF の `grammar` 宣言名。fixture/variant の表示名ではない。
- `entry`: ルール名、または `null`（`@root`）。任意 entry を生成する backend は
  指定ルールを呼び出す。未対応の文法・entry を root に置き換えずエラー応答する。
- `input`: 解析する文字列。
- `options.lexical`: capture の採取を有効にする。false なら `captures` を省略する。
- `options.scope`: scope の宣言・参照の採取を有効にする。false なら
  `declarations`/`references` を省略する。いずれも解析する文法を変更しない。
  CLI は両方 true。手書き目標の二文法には scope annotation がないため空列を返す。

artifact は一つの文法 variant を実装する。runner の既定は各ケースの先頭 grammar_ref。
`--variant NAME` は artifact が実装する variant を指定する比較用オプションであり、
リクエストの文法を変換する機能ではない。`modes` はその variant の受理期待値に使う。
手書き driver が扱うのは Operators の `normal` と Power の `text`。
別 variant を検証するときは対応する生成 artifact を用意する。

## 応答

```json
{"id":0,"accepted":true,"allConsumed":true,"consumed":1,"matched":1,"ast":{"type":"Number","span":[0,1],"fields":{"value":"1"}},"captures":[{"name":"value","span":[0,1],"text":"1"}],"declarations":[],"references":[],"diagnostics":[],"mappingError":null,"value":1}
```

上記は構造の例であり、Operators の実際のルート AST は Binary。

- `accepted`: 指定 entry の prefix 構文受理。mapping が失敗しても構文成功は true。
- `allConsumed`: 構文受理かつ入力全体を消費したか。prefix 受理とは別の観測。
- `consumed` / `matched`: prefix 解析の二カーソル。単位は Unicode code point。
- `ast`: **全入力解析**の canonical AST、または `null`。全入力不受理なら null。
  node は `{type, span:[start,end], fields}`。葉は文字列・整数・null・配列を持てる。
  node のフィールド、リスト順、型、span は完全一致で比較する。
- `captures`: 全入力解析で採用した出現の順序付き列。各項目は `{name, span, text}`。
  span は半開区間の code point 位置、text は入力のその範囲を正確に切り出す。
- `declarations`: `{name, sourceOffset, ...}` の列。
- `references`: `{name, offset, length, ...}` の列。位置・長さも code point。
- `diagnostics`: 全入力解析で得た `{kind, offset, expected:[...], ...}` の順序付き列。
  parser が成功経路内の失敗を保持する場合もそのまま返す。
- `mappingError`: mapping 例外の文字列、または `null`。通常の構文失敗と分ける。
- `value`: 全入力解析の評価値、または `null`。NaN/Infinity は JSON 数値ではないため
  手書き driver はエラー応答する。文字列や配列の値も利用できる。

手書き API は prefix と全入力を別オプションで実行するため、driver は
prefix の認識と全入力の AST 構築を二回呼び出す。生成 driver は同じ観測を
返せるなら一回の解析でもよい。手書き Java は mapping 例外が持つ syntax 結果を利用する。

処理不能なリクエストは次の一行を返し、後続リクエストを処理する。
不正な JSON から id を読めない場合は null とする。

```json
{"id":0,"error":"未対応の grammar: Unknown"}
```

エラー応答、応答欠落、不正 JSON、未知/重複 id、プロセスの非ゼロ終了は Fail。
プロセス全体のフレーミング異常は、そのプロセスに渡したケースすべてを Fail にする。
ビルド失敗は stderr/stdout と終了状態を持つ `Result::Err` で返し、CLI は失敗終了する。
まだ実行していない runtime 値を空 AST として比較しない。

## 比較と未観測

`runner::compare` は観測を正規化し、既存 `divergence::compare` で Verdict を決める。
元の response は保持する。D-017 は Oracle の承認済み観測群だけに適用し、
他系統の差分や declarations/references の差分を免除しない。

| Expectation | 比較 |
|---|---|
| Ast | 全入力受理、ast/captures/declarations の完全一致、指定 prefix/valueSpans/scope 等 |
| TokenValue | value の存在で全入力受理を決定。明示 null は成功値。value と prefix の三観測 |
| RuleValue | `accepted_in(variant)` による全入力受理、受理時の value、指定 texts/nodes/expected/prefix |
| Accept | 全入力受理、prefix、span、fields、指定 value。span/fields は canonical AST からも参照 |
| Diagnostics | 最初の診断の kind と backend ごとの javaOffset/rustOffset |
| Numeric | 全入力受理。Rust は rust_lexeme、Java は java_int と mappingError。java_int=null は mappingError 文字列を期待 |
| Oracle | 型付き全フィールド、および raw に残る references/scopeDepth/nativeFailure/parsedStatus 等 |
| GenerationFailure | build 成功後の応答は生成成否の差分。通常の runtime 差分とは混ぜない |
| ScopeOps/Catalog/Raw | driver にない操作結果・静的 metadata・期待値自体がない場合を未観測で報告 |

評価値だけは、表現の違いである安全な範囲の `1` と `1.0` を同値にする。
AST 内の整数/文字列などは型を保ち、入力から期待値を計算しない。

返らなかった観測は未観測として数え、Fail にしない。明示 null、空配列、
観測された AST 内のフィールド欠落、capture の要素不足は通常の比較対象。
Oracle capture の `binding` や診断の `origin` 等、追加の metadata が driver に
ない場合は要素ごとの未観測項目として数える。backend はこれらも返せば比較範囲を広げられる。
`texts`/`nodes`/`valueSpans`/`scope`/`parentRollback` 等も必要なら応答の追加キーで返す。

Report の `pass`/`known_divergence`/`fail` はケース数。
`unobserved` は未観測項目数、`unobserved_cases` は該当ケース数で、他の三分類と重複する。
観測が全て欠落した場合も Pass + 未観測となるため、Pass だけで完全検証とは判断しない。
JSON は差分の JSON Pointer・期待値・実測値と未観測パスを保持し、Markdown もそれらを表示する。

## CLI と手書き目標の結果

```sh
cargo run -p ubnfc-harness --offline -- run --backend rust \
  --artifact examples/handwritten-target/rust \
  --families associative,right-associative --report out.md
cargo run -p ubnfc-harness --offline -- run --backend java \
  --artifact examples/handwritten-target/java \
  --families associative,right-associative --report out.json
```

`--root` は unlaxer-parser の読み込み元を指定する（書き込みはしない）。
既定は loader の `UNLAXER_PARSER_DIR`/sibling repo。
`--families` は fixture ディレクトリ名、または6系統の kebab-case 名をカンマ区切りで指定。
report の拡張子が `.json` なら JSON、それ以外は Markdown。stdout は常に Markdown。
Fail が一つでもあれば report を書いた上で非ゼロ終了する。未知の系統/variant はエラー。

| 実行 | 系統 | Pass | Known | Fail | 未観測項目 |
|---|---|---:|---:|---:|---:|
| Rust CLI | associative | 24 | 0 | 0 | 0 |
| Rust CLI | right-associative | 28 | 0 | 0 | 0 |
| Java CLI | associative | 23 | 0 | 1 | 0 |
| Java CLI | right-associative | 28 | 0 | 0 | 0 |
| Java テストの限定除外後 | associative | 24 | 0 | 0 | 1 |

Java の一件は `1e2/4-0.5`。手書き目標は `(NUMBER) @value` を int に変換するため、
構文は全入力受理するが mapping 例外となり、value が期待値 `24.5` に対して null になる。
D-017 に登録せず、統合テストだけでこの一件の value 観測を除外する。
除外前の Fail も検証し、AST/captures/診断は既存 golden と比較する。
parser 本体・golden を補正せず、バックエンドの正しい mapping 実装を妨げない。

検証: `cargo test -p ubnfc-harness`、`cargo clippy -p ubnfc-harness -- -D warnings`、`cargo fmt`。


## 追記（2026-09-22、Java backend 接続後）

- 「runner は生成器を呼ばない」は `--artifact`（Prebuilt）経路の記述。`--ir` 経路（`Artifact::FromIr`）では
  runner が `ubnfc-java` の classes/jar（`cd ubnfc-java && mvn -o -q -DskipTests package` 済み）で IR → 生成 → `javac` を行う。
- 生成 driver の簡易評価（`value`）は、IR に operator 指定がある `left/op/right` 形 AST に対する数値演算 `+ - * / ^` のみ。
  それ以外は `null`（未観測ではなく「評価なし」）。
- 生成 Java での実測: associative 24/24、right-associative 28/28、tinycalc 8/8（Known 0 / Fail 0 / 未観測 0）。

## 追記（2026-09-22、Rust backend 接続後）

- Rust の `--ir` 経路（`Artifact::FromIr{backend: Rust}`）: runner が `ubnfc-rust` バイナリ（隣接 `target/` または `UBNFC_RUST_BIN`）で
  生成 → 一時 crate を `cargo build --release --offline` → `driver` 実行。生成物 hash ごとにビルド結果をキャッシュし、
  別文法のビルドで実行ファイルが上書きされない。
- 生成 Rust driver は既定で生成され依存ゼロ（`json` feature 時のみ serde optional）。`value` は Java driver と同じ範囲
  （operator 指定のある `left/op/right` AST の数値 fold `+ - * / ^`、他は null）。JSON の数値は字句のまま保持（`id` の丸め回避）。
- `ubnfc gen --grammar X.ubnf --target rust|java --out DIR` は front から各生成器を subprocess で呼ぶ
  （`UBNFC_RUST_BIN` / `UBNFC_JAVA_JAR` で上書き、見つからなければ探索パスを表示して exit 3）。
- 生成 Rust での実測: associative 24/24、right-associative 28/28、tinycalc 8/8。
