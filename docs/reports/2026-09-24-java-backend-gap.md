# Java backend と手書き目標形の差（2026-09-24、issue #3 第 1 ラウンド）

`docs/reports/2026-09-23-parser-speed-comparison.md` §7-1 が出した「同じ IR から出した parser で、
Rust は手書き目標形より 4.5 倍速いのに Java は 4.3 倍遅い」を、**先に profile を採ってから**潰す。
issue #3 の指示どおり Rust backend の最適化を推測で移植することはしていない。

本ラウンドで入れたのは 2 件で、主因は 1 件目（`Session.combine` の多項連結の線形化）。これは
**計測して初めて分かった**もので、Rust backend の持つ最適化（左因数分解・観測の静的再生・
バッファ再利用）のどれでもない。

## 結論（先に）

- 512 atom の Operators で 1 parse が **10.5 MB → 2.01 MB**（認識）、**10.8 MB → 2.34 MB**（AST）。
  手書き目標形（2.09 MB / 2.77 MB）を**下回った**。
- 時間は **1.82 → 0.76 ms**（認識、中央値）、**2.53 → 1.13 ms**（AST）。
  手書き目標形は 0.376 / 0.693 ms なので、**目標（0.356 ms）には届かず残り 2.0 倍**。
- 10.5 MB の **85%（8.9 MB）は `Session.combine` の逐次 2 項連結**だった。
  511 周の repeat を畳むときに、k 番目の項目が「それまでの合計」をもう一度複製していた（Θ(要素数 × 項目数)）。
  これは文法の repeat が長いときだけ出るので、**P4 ではほとんど効かない**（後述）。
- P4（complex / x64）は割当 −2〜−9%、時間は**有意差なし**（交互 3 周の比が 0.91〜1.08 で方向が一定でない）。P4 complex 0.2 ms / x64 15 ms の目標は**未達のまま**。
  P4 の残りは M5 報告の「残り 3」（Match 木の event arena 化）が支配しており、本ラウンドでは触っていない。

## 計測環境

- AMD Ryzen 9 7950X（32 論理 CPU）、86 GB、WSL2 Linux 6.18.33.2。**共有機で別エージェントが並走**し、
  作業中の load average は 9〜34 で動いた。`taskset -c 28`（family B の一部は 29）で 1 コアに固定、単一 thread。
- Oracle JDK 21.0.9 / HotSpot。`-Xms512m -Xmx2g -Xss16m`。
- **一次証拠は割当 byte（負荷に依存しない）と 1 parse の計数**、時間は
  (a) 同一 JVM 内で baseline / step1 / 本 PR / 手書きを 1 run ごとに巡回して測る A/B と
  (b) baseline binary と step1 binary を交互に 3 周走らせた値だけを載せる。
  作業終盤は他エージェントのベンチが重なって load average が 44 まで上がったので、その区間の時間は採らない。
- P4 の A/B は `origin/master` の issue #1 修正（`P4Scanners` が単引用符の value span を 1 単位内側へ寄せる）を
  取り込む**前**の binary で測った。この修正は STRING token 1 件あたり定数時間の span 補正だけで、
  rule 評価・memo 操作・確保の回数を変えない（本 PR はそれを取り込んだうえで検証を通している）。
- ドライバは `benchmarks/`（family B）と `examples/p4-java` の `Benchmark`（P4）。生成物は
  `mktemp -d` 配下に baseline（`origin/master` の jar で生成）・step1・本 PR の 3 系統を作り、
  package を分けて同じ JVM に載せた。

## profile（baseline、Operators 512 atom・認識）

JFR（`settings=profile`、40 秒、19,259 parse）。割当サンプルの重み合計は 189.7 GB で、
`ThreadMXBean` の 10,490,440 B/parse と 1 割以内で整合する。

### 割当サンプル — 確保位置別（10.49 MB に対する推定内訳）

| 位置 | 割合 | 概算 |
|---|---:|---:|
| `Session.concat`（`combine` の逐次 2 項連結） | **84.5%** | 8.86 MB |
| `Session.rule`（recipe / field list / Memo） | 5.2% | 546 KB |
| `Session.captured`（capture 出現の append） | 2.5% | 266 KB |
| `Session.combine`（Match 本体） | 1.3% | 134 KB |
| `Diagnostics.leaveRule`（memo payload の snapshot） | 1.2% | 121 KB |
| `Session.appended` | 0.9% | 98 KB |
| `Session.sequence` / `repeat` / `result` / `Match.empty` ほか | 3.5% | 370 KB |

