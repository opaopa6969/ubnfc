# AST 経路の capture 包みを読まれる site だけに絞る（2026-09-24、issue #37）

#30（PR #36）が計数で確定させた残差 —— **D-059 の capture 迂回は `tree=false` 限定**で、AST 経路は
1 parse あたり **2,276 回**の包みを払っている —— を詰める。Rust backend の D-035（`crates/ubnfc-rust`）
と同じ「AST の構築に要る capture site だけ残す」設計を Java backend へ入れる。**`crates/` は 1 行も触っていない。**

## 結論（先に）

- **静的解析の答えは issue の想定と違った**。P4 の capture site は **246 件すべてを @mapping の recipe field が読む**
  （うち 11 件は scope 効果も読む）。IR fixture 17 文法 408 site を通しても**読まれない site は 1 件だけ**
  （`ubnf.ubnf`）。つまり「誰も読まない site」は**ほぼ存在しない**。
  2,276 回の正体は **capture site を 1 つも持たない式**で、P4 の 1,043 式のうち **797 式**がこれに当たる（§1）。
- **その 797 式の包みを `tree=true` でも飛ばした**（**D-064**）。`tree=true` の包みは site が無ければ
  「意味値の射影」しか残らないので、射影を生成側の定数に畳んで `s.capture(…)` の 14 引数呼出しと
  interface 呼出し（`Call.parse`）を消す。**1 parse の包み（`Session.wrap`）は complex で 2,276 → 600、
  x64 で 138,104 → 36,762**（残るのは全て「@mapping が読む site を持つ式」）。§2。
- **時間**（同一プロセス交互・toggle、3 セッション、`bench.runs=11`）: **complex AST 0.29 → 0.22 / 0.24 → 0.20 /
  0.23 → 0.18 ms**（比 0.76〜0.84）。**M5 の `complex ≤0.2 ms` は達成**（最小値は 3 セッションとも
  0.151〜0.187、中央値も静かな 2 セッションで 0.176 / 0.200）。**x64 は 20.7 → 19.0 / 20.5 → 18.1 /
  19.2 → 19.0 ms（比 0.88〜0.99）で中央値は未達（15 ms の 1.21〜1.27 倍）**、最小値 13.3〜14.7 ms は目標内。§3。
- **AST 層だけ（AST − 認識）は complex 0.118〜0.147 → 0.074〜0.088 ms（比 0.58〜0.64）**。
  Rust の AST 層は 0.023 ms（0.089 − 0.066）なので、**5.3 倍 → 3.2 倍**まで縮んだ。
  残る 3.2 倍は**呼出しではなく中間表現**で、AST 層が 1 parse に作る `Match` 412 / `Recipe` 153 /
  `Recipe.Value` 466 / `Occurrence` 266 = 約 128 KB に対し、**Rust は AST 解析まるごとで 509 回 77 KB**。§4。
- **割当は 1 byte も動かない**（complex ast-fast 266,776 B、x64 12,084,400 B が on/off で同一）。
  本 PR が消すのは呼出しだけで、中間表現は消していない。
- 観測は動いていない: 新設 **`AstBypassEquivalenceTest` 10,608 件**、`FactorEquivalenceTest` 10,608 件 /
  `TwoModeEquivalenceTest` 2,652 件 / `PredictEquivalenceTest` 5,304 件、canonical JSON golden 13 件、
  corpus Java Pass 1,310 / Fail 0（系統別表は `2026-09-24-java-corpus-factor30.md` と byte 一致）、IR golden 不変。

## 0. 計測環境

| 項目 | 値 |
|---|---|
| 基準 | `origin/master` `ba867dd`（PR #36 = D-061 を取り込んだ後）。仕上げに `d4ff6fb` を merge（生成物は byte 不変） |
| host | AMD Ryzen 9 7950X（32 論理 CPU）、86 GB、WSL2 Linux 6.18.33.2 |
| Java | Oracle JDK 21.0.9 / HotSpot、`-Xms512m -Xmx2g -Xss16m`、`taskset -c 28`、単一 thread |
| 負荷 | **共有機で別エージェントが並走し、本ラウンドは load average 12〜45 と荒れた**。各表に採取時の値を書く |
| A/B | **同一プロセス交互 + toggle**。`Session.astBypass` を cell ごとに切り替え、`fixture × mode × {on, off}` の全 cell を warmup してから **1 run ごとに全 cell を巡回**する。機械の負荷変動は全 cell に同じように乗る。`recognize-fast` を**対照**（迂回は `tree=false` では元から有効なので on/off で同一のはず）に置き、その比を雑音帯として読む |
| 集計 | `bench.runs=11`、`bench.warmup=1000`。中央値と**全 run の最小値**を併記する |
| 回数 | 生成物の複製に計数を挿した一時 binary（`mktemp -d` 配下。commit には含めない）。**負荷に依存しない一次証拠** |

