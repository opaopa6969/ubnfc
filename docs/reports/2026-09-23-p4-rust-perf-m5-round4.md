# P4 Rust 生成 parser の性能改善 M5 round 4（2026-09-23、最終手前の round）

[`2026-09-22-p4-rust-perf-m5-round3.md`](2026-09-22-p4-rust-perf-m5-round3.md)（round 3 末 efd1114: complex 認識 0.180 / AST 0.228 ms、
complex-x64 認識 14.7 / AST 15.7 ms）の続き。目標は complex ≤ 0.2 ms（AST 付き）、complex-x64 ≤ 15 ms（AST 付き）。
意味論は変更していない（corpus は round 3 と同じ verdict、selfhost 固定点、P4 golden byte 一致、差分テスト緑、
`cargo test --workspace` 全緑、clippy `-D warnings`、fmt）。時間を区切った round（約 1.5 時間）で、
候補が尽きた時点で止めた。

## 結論

- complex.tiny（332 byte）は **認識 0.157 ms / AST 0.212 ms**（最終 A/B の同セッション baseline 0.181 / 0.241 ms、
  認識 −13%、AST −12%）。**目標 0.2 ms には届かない**（中央値 0.212、9 run の最小 0.200。中央値で差 6%）。
- complex-x64 は **認識 11.1 ms / AST 15.5 ms**（同 baseline 14.7 / 17.5 ms、認識 −24%、AST −11%）。
  認識は 15 ms を大きく下回るが AST 付きは **15 ms に届かない**（中央値 15.47、最小 13.6）。AST 側の finish
  （出現収集 1.4 ms・AST 構築 1.6 ms・value span 0.45 ms）は round 3 から変わっていない。
- 最終 A/B は、先に載せた段階別の計測（load average 2.5〜6、Java M5 の計測が並走、x64 で ±1.5 ms の雑音）とは別に、
  比較的静かな時間帯に `taskset -c 28` で 1 コアへ固定し、baseline binary と交互に 9 回ずつ実行して取り直した
  （complex 以外の x4 / x16 も同時に計測）。段階別の値と最終 A/B の値が数 % ずれるのはこの計測条件の差による。
- 効いたのは **display（公開 farthest）の失敗記録の記録時重複除去と記録単位の変更**（認識 −25 µs）、
  **finish の不要走査の省略**（recovery 走査・認識だけのときの出現走査）と AST 構築の確保削減、
  **現在 frame の field 化**（−5 µs）。試して**不採用**にしたのは event arena の 36 byte 圧縮（complex で +8 µs、
  読み出しの展開が帯域の節約を上回る）と rule 入口検査の inline 化（+5 µs、i-cache）。

## 上限見積り（どこに時間があるか）

生成物を一時的に書き換えて機能を無効化し、complex を計測した（結果は変わるので採用しない。round 3 末の binary 比）。

| 無効化した機能 | complex 認識 | complex AST | x64 認識 | x64 AST | 読み取り |
|---|---:|---:|---:|---:|---|
| baseline | 0.184 | 0.233 | 13.3 | 15.4 | — |
| display の fail / failures（reach は残す） | 0.126 | 0.169 | 9.3 | 10.8 | **失敗記録が認識の 32%**（5,400 failures + 657 fail / parse） |
| 主診断 arena の push（本 round の手順 3 の上に） | 0.149（0.163 比） | — | — | — | 主診断 DAG は 14 µs（5,947 push / parse） |
| event arena の push | 0.156 | — | — | — | event 記録 + finish の走査で 28 µs（うち走査 9 µs） |

display の記録の内訳（complex、1 parse）: `open` の結果は floor より手前で捨てる 2,307、frame の最初 411、より遠い 123、
同じ位置 3,371。label の push は 11,607 個（3,045 回の failures、平均 3.8 label）、64 個の閾値と snapshot の dedupe が
**706 回・延べ 25,406 label** を走査していた（同じ位置に同じ label 列が繰り返し積まれ、そのたびに全体を詰め直す）。

## 手順 1: finish の不要走査の省略と AST 構築の確保削減（`templates/finish.rs`、`templates/runtime.rs`）

- `recovery_occurrences` は recovery を持たない文法（`HAS_RECOVERY == false`、P4 も selfhost も該当）では空になるので
  走査しない。`occurrences`（capture / rule / token 出現）は AST か lexical のときだけ計算する（認識だけのときは
  3 つとも空になる走査だった）。complex で各 4〜5 µs。
