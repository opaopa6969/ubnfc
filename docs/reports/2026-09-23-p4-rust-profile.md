# P4 Rust 生成 parser の現時点プロファイル（2026-09-23）

master `1d2370d`（D-023 二段構え診断 + 字句 cache 既定 off）の生成物で、**時間の行き先を測り直した**報告。
本 branch は仕上げに master（`dede59f`、Java M5）を取り込んでいるが、**Rust の生成物は byte 単位で変わらない**
（master が足した plan 情報は Java 側の候補除外だけが読む）ので、以下の計測はそのまま merge 後の木にも当てはまる。
[`2026-09-23-p4-rust-lexical-cache.md`](2026-09-23-p4-rust-lexical-cache.md) §前提の訂正のとおり、
直前の 2 つの改善案は**古い内訳**から立てられ、片方（位置ごとの字句 cache）は実装後に計測で棄却された。
本 round は改善を出す前に内訳を取り直すことだけを目的にする。結論を先に書く:

- **時間は「1 つの重い機構」ではなく、`1 byte あたり 111 回の runtime 操作`に薄く広がっている**（§1）。
  1 操作あたり 2.4〜2.6 ns で、complex と complex-x64 でこの単価は 7% しか違わない。
  つまり**操作回数だけで両入力の時間の 9 割が説明できる**。切れる「大きな塊」は無い。
- 認識 89 µs（complex、332 byte）の内訳で方法 B が切り出せたのは
  **event arena 12 µs（13.5%、ただし上限）／主診断 `Diag` の配管 13 µs（14.6%）／候補除外の失敗再生 7 µs（7.7%）／display の呼出し 5 µs（5.6%）**
  で、残り 52 µs（58%）は「切ると受理が変わる」機構の束であり、**この host に `perf` が無いので割れていない**（§2.3）。
- AST の増分 41 µs は **AST 構築 22 µs（54%）／value span 11 µs（27%）／出現収集 7 µs（17%）** に
  ほぼ全部（98%）帰属できた（§3.2）。AST は node 1 個あたり **3.2 回・733 byte** 確保している。
- **測って「無かった」場所**: rule 入口の native stack 検査 0 µs、`node_spans` の記録 0 µs、
  mapping の型適合検査（`matches!` 連鎖）0 µs、認識での `Event::Token` 省略 0 µs。
  最後の 1 つは「筋は通るのに実測が雑音以下」という**字句 cache と同じ形の失敗**である（§3.3）。
- **memo と候補除外は縮める対象ではない**。off にすると認識がそれぞれ **6.8 倍 / 2.0 倍**遅くなる（§2.2）。
- **長期目標 0.02 ms（complex AST）には届かない**。作業を 4.4 倍減らす必要があるのに、
  候補除外は既に 209 選択肢中 105 個を除外していて残りは「除外できないと証明された」ものを含む。
  **この設計での現実的な床は complex AST 0.085〜0.100 ms、x64 AST 5.3〜6.2 ms**（§5）。
- 本 round で入れた変更は 1 つだけ: **fast mode で候補除外の失敗再生 `guard_e*` を丸ごと省く**
  （生成器 8 行）。採用後の A/B で complex 認識 **−6.6%**、complex AST **−5.9%**、
  x64 認識 **−7.0%**、x64 AST **−4.9%**（§6）。

## 0. 計測条件

| 項目 | 値 |
|---|---|
| rustc / cargo | `rustc 1.98.1 (48a229cea 2026-09-01)` / `cargo 1.98.1`、`--release --offline`（workspace の release profile そのまま） |
| host | 32 core、WSL2（`Linux 6.18.33.2-microsoft-standard-WSL2`）。共有機で他エージェントが並走する |
| 固定 | `taskset -c 27`。全 variant を**同一セッションで交互に 9 round**、1 round は各 variant 1 回（bench は内部で warmup 後 5 sample の中央値を出す） |
| 集計 | round 中央値と**全 run の最小値**。共有機なので採否は最小値で判断する |
| load average | round 開始時 4.4〜8.1（session 開始前に 57 → 5 まで落ちた静かな窓で実施） |
| `perf` | **この host に無い**（`perf` 未インストール、`perf_event_paranoid=2`、sudo 無し）。valgrind / callgrind も無い。したがって時間配分は §2 の方法を使う |

### 方法 A: `counters` feature（回数）

`examples/p4-rust` の `counters` feature（`build.rs` が生成物の複製へフックを挿す）で
runtime primitive・rule 関数・expression 関数の**呼出し回数**を数えた。計時値は歪むので回数だけ使う。

### 方法 B: 機能の切り離し（時間）

生成物 `examples/p4-rust/src/generated/{parser,runtime,rt/diag}.rs` を**一時的に**書き換えて 1 機構ずつ
無効化した binary を作り、同一セッションで baseline と交互に測って差を取る（round 4 で使った方法）。
生成物は計測後 `git checkout` で戻しており、commit には含まれない。

