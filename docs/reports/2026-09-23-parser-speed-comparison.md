# parser 実装の速度比較（2026-09-23）

このワークスペースにある parser 実装を **文法ごとに** 並べた 1 回の比較。
**文法をまたいだ比較はしない。** family A / B / C の数値を横に並べて読んではいけない。

全数値は本日この機械で新しく採ったもので、過去の報告からの引用はない
（`2026-09-23-legacy-baseline.md` からは**手順と構成だけ**を引き継ぎ、値は測り直した）。
測れなかったものは §4 に列挙し、代わりの数値で埋めることはしていない。
再現手順は `benchmarks/README.md`、ドライバは `benchmarks/`、
生の CSV は [`2026-09-23-parser-speed-comparison/`](2026-09-23-parser-speed-comparison/)（7 ファイル）。

---

## 1. Family A — tinyexpression P4 文法（124 rule）

入力は 4 実装で byte 一致（md5 検証済み）。中央値は 7 run、µs/byte は中央値から計算。

| 実装 | 入力 | bytes | 認識 ms | AST ms | 認識 µs/byte | AST µs/byte | 割当/parse（認識 / AST） |
|---|---|---:|---:|---:|---:|---:|---|
| **ubnfc Rust（生成）** | complex | 332 | **0.0675** | **0.0988** | 0.203 | 0.298 | 31.8 KB (324 回) / 77.2 KB (509 回) |
| ubnfc Java（生成） | complex | 332 | 0.624 | 0.625 | 1.88 | 1.88 | 504 KB / 536 KB |
| tinyexpression-rs（生成／Expr グラフ解釈） | complex | 332 | 1.569 ※CST | 1.587 | 4.72 | 4.78 | 2.25 MB (10,814 回) / 2.39 MB (11,490 回) |
| 旧 Java（tinyexpression 生成物 + unlaxer combinator） | complex | 332 | 13.77 | 13.78 | 41.5 | 41.5 | 27.4 MB / 27.6 MB |
| **ubnfc Rust（生成）** | complex-x4 | 1,293 | **0.281** | **0.359** | 0.217 | 0.278 | 63.5 KB / 234 KB |
| ubnfc Java（生成） | complex-x4 | 1,293 | 2.290 | 1.960 | 1.77 | 1.52 | 1.92 MB / 2.05 MB |
| tinyexpression-rs | complex-x4 | 1,293 | 5.365 ※CST | 5.174 | 4.15 | 4.00 | 8.96 MB / 9.49 MB |
| 旧 Java | complex-x4 | 1,293 | 67.99 | 60.84 | 52.6 | 47.1 | 107 MB / 106 MB |
| **ubnfc Rust（生成）** | complex-x16 | 5,179 | **1.132** | **1.429** | 0.219 | 0.276 | 196 KB / 867 KB |
| ubnfc Java（生成） | complex-x16 | 5,179 | 6.841 | 6.951 | 1.32 | 1.34 | 7.68 MB / 8.20 MB |
| tinyexpression-rs | complex-x16 | 5,179 | 23.65 ※CST | 26.78 | 4.57 | 5.17 | 32.9 MB / 35.0 MB |
| 旧 Java | complex-x16 | 5,179 | 328.3 | 345.7 | 63.4 | 66.7 | 420 MB / 422 MB |
| **ubnfc Rust（生成）** | complex-x64 | 20,923 | **4.989** | **7.388** | 0.239 | 0.353 | 733 KB / 3.41 MB |
| ubnfc Java（生成） | complex-x64 | 20,923 | 32.90 | 35.42 | 1.57 | 1.69 | 33.0 MB / 35.1 MB |
| tinyexpression-rs | complex-x64 | 20,923 | 92.69 ※CST | 137.8 | 4.43 | 6.59 | 133 MB / 142 MB |
| 旧 Java | complex-x64 | 20,923 | 1,614 | 1,439 | 77.1 | 68.8 | 1.67 GB / 1.68 GB |
| **ubnfc Rust（生成）** | comparison-heavy | 179 | **0.0403** | **0.0690** | 0.225 | 0.385 | 19.2 KB / 107 KB |
| ubnfc Java（生成） | comparison-heavy | 179 | 0.302 | 0.315 | 1.69 | 1.76 | 440 KB / 510 KB |
| **ubnfc Rust（生成）** | comparison-heavy-x64 | 11,456 | **2.860** | **7.082** | 0.250 | 0.618 | 43.4 KB / 5.54 MB |
| ubnfc Java（生成） | comparison-heavy-x64 | 11,456 | 23.30 | 32.83 | 2.03 | 2.87 | 32.3 MB / 36.8 MB |

