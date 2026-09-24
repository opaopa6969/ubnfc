# Java の AST 経路（2026-09-24、issue #19）

PR #18（D-056）で Java の**認識**経路は手書き目標形を追い越したが、**AST 経路は未変更**だった。
本ラウンドは AST 経路の profile を先に採り、**1 parse の中間表現を数えてから**減らす。
Rust backend（D-035 / D-036）の設計だけを移し、コードは移植していない。

## 結論（先に）

- **AST 経路にも観測の opt-in を通した**（D-057）。`ParseOptions.occurrences` / `diagnostics` は
  PR #18 では認識経路しか見ていなかったが、`buildAst=true` でも効くようにした。公開出現表を作る条件を
  Match 木を作る条件（`tree`）から分離した新しい旗 `occ` がその境目。
- **P4 complex の AST は 0.564 → 0.309 ms**（`ast-fast` = 観測を要求しない AST。同一プロセス交互、
  §4）で **1 parse の割当は 528 → 219 KB**。**x64 は 50.0 → 28.0 ms / 32.3 → 13.0 MB**。
  ただし **M5 の AST 目標（complex ≤0.2 ms / x64 ≤15 ms）は未達**で、残りは「認識そのもの」に乗っている
  （同じ条件の認識専用が complex 0.220 ms / x64 17.5 ms なので、**AST を 0 円にしても目標に届かない**。§6）。
- **Operators 512 atom の AST は 1.003 → 0.631 ms**（`ast-fast`）で、**issue が置いた目標 0.693 ms を達成**。
  ただし同一プロセスで測った手書き目標形は本セッションでは 0.560 ms なので、**手書きの 1.13 倍**は残る。
  割当は 1,444 KB で**手書き 2,785 KB の 0.52 倍**。
- **観測を全て要求する既定の AST（`ParseOptions.DEFAULT`）も速くなる**: 中間表現の作り方を直したぶんで、
  P4 complex は割当 663 → 563 KB（0.85 倍）、x64 は 31.9 → 25.8 MB（0.81 倍）、時間は 0.92〜0.95 倍。
  **認識専用経路も同じ修正の恩恵を受ける**（complex 割当 260 → 204 KB、x64 7.10 → 4.10 MB）。
- 観測（canonical JSON golden 13 件、corpus 1,310 件、同値テスト 7,956 件）は**一切動いていない**。

## 1. AST 経路の profile — 1 parse に何をいくつ作るか

`examples/p4-java` の生成物に計数を仕込んで 1 parse を測った（`origin/master` = before、本 PR = after。
load に依存しない一次証拠）。`complex`（332 B）と `complex-x64`（20,923 B）。

| 計数（complex） | before `ast` | after `ast` | after `ast-fast` | 認識専用 |
|---|---:|---:|---:|---:|
| rule 呼出し | 1,076 | 1,076 | 1,076 | 1,076 |
| うち memo hit / 保存 | 464 / 612 | 464 / 612 | 464 / 612 | 464 / 612 |
| `capture()` 呼出し | 2,983 | 2,983 | 2,983 | 2,983 |
| **`Match`** | 1,194 | 1,147 | 1,147 | 658 |
| **`Recipe`** | 142 | 142 | 142 | 0 |
| **`Recipe.Value`** | 584 | 584 | 584 | 13 |
| **`Occurrence`** | 356 | 356 | 356 | 13 |
| うち公開出現表への追加 | 356 | 356 | **0** | 0 |
| field 別 `List<Value>` の `ArrayList` | 173 | **15** | **15** | 0 |
| `combine` / うち `ArrayList` 確保 | 256 / 83 | 256 / **58** | 256 / **58** | 0 / 0 |
| 連接の `Match[]` | 759 | **322** | **322** | 0 |
| 反復の `ArrayList<Match>` | 126 | 126 | 126 | 0 |
| `result()` の `Capture` | 155 | 155 | **0** | 0 |
| typed AST ノード | 131 | 131 | 131 | 0 |
| 値の `String` 複製 | 23 | 23 | 23 | 0 |

x64 も同じ形（`ast`: rule 65,210 / Match 69,565 / Recipe 8,773 / Value 35,423 / Occurrence 21,965 /
Capture 9,731 / AST ノード 8,195、`ast-fast` は Capture 0）。

