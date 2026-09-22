# P4 Rust 生成 parser の性能改善 M5 round 2（2026-09-22）

[`2026-09-22-p4-rust-perf-m5.md`](2026-09-22-p4-rust-perf-m5.md)（round 1: complex 0.50 ms、x64 60〜92 ms）の続き。
目標は complex ≤ 0.2 ms（AST 付き）、complex-x64 ≤ 15 ms。意味論は変更していない（corpus は round7 / M5 と同じ verdict、
selfhost 固定点、P4 golden AST byte 一致、`cargo test --workspace` 全緑、clippy `-D warnings`、fmt）。

## 結論

- complex.tiny（332 byte）は **認識 0.240 ms / AST 0.312 ms**（round 1 の同セッション再計測 0.469 / 0.502 ms）、
  complex-x64 は **認識 33.2 ms / AST 35.2 ms**（同 53.7 / 61.7 ms）。
  M5 前（25.5 ms / 1,922 ms）比で complex 約 80 倍、x64 約 55 倍。目標（0.2 ms / 15 ms）にはまだ届いていない。
- rule 評価 2,455 → 1,063 / parse（memo hit 816 → 463、memo entry 1,639 → 765）、確保 2,754 → 1,822 回（AST）。
- 主な変更は **通常の `|` への保守的な候補除外**（設計 v1 §3.4 が許す「保守的証明付き除外 + 失敗診断の同値」）で、
  trivia 越しの FIRST 集合と、除外した候補が記録したはずの診断の静的再生を emitter に実装した。
  `ParseOptions::predict`（既定 true）で参照経路（全候補試行）に戻せる。
- **precedence climbing（D-004）は P4 では適用していない**。front の解析は P4 の全塔を `tierFunctions` と判定し、
  塔も浅い（Number 2 段・String 1 段・Boolean 3 段）ので、適用しても rule 評価の 2.5% しか減らない（下記）。

## 手順 1: precedence climbing の適用可否（D-004）

front の `analysis/operators.rs` は塔ごとに `plan.operators[].strategy` を出す。P4 IR の 6 塔は全て `tierFunctions`:

| 塔 | 判定 | 理由（`proof.explanation`） |
|---|---|---|
| NumberExpression → NumberTerm | tierFunctions | 到達領域に trivia・副作用・recovery・先読み等がある、または塔が循環する |
| StringExpression | tierFunctions | 同上。基底の前進を証明できない。canonical operator metadata 外の capture を lift へ投影できない |
| BooleanExpression → BooleanAnd → BooleanXor | tierFunctions | 同上。canonical operator metadata 外の capture を lift へ投影できない（各段） |
| AnnotationParameters / MethodParameters / Arguments | tierFunctions | 注釈のない `X { ',' X }` 形。到達領域に trivia、lift recipe なし |

理由の中身: (1) P4 は文法全体が `@whitespace: javaStyle` で、塔の到達領域（基底の NumberFactor 以下は文法のほぼ全体）に
trivia 境界と Extern token（STRING）と scope 効果（VariableRef の `@backref`）がある。D-004 の注記どおり trivia があると
`lift` の子 span 複製が使えず、段ごとの入口/出口と区切り位置の境界計画が要る。(2) 解析は「塔の到達領域全体」を見るので、
基底（atom）を黒箱として扱う緩和をしない限り、実用文法では常に不適格になる。

適用した場合の上限も見積もった。counters（round 1 末尾の状態、complex）で段 rule の呼出しは NumberTerm 30・BooleanAndExpression 15・
BooleanXorExpression 16 の計 61 回 / 2,455 回（2.5%）。climbing が省けるのは段の rule 呼出しと段ごとの Seq/Repeat の expression 呼出し
だけなので、trivia 境界計画を実装しても complex で 10 µs 程度（2〜3%）にしかならない。保守経路（段ごとの関数）を維持し、
解析の緩和と境界計画は別 issue に残す（末尾「残り」）。

## 手順 2: 型付き Expression の共有と候補除外（本 round の主要変更）

### 何が遅かったか

`Expression ::= NumberExpression | BooleanExpression | StringExpression | ObjectExpression | …` の各塔は共通の primary
（`$var`、`if`、`(`、関数名）を持つ。primary の解析自体は rule 本体の memo で共有されている（round 1 で hit 816 / parse）が、
その手前で **rule 参照で始まる候補が最初の token で失敗する**呼出しが大半を占めていた。例: `$price` の位置で NumberFactor は
TernaryExpression（`(`）・NumberMatchExpression（`match`）・IfExpression（`if`）・MathFunction（14 の関数名 rule）・ToNum・
LengthDotMethod・Len・Length・ExternalNumberInvocation・NUMBER を順に試してから VariableRef に至る。失敗 1 件は
rule 入口検査・mark/restore・memo lookup/store（snapshot の label 複製）・局所 frame の enter/leave・display の failures を伴い、
1 回 150〜200 ns だった。front の `mayStart` は trivia を含めた先頭文字集合なので `unknown`（空白が先頭になり得る）で、
plan の `predict` 戦略は literal だけの選択にしか使えなかった。