副次値（AST に加えて**候補選択**まで含む公開入口。認識 / AST の対比には使わない）:

| 実装 | complex | complex-x4 | complex-x16 | complex-x64 |
|---|---:|---:|---:|---:|
| tinyexpression-rs `parse`（公開 facade） | 1.588 ms | 5.657 ms | 24.92 ms | 90.92 ms |
| 旧 Java `P4PreferredAstMapper.parseDetailed` | 10.03 ms | 45.88 ms | 241.9 ms | 1,272 ms |

※ **apples-to-apples でない点**（数値の隣で読むこと）
- **tinyexpression-rs に「認識のみ」は無い。** `parse_tree_detailed_with_options` は必ず CST を作るので、
  表の「認識」列は *CST 構築込み* の値。ubnfc の認識モード（木を作らない）より多くの仕事をしている。
- **旧 Java も同じく CST（`Parsed` 木）を必ず作る。** `root` を認識列、`mapper` を AST 列に置いたが、
  `root` は既に木を持っている。AST 列との差が小さい（13.77 → 13.78 ms）のはそのため。
- 旧 Java の facade が `root` より速い（10.03 < 13.77）のは、facade が `Diagnostics.AUTO` で
  成功経路の失敗記録を作らないため。診断の仕事量が違う。
- ubnfc Rust / Java は `comparison-heavy` を `BooleanExpression` 入口で解析する。
  tinyexpression-rs と旧 Java の計測経路には入口指定 API が無いので、この 2 fixture の行が無い（§4）。

> 同じ P4 文法を **ANTLR 4 と tree-sitter へ移植**して受理と速度を比べたものは
> [`2026-09-24-p4-antlr-treesitter.md`](2026-09-24-p4-antlr-treesitter.md)（別セッションの新規測定。
> 文法が同じでも作るものが違うので、この表の数字と横に並べて読んではいけない）。

---

## 2. Family B — Operators / Power（小さな式文法）

`examples/handwritten-target` の 2 つの README は「速度比較は主張しない」と書いている。
ここで比較を作ったので条件を明記する: **生成側は既定 options（予測・左因子化・字句キャッシュ・
バッファ再利用が有効）、手書き側にはこれらの最適化が無い**（README の「FIRST 予測、memo 省略、
skip cache、SIMD、precedence climbing を加えていない」）。同じ文法・同じ入力・同じ 1 セッション。

