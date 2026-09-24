# Java / Rust バックエンドの差分 fuzzing（2026-09-23）

対象: `crates/ubnfc-fuzz`（新規）。同じ UBNF 文法・同じ入力を Java 生成物と Rust 生成物の両方で
走らせ、**観測が一致しないこと自体を不具合として扱う**。両 backend は front
（`crates/ubnfc-front`: syntax → resolve → validate → lower → analysis → IR）を共有するが、
生成器と runtime は独立なので、観測差は少なくとも片側のバグか、契約の不足である。

適合性 corpus（1,212 ケース、Java/Rust とも Fail 0。`docs/reports/2026-09-22-java-corpus-round7.md`、
`docs/reports/2026-09-23-rust-corpus-m5r4.md`）は**旧実装の観測を固定したもの**なので、
両 backend に共通して存在する欠陥や、corpus が観測していない wire 項目の差は通り抜ける。
本タスクはそこを狙う。

## 1. 構成

| モジュール | 役割 |
|---|---|
| `src/rng.rs` | SplitMix64。campaign は seed だけで完全に再現する |
| `src/model.rs` | 生成文法の構造モデル。UBNF 原文への描画、根からの導出、縮小候補の列挙 |
| `src/gen.rs` | 重み付きのランダム文法生成。機能タグを文法ごとに記録する |
| `src/input.rs` | 導出入力・変異・純乱数の 3 系統 |
| `src/observe.rs` | 2 応答の比較（契約による弱めを含む）と内部不変条件 |
| `src/engine.rs` | 文法 1 つの IR → 両 backend 生成 → build → driver 実行（制限時間つき） |
| `src/campaign.rs` | seed の並列消化、差分クラスごとの最小化、集計 |
| `src/main.rs` | CLI（`--seeds` / `--minutes` / `--jobs` / `--dump` ほか） |
| `tests/campaign.rs` | CI 用の固定 seed campaign（未登録の差分が出たら赤） |

再利用したもの（再実装していない）:

- 生成と build は harness の `runner::Artifact::FromIr` / `Runner::build`
  （Java は IR → 生成 → `javac`、Rust は IR → 生成 → `cargo build --release` と
  内容 hash キャッシュ、両者への pure scanner 同梱）。
- 差分の表現と JSON Pointer は `ubnfc_harness::canonical::diff_values`（corpus と同じ関数）。
  これを使うために harness へ追加した公開は 2 つだけで、いずれも additive:
  `canonical::diff_values` の `pub` 化と `runner::project_numeric_lexemes`（D-022 の字句射影）。
- front は subprocess ではなく `ubnfc_front::pipeline::compile` を直接呼ぶ。
  discard の理由コード（`E-*` / `W-*`）をそのまま集計に使える。

driver との JSON Lines 交換だけは `crates/ubnfc-fuzz/src/engine.rs` に持った。
fuzzing では片側が停止し得るので、**制限時間で子プロセスを落として「片側だけ応答しない」を
観測として残す**必要があり、harness の `exchange` には制限時間がないため。

### 比較で弱めた点（corpus と同じ契約に限る）

| 弱め | 根拠 |
|---|---|
| `syntax` 診断の主 `offset` を比較しない | D-011（`docs/design/04-cursor-and-diagnostics-contract.md` §2）: Java は失敗 frame 内の `max(C,M)`、Rust は `fail_at` の最遠。言語別 oracle |
| `syntax` 診断の `expected` は非空だけ比較 | D-014: hints の表記は言語別（`RustConformanceTest.java:756-765` と同じ扱い） |
| `farthestOffset` / `farthestExpected` / `kind` は厳密比較 | D-014 の共有契約。`trailing_input` の `offset` / `expected` も厳密 |
| `rulePath` / `deepestRule` / `nativeFailure` / `hints` を比較しない | D-019 / D-020 |
| AST の int field を capture 原文へ射影（Java 側のみ） | D-022。harness の `project_numeric_lexemes` をそのまま使う |
| `mappingError` は有無のみ比較 | 例外クラス名は言語で異なる。Java の `NumberFormatException` と Rust の `null` は D-022 で免除 |

