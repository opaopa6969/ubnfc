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
  Diagnostics の投機 checkpoint と匿名式 accumulator は独立モジュールでも検証する。
- `src/test/`: `JavaCompiler` と親 loader が platform の `URLClassLoader` による独立生成物テスト。
  生成された全ソースを `--release 21 -Xlint:all -Werror` でコンパイルする。
  一時領域は `mktemp -d` で確保し、テスト後に削除する。

生成先には `{Grammar}Parser.java`, `{Grammar}AST.java`, `{Grammar}Evaluator.java`, `{Grammar}Result.java`,
`api/*`, `internal/*`, `rt/*`, `driver/Driver.java` を配置する。通常の入口は
`{Grammar}Parser.parse(CharSequence[, ParseOptions]) -> ParseResult<{Grammar}AST>`。
`ParseOptions` で AST 構築、全消費、lexical 出力、memo、Extern scanner、`maxDepth` を指定できる。

`maxDepth` は同時に実行する rule 呼出し数の上限（正の整数、既定 `10_000`）。既存実装にはない安全策で、
上限到達時は解析全体を中断し、C=M=0、`syntax` 診断、空の AST/capture/scope を返す。
optional や choice に包まれていても成功へ変換しない。既存の2引数・5引数コンストラクタは既定値を使う。
生成関数は直接再帰するため、`-Xss` によっては指定上限より先に JVM のスタック限界に到達する。
この場合も最上位で回収し、`JVM stack limit` を含む同じ種類の失敗を返す。受理可能な深さの `-Xss` 非依存化は未実装。
8段の明示上限と2000段の入力、memo ON/OFF、別プロセスの `-Xss256k` / `-Xss1m` を検証する。

Extern は `scan(Input, ScanState)` を実装する。`ScanState` は UTF-16 の consumed/matched、
`Mode.consumed` / `Mode.matchOnly`、invertMatch、resetMatchedWithConsumed を渡す。
`ScanResult(ok, consumedEnd, matchedEnd, ValueSpan, diagnostics, effects)` は終了カーソルと任意の値範囲・診断・効果を返す。
`valueSpan == null` は選択カーソルの開始〜終了、null の diagnostics/effects は空リストを意味する。
失敗結果のカーソルと効果は適用しない。成功した matchOnly scanner は C を変更してはならない。
値範囲・診断位置は UTF-16 境界で検証し、公開 capture/span は code point に変換する。
`Effect` は action/name/offsetCp/lengthCp/mode を持ち、4引数版は mode=lexical。

`ParseResult.scope().events()` は採用された enter/leave/declare/use/clear/error を実行順に公開する。
各 `ScopeEvent` は order/action/mode/name/offsetCp/lengthCp を持ち、lexical/dynamic の指定を保持する。
失敗枝と先読みのイベントは復元し、memo hit では同順で再生する。

生成 AST の JDK 型は完全修飾名で参照する。`java` / `javax` / `org` / `util` は末尾に `_` を付け、
同じ語に既存の `_` が付く場合ももう1文字付けるため、`java` と `java_` は衝突しない。
record/field の元の名前は `@SourceName` に保持し、canonical JSON では元の IR 名を返す。

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
expected は farthest offset とともに返す。主 expected と `farthestExpected` を分離し、
trailing_input の主 expected は常に `["end of input"]`。driver も `farthestOffset` / `farthestExpected` を返す。scope 診断には `kind: "scope"`、空の expected と
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
| memo | plan に記載された全 ExprId（非 rule-body を含む）の成功/失敗可否を実行する。rule 全体も body の計画を使い、省略時は安全な全 rule を既定とする。キーに C/M/mode/invert/flag/stateVersion。レシピ・capture・lexical・flag 効果・scope 効果・局所診断を再生 |
| scope | enter/leave/declare/use/clear、順序付き公開 events と lexical/dynamic mode、同 rule のテキスト backref。U+0000..U+0020 の trim と位置補正、undo checkpoint、Longest と先読みの復元 |
| 診断 | 主 offset/expected と最遠位置/expected の分離、expected のソート、rule stack の再生、Not 成功時だけ子診断を破棄、不完全指数の失敗位置保持 |
| Extern | logicalId → TokenScanner。5状態の ScanState、UTF-16 位置、任意の valueSpan/診断、順序付き効果を返す。保証のない scanner とその到達可能祖先は memo しない |
| その他 | 統合済み import シンボル、catalog、構造付き lexical 出現、検証済みの直接左再帰 lowering 記録。実行するのは rewrittenBody |
| eval | 抽象 visitor、具象 variant dispatch、5種類の auto 既定実装。manual/abstractCallback/未知 kind は抽象 callback。DebugStrategy/StepCounterStrategy |
| typeof | ownCapture/referencedCapture の site ID を同じ rule の mapping field へ解決し、mapping 時に runtime class 一致を検査。不一致は構文結果と位置を保持した MappingException |
| recovery | SYNC/AUTO/SKIP、明示された ruleRef site、code point の進捗保証。D-018 の別表投影と回復1件につき recovery 診断1件。既存 mappingFailure/捕捉不要の explicitRecipe も維持 |
| enum | IR の literal/宣言順/javaName を保持。重複名の後続を _2, _3, … で区別し、既存名を予約して衝突回避。fromText と mapping は同じ対応表 |
| ERROR | transaction=once/none、到達位置で失敗し message を expected に一度だけ追加。memo 再生でも重複しない |