### 割当サンプル — 型別

`Object[]` **85.1%** / `Session$Match` 4.3% / `ArrayList` 2.0% / `ImmutableCollections$List12` 1.6% /
`Session$Memo` 0.9% / `Recipe$Value` 0.8% / `api.Capture` 0.8% / `Diagnostics$Snapshot` 0.7% /
`Session$Match[]` 0.7% / `Recipe` 0.6% / `int[]` 0.5% / `Session$Occurrence` 0.4%。

→ **`Object[]` の 85% が「中間 ArrayList の backing array」**。これが 10.5 MB の正体で、
Match 木そのもの（Match + Occurrence + Recipe + Value + List wrapper ≈ 8%）は 1 MB に満たない。

### 実行サンプル — 自己時間（2,136 サンプル）

| 位置 | 割合 |
|---|---:|
| `ArrayList.addAll` | 24.2% |
| `Session.rule` | 15.1% |
| `Arrays.copyOf` | 12.9% |
| `Session.result`（ParseResult の capture / span 化） | 9.3% |
| `Session.capture` | 5.5% |
| `ImmutableCollections.listCopy` | 4.5% |
| `Session.combine` | 4.1% |
| `List.of` / `Diagnostics.leaveRule` / `Session.trivia` | 9.6% |
| `Match.empty` / `sequence` / `concat` ほか | 8.6% |

→ **連結だけで 38.6%**（`addAll` + `copyOf` + `concat`）。

### 10.5 MB の説明（なぜ Θ(T·n) になるか）

`Expression ::= Term { AddOp Term }` の repeat は 512 atom の入力で **511 周**回る。
`repeat` は周回ごとの `Match` を集め、最後に `combine(matches, …)` で
`nodes` / `local` / `captures` / `traces` / `recoveries` / `values` の 6 列を畳む。
旧実装はこれを **2 項 `concat` の逐次適用**で書いていた:

```java
for (item : items) if (!item.captures().isEmpty()) captures = concat(captures, item.captures());
```

`concat` は毎回 `new ArrayList<>(head.size() + tail.size())` を作るので、k 番目の項目を足すときに
それまでの k−1 項ぶんを丸ごと複製する。項目数 n・最終要素数 T に対して **Θ(T·n)**。
512 atom では `captures` 列だけで最終 2,558 要素 × 511 周 ≈ 65 万要素ぶんの複写になり、
`values` / `local` 列と合わせて 8.9 MB になっていた。

### 1 parse の計数（Operators 512 atom、負荷に依存しない）

| 計数 | 認識 | AST |
|---|---:|---:|
| rule 呼出し | 2,561 | 2,561 |
| うち memo hit | **0** | **0** |
| memo 保存 | 2,561 | 2,561 |
| expression 呼出し | 0 | 0 |
| 失敗記録 `fail()` | 12,810 | 12,810 |
| trivia 呼出し | 5,633 | 5,633 |
| `combine` 呼出し | 2,561 | 2,561 |
| capture 出現の append | 2,047 | 2,047 |

**memo が 1 度も当たらない。** 512 atom の Operators は rule 呼出し 2,561 回すべてが memo miss で、
2,561 件を保存だけして捨てている（`Memo` record + `Diagnostics.Snapshot` + `MemoTable` の probe が丸損）。
参考値として `ParseOptions.memo=false` で測ると（同一 JVM 内で on/off 交互、9 run 中央値。
**この区間は別計測が並走しており絶対値は 3 倍ほど膨らんでいる**ので比だけを読む）:

| mode | memo=on | memo=off | 比 | on 割当 | off 割当 |
|---|---:|---:|---:|---:|---:|
| 認識 | 3.514 ms | 2.766 ms | 0.79 | 2,008,360 | 1,712,512 |
| AST | 2.872 ms | 2.600 ms | 0.91 | 2,364,336 | 2,068,488 |

この文法では memo を持つ意味が無い。ただし memo の要否は生成時の解析（M5 第 1 段の「memo 省略の証明」）の
判断で、Operators の 6 規則はいずれも `successMemo=true` / `failureMemo=true` のまま残っている。
**P4 では memo が効く**（M5 報告で complex 1 parse あたり memo hit 464 件）ので、
解析側の条件を変えるのは本ラウンドの範囲外にして、次ラウンドの項目に挙げる。

計数は本 PR の変更で 1 つも動かない（`combine` は確保の仕方だけを変えたので、rule 評価・memo 操作・
失敗記録・trivia の回数は baseline と同一）。**時間と割当の差は全て「1 操作あたりの費用」の差**である。

