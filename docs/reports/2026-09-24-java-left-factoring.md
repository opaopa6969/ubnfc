# Java の左因数分解と AST 経路の再計測（2026-09-24、issue #30）

本ラウンドは 2 つ。**A** は PR #29（D-059、認識が半減）の後の master で **AST 経路を測り直し**、M5 の
AST 目標（P4 complex ≤0.2 ms / x64 ≤15 ms）への距離を今の数字で確定する。**B** は Rust backend の
**左因数分解**（`crates/ubnfc-rust/src/factor.rs`）の**設計**を Java backend へ移し、PR #29 が残した
唯一の回数差（rule 関数呼出し Java 1,076 対 Rust 759）を詰める。**`crates/` は 1 行も触っていない**。

## 結論（先に）

- **A**: **M5 の P4 AST 目標は未達**。ast-fast（AST を作り観測を要求しない）は **complex 0.304 ms（目標の 1.52 倍）/
  x64 24.20 ms（1.61 倍）**。一方 **Operators 512 atom の AST は 0.537 ms で issue #19 の目標 0.693 ms を達成**し、
  同一プロセスで測った手書き目標形 0.503 ms の **1.07 倍**まで来た（割当は 1,445 KB 対 2,785 KB）。
- **A の主因**: PR #29 の最大の一手（**D-059 = capture の包みの迂回**）は `tree=false` の認識専用経路にしか効かない。
  AST 経路は 1 parse あたり **2,276 回**の capture の包みを今も払っている（認識専用は 13 回）。
  そのため「AST 層の費用」は #19 の 0.089 ms から **0.175 ms（complex）**に増えて見える（§1.3）。
- **B**: 左因数分解を Java backend へ入れた（**D-061**）。P4 complex 1 parse の
  **rule 関数呼出しは 1,076 → 782、memo hit は 464 → 170** で、**Rust の 759 / 169 にほぼ並んだ**（§2.4）。
  時間は **complex 認識 0.1277 → 0.1008 ms（最小 0.1067 → 0.0813）、complex-x64 認識 9.11 → 6.04 ms
  （最小 8.16 → 5.41）、complex AST 0.390 → 0.246 ms、x64 AST 23.9 → 18.2 ms**（binary 交互 3 周、§2.6）。
  **issue の見積もり 7 µs（6%）より大きい**のは、消えるのが memo 参照だけでなく深さ 5〜6 の段全体だから（§2.6）。
  残差 23 は左因数分解では説明できない（Rust も因数分解を切ると 983 / 393 になるので、
  **元の差 317 のうち左因数分解で説明できるのは 224 だけ**だった。§2.5）。
- **B の契約選択**: 適用は**診断を記録しない経路**（`diag=false`）と **lexical trace を要求しない経路**に限る。
  D-023 の二段構えが「失敗・末尾入力は必ず診断モードで再解析」を保証するので、`diag=false` に限れば
  Rust が要した「呼ばれない wrapper の label 再生」が**そもそも要らない**（D-061）。
- 観測は動いていない: 新設の **`FactorEquivalenceTest` 10,608 件**（因数分解経路 対 参照経路）と
  `TwoModeEquivalenceTest` 2,652 件 / `PredictEquivalenceTest` 5,304 件、canonical JSON golden 13 件、
  corpus Java Pass 1,310 / Fail 0（系統別表は byte 一致）、IR golden 不変。

## 0. 計測環境

| 項目 | 値 |
|---|---|
| 基準 | `origin/master` `c9f102a`（PR #29 = D-059 を取り込んだ後） |
| host | AMD Ryzen 9 7950X（32 論理 CPU）、86 GB、WSL2 Linux 6.18.33.2 |
| Java | Oracle JDK 21.0.9 / HotSpot、`-Xms512m -Xmx2g -Xss16m`、`taskset -c 28`、単一 thread |
| Rust | `rustc` stable、`--release --offline`、`taskset -c 29`（回数だけ。時間は使っていない） |
| 負荷 | **共有機で別エージェントが並走**。各表に採取時の load average を書く。**一次証拠は §2.4 の計数と割当 byte**（負荷に依存しない） |
| 集計 | `bench.runs=7`。中央値と**全 run の最小値**を併記する |
| mode | `ast`（観測を全て要求）/ `ast-fast`（AST を作り観測を要求しない）/ `recognize-fast`（AST も作らない） |

