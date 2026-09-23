# P4 Rust 生成 parser の性能改善 M5 round 5（2026-09-23）— finish の走査統合

> **訂正（本報告の commit 直後に確定）**: 本文の「旧実装比」列が使っている
> **complex 12.32 ms / complex-x64 958 ms は旧実装の公開 facade（`publicFacade`）の値**で、
> AST 経路の対比には使えない。確定したベースラインは
> [`2026-09-23-legacy-baseline.md`](2026-09-23-legacy-baseline.md) の
> **complex 認識 13.6 ms / AST 14.5 ms、complex-x64 認識 1,603 ms / AST 1,693 ms** であり、
> 対比は認識 ↔ 認識、AST ↔ AST で採る。本報告の ubnfc 側の絶対値と A/B は有効なまま。
> なお ubnfc Rust のその後の到達点は [`2026-09-23-p4-rust-floor.md`](2026-09-23-p4-rust-floor.md)
> （complex 認識 0.066 / AST 0.089 ms、x64 認識 4.52 / AST 5.86 ms）。

[`2026-09-23-p4-rust-perf-m5-round4.md`](2026-09-23-p4-rust-perf-m5-round4.md) の「残り 2」:

> x64 AST: finish の出現収集（1.4 ms）と AST 構築（1.6 ms）が 2 回の全木走査。`occurrences` を
> AST 構築の走査に統合できれば 1 ms 級だが、capture の完了順（全木の post-order）と rule 完了順の
> 付番を構築順から復元する必要があり、別 round。

本 round はこれを実装した。

## 本 round の途中で baseline が動いた（重要）

round 4 末の fb0ed05 を baseline に始めたが、作業中に master へ
[`2026-09-23-p4-rust-diag-two-mode.md`](2026-09-23-p4-rust-diag-two-mode.md)（二段構えの解析、
成功経路で診断を記録しない）が入り、**complex 認識 −43〜−50% / AST −29〜−43%** という大きな変化があった。
そこで本 round の A/B は **master 1788228（二段構え込み）を baseline** に取り直した（`git merge master` 済み）。
fb0ed05 との比較は「round 4 → 現在」の累積として最後に併記する。

さらに計測の後、master に Java M5 の merge（dede59f）が入り、Rust 側にも字句 cache
（`templates/lexcache.rs`）が加わった。これも merge して**生成物を再生成し、
test / clippy / fmt / corpus を取り直した**（corpus は master 側の
`2026-09-23-rust-corpus-lexcache.md` と一致）。dede59f を baseline にした確認計測では
complex AST 0.140 → 0.136、complex-x4 0.546 → 0.518、complex-x16 2.032 → 2.010 で下の表と同じ向きだが、
この窓は load average が 6 → 10 へ上がってしまい complex-x64 が使える値にならなかった
（AST 12.43 → 12.59、対照の認識も 6.82 → 6.82 で雑音に埋もれている）。
**確保の値は dede59f 上でも下の表と完全に同じ**なので、下の A/B（1788228 基準、load 4〜5 の窓）を本 round の結果とする。

その後さらに master へ `perf/rust-profile`（9e97776、guard 再生の省略）が入ったので、これも merge して
生成物を再生成し、test / clippy / fmt / corpus を取り直した（corpus は master 側の
`2026-09-23-rust-corpus-profile.md` と一致）。この時点の branch の値は
complex 認識 0.098 / AST **0.130** ms（確保 320 / 31,769、676 / 112,532）、
complex-x64 認識 6.471 / AST **11.587** ms（確保 18,706 / 729,715、40,629 / 5,690,292）。
本 round の寄与だけを切り出した A/B は下の表（1788228 基準）で読むこと。

## 結論

- **finish（AST 付きの後処理）が 8〜11% 速くなった**。同一 run 内の `AST − 認識` で見ると
  complex 0.047 → 0.042 ms、complex-x16 0.470 → 0.425 ms、complex-x64 **3.75 → 3.44 ms**
  （complex-x4 だけ 0.141 → 0.142 ms で差が出ない）。