**この方法の限界**を先に書く:
- 機構を外すと LLVM の最適化（Step の field の死後除去・inline 判断・code layout）も変わるので、
  差は「その機構が消費する時間」ではなく「**その機構を消したときに得られる時間**」である。
  改善案の期待値としてはこちらのほうが正しいが、内訳の和が全体に一致する保証は無い。
- 受理結果を変える無効化（scope 効果を壊す等）は**上限**としてしか読めない。該当箇所は明示する。
- 対照（recognition では何も変わらないはずの variant）を 3 つ混ぜて、位置効果と雑音の大きさを見た。


## 1. 回数（方法 A）: この parser は入力 1 byte あたり 111 回の runtime 操作を行う

`counters` feature、1 parse あたり。`complex`（332 byte）と `complex-x64`（20,923 byte、63.0 倍）。

| 操作 | complex | /byte | complex-x64 | /byte | x64/complex |
|---|---:|---:|---:|---:|---:|
| rule 関数呼出し | 759 | 2.29 | 46,119 | 2.20 | 60.8 |
| expression 関数呼出し | 2,399 | 7.23 | 146,228 | 6.99 | 61.0 |
| 候補除外の失敗再生 `guard_e*` | 819 | 2.47 | 49,707 | 2.38 | 60.7 |
| `enter_rule` / `limit` | 759 / 759 | 2.29 | 46,119 / 46,119 | 2.20 | 60.8 |
| `mark` / `restore` | 1,985 / 2,760 | 5.98 / 8.31 | 120,551 / 167,505 | 5.76 / 8.01 | 60.7 |
| memo `lookup` / `store` / hit | 759 / 590 / 169 | 2.29 / 1.78 | 46,119 / 35,555 / 10,564 | 2.20 / 1.70 | 60.8 |
| `event`（arena push） | 1,308 | 3.94 | 80,246 | 3.84 | 61.4 |
| `join` / `combine` | 1,408 / 1,712 | 4.24 / 5.16 | 85,639 / 103,835 | 4.09 / 4.96 | 60.8 |
| 終端照合 `primitive`（うち `literal`） | 595 (548) | 1.79 | 36,568 (33,812) | 1.75 | 61.5 |
| `number` / `identifier` / Extern `scan` | 25 / 22 / 43 | — | 1,474 / 1,282 / 2,563 | — | 59.6 |
| `fail` | 461 | 1.39 | 28,307 | 1.35 | 61.4 |
| display `reach` / `failures` / `enter` / `leave` | 5,528 / 5,400 / 590 / 590 | 36.5 合計 | 335,648 / 326,826 / 35,555 / 35,555 | 35.1 合計 | 60.6 |
| 主診断 `join` / `rule` / `push` | 4,147 / 2,334 / **0** | 19.5 合計 | 249,721 / 139,233 / **0** | 18.6 合計 | 60.0 |
| 効果 `save` / `replay` / `effect_at` | 229 / 169 / 74 | 1.42 合計 | 14,089 / 10,564 / 4,358 | 1.39 合計 | 61.3 |
| **合計** | **36,772** | **110.8** | **2,229,297** | **106.6** | **60.6** |

読み取り:

1. **完全に線形**。x64 は入力が 63.0 倍なのに操作は 60.6 倍で、byte あたりでは **4% 少ない**。
   memo・候補除外・左因数分解が入力長に対する超線形を既に消している。x64 が遅いのは操作が増えるからではない。
2. **1 byte あたり 111 操作**、rule 評価は 2.29 回/byte。最終 AST は 131 node（0.39 node/byte）なので、
   **生き残る node 1 つあたり rule 評価 5.8 回**が投機で捨てられている。これが構造的な作業量である。
3. **主診断 DAG の `push` は 0**（D-023 の fast mode）。それでも `join` 4,147・`rule` 2,334 の**呼出し**は残っている。
4. display の呼出しは 12,108 回/parse（全操作の 33%）で、fast mode では全て先頭の `enabled` 検査で戻る。

### 方法 A と方法 B の突合

complex 認識の最小値 89 µs / 36,772 操作 = **2.42 ns/操作**、x64 は 5,755 µs / 2,229,297 操作 = **2.58 ns/操作**。
7% しか違わない。つまり **操作回数だけで両入力の時間の 93% が説明できる**（残りは局所性: x64 は
byte あたり操作が 4% 少ないのに byte あたり時間は 5% 多い）。時間配分（§2）はこの単価と矛盾しないことを確認しながら読む。

## 2. 時間（方法 B）: 機構別の切り離し

`taskset -c 27`、9 round 交互、ms/parse。中央値（括弧内は全 run の最小）。**Δ は最小値どうしの差**。

### 2.1 対照（雑音の大きさ）

recognition では何も変えないはずの variant を 5 つ混ぜた（`noocc` / `novspan` / `notypecheck` / `nolimit` /
`noscopediag` は AST 経路だけを触る。`noscopediag` は認識でも `references().to_vec()` を落とすので厳密には対照でない）。

| 入力 / mode | 対照の Δ の幅 | 判定に使う雑音帯 |
|---|---|---|
| complex 認識 | 0.0 〜 −1.1% | **±1.5%**（±1.5 µs） |
| complex AST | 0.0 〜 +0.8% | **±1.5%**（±2 µs） |
| complex-x64 認識 | −1.7 〜 −3.4% | **±3.5%**（±200 µs） |
| complex-x64 AST | −5.2 〜 +2.7% | **±5.5%**（±450 µs） |

