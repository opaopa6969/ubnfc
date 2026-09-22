# ubnfc-rust

Grammar IR v1 から Rust の静的 parser と型付き AST を生成する。生成物の通常経路は std のみ。
`ubnfc-front::ir::{from_json, validate_references}` を入力境界に使う。

```rust,ignore
let ir = ubnfc_front::ir::from_json(&json)?;
let generated = ubnfc_rust::emit(&ir, &ubnfc_rust::EmitOptions::default())?;
generated.write_to("generated")?;
```

```sh
cargo build -q --offline --bin ubnfc-rust --bin ubnfc
out=$(mktemp -d)
target/debug/ubnfc-rust --ir ir/examples/tinycalc.ir.json --out "$out" --emit-driver
cargo build --release --offline --manifest-path "$out/Cargo.toml"
printf '%s\n' '{"id":0,"grammar":"TinyCalc","entry":"Expression","input":"1+2","options":{"lexical":true,"scope":true}}' | "$out/target/release/driver"
```

出力は `Cargo.toml`, `mod.rs`, `parser.rs`, `ast.rs`, `api.rs`, `runtime.rs`, `evaluator.rs`, `metadata.rs`,
`rt/{mod,cursor,input,memo,diag,scope}.rs` と `src/bin/{driver,driver_json}.rs`。
manifest の lib は `mod.rs` を指す。既存 crate のサブモジュールとしても利用できる。
`GeneratedCrate.files` は相対パスからソースへの BTreeMap。生成時に cargo/rustfmt/network は起動しない。
`--emit-driver` は既定 on。`--emit-driver false` / `--no-emit-driver` または
`EmitOptions { emit_driver: false, ..Default::default() }` で無効にできる。
生成 crate の通常依存はゼロで、`json` feature のときだけ optional な serde / serde_json を使う。
driver 自身の JSON reader/writer は std のみで動き、任意の JSON `id` と数値字句を保持する。

`ubnfc gen --grammar X.ubnf --target rust --out DIR` は IR を書いて `ubnfc-rust` を呼ぶ。
隣接するバイナリや `target/{debug,release}` を探索し、`UBNFC_RUST_BIN` で明示指定できる。
`--target java` は `UBNFC_JAVA_JAR` または `ubnfc-java/target/ubnfc-java-*.jar` を使い、
`java -cp ... org.ubnfc.java.Main` で `DIR/src/main/java` へ出力する。
明示指定が存在しない場合も含め、生成器が見つからなければ探索パスを表示し exit 3。
生成器の失敗終了は終了コードをそのまま返す。

runner の `Artifact::FromIr { backend: Backend::Rust, .. }` は事前ビルド済みの
`ubnfc-rust` を `CARGO_BIN_EXE_ubnfc-rust` または実行ファイル付近の target から解決する。
生成した全ファイルの内容 hash をキーに `Runner.build_dir/rust-cache` を作り、
同じ生成物なら cargo を再実行しない。`rust-target/<hash>` は build_dir 内で共有し、
文法の異なる driver が上書きされるのを防ぐ。キャッシュ寿命は build_dir の寿命と同じ。
ツール不足・生成失敗・offline build 失敗はエラーで、テストも skip しない。

JSON Lines driver は [harness protocol](../../docs/harness-driver-protocol.md) の認識・mapping・scope・診断を返す。
corpus の操作列・型情報 reflection など、driver に対応する観測項目がないものは未観測としてレポートする。
`grammar` と `entry`（null は文法の root）を解決し、未知の指定はエラーにする。
`lexical=false` で captures、`scope=false` で declarations/references/scope を省略し、文法の認識は変えない。
`scope` は `mode`（文法に現れるモードの順序付き配列）と `events` を持つ。イベントは
`{order, action, mode, name, offset, length}`。offset/length は code point、enter/leave の name は null。
診断は主位置の `expected` と `farthestOffset` / `farthestExpected` を独立して返す。
prefix の accepted/C/M と全入力の AST・診断を別々に観測する。
`value` は IR の operator 指定がある結果型に限り、AST の数値葉と `left/op/right` の
`+ - * / ^` を評価する。それ以外は null、非有限値はエラー応答。
不正リクエストも1行のエラーを返して後続を処理する。