- AST 付きの全体は complex **0.139 → 0.133 ms（−4.3%）**、complex-x16 1.914 → 1.857（−3.0%）、
  complex-x64 **9.93 → 9.54 ms（−3.9%）**。対照（本 round が触っていない認識 mode）は
  −0.8〜−1.3% なので、正味は −2.2〜−3.2%。
- 確保は負荷に依らない値で確実に減った。complex AST **748 → 686 回 / 128,054 → 112,772 byte（−11.9%）**、
  complex-x64 AST **45,426 → 41,269 回 / 6,643,713 → 5,705,652 byte（−14.1%）**。
  認識だけの parse の確保は master と完全に同じ（330 / 32,009、19,346 / 745,075）。
- **目標は両方とも達成済み**（master の二段構えで既に達成しており、本 round はその上に積んだ）:
  complex AST 0.133 ≤ 0.2 ms、complex-x64 AST 9.54 ≤ 15 ms。
  上積み目標 complex ≤ 0.12 / x64 ≤ 9 は **x64 が中央値 9.54（最小 7.85）で未達、complex が 0.133（最小 0.127）で未達**。
  旧実装比（12.32 ms / 958 ms）は complex **92.6 倍**、complex-x64 **100.4 倍**。
- 計測は共有ホスト（別 agent の Java M5 計測が並走、load average が 5〜80 の間で動く）で、
  load average 4〜5 の窓を待って取った。1 起動 1 入力で 3 binary を細かく交互に 24 回ずつ回している。

## 何を統合したか

finish は AST を作るとき、同じ event 木を 2 回下っていた。

1. `occurrences`: 全木の post-order 走査。公開する capture / rule / token 出現を作る。
2. AST 構築: `build_values` から rule ごとに `collect_captures(rule の child)` を呼び、
   その rule の**直下 capture**（間に別の rule を挟まないもの）を完了順に集め直す。

2 は 1 が既に訪れた event をもう一度読むだけだった（complex-x64 実測: 1 の走査 73,335 event に対し、
2 は 7,491 回の呼出しで延べ 58,894 event。1 回あたり 7.9 event を辿って平均 1.3 個の capture を取り出していた）。

### 直下 capture 表の作り方（順序をどう保つか）

`occurrences` の 1 回の走査で「rule ごとの直下 capture」を切り出す。鍵は **深さ優先だから区間になる**こと。

- capture は完了順（post-order）に作業列 `pending` へ積む。
- rule R に入った時点の `pending` の長さを `start` として rule 用の stack に積む。
- R の完了時、`pending[start..]` は **ちょうど R の直下 capture を完了順に並べたもの**になる。
  R の中に別の rule S があっても、S は自分の完了時に自分の分（`pending[s_start..]`）を抜き取って
  `pending` を `s_start` まで戻すので、R の分だけが連続して残る。
  （S に入ったのは R の capture が何個か積まれた後かもしれないが、S が抜けた後に積まれる R の capture は
  同じ `s_start` 位置から続く。したがって R の分は常に `start` から連続する。）
- これを `caps_flat` へ移し、`(開始, 個数)` を **`Event::Rule::caps`** に書く。
  AST 構築の `map_rule_i` は `collect_captures` の代わりに `rule_captures(caps, &mut …)` で
  その区間を写すだけになる。

順序が従来と一致することの根拠:

- `collect_captures` は Join → 左右、Values → child、Capture → child を先に処理してから自身を出力し、
  **Rule には降りない**。`occurrences` の走査は同じ push 順序で、Rule にだけ追加で降りる。
  よって「最も近い囲み rule が R である capture」の集合と順序は両者で同一。
- `token_extent` の capture に対する `selected_extent` も同じ位置（capture 完了時）で計算する。
  違いは、AST 構築が降りない部分木（skip / text recipe の下）でも計算されるようになったこと。
  観測は変わらない（P4 / self-host には `token_extent` の capture が無い）。
- 同じ event が木に 2 回現れる（memo / 左因数分解の共有）場合は 2 回書かれるが、内容は同じなので
  どちらが残っても結果は変わらない。