比較対象の wire 項目: `accepted` / `allConsumed` / `consumed` / `matched` / `ast` / `captures` /
`prefixCaptures` / `declarations` / `references` / `scope` / `scopeDepth` / `parsedStatus` /
`diagnostics` / `mappingError` / `recoveries` / `catalogs` / `value`。
**キーの欠落と明示 null は区別する**（protocol の契約）。片側だけ `error` を返した場合も差とする。

### 内部不変条件（backend によらず成り立つべき）

| id | 内容 |
|---|---|
| `cursor-order` | `consumed <= matched` |
| `cursor-range` | `consumed` / `matched` が `[0, 入力長]` |
| `allconsumed-consistency` | `allConsumed == (consumed == 入力長)` |
| `allconsumed-implies-accepted` | `allConsumed` なら `accepted` |
| `ast-canonical` | AST が canonical 形（`{type,span,fields}`）で JSON 往復が一致 |
| `ast-span` / `ast-span-nesting` / `ast-list-order` | span が入力内・親 span の内側・list は文書順 |
| `capture-span` / `capture-text` | capture の span が入力内で、`text` がその範囲の切り出しと一致 |
| `scope-offset-range` / `diagnostic-offset-range` / `recovery-span` | 位置が入力長の外に出ない |

## 2. campaign の規模

`cargo run -q -p ubnfc-fuzz --offline -- --seeds 320 --inputs 24 --jobs 3 --shrink-builds 10 --minutes 60`
（このホストでは他のエージェントがベンチを走らせているので worker は 3 に抑えた）

| 項目 | 値 |
|---|---:|
| 生成した文法（seed 1..320） | 320 |
| front → 両 backend 生成 → build を通った文法 | 292 |
| discard された文法 | 28（9%） |
| build 回数（縮小を含む） | 390 |
| 入力 = 比較件数 | 7,008 |
| うち全入力受理 | 1,897（27%） |
| うち prefix のみ受理（部分消費） | 2,066（29%） |
| うち不受理 | 3,045（43%） |
| driver の error / 制限時間超過 | 0 |
| 最小化まで到達した所見（差分クラス） | 92 |
| 所要 | 1,121 秒（18.7 分） |

受理・部分消費・不受理がほぼ 3 等分になっており、狙った「受理寄りだが拒否も十分ある」
入力分布になっている。driver の異常終了・停止は 1 件も無かった。

この campaign は master の D-023（`perf/rust-diag-two-mode` の「二段構えの解析」、
`docs/reports/2026-09-23-p4-rust-diag-two-mode.md`）を取り込んだ**後**に回している。
取り込み前に同じ seed で回した結果と、集計値も 92 の所見も完全に一致した
（D-023 の「観測は不変」という主張の独立な確認になっている）。

## 3. 生成器の被覆と discard 率

discard は **front（`ubnfc_front::pipeline::compile`）が拒否した文法**の数。
その文法に投入されていた機能すべてに計上しているので、機能ごとの値は
「その機能を含む文法が落ちた率」であり、原因が必ずしもその機能とは限らない。

| 機能 | 生成文法数 | うち discard | discard 率 |
|---|---:|---:|---:|
| `literal` | 320 | 28 | 9% |
| `token:number` | 44 | 4 | 9% |
| `token:identifier` | 38 | 5 | 13% |
| `token:singleQuoted` | 58 | 2 | 3% |
| `token:charRange` | 45 | 7 | 16% |
| `token:negation` | 45 | 4 | 9% |
| `token:ci` | 53 | 4 | 8% |
| `token:any` | 53 | 3 | 6% |
| `token:eof` | 42 | 6 | 14% |
| `token:empty` | 49 | 6 | 12% |
| `token:until` | 62 | 8 | 13% |
| `token:lookahead` | 55 | 4 | 7% |
| `token:negativeLookahead` | 53 | 4 | 8% |
| `sequence` | 319 | 28 | 9% |
| `choice` | 307 | 28 | 9% |
| `optional` | 259 | 26 | 10% |
| `repeat` | 239 | 24 | 10% |
| `group` | 243 | 22 | 9% |
| `quant:plus` | 264 | 25 | 9% |
| `quant:star` | 260 | 26 | 10% |
| `quant:question` | 266 | 27 | 10% |
| `quant:bounded` | 302 | 27 | 9% |
| `separated` | 257 | 26 | 10% |
| `capture` | 319 | 28 | 9% |
| `capture:nested` | 141 | 16 | 11% |
| `capture:parallel` | 197 | 17 | 9% |
| `mapping` | 287 | 27 | 9% |
| `mapping:nested` | 120 | 18 | 15% |
| `mapping:shared` | 120 | 18 | 15% |
| `enum` | 145 | 20 | 14% |
| `commonField` | 53 | 9 | 17% |
| `skip` | 59 | 2 | 3% |
| `whitespace` | 252 | 25 | 10% |
| `interleave` | 112 | 7 | 6% |
| `comment` | 99 | 7 | 7% |
| `leftAssoc` | 159 | 7 | 4% |
| `rightAssoc` | 107 | 15 | 14% |
| `precedence` | 223 | 20 | 9% |
| `longestChoice` | 48 | 6 | 12% |
| `predictiveChoice` | 53 | 6 | 11% |
| `scopeTree` | 109 | 14 | 13% |
| `declares` | 81 | 12 | 15% |
| `backref` | 71 | 12 | 17% |
| `typeof` | 76 | 12 | 16% |
| `recovery` | 100 | 9 | 9% |
| `error` | 187 | 18 | 10% |
| `import` | 43 | 4 | 9% |
| `eval` | 39 | 3 | 8% |
| `catalog` | 62 | 9 | 15% |
| `doc` | 50 | 8 | 16% |