`ubnfc-rt-template/src/{cursor,input,memo,diag,scope}.rs` は、この crate の
`templates/*.rs` を直接 `include!` する。生成物とテンプレートテストが単一ソースを検証する。
cursor は `advance::<false>(0)` でも M=C、memo は State・mode・version を含む open addressing、
診断は局所 DAG の参照を親 rule に接続して再生する。input/scope の契約は維持する。

`parse`, `parse_with_options`, `parse_with_scanner`, `parse_entry_with_options` が入口。結果には構文成功、code point の C/M、
AST、node span、capture、任意の lexical occurrence、診断、scope の宣言・参照・診断、統計を含む。
`build_ast=false` で AST と capture 出力を省略し、scope 効果と構文診断は維持する。
`require_eof=false` は prefix 受理。mapping 失敗は `ok=true, ast=None` と Mapping 診断で表す。
`feature="json"` の `Ast::{canonical_value,canonical_json}` と Serialize が canonical AST を出力する。
型付きフィールドは String、Box<Ast>、Option<Box<Ast>>、Vec<Ast>/Vec<String> 等。
Rust の型名・field 名は IR ID から決定的にエンコードする。`Node.Item` は独立 struct と
`g_Node_nested::g_Item` の再 export で表現し、commonField は Ast の `common_<index>_<field>` メソッドにする。

## 達成状態（round2）

| 段階 | 達成状態・範囲 |
|---|---|
| 生成 | catalog / eval / typeof / recovery / ERROR / Unicode CI / enum 重複を受理。regex は供給された DFA だけを実行 |
| build | std のみの生成 crate、optional json feature、driver を offline build / clippy。crate root は forbid(unsafe_code) |
| 認識 | 二重 cursor、scope 全出現、DAG 診断、資源上限。全 corpus の完全一致は未達 |
| mapping | rule 別関数による遅延 AST、typeof 型一致、recovery 欠損の伝播、深さ予算。数値は Rust oracle の字句 |
| 公開 metadata | catalogs、scope mode/events、rule 出現 lexical と token 別表 tokens、recoveries、主診断と hints/deepest |
| 固定点 | selfhost の生成 byte 一致と、生成 frontend → IR → Rust の固定点を tests/regenerate.rs で検証 |

`lexical=true` は API の `lexical`（rule 表）と `tokens`（token 表）を生成する。
既存 selfhost 変換器は rule 表だけを使うため両表を分けた。token は ExprId/RuleId を持ち、
親 ID は rule 表を参照し、token ID は rule ID 列の後に続く。完了順は両表で共通。
driver の `lexical` は両表を結合して公開する。未 capture の `a b x` も3 tokenになる。
成功した投機的失敗は `ParseResult.hints` / driver `hints` に保持し、主診断へ混ぜない。
trailing の主位置・expected と farthest、`deepestRule` / `rulePath` は別フィールド。

`metadata::CATALOGS` は rule/context/site IDs と重複除去・ソート済み capture 名を公開する。
context の空白は保存する。enum は IR の名前変換を維持し、同じ名前は一つの variant、
同じ literal は最初の対応を使う（`on` と `ON` はともに `ON`）。
`@typeof` は mapping 後の具象型を比較する。list は list、optional 不在は比較を省略する。
不一致は認識成功を保ち `ast=None` / Mapping 診断にする。

