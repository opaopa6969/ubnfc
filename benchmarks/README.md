# parser 速度比較のドライバ

`docs/reports/2026-09-23-parser-speed-comparison.md` を作るための計測一式。
**文法をまたいだ比較はしない。** family ごとに同じ入力・同じ機械・同じセッションで交互に測る。

| family | 文法 | 実装 |
|---|---|---|
| A | tinyexpression P4（124 rule） | ubnfc Rust 生成 / ubnfc Java 生成 / tinyexpression-rs（unlaxer Rust codegen + Expr 解釈） / 旧 Java（tinyexpression 生成物 + unlaxer combinator） |
| B | Operators / Power | ubnfc Rust 生成 / ubnfc Java 生成 / 手書き Rust 目標形 / 手書き Java 目標形 |
| C | UBNF 自身 | ubnfc-front（手書き Rust） / ubnfc-selfhost（生成 Rust） / unlaxer-ubnf（手書き Rust） / unlaxer-dsl `UBNFParsers`（旧 Java combinator） |
| D | tinyexpression P4（同上） | ubnfc Rust 生成（scope あり / scope 無効化 IR） / ubnfc Java 生成 / **ANTLR 4**（`examples/p4-antlr`） / **tree-sitter**（`examples/p4-treesitter`） |

## 使い方

```sh
WORK=$(mktemp -d) M2_SEED=/tmp/unlaxer-predictive-203-m2 benchmarks/scripts/prepare.sh
WORK=$WORK benchmarks/scripts/run-family.sh a
WORK=$WORK benchmarks/scripts/run-family.sh b
WORK=$WORK benchmarks/scripts/run-family.sh c
```

`prepare.sh` は兄弟リポジトリ（`unlaxer-parser` / `tinyexpression`）のソース・pom・git 状態を変更しない。
private maven repository・tinyexpression のビルドコピー・生成コード・計測 crate は全て `$WORK` 配下に作る。
`M2_SEED` は依存 jar の再取得を避けるための既存 isolated repo（`org/unlaxer` は捨てて seed にする）。

family D だけは別スクリプト。外部ツール（ANTLR jar・tree-sitter CLI と runtime）は
`$WORK` 配下へ取ってくる。報告は `docs/reports/2026-09-24-p4-antlr-treesitter.md`。

```sh
WORK=$(mktemp -d) benchmarks/scripts/p4-external.sh prepare
WORK=$WORK benchmarks/scripts/p4-external.sh accept
WORK=$WORK benchmarks/scripts/p4-external.sh negatives
WORK=$WORK CORE=26 RUNS=7 benchmarks/scripts/p4-external.sh speed
WORK=$WORK benchmarks/scripts/p4-external.sh profile
```

同じ family D で、**ANTLR / tree-sitter の流儀に書き直した文法**（塔を 1 本の
`expression` に統合したもの、issue #39）を並べて測るのは `p4-antlr-idiomatic.sh`。
上の `prepare` が作った生成物を同じ `$WORK` から借りる。

```sh
WORK=$WORK benchmarks/scripts/p4-antlr-idiomatic.sh prepare
WORK=$WORK benchmarks/scripts/p4-antlr-idiomatic.sh accept
WORK=$WORK benchmarks/scripts/p4-antlr-idiomatic.sh superset
WORK=$WORK benchmarks/scripts/p4-antlr-idiomatic.sh profile
WORK=$WORK CORE=26 RUNS=7 benchmarks/scripts/p4-antlr-idiomatic.sh speed
```

JSON を **JSON 専用ライブラリ**（Jackson / Gson / serde_json）と並べるのは `run-json-libraries.sh`（issue #68）。
`prepare-external.sh` が作った `$WORK`（ANTLR4 / tree-sitter / ubnfc / 入力）を借り、入力ごとに 15 モードを交互に回す。
jar は `scripts/fetch-json-libraries.sh` が sha256 固定で `benchmarks/external/` に取る。serde_json は `--offline`。
報告は `docs/reports/2026-09-24-json-libraries-comparison.md`。