**構成で消せたもの**（`ast-fast` の列が 0 になっているもの）:

1. **公開 capture 出現表**（356 / parse、x64 で 21,965）と `result()` の `Capture`（155 / 9,731）。
   AST は `Recipe` の field 値だけを読むので、出現表は AST の**構築に一度も要らない**。
2. **診断の記録一式**（`Diagnostics` の frame 出入り・expected 吸収・rule stack・memo payload の snapshot）。
   AST 経路でも D-023 の二段構えがそのまま効く（失敗・末尾入力は同じ入力を診断モードで再解析する）。
3. `Diagnostics` の実体そのもの（frame 並列配列 9 本 + expected stack 1,024 = 1 parse 約 9 KB）。

**構成では消せないもの**: `Match` / `Recipe` / `Recipe.Value` / 局所出現は AST の材料なので残る。
Rust backend の 0.099 ms はこの 3 段を event arena 1 本に畳んでいるが、Java へ同じ形を入れるのは
生成側（`build` を `rule` の完了時に呼ぶ）まで含む変更になるので本 PR の範囲外にした（§6 の残り）。

## 2. 入れたもの

### (a) 観測の opt-in を AST 経路へ広げる（契約、D-057）

`Session` の旗を 3 つに分けた。`tree`（Match 木・Recipe を作るか）と **`occ`（公開出現表を作るか）**を分離し、
`diag` は従来どおり。既定（`ParseOptions.DEFAULT` や既存の構築子）は 3 つとも true で**振る舞いは不変**。

```java
this.tree = !twoMode || options.buildAst() || options.lexical() || options.occurrences();
this.occ  = !twoMode || options.occurrences();          // ← 新設
this.diag = !twoMode || options.diagnostics();
```

AST を作りつつ観測を要求しない解析は `options.withObservations(false, false)`（= 本報告の `ast-fast`）。
生成される入口の `if (!syntax.ok() && !s.diag) return parseEntryInternal(..., options.withDiagnostics(), ...)`
は AST 経路でも同じなので、**失敗・末尾入力の診断は要求して解析したものと完全に一致する**。
既定で空になるのは**成功した解析の hints** だけ（D-023 副作用 (a)）。

### (b) 中間表現の作り方（観測は 1 bit も変えない）

| 場所 | 従来 | 本 PR |
|---|---|---|
| `Session.rule` の field 表 | 呼出しごとに `String[] names` を作り、conversion / cardinality を `String.equals` で判定 | 規則ごとの定数 `Fields` に畳む（名前配列は `Recipe` と共有、判定は `boolean[]`） |
| 同 field 値の収集 | 値が 1 つでも `ArrayList` + `List.copyOf` | 2 件目が出るまで `ArrayList` を作らない（173 → 15 件） |
| `Session.capture` の射影 | `project()` が `Match` を包み直す | `projected()` が値列だけ返し、出現の追加と 1 個の `Match` にまとめる |
| `appended`（出現の追加） | `List.of(Object[])` が配列をもう一度複製 | 確保済み配列をそのまま包む |
| `Session.sequence` | 先頭で `Match[children.length]` を確保（失敗しても捨てる） | 最初の子が通ってから確保（759 → 322 件） |
| memo payload | `Memo` record を 1 件ずつ（complex 612 / parse、x64 36,711） | 4 本の並列配列 + `MemoTable` と同じ thread local の持ち越し |
| `effectsSince` / `restore` | `subList` + `List.copyOf` | 配列 1 本 / 末尾からの除去（`SubList` を作らない） |
| `replay` / `applyEffects` / `result` | 不変 list の for-each（`ListItr` を確保） | 添字ループ |
| `Diagnostics`（`diag=false`） | 毎回 1 実体（約 9 KB） | 読み出し専用の空実体を共有 |
| `result()` の観測列 | `captures` / `lexical` / `catalog` / `scopeEvents` / `recoveries` を必ず確保 | 空なら確保しない |
| 生成 `build()` の field 参照 | `s.scalar(r, "name", ...)` が名前で線形走査 | 生成時に確定した添字 `s.scalar(r, 0, "name", ...)` |

どれも「同じものを同じ順で作る」ままで、**1 parse の計数のうち `Match` / `ArrayList` / `Match[]` の
本数だけが減る**（§1 の表）。要素の同一性・順序・list の内容は変わらない（D-055 の前提を引き継ぐ）。