`evaluator::Evaluator` は具象 AST ごとの trait callback と `eval` dispatch を持つ。
auto の literal / passthrough / variable_ref / conditional / binary_arithmetic は既定メソッド。
manual・未知 kind と意味依存の apply_binary / eval_leaf / resolve_variable は実装側の callback。
`DefaultEvaluator` は未実装 callback を EvalError にする。literal は Rust の字句を Text として返し、
conditional の真偽は Java と同じ bool true または大文字小文字を無視した文字列 true。
driver の既存 `value` は operator 用の数値評価であり、この任意 callback API とは別。

recovery は D-018 に従い root にも適用し、typed AST の型を変えない。
SYNC は同期記号を消費、AUTO は FOLLOW の前（空 FOLLOW の fallback `;` は消費）、
SKIP は IR の FOLLOW の前まで、空集合なら1 code pointを進める。ゼロ前進は失敗。
回復 node は list から省略、optional は None、必須 scalar の欠損は親へ伝播する。
`ParseResult.recoveries` / driver `recoveredRegions` は span・syncSpan・capture 出現 ID を持ち、
1領域に1つの recovery/ERROR 診断を返す。rollback・先読み・memo は採用領域だけを公開する。
prefix の回復表・capture は保持し、全入力を満たさない場合の AST は None。

右結合の rule 局所 trivia は、内側ノードの末尾から除き、entry の消費範囲は保持する。
`RightScoped` の ` x /*r*/ ^ x ` は root `[0,13]`、右子 `[11,12]`。
IR のこの形は operator が null でも rightAssoc/interleave 注釈が残るため、emitter が注釈を使う。
消費範囲そのものは右子も `[11,13]` のため、generators.md #138 の一般規則に対する
局所例外として corpus を優先する。global javaStyle で末尾 trivia を含む既存 span は維持する。

`ParseOptions.limits` は入力16 MiB、arena/diagnostic各200万、memo/効果各100万、
効果文字列64 MiB、mapping 深さ512が既定。効果は破棄枝も含む累計作業量で制限する。
超過は resource 診断で全体失敗（mapping 深さのみ認識成功 + Mapping 診断）。
parse と mapping はそれぞれ native stack 256 KiB も監視する。mapping は rule 別関数に分割する。
任意サイズの scanner 内部確保や OS のメモリ枯渇を回復する仕組みではない。
生成時も式10万・rule1万・AST型256・式の子/record field各256・ValueType深さ64を先に検査する。
式到達性と型受理判定は明示 stack で走査し、生成予算超過は EmitError::ResourceLimit。

## 対応範囲

requiredFeatures は生成前に検査する。以下は対応するが、括弧内の制限を含む。

- `dualCursor`, `cursorFrameFlag`: `(C,M,mode,invert,R)`。consume(0) でも M=C。
  begin は R に応じて M を戻し、commit は C/M のみをコピーする。
  各式に consumed 版、到達する式だけに matchOnly 版を生成する。
- `orderedChoice`, `longestChoice`, `predictiveChoice`: 宣言順、消費量最大・同長先勝ち。
  FIRST の異なる非空・case-sensitive literal は char の match で直接分岐し、
  除外された先行 literal の診断も再現する。共通接頭辞、複合候補、未知候補は全順序選択に戻す。
  Predict / PredictiveWithFallback は同じ受理と診断を保つ。trie による複合候補の省略はまだ行わない。
- Seq、Group、Optional、Repeat、Separated、Any、Eof、Empty、正負先読み、Error、rule/token 参照。
  Repeat は外側で一度 begin。terminator → child → count → active cursor の進捗 → max → bounds。
  `max=0` でも子を試す。Separated は separator と次の element を一組で巻き戻す。
- `triviaInsertion`: entry / afterChild / exit / helperEntry / helperExit の指定順にのみ実行。
  policy の文字集合、行コメント、閉じた block comment。trivia cache の有効・無効も plan に従う。
- `captures`, `deferredMapping`, `operators`: 採用結果の arena を辿って初めて String と公開 AST を作る。
  scalar / optional / list、空 capture、内側→外側の完了順、transparent、text、skip、
  construct、assocLists と leafFallback、左右結合の各段を保持する。
  Rust の Number capture は字句 String。パース中に数値変換しない。