## 施策と A/B

### step 1 — `combine` の多項連結を線形化（本ラウンドの主変更）

列ごとに「1 回目の走査で合計要素数を数え、2 回目で合計長の list を 1 個だけ確保する」形にした
（`ubnfc-java/src/main/resources/templates/internal/Session.java` の `combine` / `join`）。
寄与する項目が 1 つだけなら従来の 2 項 `concat` と同じくその list をそのまま返すので、
**要素の同一性・順序・list の内容は変わらない**。Θ(T·n) → Θ(T)。

### step 2 — field list の遅延確保と `Recipe.Value` の防御的複製の削除

`rule()` の recipe 構築で、値を 1 つも集めない field のために `ArrayList` を作るのをやめた。
`Recipe.Value` の compact constructor が `nodes` / `elements` を毎回 `List.copyOf` していたのもやめた
（どちらも `Session` の中だけで組み立てられ、構築後に変更しない。`Match` の list と同じ約束）。

**family B での効果は小さい**（Operators 512 atom の割当 −1.2%、Power power-64 は逆に +1.3%。
時間はどちらも差なし）。小さい list では `List.copyOf` の作る `List12` のほうが
`ArrayList` + `Object[]` より小さいので、複製をやめると却って増えることがある。
**効くのは P4 のほう**で、step 1 単独では complex の割当が +1.0% 増えるところを、
step 2 と合わせて **−1.9%** に反転させている（後掲 P4 表）。

### 試して戻したもの（記録）

- **`combine` の連結を `ArrayList.addAll` から要素ループへ**（`addAll` が内部で `toArray()` の
  中間配列を作るのを避ける狙い）。割当は Operators 512 atom で 2.01 → 1.79 MB、Power power-64 で
  343 → 267 KB と減ったが、**power-64 の時間が 1.45 倍に悪化**した
  （認識 0.0743 → 0.1109 ms、AST 0.0952 → 0.1300 ms、3 セッションで再現）。
  `ArrayList.addAll` の `toArray` + `System.arraycopy` 経路のほうが JIT に扱いやすい。**戻した**。
- **field list の初期容量を集める個数から見積もる**（`new ArrayList<>(2)` 等）。
  既定容量 10 より小さく始めると再確保が増え、power-64 の悪化は直らなかった。**戻した**。

### family B（Operators / Power）— 同一 JVM で baseline / step1 / 本 PR / 手書きを巡回

`bench.runs=9`、`bench.iterations=400`、warmup 20,000。load average 10〜21。

| 文法 | 入力 | mode | baseline 中央値 | step1 | 本 PR | 手書き目標形 | baseline 割当 | 本 PR 割当 | 手書き割当 |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| Operators | `1+2*3` (5 B) | 認識 | 0.0233 | 0.0332 | 0.0229 | 0.0226 | 20,704 | 20,616 | 10,928 |
| Operators | `1+2*3` | AST | 0.0221 | 0.0213 | 0.0267 | 0.0164 | 22,624 | 23,024 | 15,128 |
| Operators | 括弧深さ 8 (21 B) | 認識 | 0.0364 | 0.0400 | 0.0400 | 0.0233 | 48,840 | 48,024 | 86,936 |
| Operators | 括弧深さ 8 | AST | 0.0214 | 0.0212 | 0.0216 | 0.0136 | 54,040 | 52,824 | 97,944 |
| Operators | **512 atom (1,023 B)** | **認識** | **1.817** | 0.800 | **0.763** | **0.376** | **10,490,440** | **2,008,328** | 2,094,576 |
| Operators | **512 atom** | **AST** | **2.529** | 1.238 | **1.131** | **0.693** | **10,846,368** | **2,343,776** | 2,772,912 |
| Power | `2^3^2` (5 B) | 認識 | 0.0057 | 0.0172 | 0.0058 | 0.0017 | 16,416 | 16,600 | 5,328 |
| Power | `2^3^2` | AST | 0.0160 | 0.0121 | 0.0169 | 0.0015 | 17,768 | 18,312 | 7,776 |
| Power | 64 段 (127 B) | 認識 | 0.0789 | 0.0803 | 0.0859 | 0.0241 | 338,624 | 343,184 | 147,056 |
| Power | 64 段 | AST | 0.0923 | 0.0949 | 0.0932 | 0.0309 | 384,744 | 389,304 | 197,936 |