### 候補除外の設計（`crates/ubnfc-rust/src/predict.rs`）

1. **trivia 越しの先頭文字集合**。候補 expression ごとに `Start = Unknown | Known { nullable, direct, skipped, policy }` を
   最小不動点で求める。`direct` は候補の呼出し位置で読む文字の集合、`skipped` は候補が最初の token の前に必ず通る
   trivia policy（区切り集合が等しい policy は同一視）で読み飛ばした位置で読む文字の集合。Seq の入口 / 各要素後の trivia 境界で
   `wrap`（direct を skipped へ移す）し、policy が混在すれば Unknown。literal（大小区別あり）、組込み token（Number は
   `+ - . 0-9`、Identifier は英字と `_`、引用符、文字集合、DFA の開始遷移）だけ Known で、Extern（P4 の STRING）・先読み・
   Empty/Eof/Any・nullable な候補・`childMode` の切替は Unknown。
2. **失敗経路の静的再生**。除外できるのは「その候補を実行しても最初の significant token で失敗する」と証明できた場合だけで、
   そのとき候補が記録したはずの観測を同じ順序・同じ位置で再生する `guard_e{i}` 関数を生成する。再生する経路は
   Seq（先頭要素）・Choice（全候補）・Group・RuleRef（入口 scope イベントと recovery のない rule）で、葉の literal は
   `Failure(label)@位置` の主診断 node と display label、token 葉は本物の expression 関数を呼ぶ（token 内部の記録位置は
   token ごとに違う）。rule は `Rule` node、choice は `Join` を同じ形で作る。display の label は「最初の trivia より前」
   「trivia 後の位置 T」「trivia 後に T より手前へ記録するもの」に分け、後者は位置が T と等しいときだけ記録する
   （frame の `open` は far より手前を捨て、同じ位置は連結するので同値）。trivia は同じ policy で冪等なので経路上の最初の
   1 回だけ本物を呼ぶ（未終端 block comment の label もそこで出る）。再生関数は経路を共有せずに展開するので、
   展開の node 数が `MAX_TEMPLATE_SIZE`（192。P4 の最大は NumberExpression の 61）を超える候補は除外しない
   （例: `tests/round2.rs` の `D28 ::= D27 | D27` 連鎖は 2^28 に広がるが、本物の経路は memo で線形。上限が無いと
   生成が終わらず `shared_diagnostic_dag_is_bounded` が落ちる）。
3. **本物との差は memo と局所 frame だけ**。再生は memo の lookup/store をせず（memo 有無の同値は既存 oracle
   `tests/round2.rs` と `fixtures.rs` の memo off 比較で保証済みの性質）、局所 frame を作らない（`leave` の伝播規則は
   `open` の位置比較と同じ）。予算（`limits.diagnostics`）の計数は「記録の回数」なので、まとめて記録する分だけ減る
   （既定 200 万で全 fixture 受理、予算 200 で Resource 失敗、の両テストは維持）。深さ上限の近傍・予算枯渇・確定失敗の
   状態では再生せず本物の候補関数へ戻す（`Session::can_replay`）。`rule_evaluations` は再生分を数えない。
4. **選択の生成**（`expressions.rs`）: 除外対象の候補があれば呼出し位置の文字 `c0` と、純粋な読み飛ばし `skip_{p}`
   （診断を記録しない trivia）で求めた位置の文字 `ct` を 1 回だけ読み、候補ごとに `matches!(c as u32, …)` で判定する。
   `invert` 状態では判定しない（predict_literals と同じ）。

P4 では 41 の選択のうち 22 で 96 候補が除外対象になった。除外できない主な候補: STRING（Extern）で始まる StringTerm /
SliceBaseReceiver の候補、`SliceBaseExpression` の 8 候補（全て同じ SliceBaseReceiver で始まるので先頭文字では区別できない）、
literal だけの選択（葉は本物を呼ぶ。plan の predict はそのまま）、nullable で始まる候補。

### 左因数分解（共有 primary を 1 回だけ解析して 4 塔の継続を試す）

見送った。理由: 型付き 4 塔の分岐は candidate の Seq 単位で capture site と `Values` イベントが違い、共通 primary の
イベント木を候補ごとに包み直す必要がある。PEG 意味論としては「X が同じ状態で同じ結果を返す」（memo が保証）ので
`X A | X B ≡ X (A | B)` は成り立つが、診断 DAG（`Rule(X)` を候補ごとに `Join` する形）と capture の投影を同値に保つ変換を
emitter で証明する作業が本 round に収まらなかった。SliceBaseExpression / SliceNestedExpression の 8 候補 × 2 段（memo hit の
rule 呼出し 1 parse 約 330 回、全 rule 呼出しの約 1/4）がこの形なので、次 round の第一候補にする。