認識のみでも capture と scope は取得できる。lexical は既定で無効。
構造付き出現 ID は採用結果の materialize 時に振り直すため、memo 再生でも重複しない。
AST の位置表は IdentityHashMap により同値な record の別出現を保持する。

## 明示拒否

次の機能は推測で補わず `IllegalArgumentException` に機能名を含める。

- 版管理された `frontendTransform`、具象型を必要とする auto eval の和型指定や不正な field 型/cardinality。
- 未 lowering の直接左再帰、間接左再帰、lowering proof / rewrittenBody の不整合。
- `precedenceClimbing`, `lexical.dfa`、Java fallback を許さない regex。
- capture を要求する recovery explicitRecipe、空同期文字列、参照先の不一致・重複した applicationSites。
- ValueType 自体の `optional` / `list` 入れ子。通常の AstField/Capture.cardinality による Optional/List は対応する。
- 対応しない execution transaction / childMode、delegate group や choice 等への trivia 挿入、repeat/separated の afterChild 挿入。
- AST/recipe の型・cardinality 不整合、不正 Java 識別子、未解決・重複 ID、unsafe Extern を含む memo 指定、証明のない memo 省略。

prefix trie、trivia cache は読み込み可能だが、現在は先頭 code point の保守的予測とスカラー trivia を使う。
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
- 既存テストに加え、2026-09-22 の敵対的レビューの回帰テストを追加。生成 byte の決定性、未知キーの無視と feature 拒否、memo on/off、scope undo と replay、入れ子/enum/commonField、反復とカーソルの境界条件も検証する。

`ReviewRegressionTest` は `empty-zero` / `empty-two` / `eof` / `ci` / `regex-zero` / `comment-line` の oracle JSON と照合し、
追加実測の R=true 反復、optional の M=0、Not 失敗時の offset=1、末尾入力の offset=1/farthest=2 も固定する。
既存 golden は変更せず、旧形式の expected（最遠 accumulator に EOF を追加していた欄）を分解して比較する。
空文字 Word は成功しないため、従来の「空 capture」の手製 IR は空 sequence を使う。

| 実用経路 | 検証できた範囲 | 残る範囲 |
|---|---|---|
| ubnf.ubnf fixture | 生成・警告なしコンパイル・認識・AST、annotations=[] / [RootAnnotation, MappingAnnotation] の golden | occurrence → GrammarAst の frontendTransform、IR byte 固定点 |
| P4 fixture | 元 IR の拒否が Recipe conversion mismatch だけであること。報告された mismatch だけをメモリ上で補正後、inputState を保ったまま生成・コンパイル | front の fixture 修正、実 scanner を使う P4 全体の実行 |


## M3 の契約と oracle 固定値

`{Grammar}Evaluator<T>` は `eval(AST)`、型別の protected `eval*`、DebugStrategy を持つ。
`binary_arithmetic` は左から fold（evalLeaf/applyBinary）、`variable_ref` は strip_prefix 除去後に
resolveVariable、`conditional` は Boolean.TRUE または文字列 true を条件として片側だけ評価する。
`passthrough` は AST 値を再帰評価し、それ以外はその値を返す。`literal` は値をそのまま返す。
manual/未知種別/注釈なしの具象型は抽象 callback。和型の adapter は具象 variant に dispatch し、
自動評価を和型に指定する IR は生成前に拒否する。driver の value は従来の数値 fold のままで、
利用者の抽象 callback を勝手に実装しない。

`ParseResult.recoveries()` は採用順に `ruleId, mode, span, syncPattern, skippedSpan, syncSpan,
captureOccurrences, diagnostic` を返す。位置は code point、syncSpan は未検出時 null。
span は回復開始から再開まで、skippedSpan は同期文字列より前の飛ばした範囲。
`diagnostic` は `kind=recovery, severity=ERROR, origin=recovery, offset=span.start, length=span.length`。
失敗試行の局所 expected/farthest を保持し、後続の失敗や不採用枝の診断で置き換えない。
driver は同じ回復表を公開し、回復診断を trailing_input 等の終端診断より先に出力する。

