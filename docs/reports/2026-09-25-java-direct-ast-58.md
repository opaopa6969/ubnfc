# Java AST 経路の Recipe → typed AST 段を畳む（2026-09-25、issue #58）

issue #58 は Java の AST 経路にある `Match` → `Recipe` → typed AST の 3 段を「認識の出来事から直接組む形」へ畳み、
x64 AST ≤15 ms を狙う。本ラウンドでは**最後の段（Recipe → typed AST の後処理）と、rule 完了時の Recipe 化の
中間表現**を畳んだ（D-079）。**`Match` 木と capture 出現（1 段目）は残した**。理由と残りの手は §4。
`crates/`・Rust backend・認識専用経路のコードは触っていない（認識専用経路は `Session` を共有するので、同じ走行で対照として測る）。

## 結論（先に）

- **P4 complex AST（`ast-fast`）は 0.146〜0.166 → 0.127〜0.148 ms（比 0.84〜0.92）**。issue の受け入れ条件 **complex AST ≤0.15 ms を 5 run すべてで達成**。
- **x64 AST は 10.5〜12.9 → 9.2〜12.1 ms（比 0.87〜0.95）**。静かな窓では base も 15 ms 未満（#37 の 18 ms は load 12〜45 の値）で、
  **本 PR 後は 5 run すべてで ≤15 ms**。
- **1 parse の割当は complex 264 → 248 KB（−6%）、x64 11.92 → 10.88 MB（−8.7%）**。
- **AST 層（AST − 認識）は complex 0.068〜0.084 → 0.050〜0.060 ms**。Rust の AST 層 0.023 ms に対して **3.2 倍 → 2.2〜2.6 倍**で、
  受け入れ条件「Rust の 2 倍以内」は**未達**。残りは 1 段目（`Match` / `Occurrence` / `Value` の包み）に乗っている（§3）。
- 観測は動いていない: canonical JSON golden 13 件（span を含む）、p4-java の同値テスト 5 本（Factor / TwoMode / AstBypass / Predict / Exclusive）、
  ubnfc-java 901 件、corpus Java Pass 1,310 / Fail 0（系統別表は同日の `origin/master` 生成器と byte 一致）、IR golden 不変。

## 0. 計測環境

| 項目 | 値 |
|---|---|
| 基準 | `origin/master` `44f2ea0` の生成器で作った `examples/p4-java`（base）と本 PR（after） |
| host | AMD Ryzen 9 7950X（32 論理 CPU）、WSL2 Linux 6.18.33.2、Oracle JDK 21.0.9 |
| JVM | `-Xms512m -Xmx2g -Xss16m`、`taskset -c 28`、単一 thread |
| A/B | **binary 交互**（base と after の JVM を交互に起動、`Benchmark` の同一プロセス巡回で ast-fast / recognize-fast の 2 mode × 4 fixture）。`bench.warmup=1000`、`bench.runs=7`。**2 セッション**（S1 = 2 周、S2 = 3 周） |
| 負荷 | S1 load 7.6〜8.4、S2 load 10.0 → 5.9（共有機、別エージェント並走） |
| 対照 | `recognize-fast`（AST を作らない。`Session` の変更は AST 経路だけに効くので比 1.0 が期待値） |
| 一次証拠 | 割当 byte（`ThreadMXBean`）と JFR（allocation sample / execution sample、x64） |

## 1. どこに時間が乗っていたか（base、x64 `ast-fast`、JFR）

execution sample の **27% が `build`（Recipe → typed AST の後処理）とその下**だった。内訳の上位:

| 場所 | 中身 |
|---|---|
| `Session.scalar` → `Function.apply` | recipe 数ぶんの型を見る **megamorphic な interface 呼出し**（inline されない）と、冷えた `Recipe` / `Value` の pointer chase |
| `Session.remember` → `IdentityHashMap.put` / `resize` | 構築済み表 `built`（Recipe → AST）と span 表を既定容量 32 から倍々で育て直す |
| `ParseResult` 構築子 → `IdentityHashMap(Map)` | span 表の複製が entry を 1 件ずつ取り出して入れ直す（`EntryIterator$Entry` が割当の 2.7%） |
| `Session.node` → `new Recipe("#leaf:" + fallback, …, Map.of(), null)` | leaf 1 個ごとに文字列連結・hash 計算・空 Map からの配列化 |
| 生成 `build` の `v -> build(s.node(v, r), s)` | `r` と `s` を捕捉する lambda を **field 1 つごとに確保** |

## 2. 入れたもの（D-079）