全 50 機能が 38 件以上の文法で実際に使われている。discard は 0 件の機能が無く、
特定機能だけが全滅している状態ではない。discard の内訳は 2 種類だけで、いずれも
**front の内部不整合**（§4.D）であり、生成器が不正な UBNF を書いたからではない:

| コード | 件数 | 内容 |
|---|---:|---|
| `internal` | 19 | `/semantics/recipes/*/fields/*/cardinality: recipe と capture の意味値 cardinality が不一致` |
| `analysis` | 9 | `CaptureCardinality { declared: List, inferred: Scalar/Optional }` |

`E-*`（validate）・`E-LOWER-*`（lowering）・syntax エラーでの discard は **0 件**。
生成器は「左再帰を作らない」「`@mapping` の params は本体の capture と完全一致させる」
「`@enum` の選択肢は Java/Rust の識別子になる語にする」「`@typeof` は `@mapping` のある規則にだけ置く」
といった構成的制約を持たせてあるので、検証エラーはそもそも作られない。

## 4. 所見と三分類

三分類の記号: **(a)** Java のバグ / **(b)** Rust のバグ / **(c)** front・IR のバグ /
**(d)** 契約が決まっていない。

所見は「差分クラス」= JSON Pointer の配列添字を伏せたもの、または不変条件の id 単位で
1 件にしている。1 つの入力に複数のクラスが出たらそれぞれ別に最小化する。
**92 クラス**が出たが、根本原因で束ねると 12 件になる。最小再現例は
`crates/ubnfc-fuzz/findings/<slug>/`（`grammar.ubnf` + `input.txt` + `finding.json` + `README.md`）。
A-2 は入れ子の深さぶん 44 クラスに分かれるので、repo には最小の 1 件だけを置いた
（残りは §5 のコマンドで再生成できる。`crates/ubnfc-fuzz/findings/README.md`）。

| # | 根本原因 | クラス数 | 代表 slug | 三分類 |
|---|---|---:|---|---|
| A-1 | `@recovery` の適用が両者で食い違い、受理そのものが変わる | 13 | `diff-captures-span-94600c41-seed269` | **(a)/(b)** |
| A-2 | 入れ子ノード span に末尾 trivia を Rust だけ含める | 44 | `diff-ast-fields-left-fields-right-span-3fb5191f-seed5` | **(d)→(b)** |
| A-3 | `@enum` を演算子の operand にすると Rust だけ mapping 失敗 | 1 | `diff-ast-189b6e83-seed6` | **(b)** |
| A-4 | 同名の並行 capture のリスト要素が `null` と値で食い違う | 6 | `diff-ast-fields-head-fields-left-fields-left-10db0a79-seed38` | **(a)/(b)** |
| A-5 | 成功した解析に Java だけ投機的 `syntax` 診断を残す（以降の添字が全部ずれる） | 9 | `diff-diagnostics-988b1c0e-seed1` | **(d)** |
| A-6 | `farthestOffset` / `farthestExpected`（共有契約）が一致しない | 3 | `diff-diagnostics-farthestoffset-57125337-seed34` | **(a)/(b)** |
| A-7 | 全入力解析が失敗したのに Rust が `captures` を返す | 1 | `diff-captures-ccc0d8bb-seed1` | **(b)** |
| A-8 | 根が空一致したとき `mappingError` / `value` が食い違う | 2 | `diff-mappingerror-present-1278776d-seed1` | **(b)+(d)** |
| A-9 | scope event の `name` / `action` / `offset`、scope 診断の有無 | 4 | `diff-scope-events-name-869b803a-seed9` | **(b)/(d)** |
| B-1 | driver の wire キー（`recoveries` / `scope` の中身 / `catalogs` / `scope.mode`） | 7 | `diff-recoveries-7906c206-seed1` | **(b)+(d)** |
| C-1 | `allConsumed == (consumed == 入力長)` が成り立たない | 2 | `java-allconsumed-consistency-e6b69b6f-seed3` | **(d)** |
| D-1/D-2 | front が自分の IR / 解析の契約に落ちる（campaign の discard 28 件すべて） | – | `front-ir-recipe-cardinality` | **(c)** |