SYNC は同期文字列を消費し、未検出は失敗。AUTO は FOLLOW の直前、空なら `;` を消費して再開する。
SKIP は FOLLOW 直前まで（未検出は EOF まで）、FOLLOW が空または開始位置なら1 code point を飛ばす。
ゼロ前進・EOF での回復成功は禁止。root には一律適用せず applicationSites の rule 参照だけを包む。
この点は D-018 文書の root 適用案より、本タスクの明示指示と IR schema 契約を優先している。

legacyMarker は D-018 の recovery-projection/recovery-diagnostics として扱う。
回復した typed node は作らず、list は省略、optional は空、必須 scalar が欠ける親は上へ欠損を伝播する。
構築不能な root は ast=null、欠損自体では MappingException を出さない。正常 node の値・位置と
回復を直接捕捉する capture は保持する。回復済み prefix の全消費失敗でも capture・回復表を公開し、
oracle 規則どおり ast は null。失敗枝/先読みでは回復表を破棄し、memo hit では1回だけ再生する。

| 対象 | 固定した値 |
|---|---|
| eval 7 corpus | oracle の AST/capture、eval* の abstract 属性。追加実行: `1+2=3`, `1+2+3=6`, `1?2:3=3`, literal/passthrough/manual/unknown の `7=7`, 変数 `x`。非 BMP prefix、和型 dispatch、条件の短絡も検証 |
| typeof | `a:a`, `b:b`: C=M=3 と Pair(A,A)/Pair(B,B)。`a:b`, `b:a`: 構文成功 C=M=3、MappingException、expected A/B と got B/A。`a:`: 構文失敗 offset=2 |
| recovery SYNC/AUTO | `bad;ok;`: C=M=7、Stmt[4,7]{value:ok} のみ、回復[0,4]と診断1件。AUTO の旧 C=M=6 を D-017/D-018 により修正 |
| recovery 非 BMP | `😀;x`: prefix 成功 C=M=2、ast=null、capture[0,2]、recovery@0 と trailing_input@2。旧過剰消費を再現しない |
| recovery root/SKIP | root `bad;` は失敗 C=M=0、`ok;` は成功。SKIP は生成可能にし `bad;ok;` を全消費、`bad` は C=M=3 と空 items。旧生成 API 不整合を再現しない |
| ERROR | `a`: C=M=1 成功。`x`/空入力: C=M=0 失敗、offset=0、expected=["'a'","need a"]、hint は1件 |
| enum | `on/off` は ON/OFF、未知値は fromText で例外。衝突 `on/ON` は ON/ON_2 として生成・mapping 成功。D-017 により旧コンパイル失敗を再現しない |

`M3FeatureTest` / `M3RecoveryTest` は front 由来の16 IR、既存 eval-default/elements fixture、
手製の境界 IR を生成→`--release 21 -Xlint:all -Werror` コンパイル→実行する。
既存 oracle・divergences は読み取り専用。追加実行値は oracle の採取値と区別している。
回復の nested scalar/optional/list、memo ON/OFF、Longest/先読み/失敗枝の復元、scope undo、
破棄された数値 mapping、同じ ERROR の再訪も固定する。

## IR 契約で補足が必要な点

1. `Execution` の各値と ExprNode の全組合せは schema では制限されない。tinycalc の group=delegate / optional=occurs を尊重し、実装した組合せだけ受け付ける。
2. `MemoPlan.entries` を ExprId 単位で実行する。rule wrapper は scope/mapping も含め別名前空間のキーを使う。匿名式の局所診断には呼出元 rule を重複保存しない。
3. recovery は通常 resolvedSyncPatterns を使う。現行 front の裸の auto/skip 注釈が SYNC に落ちる旧 IR は、legacyMarker に限り注釈から mode を復元し、式の継続位置から literal FOLLOW を固定点計算する。未知 token の同期文字列は推測しない。正常な明示 IR の mode は変更しない。
4. `ValueType.optional/list` と別欄の cardinality の二重指定、frontendTransform の transformId 別仕様は未対応。typeof の ownCapture/referencedCapture は正規化 IR の capture site ID とし、rule-local の mapping field に解決する。
5. Extern の5状態を ScanState で注入し、ValueSpan と Effect record を定義した。設計03 §6 の旧3引数 scanner 署名はこの API に合わせた更新が必要。
6. 共有 runtime の Cursor.advance は C/M の両方に加算する。D-013 の consume は M=C なので、生成側の Session.Frame で5状態を実装する。Cursor クラス自体の API は維持する。
7. Power の inline `'^'` も Operator.operatorRuleId は rule ID を必須とするため、fixture では trivia のない合成 PowOp rule を置く。
8. tinycalc の VarDecl keyword は `'var' | 'variable'` の順。境界を追加しない PEG 契約により、`variable y` が後者へ自動切替されることはない。

これらの制限を外す場合は、対応する IR fixture と生成物の意味論テストを同時に追加する。