小さい入力（5〜127 B）は 1 parse が 5〜80 µs なので、共有機の負荷が run 中央値をそのまま揺らす
（同じ binary の再測定が 0.0057 と 0.0172 になる）。**小入力の行から差を読んではいけない。**
差が読めるのは 512 atom の行と、全行の割当である。

### P4（`examples/p4-java`）— baseline binary と本 PR binary を交互に 3 周

時間は **baseline binary と step1 binary を交互に 3 周**（各周 warmup 2,000・5 run、load average 12〜31）、
割当は **baseline binary と本 PR binary（step1+2）を交互に 1 周**した値（割当は決定的なので 1 周で足りる）。
時間の 2 度目の測定は load average が 23〜44 まで上がった区間に当たったので採らない。

| fixture | mode | baseline 最小 / 中央値 | step1 最小 / 中央値 | 時間比（最小） | baseline 割当 | step1 割当 | 本 PR 割当 | 本 PR / baseline |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| complex | 認識 | 0.727 / 0.741 | 0.711 / 0.732 | 0.98 | 503,680 | 508,480 | 499,896 | 0.981 |
| complex | AST | 0.565 / 0.799 | 0.595 / 0.802 | 1.05 | 535,856 | 540,656 | 532,072 | 0.984 |
| complex-x4 | 認識 | 1.852 / 1.866 | 1.974 / 2.008 | 1.07 | 1,912,680 | 1,931,600 | — | — |
| complex-x4 | AST | 1.997 / 2.452 | 2.032 / 2.095 | 1.02 | 2,042,488 | 2,061,408 | — | — |
| complex-x16 | 認識 | 7.319 / 10.144 | 6.803 / 9.459 | 0.93 | 7,660,520 | 7,592,752 | 7,530,944 | 0.972 |
| complex-x16 | AST | 9.517 / 9.614 | 10.235 / 12.644 | 1.08 | 8,179,832 | 8,112,064 | 8,050,256 | 0.975 |
| complex-x64 | 認識 | 39.166 / 44.491 | 40.805 / 43.148 | 1.04 | 32,917,488 | 30,429,368 | 30,182,088 | 0.907 |
| complex-x64 | AST | 44.598 / 48.521 | 40.723 / 49.196 | 0.91 | 34,994,944 | 32,506,824 | 32,259,544 | 0.914 |

時間比は 0.91〜1.08 に散らばり、**方向が一定でない = 有意差なし**。
割当は complex で −2%、complex-x64 で −9%。step 1 単独だと complex の割当は逆に **+1%** 増える
（`join` が「合計長で 1 回確保する」ので、2 項のうち片方だけが空という P4 に多い形では
旧 `concat` の「空なら相手をそのまま返す」に負ける場面がある）。step 2 がその分を取り返して −2% になる。

**P4 では効かない。** P4 の repeat は周回数が小さく（`combine` の項目数が 2〜3）、
Θ(T·n) の n が立たないため。割当が x64 で 1 割弱減るのは、長めの列を持つ数少ない `combine` のぶん。

## 目標に対する到達

| 目標 | 値 | 本 PR | 判定 |
|---|---|---|---|
| Operators 512 atom ≤ 0.356 ms（手書き Java 目標形） | 0.376 ms（本セッションの手書き実測） | 0.763 ms（認識） | **未達（2.0 倍）** |
| P4 complex ≤ 0.2 ms | — | 0.596 ms（AST） | **未達** |
| P4 complex-x64 ≤ 15 ms | — | 56.5（この区間は負荷 23〜44。負荷 12〜19 の区間では 44〜49） ms（AST） | **未達** |
| 割当で手書き目標形以下 | 2.09 MB（認識）/ 2.77 MB（AST） | 2.01 MB / 2.34 MB | **達成** |

### 残り 2 倍の内訳（step 1 後の profile、同条件）

| 位置 | 自己時間 | 手書き目標形に同じ仕事があるか |
|---|---:|---|
| `Session.rule` | 20.6% | ある（`begin`/`end` + memo）。ただし生成側は recipe・field list・scope effect の分岐を持つ |
| `Session.result` | 12.9% | **無い**。生成 parser は認識モードでも capture 出現（512 atom で約 2,500 件）と span を必ず作る。手書き側は `buildAst=false` のとき `Events` を作らないので `List.of()` を返す |
| `Session.capture` | 10.3% | **無い**（同上。`project` が意味値 `Value` を毎回作る） |
| `ImmutableCollections.listCopy` | 7.2% | 無い。**この分は step 2 が落とした**（`Recipe.Value` の防御的複製） |
| `Diagnostics.leaveRule` | 6.1% | ある（snapshot を memo payload に入れる） |
| `Session.trivia` | 5.9% | ある |
| `combine` / `join` / `captured` / `Match.empty` / `sequence` | 21.5% | **無い**（手書きは Match 木を作らず、capture は O(1) の cons list） |