## 3. 計測環境

- AMD Ryzen 9 7950X（32 論理 CPU）、86 GB、WSL2 Linux 6.18.33.2。`taskset -c 28`、単一 thread。
- Oracle JDK 21.0.9 / HotSpot、`-Xms512m -Xmx2g -Xss16m`。
- **共有機で別エージェントが並走**しており、作業を通じて load average は 8〜45 で動いた。
  §4・§5 の表は **load average 7〜21** の区間で採った。**一次証拠は割当 byte と §1 の計数**（負荷に依存しない）。
- A/B は 2 通り: (i) **同一プロセス交互**（全 cell を warmup してから 1 run ごとに全 cell を巡回）で mode 間を比べる、
  (ii) **binary 交互**（`origin/master` の生成物と本 PR の生成物を 3 周交互に走らせる）で before/after を比べる。
- mode: `ast`（従来どおり観測を全て要求）/ `ast-noocc`（出現表を要求しない）/ `ast-fast`（観測を何も要求しない）/
  `recognize-fast`（AST も作らない。PR #18 の到達点）。

## 4. A/B — P4（`examples/p4-java`）

### 4.1 mode 間（本 PR、同一プロセス交互、`bench.runs=7`、load 12〜21）

| fixture | mode | 中央値 ms | 最小 ms | 割当 B/parse | 対 `ast` |
|---|---|---:|---:|---:|---:|
| complex (332 B) | ast | 0.5637 | 0.5276 | 432,904 | 1.00 |
| complex | ast-noocc | 0.5130 | 0.4999 | 390,864 | 0.91 |
| complex | **ast-fast** | **0.3091** | 0.2951 | **218,616** | **0.55** |
| complex | recognize-fast | 0.2198 | 0.2120 | 74,168 | 0.39 |
| complex-x4 | ast | 1.9739 | 1.8785 | 1,643,184 | 1.00 |
| complex-x4 | ast-noocc | 1.9821 | 1.8188 | 1,468,136 | 1.00 |
| complex-x4 | **ast-fast** | **1.2099** | 1.0590 | **831,368** | **0.61** |
| complex-x4 | recognize-fast | 0.8276 | 0.7906 | 267,984 | 0.42 |
| complex-x16 | ast | 9.3625 | 8.7665 | 6,466,072 | 1.00 |
| complex-x16 | ast-noocc | 9.9598 | 8.3216 | 5,763,864 | 1.06 |
| complex-x16 | **ast-fast** | **5.8334** | 4.9143 | **3,269,016** | **0.62** |
| complex-x16 | recognize-fast | 3.5641 | 3.2570 | 1,032,952 | 0.38 |
| complex-x64 (20,923 B) | ast | 50.022 | 43.628 | 25,776,632 | 1.00 |
| complex-x64 | ast-noocc | 47.795 | 39.107 | 22,967,720 | 0.96 |
| complex-x64 | **ast-fast** | **27.976** | 25.679 | **13,040,552** | **0.56** |
| complex-x64 | recognize-fast | 17.494 | 16.820 | 4,114,960 | 0.35 |

**step 別**（complex）: `ast` 0.564 ms → 出現表を落として 0.513（0.91）→ 診断も落として 0.309（0.55）。
割当は 433 → 391 → 219 KB。**AST でも診断のほうが大きい**のは認識経路（PR #18 §4）と同じ傾向。

### 4.2 before / after（binary 交互 3 周、`bench.runs=5`、load 7〜15）

3 周それぞれの中央値の中央値。**この走行だけ complex と x64 を同じ JVM に載せている**ので、
`MemoTable` の thread local 持ち越しが x64 の大きな表に占有されて complex が毎回表を確保し直す
（complex の割当が §4.1 より 130 KB 多いのはこれ。**両側に同じだけ乗る**ので比は読める）。

| fixture | mode | base 中央値 | 本 PR 中央値 | 比 | base 最小 | 本 PR 最小 | base 割当 | 本 PR 割当 | 割当比 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| complex | ast | 0.637 | 0.608 | 0.95 | 0.611 | 0.511 | 663,408 | 562,680 | **0.85** |
| complex | recognize-fast | 0.258 | 0.243 | 0.94 | 0.231 | 0.223 | 260,304 | 203,848 | **0.78** |
| complex-x64 | ast | 45.69 | 42.23 | 0.92 | 41.42 | 33.50 | 31,899,448 | 25,757,912 | **0.81** |
| complex-x64 | recognize-fast | 18.08 | 15.71 | 0.87 | 16.32 | 13.06 | 7,103,256 | 4,096,240 | **0.58** |