- `backendDiagnostics`, `memoReplay`, `structuredOccurrences`: 局所診断 DAG と採用イベント DAG を再生。
  呼出し元に rule path を接続し、memo hit でも occurrence ID を新しく付ける。
  negative lookahead 成功時の子失敗は破棄し、成功 rule 内の最遠失敗は保持する。
  trailing-input の主位置と最遠失敗位置を分ける。
- `scopeUndo`: ScopeStore の undo log、enter/leave/declare/use/clear、state version、rollback/replay。
  lexical/dynamic は指定イベントを実行。sameRuleText は rule 内 capture の一致を検査し、
  不一致は ERROR、scope 内の未定義参照は WARNING。どちらも構文成功とは分離する。
- `externTokens`: TokenScanner trait に logicalId、入力、全カーソル状態、mode、scope を渡す。
  ScanResult の UTF-8 境界・単調性を検査し、valueSpan、診断、順序付き効果を扱う。
  analysis が unsafe とする領域は memo しない。memo-safe の保証は front / alias 宣言の責務。
- 全 BuiltinToken、UNTIL、NEGATION、CHAR_RANGE、token 先読み、Unicode CI。
  CI は input.rs の Java simple-case 近似（é/É、İ/ı、ſ、K、sigma、非 BMP）。複数文字展開はしない。
  Rust/JDK の Unicode バージョン差は残る。Space token は **1 文字**、trivia は反復。
- `regex` は plan.lexical の DFA がある場合に対応。最長 prefix、ゼロ幅受理は失敗。
- `imports` は解決済み symbols を利用。`enum`, `commonField`、enclosingTypeId に対応。

次は **機能名を含む EmitError::Unsupported** で拒否する。
`leftRecursionRewrite`、frontendTransform、precedenceClimbing、DFA のない regex / Java Pattern fallback、
ValueType 自体にさらに入れ子にした Optional/List（field cardinality の Optional/List は対応）。
不正な参照、重複、memo の安全性違反・省略証明不足、重複 DFA 遷移、commonField の型不一致、
Choice plan の候補欠落・順序変更、空 trivia delimiter は InvalidIr。
未対応の execution transaction と node の組合せも明示拒否する。

## 検証

```sh
cargo build -q --bin ubnfc-rust --offline
cargo test -p ubnfc-rust -p ubnfc-rt-template --offline
cargo test -p ubnfc-harness --test runner_rust_e2e --offline
cargo clippy -p ubnfc-rust --offline --all-targets -- -D warnings
cargo fmt -p ubnfc-rust -- --check
```

- driver E2E: associative 24/24、right-associative 28/28、tinycalc 8/8。Known/Fail/未観測は各0。
- TinyCalc: 既存 IR を直接読み、独立して組み立てた canonical JSON golden に一致。
- Operators: 手で記述した27 Expr の IR、24 corpus 行に一致。
- Power: 50 Expr の IR に text/mapped/unicode/grouped の入口を収録。28行×4モードに一致。
- 136組は AST 付き・認識のみの両方で検証。入力列は sibling `unlaxer-parser` の
  associative / right-associative corpus を直接読み、AST・span・C/M・成否は
  `examples/handwritten-target/rust/tests/golden/*.json` と完全一致させる。
  sibling の位置は `UNLAXER_PARSER_ROOT` で指定可能。
- `mktemp -d` に workspace 外の crate を作り、生成 rt を同梱する。
  serde 無効の offline build/run、json 有効の build/run、生成物の clippy `-D warnings` を実行。
- 同一 IR の全出力 byte 一致、CLI とライブラリの出力一致、未対応機能名のエラーも検証。
- 追加生成文法で cursor、反復、Longest/同長、予測、負先読み、空/nested capture、
  Separated rollback、memo の capture/diagnostic/lexical 再生、scope rollback、
  Extern、DFA、組込み token、enum、入れ子 mapping、commonField、mapping 失敗を検証する。
