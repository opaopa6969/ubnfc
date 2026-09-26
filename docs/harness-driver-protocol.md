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
- `options.scan`（任意、Rust 生成 driver と Java 生成 driver。Java は D-080 から）: D-072 の一括走査（`plan.scans`）を
  切り替える **A/B 用の穴**。このキーがあるときは、走査が効く条件そのもので測るために
  **fast mode** で解析する（`diagnostics: false`、`lexical` は false 扱い）。したがって
  `hints` は「fast mode が失敗して診断モードへ上がったとき」だけ出る。キーが無ければ
  従来どおり診断ありで解析し、走査経路には入らない。**同じキーを true / false で 2 度
  送った応答は完全に一致しなければならない**（差分ファジングの in-backend A/B）。
  Java driver は同じ意味で受ける（`Session.scanning` をこの要求だけ切り替え、
  `ParseOptions.withObservations(false, false)` と `lexical: false` で解析する。キーが無い要求は
  起動時の `-Dubnfc.scan` の値へ戻す）。他の backend はこのキーを無視してよい。

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
| Numeric | 全入力受理。両 backend とも rust_lexeme（入力字句）と `mappingError=null`（D-074。java_int は旧 Java の観測として残るが比較しない） |
| Oracle | 型付き全フィールド、および raw に残る references/scopeDepth/parsedStatus 等（nativeFailure は D-019 により情報保存のみ） |
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

## 追記（2026-09-22、レビュー修正後の応答拡張）

- `options.scope=true` のとき `scope` は `{ mode: [...], events: [ {order, action: "enter"|"leave"|"declare"|"reference", mode, name, offset, length} ], declarations, references, diagnostics }`。
  位置・長さは code point。
- 診断は主診断の `offset`/`expected` に加えて `farthestOffset` / `farthestExpected` を持つ（primary と farthest の分離。
  全入力要求で末尾入力が残る場合、主診断は `trailing_input`）。
- `ParseOptions.maxDepth`（既定 10,000）超過は `syntax` 失敗として返す（両バックエンド）。


## Extern scanner の登録（round3）

D-019（`docs/design/03-design-v1.md` §8）に従い、`nativeFailure` は期待値・実測値の
report JSON に保持するが、Verdict と未観測の判定から除く。公開 `diagnostics` の
主 offset、farthest、hints、deepest、kind の比較は維持する。

生成 driver は IR の `originalClassName` と `logicalId` の対応を同梱する。
registry は **完全な元クラス名**から scanner を引き、driver が論理 ID に結び直す。
短名は登録した alias だけを認める。他 package の同名クラスには一致させない。
未知クラスは登録エラーとして報告し、通常の構文拒否に置き換えない。

Java は `--scanners <classes-dir|jar>` または `UBNFC_SCANNERS` を使用する。
引数が環境変数に優先する。指定先に `ubnfc.scanners.Registry` を置き、公開 static
`scanners()` が `Map<String, TokenScanner>` を返す。scanner は対象生成物と同じ
package の `api.TokenScanner` / `rt.Input` に対してコンパイルする。
`scanners/java/Registry.java` の `generated.` は生成 package に置換するための印である。
生成 package が異なる jar の使い回しはしない。追加の Maven 依存はない。

```sh
java -cp generated-classes target.driver.Driver --scanners scanner-classes
java -cp generated-classes target.driver.Driver --scanners scanners.jar
UBNFC_SCANNERS=scanners.jar java -cp generated-classes target.driver.Driver
```

Rust は生成 crate の `src/bin/scanners.rs` に registry を同梱し、`scanners` feature で
選択する。実行時の動的 crate ロードは行わない。module は
`Registry::new(&'static [(&'static str, &'static str)]) -> Result<Registry, &'static str>` と
`TokenScanner` 実装を提供する。同梱集合を使う場合は `scanners/rust/scanners.rs` と
`scanners_tests.rs` を `src/bin/` にコピーする。

```sh
cargo build --offline --features scanners --bin driver
cargo test --offline --features scanners --bin driver
```

Rust の既存公開 entry API には scanner 引数がないため、driver 生成フックが同じ
parser 生成関数から driver 内の parser を作り、scanner を受け取る entry を追加する。
元の grammar/root/entry 分岐を再利用し、root 以外の entry にも登録を適用する。
公開 parser API と runtime テンプレートは変更しない。feature 無効時のコードは
コンパイル対象外になる。Extern のある文法で feature を無効にすると未登録エラーになる。