**観測を全て要求したままでも**割当が 15〜42% 減る。時間は 0.87〜0.95 で方向が一定（3 周とも同じ向き）。

## 5. A/B — family B（Operators / Power、生成と手書き目標形を同一プロセスで）

`bench.runs=7`、`bench.iterations=400`。load 8〜15。手書き目標形は AST と認識の 2 mode しか持たない。

| 文法 / 入力 | 実装・mode | 中央値 ms | 最小 ms | 割当 B/parse |
|---|---|---:|---:|---:|
| Operators **512 atom (1,023 B)** | base 生成・ast | 1.0027 | 0.9781 | 2,364,184 |
| Operators 512 atom | **本 PR 生成・ast** | **0.9372** | 0.8837 | **2,040,824** |
| Operators 512 atom | **本 PR 生成・ast-noocc** | **0.8129** | 0.7821 | **1,692,712** |
| Operators 512 atom | **本 PR 生成・ast-fast** | **0.6314** | 0.6097 | **1,444,040** |
| Operators 512 atom | 本 PR 生成・recognize-fast | 0.2130 | 0.2116 | 344,488 |
| Operators 512 atom | **手書き目標形・ast** | **0.5598** | 0.5189 | 2,785,200 |
| Operators 括弧深さ 8 | 本 PR 生成・ast / ast-fast | 0.0233 / 0.0147 | | 43,808 / 26,048 |
| Operators 括弧深さ 8 | 手書き目標形・ast | 0.0141 | | 98,376 |
| Power 64 段 (127 B) | base 生成・ast | 0.1031 | 0.0922 | 389,352 |
| Power 64 段 | 本 PR 生成・ast | 0.0876 | 0.0858 | 336,872 |
| Power 64 段 | **本 PR 生成・ast-fast** | **0.0501** | 0.0485 | **156,016** |
| Power 64 段 | 手書き目標形・ast | 0.0312 | 0.0293 | 197,960 |

- **Operators 512 atom の AST は 1.003 → 0.631 ms（0.63 倍）で、issue の目標 0.693 ms を達成**。
  割当は 2,364 → 1,444 KB で**手書き 2,785 KB の 0.52 倍**。
- ただし**同一プロセスで測った手書きは 0.560 ms** なので、時間では**手書きの 1.13 倍**が残る。
  （0.693 ms は `2026-09-24-java-backend-gap.md` の走行で測った手書きの値。手書き側の絶対値は
  セッションごとに 0.48〜0.69 ms の幅があるので、**同一プロセスで測った 0.560 を判定に使う**。）
- 小さい入力（5〜127 B）は 1 parse が 1〜90 µs で負荷の影響が読み切れないので、判定は 512 atom の行と割当で行う。

## 6. 目標に対する到達

| 目標 | 本 PR | 判定 |
|---|---|---|
| Operators 512 atom AST ≤ 0.693 ms | **0.631 ms**（`ast-fast`、同一プロセス手書き 0.560 の 1.13 倍） | **達成**（手書き同等には 13% 足りない） |
| P4 complex AST ≤ 0.2 ms | 0.309 ms（`ast-fast`。`ast` は 0.564） | **未達（1.55 倍）** |
| P4 complex-x64 AST ≤ 15 ms | 27.98 ms（`ast-fast`。`ast` は 50.0） | **未達（1.87 倍）** |
| AST の割当が手書き目標形以下 | 1,444 KB 対 2,785 KB | **達成** |
| canonical JSON golden 無変更 | 13 件一致 | 達成 |

### 残ギャップの内訳（計数で説明する）

**P4 の AST 目標に届かない分は、AST の組み立てではなく認識そのものに乗っている。**
同じ走行の認識専用（`recognize-fast`）が **complex 0.220 ms / x64 17.49 ms** なので、
**AST の組み立てを 0 円にしても complex は 0.2 ms に 10% 届かず、x64 は 15 ms を 17% 超える**。
AST 層の費用は complex で 0.089 ms（0.309 − 0.220、割当 144 KB）、x64 で 10.5 ms（割当 8.9 MB）で、
その内訳は §1 の計数どおり `Match` 489 件（1,147 − 658）・`Recipe` 142・`Value` 571・`Occurrence` 343・
連接の `Match[]` 322・反復の list 126・typed AST 131（x64 は概ね 64 倍）。

