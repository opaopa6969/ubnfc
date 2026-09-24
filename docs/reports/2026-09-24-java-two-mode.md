# Java backend の認識専用経路（二段構え、2026-09-24、issue #15）

`docs/reports/2026-09-24-java-backend-gap.md`（PR #14）が「残り 2.0 倍の約 45% は、**認識モードでも
Match 木と capture 出現（512 atom で約 2,500 件）と Recipe を必ず作る**ことに由来する」と測った。
その残差を、Rust backend が既に持つ二段構え（D-023 / D-035 / D-036）の**設計**を Java へ移して潰す。
Rust のコードは移植していない（`const DIAG` の単相化に相当するものは Java には作らない。後述）。

## 結論（先に）

- **Operators 512 atom の認識は 0.778 → 0.207 ms**（3.76 倍）。手書き目標形 0.350 ms の **0.59 倍**で、
  **目標 ≤0.4 ms を達成**し、初めて手書きより速くなった。割当は 2,008 KB → 466 KB（手書き 2,095 KB の 0.22 倍）。
- **P4 complex-x64 の認識は 31.96 → 13.71 ms**（2.33 倍）。**目標 ≤15 ms を達成**。
- **P4 complex の認識は 0.483 → 0.213 ms**（2.27 倍）。**目標 0.2 ms に 6% 届かない**（残ギャップの内訳は §6）。
- **AST 経路は変えていない**ので、M5 の「complex ≤0.2 ms（AST）/ x64 ≤15 ms（AST）」は**未達のまま**
  （complex AST 0.538 ms、x64 AST 36.7 ms）。認識だけが速くなったことを正直に分けて書く。
- memo: **Operators は memo を持つ意味が無い**（認識 fast で `memo=false` が 0.81 倍・割当 0.75 倍）が、
  **P4 は memo が無いと 67 倍遅い**（0.219 → 14.69 ms）。文法ごとの判断が要るので front の memo plan の
  issue に回す（§7）。`crates/ubnfc-front` は本 PR では触っていない。

## 1. 入れたもの（認識経路が作らなくなったもの）

`ParseOptions` に観測の要求を 2 つ足した。どちらも既存の構築子では `true`（＝従来どおり全部作る）。

| 要求 | 既定 | 対応する Rust の決定 |
|---|---|---|
| `occurrences` | 既存構築子では true、`RECOGNIZE` では false | D-035（出現表は opt-in） |
| `diagnostics` | 既存構築子では true、`RECOGNIZE` では false | D-023（診断は opt-in、失敗時に決定的再解析） |

`Session` は構築時に 2 つの boolean を決める。`twoMode` は生成側（`JavaEmitter`）が文法ごとに出す定数。

```java
this.tree = !twoMode || options.buildAst() || options.lexical() || options.occurrences();
this.diag = !twoMode || options.diagnostics();
```

### `tree == false` のとき作らないもの

| 位置 | 従来 | 認識経路 |
|---|---|---|
| `rule()` の recipe 構築 | `Recipe` + field ごとの `List<Value>` + `Value` | 何も作らない（`Match.empty(c, f.c)` を返す） |
| `capture()` / `captured()` | capture 出現 `Occurrence` + `Value` を `local` と `captures` の両方へ append | scope 効果が読む site だけ `local` へ。公開する `captures` は作らない |
| `capture()` の `project()` | 意味値 `Value` の射影 | 呼ばない |
| `combine()` / `join()` | `nodes`/`local`/`captures`/`traces`/`recoveries`/`values` の 6 列を畳む | `local` の合計だけ数える。0 件なら `Match.empty` 1 個 |
| `sequence()` | `Match[children.length]` + `Arrays.asList` | 配列を作らず子を順に呼ぶ |
| `repeat()` / `separated()` | 周回ごとの `Match` を `ArrayList` に溜める（512 atom で 511 件） | 溜めない。`separated` の下限判定は件数カウンタ |
| `result()` | `match.captures()` を `Capture` へ写す（512 atom で約 2,500 件） | `captures` が空なので 0 件 |

### `diag == false` のとき記録しないもの

`Diagnostics`（frame 配列・expected stack・rule stack の親リンク・memo payload の `Snapshot`）への
呼出しを全て落とす。残すのは**主位置 `reached`（int 1 本）**だけ。具体的には `enterRule` / `leaveRule` /
`failAt` / `failBatch` / `reach` / `merge` / `checkpoint` / `restore` / `failureCursor` を呼ばない。
memo payload の `Diagnostics.Snapshot` も `null` になる。