- `has_recovery` / `directly_recovered` / `missing_field` は `HAS_RECOVERY` で短絡し、`has_value_group` /
  `selected_extent` / `has_recovery` の走査 stack を pool（`take_stack`）にする（node ごとの `vec![root]` 確保を廃止）。
- leaf 昇格（`node_values` の `fallback`）は `Ast::Text` の String を複製せず移動する。
- 3 回同じ経路を下っていた `text_extent` / `text_content_extent` / `has_token_text` を `semantic_text` で 1 回の走査に
  まとめる（規則は同じ）。`occurrences` の stack 要素を 48 → 32 byte にし、出現 Vec を event 数から事前確保する。
- pool 再利用時の trivia cache を `reset(0)` から前回の entry 数にする（0 だと parse ごとに縮小・再拡張していた）。
  `node_spans` は入力長から事前確保する。

## 手順 2〜3: display の失敗記録（`templates/diag.rs`）

観測できるのは「frame ごとの最遠位置と、その位置の label **集合**」だけで、label の重複と順序は観測できない
（`summary` は集合にして sort、snapshot は集合を再生、予算は記録回数）。これを使って:

- **記録時の重複除去**: frame の生存 label 集合（`log[base..]`）に世代印を付け、pointer 同一性の open addressing 表
  （256 slot、8 probe まで）で「既にある label は push しない」。range が空になる操作（`open` の位置更新、leave /
  discard の切り捨て）は必ず世代を付け替え、同じ位置の子 frame の leave は親の世代へ取り込む（`absorb`）。
  表が混んで判定できないときは重複を残すだけ（「無い label を有る」と誤る経路は無い: (label, 世代) は世代の range へ
  push したときにだけ置く）。64 個の閾値 dedupe と snapshot の dedupe は不要になり、128 個以上に伸びたときの
  詰め直しだけ残す。
- **記録単位を label 列にする**（手順 3）: emitter が置く `failures(pos, &["':'", "'['", "']'"])` の静的配列を 1 要素
  （`Entry::Many`、先頭 pointer が key）として記録し、個々の label に展開するのは `summary` だけ。1 回の failures が
  1 probe・1 push になる（3.8 label 分の hash を避ける）。`fail(pos, label)` は `Entry::One`。
  `failures` の引数は `&'static [&'static str]`（呼出し側は全て静的配列。tokens / predict / factor / expressions の
  emitter を確認）。
- 手前の位置（大半）の記録は inline の先頭検査で捨て、frame を開かない。
- **現在 frame の field 化**（手順 5）: 最も内側の frame を `Vec` の末尾ではなく field に持つ（`reach` が 5,528 回 /
  parse、`open` が 3,900 回。毎回 `last_mut().expect()` を引かない）。
- `crates/ubnfc-rust/tests/runtime.rs` の snapshot の「保存 label 数」の assertion は記録単位（1 記録）に合わせた
  （summary の集合は同じ）。

## 手順 4: 主診断 summary の hasher（`templates/diag.rs`）

`Diagnostics::summary` の `HashMap<u32, usize>`（node id → 深さ）を乗算 hash に、label の重複除去を `Vec::contains`
にした（label は少数）。計測差は雑音以下（確保 −2 回）。

## 不採用

- **event arena の圧縮**（`Event` 80 byte → 36 byte の `Packed`、読み出しで展開）: complex 認識 +3 µs、AST +8 µs
  （5 run の中央値 0.164 / 0.214 → 0.166 / 0.222）。読み出し側（finish の走査、AST 構築の毎 node の展開）が
  arena の書込み量の節約を上回った。x64 では最小値で 0.3 ms 良い程度で、雑音以下。
- **rule 入口検査 `limit` の inline 化**（`#[inline(never)]` → `#[inline]`）: 認識 +5 µs。759 rule 関数に展開されて
  i-cache を圧迫する（生成 parser は 20k 行）。
- AST 構築で Token 直下の capture の再帰を省く fast path: mapping 深さ・native stack 検査の発火位置が 1 frame ずれ得る
  （観測可能な差になり得る）ので見送り。
- 主診断の Rule node の省略（既知の最遠位置より手前なら wrap しない）: Not 成功時に子の診断を捨てる経路があるので
  「手前」を判定できない。見送り。