次の一手は 2 つに分かれる。

1. **認識側**（目標の支配項）。`recognize-fast` の profile では `MemoTable.locate` 8.6%・
   `ScopeStore.restore` 7.7%・`ScopeStore$Undo` の確保 2.8% が残る。表の slot 配置（4 long/slot、
   load factor 0.5）と scope の取消し記録は共有機では分離して測れないので、専有区間を取ってから。
   規則ごとの memo 要否（issue #17）もここに効く。
2. **AST 側**（`Match` → `Recipe` → AST の 3 段を 1 段に畳む）。`rule()` が recipe を作る代わりに
   その場で typed AST ノードを組めば `Recipe` 142 と `Value` 571 と `built` / `spans` の
   `IdentityHashMap` が消える。Rust backend が 0.099 ms に到達しているのはこの形。
   ただし生成側（`build` の呼び出し位置・`@rightAssoc` の span 補正・`#recovery` の遅延）を
   まとめて変えることになるので、**1 施策ずつ A/B する本ラウンドの作法には収まらない**。
   計数で見た上限は「AST 層 0.089 ms（complex）の大半」で、**それでも目標には届かない**（上記）。

## 7. 同値性の検証

- **二段構えの同値**: `examples/p4-java` の `TwoModeEquivalenceTest` **2,652 件**（884 入力 × 入口 3 種）と
  `PredictEquivalenceTest` **5,304 件**が緑。前者は AST 経路の観測要求あり / なしも突き合わせる。
- **canonical JSON golden**: `examples/p4-java` の 13 golden と
  `java_and_rust_goldens_are_identical_without_normalization` が**無変更で緑**。
- **corpus**: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java`
  → **Java Pass 1,310 / 期待拒否 Pass 28 / Known 28 / Fail 0 / 未観測 59**
  （`docs/reports/2026-09-24-java-corpus-ast19.{md,json}`）。系統別の表は
  `docs/reports/2026-09-23-java-corpus-contract.md` と **1 行も違わない**（`diff` で確認）。
- **IR golden 不変**: `crates` / `corpus` / `ir` は `origin/master`（#12 の PR #20 を取り込んだ 8c733a9）と
  byte 一致。front も self-host も触っていない。
- **差分ファジング**: `cargo test -p ubnfc-fuzz --offline` の固定 seed キャンペーンが緑。
- `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --check` /
  `cd ubnfc-java && mvn -o test`（148 件）/ `cd examples/p4-java && mvn -o test` /
  `examples/p4-java/scripts/generate.sh --check` はいずれも緑。
- `cargo test --workspace --offline --no-fail-fast` は **1 target だけ赤**:
  `ubnfc-harness` の `spec_corpus::ubnf_citations_resolve_to_real_spec_headings`。
  `origin/master` でも同じく赤で、原因は兄弟 repo の診断コード改名によるドリフト（仕様由来オラクル側、issue #11）。
  **本 PR とは無関係**で、`crates` / `corpus` / `ir` は `origin/master` と byte 一致。他の target は全て緑。

## 8. 再現

```bash
cd ubnfc-java && mvn -o -q -DskipTests package && mvn -o test
bash examples/p4-java/scripts/generate.sh --check
cd examples/p4-java && mvn -o -q -DskipTests package && mvn -o test
# P4（同一プロセス交互 A/B）
cd examples/p4-java && taskset -c 28 java -Xms512m -Xmx2g -Xss16m \
  -Dbench.fixtures=complex,complex-x4 -Dbench.runs=7 -Dbench.warmup=2000 \
  -Dbench.modes=ast,ast-noocc,ast-fast,recognize-fast \
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
  -Dbench.modes=ast,ast-noocc,ast-fast,recognize-fast \
  -cp "$WORK/benchjava:$WORK/genjava/classes:examples/handwritten-target/java/target/classes" FamilyBJavaBench
# corpus
cargo run -q --offline --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-24-java-corpus-ast19.md --json docs/reports/2026-09-24-java-corpus-ast19.json
```