x64 は入力が大きく 1 run の反復数が少ないので雑音帯が広い。x64 の判定は complex と符号が一致するものだけ信用する。

### 2.2 認識経路の内訳

baseline（`complex` 認識 中央値 0.094 / 最小 **0.089** ms、`complex-x64` 認識 中央値 6.282 / 最小 **5.755** ms）。

| 切り離した機構 | complex Δ | % | x64 Δ | % | 方法 | 確度 |
|---|---:|---:|---:|---:|---|---|
| **event arena（`event()` を認識で無効化）** | **−12 µs** | **13.5%** | **−904 µs** | **15.7%** | `event()` の `HAS_SCOPE` 条件を外す | **上限**。P4 は `HAS_SCOPE=true` で scope 効果が Capture event を読むので、この binary の scope 結果は壊れている（受理は同じ） |
| **主診断 `Diag` の配管（`join`/`rule`/`push` を定数化）** | **−13 µs** | **14.6%** | **−655 µs** | **11.4%** | `rt/diag.rs` の `Diagnostics` を常に `Diag::NONE` にする | 高。fast mode で観測は既に空。`Step.diag` の死後除去も含む |
| display（`DisplayDiagnostics` を完全に消す） | −5 µs | 5.6% | −114 µs | 2.0%（雑音帯内） | `if !self.enabled` を `if true` に | 高（complex）。x64 は雑音帯内 |
| rule 入口の native stack 検査（`limit` の addr_of/abs_diff） | **0 µs** | **0.0%** | −100 µs | 1.7%（雑音帯内） | `limit` の深さ検査を `None` に | 高。**ここには時間が無い** |
| `mark` / `restore` | （計測不能） | — | — | — | `restore` を no-op に | **無効**。`effects` が切り詰められず作業が増えて 35〜54% 遅くなった。この方法では測れない |

memo と候補除外は「外せる機構」ではなく**作業量を決める機構**なので、切るのではなく off にして純益を測った。

| 機構 | complex 認識 | x64 認識 | 読み取り |
|---|---:|---:|---|
| memo off（`UBNFC_BENCH_OPTS=memo=0`） | +520 µs（**6.8 倍**） | +57.1 ms（**11.9 倍**） | memo は律速そのもの。縮める対象ではない |
| memo の `lookup` を強制 miss に | +572 µs（7.4 倍） | +63.4 ms（12.0 倍） | 同上（表の確保は残したまま） |
| 候補除外 off（`predict=0`） | +86 µs（**2.0 倍**） | +6.4 ms（**2.1 倍**） | 候補除外は現状で認識時間を半分にしている |

### 2.3 認識 89 µs の帰属（complex、332 byte）

| bucket | µs | % | 根拠 |
|---|---:|---:|---|
| event arena（`Event` 80 byte × 1,308 push） | 12 | 13.5 | 方法 B（上限） |
| 主診断 `Diag` の配管 | 13 | 14.6 | 方法 B |
| display の呼出し（12,108 回、全て早期 return） | 5 | 5.6 | 方法 B |
| **候補除外の失敗再生 `guard_e*` の本体**（819 回、fast mode では全て観測なし） | **7** | **7.7** | 方法 B（§3.3。**本 round で採用**、§6） |
| rule 入口の stack 検査 | 0 | 0 | 方法 B |
| **残り**: rule/expression の関数呼出し 3,158、`mark`/`restore` 4,745、memo `lookup`/`store` 1,349、`join`/`combine` 3,120、終端照合 685、`fail` 461、効果 472 | **52** | **58.4** | 差分（**帰属は不確実**。方法 B で 1 つずつ切り離せない＝切ると受理が変わる機構が集まっている） |

残り 52 µs を回数で割ると 13,990 操作で **3.7 ns/操作**。全体平均 2.42 ns/操作より高いのは、
この束に「本物の分岐・memo の hash・関数呼出しの prologue/epilogue」という単価の高い操作が集まっているためで、
矛盾はしない。**この 52 µs を機構別にさらに割るには、方法 B ではなく `perf`（この host には無い）か、
生成器側で機構を選択的に外せる flag が要る。現時点では割れていないと明記する。**

## 3. AST 経路

### 3.1 AST は何を増やすか（確保の計数、GlobalAlloc）

| 入力 / mode | allocs/parse | alloc bytes/parse | AST node | 増分 allocs | 増分 bytes | node あたり |
|---|---:|---:|---:|---:|---:|---|
| complex 認識 | 330 | 32.0 KB | — | — | — | — |
| complex AST | 748 | 128.1 KB | 131 | **418** | **96.0 KB** | **3.2 回 / 733 byte** |
| complex-x64 認識 | 19,346 | 745 KB | — | — | — | — |
| complex-x64 AST | 45,426 | 6.64 MB | 8,195 | **26,080** | **5.90 MB** | **3.2 回 / 719 byte** |