| 段 | 従来 | 本 PR |
|---|---|---|
| 生成 `build` | 1 method の巨大 switch、field ごとに `s.scalar(r, i, name, v -> …)` | **recipe ごとの static method**（`b0` … `bN`）が field を添字で直に読み、変換を直書きする。lambda も interface 呼出しも無い |
| scalar / optional / list | `Session.scalar` / `optional` / `list`（`Function<Value, T>`） | `scalarValue` / `firstValue` / `present` / `listOf`（例外・空・null の扱いは同一。list は `ArrayList` + `List.copyOf` → 配列 1 本 + `List.of`） |
| 構築済み表 | `IdentityHashMap<Recipe, Object> built` | `Recipe.built` field（recipe は 1 parse の中でしか生きない） |
| span 表 | 既定容量で確保し倍々に育つ | 構築が始まってから、この解析が作った recipe 数 × 1.5 で 1 度だけ確保 |
| `ParseResult` の span 表複製 | `new IdentityHashMap<>(map)` | `IdentityHashMap` なら `clone()`（配列複製。中身は同じ、外側は従来どおり unmodifiable） |
| leaf fallback / recovery marker | `new Recipe("#leaf:" + f, …, Map.of(), null)` | `Recipe.leaf(id, …)`（id は fallback ごとに 1 度だけ連結、空の列は共有） |
| rule 完了時の結果 | Recipe 1 個につき `List.of(recipe)` と `new Value(…, nodes)` と `List.of(value)`、list field の要素ごとに `new Value(…, List.of(node))` | recipe が `self()` / `value()` / `values()` を 1 度だけ作って共有（list field の要素・rule の値列・scalar の単独値が同じ実体を指す） |
| rule 完了時の包み直し | 成功した rule は必ず `new Match(…)` | 局所出現・項目が空で区間・nodes・values が同じなら結果をそのまま返す（内容は同一） |
| `Session.text` | `new String(source.substring(a, b).strip())`（最大 3 確保） | strip と同じ範囲を先に求めて `substring` 1 回。span 表は識別子で引くので、共有され得る文字列（入力全体・空）だけ複製する |

どれも「同じ値を同じ順で作る」ままで、構築される typed AST・span 表の中身・mapping 失敗の文言と位置・
`RecoveredNodeMissing` の伝播は変わらない（`present` は従来 `scalar` の検査と同じ式）。

## 3. A/B（binary 交互、2 セッション）

### 3.1 P4 `ast-fast`（中央値 ms、各周の値。括弧は対照 `recognize-fast` の base / after）

| fixture | S1 base | S1 after | S2 base | S2 after | 比（周ごと） |
|---|---|---|---|---|---|
| complex | 0.166 / 0.146 | **0.139 / 0.127** | 0.151 / 0.153 / 0.161 | **0.128 / 0.138 / 0.148** | 0.84 / 0.87 / 0.84 / 0.90 / 0.92 |
| complex-x4 | 0.571 / 0.513 | **0.520 / 0.457** | 0.549 / 0.529 / 0.545 | **0.451 / 0.461 / 0.480** | 0.91 / 0.89 / 0.82 / 0.87 / 0.88 |
| complex-x16 | 2.449 / 2.045 | **1.871 / 1.795** | 1.949 / 1.933 / 1.985 | **1.799 / 1.712 / 1.790** | 0.76 / 0.88 / 0.92 / 0.89 / 0.90 |
| complex-x64 | 12.95 / 10.78 | **11.83 / 11.41** | 10.51 / 12.12 / 12.77 | **9.17 / 10.65 / 12.13** | 0.91 / 1.06 / 0.87 / 0.88 / 0.95 |
| 対照 complex `recognize-fast` | 0.083 / 0.079 | 0.082 / 0.077 | 0.083 / 0.078 / 0.090 | 0.076 / 0.079 / 0.094 | 雑音帯 0.95〜1.04 |
| 対照 x64 `recognize-fast` | 5.54 / 4.51 | 5.09 / 4.99 | 4.48 / 4.98 / 4.50 | 4.26 / 5.23 / 5.96 | 雑音帯 0.92〜1.32 |

x64 は対照の雑音帯（±30%）が広く、比 0.87〜1.06 の幅は採否には使えない。**x64 の採否は割当（§3.2）と、最小値
（base 9.47〜11.81 → after 8.44〜10.19 ms）で読む**。complex / x4 / x16 は 5 周すべてで after が速く、対照の帯（±5%）の外。

### 3.2 1 parse の割当（負荷に依存しない）

| fixture | base | after | 比 |
|---|---:|---:|---:|
| complex | 264,120 | 247,968 | 0.94 |
| complex-x4 | 1,016,944 | 951,936 | 0.94 |
| complex-x16 | 2,988,512 | 2,728,608 | 0.91 |
| complex-x64 | 11,916,432 | 10,877,136 | **0.91** |
| complex `recognize-fast`（対照） | 136,192 | 135,504 | 0.99 |