## 手順 3: trivia の高速経路

生成 `trivia_{p}` の各周回の先頭で、現在 byte が区切りの先頭 byte（空白 6 種と `/`）でなければ、各区切り block が記録するはずの
候補 label（`' '`, `'//'`, `'/*'`, `'*/'`）を 1 回の `failures` で記録して抜ける（記録内容・位置は同じ）。行コメント 1 種だけの
policy が作る主診断 node も同じ位置で作る。区切りの無い policy には置かない（`-D warnings` で「loop never loops」）。
per-(position, policy) の skip cache は置かなかった: 高速経路後の trivia 呼出しは空白の無い位置で 10 ns 程度で、
cache の lookup（hash + probe）より安い。空白のある位置の再走査は memo miss の再訪だけで短い。

## 手順 4: memo

- entry の縮小はしていない（`Saved` 48 byte + key 20 byte = 72 byte のまま）。候補除外で失敗 entry が 1,639 → 765 に減り、
  事前確保の見積りを 6 → 3 entry / byte に下げた（arena は 8 → 5 event / byte）。
- 失敗 entry の省略: 「再訪しない」証明は plan の `provedOmission`（単一実行経路）と同じ性質で、候補除外後の失敗は
  memo せず再生するので、残る失敗 entry は memo hit で実際に再訪されるもの（SliceBase 系）が中心。省略しない。
- 到達位置（display の `reached`）の記録を葉・trivia・memo frame・recovery に限定した（複合式と rule の入口/出口の位置は
  いずれも子の出口か自身の入口の位置に一致するので、記録しなくても最大値は変わらない）。`display_reach` 22,700 → 約 9,000 / parse。

## 手順 5: AST 経路の確保

- 効果（scope の Declare/Use）を `Rc<ScanEffect>` で共有し、memo の save/replay で文字列を複製しない。
- AST 構築の走査 stack（`owned_captures` / `build_values`）を Session の pool から再利用。
- `text_spans`（mapped text の pointer → span）を乗算 hash の `PtrHasher` に。
- 残り: `ValueSpan.path` / `text` の String（結果 API）、node ごとの `Box`、`field_text` の Vec<String>。認識 → AST の差は
  約 70 µs（131 node）。

## 段階別 A/B（`examples/p4-rust` bench、release、warmup 後 5 run の中央値、ms/parse）

共用ホストで Java M5 の計測が並走しており（load average 3〜12）、段階 2〜6 の絶対値は run ごとに 1.5 倍程度ぶれる
（同じ binary で x64 認識が 32〜65 ms）。段階 1〜5 は各段階の直後に 1 回ずつ計った値、最終 2 行は「最終 A/B」
（baseline binary と最終 binary を同じセッション、load average 8〜10 で交互に 2 回ずつ実行し、各列の小さい方の中央値）。allocs / bytes は GlobalAlloc の計数（AST 付き）。

| 段階 | complex 認識 | complex AST | x4 AST | x16 AST | x64 認識 | x64 AST | rule 評価 | memo hit | allocs (AST) | bytes (AST) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| round 1 末（27a7f76、作業開始時 load 12） | 0.500 | 0.554 | 3.01 | 8.65 | 62.1 | 78.8 | 2,455 | 816 | 2,754 | 1.71 MB |
| 1 候補除外（関数ごとの再生）+ trivia 高速経路 | 0.359 | 0.445 | 1.49 | 6.23 | 44.9 | 61.6 | 1,394 | 629 | 2,753 | 1.38 MB |
| 2 再生の静的畳み込み | 0.280 | 0.372 | 1.34 | 5.36 | 32.4 | 36.3 | 1,394 | 629 | 2,753 | 1.38 MB |
| 3 確保削減（Rc 効果・stack pool・hasher・容量） | 0.265 | 0.340 | 1.97 | 4.85 | 40.0 | 37.7 | 1,394 | 629 | 1,822 | 1.17 MB |
| 4 reach の限定 | 0.256 | 0.330 | 1.91 | 4.83 | 39.8 | 38.6 | 1,394 | 629 | 1,822 | 1.17 MB |
| 5 入口イベントの無い rule も再生（VariableRef / MethodInvocation） | 0.296 | 0.366 | 2.36 | 6.08 | 33.1 | 36.8 | 1,063 | 463 | 1,822 | 1.17 MB |
| 最終 A/B: baseline | 0.469 | 0.502 | 2.73 | 7.54 | 53.7 | 61.7 | 2,455 | 816 | 2,754 | 1.71 MB |
| 最終 A/B: 本 round | 0.240 | 0.312 | 1.11 | 4.35 | 33.2 | 35.2 | 1,063 | 463 | 1,822 | 1.17 MB |