Rust の D-036 は `Session<'a, const DIAG: bool>` で単相化するが、Java には値パラメタ多相が無いので
**`final` フィールドの分岐**にした。Session は 1 parse に 1 個で、`tree` / `diag` は構築後に変わらないので
JIT から見れば実質定数になる。生成物を 2 部にしない（Rust は binary +56%）ぶん、i-cache の代償も無い。

### 契約（D-023 と同じ二段構え）

生成される入口が次の 1 行を持つ。

```java
if (!syntax.ok() && !s.diag) return parseEntryInternal(grammar, entry, source, options.withDiagnostics(), rollback);
```

失敗・末尾入力（`ok == false`）なら、**同じ入力を診断モードでもう一度解析して**その結果を返す。解析は
決定的なので観測は「最初から要求して解析した」ものと同一。成功した解析の `hints`（`expected` /
`farthestExpected` / `nativeFailure`）だけが既定で空になる — これは D-023 が明記した副作用 (a) そのもの。
corpus の driver は従来どおり観測を全て要求する構築子を使うので、**corpus の観測は 1 行も動かない**。

### 二段構えを使わない文法（`twoMode == false`）

`JavaEmitter.twoMode()` が false を返すのは次の 2 つ。この文法では `RECOGNIZE` でも従来の完全経路を通る。

1. `@recover` を持つ文法 — `RecoveryEvent` は認識結果にも載り、marker `Recipe` と
   `capture_occurrence_ids` を通じて Match 木と出現表に依存する。
2. scope 効果が**反復下（repeat / optional / separated）の capture site** を読む文法 —
   出現表を落とすと scope 観測（宣言・参照・診断）が変わる。

P4 はどちらにも当たらない（scope 効果の site は全て `tokenRef` の capture）ので `twoMode = true`。
scope 効果が読む site は `capture()` へ渡す静的な `boolean[] scoped` で印を付け、認識経路でも
その出現だけは `local` に作る。だから **scope の観測は認識経路でも完全に一致する**（§5 の同値テスト）。

## 2. 計測環境

- AMD Ryzen 9 7950X（32 論理 CPU）、86 GB、WSL2 Linux 6.18.33.2。`taskset -c 28` で 1 コアに固定、単一 thread。
- Oracle JDK 21.0.9 / HotSpot、`-Xms512m -Xmx2g -Xss16m`。
- **load average は作業を通じて 3.4〜6.3**（PR #14 の 9〜44 より大幅に低い。他エージェントのベンチは走っていない）。
- A/B は**同一プロセス・同一コード・`ParseOptions` だけを差し替え**て行う。`bench.modes` で cell（入力 × mode）を
  作り、全 cell を warmup したあと **1 run ごとに全 cell を巡回する**ので、負荷変動は全 cell に同じように乗る。
  外部の binary 差し替えも system property による静的分岐も使っていない。
- 割当は `ThreadMXBean` の thread allocated bytes（計時 run と同じループ）。負荷に依存しない一次証拠。
- mode: `ast`（AST を作る）/ `recognize`（認識、観測を全て要求＝ PR #14 の振る舞い）/
  `recognize-notree`（step A: 出現表を要求しない）/ `recognize-fast`（step A+B: 観測を何も要求しない）。

## 3. A/B — family B（Operators / Power）

`bench.runs=7`、`bench.iterations=400`、warmup 20,000。load average 5.07 → 3.42。