（A-1 … C-1 のクラス数の合計は 92 で、所見の総数と一致する。）

### A-1 `@recovery` の適用が食い違い、受理そのものが変わる — **(a)/(b)**

最小再現（`diff-captures-span-94600c41-seed269`、入力 `" ;;"`）:

```ubnf
grammar Fuzz {
  token T0 = EOF
  @recovery(AUTO) M0 ::= T0? 'c';
  @root @mapping(Top, params=[head]) Root ::= M0 @head ';' @head;
}
```

| 観測 | Java | Rust |
|---|---|---|
| `allConsumed` | `true` | `false` |
| `consumed` | 3 | 2 |
| `ast` | `{"type":"Top","span":[0,3],"fields":{"head":[";",";"]}}` | `null` |
| `captures[0]` | `{"span":[0,2],"text":" ;"}` | `{"span":[0,1],"text":" "}` |

逆向きの例もある（`diff-accepted-02147df8-seed269`、入力 `"c;"`、
`@recovery(AUTO) M0 ::= T0 T0? 'c';`）: Java は `accepted=false` / `consumed=0`、
Rust は `accepted=true` / `allConsumed=true` / `consumed=2` で AST まで作る。

`@recovery` が失敗をどこまで飲み込み、飲み込んだ範囲を capture / AST にどう投影するかが
両者でまったく違う。`docs/decisions/D-018-undecided-divergences.md` の名前付き契約
`recovery-projection` / `recovery-diagnostics` と D-021（`03-design-v1.md` §8）で
「Java も `@root`・直接 entry の rule に recovery wrapper を適用する」までは決まっているが、
**非 entry の rule に付いた `@recovery` の適用範囲と AST 投影**は決まっていない。
corpus の recovery 系 fixture（`oracle/recovery-*.json`）は決まった形の入力しか持たないため通る。
このクラス（`/accepted` `/allConsumed` `/consumed` `/matched` `/parsedStatus` と
`/captures/*/{name,span,text}` `/prefixCaptures/*{,/binding,/name,/span,/text}`）が 13 件。

### A-2 入れ子ノード span に末尾 trivia を Rust だけ含める — **(d) → 実質 (b)**

最小再現（`diff-ast-fields-left-fields-right-span-3fb5191f-seed5`、入力 `"😀+😀\t"`）:

```ubnf
@whitespace: javaStyle
@mapping(Bin0, params=[left, op, right]) @rightAssoc @precedence(level=90)
Op0 ::= T2 @left { '+' @op Op0 @right };
```

`/ast/fields/left/fields/right/0/span/1` が Java=3 / Rust=4。末尾のタブを Rust は
入れ子ノードの span に含める。コメントでも同じ（`'q' -   ''-'😀' \tab,/* c */` で Java=21 / Rust=23）。
入れ子が深いほど同じ差が別パスで出るので **44 クラス**（全体の半分）を占めるが原因は 1 つ。
`docs/design/03-design-v1.md` D-007 は「公開 span は code point」しか決めておらず、
**trivia をノード span の端に含めるか**が無い **(d)**。ただし root span については
corpus が `[0, 入力cp長]` を要求しており（`docs/compat-matrix.md` の P/L 規則）、
内側だけ trivia を含める Rust の挙動は一貫していない。