`Ast` は 86 個の struct を持つ enum で、最大の variant（`FormulaExpr`）が 104 byte、つまり
**enum 1 個で約 112 byte**。加えて `Box<Ast>`・`Vec<Ast>`・`Vec<String>`・`String` が field ごとに
独立の確保になる。node あたり 3.2 回・733 byte は完全に説明がつく。

**認識だけの parse でも 330 回 / 32 KB 確保している**ことにも注意する（buffer pool は効いているのに）。
内訳は未分解だが、`ScopeStore::new()` が parse ごとに作られる（`BTreeMap` を持つ）・scope 宣言ごとに
`self.text(span)` が `String` を作る、が主な候補である。§4 の順位には入れていない（複数の小さな確保の束で、
1 つ外しても測れない）。

### 3.2 AST 増分 41 µs の帰属（complex）

baseline（session 2、9 round）: 認識 最小 **0.091**、AST 最小 **0.132** ms → **増分 41 µs**。

| bucket | µs | 増分に占める % | 方法 |
|---|---:|---:|---|
| **AST 構築**（`build_values` の arena 走査 + `Box`/`String`/`Vec` 確保 + `node()`） | **22** | 54% | `noast`（`build_values` を `Ok(vec![Ast::Null])` に）で −33 µs、うち value span 分 11 µs を引いた残り |
| **value span の収集**（`ast.collect`、`ParseResult::value_spans`） | **11** | 27% | `novspan`（`ast.collect` を飛ばす）で −11 µs |
| **出現収集**（`occurrences`、capture / lexical / token） | **7** | 17% | `noocc`（AST のときの `occurrences` を飛ばす）で −7 µs |
| `node_spans` の push（131 件） | **0** | 0% | `nonodespan`（`node()` の push を飛ばす）で −0 µs |
| scope の `references()` / `diagnostics()` の `to_vec` | **0** | 0% | `noscopediag` で −0 µs |
| mapping の型適合検査（`matches!` 連鎖 140 箇所・最大 87 分岐） | **0** | 0% | `notypecheck`（140 行を削除）で −0 µs。LLVM が畳んでいる |
| 合計（説明できた分） | **40** | 98% | — |

x64 でも比率は同じ（`noast` −2,453 µs = AST 最小 9.112 ms の 26.9%、`novspan` −842 µs = 9.2%、`noocc` −215 µs = 2.7%）。
**x64 の AST 増分は session 間で 2.1〜3.2 ms とばらついた**（同じ binary で最小値が 15% 動く）ので、
絶対値ではなく比率で読むこと。

### 3.3 補助の実測（§4 の候補の裏付け）

session 2（9 round 交互、Δ は最小値どうし）。baseline は complex 認識 0.091 / AST 0.132、x64 認識 5.875 / AST 9.112 ms。

| variant | complex 認識 | complex AST | x64 認識 | x64 AST | 意味 |
|---|---:|---:|---:|---:|---|
| `noguardbody`（fast mode で候補除外の再生を丸ごと省く） | **−7 µs (7.7%)** | **−5 µs (3.8%)** | **−279 µs (4.7%)** | **−776 µs (8.5%)** | 4 つ全てで符号が同じ。**採用した**（§6） |
| `noast`（AST 構築を `Ast::Null` に） | −2 µs（対照） | −33 µs (25.0%) | −45 µs（対照） | −2,453 µs (26.9%) | AST 構築の上限 |
| `novspan`（value span を作らない） | −2 µs（対照） | −11 µs (8.3%) | −35 µs（対照） | −842 µs (9.2%) | value span の費用 |
| `nonodespan`（`node_spans` を積まない） | 0 µs | −2 µs | −143 µs（雑音） | −198 µs（雑音） | ここには無い |
| `notokenevent`（認識で `Event::Token` を置かない） | **−1 µs** | 0 µs | **−69 µs (1.2%)** | −220 µs（雑音） | **負の結果**。§4.3 参照 |

**`notokenevent` は測って落ちた案である。** 「認識では誰も読まない `Event::Token`（595 回/parse、
event の 45%）を置かなければ event arena の費用が半分減る」という筋は通っているのに、実測は雑音以下だった。
`Event::Token` の push は `primitive()` の直後で、書き込む arena が既に L1 にあり、追加した分岐のほうが高い。
**字句 cache と同じ形の失敗**（置き換える先が既に安い）で、`noevent` の 12 µs は Token 以外の event
（`Capture` / `Rule` / `Values` / `Join`、713 回/parse）と、それらが作る arena の**総量**に由来する。

## 4. 候補（順位付き）

期待値は**本 round の計測だけ**から出している（過去 report の値は使っていない）。
「外れる条件」は、字句 cache の教訓（**置き換える先の仕事が既に安ければ、cache のほうが高い**）を各項目に当てはめたもの。

### 4.1 第 1 位: fast mode で候補除外の失敗再生 `guard_e*` を丸ごと省く（**本 round で採用済み**、§6）