---

## 1. 静的解析 — 誰が capture 出現を読むのか

実行時に capture 出現（`Match.local`）を読むのは **2 か所だけ**である。

| 読む側 | 何で引くか | 条件 |
|---|---|---|
| `Session.rule` の field 収集 | recipe field の `siteIds` | `tree=true` かつ規則が recipe を作る |
| `Session.applyEffects`（scope 効果） | 効果の `captureSiteId`（`sameRuleText` は capture 名） | 常に |
| 公開出現表 `ParseResult.captures()` | 全件 | `occ=true`（= `occurrences` を要求したとき）だけ |

したがって「AST 経路が省ける site」= **recipe field にも scope 効果にも現れない site**。
`JavaEmitter.readSites()` がこれを IR から静的に求める（`sameRuleText` の mode があるときは名前一致も読む側に数える）。

**P4 の答え**:

| 数 | 値 |
|---|---:|
| capture site | 246 |
| @mapping の recipe field が読む site | **246** |
| scope 効果が読む site（上の内数） | 11 |
| **読まれない site** | **0** |

IR fixture 全体（capture を持つ 17 文法、408 site）でも**読まれない site は 1 件**（`unlaxer-dsl__grammar__ubnf`）。
**issue #37 の「2,276 のうち何件が読まれるか」への答えは「site を持つ包みは全部読まれる」**である。

では 2,276 回は何か。**capture site を 1 つも持たない式の包み**である。

| 分類（生成される式 1,043 件） | 件数 | 意味形 `text` | それ以外 |
|---|---:|---:|---:|
| capture site を持たない | **797** | 505 | 292 |
| site を持つ（全て @mapping が読む） | 246 | 42 | 204 |

site が無い包みは、`tree=true` でも **`projected()`（意味値の射影）以外は恒等**になる
（出現を 1 件も足さないので `local` は同一、`values` が変わらなければ `wrap` は `result` をそのまま返す）。

## 2. 入れたもの（D-064）

`JavaEmitter.treeBypassable(expr)` が「読まれる（かつ `skip` でない）site を 1 つも持たない式」を印付けし、
式の先頭に迂回を前置する。`Session.bypassWrap = !occ && !diag && !options.lexical()` が実行時の条件。

```java
// 意味形が text の式（射影も恒等）
if (s.bypassWrap) { Match r = f.matched ? eNM(s, f) : eNC(s, f); s.progress(f); return r; }
// それ以外（射影だけを定数 mixedScalar 付きで呼ぶ。認識専用経路は tree=false で畳む）
if (s.bypassWrap) { Match r = …; s.progress(f); return s.tree ? s.project(r, false) : r; }
```

- **観測は 1 bit も変わらない**。到達位置の更新（`progress`）は包みの中と同じ位置、失敗 hint は `diag` のときだけ
  記録されるので `!diag` の経路には無く、公開出現表を要求する経路（`occ`）と lexical trace を要求する経路は
  この分岐に入らない。`Session.project` は `wrap` の射影と同じ結果を返し、値列が変わらなければ `Match` を作らない。
- `!tree` は `!occ` かつ `!lexical` を含むので、**`bypassWrap` は D-059 の `bypassCapture` を包含する**。
  site を持たない 797 式は `bypassWrap` 1 つに集約され、残り 235 式が従来どおり `bypassCapture`、
  11 式（scope 効果が読む site を持つ）が包みを通る。
- 反復（`repeat` / `optional` / `separated`）で **site を持つ**式は対象外にした。包みが項目ごとに `Match` を
  組み直す枝があるので、site が読まれない場合でも組み直しの有無で `items` の `text` が変わり得る。
