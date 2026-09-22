# ubnfc-java

Grammar IR v1 から、JDK 21 だけでコンパイル・実行できる Java ソースを生成する。
入力は `ubnfc-analysis.v1-draft1` / `ubnfc-json.v1`。生成器の実行時依存も JDK のみ。

```sh
cd ubnfc-java
mvn -o -q test
mvn -o -q package
java -cp target/ubnfc-java-0.1.0-SNAPSHOT.jar org.ubnfc.java.Main \
  --ir ../ir/examples/tinycalc.ir.json --out /path/to/output --package example.calc
```

`--package` を省略すると entry grammar の package を使う。未設定なら
`generated.<grammar名の小文字>`。検証と全ソースの組み立てを済ませてから出力する。
生成物に IR JSON、生成器クラス、外部ライブラリは不要。

## 構成

- `src/main/java/org/ubnfc/java/ir/`: JSON reader、当該 schema の検証器、型付き IR、参照・対応範囲の検証。
  JSON の重複キー、不正 escape、不正数値、末尾の余分な値を拒否する。未知 object キーは型への読み込み時に捨てる。
  未知・未対応 `requiredFeatures` と `analysisVersion` は出力前に拒否する。
- `src/main/java/org/ubnfc/java/emit/JavaEmitter.java`: AST と直接呼出しの parser ソースを生成する。
  rule ごとの静的関数、式ごとの consumed / matchOnly 関数を出力する。実行時 IR interpreter は持たない。
- `src/main/resources/templates/api/`: `ParseResult`, `Capture`, `Diagnostic`, `Span`,
  `MappingException`, `TokenScanner`, `ParseOptions`, canonical JSON。
- `src/main/resources/templates/internal/`: セッション、トランザクション、immutable 構築レシピ。
- `runtime-template/src/main/java/ubnfc/rt/*.java`: ビルド時に resource として同梱し、生成時に package だけ変更してコピーする。
  元テンプレートと独立モジュールは変更しない。
- `src/test/`: `JavaCompiler` と親 loader が platform の `URLClassLoader` による独立生成物テスト。
  生成された全ソースを `--release 21 -Xlint:all -Werror` でコンパイルする。
  一時領域は `mktemp -d` で確保し、テスト後に削除する。

生成先には `{Grammar}Parser.java`, `{Grammar}AST.java`, `{Grammar}Result.java`,
`api/*`, `internal/*`, `rt/*`, `driver/Driver.java` を配置する。通常の入口は
`{Grammar}Parser.parse(CharSequence[, ParseOptions]) -> ParseResult<{Grammar}AST>`。
`ParseOptions` で AST 構築、全消費、lexical 出力、memo、Extern scanner を指定できる。

## Harness driver

`--emit-driver` は既定で有効。`--emit-driver false` または `--no-emit-driver` で無効にできる。
API は `new JavaEmitter(ir, packageName, emitDriver)`。生成先は空のディレクトリを使う。
`<package>.driver.Driver` は UTF-8 JSON Lines を読み、各応答の直後に flush する。
JSON reader/writer は生成 `api/Json` を使い、JDK 以外の依存を持たない。

`grammar` は IR に含まれる全 grammar unit の宣言名、`entry` はその unit の rule 名。
`entry: null` はその unit の一意な `@root` を選ぶ。entry grammar では、root annotation が
省略された正規化 IR の `entry.ruleId` も利用する。root のない他 unit は明示 entry が必要。
未知・曖昧な root、未知 grammar/entry、壊れた JSON は `id` と `error` を返し、次の行へ進む。
生成 parser の `parseEntry(grammar, entry, source, options)` も同じ dispatch を利用できる。

prefix を認識し、全入力を AST 構築付きで解析する。応答は `accepted`, `allConsumed`,
`consumed`, `matched`, canonical `ast`, `captures`, `declarations`, `references`,
`diagnostics`, `mappingError`, `value`。位置は Unicode code point。
`options.lexical: false` では captures、`options.scope: false` では宣言・参照を省略する。
解析と scope 診断はオプションで変えない。診断は primary offset を使い、成功時にも残る
expected は farthest offset とともに返す。scope 診断には `kind: "scope"`、空の expected と
message/length/severity を付ける。mapping 失敗でも prefix と全入力の構文受理は保持する。