- `Event::Rule::caps` は event を作るとき `(0,0)` で、AST を作る parse では `occurrences` が
  `build_values` より前に木の全 Rule event を訪れて書く（`ok ⇒ step.ok`、`build_ast ⇒ occurrences` が走る）。
  arena は parse ごとに作り直すので前の parse の値が残ることも無い。

### 付番も走査中に確定させる

`Capture::rule_completion_order` は「所属 rule 出現の配列 `capture_owners`」と「rule 出現ごとの完了順
`rule_orders`」を作り、走査後に全 capture を舐めて書き直していた。直下 capture 表と同じ区間が使えるので、
**rule の完了時にその rule の直下 capture へ直接書く**ようにして、2 本の配列と後段の走査を廃止した。
値は従来と同じ（rule の完了順は増加前の値、囲み rule の無い capture は 0）。

## そのほかの手当て（同じ finish 経路）

- 直下 capture の区間 `(開始, 個数)` は **`Event::Rule` の field** に置く。最初は event id で索く
  別表（`Vec<(u32,u32)>`）にしたが、complex-x64 では 640 KB の疎な表に 10,824 回書いて 7,491 回読むことになり、
  省いた走査と同じだけ cache miss を作っていた（交互 A/B で差が出なかった）。Rule event は
  AST 構築が `rule` / `span` / `child` を読むときに必ず載るので、同じ cache line で受け取れば表は要らない。
  `Event` の最大 variant は `Token`（`Option<Span>` 2 つで 80 byte）なので、`Rule` に 8 byte 足しても
  `size_of::<Event>()` は 80 byte のまま（確認済み）。
- 出現収集の作業 stack を 32 → 16 byte（event id も出現番号も入力 byte 数 = u32 上限で抑えられる）。
- 事前確保の見積りを event 数 /4 → /6。P4 実測は capture も rule も event の 1/8 前後
  （complex 155 + 177 / 1,308、complex-x64 9,731 + 10,824 / 80,246）。/4 は 2 倍以上の過大確保で、
  x64 では `Capture` 72 byte × 20,061 を確保して半分しか使っていなかった。
- `remember_text` の識別用 1 byte を**複製時に**確保する。`to_owned()` は丁度 len で確保するので
  直後の `reserve(1)` が必ず再確保になっていた（text 1 個につき確保 1 回と memcpy 1 回の無駄）。
  この複製は `text_for_identity` に分けた。`text()` は scope 効果（宣言・参照の名前）でも使われ、
  そちらに 1 byte 足すと認識だけの parse の確保 byte が増えてしまう。
- `value_spans` の添字を共有 path buffer へ直接書く（要素ごとの `to_string` の確保を廃止）。
- `build_values_inner` に Join を含まない枝の速路を入れ、pool から作業 stack を借りない
  （complex-x64 実測で 10,883 回の呼出しが全て 1 event だった）。

## A/B

`taskset -c 4` で 1 コアに固定。`UBNFC_BENCH_RUN_SECONDS=0.25` の短い sample にして、
**1 起動 1 入力**で 3 binary（r4 = fb0ed05 / base = master 1788228 / new = 本 round）を
**交互に 24 回ずつ**回した（起動単位で交互にすると、1 起動が丸ごと混雑の窓に入って歪むため）。
中央値は 24 起動の「起動内 5 sample の中央値」の中央値、最小は全 120 sample の最小。
開始時 load average 5.05、終了時 3.85。

| 入力 | bytes | mode | r4 中央 (最小) | master 中央 (最小) | 本 round 中央 (最小) | 本 round / master |
|---|---:|---|---:|---:|---:|---:|
| complex | 332 | 認識 | 0.157 (0.150) | 0.092 (0.088) | 0.091 (0.087) | −1.1%（対照） |
| complex | 332 | AST | 0.209 (0.200) | 0.139 (0.131) | **0.133** (0.127) | **−4.3%** |
| complex-x4 | 1,293 | 認識 | 0.629 (0.586) | 0.375 (0.348) | 0.370 (0.350) | −1.3%（対照） |
| complex-x4 | 1,293 | AST | 0.792 (0.727) | 0.516 (0.484) | **0.512** (0.469) | −0.8% |
| complex-x16 | 5,179 | 認識 | 2.434 (2.300) | 1.444 (1.385) | 1.432 (1.384) | −0.8%（対照） |
| complex-x16 | 5,179 | AST | 3.002 (2.774) | 1.914 (1.844) | **1.857** (1.788) | **−3.0%** |
| complex-x64 | 20,923 | 認識 | 11.101 (10.120) | 6.179 (5.687) | 6.097 (5.673) | −1.3%（対照） |
| complex-x64 | 20,923 | AST | 14.862 (12.891) | 9.928 (7.840) | **9.537** (7.854) | **−3.9%** |