counters（`--features counters`、complex 認識、計測込み）: rule 関数呼出し 2,455 → 1,063、expression 関数呼出し 7,205 → 3,420、
再生関数呼出し 919 回（自己時間 0.10 ms、計測 overhead 込み）、memo store 1,639 → 765、
display failures 7,535 → 6,854、display reach 20,711 → 6,657、diag push 5,918 → 6,661。

## 最終値と目標

| 入力 | bytes | 認識 ms | AST ms | µs/byte (AST) | 目標 | 手書き目安 0.4 µs/byte |
|---|---:|---:|---:|---:|---:|---:|
| complex | 332 | 0.240 | 0.312 | 0.94 | ≤ 0.2 ms | 0.13 ms |
| complex-x4 | 1,293 | 1.51 | 1.11 | 0.86 | — | 0.52 ms |
| complex-x16 | 5,179 | 6.20 | 4.35 | 0.84 | — | 2.1 ms |
| complex-x64 | 20,923 | 33.2 | 35.2 | 1.68 | ≤ 15 ms | 8.4 ms |

## 契約の検証

- `docs/design/04-cursor-and-diagnostics-contract.md` §1（cursor）: 生成 cursor コードは `State::entered`（再生が使う
  reset/begin の合成、既存操作の組合せ）を足しただけで遷移は不変。§2（診断）: 主 offset / expected / rulePath は主診断 DAG、
  farthest / farthestExpected は display frame で、再生はどちらも同じ形・同じ位置で作る。
- `03-design-v1.md` §3.4: 「生成器は通常の `|` にも予測を適用してよいが、候補除外は保守的証明付きで、除外後の失敗診断が元と同値」
  に従う。§3.7: memo key / 再生は不変（再生関数は memo に触らない）。§3.11: 保守的生成（段ごとの関数）を維持。
- D-004: 適用せず（上記）。既存 AST 契約（`left + op[] + right[]`、span）は不変で、P4 golden と handwritten-target の
  Operators / Power golden は byte 一致（`crates/ubnfc-rust/tests/generated.rs`）。
- 同値の検証: `examples/p4-rust/tests/fixtures.rs::predict_matches_reference_path_on_truncated_and_corrupted_inputs` が
  全 16 fixture の完全入力・各 byte 境界の切り詰め・1 文字破壊（大きい fixture は間引き）で `predict` on/off × memo on/off の
  ParseResult（診断・farthest・rulePath・AST・capture・scope イベント）が等しいことを確認する（比較 約 3,900 件）。
- corpus: `docs/reports/2026-09-22-rust-corpus-m5r2.{md,json}`。系統別の表が round7 / M5 と同じ
  （Pass 1292 / 期待拒否 Pass 30 / Known 28 / Fail 0 / 未観測 75）。
- selfhost 固定点（`crates/ubnfc-selfhost/tests`）緑、`src/generated` を再生成して commit（ubnf.ubnf の 6 選択のうち 39 候補が除外対象）。

## 残り

1. **共通接頭辞を持つ候補列の左因数分解**（SliceBaseExpression / SliceNestedExpression の 8 候補 × 2 段、型付き 4 塔）。
   memo hit の rule 呼出し（463 / parse）の大半で、1 hit は rule 入口検査・mark・lookup・snapshot merge・効果再生・`Rule` node で
   50〜80 ns。rule 単位の memo（wrapper 後の Step を保存）でも hit の後半を省ける。
2. Extern token の先頭文字集合。`@memoSafeToken` と同様の利用者宣言（例 `@mayStart`）があれば STRING で始まる候補も除外できる。
3. precedence climbing の解析緩和（基底を黒箱にし、塔の段だけを見る）と trivia 境界計画。P4 では 2〜3% なので優先度は低い。
4. AST 経路: 結果 API の String（`ValueSpan.path` / `text`）と node ごとの `Box`。
5. x64 の µs/byte が complex の 1.5〜2 倍になる cache 効果（memo 表 45,558 entry × 72 byte、診断 arena）。entry の縮小
   （`effects` / `display.labels` を失敗 entry から外す）は未着手。

## 再現

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cargo run -q --offline --release -p ubnfc-p4-rust --example bench -- complex complex-x4 complex-x16 complex-x64
cargo run -q --offline --release -p ubnfc-p4-rust --features counters --example bench -- --counters complex
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust --report docs/reports/2026-09-22-rust-corpus-m5r2.md --json docs/reports/2026-09-22-rust-corpus-m5r2.json
```