### 3.3 AST 層（`ast-fast` − `recognize-fast`、S2 の周ごと）

| fixture | base | after | Rust（`2026-09-23-p4-rust-floor.md`） |
|---|---|---|---:|
| complex | 0.068 / 0.075 / 0.070 | **0.052 / 0.060 / 0.054** | 0.023 |
| complex-x64 | 6.0 / 7.1 / 8.3 | **4.9 / 5.4 / 6.2** | 1.34 |

## 4. 残ったもの（1 段目）と採らなかった案

after の x64 `ast-fast` の割当上位（JFR allocation sample）は **`Session.wrap` が作る `Match` 9.8% / `Occurrence` 5.6% /
`Value` 4.9% / 出現列 3.7%（計 24%）**、連接・反復の `combine` の `Match` 約 10%、終端の `Match.empty` 3.7%。
Recipe → AST 段と rule 完了時の Recipe 化はもう上位に無い。

- **採らなかった (a): rule 完了時に typed AST を即時に組む（Recipe を経由しない）**。rule の結果は memo に入り、
  後の選択で捨てられることがある（complex で Recipe 153 に対し最終木の AST ノード 131）。即時に組むと
  (i) 捨てられた AST も公開 `nodeSpans` に入る（観測が変わる）、(ii) 捨てられる枝の mapping 失敗（`Integer.parseInt` 等）が
  例外として漏れる、(iii) 登録を最終木だけに絞るには結局木を 1 回辿る必要がある。観測を保ったまま畳めるのは
  「後処理を安くする」までで、本 PR はそこまでにした。
- **採らなかった (b): capture 出現を `Match` の不変列ではなく session の出現 stack（Rust の event arena 相当）に積む**。
  `wrap` の `Match` 包み直しと `combine` の列連結が消える見込み（割当の約 2 割）だが、backtrack する全経路
  （選択の `longest`・左因数分解の付け替え・expression memo の payload・反復の項目ごとの出現・回復）で
  stack の巻戻しと再生を揃える必要があり、`Session` の tree 経路をほぼ書き直すことになる。**x64 ≤15 ms と complex ≤0.15 ms は
  本 PR で届いた**ので、次の issue（AST 層を Rust の 2 倍以内へ）の手として残す。
- 採らなかった (c): 終端の `Match.empty` を tree 経路でも共有実例にする（D-075 の拡張）。区間を `wrap` の
  `selectedChildExtent`・`project`・反復の項目 text が読むので、読まれないことの静的証明が要る。

## 5. 同値性の検証

- `cd ubnfc-java && mvn -o test`: 901 件緑（`BackendTest` 65、`PlainMatchEquivalenceTest` 748 ほか）。
- `cd examples/p4-java && mvn -o test`: 32 件緑（canonical JSON golden、`generate.sh --check` 相当の byte 一致、
  `FactorEquivalenceTest` / `TwoModeEquivalenceTest` / `AstBypassEquivalenceTest` / `PredictEquivalenceTest` / `ExclusiveEquivalenceTest`）。
- corpus: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java --jobs 8` → **Java Pass 1,310 / 期待拒否 Pass 27 /
  Known 29 / Fail 0 / 未観測 59**。同日の `origin/master`（`44f2ea0`）の生成器で同じ走行をすると同じ合計で、系統別・loader 別・
  文法別の表は byte 一致（期待拒否 Pass 28 → 27 / Known 28 → 29 は #37 の報告以後の master で既に動いており、本 PR 由来ではない）。
- `cargo test --workspace --no-fail-fast` / `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --all --check`: 緑。
- 生成物: `examples/p4-java/scripts/generate.sh --write` で再生成して commit（`ir/examples/*` は Java を持たないので対象なし）。

## 6. 再現

```bash
cd ubnfc-java && mvn -o -q -DskipTests package && mvn -o test
bash examples/p4-java/scripts/generate.sh --check
cd examples/p4-java && mvn -o -q -DskipTests package && mvn -o test
# binary 交互: base（origin/master の生成器で生成した classes）と after を交互に起動する
taskset -c 28 java -Xms512m -Xmx2g -Xss16m -Dbench.fixtures=complex,complex-x4,complex-x16,complex-x64 \
  -Dbench.modes=ast-fast,recognize-fast -Dbench.warmup=1000 -Dbench.runs=7 \
  -cp <classes> org.unlaxer.ubnfc.examples.p4.Benchmark src/test/resources/fixtures
```
