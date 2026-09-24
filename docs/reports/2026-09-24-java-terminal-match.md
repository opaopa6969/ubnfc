# 認識専用経路の終端が `Match` を作らない（Java backend、issue #55）

2026-09-24。issue [#55](https://github.com/opaopa6969/ubnfc/issues/55)。
前提は [`2026-09-24-java-d066.md`](2026-09-24-java-d066.md) §3 の申し送り
（json-1m 1 parse で `Match.empty` 2,533,873 個 = 2.53 個/byte、1 個 64.5 B で割当のほぼ全部）と
[`2026-09-24-java-two-mode.md`](2026-09-24-java-two-mode.md)（D-056 の二段構え）、
[`2026-09-24-java-recognition.md`](2026-09-24-java-recognition.md)（D-059 capture 迂回）、
[`2026-09-24-java-ast-captures.md`](2026-09-24-java-ast-captures.md)（D-068）。

## 0. 結論

**認識専用経路（`tree=false`）の成功値から `Match` の割当を消した**。終端も結合子も
「成功したが情報を持たない」ことだけを表す**共有の不変実例 1 個**（`Session.OK`）を返す。
AST 経路（`tree=true`）は 1 byte も変えていない。

| もの | before | **after** |
|---|---:|---:|
| json-1m 認識 1 parse の `Match.empty` | 2,533,873 | **0** |
| json-1m 認識 1 parse の `Match` 生成（全種） | 2,857,582 | **0** |
| json-1m 認識 1 parse の割当 | 163,343,888 B | **3,319,296 B**（0.020 倍） |
| json-1m 認識（最小値、同一 JVM 交互 session 2） | 50.52 ms / 0.0505 µs/byte | **39.48 ms / 0.0395 µs/byte** |
| **json-1m の Java / Rust（同一走行・最小値、Rust は #64 前）** | **1.49 倍**（#49） | **1.16 倍** |
| json-1m の Java / Rust（同、Rust は #64 = D-072 bulk scan 後） | — | 2.90 倍 |
| Operators 512 atom 認識（最小値） | 0.1012 ms | **0.0873 ms** |

**issue の目標「JSON 1 MB で Rust の 1.2 倍以内」は、issue が基準にした Rust（#49 時点の 0.0348 µs/byte →
上限 0.042）に対しては達成した**（同一走行の最小値で ANTLR4 0.0104 / ubnfc Rust 0.0331 /
**ubnfc Java 0.0385 µs/byte**、master merge 後の最終 build でも Java 0.0397。§2.3）。
**ただし作業中に PR #64（D-072、観測を要求しない終端領域を 1 本の走査へ畳む。Rust だけ）が入り、
Rust は 0.0137 µs/byte へ下がったので、最新の Rust に対しては 2.9 倍**である。この差は
Java に bulk scan が無いことによるもので、Java 版は #69 が持つ（§2.3.1）。
割当の目標「Rust 並み（≤80 MB）」も達成で、実際には **Rust の arena 76.6 MB の 0.043 倍**、
ANTLR4 の 27.1 MB の 0.12 倍になった。

**適用は「`@scope` 効果を 1 つも持たない文法」に限る**（§1.3）。P4 は `@scope` を使うので
参照経路のまま（非退行のみ）。残る回数の異常は §3 に置いた。

---

## 1. 何を変えたか

### 1.1 契約（D-075）

認識専用経路が終端から必要とするのは「消費した位置」だけで、それは `Frame`（`f.c` / `f.m`）が
運んでいる。`Match` は「成功したか（null か否か）」と、AST 経路のための中身を運ぶ型である。
`tree=false` では中身（nodes / local / captures / traces / recoveries / items / values）は
すべて空の共有 `List.of()` なので、**残るのは `start` / `end` だけ**だった。

**D-075 はその `start` / `end` も読まれないことを生成時に証明し、共有実例 1 個を返す**。

```java
/** 認識専用経路の「成功したが情報を持たない」ことだけを表す共有実例。 */
public static final Match OK = Match.empty(0, 0);
/** 成功の返り値。認識専用経路（plain）では区間が読まれないので共有実例を返す。 */
private Match success(int start, int end) { return plain ? OK : Match.empty(start, end); }
```

`plain = plainMatch && plainRecognition && !tree` で、`plainRecognition` は生成側が
`new Session(source, options, K0, twoMode, plainRecognition)` の第 5 引数で渡す文法ごとの可否。
置き換えたのは `literal` / `any` / `range` / `number` / `dfa` / `regex` / `builtin` / `external` /
`look` / `sequence` / `repeat` / `separated` / `choice`（`combine` / `fast`）/ `rule` /
左因数分解の `tail` の成功値で、**認識専用経路の成功値はすべてこの 1 個**になる。
`builtin` の `Quoted` / `SingleQuoted` と `external` は、この経路では読まれない
`source.substring(...)` の確保もまとめて消える。

### 1.2 どこで区間が読まれるか（適用条件）

`tree=false` の経路で `Match` の `start` / `end` / `text` / 出現列を読むのは **2 か所だけ**である。

1. **`Session.applyEffects`**（`@scope` 効果）。site を持たない効果（`enter` / `leave` など）が
   `input.cpIndex(result.end())` を効果の位置として記録する。これは認識専用経路でも
   `ParseResult.scope().events()` に出る観測なので落とせない。
2. **`Session.wrap` の scoped site**。`spanRule = selectedChildExtent` の出現が
   `result.start()` / `result.end()` を出現の範囲に使う。`tree=false` では
   `scoped[i] && !skips[i]` の site だけが包まれる（D-035）。

どちらも **`@scope` 効果が 1 つも無ければ存在しない**（scope site が空なら
`JavaEmitter.bypassable()` が常に真になり、認識専用経路は `capture` の包みを
D-059 で丸ごと飛ばす）。`@mapping` の field 値を読むのは `Session.rule` の AST 経路だけである。
そこで生成側の条件はこの 1 行になる。

```java
private boolean plainRecognition() { return ir.semantics().scope().effects().isEmpty(); }
```

JSON / Operators / Power は効果 0 件なので `true`、P4（tinyexpression-p4）は
`enter` / `leave` / `declare` を持つので `false` が渡る。

### 1.3 健全性（なぜ観測が変わらないか）

- **`Match` の同一性を見る場所が無い**。`Session` の中で `Match` を identity 比較するのは
  `wrap` の `local == result.local()` と `rule` の `nodes != result.nodes()`、`project` の
  `values == result.values()` だけで、いずれも列の同一性であり `tree=true` の枝にある。
- **top level が読むのは null か否かだけ**。`Session.result()` は `match.captures()` /
  `match.recoveries()` / `match.traces()` を読むが、認識専用経路ではすべて空である
  （`OK` も空）。`match.start()` / `end()` / `nodes()` を読む `parseEntryAttempt` の枝は
  `options.buildAst()` の後ろにあり、`buildAst=true` は `tree=true` を含む。
- **memo に入れても安全**。payload に入るのは `OK` 1 個で、`consumedEnd` / `matchedEnd` は
  memo 表が別に持つ（`Match` からは読まない）。
- **`@recover` を持つ文法は対象外**。`twoMode()` が false を返すので `tree=true` になり、
  `plain` が立たない。

**参照経路への戻し方**: `-Dubnfc.plainmatch=false`（`Session.plainMatch = false`）。

---

## 2. 計測

共有機（AMD Ryzen 9 7950X / WSL2）。`taskset -c 28`（A/B は 29 も使用）、JVM は `-Xms2g -Xmx2g`、
暖機 200 反復。**load average 11〜25 の中で測っているので最小値を主に読む**（中央値も併記）。

### 2.1 1 parse の `Match` 生成回数（[`examples/json/evidence/ops-json-55-java.csv`](../../examples/json/evidence/ops-json-55-java.csv)）

`Match` の正準コンストラクタと `Match.empty` に静的カウンタを入れた複製を別ディレクトリで組み、
同一 JVM の中で `Session.plainMatch` を反転して 1 parse ずつ数えた（計時には使っていない）。

| 文法 / 入力 | mode | `Match` 生成 before → **after** | 割当 B before → **after** |
|---|---|---:|---:|
| json-1k (1,067 B) | 認識 | 2,840 → **0** | 165,776 → **6,608** |
| json-10k (10,066 B) | 認識 | 28,678 → **0** | 1,642,496 → **36,528** |
| json-100k (100,039 B) | 認識 | 285,775 → **0** | 16,338,304 → **334,904** |
| **json-1m (1,000,112 B)** | 認識 | **2,857,582 → 0** | **163,343,888 → 3,319,296** |
| json-1m | typed AST | 3,495,665 → 3,495,665 | 350,864,360 → 350,851,856 |
| operators-512 (1,023 B) | 認識 | 5,632 → **0** | 332,832 → **17,440** |
| operators-512 | typed AST | 8,192 → 8,192 | 1,473,424 → 1,473,424 |
| power-64 (127 B) | 認識 | 510 → **0** | 32,440 → **3,880** |
| p4 complex (332 B) | 認識 | 658 → 658 | 73,840 → 73,840 |
| p4 complex-x64 (20,923 B) | 認識 | 39,844 → 39,844 | 4,127,616 → 4,127,664 |

`Match.empty` だけを数えると json-1m は **2,533,873 → 0** で、issue が引いた数と一致する。
**残る 3.3 MB**（3.3 B/byte）は主に D-066 の `eligible` 配列（`new boolean[]{…}` 24 B、
呼出しごと）で、`Match` 由来の割当は 0 になった。AST 経路は `Match` 回数が完全に一致する
（割当の数千 B の揺れは同一 JVM で後に測った側が暖まっているぶんの雑音）。

### 2.2 速度（JSON 認識、同一 JVM 交互 A/B）

同一 JVM の中で `Session.plainMatch` を 1 run ごとに反転して交互に測った。暖機も両変種を
交互に 200 反復回して JIT に両方の分岐を見せている（#49 と同じ規約）。

**session 1（9 run × 0.5 秒、load 15.7 → 16.9）— 中央値 ms / 最小値 ms**

| 入力 | before | **after** | 比（最小） |
|---|---:|---:|---:|
| json-1k | 0.0933 / 0.0631 | **0.0723 / 0.0567** | **0.899** |
| json-10k | 0.7285 / 0.5479 | **0.4791 / 0.4383** | **0.800** |
| json-100k | 6.4198 / 5.2571 | **5.0679 / 4.2001** | **0.799** |
| json-1m | 59.522 / 51.103 | **54.022 / 41.734** | **0.817** |

**session 2（11 run × 1.0 秒、load 15.7）**

| 入力 | before | **after** | 比（最小） |
|---|---:|---:|---:|
| json-1k | 0.0585 / 0.0507 | **0.0491 / 0.0406** | **0.800** |
| json-10k | 0.6214 / 0.5551 | **0.4724 / 0.4140** | **0.746** |
| json-100k | 5.7484 / 5.1080 | **4.7307 / 3.9940** | **0.782** |
| json-1m | 57.342 / 50.523 | **49.906 / 39.480** | **0.782** |

**µs/byte（session 2 の最小値）とサイズ耐性**

| 変種 | 1 KB | 10 KB | 100 KB | 1 MB | 1 MB / 1 KB |
|---|---:|---:|---:|---:|---:|
| before | 0.0475 | 0.0552 | 0.0511 | 0.0505 | **1.06** |
| **after** | **0.0381** | **0.0411** | **0.0399** | **0.0395** | **1.04** |

**サイズに依らず一律 0.75〜0.90 倍**である。1 KB でも効くのは、割当が入力長に比例する
（＝1 parse の中で GC に渡る若い世代の量がそのまま減る）ためで、サイズで平坦なのは
D-066 の時点と変わらない。

### 2.3 ANTLR4 / ubnfc Rust / ubnfc Java の絶対値（JSON、同一走行で入力ごとに交互）

`benchmarks/scripts/run-external.sh json`（`CORE=28 RUNS=9 RUN_SECONDS=0.5`）。
**最小値**（load 11.4 の静かな走行）。

| 入力 | ANTLR4 | ubnfc Rust | **ubnfc Java** | Java / Rust | Java / ANTLR |
|---|---:|---:|---:|---:|---:|
| json-1k | 0.0153 µs/B | 0.0461 | 0.1733 | 3.76 | 11.3 |
| json-10k | 0.0152 | 0.0513 | **0.0570** | 1.11 | 3.75 |
| json-100k | 0.0151 | 0.0499 | **0.0417** | **0.84** | 2.76 |
| **json-1m** | **0.0104** | **0.0331** | **0.0385** | **1.16** | **3.70** |

**json-1m は Rust の 1.16 倍で目標（1.2 倍以内 = 0.042 µs/byte 以下）を満たす**（#49 は 1.49 倍）。
100 KB では Java が Rust より速い。**1 KB の Java は読まない**: `run-external.sh` は冷えた
1 反復から反復数を決めるので、負荷の高い瞬間に当たると小さい入力で反復数が極端に小さくなる
（この走行は 155 反復。同じ入力を §2.2 の暖機付き A/B で測ると 0.0381 µs/B）。
#49 と同じ既知の雑音である。

割当（同じ走行）: ANTLR4 27,067,376 B / ubnfc Rust 76,557,137 B（arena）/ **ubnfc Java 3,319,296 B**。
**Java が 3 実装で最も少ない**。

#### 2.3.1 master（#63 / #64 / #70）merge 後の最終 build

同じ手順を最終 build（origin/master 103842e を merge）で 2 周取り直した。load 8〜15 だが
core 28 に他の process が載っており、Java の 1 run 内の max/min が 2〜3 倍開く走行である。
**round 2 の最小値**（round 1 は Java 1 MB が 0.195 µs/B と明らかに乱れたので読まない）:

| 入力 | ANTLR4 | ubnfc Rust（D-072 入り） | **ubnfc Java** | Java / Rust |
|---|---:|---:|---:|---:|
| json-10k | 0.0138 | 0.0164 | **0.0459** | 2.80 |
| json-100k | 0.0148 | 0.0169 | **0.0483** | 2.86 |
| json-1m | 0.0451（乱れ） | 0.0137 | **0.0397** | 2.90 |

**Java の値は merge 前と同じ帯**（1 MB 0.0385 → 0.0397）で、動いたのは Rust の側
（0.0331 → 0.0137、#64 の報告どおり）。Java に残る 2.9 倍は、Rust が D-072 で
「観測を要求しない終端領域（JSON の string / number / 空白）を式ごとに歩かず 1 本の走査で読む」
ようになった差で、`Match` の割当ではない（本 PR 後の Java は `Match` を 1 個も作らない）。
Java 版の bulk scan は #69。

同じ最終 build の同一 JVM 交互 A/B（[`speed-json-55-java.csv`](../../examples/json/evidence/speed-json-55-java.csv)
の `final-ab-*`）も向きは同じで、最小値の比は json-1m 0.736 / 0.746、json-100k 0.771 / 0.835、
json-1k 0.729 / 0.829（json-10k は session 1 が 0.51 と乱れ、session 2 が 0.89）。

### 2.4 Operators / Power / P4

**Operators・Power（同一 JVM 交互 A/B、11 run × 0.5 秒、`taskset -c 29`、load 20〜24）**

| fixture | mode | before 中央値 / 最小 | **after 中央値 / 最小** | 割当 before → after |
|---|---|---:|---:|---:|
| operators 512 atom (1,023 B) | 認識 | 0.1242 / 0.1012 | **0.1034 / 0.0873** | 332,680 → **17,288** |
| operators 512 atom | AST | 1.1786 / 1.0145 | 1.1839 / 0.8623 | 1,452,944 → 1,452,944 |
| power-64 (127 B) | 認識 | 0.01452 / 0.01248 | **0.01235 / 0.01055** | 32,320 → **3,760** |
| power-64 | AST | 0.6035 / 0.4400 | 0.5418 / 0.3882 | 154,472 → 154,472 |

認識は最小値で 0.845〜0.863 倍。**AST は割当が 1 byte も動かない**（`plain` が立たない）ので、
時間差は雑音（同じ cell の max/min が 2 倍以上開く走行）。

**P4（`org.ubnfc.p4.Benchmark`、`bench.runs=9`、load 20.8）**。P4 は `@scope` 効果を持つので
生成側が `plainRecognition=false` を渡す＝**両変種が同じ経路**である。非退行の確認として測った。

| fixture | mode | 中央値 / 最小 ms | 割当 B | #49（`2026-09-24-java-d066.md` §2.4） |
|---|---|---:|---:|---|
| complex (332 B) | recognize-fast | 0.0938 / 0.0754 | 136,104 | 0.0869 / 0.0796、136,192 |
| complex | ast-fast | 0.1838 / 0.1566 | 263,504 | 0.2027 / 0.1686、263,592 |
| complex-x64 (20,923 B) | recognize-fast | 7.181 / 4.876 | 3,944,808 | 7.232 / 4.761、3,944,896 |
| complex-x64 | ast-fast | 14.645 / 12.593 | 11,882,552 | 17.527 / 13.923、11,882,640 |

最終 build（master merge 後、load 12〜15、core 28 に他 process）でも割当は complex 認識 136,216 B /
x64 認識 3,944,920 B で同じ（時間は complex 認識 最小 0.123 ms と走行の雑音で遅く出た。
`plain` は P4 では立たないので、差は機械の状態である）。Operators / Power の最終 build の A/B は
認識の最小値比 0.915（operators）/ 0.729（power）、割当は 332,704 → 17,312 / 32,344 → 3,784 B
（[`speed-match55.csv`](../../examples/p4-java/evidence/speed-match55.csv) の `final-ab`）。

---

## 3. 申し送り — P4（`@scope` を持つ文法）の残り

§2.1 のとおり P4 の認識は `Match` を **complex 658 個 / x64 39,844 個**作り続ける
（うち `Match.empty` は 603 / 36,576）。1 parse の割当に占める割合は
**complex 42 KB / 74 KB ≈ 57%、x64 2.55 MB / 4.13 MB ≈ 62%** で、
**x64 認識は 197 B/byte**（D-075 後の JSON は 3.3 B/byte）である。

**外せない理由は §1.2 の 2 か所だけ**で、どちらも「区間を読む側」が静的に分かっている。
畳むには「区間が要る呼出しの間だけ `Match` を作る」動的な印（`Session.rule` が
`scopeEffects.length > 0` のとき、`Session.capture` が scoped site を持つとき、
左因数分解の群が scoped な `Cap` を含むときに立て、memo 鍵の `mode` に 1 bit 足す）が要る。
`Match` は連接・反復・capture・左因数分解の全経路の返り値型なので、
**1 PR 1 意味単位の原則で別 issue にする**（本 PR は「区間が読まれないと証明できる文法」だけ）。

もう 1 つの残り: D-075 後に json-1m 認識で残る **3.3 MB / parse** は、D-066 の predict 選択が
呼出しごとに確保する `new boolean[]{…}`（24 B）が主である。候補数が 64 以下なら `long` の
bit mask に畳めるので、`Match` の次はここになる。

---

## 4. 検証

- **新設 `PlainMatchEquivalenceTest` 748 件**（`ubnfc-java`）。`@scope` 効果を持たない
  operators / power を `CompilerHarness` で生成・コンパイルし、handwritten-target の
  corpus 入力・その全接頭辞・各位置の 1 文字破壊について、`Session.plainMatch` の on / off で
  受理・cursor・主診断・nativeFailure・出現表・scope・canonical JSON が一致することを固定する
  （認識専用 options と `withDiagnostics()` の両方）。AST 経路が影響を受けないことと、
  生成側の条件（operators は `plainRecognition=true`、P4 は `false`）も同じテストで固定する。
- `examples/p4-java`: `TwoModeEquivalenceTest` / `PredictEquivalenceTest` /
  `FactorEquivalenceTest` / `AstBypassEquivalenceTest` / `ExclusiveEquivalenceTest` が緑
  （P4 は参照経路のままなので、これらは本 PR の非退行の確認）。
- canonical JSON golden **13 件**が無変更で緑（`cargo test --workspace` の中）。
- corpus: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java` が
  **Java Pass 1,310 / Fail 0**。系統表は [`2026-09-24-java-corpus-d066-49.md`](2026-09-24-java-corpus-d066-49.md)
  と **byte 一致**（差分は先頭の generator / front の SHA-256 行だけ）。
  出力は [`2026-09-24-java-corpus-match55.md`](2026-09-24-java-corpus-match55.md) / `.json`。
- **JSONTestSuite 318 件の判定が不変**: merge 前の build では `benchmarks/scripts/check-jsontestsuite.sh` の
  `ubnfc-java` 列が committed の
  [`examples/json/evidence/jsontestsuite-verdicts.tsv`](../../examples/json/evidence/jsontestsuite-verdicts.tsv)
  と 318 件すべて一致（y_ 95/95・n_ 188/188）。最終 build では `i_structure_500_nested_arrays.json`
  （実装依存）だけが reject → accept に変わるが、これは PR #63（#35、深い JSON の入れ子を受理する）の
  意図した変化で、同じ最終 build で `-Dubnfc.plainmatch=true / false` を切り替えた 318 件の判定は
  完全に一致する（本 PR 起因の差は 0）。
- **IR golden 不変**: front も Rust backend も触っていない（`crates` / `ir` / `corpus` は
  merge した `origin/master` と byte 一致）。
- 検証は origin/master 103842e（#63 / #64 / #70）を merge した最終 tree で全部取り直した
  （corpus の系統表は #64 の [`2026-09-24-java-corpus-bulkscan.md`](2026-09-24-java-corpus-bulkscan.md) とも byte 一致）。
- 差分ファジング: `cargo test -p ubnfc-fuzz --offline` の固定 seed campaign が緑。
- `cargo test --workspace --offline --no-fail-fast` / `cargo clippy --workspace --all-targets -- -D warnings` /
  `cargo fmt --check` / `cd ubnfc-java && mvn -o -q test` / `cd examples/p4-java && mvn -o -q test` /
  `examples/p4-java/scripts/generate.sh --check` が緑。

## 5. 再現

```bash
cd ubnfc-java && mvn -o -q -DskipTests package && mvn -o -q test
bash examples/p4-java/scripts/generate.sh --check
cd examples/p4-java && mvn -o -q -DskipTests package && mvn -o -q test
# 3 実装の突き合わせと A/B（AbRunner / CountRunner は本報告の scratch driver）
WORK=$(mktemp -d) benchmarks/scripts/prepare-external.sh
WORK=$WORK CORE=28 RUNS=9 benchmarks/scripts/run-external.sh json
taskset -c 28 java -Xms2g -Xmx2g -cp "$WORK/ubnfc/java/classes" AbRunner json 認識 11 1.0 "$WORK"/inputs/json-*.json
# 別 JVM で参照経路へ戻したいとき
taskset -c 28 java -Xms2g -Xmx2g -Dubnfc.plainmatch=false -cp "$WORK/ubnfc/java/classes" \
  UbnfcRunner bench json 認識 9 0.5 "$WORK/inputs/json-1m.json"
```

## 6. 生データ

| もの | 場所 |
|---|---|
| 1 parse の `Match` 生成回数と割当（JSON / Operators / Power / P4、before / after） | [`examples/json/evidence/ops-json-55-java.csv`](../../examples/json/evidence/ops-json-55-java.csv) |
| 速度（JSON の A/B と 3 実装の突き合わせ。merge 前 / 最終 build の各 2 回） | [`examples/json/evidence/speed-json-55-java.csv`](../../examples/json/evidence/speed-json-55-java.csv) |
| 速度（Operators / Power の A/B と P4 の非退行） | [`examples/p4-java/evidence/speed-match55.csv`](../../examples/p4-java/evidence/speed-match55.csv) |
| corpus | [`2026-09-24-java-corpus-match55.md`](2026-09-24-java-corpus-match55.md) / `.json` |
| 入力の sha256（PR #32 / #46 / #54 と同一） | [`examples/json/evidence/inputs.sha256`](../../examples/json/evidence/inputs.sha256) |
