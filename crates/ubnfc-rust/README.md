# ubnfc-rust

Grammar IR v1 から Rust の静的 parser と型付き AST を生成する。生成物の通常経路は std のみ。
`ubnfc-front::ir::{from_json, validate_references}` を入力境界に使う。

```rust,ignore
let ir = ubnfc_front::ir::from_json(&json)?;
let generated = ubnfc_rust::emit(&ir, &ubnfc_rust::EmitOptions::default())?;
generated.write_to("generated/src")?;
```

```sh
cargo run -p ubnfc-rust -- --ir ir/examples/tinycalc.ir.json --out /tmp/generated/src
```

出力は `mod.rs`, `parser.rs`, `ast.rs`, `api.rs`, `runtime.rs`, `rt/{mod,cursor,input,memo,diag,scope}.rs`。
既存 crate のサブモジュールとしても、次の manifest の独立 crate としても使用できる。
`GeneratedCrate.files` は相対パスからソースへの BTreeMap。生成時に cargo/rustfmt/network は起動しない。

```toml
[package]
name = "generated-parser"
version = "0.1.0"
edition = "2021"
[lib]
path = "src/mod.rs"
[features]
default = []
json = ["dep:serde", "dep:serde_json"]
[dependencies]
serde = { version = "1", optional = true }
serde_json = { version = "1", optional = true }
```

`parse`, `parse_with_options`, `parse_with_scanner` が入口。結果には構文成功、code point の C/M、
AST、node span、capture、任意の lexical occurrence、診断、scope の宣言・参照・診断、統計を含む。
`build_ast=false` で AST と capture 出力を省略し、scope 効果と構文診断は維持する。
`require_eof=false` は prefix 受理。mapping 失敗は `ok=true, ast=None` と Mapping 診断で表す。
`feature="json"` の `Ast::{canonical_value,canonical_json}` と Serialize が canonical AST を出力する。
型付きフィールドは String、Box<Ast>、Option<Box<Ast>>、Vec<Ast>/Vec<String> 等。
Rust の型名・field 名は IR ID から決定的にエンコードする。`Node.Item` は独立 struct と
`g_Node_nested::g_Item` の再 export で表現し、commonField は Ast の `common_<index>_<field>` メソッドにする。

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
- 全 BuiltinToken、UNTIL、NEGATION、CHAR_RANGE、token 先読み、ASCII パターンの CI。
  CI の ASCII 等価クラスには İ/ı/ſ/K を含む。Space token は **1 文字**、trivia は反復。
- `regex` は plan.lexical の DFA がある場合に対応。最長 prefix、ゼロ幅受理は失敗。
- `imports` は解決済み symbols を利用。`enum`, `commonField`、enclosingTypeId に対応。

次は **機能名を含む EmitError::Unsupported** で拒否する。
`recovery`, `eval`, `catalog`, `typeof`, `leftRecursionRewrite`、frontendTransform、
precedenceClimbing、DFA のない regex / Java Pattern fallback、非 ASCII パターンの CI、
ValueType 自体にさらに入れ子にした Optional/List（field cardinality の Optional/List は対応）。
不正な参照、重複、memo の安全性違反・省略証明不足、重複 DFA 遷移、commonField の型不一致、
Choice plan の候補欠落・順序変更、空 trivia delimiter は InvalidIr。
未対応の execution transaction と node の組合せも明示拒否する。

## 検証

```sh
cargo test -p ubnfc-rust --offline
cargo clippy -p ubnfc-rust --offline --all-targets -- -D warnings
cargo fmt -p ubnfc-rust -- --check
```

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
  生成版は固定整数をまとめて混合し、Vec の slot に index のみを置く。
  payload を表の拡張ごとに複製せず、key/payload の位置は u32 に圧縮した。
- **診断の確保**: 手書き版は失敗のたびに rule stack を Vec にし、全 active frame の
  局所 Snapshot を更新・複製する。生成版は共有できる診断 DAG を保存し、最終結果の
  rule path / expected を一度展開する。memo hit の相対 stack 再生もこの構造で処理する。
  認識のみでの確保回数の大幅な差は、String よりもこの部分が原因であることを裏付ける。
- **arena / 容量**: 生成版は per-rule の event Vec を単一 arena の Join/Capture/Rule/Token に統合する。
  失敗枝から参照を外し、memo が参照する ID は再利用しない。
  memo は `min(input_bytes*10+64, 32768)`、AST arena は `min(input_bytes*8+8, 32768)` の
  初期容量を予約する。この値は初期予約量のみで、必要なら拡張するため受理上限ではない。
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
   非 ASCII パターンの CI は完全互換を装わず拒否する。

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

memo の byte offset / arena index / state version は u32 で圧縮し、超過時は切り捨てず明示的に停止する。
入力そのものは &str、公開 span は code point。深い括弧・右再帰・mapping の完全な明示 stack 化は未実装。
memo / arena は入力長から上限付きの容量を予約し、拡張時のコピーと再確保を減らす。

変更範囲の指定に従い workspace の Cargo.lock は commit に含めない。
呼び出し側で統合する際は `ubnfc-rust` の依存リストに `ubnfc-front` を加える lock 更新が必要
（通常の cargo コマンドが自動更新する。`--locked` では事前更新が必要）。