### A-3 `@enum` を演算子の operand にすると Rust だけ mapping に失敗する — **(b)**

`diff-ast-189b6e83-seed6`、入力 `"alpha;"`:

```ubnf
@enum L0 ::= 'alpha';
@mapping(Bin0, params=[left, op, right]) @rightAssoc @precedence(level=90)
Op0 ::= L0 @left { '*' @op Op0 @right };
@root @mapping(Top, params=[head]) Root ::= Op0 @head ';' @head;
```

`accepted` / `allConsumed` / `consumed` / `matched` / `captures` は**完全に一致**（ともに全入力受理）。
それでも Java は `ast = {"type":"Bin0","span":[0,5],"fields":{"left":"ALPHA","op":[],"right":[]}}`、
Rust は `ast = null` で `mappingError = "java.lang.IllegalArgumentException: left: mapped value type mismatch"`。
構文が同一に成功しているので mapping 実装の片方が誤り。AST を作れていない Rust 側 **(b)**。
なお Rust runtime が **Java の例外クラス名をそのまま**メッセージに入れている（A-8 と同根）。

### A-4 同名の並行 capture のリスト要素 — **(a)/(b)**

`diff-ast-fields-head-fields-left-fields-left-10db0a79-seed38`（空入力）で
`/ast/fields/head/fields/left/fields/left/0` が Java=`null` / Rust=`""`。
`diff-ast-fields-head-*-seed9`（入力 `gammaSELECTSELECT`）では `/ast/fields/head/1` が
Java=`null` / Rust=`"SELECTSELECT"`。同じ名前の capture を 2 か所に置いたときの
list 要素の埋め方が違う。`03-design-v1.md` §5 は「動的 capture は完了順・同名 nested は内側→外側」
とだけ書き、**同名の並行（兄弟）capture のリスト化**を規定していないので、
どちらが正かは仕様側の決定待ち。

### A-5 成功した解析に Java だけ投機的 `syntax` 診断を残す — **(d)**

`diff-diagnostics-988b1c0e-seed1`（空入力、根が epsilon に一致）:
`accepted=true` / `allConsumed=true` なのに Java の `diagnostics` は
`[{kind:"syntax", farthestOffset:0, ...}]`、Rust は `[]`（同じ内容を `hints` に入れている）。
protocol は「parser が成功経路内の失敗を保持する場合もそのまま返す」と両方を許しており **(d)**。
harness の Oracle 経路が `allConsumed==true` のとき `syntax` / `trailing_input` を捨てるので
corpus には出ない。この 1 件の有無で以降の添字が全部ずれるため
`/diagnostics/*/{kind,offset,length,message,severity,expected,expectedNonEmpty}`
の 9 クラスに波及している（代表 `diff-diagnostics-kind-6cf65a39-seed40`）。

### A-6 共有契約のはずの `farthestOffset` / `farthestExpected` が一致しない — **(a)/(b)**

`diff-diagnostics-farthestoffset-57125337-seed34`（入力 `"*"`、両者とも不受理）:
`farthestOffset` が Java=1 / Rust=0 で、`farthestExpected` の中身も別物
（Java `["'*'", "fuzz error"]` / Rust `["' '", "'('", "'*/'", …, "fuzz error"]`）。
`diff-diagnostics-farthestexpected-d88564a1-seed2` では Java 側にだけ合成 repeat parser の
表示名 `"Repeat"` が混じる。D-014（`04-cursor-and-diagnostics-contract.md` §2）は
主 offset だけを言語別 oracle とし、**最遠到達（`farthestOffset` / `farthestExpected`）は共有契約**
としている（corpus も両 backend で厳密比較して通っている）。したがってこれは契約違反。

### A-7 全入力解析が失敗したのに Rust が `captures` を返す — **(b)**

`diff-captures-ccc0d8bb-seed1`（入力 `"#"`、`allConsumed=false` / `ast=null`）:
Java `captures = []`、Rust `captures = [{"name":"head","span":[0,0],"text":""}]`。
protocol は `captures` を「**全入力解析**で採用した出現の順序付き列」と定義しているので、
全入力解析が成立していない以上、採用出現は無い。Rust が prefix 側の出現を漏らしている。
corpus では Oracle 系統が `prefixCaptures` を見るため通る。