---

## 1. Part A — AST 経路の再計測

### 1.1 P4（`examples/p4-java`、同一プロセス交互、`bench.warmup=2000`、load 9.2〜14.5）

| fixture | mode | 中央値 ms | 最小 ms | 割当 B/parse |
|---|---|---:|---:|---:|
| complex (332 B) | ast | 0.5820 | 0.5103 | 498,576 |
| complex | **ast-fast** | **0.3043** | 0.2789 | 284,568 |
| complex | recognize-fast | 0.1293 | 0.1157 | 139,760 |
| complex-x4 | ast | 2.1375 | 1.7138 | 1,904,464 |
| complex-x4 | **ast-fast** | **1.1355** | 0.9688 | 1,093,552 |
| complex-x4 | recognize-fast | 0.4757 | 0.4295 | 528,824 |
| complex-x16 | ast | 8.3461 | 7.7814 | 6,483,416 |
| complex-x16 | **ast-fast** | **4.6441** | 4.0312 | 3,289,760 |
| complex-x16 | recognize-fast | 1.6326 | 1.5792 | 1,049,472 |
| complex-x64 (20,923 B) | ast | 38.199 | 35.335 | 25,838,936 |
| complex-x64 | **ast-fast** | **24.201** | 21.814 | 13,116,240 |
| complex-x64 | recognize-fast | 9.733 | 8.841 | 4,173,752 |

### 1.2 family B（Operators / Power、生成と手書き目標形を同一プロセスで。`bench.iterations=400`、load 7.1〜10.8）

| 文法 / 入力 | 実装・mode | 中央値 ms | 最小 ms | 割当 B/parse |
|---|---|---:|---:|---:|
| Operators **512 atom (1,023 B)** | 生成・ast | 0.8183 | 0.7968 | 1,860,992 |
| Operators 512 atom | **生成・ast-fast** | **0.5371** | 0.5303 | **1,444,560** |
| Operators 512 atom | 生成・recognize-fast | 0.0930 | 0.0916 | 345,008 |
| Operators 512 atom | **手書き目標形・ast** | **0.5032** | 0.4914 | 2,785,200 |
| Operators 括弧深さ 8 | 生成・ast / ast-fast | 0.0195 / 0.0123 | | 39,712 / 26,568 |
| Operators 括弧深さ 8 | 手書き目標形・ast | 0.0138 | | 98,376 |
| Power 64 段 (127 B) | 生成・ast / ast-fast | 0.0800 / 0.0466 | | 327,152 / 156,536 |
| Power 64 段 | 手書き目標形・ast | 0.0284 | | 197,888 |

### 1.3 目標に対する到達（§7.1 に記録する行）

| 目標 | #19 の値 | **本測定** | 判定 |
|---|---|---|---|
| P4 complex AST ≤ 0.2 ms | 0.309 | **0.3043**（最小 0.2789） | **未達（1.52 倍）** |
| P4 complex-x64 AST ≤ 15 ms | 27.98 | **24.20**（最小 21.81） | **未達（1.61 倍）** |
| Operators 512 atom AST ≤ 0.693 ms | 0.631 | **0.537**（手書き 0.503 の 1.07 倍） | **達成** |
| AST の割当が手書き目標形以下 | 1,444 KB | 1,445 KB 対 2,785 KB | 達成 |