| 文法 | 実装 | 入力 | bytes | 認識 ms | AST ms | 認識 µs/byte | 割当/parse（認識 / AST） |
|---|---|---|---:|---:|---:|---:|---|
| Operators | **ubnfc Rust（生成）** | `1+2*3` | 5 | **0.00157** | **0.00241** | 0.314 | 18.7 KB (4 回) / 23.5 KB (23 回) |
| Operators | 手書き Rust（目標形） | `1+2*3` | 5 | 0.00234 | 0.00323 | 0.467 | 10.2 KB (47 回) / 16.1 KB (85 回) |
| Operators | ubnfc Java（生成） | `1+2*3` | 5 | 0.00526 | 0.00612 | 1.05 | 20.6 KB / 22.6 KB |
| Operators | **手書き Java（目標形）** | `1+2*3` | 5 | **0.00139** | 0.00555 | 0.277 | 10.8 KB / 15.1 KB |
| Operators | **ubnfc Rust（生成）** | 括弧深さ 8 | 21 | **0.00463** | **0.00784** | 0.220 | 18.7 KB / 31.7 KB |
| Operators | 手書き Rust（目標形） | 括弧深さ 8 | 21 | 0.01942 | 0.02331 | 0.925 | 68.4 KB / 89.7 KB |
| Operators | ubnfc Java（生成） | 括弧深さ 8 | 21 | 0.02149 | 0.02049 | 1.02 | 48.8 KB / 54.0 KB |
| Operators | 手書き Java（目標形） | 括弧深さ 8 | 21 | 0.01156 | 0.01366 | 0.551 | 87.0 KB / 97.9 KB |
| Operators | **ubnfc Rust（生成）** | 512 atom | 1,023 | **0.1810** | **0.3678** | 0.177 | 18.7 KB / 512 KB |
| Operators | 手書き Rust（目標形） | 512 atom | 1,023 | 0.8224 | 0.9641 | 0.804 | 2.27 MB / 3.55 MB |
| Operators | ubnfc Java（生成） | 512 atom | 1,023 | 1.548 | 1.813 | 1.51 | 10.5 MB / 10.8 MB |
| Operators | 手書き Java（目標形） | 512 atom | 1,023 | 0.3563 | 0.5666 | 0.348 | 2.09 MB / 2.79 MB |
| Power | **ubnfc Rust（生成）** | `2^3^2` | 5 | **0.00102** | **0.00162** | 0.203 | 18.7 KB / 22.9 KB |
| Power | 手書き Rust（目標形） | `2^3^2` | 5 | 0.00118 | 0.00189 | 0.237 | 5.65 KB / 10.0 KB |
| Power | ubnfc Java（生成） | `2^3^2` | 5 | 0.00336 | 0.00404 | 0.671 | 16.3 KB / 17.9 KB |
| Power | **手書き Java（目標形）** | `2^3^2` | 5 | **0.00061** | **0.00097** | 0.121 | 5.33 KB / 7.78 KB |
| Power | **ubnfc Rust（生成）** | `2^…^2` 64 段 | 127 | **0.01050** | 0.02557 | 0.083 | 18.7 KB / 80.1 KB |
| Power | 手書き Rust（目標形） | `2^…^2` 64 段 | 127 | 0.02336 | 0.04649 | 0.184 | 170 KB / 290 KB |
| Power | ubnfc Java（生成） | `2^…^2` 64 段 | 127 | 0.07755 | 0.08644 | 0.611 | 343 KB / 389 KB |
| Power | 手書き Java（目標形） | `2^…^2` 64 段 | 127 | 0.01983 | 0.02791 | 0.156 | 147 KB / 198 KB |

※ **apples-to-apples でない点**
- 生成側はバッファ再利用があるので、小さい入力では**確保回数が 4 回**（固定バッファの使い回し）になる。
  手書き側は毎回作る。速度の差にはこの再利用が含まれる。
- 生成 Rust の割当 *byte* が小さい入力で手書きより大きい（18.7 KB vs 5.6 KB）のは、
  再利用バッファを 1 回で確保するため。入力が伸びるほど生成側が有利になる（512 atom で 18.7 KB vs 2.27 MB）。
- 手書き Java の `1+2*3` AST（0.00555 ms）は run のばらつきが大きい（最小 0.00394 ms）。
  同じ入力の認識（0.00139 ms）との差はこのばらつきを含む。
- x4 / x16 / x64 に相当する fixture がこの文法群には無い。スケーリングは
  `1+2*3`(5 B) → 512 atom(1,023 B) の µs/byte で見る（生成 Rust 0.314 → 0.177、手書き Java 0.277 → 0.348）。

---

## 3. Family C — UBNF 文法そのもの（1 文法・4 parser）