`value` は IR の operator 指定がある AST 型を起点に、`left/op/right` のリストをたどり、
数値の葉に対する `+`, `-`, `*`, `/`, `^` を double で計算する。
右結合は AST の右の子の入れ子に従う。数値テキストの末尾の空白・Java コメントも扱う。
`value` フィールドだけの数値葉、assocLists の leafFallback はこの fold 内でのみ使う。
汎用の文法評価器ではなく、変数・宣言・プログラム全体・他の形・未知の演算子は null。
NaN/Infinity はプロトコルのエラー応答。captures の binding 等の追加 metadata や
`texts/nodes/valueSpans/scopeDepth` は出力しない。

repo root から事前に jar を用意し、生成から corpus 比較まで実行できる。

```sh
(cd ubnfc-java && mvn -o -q package)
cargo run -p ubnfc-harness -- run --backend java \
  --ir ubnfc-java/src/test/resources/ir/operators.ir.json --families associative
cargo run -p ubnfc-harness -- run --backend java \
  --ir ubnfc-java/src/test/resources/ir/power.ir.json --families right-associative
```

runner の `Artifact::FromIr { ir_path, backend: Backend::Java }` は生成パッケージを `target` に
固定して既存の `target.driver.Driver` 起動と接続する。jar は
`ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar` を使い、runner は Maven を起動しない。
`Artifact::Prebuilt { dir }` と従来の `runner.build(path)` も利用できる。
`build_with_java_generator(artifact, classpath)` で生成器の jar/classes を明示できる。
生成ソースと classes は build ごとに分離し、失敗時も生成ソースを削除する。
E2E は `target/classes` を使い、なければプロセス内で一度だけオフライン package を行う。
`java`/`javac` がない環境では失敗し、skip しない。

| E2E | Pass | Known | Fail | 未観測項目 / ケース |
|---|---:|---:|---:|---:|
| associative | 24 | 0 | 0 | 0 / 0 |
| right-associative | 28 | 0 | 0 | 0 / 0 |
| tinycalc（独立した受理・prefix・entry の8件） | 8 | 0 | 0 | 0 / 0 |

corpus の value 観測は除外しない。`1e2/4-0.5` も AST と value 24.5 を返す。
JSON Lines の異常後の継続、任意の id、複数 grammar、scope イベント、mapping 失敗、
非 BMP capture、採取オプション、非有限評価値も生成 driver のプロセス経由で検証する。

## 対応範囲

| 分野 | 対応 |
|---|---|
| カーソル | C/M/mode/invert/frame flag。consume(0) による M のリセット、begin/commit/rollback、EMPTY、EOF、先読み、UNTIL |
| 構文 | seq、group、optional、repeat の周回後上限判定、separated の separator+element 巻き戻し、literal、rule/token ref、error |
| choice | Ordered、Longest（同長は宣言順）、Predict、PredictiveWithFallback。mayStart から code point switch を生成。unknown は除外しない |
| token | Number、ASCII Identifier、引用符と任意1 code point escape、Digit/Alphabet 系、Space、CodePointSet、ANY/EOF/EMPTY、UNTIL、CI、NEGATION、CHAR_RANGE、Java Pattern fallback |
| trivia | effectiveDelimiterSet と insertionPlan。seq の entry/afterChild/exit、group/helper 境界、repeat/separated の entry/exit。文字集合、行・ブロックコメント |
| capture / AST | 完了順・空 span、scalar/optional/list、text/number/node/mixed、透明 alias/union、sealed interface と record、入れ子、enum、commonField、assocLists と leafFallback |
| mapping | 採用後だけ public AST を構築。Integer.parseInt もこの段階だけ。MappingException は構文結果と span を保持 |
| memo | rule 単位。body の MemoEntry の成功/失敗可否を使用し、省略時は安全な全 rule を既定とする。キーに C/M/mode/invert/flag/stateVersion。レシピ・capture・lexical・flag 効果・scope 効果・局所診断を再生 |
| scope | enter/leave/declare/use/clear、lexical/dynamic metadata、同 rule のテキスト backref。U+0000..U+0020 の trim と位置補正、undo checkpoint、Longest と先読みの復元 |
| 診断 | 主 offset と最遠位置の分離、expected のソート、rule stack の再生、負の先読みの子失敗抑止、不完全指数の失敗位置保持 |
| Extern | logicalId → TokenScanner。UTF-16 位置、診断、順序付き効果を返す。保証のない scanner とその到達可能祖先は memo しない |
| その他 | 統合済み import シンボル、catalog、構造付き lexical 出現、検証済みの直接左再帰 lowering 記録。実行するのは rewrittenBody |
| recovery | SYNC/AUTO の解決済み同期文字列、consumeSync/noSyncAction、明示された ruleRef site。mappingFailure または capture 不要の explicitRecipe 投影 |