- 左因数分解（D-061）の `Session.Cap` にも同じ印（`treeBypass`）を足し、`tail()` が同条件で包みを飛ばす。
- **A/B 用の toggle**: `-Dubnfc.astwrap=false` / `Session.astBypass = false` で AST 経路だけ参照経路に戻る
  （認識専用経路の迂回は D-059 のまま常に有効なので、対照 `recognize-fast` は toggle で動かない）。

### 2.1 回数（1 parse。負荷に依存しない一次証拠）

| 計数 | complex / off | **complex / on** | x64 / off | **x64 / on** |
|---|---:|---:|---:|---:|
| `Session.wrap`（capture の包み）ast-fast | 2,276 | **600** | 138,104 | **36,762** |
| うち射影だけの呼出し `Session.project` | 0 | 919 | 0 | 55,162 |
| うち射影が値列を変えた（`Match` を作った） | — | **20** | — | **1,154** |
| `Session.wrap` recognize-fast（対照） | 13 | 13 | 832 | 832 |
| `Session.wrap` ast（観測を全て要求。`diag=true` なので迂回しない） | 2,983 | 2,983 | 181,021 | 181,021 |
| `Match` 確保 ast-fast | 1,070 | **1,070** | 65,078 | **65,078** |
| `Recipe` / `Recipe.Value` / `Occurrence` ast-fast | 153 / 479 / 279 | **153 / 479 / 279** | 9,477 / 29,585 / 17,478 | 同左 |
| 1 parse の割当 ast-fast | 266,776 B | **266,776 B** | 12,084,400 B | **12,084,400 B** |

**消えたのは 1,676 回の 14 引数呼出しと同数の interface 呼出し**で、中間表現は 1 個も減っていない。
残った 600 回は全て「@mapping が読む site を持つ式」＝ issue の受け入れ条件がいう「mapping が要る分」である。
迂回した 1,676 回のうち **757 回は意味形 `text`**で射影ごと恒等に畳まれ、919 回が `project` を通り、
**実際に `Match` を作り直したのは 20 回**だった。

## 3. A/B（同一プロセス交互 + toggle、`bench.runs=11`、3 セッション）

`on` = 本 PR の既定、`off` = `-Dubnfc.astwrap=false`（AST 経路だけ参照経路）。
`recognize-fast` は**対照**（両者で同じコードを通るので比 1.0 が期待値。ここが雑音帯）。

### 3.1 P4（`examples/p4-java`）

| セッション（load） | fixture | on 中央値 | off 中央値 | 比 | on 最小 | off 最小 | 比 | 対照 `recognize-fast` の比（中央値） |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| S1（32→20） | complex | **0.2203** | 0.2917 | **0.76** | **0.1868** | 0.2472 | **0.76** | 0.93 |
| S1 | complex-x4 | 1.0241 | 1.2211 | 0.84 | 0.7748 | 0.8593 | 0.90 | 1.03 |
| S1 | complex-x16 | 4.0723 | 4.3253 | 0.94 | 3.0481 | 3.1684 | 0.96 | 1.02 |
| S1 | complex-x64 | 18.962 | 20.700 | 0.92 | 13.625 | 14.264 | 0.96 | 1.07 |
| S2（20→17） | complex | **0.2002** | 0.2374 | **0.84** | **0.1538** | 0.2031 | **0.76** | 1.12 |
| S2 | complex-x4 | 0.7015 | 0.8775 | 0.80 | 0.6080 | 0.7382 | 0.82 | 1.06 |
| S2 | complex-x16 | 3.3765 | 3.6851 | 0.92 | 2.3963 | 2.8481 | 0.84 | 1.07 |
| S2 | complex-x64 | 18.096 | 20.465 | 0.88 | 13.332 | 15.908 | 0.84 | 1.06 |
| S3（17→12） | complex | **0.1759** | 0.2262 | **0.78** | **0.1514** | 0.1901 | **0.80** | 0.94 |
| S3 | complex-x4 | 0.7766 | 0.8293 | 0.94 | 0.5542 | 0.6819 | 0.81 | 0.97 |
| S3 | complex-x16 | 3.2616 | 3.5221 | 0.93 | 2.1769 | 2.3747 | 0.92 | 1.00 |
| S3 | complex-x64 | 19.040 | 19.200 | 0.99 | 14.743 | 13.742 | 1.07 | 0.90 |