**#22 で認識が半減したのに AST が減っていないのはなぜか。** 同じ走行の認識専用は complex 0.129 ms /
x64 9.73 ms なので、差し引きの「AST 層の費用」は **complex 0.175 ms / x64 14.5 ms** になる。
#19 の同じ引き算は 0.089 / 10.5 ms だった。増えて見える主因は **D-059（capture の包みの迂回）が
`tree=false` にしか効かない**ことで、計数で見ると 1 parse の `Session.capture` 呼出しは
**ast-fast 2,276 回 / recognize-fast 13 回**（§2.4 の計数器で同時に採った）。#22 の単価表（7.6 ns）で
17 µs ぶんがこの差に乗っている。残りは負荷と、complex と x64 を同じ JVM に載せたことによる
memo 表 pool の取り合い（#19 §4.2 と同じ効果）。

**結論**: P4 の AST 目標に届かせるには「AST の 3 段（`Match` → `Recipe` → typed AST）を畳む」だけでなく、
**`tree=true` の経路にも capture の包みを通さない形**が要る（D-059 を `tree` 経路へ広げる = 出現を
包みの外で作る）。どちらも生成側の構造変更なので本 PR の範囲外で、#19 §6 の「残り 2」のまま。

---

## 2. Part B — 左因数分解を Java backend へ

### 2.1 何を移したか

Rust の `factor.rs` は「連続する候補 Seq が構造的に等しい先頭要素列を持つ」群を見つけ、共有接頭辞を
1 回だけ評価して続きを宣言順に試す trie を**生成コードとして展開**する。Java 版は同じ**判定**を
`ubnfc-java/src/main/java/org/ubnfc/java/emit/Factoring.java` に置き（`element_shape` / `seq_shape` /
`share` / `plan` / `level` と同じ条件）、**実行は `Session` に置いた**（`Session.Group` / `Level` /
`Branch` / `Cap`）。生成コードは定数の trie を 1 つ渡すだけで、`s.choice(f, …, guards, groups)` が
群に当たった候補を `factored()` へ回す。Java の `Session` は連接・選択・capture の意味論を
すでに 1 か所に持っているので、Rust のように展開するより**同じ意味論を 1 か所で使い回す方が差が出にくい**。

対象外は Rust と同じ: 最長一致・予測選択、Seq でない候補、先頭要素が RuleRef / literal / token 以外、
execution・trivia policy・capture 規則（span 規則・skip・scope 効果が読むか）・label・意味形が一致しない、
要素や候補 Seq が memo される、`@recovery` の適用先、群内で候補除外（`Prediction`）の条件が一致しない。
P4 では **2 つの選択（`SliceBaseReceiver` / `SliceBaseExpression` の本体、各 8 候補）**が群になり、
それぞれ深さ 5〜6 まで接頭辞を共有する（Rust の生成物と同じ 2 か所）。

### 2.2 同値の根拠（Java 版）

PEG では memo が「同じ状態の X は同じ結果を返す」ことを保証するので `X A | X B ≡ X (A | B)`。
残るのは**観測**で、Java では 3 つに分かれる。

1. **診断**（主診断 DAG・display frame）。Rust は「呼ばれない wrapper が失敗時に記録したはずの
   display label を同じ位置へ再生する」ことで同値にする。Java は **D-023 の二段構え**により
   `diag=false` の解析が失敗・末尾入力になれば**必ず同じ入力を診断モードで再解析する**ので、
   **`diag=false` に限れば再生そのものが要らない**。そこで `Session.factor = factoring && !diag && !options.lexical()`
   とし、診断を記録する解析は参照経路のままにした（**D-061 の契約選択**）。
   成功した解析が公開する hints は D-023 副作用 (a) のとおり元から空。