| 文法 | 入力 | mode | 中央値 ms | 最小 ms | 割当 B/parse |
|---|---|---|---:|---:|---:|
| Operators | `1+2*3` (5 B) | ast | 0.00739 | 0.00695 | 22,720 |
| Operators | `1+2*3` | recognize | 0.00642 | 0.00617 | 20,696 |
| Operators | `1+2*3` | **recognize-notree** | 0.00426 | 0.00415 | 13,248 |
| Operators | `1+2*3` | **recognize-fast** | **0.00312** | 0.00272 | **10,816** |
| Operators | `1+2*3` | 手書き目標形・認識 | 0.00176 | 0.00126 | 10,904 |
| Operators | 括弧深さ 8 (21 B) | recognize | 0.02101 | 0.02017 | 47,912 |
| Operators | 括弧深さ 8 | recognize-notree | 0.01269 | 0.01239 | 24,960 |
| Operators | 括弧深さ 8 | **recognize-fast** | **0.00769** | 0.00740 | **18,032** |
| Operators | 括弧深さ 8 | 手書き目標形・認識 | 0.01166 | 0.01094 | 86,984 |
| Operators | **512 atom (1,023 B)** | ast | 0.92559 | 0.91200 | 2,364,216 |
| Operators | **512 atom** | recognize | 0.77820 | 0.75668 | 2,008,240 |
| Operators | **512 atom** | **recognize-notree** | 0.35952 | 0.35791 | 708,808 |
| Operators | **512 atom** | **recognize-fast** | **0.20690** | **0.20494** | **465,928** |
| Operators | **512 atom** | 手書き目標形・認識 | 0.35031 | 0.34519 | 2,094,600 |
| Operators | 512 atom | 手書き目標形・AST | 0.48236 | 0.47580 | 2,772,888 |
| Power | `2^3^2` (5 B) | recognize | 0.00381 | 0.00343 | 16,288 |
| Power | `2^3^2` | **recognize-fast** | **0.00202** | 0.00166 | **9,760** |
| Power | `2^3^2` | 手書き目標形・認識 | 0.00087 | 0.00057 | 5,352 |
| Power | 64 段 (127 B) | ast | 0.09143 | 0.09033 | 389,384 |
| Power | 64 段 | recognize | 0.07539 | 0.07453 | 343,264 |
| Power | 64 段 | recognize-notree | 0.02655 | 0.02604 | 66,016 |
| Power | 64 段 | **recognize-fast** | **0.01695** | 0.01669 | **43,752** |
| Power | 64 段 | 手書き目標形・認識 | 0.02037 | 0.01919 | 147,080 |

**step 別の効き**（512 atom 認識、中央値）:

| step | 時間 | 対 baseline | 割当 | 対 baseline |
|---|---:|---:|---:|---:|
| recognize（PR #14） | 0.778 ms | 1.00 | 2,008 KB | 1.00 |
| + step A（Match 木・capture・Recipe を作らない） | 0.360 ms | **0.46** | 709 KB | **0.35** |
| + step B（診断観測を記録しない） | 0.207 ms | **0.27** | 466 KB | **0.23** |

小さい入力（5〜127 B）の行も、今回は負荷が低いので再現性がある（run の最小と中央値の差が 5% 以内）。
ただし 5 B の絶対値は 2〜7 µs なので、手書きとの比を読むのは 512 atom と 64 段の行にする。

## 4. A/B — P4（`examples/p4-java`）

`bench.runs=7`。complex / x4 は warmup 2,000・反復 2,000 / 500、x16 / x64 は warmup 300・反復 125 / 50。
load average は complex/x4 が 3.59 → 5.30、x16/x64 が 4.35 → 5.08。

| fixture | mode | 中央値 ms | 最小 ms | 割当 B/parse |
|---|---|---:|---:|---:|
| complex | ast | 0.5376 | 0.5246 | 528,008 |
| complex | recognize | 0.4888 | 0.4817 | 495,584 |
| complex | **recognize-notree** | 0.3967 | 0.3896 | 290,848 |
| complex | **recognize-fast** | **0.2132** | **0.2113** | **124,872** |
| complex-x4 | ast | 1.9951 | 1.9415 | 2,006,296 |
| complex-x4 | recognize | 1.8095 | 1.7704 | 1,875,432 |
| complex-x4 | recognize-notree | 1.4326 | 1.4097 | 1,070,000 |
| complex-x4 | **recognize-fast** | **0.7725** | 0.7638 | **439,504** |
| complex-x16 | ast | 7.7370 | 7.5637 | 8,055,160 |
| complex-x16 | recognize | 6.9124 | 6.7360 | 7,535,848 |
| complex-x16 | recognize-notree | 5.7908 | 5.5862 | 4,343,584 |
| complex-x16 | **recognize-fast** | **3.1042** | 2.9212 | **1,855,008** |
| complex-x64 | ast | 36.671 | 35.916 | 32,278,272 |
| complex-x64 | recognize | 31.955 | 31.395 | 30,200,816 |
| complex-x64 | recognize-notree | 25.181 | 23.462 | 17,419,840 |
| complex-x64 | **recognize-fast** | **13.710** | **12.387** | **7,498,944** |