| 実装 | 入力 | bytes | mode | ms（中央値 / 最小） | µs/byte | 割当/parse |
|---|---|---:|---|---:|---:|---|
| **ubnfc-front（手書き Rust）** | `ubnf.ubnf` | 10,650 | AST(`UbnfFile`) | **0.0917 / 0.0738** | 0.0086 | 470 KB (1,215 回) |
| **unlaxer-ubnf（手書き Rust）** | `ubnf.ubnf` | 10,650 | AST(`UbnfFile`) | **0.0781 / 0.0742** | 0.0073 | 433 KB (1,567 回) |
| ubnfc-selfhost（生成 Rust） | `ubnf.ubnf` | 10,650 | 認識 | 0.4428 / 0.4362 | 0.0416 | 809 KB (258 回) |
| ubnfc-selfhost（生成 Rust） | `ubnf.ubnf` | 10,650 | AST（生成 AST） | 0.5622 / 0.5511 | 0.0528 | 1.37 MB (1,473 回) |
| ubnfc-selfhost（生成 Rust） | `ubnf.ubnf` | 10,650 | AST(`UbnfFile` 射影) | 5.238 / 5.071 | 0.492 | 2.67 MB (7,073 回) |
| 旧 Java `UBNFParsers`（combinator） | `ubnf.ubnf` | 10,650 | CST（認識相当） | 66.25 / 61.88 | 6.22 | 98.6 MB |
| ubnfc-front | `elements.ubnf` | 276 | AST | 0.00370 / 0.00367 | 0.0134 | 31.5 KB |
| unlaxer-ubnf | `elements.ubnf` | 276 | AST | 0.00478 / 0.00439 | 0.0173 | 28.7 KB |
| ubnfc-selfhost | `elements.ubnf` | 276 | 認識 / AST / 射影 | 0.0231 / 0.0299 / 0.0835 | 0.0838 | 70.0 KB / 93.9 KB / 168 KB |
| 旧 Java `UBNFParsers` | `elements.ubnf` | 276 | CST | 1.699 / 1.646 | 6.16 | 4.77 MB |
| ubnfc-front | `boundaries.ubnf` | 334 | AST | 0.00226 / 0.00224 | 0.0068 | 20.8 KB |
| unlaxer-ubnf | `boundaries.ubnf` | 334 | AST | 0.00243 / 0.00242 | 0.0073 | 17.0 KB |
| ubnfc-selfhost | `boundaries.ubnf` | 334 | 認識 / AST / 射影 | 0.0214 / 0.0265 / 0.0681 | 0.0640 | 56.9 KB / 81.2 KB / 150 KB |
| 旧 Java `UBNFParsers` | `boundaries.ubnf` | 334 | CST | 2.015 / 1.934 | 6.03 | 5.74 MB |
| ubnfc-front | `all-tokens.ubnf` | 559 | AST | 0.00478 / 0.00474 | 0.0086 | 44.3 KB |
| unlaxer-ubnf | `all-tokens.ubnf` | 559 | AST | 0.00614 / 0.00606 | 0.0110 | 44.7 KB |
| ubnfc-selfhost | `all-tokens.ubnf` | 559 | 認識 / AST / 射影 | 0.0246 / 0.0322 / 0.0995 | 0.0441 | 91.3 KB / 126 KB / 211 KB |
| 旧 Java `UBNFParsers` | `all-tokens.ubnf` | 559 | CST | 2.057 / 1.887 | 3.68 | 5.69 MB |
| ubnfc-front | `annotations.ubnf` | 653 | AST | 0.00469 / 0.00462 | 0.0072 | 40.5 KB |
| unlaxer-ubnf | `annotations.ubnf` | 653 | AST | 0.00581 / 0.00579 | 0.0089 | 43.4 KB |
| 旧 Java `UBNFParsers` | `annotations.ubnf` | 653 | CST | 2.666 / 2.116 | 4.08 | 6.26 MB |

※ **apples-to-apples でない点**
- **3 つの Rust 実装が作るものは同じではない。** ubnfc-front と unlaxer-ubnf は
  それぞれの `UbnfFile`（別 crate の別の型）を直接作る。ubnfc-selfhost は生成 parser の AST を作り、
  そこから `UbnfFile` へ**射影する**。同じ成果物で比べられるのは「AST(`UbnfFile` 射影)」行だけで、
  生成 parser そのものの値は「認識」「AST（生成 AST）」行。