- runtime の memo は20,000キーで拡張・衝突・全キー成分の区別を検証する。
- Operators/Power は JSON Schema Draft 2020-12 の validator でも適合を確認した。
  `tests/ir/build_fixtures.py` は手書き意味木の直列化補助で、front lowering に依存しない。
  fixture の source はこの仮想意味木の出自情報であり、実ソース位置の lowering のテストではない。

## 性能

測定の再現用コマンド:

```sh
UBNFC_BENCH_ITERATIONS=10000 cargo run -p ubnfc-rust --offline --example bench
UBNFC_BENCH_ALLOCATIONS=1 cargo run -p ubnfc-rust --offline --example bench
```

benchmark も専用の workspace 外 crate を作り、release で実行する。
warmup 100回、5 run の中央値。毎回新しい session/memo/arena を作成・破棄し、
AST 付きは capture・node span・mapping も含む。JSON 直列化と評価器は計測外。
両実装に同じ入力・mode を渡し、black_box で結果を保持する。
全 Expr の memo を有効にした保守 plan を使用し、省略証明は付けていない。

### 最終測定（2026-09-22）

AMD Ryzen 9 7950X、WSL2 Linux 6.18.33.2、rustc 1.98.1、既定 release profile、CPU pinning なし。
共有環境で各10,000回×5 run。単位は µs/parse、各セルは **認識 / AST付き**。
全 sample は [results.csv](benchmarks/results.csv)、確保量は [allocations.csv](benchmarks/allocations.csv)。

| 入力 | 生成 Rust | 手書き Rust（同時再測定） | 手書き Rust（既存記録） | Java（既存記録） |
|---|---:|---:|---:|---:|
| `1+2*3` | 2.183 / 4.349 | 2.497 / 4.069 | 2.318 / 3.315 | 2.13 / 2.29 |
| 括弧深さ8 | 11.571 / 19.174 | 22.937 / 27.847 | 20.030 / 26.110 | 14.2 / 15.8 |
| 512 atom | 419.729 / 710.125 | 1067.810 / 672.355 | 1015.251 / 1061.853 | 409 / 573 |

指定された512 atom の基準1,015µsを認識・AST付きとも下回る。
ただし **全入力・全モードで手書き版を上回ったわけではない**。
短い入力の AST付き、および512 atom の AST付きの同時再測定では手書き版より遅い。
Rust の既存記録は100,000回×5 run、Javaは200,000回×5 run。
allocator の状態、測定順序、共有環境の負荷の差があり、今回の再測定値と既存値を区別する。

確保回数は GlobalAlloc の alloc/realloc 呼出しを数えた別実行であり、タイミング測定には加えていない。
確保 byte は realloc 後のサイズも合計した要求量で、peak RSS ではない。

| 512 atom | 生成 Rust | 手書き Rust |
|---|---:|---:|
| 認識の確保回数 | 24 | 5,163 |
| 認識の要求 byte | 888,128 | 2,266,576 |
| AST付きの確保回数 | 11,376 | 9,310 |
| AST付きの要求 byte | 3,088,239 | 3,548,775 |
| memo entry | 9,219 | 4,609 |

### 遅さの原因と変更の根拠

手書き目標の `support.rs` / `rt/{memo,diag}.rs` を確認し、同じ corpus で測定した。

- **memo の量と格納方法**: 手書き版は全 rule/helper、生成版は plan に従って全 Expr を memo する。
  512 atom のエントリは手書き4,609、生成9,219で、生成版の仕事量の方が多い。
  手書きの整数キー hasher は byte ごとに rotate/xor/multiply を実行する。
  当時の生成版は固定整数をまとめて混合し、Vec の slot に index のみを置いた。
  round2 では RandomState によるセッションごとの hash と負荷率50%へ変更し、最大 probe 長を公開する。
  payload を表の拡張ごとに複製せず、key/payload の位置は u32 に圧縮した。