step 別（complex 認識）: 0.489 → 0.397（step A、0.81 倍）→ 0.213（step A+B、0.44 倍）。
**P4 では step B（診断）のほうが大きい**。Operators と逆で、P4 は rule 呼出しと候補除外の
失敗記録が多く、`Diagnostics` の frame 出入りと expected 吸収が支配的だったことを示す
（PR #14 の profile で `Diagnostics.leaveRule` 6.1% + `rule` 20.6% の内訳に当たる）。

## 5. 同値性の検証

- **二段構えの同値**（新規 `examples/p4-java` の `TwoModeEquivalenceTest`）: 全 fixture の完全入力・
  code point 境界での切り詰め・1 文字破壊（884 入力）× 入口 3 種 = **2,652 件**で、
  認識専用（観測要求なし）と認識＋観測要求を比べる。受理・cursor・scope（宣言・参照・診断・イベント・深さ）・
  recovery が常に一致、**失敗・末尾入力では診断（kind / offset / farthest / expected / deepestRule /
  ruleStack / nativeFailure）が完全一致**（D-023 の再解析）、成功では hints が空、出現表と lexical と AST が空。
  さらに**同じ入力の 2 度目の解析が 1 度目と同じ観測**（決定性。Rust 側の fast/診断 同値テストと同じ役割）。
- **候補除外の同値**（既存 `PredictEquivalenceTest`）: 884 入力 × 入口 3 種 × AST 有無 = **5,304 件**の
  `Session.predict` on/off 比較が緑（この経路は `lexical: true` なので完全経路を通る）。