## 段階別 A/B（`examples/p4-rust` bench、release、warmup 後 5 run の中央値、ms/parse）

共用ホスト（load average 2.5〜6、Java M5 の計測が並走）。各段階は baseline binary（efd1114）と交互に 3〜5 回実行した
同セッションの代表値。「最小」は全 run の最小値（干渉に強い）。x64 は雑音が大きい（同 binary で 15.5〜30 ms）ので
参考値。allocs / bytes は GlobalAlloc の計数（AST）。

| 段階 | complex 認識 | complex AST | x64 認識 | x64 AST | allocs (AST) | bytes (AST) |
|---|---:|---:|---:|---:|---:|---:|
| baseline（efd1114） | 0.184 | 0.233 | 13.3 | 15.4〜15.9 | 858 | 163 KB |
| 1 finish の不要走査省略・確保削減（一部） | 0.184 | 0.234 | 12.0 | 15.6 | 824 | 160 KB |
| 2 display: 記録時の重複除去（label 単位） | 0.169 | 0.226 | 11.8 | 16.5 | 824 | 160 KB |
| 3 display: 記録単位を label 列に | 0.159 | 0.212 | 12.8 | 17.7 | 824 | 160 KB |
| 1' 事前確保・trivia reset・leaf 移動 | 0.160 | 0.212 | — | — | 773 | 123 KB |
| （不採用）event arena 36 byte 圧縮（1' の上） | 0.166 | 0.222 | — | — | 773 | 123 KB |
| （不採用）`limit` inline（1' + 圧縮の上） | 0.171 | 0.223 | 13.0 | 16.3 | 773 | 123 KB |
| 5 現在 frame の field 化・text 走査統合・出現 stack 縮小 | 0.155 | 0.207 | — | — | 772 | 122 KB |
| 4 summary の hasher | 0.155 | 0.209 | 13.0 | 17.1 | 770 | 122 KB |

### 最終 A/B（コア固定 `taskset -c 28`、baseline と交互に 9 run、中央値 / 括弧内は全 run の最小、ms/parse）

| 入力 | bytes | 認識 baseline | 認識 本 round | AST baseline | AST 本 round | allocs (AST) base → new | bytes (AST) base → new |
|---|---:|---:|---:|---:|---:|---:|---:|
| complex | 332 | 0.181 (0.176) | **0.157** (0.152) | 0.241 (0.227) | **0.212** (0.200) | 858 → 771 | 163 KB → 132 KB |
| complex-x4 | 1,293 | 0.706 (0.680) | **0.606** (0.576) | 0.877 (0.820) | **0.751** (0.720) | 3,152 → 2,907 | 592 KB → 442 KB |
| complex-x16 | 5,179 | 2.955 (2.719) | **2.391** (2.280) | 3.467 (3.206) | **2.993** (2.790) | 12,278 → 11,425 | 2.30 MB → 1.68 MB |
| complex-x64 | 20,923 | 14.67 (13.24) | **11.13** (10.26) | 17.46 (15.60) | **15.47** (13.63) | 48,712 → 45,449 | 9.15 MB → 6.65 MB |

認識の確保は complex で 367 → 353 回だが byte は 31.9 KB → 36.2 KB に増える（display の記録番号表 `Registry` の
直接写像 cache と lookup の分。log / saved が `&'static str` 16 byte から u32 4 byte になった分で x64 では逆に減る:
773 KB → 749 KB）。

## 最終値と目標

| 入力 | bytes | 認識 ms | AST ms | µs/byte (AST) | 目標 | 判定 |
|---|---:|---:|---:|---:|---:|---|
| complex | 332 | 0.157 | 0.212 | 0.64 | ≤ 0.2 ms | **未達**（中央値で +6%、12 µs。9 run の最小 0.200 は境界） |
| complex-x4 | 1,293 | 0.606 | 0.751 | 0.58 | — | — |
| complex-x16 | 5,179 | 2.391 | 2.993 | 0.58 | — | — |
| complex-x64 | 20,923 | 11.13 | 15.47 | 0.74 | ≤ 15 ms | **未達**（中央値で +3%、0.5 ms。最小 13.6 は下回る） |

round 3 末（0.180 / 0.228、14.7 / 15.7）との比較は計測条件が違うので、同じ binary を同じセッションで計った
上の最終 A/B の列（baseline = efd1114）で読むこと。手書き目安 0.4 µs/byte に対して complex は 1.6 倍、
x64 は 1.85 倍（AST 付き）。