- 旧 Java `UBNFParsers` は CST（`Parsed` 木）しか作らない。UBNF の AST 化は別コンポーネントなので、
  この表の Java 行は **AST 構築を含まない**。つまり Java 側が有利な条件での差である。
- `ubnf.ubnf` は 10,650 byte / 9,353 code point（md5 `2b98dabfbd598a75ad5b6565e6826c15`）。
  4 実装すべてに同じファイルを渡した。
- ubnfc-selfhost は `annotations.ubnf` を受理しない（構文エラー）。その行は無い。
- `unlaxer-dsl/ubnf-vscode` の「生成 `UBNFParsers`」は
  `org.unlaxer.dsl.bootstrap.UBNFParsers.getRootParser()` へ委譲するだけの 9 行の stub で、
  **独立した生成 parser ではない**。したがって Family C の Java 実装は 1 本しかない（§4）。

---

## 4. 測れなかったもの（数値で埋めていない）

| 対象 | 理由 |
|---|---|
| tinyexpression-rs と旧 Java の `comparison-heavy` / `comparison-heavy-x64` | どちらの計測経路にも入口（entry rule）指定の API が無く、`Formula` 入口では受理されない。ubnfc 側だけの値は表に残したが、比較はしていない |
| tinyexpression-rs / 旧 Java の「認識のみ」 | 実装に存在しない（必ず CST を作る）。CST 構築込みの値を「認識」欄に置き、`※CST` と注記した |
| 旧 Java `UBNFParsers` の AST モード | bootstrap parser に AST mapping が無い。CST までしか測っていない |
| ubnf-vscode の「生成 `UBNFParsers`」 | bootstrap への委譲 stub で、独立実装が存在しない |
| ubnfc-selfhost の `annotations.ubnf` | 受理しない |
| Java の割当**回数** | `ThreadMXBean` は byte しか出さない。Rust 側は GlobalAlloc で回数も数えている |
| Family B の x4/x16/x64 | 該当 fixture が無い。入力長 5 B → 1,023 B で代用 |
| 旧 Java の x4 以上の warmup | 1 parse が 68 ms〜1.6 秒なので warmup を 200 → 60 / 20 / 5 回へ落とした（complex だけ 200 回）。JIT 飽和は complex ほど十分ではない。**complex-x64 の 1,614 ms は「1.5〜1.7 秒の桁」までしか主張しない** |
| criterion / JMH による統計 | 使っていない。全て自前の nanoTime / `Instant` harness（`benchmarks/`） |

すべて本セッションの新規測定。**過去報告からの引用値はこの報告に 1 つも無い。**

---

## 5. 測定条件

- host: AMD Ryzen 9 7950X（32 論理 CPU）、86 GB、WSL2 Linux 6.18.33.2。共有機（別作業が並走）。
  `taskset -c 30` で 1 コアに固定、単一 thread。
  load average（1 分値、開始 → 終了）: family A 2.27 → 5.02、family B 4.58 → 3.29、family C 4.59 → 5.15。
- **family 内の全実装を 1 セッションで連続実行**（Rust bench は 1 プロセス内で交互、Java は同じ直後に実行）。
- Rust: rustc 1.98.1、cargo の既定 release profile。1 run は 1.0 秒（family A）/ 0.5 秒（B・C）ぶんの反復、
  warmup は反復数の半分（最大 2,000）、**7 run** の中央値・最小・最大。
  割当は GlobalAlloc を数える別 parse（計時外）。
- Java: Oracle JDK 21.0.9 / HotSpot 64-bit。`-Xms512m -Xmx2g -Xss16m`。
  旧 Java 経路のみ `--add-opens=java.base/{util,lang}=ALL-UNNAMED` と
  `-Dtinyexpression.p4.memoize=true -Dtinyexpression.p4.parse.timeout.millis=0`。
  warmup: ubnfc Java 2,000 回、family B は 20,000 回以上（反復数の半分）、
  旧 Java P4 は complex 200 / x4 60 / x16 20 / x64 5 回、旧 Java UBNF は 100 回。**7 run** の中央値。
  割当は `ThreadMXBean.getThreadAllocatedBytes`。