**認識 mode は本 round が一切触らない経路なので対照になる**（出現収集も AST 構築も走らない）。
そこに残る −0.8〜−1.3% が、この計測環境の系統誤差・雑音の大きさ。AST の改善はそれを引いた分で読む。

finish の費用（同じ起動の中の `AST − 認識`。中央値、ms）:

| 入力 | r4 | master | 本 round | 本 round / master |
|---|---:|---:|---:|---:|
| complex | 0.052 | 0.047 | **0.042** | **−11%** |
| complex-x4 | 0.163 | 0.141 | 0.142 | +1% |
| complex-x16 | 0.568 | 0.470 | **0.425** | **−10%** |
| complex-x64 | 3.761 | 3.749 | **3.440** | **−8.2%** |

確保（`GlobalAlloc` の alloc / realloc 計数。負荷に依らないので雑音が無い）:

| 入力 | mode | r4 回 / byte | master 回 / byte | 本 round 回 / byte |
|---|---|---:|---:|---:|
| complex | 認識 | 353 / 36,225 | 330 / 32,009 | 330 / 32,009 |
| complex | AST | 771 / 132,270 | 748 / 128,054 | **686 / 112,772** |
| complex-x64 | 認識 | 19,369 / 749,291 | 19,346 / 745,075 | 19,346 / 745,075 |
| complex-x64 | AST | 45,449 / 6,647,929 | 45,426 / 6,643,713 | **41,269 / 5,705,652** |

減った確保の内訳: `remember_text` の再確保（complex 62 回、x64 4,157 回。text 1 個に 1 回）、
出現収集の過大確保（x64 で `Capture` 72 byte × 6,679 個ぶんと所属表 1 本ぶん、計 約 930 KB）、
`value_spans` の添字文字列。

## 目標との対応

| 入力 | bytes | 認識 ms | AST ms | µs/byte (AST) | 目標 | 判定 | 旧実装比 |
|---|---:|---:|---:|---:|---:|---|---:|
| complex | 332 | 0.091 | **0.133** | 0.40 | ≤ 0.2 ms | **達成** | 92.6 倍（12.32 ms → 0.133） |
| complex-x4 | 1,293 | 0.370 | 0.512 | 0.40 | — | — | — |
| complex-x16 | 5,179 | 1.432 | 1.857 | 0.36 | — | — | — |
| complex-x64 | 20,923 | 6.097 | **9.537** | 0.46 | ≤ 15 ms | **達成** | 100.4 倍（958 ms → 9.537） |

上積み目標（complex ≤ 0.12 ms / x64 ≤ 9 ms、旧実装の約 100 倍）:

- complex は **0.133（最小 0.127）で未達**。あと 13 µs。内訳は下の「残り」の 1。
- complex-x64 は **9.537（最小 7.854）で未達**だが、旧実装比では 100.4 倍に届いている
  （9 ms は 106 倍相当）。混雑の無い窓の最小値 7.85 は 9 ms を下回る。
- 手書き目安 0.4 µs/byte に対して complex は 1.00 倍、x64 は 1.15 倍（AST 付き）。
  round 4 時点は 1.6 倍 / 1.85 倍だった。

## 採らなかったもの

- **round 4 が理由付きで却下した項目は再検討していない**: `ValueSpan` の遅延実体化、memo entry の縮小、
  event arena の圧縮、rule 入口検査の inline 化。
- `Event` の span を `[u32;2]` にして arena を半分にする案は、round 4 の「event arena 圧縮（計測で悪化）」と
  同じ土俵なので本 round では触っていない。ただし**却下された版は「36 byte に詰めて読み出しで展開する」もの**で、
  展開の無い u32 化は別物として残っている（下の「残り」）。