- **corpus**: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java`
  → **Java Pass 1,310 / 期待拒否 Pass 28 / Known 28 / Fail 0 / 未観測 59**
  （`docs/reports/2026-09-24-java-corpus-twomode.{md,json}`）。系統別の表は
  `docs/reports/2026-09-23-java-corpus-contract.md` と **1 行も違わない**（`diff` で確認）。
- **IR golden 不変**: `git diff origin/master -- crates corpus ir` が空。front も self-host も動かしていない。
- **差分ファジング**: `cargo test -p ubnfc-fuzz --offline` の固定 seed キャンペーンが緑。
- `ubnfc-java` / `examples/p4-java` の maven test 緑、`examples/p4-java/scripts/generate.sh --check` 差分なし、
  `cargo fmt --check` と `cargo clippy --workspace --all-targets -- -D warnings` 緑。
- **既存テストの契約変更 1 件**: `BackendTest` は `RECOGNIZE` の診断が AST 経路と一致することを見ていた。
  D-023 の副作用 (a)（成功解析の hints は要求時だけ）に合わせ、**診断を要求した認識解析**と AST 経路を
  比べる形に直し、認識専用については「失敗なら再解析で一致・成功なら hints が空・出現表が空」を足した。

## 6. 目標に対する到達

| 目標 | 本 PR | 判定 |
|---|---|---|
| Operators 512 atom 認識 ≤ 0.4 ms | **0.207 ms**（手書き 0.350 ms の 0.59 倍） | **達成** |
| P4 complex-x64 認識 ≤ 15 ms | **13.71 ms**（最小 12.39） | **達成** |
| P4 complex 認識 ≤ 0.2 ms | 0.213 ms（最小 0.211） | **未達（6%）** |
| P4 complex AST ≤ 0.2 ms | 0.538 ms | **未達**（本 PR は AST 経路を変えていない） |
| P4 complex-x64 AST ≤ 15 ms | 36.7 ms | **未達**（同上） |

### 残ギャップ（complex 認識の 0.013 ms と AST 経路）

- **complex 認識の残り 6%** は、認識経路にまだ残る 2 つに割り当てられる。
  (a) `MemoTable` の probe と payload（complex は 1 parse あたり memo hit 464 件・保存も同数。§7 の
  memo off で 67 倍悪化する＝ hit が本質なので、消せるのは payload の作り方だけ）、
  (b) `Session.result()` の scope 出力（`scope.allDeclarations()` / `resolve()` / `ScopeEvent` の写し）。
  どちらも「認識経路が作らないもの」ではなく「認識でも公開する観測」なので、本 PR の範囲では落とせない。
  次の一手は scope 出力も要求時だけにする（D-035 と同じ形の 3 つ目の opt-in）ことだが、
  **corpus の scope 系統がそのまま観測しているので、契約を変える前に oracle 側の合意が要る**。
- **AST 経路は手つかず**。M5 の complex ≤0.2 ms / x64 ≤15 ms は AST 目標なので、
  そちらは「Recipe から AST を組む段」（`build` と `Session.list/scalar`）と、AST でも作る出現表の
  両方に残っている。PR #14 の profile を AST モードで採り直すところから。

## 7. memo の A/B（memo hit 0 の文法での要否）

同一プロセス・同一コード・`ParseOptions.withMemo(false)` だけを差し替えた 7 run の中央値。

| 文法 / 入力 | mode | memo=on | memo=off | 比 | on 割当 | off 割当 |
|---|---|---:|---:|---:|---:|---:|
| Operators 512 atom | ast | 0.947 ms | 0.872 ms | 0.92 | 2,364,184 | 2,068,336 |
| Operators 512 atom | recognize | 0.796 ms | 0.704 ms | 0.88 | 2,008,208 | 1,712,360 |
| Operators 512 atom | **recognize-fast** | 0.221 ms | **0.180 ms** | **0.81** | 465,896 | **350,400** |
| Power 64 段 | recognize-fast | 0.0172 ms | 0.0158 ms | 0.92 | 43,696 | 37,552 |
| **P4 complex** | ast | 0.550 ms | **26.44 ms** | **48.1** | 534,168 | 28,845,800 |
| **P4 complex** | recognize | 0.483 ms | **26.44 ms** | **54.7** | 501,744 | 28,813,376 |
| **P4 complex** | **recognize-fast** | 0.219 ms | **14.69 ms** | **67.0** | 131,032 | 13,753,192 |

- **Operators は memo が 1 度も当たらない**（PR #14 で rule 呼出し 2,561 件すべて miss）ので、
  memo を外すと速くなる。**認識経路ではその損の比率が上がる**（0.88 → 0.81）。他が安くなったぶん、
  「当たらない memo の保存」が相対的に重くなるため。
- **P4 は memo が無いと 67 倍遅い**。認識経路でも比率はむしろ上がる（54.7 → 67.0）。
- → **memo の要否は文法ごと（本当は規則ごと）に決めるべきで、いまは全規則 `successMemo=true` /
  `failureMemo=true` のまま**。これは front の解析（`analysis` の memo plan）の判断なので、
  指示どおり `crates/ubnfc-front` は本 PR では触らず、上の数値を根拠に **issue #17** を立てた。

## 8. 再現

```bash
cd ubnfc-java && mvn -o -q -DskipTests package && mvn -o test
bash examples/p4-java/scripts/generate.sh --check
cd examples/p4-java && mvn -o -q -DskipTests package && mvn -o test
# P4（同一プロセス交互 A/B）
cd examples/p4-java && taskset -c 28 java -Xms512m -Xmx2g -Xss16m \
  -Dbench.fixtures=complex,complex-x4 -Dbench.runs=7 -Dbench.warmup=2000 \
  -Dbench.modes=ast,recognize,recognize-notree,recognize-fast \
  -cp target/classes org.ubnfc.p4.Benchmark src/test/resources/fixtures
# family B（生成 Java と手書き目標形）
WORK=$(mktemp -d); jar=ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar
mkdir -p "$WORK/genjava/src" "$WORK/genjava/classes" "$WORK/benchjava"
for pair in operators:associative__Operators power:right-associative__Power; do
  java -cp "$jar" org.ubnfc.java.Main --package "gen.${pair%%:*}" --no-emit-driver \
    --ir "ir/fixtures/unlaxer-dsl__src__test__resources__${pair##*:}.ubnf.ir.json" --out "$WORK/genjava/src"
done
javac -nowarn -d "$WORK/genjava/classes" $(find "$WORK/genjava/src" -name '*.java')
(cd examples/handwritten-target/java && mvn -o -q -DskipTests test-compile)
javac -nowarn -cp "$WORK/genjava/classes:examples/handwritten-target/java/target/classes" \
  -d "$WORK/benchjava" benchmarks/java/FamilyBJavaBench.java
taskset -c 28 java -Xms512m -Xmx2g -Xss16m -Dbench.runs=7 -Dbench.iterations=400 \
  -Dbench.modes=ast,recognize,recognize-notree,recognize-fast \
  -cp "$WORK/benchjava:$WORK/genjava/classes:examples/handwritten-target/java/target/classes" FamilyBJavaBench
# corpus
cargo run -q --offline --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-24-java-corpus-twomode.md --json docs/reports/2026-09-24-java-corpus-twomode.json
```