```sh
WORK=$WORK benchmarks/scripts/run-json-libraries.sh prepare
WORK=$WORK benchmarks/scripts/run-json-libraries.sh bench
WORK=$WORK benchmarks/scripts/run-json-libraries.sh check           # JSONTestSuite: Jackson 3 設定 / Gson 2 設定 / serde_json
WORK=$WORK benchmarks/scripts/run-json-libraries.sh check-features  # Jackson の寛容 feature 別の切り分け
```

Rust typed AST の arena 化（issue #59、D-077）の A/B は `run-arena-ab.sh`。基準 revision（`BASE`）の生成器と
作業ツリーの生成器で P4 / JSON / Operators を生成し、1 つの bin（`src/family_arena.rs`）で 5 変種
（before / after の認識と所有 AST、after の木）を交互に測る。報告は `docs/reports/2026-09-24-rust-arena-ast.md`。

```sh
WORK=$(mktemp -d) BASE=618225a benchmarks/scripts/run-arena-ab.sh prepare
WORK=$WORK benchmarks/scripts/run-arena-ab.sh alloc   # 定常状態と cold の確保
WORK=$WORK benchmarks/scripts/run-arena-ab.sh tree    # 木の節点・slot・容量
WORK=$WORK CORE=30 RUNS=7 benchmarks/scripts/run-arena-ab.sh time
```

## 計測の作法

- CPU は `taskset -c $CORE`（既定 30）で 1 コアに固定、単一 thread。
- Rust: `std::time::Instant`、1 run は `UBNFC_CMP_RUN_SECONDS`（既定 0.5 秒）ぶんの反復、
  warmup は反復数の半分、`UBNFC_CMP_RUNS`（既定 7）run の中央値・最小・最大。
  確保回数と byte は GlobalAlloc を数える別 parse（計時には含めない）。
- Java: `System.nanoTime()`、JIT warmup を明示（family B は 20,000 回以上、反復数の半分）。
  割当は `ThreadMXBean.getThreadAllocatedBytes`。JVM は `-Xms512m -Xmx2g -Xss16m`。
- 認識のみと AST 付きは、分けられる実装では分けて測る。分けられない実装（CST を必ず作るもの）は
  その旨を出力のモード名に書く（`認識(CST)` 等）。

## ファイル

- `src/bench_util.rs` … Rust 側の共通計時・計数（各 family の bench から `include!`）
- `src/family_{a,b,c}.rs` … family ごとの Rust 計測本体
- `src/family_d.rs` … family D の ubnfc Rust 側（`accept` / `bench`）
- `src/family_arena.rs` / `scripts/run-arena-ab.sh` … arena AST（#59）の before / after A/B
- `scripts/p4-external.sh` … family D の一式（外部ツールの取得・生成・受理・速度・profile）
- `scripts/p4-antlr-idiomatic.sh` … family D の「流儀で書き直した文法」側（生成・受理・上位集合・profile・速度）
- `java/FamilyBJavaBench.java` … family B の Java 2 実装を 1 JVM で交互に測る（反射）
- `java/UbnfLegacyBench.java` … family C の旧 Java `UBNFParsers`（反射）
- family A の Java は既存の `examples/p4-java` の `Benchmark` / `LegacyBenchmark` をそのまま使う

## 関数別の帰属（perf / callgrind / JFR）

P4 complex と JSON 100 KB の認識／所有 AST を、自己 CPU 時間・命令数・分岐ミス・cache miss に
分けて調べる道具と再現手順は [`attribution/README.md`](attribution/README.md)。Docker 内の
perf / valgrind と、暖機後の Java JFR / PrintInlining を使う。実機カウンタと模擬値は区別する。
初回の結果は [`2026-11-01-attribution.md`](../docs/reports/2026-11-01-attribution.md)。