**対照の比は 0.90〜1.12** なので、雑音帯はおよそ ±10%。complex の 0.76〜0.84 は**3 セッションとも帯の外**、
x4 / x16 / x64 は帯の縁（0.80〜0.99、12 cell 中 11 cell で on が速い）。

### 3.2 M5 目標に対する到達

| 目標 | #30 の値 | **本 PR（中央値の範囲 / 最小値の範囲）** | 判定 |
|---|---|---|---|
| P4 complex AST ≤ 0.2 ms | 0.3043（最小 0.2789） | **0.176 / 0.200 / 0.220**、最小 **0.151〜0.187** | **達成**（最小値は 3 セッションとも、中央値は静かな 2 セッションで） |
| P4 complex-x64 AST ≤ 15 ms | 24.20（最小 21.81） | **18.10 / 18.96 / 19.04**、最小 **13.33〜14.74** | **中央値は未達（1.21〜1.27 倍）／最小値は目標内** |
| Operators 512 atom AST ≤ 0.693 ms | 0.537 | **0.520**（手書き目標形 0.602 の **0.86 倍**） | **達成** |

### 3.3 family B（Operators 512 atom、`FamilyBBypassBench`、同一プロセス交互 + toggle、`bench.runs=9`）

| セッション（load） | 実装・mode | on 中央値 | off 中央値 | 比 | on 最小 | off 最小 | 割当 B/parse |
|---|---|---:|---:|---:|---:|---:|---:|
| S1（41→35） | 生成・ast-fast | 1.1014 | 1.2232 | 0.90 | 0.5297 | 0.4847 | 1,444,544 |
| S1 | 生成・ast（対照。`diag=true` で迂回しない） | 2.3376 | 2.0605 | 1.13 | 1.0248 | 0.7955 | 1,860,976 |
| S1 | 手書き目標形・ast | 1.2360 | — | — | 0.6169 | — | 2,785,176 |
| **S2（22→15）** | **生成・ast-fast** | **0.5203** | **0.5831** | **0.89** | 0.4573 | 0.4747 | 1,444,576 |
| S2 | 生成・ast（対照） | 0.7928 | 0.8532 | 1.07 | 0.7496 | 0.7387 | 1,861,008 |
| S2 | 手書き目標形・ast | 0.6019 | — | — | 0.5535 | — | 2,785,176 |

S1 は load 41 で雑音帯が広く（対照 1.13）採否に使えない。**S2（対照 1.07）で ast-fast は 0.89 倍**、
**生成 AST 0.520 ms は同一プロセスの手書き目標形 0.602 ms の 0.86 倍**（割当は 0.52 倍）。
Operators は 28 式中 21 式が迂回対象で、P4 ほど包みの比率が高くないぶん効きも小さい。

## 4. AST 層だけ（AST − 認識）と Rust との差

同じ走行の `ast-fast` と `recognize-fast` の差を「AST 層の費用」とする。

| fixture | セッション | **on の AST 層** | off の AST 層 | 比 | Rust の AST 層 |
|---|---|---:|---:|---:|---:|
| complex | S1 / S2 / S3 | **0.0852 / 0.0879 / 0.0741** | 0.1467 / 0.1371 / 0.1184 | **0.58 / 0.64 / 0.63** | **0.023**（0.089 − 0.066） |
| complex-x4 | S1 / S2 / S3 | 0.5333 / 0.2788 / 0.3808 | 0.7451 / 0.4799 / 0.4218 | 0.72 / 0.58 / 0.90 | — |
| complex-x16 | S1 / S2 / S3 | 2.456 / 1.934 / 1.739 | 2.735 / 2.337 / 2.006 | 0.90 / 0.83 / 0.87 | — |
| complex-x64 | S1 / S2 / S3 | 8.95 / 9.05 / 12.60 | 11.33 / 11.90 / 12.03 | 0.79 / 0.76 / 1.05 | **1.34**（5.86 − 4.52） |

（Rust は `docs/reports/2026-09-23-p4-rust-floor.md` の D-035 / D-036 採用後の値。）

**complex の AST 層は #30 の 0.175 ms → 本 PR 0.074〜0.088 ms** で、**Rust の 0.023 ms に対する比は 7.6 倍 → 3.2〜3.8 倍**。