- **診断の確保**: 手書き版は失敗のたびに rule stack を Vec にし、全 active frame の
  局所 Snapshot を更新・複製する。生成版は共有できる診断 DAG を保存し、最終結果の
  rule path / expected を一度展開する。memo hit の相対 stack 再生もこの構造で処理する。
  認識のみでの確保回数の大幅な差は、String よりもこの部分が原因であることを裏付ける。
- **arena / 容量**: 生成版は per-rule の event Vec を単一 arena の Join/Capture/Rule/Token に統合する。
  失敗枝から参照を外し、memo が参照する ID は再利用しない。
  memo は `min(input_bytes*10+64, 32768)`、AST arena は `min(input_bytes*8+8, 32768)` の
  初期容量を予約する。round2 ではこれとは別に ResourceLimits の総量上限を設けた。
- **trivia**: 手書き版は隣接する chain/helper 境界で同じ位置を再走査する。
  生成版も意味論上必要な境界を維持し、plan が有効にした場合のみ policy/C/M/mode/R ごとの cache を使う。
  今回のベンチは trivia cache 無効なので、その効果を速度改善の理由には数えていない。
- **String の早期生成という仮説は手書き目標には当てはまらない**。
  手書き版も採用後に String を生成する。AST 付きでは NUMBER の token span を得る追加走査と
  f64 評価値の変換がある。生成版は token span を保持し、既存契約どおり字句 String だけを作る。
- **残るコスト**: 生成版の汎用 field mapping は capture と子の Vec を一時的に確保する。
  短い入力の AST 経路、および同時再測定した手書き版との一部比較では生成版がまだ遅い。
  Java の値は既存記録であり、今回の Rust 再測定から言語自体の速度差とは断定できない。

## IR v1 で不足・曖昧だった点

1. schema は execution の任意の組合せを許す。node ごとの transaction と、
   primitive 失敗時の cursor / frame flag の影響は JSON Schema だけでは検証できない。
   本実装は [カーソル契約](../../docs/design/04-cursor-and-diagnostics-contract.md) §1 と
   Once / PerAlternative / Occurs / Delegate の正規形を基準にしている。
   Occurs の begin は反復全体に一度であり、各 child に begin を追加してはいけない。
2. memo.entries にない Expr の扱いは配列型だけでは分からない。
   `referenceMode=allEligible` として analysis.memoSafety が安全とした全式を既定で memo する。
   明示的な省略は proof を要求する。scope の version は rollback 時に復元するが払い出し番号は戻さない。
3. AST の Rust 名前空間、共通 interface の具体表現、enum の canonical 表現、
   lexical occurrence の収集有無を設定する公開 API は固定されていない。
   本実装は ID エンコード、nested module の alias、enum variant の文字列、`lexical` option を採用した。
   canonical AST の object key は文字列順に生成し、serde_json の preserve_order にも依存しない。
4. field.cardinality と ValueType.Optional/List の二重指定は意味が重複する。
   field cardinality を実装し、さらに入れ子の ValueType container は明示拒否した。
   capture と recipe と AST field のすべての型一致を front の参照検査だけでは証明できないので、
   mapping 時にも cardinality / node type を検査して Mapping 診断にする。
5. capture の consumedExtent / matchedExtent / selectedChildExtent の違いは重要。
   selectedChildExtent には Extern の valueSpan を使えるよう、採用 token span も arena に保持した。
   source span と input の code point span は別物として扱う。
6. Number の `rust=lexeme` と grouped NUMBER の text は、ともに String として扱う。
   [事実集 §D](../../docs/design/02-facts-tokens-bench-corpus.md) の訂正に従い、
   手書き Java の int と同一視しない。case folding の Unicode/JDK 版は IR に無いため、
   Unicode CI は input.rs の近似を採用し、oracle の é/É を回帰で固定する。