2. **capture 出現 / 意味値**。共有要素は群の先頭候補の式で評価するので、採用候補が別のときは
   出現の site / name を採用候補のものへ**付け替える**（`Session.retag`、Rust の `Session::retag` と同じ）。
   出現の実体は評価のたびに新しく作られる（`rule()` は tree 経路でも局所出現を空にして返すので、
   共有要素の包みが作る 1 件だけが `local` の末尾に乗る）ので、付け替えは他から参照されない 1 件に効く。
   要素が持てる capture site は 1 つまで（Rust と同じ制約）。候補 Seq 側の Values / capture / 出口 trivia は
   採用候補の `Cap` で作る。
3. **lexical trace**。共有要素の `Trace(rule, expr, …)` も付け替えが要るが、`options.lexical()` は
   `tree=true` を含む重い経路なので**対象外**にした（参照経路のまま）。
4. **scope 効果**。共有接頭辞の効果は 1 回だけ記録され、各候補は接頭辞後の印へ巻き戻して続きを試す。
   参照経路（候補ごとに巻き戻して X を再評価 = memo hit で効果を再生）と最終的な効果列は同じ。

### 2.3 入れたもの（差分の要約）

| 場所 | 変更 |
|---|---|
| `emit/Factoring.java`（新規） | 群の計画。Rust `factor.rs` の判定を移植 |
| `emit/JavaEmitter.java` | `choice` が群を見つけたら `Group[]` 定数を出し、7 引数の `s.choice` を呼ぶ。`capArgs` / `bypassable` を切り出して `Cap` と共有 |
| `internal/Session.java`（template） | `Group` / `Level` / `Branch` / `Cap`、`factored` / `level` / `tail` / `retag`、群を持つ `choice` の overload。`capture()` を「評価する側」と「包む側（`wrap`）」に分け、因数分解経路は `wrap` を直に使う |
| `internal/Session.java` | `Occurrence.site/name` を付け替え可能に（`retag`）。`Session.factor` / `Session.factoring`（`-Dubnfc.factor=false` で参照経路） |
| `examples/p4-java` | 生成物の再生成と `FactorEquivalenceTest`（新規） |

### 2.4 回数（1 parse、complex 332 byte）

生成物の複製に計数を挿した一時 binary（`mktemp -d` 配下。commit には含めない）。**負荷に依存しない一次証拠**。

| 計数 | Java `origin/master` | **Java 本 PR** | Rust（factor 有） | Rust（factor 無） |
|---|---:|---:|---:|---:|
| rule 関数呼出し | 1,076 | **782** | 759 | 983 |
| うち memo hit | 464 | **170** | 169 | 393 |
| うち本体を評価 | 612 | 612 | 586 | 586 |
| 群の実行（`factored`） | — | 42 | — | — |
| `Session.capture`（迂回しなかった包み、recognize-fast） | — | 13 | — | — |
| `Session.capture`（同、ast-fast） | — | 2,276 | — | — |

**rule 関数呼出しは 1,076 → 782、memo hit は 464 → 170** で、Rust の 759 / 169 に並んだ。

complex-x64（20,923 byte、認識専用）も同じ形:

| 計数 | Java `origin/master` | **Java 本 PR** | Rust |
|---|---:|---:|---:|
| rule 関数呼出し | 65,210 | **47,276** | 46,119 |
| うち memo hit | 28,499 | **10,565** | 10,564 |
| うち本体を評価 | 36,711 | 36,711 | 35,362 |
| 群の実行（`factored`） | — | 2,562 | — |

**memo hit は x64 で 10,565 対 10,564 とほぼ完全に一致する**（rule 呼出しの残差 1,157 = 2.5% は §2.5 の別要因）。

### 2.5 残差 23 の説明（issue の前提の訂正）

issue #30 は「差 317 はすべて左因数分解」と書いたが、**Rust 側で因数分解を切って測ると 983 / 393** で、
**左因数分解が説明するのは 224 回**しかない。残り 93 回は別の要因（Java と Rust で choice の候補を
実際に呼ぶ範囲が違う場面）で、本 PR の因数分解はそのうち 294 回を消して **782** まで来た
（Rust より 23 多い）。**目標「≈759」に対して 23 回（3%）残る**が、これは
「Rust だけが群にできて Java が群にできない候補列」ではなく、上の 93 回の残りである。
1 回あたり 23.2 ns（#22 §2 の memo 探索の単価）として **0.5 µs** なので、追う価値は小さい。