### A-8 根が空一致したときの `mappingError` / `value` — **(b) + (d)**

`diff-mappingerror-present-1278776d-seed1` / `diff-value-1e979fc2-seed1`（空入力）:

| 観測 | Java | Rust |
|---|---|---|
| `mappingError` | `null` | `"java.lang.IllegalArgumentException: entry must produce one AST value"` |
| `value` | キーごと出さない（未観測） | `null`（観測済みの null） |

`entry が AST 値を作らないのは mapping 失敗か` は契約が無い **(d)**。
Rust runtime のメッセージが Java の例外クラス名である点は実装漏れ **(b)**。
`value` は「観測済みの `null` と未観測のキー欠落を区別する」という protocol の中核に反する。

### A-9 scope の観測 — **(b)/(d)**

- `diff-scope-events-name-869b803a-seed9`: enter / leave イベントの `name` が
  Java=規則名（`"M1"`） / Rust=`null`。protocol は `events` の形しか書いておらず、
  enter / leave の `name` の意味が無い **(d)**。値を落としている Rust が情報量として劣る。
- `diff-scope-events-{action,offset}-*-seed92`: イベント列そのものが違う。
- `diff-scope-events-b247d3d4-seed92`: `@backref` の未解決シンボル警告
  （`"未定義のシンボル: ''"`）を Rust は出し Java は出さない。

### B-1 driver の wire 形式 — **(b) + (d)**

いずれも corpus が比較していない項目で、`docs/harness-driver-protocol.md` の記述とも食い違う。

| slug | 差 | 三分類 |
|---|---|---|
| `diff-recoveries-7906c206-seed1` | Java は `recoveries`、Rust は `recoveredRegions`。要素も `skippedSpan` / `syncPattern` / `captureOccurrences`（Java）と `captureOccurrenceIds`（Rust）で名前が違う | **(d)** protocol に両方とも書かれていない + **(b)** 実装が揃っていない |
| `diff-scope-{declarations,depth,diagnostics,references}-*-seed1` | protocol の追記は `scope` を `{mode, events, declarations, references, diagnostics}` と定義しているが、Rust の `scope` は `{mode, events}` しか持たない | **(b)**。harness の `scope_fields` が top-level へ fallback するので corpus では露出しない |
| `diff-scope-mode-f904d076-seed3` | `scope.mode` の意味が違う。Java は**観測されたイベントの mode を distinct したもの**（イベントが無ければ `[]`）、Rust は**宣言された mode**（`["dynamic"]`） | **(d)** |
| `diff-catalogs-dd4854a9-seed1` | Rust は `catalogs`（`@catalog` の静的表）を返し、Java は返さない | **(d)** protocol に記載が無い |

### C-1 内部不変条件

| id | 結果 |
|---|---|
| `cursor-order` (`consumed <= matched`) | 違反なし（7,008 比較 × 2 backend） |
| `cursor-range` | 違反なし |
| `allconsumed-implies-accepted` | 違反なし |
| `ast-canonical` / `ast-span` / `ast-span-nesting` / `ast-list-order` | 違反なし |
| `capture-span` / `capture-text` | 違反なし |
| `scope-offset-range` / `diagnostic-offset-range` / `recovery-span` | 違反なし |
| `allconsumed-consistency` | **両 backend で違反**（Java 355 件 / Rust 355 件） |

`allConsumed == (consumed == 入力長)` は成り立たない。最小例
（`java-allconsumed-consistency-e6b69b6f-seed3`、空入力）は
`allConsumed=false` / `consumed=0` / 入力長 `0`。
2 つのカーソルが**別々の解析**から来ることが理由で、driver は prefix 解析（`requireAll=false`）から
`consumed` / `matched` を、全入力解析（`requireAll=true`）から `allConsumed` を取る
（`Driver.java:95-100`、`crates/ubnfc-rust/templates/driver.rs`）。
protocol の「`allConsumed`: 構文受理かつ入力全体を消費したか」は「消費し切ったが構文は失敗」を
排除できていない **(d)**。正しい言い換えは `allConsumed == (全入力解析が成功したか)` で、
`consumed` からは導けない。両 backend が同じ振る舞いなので実装差ではなく文書の不足。
`capture-text`（`text` が `input[span]` の切り出しと一致）が 14,016 観測で 1 件も破れていないのは、
span の単位（code point）と非 BMP の扱いが両 backend で正しいことの独立な確認になっている。