## 共通テンプレートとの差分

`crates/ubnfc-rt-template` 本体は変更していない。

| バックエンド内 template | 元 template との差分 |
|---|---|
| `input.rs` | byte / code point 変換をそのままコピー |
| `scope.rs` | undo log と state version をそのままコピー |
| `cursor.rs` | 2位置の Cursor から invert/R を含む State に変更。begin / commit / consume(0) を D-013 に合わせた |
| `memo.rs` | HashMap を Vec open addressing に置換。全7キー成分を保持、slot は u32 payload index、rehash は index だけ |
| `diag.rs` | active frame ごとの Vec snapshot 複製を、最遠位置で枝刈りする immutable diagnostic DAG に置換 |
| `runtime.rs`, `finish.rs`, `api.rs` | 生成物専用の共有コードを追加。採用 event arena、遅延 mapping、scope effect replay、TokenScanner、公開結果 API |
| `bench.rs`, `allocations.rs` | 外部一時 crate での測定専用。生成物には含めない |

memo の byte offset / arena index / state version は u32 で圧縮する。セッション予算を先に検査し、超過時は resource 診断で失敗する。
入力そのものは &str、公開 span は code point。深い括弧・右再帰・mapping の完全な明示 stack 化は未実装。

### Java レビューから移植した契約（2026-09-22）

- `ParseOptions.max_depth` は同時に実行中の rule 呼出し上限（既定 10,000、0 は entry から失敗）。
  上限超過は解析全体を `syntax` / `maximum parse depth exceeded` で失敗させる。
  optional / choice / Not はこの失敗を成功へ変換できず、結果のカーソルと意味イベントは空に戻す。
- Rust の stack overflow は回復不能なので、追加で parse 開始からの native stack 使用量を
  256 KiB に制限する。rule / expression の入口で検査し、同じ構文失敗へ変換する。
  この保護により `max_depth` に達する前に失敗する場合がある（Java の JVM stack 上限と同様）。
  呼出し元は少なくとも 1 MiB の未使用 stack を確保する。小さな独自 thread stack、非常に大きい
  単一生成関数、scanner 自身の再帰まで安全を保証する仕組みではない。
- `ParseOptions.memo=false` で式 memo を無効化できる。既定は true。
- 空 Word の LOOKAHEAD は失敗、NEGATIVE_LOOKAHEAD は成功。空の IR literal の意味は既存のまま。
- Not 成功時だけ子診断を破棄する。memo payload 内の診断は残り、後の通常評価に再生する。
- tokenRef の表示は各参照の `expectedDisplay.rust` を使う。同じ token の別参照も独立する。
- 末尾入力診断の主 expected は `end of input`。投機的失敗は `farthest_expected` に保存する。
- `ParseResult.scope.events` は採用した enter / leave / declare / use / clear / error を実行順に保持し、
  各イベントに mode と code point 位置を含む。rollback・先読み・memo replay に追従する。
  `scope.mode` は IR の効果定義に現れるモードを重複なしで保持する（lexical と dynamic の混在に対応）。
- 行コメントの終端先読みは consumed を改行前に残し matched を LF/CRLF の後へ進める。
  `a//x\nb` の診断位置は 5、CRLF は 6。`neg(1e+)` は **Rust oracle の 5** を維持する。

`tests/review.rs` は Java `ReviewRegressionTest` 由来の生成物テストを debug / release / clippy で実行する。
反復・optional の境界、mixed list の空列と全 occurrence、Rust 識別子のエスケープ、scanner の
5 状態（State の C/M/invert/reset と独立の match_only）、式単位 memo の成功・失敗・副作用も固定する。
`tests/driver.rs` で JSON 応答も検証する。Java oracle の expected 集合を Rust 全体へ強制せず、
Rust の診断位置と Rust 用ラベルを独立に検証する。
memo / arena は入力長から上限付きの容量を予約し、拡張時のコピーと再確保を減らす。