**残る Δ の説明**: 呼出しはもう Rust と同じ桁（包み 600 回 / Rust 0 回だが、600 回は @mapping が要求する仕事そのもの）。
残るのは**中間表現の 3 段**で、AST 層が 1 parse に余分に作るのは

| 中間表現（complex、ast-fast − recognize-fast） | 個数 |
|---|---:|
| `Match` | 412 |
| `Recipe` | 153 |
| `Recipe.Value` | 466 |
| `Occurrence` | 266 |
| **AST 層の割当** | **約 128 KB**（266,776 − 138,584） |

Rust は **AST 解析まるごとで 509 回 / 77 KB**（event arena 1 本に畳んでいる）。
つまり**Java の AST 層だけで Rust の parse 全体より多く確保している**。
本 PR は割当を 1 byte も減らしていない（§2.1）ので、**残り 3.2 倍は `Match` → `Recipe` → typed AST の
3 段を畳む変更**（`build` を `rule` の完了時に呼ぶ、生成側の構造変更）でしか詰まらない。
これは #19 §6 の「残り 2」のうち未着手の 1 つで、本 PR の範囲外。

## 5. 同値性の検証

- **新設 `AstBypassEquivalenceTest` 10,608 件**（884 入力 × 入口 3 種 × `buildAst` 2 × 出現表要求 2）が緑。
  `Session.astBypass` を true / false に切り替えて、受理・cursor・主診断・farthest・nativeFailure・
  **capture 出現表**・scope（宣言・参照・診断・イベント）・recovery・canonical JSON AST を突き合わせる。
  出現表を要求する行が「迂回が効かない側」（`occ=true`）を固定する。
- `FactorEquivalenceTest` 10,608 件・`TwoModeEquivalenceTest` 2,652 件・`PredictEquivalenceTest` 5,304 件が緑。
- canonical JSON golden **13 件**が無変更で緑（§2.1 のとおり `Recipe` / `Recipe.Value` の個数まで同一）。
- corpus: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java`
  → **Java Pass 1,310 / 期待拒否 Pass 28 / Known 28 / Fail 0 / 未観測 59**
  （`docs/reports/2026-09-24-java-corpus-ast37.{md,json}`）。系統別・loader 別・文法別の表は
  `2026-09-24-java-corpus-factor30.md` と **1 行も違わない**（差分は jobs 数と生成器 SHA だけ）。
- **IR golden 不変**: `crates` / `ir` / `corpus` は `origin/master` と byte 一致。
- 差分ファジング: `cargo test -p ubnfc-fuzz --offline` の固定 seed キャンペーンが緑。
- `cargo test --workspace --offline --no-fail-fast` / `cargo clippy --workspace --all-targets -- -D warnings` /
  `cargo fmt --all --check` / `cd ubnfc-java && mvn -o test` / `cd examples/p4-java && mvn -o test` /
  `examples/p4-java/scripts/generate.sh --check`。

## 6. 再現

```bash
cd ubnfc-java && mvn -o -q -Dmaven.test.skip=true package && mvn -o test
bash examples/p4-java/scripts/generate.sh --check
cd examples/p4-java && mvn -o -q -DskipTests package && mvn -o test
# A/B（同一プロセス交互 + toggle）。AstBypassBench は本報告の scratch にある小さな driver で、
# Benchmark と同じ巡回をしながら cell ごとに Session.astBypass を切り替える。
taskset -c 28 java -Xms512m -Xmx2g -Xss16m -Dbench.runs=11 -Dbench.warmup=1000 \
  -Dbench.modes=ast-fast,recognize-fast -cp <scratch>:target/classes AstBypassBench src/test/resources/fixtures
# 参照経路だけで測りたいときは JVM 全体を戻す
taskset -c 28 java -Dubnfc.astwrap=false … org.ubnfc.p4.Benchmark src/test/resources/fixtures
```

## 7. 次の一手

1. **`Match` → `Recipe` → typed AST の 3 段を畳む**（AST 層 128 KB / 1,297 個の中間表現を減らす）。
   complex の AST 層 0.074〜0.088 ms のほぼ全部と、x64 の 9 ms がここに乗っている。
2. **x64 の中央値 18〜19 ms**。認識専用も同じ走行で 9〜10 ms（#30 の 6.0 ms より悪い）なので、
   静かな窓での再計測がまず要る。認識側の残差は #22 / #30 の続きで、本 PR の対象外。