- 出現収集の走査本体から `&mut self` 呼出し（`selected_extent`）を追い出して `arena` の
  register 保持を助ける案は、効果を切り分けられる計測環境が取れなかったので入れていない。

## 契約の検証

- 公開する並びと番号付けは不変: capture 完了順（内側→外側）、出現順、`valueSpans`、`texts` / `nodes`、
  字句 / token 出現、scope event 順。`docs/design/03-design-v1.md` §3.6 / §6 と
  `docs/harness-driver-protocol.md` の契約に変更なし。
- `cargo test --workspace --offline` 全緑（差分テスト
  `predict_matches_reference_path_on_truncated_and_corrupted_inputs` を含む）、
  `cargo clippy --workspace --all-targets --offline -- -D warnings` 無警告、`cargo fmt --all -- --check` 緑。
- selfhost 固定点緑（`crates/ubnfc-selfhost/src/generated` を再生成して commit）。
  P4 `examples/p4-rust/src/generated` も再生成して commit。golden AST / driver JSON は byte 一致
  （`examples/p4-rust/tests/fixtures.rs`）、handwritten-target の golden も byte 一致
  （`crates/ubnfc-rust/tests/generated.rs`）。
- corpus: `docs/reports/2026-09-23-rust-corpus-m5r5.{md,json}`。系統別の表は
  `2026-09-23-rust-corpus-diag2.md`（= `m5r4`）と完全一致
  （Pass 1292 / 期待拒否 30 / Known 28 / Fail 0 / 未観測 75）。
- `ubnfc-java`、corpus / oracle、`crates/ubnfc-front`、unlaxer-parser、tinyexpression は触っていない。

## 残り

1. **complex の 0.12 ms**: あと 13 µs。finish は既に 42 µs しかなく、そのうち
   `value_spans` が約 6.6 µs（`ValueSpan` ごとに `path` の String 複製 = 確保 1 回、
   complex で約 200 回）、AST node の `Box` / `Vec` が約 9 µs。どちらも公開 API の形
   （`ValueSpan.path: String`、`Ast::Text(String)`、`Box<Ast>`）が要求する確保なので、
   API を変えない限り下がらない。残りは認識側（0.091）で、そちらは別の話。
2. **complex-x64 の 9 ms**: finish が 3.44 ms 残る。内訳（round 4 の計測器を当てた概算）は
   出現収集の全木走査 約 1.1 ms、AST 構築 約 1.1 ms、`value_spans` 約 0.5 ms、その他。
   走査は event arena（80,246 event × 80 byte = 約 6.4 MB）を辿る**帯域律速**に見える
   （73,335 event 訪問で 1.1 ms ≒ 15 ns/event）。`Event` の `Span` を `[u32;2]`、
   `Option<Span>` を番兵にすれば arena は半分になるが、round 4 が「event arena 圧縮」を
   計測で却下しているので本 round では触っていない。**却下されたのは「36 byte に詰めて
   読み出しで展開する」版で、展開の無い u32 化は別物**なので、次の round の候補として残す。
3. **`collect_captures` と同じ手を AST 構築の局所走査にも当てる**案（rule の直下にある
   Rule / Values event の並びを出現収集で切り出しておく）は、`build_values_inner` の
   呼出しが complex-x64 実測で 10,883 回すべて 1 event だったので、走査を省く余地が無かった。
   速路（pool から stack を借りない）だけ入れてある。
4. 計測環境。共有ホストで別 agent の Java 計測が並走し、load average が 5〜80 の間で動く。
   x64 は混雑時に中央値が 2 倍以上ぶれるので、load average 4〜5 の窓を待って取り直した。
   専有ホストでの再現は未実施。

## 再現

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cargo run -q --offline --release -p ubnfc-p4-rust --example bench -- complex complex-x4 complex-x16 complex-x64
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust \
  --report docs/reports/2026-09-23-rust-corpus-m5r5.md --json docs/reports/2026-09-23-rust-corpus-m5r5.json
```