- **期待**: complex 認識 −7 µs（7.7%）、complex AST −5 µs（3.8%）、x64 認識 −279 µs（4.7%）、x64 AST −776 µs（8.5%）（実測。§3.3）
- **中身**: `guard_e*` は「候補を呼ばずに、候補が残したはずの観測（display の reach/failures、主診断の
  Failure/Rule/Join）だけを再生する」関数である（`crates/ubnfc-rust/src/predict.rs` の doc）。
  **fast mode はその観測をどちらも記録しない**（D-023）。それでも guard は本体を走り、途中で
  `trivia_1::<false>(...)` という**本物の trivia 走査**まで行っている。819 回/parse（2.47 回/byte）。
  fast mode では `can_replay` の検査の直後に「失敗の Step をそのまま返す」でよい。
- **契約**: `04-…-contract.md` §2 と `03-design-v1.md` §3.4 の「除外後の失敗診断が元と同値であること」は
  **診断を記録する mode でだけ意味を持つ**。fast mode は定義上 farthest も hints も出さない（D-023）ので、
  再生しないことは観測を変えない。カーソル・memo・scope・AST には元から触れていない（`Step{ok:false,state,..}`）。
- **規模**: 生成器 1 箇所（`predict.rs` の guard 本体の先頭に 1 行）。実質 5 行。
- **外れる条件**: (a) 将来 fast mode でも何か観測を返すようになる（例: hints を診断 DAG 無しで作る案）。
  そのときは再生が必要になる。(b) `can_replay` が false のときは本物の候補関数へ落ちる経路が残るので、
  深い入力では効きが薄れる。(c) guard を消すと候補除外そのものが「ただの分岐」になり、
  i-cache が縮んで**別の場所が速くなる**可能性もある（実測はそれを含んだ値）。

### 4.2 第 2 位: fast mode で主診断 `Diag` の配管を単相化で消す

- **期待**: complex 認識 −13 µs（−14.6%）、complex AST −16 µs（−12.0%）、x64 認識 −655 µs（−11.4%）、
  x64 AST −943 µs（−11.9%）（実測。§2.2）
- **中身**: fast mode では `Diagnostics::push` が 1 度も呼ばれない（計数 0）のに、`join` 4,147 回・
  `rule` 2,334 回の**呼出し**と、`Step` の `diag: Diag`（8 byte）を全ての `combine` で運ぶ配管が残っている。
  `const DIAG: bool` で生成 parser を単相化すると、この配管が丸ごと死ぬ。
- **注記**: D-023 の報告は「`const DIAG` の上限は complex 認識であと 8 µs」として**作らない**と判断している。
  本 round の実測は 13 µs で、判断の前提が 1.6 倍ずれている。ただし両者は違うものを測っている
  （あちらは display も含む単相化の上限、こちらは主診断だけを定数化した差）ので、**再判断には
  `const DIAG` の実装を作って測り直す**必要がある。ここでは「前提が変わった」とだけ記録する。
- **契約**: 観測は変わらない（fast mode は既に診断を返さず、必要なら診断モードで解析し直す）。
- **規模**: 大きい。生成 parser（P4 で 20k 行）を 2 部にするか、`Session` を `const DIAG: bool` で
  単相化して全 rule 関数を型引数付きにする。**300〜800 行 + 生成物の倍増**。
- **外れる条件**: **i-cache**。round 4 で `limit` を 759 rule 関数へ inline しただけで +5 µs 悪化している。
  parser 本体を 2 倍にすると、単相化で得た 13 µs を i-cache miss が食い潰す可能性が高い。
  **これが本案の最大の失敗要因**で、作る前に「生成物を 2 倍にした binary を 1 つ作って測る」ことが要る。
  併せて display の単相化（§2.2 の −5 µs）も同じ判断に乗る。

### 4.3 第 3 位: 認識だけの parse で event arena を削る

- **期待（上限）**: complex 認識 −12 µs（−13.5%）、x64 認識 −904 µs（−15.7%）（実測。上限であることに注意）
- **中身**: P4 は `HAS_SCOPE == true` なので、**認識だけの parse でも event arena を全部作っている**
  （1,308 event/parse、`Event` は 80 byte なので complex で 105 KB、x64 で **6.4 MB の書き込み/parse**）。
  x64 で効きが大きいのは帯域のため。arena を必要とするのは scope 効果（`scope_captures` が読む `Event::Capture`）
  だけで、**`primitive()` が置く `Event::Token` は認識では誰も読まない**。
  2 段階が考えられるが、**安い方は測って落ちている**:
  1. ~~認識モードで `Event::Token`（595 event/parse、全体の 45%）を置かない~~ → `notokenevent` の実測は
     **雑音以下**（§3.3）。10 行で済む安全な案だが、得が無い。**やらない。**
  2. **全部**（要 front 解析）: 「この expression の部分木は scope capture site に到達し得るか」を plan に持たせ、
     到達し得ない site では認識モードで event を置かない。**12 µs はこちらでしか取れない。**
- **契約**: `03-design-v1.md` §3.6 は「認識中は capture の出現と構築レシピを保持」と書くが、これは
  **AST を作るときの話**で、`event()` には既に「AST でも lexical でも scope でも recovery でも無ければ置かない」
  という条件がある。今回はその条件を `HAS_SCOPE` について精密化するだけである。
  §3.8 の scope 契約（宣言・参照・診断・`scope.events`）は不変でなければならず、**これが受け入れ条件**。