`Artifact::FromIr` は両言語の同梱集合を自動配置する。Java は registry を生成物と
一緒に javac し、その classes を `--scanners` で渡す。Rust はコピー後に default
feature を有効にする。run-all の一括 build も同じ準備関数を使う。
Rust のキャッシュ key には scanner ソース・manifest・テストも含まれ、変更後の
実行ファイルを以前のキャッシュと混同しない。既存 Prebuilt のソースは書き換えない。

同梱 scanner はすべて `PURE = true` と宣言し、scope や外部状態を読まず、効果を返さない。
memo の可否は引き続き入力 IR に従う。位置は Java が UTF-16、Rust が UTF-8 byte。
`consumed` / `matched` / mode / invert / reset の5状態を受け取り、失敗時は入口の
cursor を返す。value span は消費範囲と独立し、single quote の内側、フェンスの改行前、
Extern probe の先読み前を表現する。

| 元クラス（`org.unlaxer.` 以下） | 受理範囲 | 根拠 |
|---|---|---|
| `parser.elementary.DoubleQuotedParser` | double quote、任意1 code point の escape、改行・NUL 可 | `unlaxer-common/src/main/java/org/unlaxer/parser/elementary/DoubleQuotedParser.java:10`、`QuotedParser.java:55`、`EscapeInQuotedParser.java:22` |
| `parser.elementary.SingleQuotedParser` | single quote、同じ escape。value は引用符の内側 | 同ディレクトリ `SingleQuotedParser.java:7`、`QuotedParser.java:55` |
| `tinyexpression.parser.StringLiteralParser` | 上記2種の選択 | `tinyexpression/src/main/java/org/unlaxer/tinyexpression/parser/StringLiteralParser.java:19` |
| `tinyexpression.parser.javalang.CodeStartParser` | 行頭、3 backtick、ASCII identifier、colon、dot 区切り ASCII identifier、CRLF/CR/LF/EOF | `tinyexpression/src/main/java/org/unlaxer/tinyexpression/parser/javalang/CodeStartParser.java:32`、`parser/javatype/JavaClassNameParser.java:28` |
| `tinyexpression.parser.javalang.CodeEndParser` | 行頭、3 backtick、CRLF/CR/LF/EOF | 同ディレクトリ `CodeEndParser.java:21` |
| `oracle.ExternalCursorParser`（完全名） | 😀 を消費し、続く x を先読み | `corpus/oracle/tools/ExternalCursorParser.java:8` |

`unlaxer-common` の根拠は隣接 `unlaxer-parser` checkout、tinyexpression の根拠は
同名の隣接 checkout にある。いずれも読み取り専用で参照した。
共通補助の根拠は `unlaxer-common/src/main/java/org/unlaxer/parser/` 以下の
`clang/IdentifierParser.java:24`、`elementary/StartOfLineParser.java:38`、
`elementary/LineTerminatorParser.java:20`。scanner は whitespace/comment を暗黙に飛ばさない。

現在の mapper には、Extern の value span より消費範囲を優先する経路があり、
tiny-string-token の single quote 内容抽出は残る Fail として報告する。
scanner 自体の value span は両言語の契約テストで検証し、登録 E2E は corpus の
`direct-eof` 文法の double quote 入力と拒否入力を使う。期待値・oracle・乖離登録は変更しない。

### Extern scanner の先頭文字宣言（M5 round 3）

front の解析は Extern token の先頭文字集合（`mayStart`）を unknown として扱い、backend は
Extern で始まる候補を候補除外（設計 v1 §3.4）の対象にしない。scanner registry は
`scanners/first-chars.json` で、元クラス名（`originalClassName`、または registry が認める
alias）ごとに「成功し得る先頭 code point の集合」を宣言できる。

```json
{"scanners": {"org.unlaxer.tinyexpression.parser.StringLiteralParser": ["'", "\""],
              "oracle.ExternalCursorParser": ["😀"],
              "some.UnknownScanner": null}}
```