### 2.6 A/B

**2 通りで測った**。(i) **同じ binary で因数分解だけ切る**（`-Dubnfc.factor=false` 対 既定。
生成物も runtime も同一なので、差は因数分解だけ）、(ii) **binary 交互**（`origin/master` の
生成物 対 本 PR の生成物。本 PR の refactor 全体を含む）。どちらも **1 round ごとに交互**に走らせ、
「中央値」= round 中央値の中央値、「最小」= 各 round の最小値の中央値。`bench.runs=7`、`bench.warmup=1000`。

#### (i) 因数分解の単独効果（同一 binary、3 周、load 7.6〜10.1）

| fixture / mode | off 中央値 | on 中央値 | 比 | off 最小 | on 最小 | 比 | off 割当 | on 割当 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| complex / recognize-fast | 0.1609 | **0.1019** | **0.63** | 0.1094 | **0.0803** | **0.73** | 138,328 | 137,272 |
| complex / ast-fast | 0.3041 | **0.2408** | **0.79** | 0.2687 | **0.2135** | **0.79** | 282,584 | 265,200 |
| complex-x64 / recognize-fast | 8.944 | **6.750** | **0.75** | 7.751 | **5.950** | **0.77** | 4,091,680 | 4,014,016 |
| complex-x64 / ast-fast | 24.145 | **18.334** | **0.76** | 20.607 | **17.726** | **0.86** | 13,000,352 | 11,985,552 |

**3 周とも、4 cell すべてで同じ向き**（因数分解ありが速い）。round ごとの中央値 / 最小は
complex 認識が off 0.1699/0.1094・0.1150/0.1051・0.1609/0.1133 に対し on 0.0844/0.0778・0.1019/0.0803・0.1138/0.0854。

#### (ii) `origin/master` 対 本 PR（binary 交互、3 周、load 7.8〜15.6）

| fixture / mode | base 中央値 | 本 PR 中央値 | 比 | base 最小 | 本 PR 最小 | 比 | base 割当 | 本 PR 割当 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| complex / recognize-fast | 0.1277 | **0.1008** | **0.79** | 0.1067 | **0.0813** | **0.76** | 138,384 | 137,296 |
| complex / ast-fast | 0.3899 | **0.2464** | **0.63** | 0.3117 | **0.2192** | **0.70** | 282,664 | 265,464 |
| complex-x64 / recognize-fast | 9.109 | **6.040** | **0.66** | 8.163 | **5.414** | **0.66** | 4,091,736 | 4,014,040 |
| complex-x64 / ast-fast | 23.926 | **18.208** | **0.76** | 22.248 | **16.709** | **0.75** | 13,000,432 | 12,002,448 |

**参照経路（`-Dubnfc.factor=false`）が base より遅くなっていないこと**も (i) の off 列と (ii) の base 列の
一致で確かめられる（complex 認識 0.1609 対 0.1277、x64 認識 8.94 対 9.11 で、負荷差の範囲）。

#### 見積もりとの差

issue #30 の見積もりは「memo 参照 295 回 × 23.2 ns = 約 7 µs（当時の 0.11 ms の 6%）」だったが、
**実測は complex 認識で 29 µs（最小値、0.1094 → 0.0803）**と 4 倍大きい。理由は計数で説明できる:
消えるのは memo 参照だけではない。P4 の 2 群は**深さ 5〜6 まで接頭辞を共有する 8 候補**なので、
参照経路は候補ごとに
「候補 Seq の `capture` 包み → `sequence` の印・cursor 退避・入口 trivia → 共有要素 5〜6 個の
`capture` 包みと `rule` 呼出し（memo hit）→ 各要素後の trivia」を丸ごと繰り返す。
因数分解はこの**段全体**を 1 回に畳むので、rule 呼出し 294 回のほかに trivia・印・巻戻し・
interface 呼出しがまとめて消える（群の実行は complex で 42 回 / x64 で 2,562 回）。