- **規模**: front/plan の到達解析 + emitter で **200〜400 行**。
- **外れる条件**: (a) scope site に到達し得る expression が実は大半で、削れる event が少ない文法。
  P4 では scope site は 11 rule だけなので望みがあるが、**削減率を先に計数で確かめること**（`event` の
  うち scope 経路上のものを数える計数を足すのが先）。(b) AST 経路では何も削れない（AST は arena を読む）。
  したがって **AST 付きの利用者には効かない**。(c) `Event::Token` は `text_extent` / `has_token_text` /
  `semantic_text` が読むので、AST・lexical では絶対に落とせない。

### 4.4 第 4 位: AST 経路の出現収集と value span

- **期待**: `occurrences` の省略 −7 µs（complex AST の 17%）、`value_spans` の省略 −11 µs（同 27%）（実測）
- **中身**: `finish` は AST を作るときに arena を **3 回**走る（出現収集 `occurrences`、AST 構築
  `build_values`、その後 `ast.collect` で value span）。`captures` / `lexicalOccurrences` は
  `03-design-v1.md` §3.6 で「**オプション出力**」と明記されている（LSP・self-host 用）。
  `value_spans` も同様に常時必要とは書かれていない。`ParseOptions` に「出現を返すか」「value span を返すか」の
  opt-in を足せば、AST だけ欲しい利用者はこの 2 走査を払わない。
- **契約**: 公開 API の既定値変更になる（D-023 の `diagnostics` と同じ形）。
  self-host は `lexicalOccurrences` を使うので、self-host と driver は opt-in する。
- **規模**: 50〜120 行（`ParseOptions` 2 個 + `finish` の分岐 + 呼出し側の追従）。
- **外れる条件**: 利用者が結局どちらも要求する（driver は要求する）。その場合は何も速くならない。
  **既定を変えるだけの案なので、「誰が要らないか」を先に決めないと意味が無い。**

### 4.5 第 5 位: AST 表現の arena / SoA 化（過去に「約 45 µs」と見積もられた案）

- **本 round の実測**: AST 付きの増分は complex で **44 µs**（133 − 89）、x64 で **2,143 µs**（7,898 − 5,755）。
  その内訳は **構築 22 µs / value span 11 µs / 出現収集 7 µs**（§3.2）。
  つまり **「AST 構築そのもの」に残っているのは **22 µs（AST 増分の 54%、AST 付き全体の 17%）** であり、過去の「45 µs」は増分全体の値**である。
  `Box` / `String` / `Vec` を arena / SoA に置き換えて取れるのは、そのうち確保に由来する分だけになる。
- **契約**: `Ast` は公開型（`ast.rs` の 86 struct、`Box<Ast>` / `Vec<Ast>` / `String` を field に持つ）。
  SoA 化は **公開 API の破壊的変更**で、`03-design-v1.md` §6 の結果 API と golden（canonical JSON）に直撃する。
  `06 D-006` は「公開は record 維持、内部 arena は計測後」としており、**公開型を変えない内部 arena** なら
  契約は守れるが、そのときは最後に公開型へ materialize する費用が戻ってくる。
- **規模**: 非常に大きい（生成器の mapping 全面、公開型、golden、Java 側の対称性も要る）。
- **外れる条件**: **字句 cache と同じ形で外れる**。置き換える先（確保）が測ってみると増分の一部でしかなく、
  arena 化で増える間接参照（node を index で辿る）と materialize が節約を上回る。
  **本 round の数字は「まず内訳を割ってから決めよ」と言っている。着手前に `noast`（AST 構築だけを外す binary、
  §3.3）の値を根拠にすること。**

### 4.6 測って「ここには無い」と分かったもの（着手しない）

| 案 | 実測 | 判定 |
|---|---:|---|
| rule 入口の native stack 検査（`limit` の `addr_of!` + `abs_diff`）を軽くする | **0 µs**（complex 認識） | **やらない**。ここに時間は無い |
| mapping の型適合検査（`matches!` の連鎖、140 箇所・最大 87 分岐）を bitmask 化 | **0 µs**（complex AST） | **やらない**。LLVM が既に畳んでいる |
| `node_spans` の記録をやめる | **0 µs**（complex AST） | **やらない**。ここに時間は無い |
| memo を減らす・外す | +520 µs（6.8 倍） | **やらない**。memo は律速を作っている側ではなく消している側 |
| 候補除外（predict）を減らす | +86 µs（2.0 倍） | **やらない**。同上 |
| 認識で `Event::Token` を置かない（event の 45%） | **0 µs**（complex）/ −1.2%（x64 認識、雑音帯内） | **やらない**。§3.3 の負の結果 |
| 位置ごとの字句 cache | 既定 off（前 round） | 終端照合は 1.79 回/byte・全操作の 1.6% しかない。再訪しない |

## 5. 0.02 ms は届くか

**届かない。** 根拠は 3 つとも本 round の計測から出る。