complex AST の内訳（手順 3 時点の phase 計測、µs）: Session 1.3、parse 170（認識だけだと 160。AST の確保と drop が
cache を汚す分）、主診断 summary 3.3、出現収集 8.8、AST 構築 20.2、value span 6.8、結果組立 1.0。認識との差
約 50 µs のうち API が確保を要求する分（`Box<Ast>`、`Ast::Text(String)`、`ValueSpan.path` / `text` の String、
`Capture` 72 byte、`scope.events` の name 複製）が大半で、round 3 の結論と同じ。

x64 AST の内訳（同、ms）: Session 0.08〜0.12、parse 12.7、出現収集 1.4、AST 構築 1.6、value span 0.45、結果組立 0.07。
認識 12.2 に対して finish が 3.6 ms。

## 契約の検証

- `docs/design/04-cursor-and-diagnostics-contract.md` §1（cursor）: 生成 cursor コードは無変更。§2（診断）: 主診断 DAG
  （offset / expected / rulePath）は無変更。公開 farthest（D-014）は「frame ごとの最遠位置と label 集合」の観測が同じ
  （記録時重複除去は集合を変えない。予算は記録回数のまま）。
- `03-design-v1.md` §3.4 / §3.7: 候補除外・左因数分解・memo は無変更。
- 差分テスト `predict_matches_reference_path_on_truncated_and_corrupted_inputs`（16 fixture × 切り詰め・破壊 × 経路 ×
  memo on/off、約 11,000 件）緑。`crates/ubnfc-rust/tests/runtime.rs` の display frame テスト緑。
- corpus: `docs/reports/2026-09-23-rust-corpus-m5r4.{md,json}`。系統別の表が round 3 と完全に同じ
  （Pass 1292 / 期待拒否 Pass 30 / Known 28 / Fail 0 / 未観測 75）。
- selfhost 固定点（`crates/ubnfc-selfhost/tests`）緑、`src/generated` を再生成して commit（`rt/diag.rs`、`runtime.rs`）。
  P4 golden AST / driver JSON は byte 一致（`examples/p4-rust/tests/fixtures.rs`）、handwritten-target の golden も
  byte 一致（`crates/ubnfc-rust/tests/generated.rs`）。
- `ubnfc-java`、corpus / oracle、unlaxer-parser、tinyexpression は触っていない。

## 残り

1. complex AST の 0.2 ms: 残り約 12 µs（6%。最終 A/B の最小値では 0.200 で境界）。AST 経路の確保は API 契約（round 3 の残り 1 と同じ）。
   認識側は display の `open` + 予算 + 1 probe が 3,900 回、主診断 DAG 5,947 push、memo の save / lookup が残る。
   `ValueSpan` の遅延実体化（span だけ持ち、String は値の取得時に作る）は bench の AST mode では「やらない仕事」に
   なるだけで、driver が JSON を書く経路では同じ確保が起きる（総量は減らない）。計測が良くなっても実際の利用者の
   仕事は減らないので、この round では採らなかった。`Ast::Text` の `Rc<str>` 化・`ValueSpan` の `&str` 化は
   公開 API の変更なので別 issue。
2. x64 AST: finish の出現収集（1.4 ms）と AST 構築（1.6 ms）が 2 回の全木走査。`occurrences` を AST 構築の走査に
   統合できれば 1 ms 級だが、capture の完了順（全木の post-order）と rule 完了順の付番を構築順から復元する必要があり、
   別 round。event arena の圧縮は x64 でも雑音以下だった。
3. 静かなホストでの再計測は最終 A/B（コア固定・交互 9 run）で行い、x64 でも baseline との差が雑音を超えることを
   確認した（認識 −24%、AST −11%）。それでも 1 コアに固定しただけで他プロセス（Java M5 の計測）の影響は残るので、
   専有ホストでの再現は未実施。
4. round 3 の残り 3（`analysis.mayStart` への `firstChars` 反映、Java と調整）と 4（memo entry の縮小）は未着手。

## 再現

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cargo run -q --offline --release -p ubnfc-p4-rust --example bench -- complex complex-x4 complex-x16 complex-x64
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust --report docs/reports/2026-09-23-rust-corpus-m5r4.md --json docs/reports/2026-09-23-rust-corpus-m5r4.json
```