**採否**: 計数が下がり（1,076 → 782）、時間も 2 通りの A/B で 3 周とも同じ向きに下がり、
割当も減った（complex ast-fast 282 → 265 KB、x64 ast-fast 13.00 → 12.00 MB）ので**採用**。

---

## 3. 同値性の検証

- **新設 `FactorEquivalenceTest` 10,608 件**（884 入力 × 入口 3 種 × `buildAst` 2 × 出現表要求 2）が緑。
  `Session.factoring` を true / false に切り替えて、受理・cursor・主診断・farthest・nativeFailure・
  **capture 出現表**・scope（宣言・参照・診断・イベント）・recovery・canonical JSON AST を突き合わせる。
  出現表を要求する行が `Session.retag` を固定する。
- `TwoModeEquivalenceTest` **2,652 件**・`PredictEquivalenceTest` **5,304 件**が緑。
- canonical JSON golden **13 件**と `java_and_rust_goldens_are_identical_without_normalization` が無変更で緑。
- corpus: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java`
  → **Java Pass 1,310 / 期待拒否 Pass 28 / Known 28 / Fail 0 / 未観測 59**
  （`docs/reports/2026-09-24-java-corpus-factor30.{md,json}`）。系統別の表は
  `docs/reports/2026-09-24-java-corpus-rec22.md` と **1 行も違わない**。
- **IR golden 不変**: `crates` / `ir` / `corpus` は `origin/master` と byte 一致（front も Rust backend も触っていない）。
- 差分ファジング: `cargo test -p ubnfc-fuzz --offline` の固定 seed キャンペーンが緑
  （新しい checkout では `cargo build --offline --bin ubnfc-rust` を先に走らせる）。
- `cargo test --workspace --offline --no-fail-fast` / `cargo clippy --workspace --all-targets -- -D warnings` /
  `cargo fmt --all --check` / `cd ubnfc-java && mvn -o test` / `cd examples/p4-java && mvn -o test` /
  `examples/p4-java/scripts/generate.sh --check`。

## 4. 再現

```bash
cd ubnfc-java && mvn -o -q -DskipTests package && mvn -o test
bash examples/p4-java/scripts/generate.sh --check
cd examples/p4-java && mvn -o -q -DskipTests package && mvn -o test
# Part A（同一プロセス交互）
taskset -c 28 java -Xms512m -Xmx2g -Xss16m \
  -Dbench.fixtures=complex,complex-x4,complex-x16,complex-x64 -Dbench.runs=7 -Dbench.warmup=2000 \
  -Dbench.modes=ast,ast-fast,recognize-fast -cp target/classes org.ubnfc.p4.Benchmark src/test/resources/fixtures
# Part B の A/B（同じ binary で因数分解だけ切る）
taskset -c 28 java -Xms512m -Xmx2g -Xss16m -Dubnfc.factor=false \
  -Dbench.fixtures=complex,complex-x64 -Dbench.runs=7 -Dbench.warmup=1000 \
  -Dbench.modes=recognize-fast,ast-fast -cp target/classes org.ubnfc.p4.Benchmark src/test/resources/fixtures
# Rust の回数（因数分解の有無）
taskset -c 29 cargo run -q --offline --release -p ubnfc-p4-rust --features counters --example bench -- --counters complex
UBNFC_BENCH_OPTS=factor=0 taskset -c 29 cargo run -q --offline --release -p ubnfc-p4-rust --features counters --example bench -- --counters complex
# corpus
cargo run -q --offline --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-24-java-corpus-factor30.md --json docs/reports/2026-09-24-java-corpus-factor30.json
```