1. **作業量**。complex は **1 byte あたり 111 回の runtime 操作**を実行する（§1）。
   0.02 ms = 60 ns/byte なので、仮に 1 操作 2.4 ns（本 round の実測平均）のままなら
   **1 byte あたり 25 操作**まで減らさねばならない＝**作業を 4.4 倍減らす**。
   単価を下げる案（§4.1〜4.4 の全部）を足しても、効くのは操作の 3 割ほどで、単価そのものは変わらない。
2. **投機の割合**。rule 評価 759 回に対して生き残る AST node は 131。5.8 倍の投機を
   1.5 倍以下にできなければ 4.4 倍は出ない。候補除外は既に 209 個の選択肢のうち **105 個（50%）**に
   guard を置いていて、それで認識時間が半分になっている（§2.2）。残り 104 個を全部除外できたとしても
   同じ比率では **もう 1 回 2 倍**が上限で、4.4 倍には足りない。しかも残りは「除外できないと証明された」
   （nullable・先読み・Extern・scope/recovery を持つ rule・policy 混在）ものが含まれる。
3. **現時点の到達点**。complex AST の最小値は 0.133 ms / 332 byte = **0.40 µs/byte** で、
   これは過去の report が基準にしてきた**手書き目安 0.4 µs/byte にちょうど並んでいる**。
   0.02 ms はその基準の **6.7 倍速い**値であり、そもそも「手書きより 1 桁速い」を要求している。

**現実的な床**（この設計のまま、本 round の実測の積み上げ）:

| 入力 / mode | 現在（最小） | §4.1〜4.4 を全部入れた見込み | 加えて候補除外を全選択肢へ広げた場合 | 長期目標 |
|---|---:|---:|---:|---:|
| complex 認識 | 0.089 ms | **0.055〜0.065 ms** | 0.035〜0.045 ms | — |
| complex AST | 0.133 ms | **0.085〜0.100 ms** | 0.055〜0.070 ms | 0.02 ms（**届かない**） |
| complex-x64 認識 | 5.76 ms | **3.6〜4.3 ms** | 2.2〜3.0 ms | — |
| complex-x64 AST | 7.90 ms | **5.3〜6.2 ms** | 3.4〜4.4 ms | 1.5 ms（**届かない**） |

見込みは各案の実測 Δ を単純に足したうえで、重なり（display と Diag の単相化は同じ i-cache 予算を食う、
event と AST は排他）を 25% 割り引いた値。**2 列目までは本 round の実測の積み上げで裏付けがある。
3 列目は裏付けが無い外挿で、証明できない候補除外を含むので実現性は低い。**

床を更に下げるには設計を変えるしかない: (a) 文法ごとに「決定的に降りられる」ことを証明して
rule 関数の入口検査・memo・mark/restore を丸ごと外す（`03-design-v1.md` §3.7 の「再訪不能を証明できた単位」を
P4 の 124 rule へ実際に適用する）、(b) 優先順位の連鎖（NumberExpression → NumberTerm → NumberFactor → …）を
precedence climbing へ lowering して降下段数を減らす。どちらも **front の解析追加**であり、
runtime の微修正では届かない。

## 6. 本 round で入れた変更（1 つだけ）

`crates/ubnfc-rust/src/predict.rs`、8 行（生成物は 105 + 39 箇所へ 1 行ずつ展開）。§4.1 の案。

```rust
// 再生が作るのは失敗した候補の**観測**（display の reach / failures と主診断 DAG）だけで、
// カーソル・memo・scope・event には触れない。fast mode（D-023）はその観測をどちらも記録しないので、
// 再生する中身が無く、丸ごと省ける。
s.push_str("if !self.options.diagnostics {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}\n");
```

fast mode で `guard_e*` の本体（display の reach / failures、主診断の `fail`/`join`/`rule`、および
**本物の `trivia_N::<false>` 走査**）を全て省き、候補が返すはずの `Step{ok:false,state,..}` をそのまま返す。
`diag` は fast mode では必ず `Diag::NONE` になる（`Diagnostics::fail` が `enabled==false` で `NONE` を返し、
`join(NONE,NONE)` も `rule(_,NONE)` も `NONE`）ので、値としても同一である。

### 検証

- `cargo test --workspace --offline` は下の既存失敗 1 件を除いて緑。
  とくに `examples/p4-rust/tests/fixtures.rs` の
  **`predict_matches_reference_path_on_truncated_and_corrupted_inputs`（候補除外あり／なしの同値、312 秒）** と
  **fast mode と診断モードの同値**が、この変更を直接守る軸である。どちらも緑。
- `cargo clippy --workspace --all-targets --offline -- -D warnings` 緑、`cargo fmt --all -- --check` 緑。
- 生成物は**手で直していない**。`examples/p4-rust/scripts/generate.sh --write` と
  `ubnfc-rust --ir …ubnf.ubnf.ir.json --out … --no-emit-driver` で再生成し、再実行で byte 一致を確認した
  （`tests/regenerate.rs` / `crates/ubnfc-selfhost/tests/regenerate.rs`）。selfhost の固定点も緑。