認識のみでも capture と scope は取得できる。lexical は既定で無効。
構造付き出現 ID は採用結果の materialize 時に振り直すため、memo 再生でも重複しない。
AST の位置表は IdentityHashMap により同値な record の別出現を保持する。

## 明示拒否

次の機能は推測で補わず `IllegalArgumentException` に機能名を含める。

- `eval`, `typeof`, 版管理された `frontendTransform`。
- 未 lowering の直接左再帰、間接左再帰、lowering proof / rewrittenBody の不整合。
- `precedenceClimbing`, `lexical.dfa`、Java fallback を許さない regex。
- recovery の `legacySkip` / `legacyMarker`、capture を要求する recovery explicitRecipe。
- Extern の追加 `inputState`（現在の scanner API に状態注入契約がない）。
- ValueType 自体の `optional` / `list` 入れ子。通常の AstField/Capture.cardinality による Optional/List は対応する。
- 対応しない execution transaction / childMode、delegate group や choice 等への trivia 挿入、repeat/separated の afterChild 挿入。
- AST/recipe の型・cardinality 不整合、不正 Java 識別子、未解決・重複 ID、unsafe Extern を含む memo 指定、証明のない memo 省略。

prefix trie、trivia cache は読み込み可能だが、現在は先頭 code point の保守的予測とスカラー trivia を使う。
非 rule-body の memo entry は将来の式単位最適化用 metadata として読み込む。
最適化 profile にかかわらず、現在の emitter は段別 operator 関数を使う。

## 検証と手書き目標との差分

- tinycalc: 宣言、optional 初期値、演算子階層、空入力、末尾入力、非 BMP コメント等の canonical AST と offset/expected を固定した 8 golden。
- Operators: 元の 24 corpus。Power: 元の 28 corpus。受理、全消費、double 評価値、canonical AST、capture の完了順と span、最遠診断を比較する。
- `(NUMBER) @value` は設計資料の訂正に合わせて String。Operators の Number.value は capture の原文を strip した文字列で、末尾コメントも保存する。
  したがって手書き側が int mapping failure になる `1e2/4-0.5` は生成側では AST ができ、評価値 24.5。
  この入力の AST は別 golden に固定。ほかの Number は手書き golden の capture span から文字列へ訂正して比較する。
- 上記の int 訂正を除き、Operators/Power の canonical AST の差分はない。
- 手書き目標の expected は発見順、IR 契約はソート済みなので、expected の集合・ソート後の値を比較する。
  Capture.siteId は手書きの整数と異なり、IR の安定 siteId 文字列を公開する。
- Number の直接 capture を int にする別 IR で、採用枝だけの数値 mapping failure と認識成功を検証する。
- 合計 93 テスト。生成 byte の決定性、未知キーの無視と feature 拒否、memo on/off、scope undo と replay、入れ子/enum/commonField、反復とカーソルの境界条件も検証する。

## IR 契約で補足が必要な点

1. `Execution` の各値と ExprNode の全組合せは schema では制限されない。tinycalc の group=delegate / optional=occurs を尊重し、実装した組合せだけ受け付ける。
2. `MemoPlan.entries` は ExprId 単位だが、rule-entry と匿名 helper の選択境界は未指定。今回は rule body を rule-entry の計画として使用する。
3. recovery の `legacySkip` と `legacyMarker` は名前だけでは前進・型投影を一意に実装できないため拒否する。AUTO の FOLLOW 推測はせず resolvedSyncPatterns を使う。
4. `ValueType.optional/list` と別欄の cardinality の二重指定、`typeof` の capture 名と site/occurrence の参照解決、frontendTransform の transformId 別仕様は未確定。該当部分は拒否する。
5. Extern の resultFields は effects を要求するが効果 payload と inputState の注入方法は schema にない。生成 API は明示的な Effect record を定義し、追加 inputState は拒否する。
6. 共有 runtime の Cursor.advance は C/M の両方に加算する。D-013 の consume は M=C なので、生成側の Session.Frame で5状態を実装する。元 runtime はコピー以外変更しない。
7. Power の inline `'^'` も Operator.operatorRuleId は rule ID を必須とするため、fixture では trivia のない合成 PowOp rule を置く。
8. tinycalc の VarDecl keyword は `'var' | 'variable'` の順。境界を追加しない PEG 契約により、`variable y` が後者へ自動切替されることはない。

これらの制限を外す場合は、対応する IR fixture と生成物の意味論テストを同時に追加する。