- 旧 Java（family A）の構成は `2026-09-23-legacy-baseline.md` §4 の B3 をそのまま再現した:
  `mktemp -d` 配下の private maven repository へ unlaxer master `ed4178e` を install し、
  tinyexpression `c70416e1` の**ソースコピー**をその repo でビルドし直し、
  classpath 順は tinyexpression 自身の順（`target/test-classes:target/classes:依存 jar`）、
  `Memoization.SAFE_FAILURES`。起動時 probe で `safeFailure=true` を確認済み
  （共有ツリーの `target/classes` は 2026-08-28 のままで memo marker が 0 個なので使えない）。
  complex の再現値 13.77 ms は baseline 報告の 13.64 ms と 1% 差。
- 旧 Java（family C）は unlaxer master `ed4178e` の `unlaxer-dsl/target/classes`、`SAFE_FAILURES`。
- tinyexpression-rs が使う `unlaxer-runtime` は tinyexpression が pin している
  git rev `6a4187c7`（master より 16 commit 古く、`#273`/`#274`＝ケース 30/31 を含まない）。
  family A の旧 **Java** は master `ed4178e`。**同じ unlaxer 版ではない。**
- 入力は md5 で byte 一致を確認（family A の 6 fixture は p4-rust / p4-java / tinyexpression の 3 箇所で同一）。
- 共有 `~/.m2`、`unlaxer-parser` と `tinyexpression` のソース・pom・git 状態は変更していない。

## 6. スケーリング（µs/byte、認識モード）

| 実装 | complex 332 B | x4 1,293 B | x16 5,179 B | x64 20,923 B | 332 B → 20,923 B |
|---|---:|---:|---:|---:|---:|
| ubnfc Rust（生成） | 0.203 | 0.217 | 0.219 | 0.239 | **×1.17** |
| ubnfc Java（生成） | 1.88 | 1.77 | 1.32 | 1.57 | ×0.84 |
| tinyexpression-rs | 4.72 | 4.15 | 4.57 | 4.43 | ×0.94 |
| 旧 Java | 41.5 | 52.6 | 63.4 | 77.1 | **×1.86** |

超線形なのは旧 Java（×1.86）だけ。ubnfc Rust の ×1.17 はキャッシュ局所性の範囲。
ubnfc Rust の **AST モード**の `comparison-heavy` だけ別で、179 B の 0.385 µs/byte が
11,456 B で 0.618 µs/byte（×1.6）になる（AST node 数が入力長に対して非線形に増える fixture）。

## 7. 目についたこと

1. **生成器の優劣が言語で逆転する。** 同じ IR から出した parser で、Rust では生成が手書き目標形より
   速い（512 atom で 0.181 vs 0.822 ms、×4.5）のに、Java では生成が手書きより**遅い**
   （1.548 vs 0.356 ms、×4.3）。割当も生成 Java が 10.5 MB、手書き Java が 2.09 MB（×5）。
   Rust backend が持つ予測・左因子化・字句キャッシュ・バッファ再利用が Java backend に無いことと、
   生成 Java の 1 parse あたり 10 MB という割当が同じ方向を指している。Java backend の最適化は未着手。
2. **UBNF では「生成 parser を UbnfFile へ射影する」部分が parse より高い。**
   `ubnf.ubnf` で生成 parser の AST 構築が 0.562 ms なのに、そこから `UbnfFile` へ射影すると 5.238 ms
   （×9.3、確保回数 1,473 → 7,073）。手書き front-end は同じ `UbnfFile` を 0.092 ms で直接作る。
   self-host の遅さは parser ではなく**射影層**にある。
3. **旧 Java だけが超線形。** 認識の µs/byte は 332 B の 41.5 から 20,923 B の 77.1 へ（×1.86）。
   他の 3 実装は ×0.84〜×1.17 で頭打ちしない。同じ P4 文法・同じ入力で、
   ubnfc Rust 生成との比は complex の 204 倍から complex-x64 の 323 倍へ開く。
   割当も 1 parse で 1.67 GB に達する（ubnfc Rust は 733 KB、×2,280）。