- corpus: `ubnfc-harness run-all --backend rust` は **Pass 1292 / 想定拒否 30 / known 28 / Fail 0 / 未観測 75**
  （`docs/reports/2026-09-23-rust-corpus-profile.md`）。指示の期待値と完全一致。
- **既存の失敗（本 round とは無関係）**: `crates/ubnfc-fuzz --test campaign` の
  `a_fixed_seed_campaign_reaches_both_backends_and_finds_no_unregistered_divergence` が
  `文法が build できていない: {"build:java": 3, "internal": 1}` で落ちる。
  **本 round の変更を戻した木でも同じ内容で落ちる**ことを確認済み（Java backend 側、`fix/fuzz-front-and-recovery` の領域）。

### A/B（採用後の確認、9 round 交互、最小値）

採用後の binary（emitter からの再生成 + merge 後）と、変更前の binary を**同一セッションで交互に 9 round**
（`taskset -c 27`、load average 8〜19、他エージェントの Java ベンチが並走）。Δ は最小値どうし。

| 入力 | mode | 変更前 中央値 / **最小** | 変更後 中央値 / **最小** | Δ（最小） |
|---|---|---:|---:|---:|
| complex (332 B) | 認識 | 0.096 / **0.0910** | 0.091 / **0.0850** | **−6 µs（−6.6%）** |
| complex | AST | 0.143 / **0.1360** | 0.136 / **0.1280** | **−8 µs（−5.9%）** |
| complex-x64 (20,923 B) | 認識 | 6.806 / **6.229** | 6.424 / **5.791** | **−438 µs（−7.0%）** |
| complex-x64 | AST | 12.358 / **10.751** | 11.687 / **10.223** | **−528 µs（−4.9%）** |

4 つの組合せ全てで中央値も最小値も同じ符号。切り離し計測（§3.3 の `noguardbody`、別セッション 9 round で
−7.7% / −3.8% / −4.7% / −8.5%）とも 2〜4 ポイント以内で一致する。
**得は fast mode にしか出ない**（診断モードは従来どおり再生する）。

## 7. 現時点の絶対値（参考、本 branch・静かな窓・`taskset -c 27`）

`examples/p4-rust` の bench（warmup 後 5 sample の中央値）。確保は別 parse の GlobalAlloc 計数。

| 入力 | bytes | mode | 中央値 ms | min | µs/byte | allocs/parse | alloc bytes/parse | rule 評価 | memo hit | event | AST node |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| complex | 332 | 認識 | 0.098 | 0.095 | 0.29 | 330 | 32,009 | 759 | 169 | 1,308 | — |
| complex | 332 | AST | 0.146 | 0.140 | 0.44 | 748 | 128,054 | 759 | 169 | 1,308 | 131 |
| complex-x4 | 1,293 | 認識 | 0.389 | 0.353 | 0.30 | 1,241 | 64,232 | 2,919 | 664 | 5,066 | — |
| complex-x4 | 1,293 | AST | 0.525 | 0.500 | 0.41 | 2,884 | 437,739 | 2,919 | 664 | 5,066 | 515 |
| complex-x16 | 5,179 | 認識 | 1.569 | 1.508 | 0.30 | 4,867 | 199,315 | 11,559 | 2,644 | 20,102 | — |
| complex-x16 | 5,179 | AST | 2.222 | 2.038 | 0.43 | 11,402 | 1,674,073 | 11,559 | 2,644 | 20,102 | 2,051 |
| complex-x64 | 20,923 | 認識 | 7.630 | 6.657 | 0.36 | 19,346 | 745,075 | 46,119 | 10,564 | 80,246 | — |
| complex-x64 | 20,923 | AST | 9.133 | 8.547 | 0.44 | 45,426 | 6,643,713 | 46,119 | 10,564 | 80,246 | 8,195 |

µs/byte は x4 / x16 で 0.30（認識）・0.41〜0.43（AST）と**平ら**で、x64 だけ 20% ほど上がる。
これは §1 の「操作は byte あたりで減っているのに時間は増える」＝ **L2 / L3 の作業集合**を示す
（x64 の AST は 1 parse で 6.6 MB 確保し、event arena だけで 6.4 MB 書く）。
**入力長に対する超線形は算法にはもう無く、memory hierarchy にだけ残っている。**

## 8. 次にやるなら（順序）

1. **§4.2 の前提検査**: `const DIAG: bool` で生成物を 2 倍にした binary を 1 つだけ作り、i-cache の代償を測る。
   これで 4.2 と display の単相化（合計 18 µs の見込み）の採否が決まる。作らずに実装を始めないこと。
2. **§4.3 の削減率の計数**: 「scope capture site に到達し得る event」を数える計数を `counters` に足す。
   1,308 event のうち何回かが分かるまで、front の到達解析には手を付けない。
3. **§4.4 の利用者調査**: `captures` / `lexicalOccurrences` / `value_spans` を誰が要求するかを先に決める。
   要求者が全員なら 18 µs の案は消える。
4. 残り 52 µs（認識の 58%）を割る手段が要る。`perf` が使える host を 1 台確保するのが、
   ここから先のどの案よりも先に効く。