要素は 1 code point の文字列か整数、または `{"start": "0", "end": "9"}` の閉区間。`null` は
unknown（宣言しないのと同じ）。front は `ubnfc ir|gen --extern-first-chars FILE` で読み込み、
`TokenSpec.extern.firstChars`（省略可・昇順・互いに素な区間）へ写す。宣言の無いクラスは
警告を出して unknown のまま残す。`ir/fixtures` の golden はこのファイルを渡して生成する
（`crates/ubnfc-front/tests/pipeline_golden.rs`）。harness の run-all は渡さない
（corpus の verdict は宣言に依存しない）。

**UBNF v2 では宣言が必須になる（D-042）。** v2 の token 宣言は host 実装を
`token X = extern '<id>'` と明示し、`'<id>'` は**この registry が解決する ID** である
（FQN でなくてよい。短い ID の別名を FQN と並べて登録する）。`@ubnf: 2` の文法に
`--extern-first-chars` を渡したとき、宣言の無い Extern があれば `ubnfc ir|gen` は
`E-TOKEN-EXTERN-UNREGISTERED` を出して **exit 1** で止まる。v1 は凍結なので従来どおり
警告のみ。詳細は `docs/reports/2026-09-23-ubnf-v2-token-vocabulary.md` §3。

宣言は必要条件で、健全性は scanner 実装の責任である: 集合外の先頭文字（Extern を呼ぶ位置の
文字。同梱集合は `match_only && reset` の STRING を含め、消費 cursor の位置）では scanner が
消費なしで失敗しなければならない。backend（Rust）はこの宣言を候補除外の判定にだけ使い、
除外した候補の失敗再生では Extern の葉を本物の scanner 呼出しで再生するので、失敗診断
（scanner が返す表示文字列）は宣言の有無で変わらない。同梱集合の宣言:
DoubleQuoted `"`、SingleQuoted `'`、StringLiteral `'` `"`、CodeStart / CodeEnd `` ` ``
（行頭でなければ失敗するが、先頭文字は必ず backtick）、ExternalCursor `😀`。
`examples/p4-rust/tests/fixtures.rs` の `extern_first_chars_declaration_matches_bundled_scanners`
が、宣言外の ASCII 文字で各 scanner が消費なしに失敗することを確認する。
front の `analysis.mayStart`（両 backend の predict 表）は変えない。変えると P4 の plan と
Java 生成物が同時に動くため、別 issue とする。

## round5: prefix capture と内部診断情報

Java/Rust とも `options.lexical=true` の応答で `prefixCaptures` を返す。
各要素は `name / binding / span / text`。`span` は Unicode code point の半開区間で、
`text` は引用符・trivia を含む元入力の当該区間。完了順と重複を保持する。
全入力の `captures` と、prefix 解析の `prefixCaptures` は独立して返す。
根拠: `OracleCollector.java:171-174`、`SemanticsTest.java:143-151`。

D-020 により `rulePath` / `deepestRule` は raw 応答・レポートに保持するが、
差・欠落を Verdict / 未観測に使わない。`kind / offset / farthestOffset /
farthestExpected` の比較は維持する（D-014）。Extern の失敗候補は scanner が返す
表示文字列を用い、scanner の Java クラス名や capture wrapper を追加しない。

## 観測契約の統一（2026-09-23、D-024 〜 D-029）

差分 fuzzing（`docs/reports/2026-09-23-differential-fuzzing.md`）が、二つの driver が別々の形で
同じ観測を出していること、および「契約が決まっていない」項目を露出させた。以下を正とする。
決定の根拠は `docs/design/03-design-v1.md` §8 の D-024 〜 D-029。

### 二つの解析と、どの観測がどちらに属するか（D-024）

生成 driver も手書き driver も、1 リクエストにつき **prefix 解析**（`requireAll=false` /
`require_eof=false`）と**全入力解析**（`requireAll=true` / `require_eof=true`）を実行する。

| 観測 | 由来 |
|---|---|
| `accepted` / `consumed` / `matched` / `prefixCaptures` / `parsedStatus` / `nativeFailure` | prefix 解析 |
| `allConsumed` / `fullConsumed` / `fullMatched` / `ast` / `captures` / `lexical` / `diagnostics` / `mappingError` / `value` / `recoveries` / `scope` / `declarations` / `references` / `scopeDepth` / `texts` / `nodes` / `valueSpans` | 全入力解析 |

- `allConsumed` は **全入力解析が構文成功したか**。`consumed` は別の解析のカーソルなので
  `allConsumed == (consumed == 入力長)` は成り立たない（成り立たせる契約でもない）。
- `fullConsumed` / `fullMatched` は全入力解析の二カーソル（code point）。
  成り立つのは `allConsumed ⇒ fullConsumed == 入力長` と `allConsumed ⇒ accepted`。
- 全入力解析が成功したのに prefix 解析が手前で止まることはある（`A ::= 'a' | 'ab'` に `"ab"`）。

### `recoveries`（D-025）

キーは `recoveries`。要素は
`{ruleId, mode, span, syncPattern, skippedSpan, syncSpan, captureOccurrences}`。
位置は code point の半開区間。`syncPattern` は同期文字列、見つからなければ `null`。
`skippedSpan` は `mode=="SKIP"` または `syncSpan==null` なら `span`、それ以外は
`[span[0], min(span[1], syncSpan[0])]`。

`captureOccurrences` は `captures` の添字のうち、**回復領域を自分の値としてそのまま持つ**
出現だけを並べたもの（D-069）。回復と capture の間に AST ノードを作る（または捨てる）規則が
入ると、その外側の capture の値はノードであって回復領域ではないので数えない。
規則が透過（`@mapping` が無い / `transparent` / `text`）なら、その外側の capture も数える。

`kind: "recovery"` の診断は、回復の引き金になった失敗を表す。`expected` /
`farthestExpected` は `farthestOffset` と同じ**公開（表示）語彙**で（D-027）、
範囲は**回復した規則の解析**に限る（外側の、回復が位置を進めた後の失敗は入らない）。
回復つき規則が自分の開始位置で失敗したときは、その規則の同期文字列も候補に載る。

### `scope`（D-025）

`options.scope=true` のとき、`scope` は
`{depth, mode, events, declarations, references, diagnostics}` を持つ。
`declarations` / `references` / `scopeDepth` は top-level にも同じ値を置く（既存互換）。

- `mode`: **文法が宣言した scope mode**（`lexical` / `dynamic` / `sameRuleText`）の宣言順・重複除去。
  入力に依存しない静的な値で、観測されたイベントの mode ではない。
- `events`: `{order, action, mode, name, offset, length}`。`action` は
  `enter` / `leave` / `declare` / `reference` / `clear` / `error`。
  `name` は `declare` / `reference` では対象シンボル名、`enter` / `leave` / `clear` では
  その scope を持つ**規則名**（D-029）。
- `diagnostics`: `{message, offset, length, severity}` の列。
- trim（U+0000〜U+0020）後に空になった記号名の `declare` / `reference` は記録しない（D-029）。

### `catalogs`（D-025）

`@catalog` の静的表を**両 driver が返す**。要素は `{rule, context, captures}` で、
規則の宣言順・context は初出順・`captures` は重複除去してソート。入力に依存しない。

### `diagnostics` と `hints`（D-026）

- 全入力解析が構文成功した応答は `syntax` / `trailing_input` 診断を**含まない**。
  成功経路内の投機的失敗は `hints` の領分（D-023）。
- recovery / scope / mapping / resource 由来の診断は成功時も残る。
- `hints` は言語別の情報出力で、Verdict にも未観測の判定にも使わない（D-019 / D-020 と同じ）。
- `farthestOffset` / `farthestExpected` は共有契約のまま（D-014 / D-027）。ただし文法に現れない
  合成 parser の表示名（`Repeat` / `OneOrMore` / `Bounded` / `Optional` / `__CaptureSite` /
  `<Rule>…Parser`）は言語別で、corpus の Java 採取 oracle に固定された値だけを厳密比較する。

### `mappingError` と `value`（D-028）

- `mappingError` は意味変換が**実際に失敗した**ときだけ非 null。entry が AST 値を作らないこと
  （`@mapping` の無い entry、根が空一致）は失敗ではなく、`ast=null` / `mappingError=null`。
- 文字列は言語ネイティブの表現。driver は他言語の例外クラス名を足さない。
  `@typeof` 違反だけは旧実装の観測を固定した契約値（`corpus/oracle/typeof.json`）なので、
  生成器が互換の文字列を組み立てる。
- `value` は driver が評価した結果があるときだけキーを置く。全入力解析が成功して AST が無い
  ときは**キーごと出さない**（未観測）。観測済みの `null` は評価結果としての null。

## 負荷に応じた driver タイムアウトと未観測の扱い（issue #73）

`cargo test --workspace` を共有機の高負荷（load average 30〜60、並行 agent 8 本）で回すと、
`ubnfc-fuzz` の driver 交換が散発的に壁時計タイムアウトへ落ちる（`docs/reports/2026-09-23-differential-fuzzing.md`
の観測、`docs/reports/2026-09-24-load-aware-timeouts.md` の再現試行）。これは driver 自体の
バグではなく、`javac` / `cargo build --release` と JVM 起動がホストの他プロセスと CPU を奪い
合って**基準タイムアウト（既定 90 秒）の中で終わらなくなる**ことが原因である。

### タイムアウトの倍率（`crates/ubnfc-fuzz/src/load.rs`）

- `/proc/loadavg` の 1 分平均を `std::thread::available_parallelism()`（オンライン CPU 数の
  見積もり）で割り、「1 CPU あたりの負荷」を倍率にする。
- 倍率は `[1.0, 4.0]` にクランプする。**無負荷（1 CPU あたり 1 未満）では倍率 1 で、
  基準タイムアウトそのまま**（既定動作は変えない）。上限 4 倍を超えて伸ばさないのは、
  恒常的な高負荷ホストでも「driver が本当に停止した」の検知が無限に遅れないようにするため。
- `/proc/loadavg` が読めない環境（Linux 以外）では倍率 1 のまま動く。
- 倍率は `Engine::run` が driver を起動するたびに**その時点の負荷**から計算し直す
  （campaign 開始時の 1 回きりではない）。`Engine.timeout` は基準タイムアウト、実際に
  `exchange` へ渡すのは `base.mul_f64(factor)`。

### `driver_timeout` は「失敗」ではなく「未観測」

- タイムアウトで打ち切られた応答には `{"error": "...", "kind": "driver_timeout"}` の印が付く
  （`ubnfc_fuzz::engine::is_driver_timeout`）。本物の driver エラー（異常終了・不正 JSON・
  未知/重複 id など）とはこの印の有無で区別する。
- `ubnfc_fuzz::campaign` は `driver_timeout` になった比較だけを、**該当 backend だけ**新しい
  プロセスで最大 `Config::timeout_retries`（既定 2）回再試行する（`run_pair_with_retries`）。
  同じバッチの他の入力は巻き込まない。
- 再試行を使い切ってもなお `driver_timeout` の比較は `Summary.unobserved_timeouts` に数え、
  `driver_errors` にも `findings`（差分）にも入れない。片側だけ応答が欠けたことを「両バック
  エンドの観測差」として登録しない、という corpus 側の未観測の扱い（本書冒頭）と同じ考え方を
  driver 交換そのものにも広げたもの。build discard（`build:java` / `build:rust` など）も
  同じ意味で「未観測」であり、`Summary.discards` に理由コード別で残る。
- `Summary` は `retried_comparisons` / `retry_attempts` / `unobserved_timeouts` /
  `max_load_factor`（campaign 中に観測した負荷率の最大）を JSON レポートに残す。

固定 seed campaign（`crates/ubnfc-fuzz/tests/campaign.rs`）の `summary.driver_errors == 0` は
「本物の driver エラーが無い」ことの主張のまま変えていない。無負荷なら `unobserved_timeouts`
も 0 のはずなので、この変更は無負荷時の green を壊さない。

### CI での方針（`.github/workflows/ci.yml` の `native` job）

- `--test-threads` は固定しない（cargo の既定 = CPU 数）。runner は他の作業と共有する
  self-hosted 1 台で、空き CPU が時間ごとに変わるため、スレッド数を絞る代わりに上記の
  負荷倍率（基準 90 秒 × 1〜4 倍）と再試行・未観測扱いで吸収する。
- job 全体の timeout は GitHub Actions の既定のまま（native は通常 15 分前後）。
- それでも `driver_timeout` だけで赤くなった場合は、失敗した job を 1 回だけ再実行する
  （`gh run rerun <id> --failed`）。2 回続けて落ちるなら負荷ではなく本物の不具合として調べる。