（step 1 後の binary で採った profile。step 2 は `listCopy` の行を消すだけで他の順位は変えない。）

つまり**残りの約 2 倍は「認識モードでも Match 木と capture 出現を必ず作る」ことに由来する**。
これは M5 報告の「残り 3（Match 木の廃止 = event arena 化）」と D-023 の二段構えそのもので、
本ラウンドの範囲（推測で移植しない・1 施策ずつ A/B）には収まらない。**正直に未達として残す。**

`MemoTable.locate` の自己時間は run により 1.6%〜16.7% と大きく振れた（表は 1,023 B の入力に対し
8,192 slot × 32 byte = 256 KB の表を毎 parse 走査するので、同居プロセスの L2/L3 汚染をそのまま拾う）。
共有機では slot 配置の改善効果を分離できないので、本ラウンドでは手を付けていない。

## 同値性の検証

- `examples/p4-java` の `PredictEquivalenceTest`（全 16 fixture × 切り詰め・1 文字破壊 884 入力 ×
  入口 3 種 × AST 有無 = **5,304 件**）の `Session.predict` on/off 比較が緑。
- **corpus**: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java`
  → **Java Pass 1,310 / 期待拒否 Pass 28 / Known 28 / Fail 0 / 未観測 59**
  （`docs/reports/2026-09-24-java-corpus-perf3.{md,json}`）。系統別の表は
  `2026-09-23-java-corpus-contract.md` および直近の `2026-09-23-java-corpus-crosscheckfix.md` と
  **1 行も違わない**（`diff` で確認）。
- **IR golden 不変**: `crates/ubnfc-front` を変更していないので IR も self-host も動かない
  （`cargo test --workspace --offline` が golden を照合して緑）。
- **差分ファジング**: `cargo test -p ubnfc-fuzz --offline` が固定 seed キャンペーン
  （`a_fixed_seed_campaign_reaches_both_backends_and_finds_no_unregistered_divergence` ほか 4 件）、
  `choice_reachability` 2 件、`right_assoc_span` 1 件、unit 22 件すべて緑。
- **1 parse の計数が完全に一致**（Operators 512 atom、認識・AST とも
  rule 2,561 / memo hit 0 / memo store 2,561 / fail 12,810 / trivia 5,633 / combine 2,561 /
  capture append 2,047）。確保の仕方だけを変えたことの直接の証拠。
- `ubnfc-java` / `ubnfc-java/runtime-template` / `examples/p4-java` の maven test 緑、
  `examples/p4-java/scripts/generate.sh --check` 差分なし、`cargo fmt --check` と
  `cargo clippy --workspace --all-targets -- -D warnings` 緑。
- `origin/master`（issue #1 の `P4Scanners` 引用符修正、D-050〜D-053 を含む）を取り込んだ上での結果。
  本 PR の D 行は **D-055**。

## 残り（次のラウンドの順序）

1. **認識モードで Match 木を作らない**（D-023 の二段構えの Java 版 = M5 報告「残り 3」）。
   `Session.result` + `capture` + `combine` 系で自己時間の約 45%。手書き目標形との差の大半。
2. **capture 出現を append-only の cons list にする**（`appended` の O(k) 複写をやめる）。
3. `MemoTable` の slot 配置（4 long/slot、load factor 0.5）。**共有機では分離して測れない**ので、
   専有できる区間を取ってから。
4. 左因数分解（M5 報告「残り 2」）と token 葉を含む候補の除外（同「残り 1」）。
   本ラウンドの profile では、どちらも Operators / Power には出番が無い（選択が 2 候補しか無い）。

## 再現

```bash
cd ubnfc-java && mvn -o -q -DskipTests package && mvn -o test
bash examples/p4-java/scripts/generate.sh --check
cd examples/p4-java && mvn -o -q -DskipTests package && mvn -o test
# family B（生成 Java と手書き目標形）
WORK=$(mktemp -d) M2_SEED=... benchmarks/scripts/prepare.sh && WORK=$WORK benchmarks/scripts/run-family.sh b
# P4
CORE=28 bash examples/p4-java/scripts/benchmark.sh
# corpus
cargo run -q --offline --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-24-java-corpus-perf3.md --json docs/reports/2026-09-24-java-corpus-perf3.json
```