### D front（runtime 差ではないが campaign が露出させたもの）

campaign の discard 28 件は**すべて** front の内部不整合で、`ubnfc validate` は通るのに
`ubnfc ir` が落ちる形。いずれも **(c)**。

- `front-ir-recipe-cardinality`（19 件）:
  `@mapping(N) N ::= 'n'; @root @mapping(C, params=[v]) R ::= [ N? ] @v;` が
  `内部エラー: 生成 IR が契約に違反しています /semantics/recipes/1/fields/0/cardinality`。
  `[ N ] @v`（`?` なし）や `[ 'n'? ] @v`（mapped node でない）は通る。
- `front-capture-cardinality-bounded`（9 件）:
  `'a'{1} @v` / `'a'{0} @v` / `'a'{0,1} @v` / `( 'a' @v ){1}` が
  `CaptureCardinality { declared: List, inferred: Scalar/Optional }`。
  lowering は Bounded を無条件に `List` と宣言し、analysis は `max <= 1` を `Scalar`/`Optional` と推論する。

どちらも `cargo run -p ubnfc-fuzz -- --minimize-front <seed>`（seed 15 / 32）で
front だけを使って自動最小化できる。各ディレクトリの `README.md` に修正案を書いた。

## 5. 再現方法

```sh
# 前提: cargo build --bin ubnfc --bin ubnfc-rust && (cd ubnfc-java && mvn -o -q -DskipTests package)

# CI 用（固定 seed・数十秒。未登録の差分が出たら赤）
cargo test -p ubnfc-fuzz --offline

# 本稿の campaign
cargo run -q -p ubnfc-fuzz --offline -- \
  --seeds 320 --inputs 24 --jobs 3 --shrink-builds 10 --minutes 60 \
  --findings crates/ubnfc-fuzz/findings --report out.md --json out.json

# 生成される文法と front の判定だけを見る（生成器の triage 用）
cargo run -q -p ubnfc-fuzz --offline -- --start-seed 1 --dump 8
```

seed から文法・入力を完全に再現する。`--start-seed N --dump 1` で所見の seed の文法を再表示できる。
生成 crate の `cargo build --release` は最適化レベルだけを 0 に落としている
（`ubnfc_fuzz::tune_generated_builds`。`debug-assertions` / `overflow-checks` は release の既定のまま）。
1 文法あたりの build が campaign 時間を支配するためで、観測される振る舞いは変わらない。

## 6. この fuzzer が見ていないもの

- **両 backend が同じように間違っている場合**は検出できない。差分 fuzzing の原理的な限界で、
  そこは corpus と独立 oracle の担当。
- 生成文法は「左再帰なし・非終端は深さで階層化」という構成的制約を置いている。
  左再帰（`W-LEFT-RECURSION` / `E-LOWER-*-LEFT-RECURSION`）の runtime 差は未検証。
- `REGEX` token は生成していない（Rust が非正則 REGEX を生成時拒否する契約差があり、
  discard 率だけが増えて runtime の比較にならないため）。
- `@import` は 2 ファイル・1 規則の最小形のみ。alias 衝突や循環は front の拒否契約なので対象外。
- 入力は 1 文法あたり 24 件・数十 code point 規模で、大きな入力での性能差・memo の差は見ていない。
- `entry` は常に `@root`。任意 entry での差は未検証。

## 追記（2026-09-24、issue #41 / D-069）

v2 seed 帯 100276 が、この campaign の帯（seed 1..320）が踏んでいなかった
`@recovery` の観測差を 2 つ出した（`diff:/diagnostics/*/expected/*` と
`diff:/recoveries/*/captureOccurrences/*`）。いずれも Rust backend の欠陥で、
同じ原因が同 seed の `diff:/diagnostics/*/farthestExpected/*`（§4 A-6 の残り）も説明する。
triage と修正は [`2026-09-24-fuzz-recovery-observations.md`](2026-09-24-fuzz-recovery-observations.md)。

`diff:/diagnostics/*/farthestExpected/*` は**クラスとしては `KNOWN` に残る**
（`@leftAssoc` の静的失敗ヒント計画と Extern token の表示名という別原因が seed 34 / 4001 に残る）。
