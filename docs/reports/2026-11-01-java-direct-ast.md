# Java の認識 event から typed AST を直接構築する（issue #4）

## 実装と観測契約

作業開始時の基準は `66a18ff8e71839b893d2edabdad14825f8436dae`。並行変更は merge で取り込み、最終計測時の front は `65a9edd`。本 PR は front・外部 pin・IR・LSP / DAP を変更しない。
規則完了時に field ごとの `List<Value>` を詰めた `Recipe` を作る段を除いた。認識順の capture と静的 `Fields` を持つ `AstEvent` を、生成 `build` / `bN` が直接読む。scalar / optional は最初の値を列の確保なしで読み、list だけ必要時に平坦化する。leaf fallback も capture から typed record を直接作り、中間 event を作らない。既定経路の Recipe は 0。
Rust D-035 と同様、公開 capture 出現表と AST が読む規則直下の capture を区別する。`Match` / `Occurrence` / `Value` は認識の出来事として残す。rightAssoc の trivia 区間補正は従来と同じ射影で最大終端を求め、その一時的な値列は event に保持しない。
AST は構文成功・全入力条件の確定後、選択された event から構築する。規則成功ごとの即時構築では、memo や longest で後から捨てる枝の数値変換失敗や span を公開してしまうためである。recovery・memo 再生・変換例外・公開 span の契約を維持する。`-Dubnfc.directAst=false` / `Session.directAst=false` は同値比較と交互 A/B 用の参照経路。

## 計測の訂正と再現条件

初期測定後の監査で、参照経路の leaf fallback に旧実装にはなかった空配列の毎回確保を発見した。旧実装と同じ共有空配列へ修正し、既定経路の leaf event も省略してから、全3セッションを取り直した。初期の数値を最終判定へ混ぜない。初期原始記録はこのデータディレクトリ直下、当時の報告は [initial-report.md](2026-11-01-java-direct-ast-data/initial-report.md)、正式な最終記録は [final/](2026-11-01-java-direct-ast-data/final/) に保存する。初期の field 配列 153 / 9,477 という計数もこの余分な配列を含むため、下表の 142 / 8,773 が正しい。
Operators 初回にはテスト用 IR を誤指定した予備測定もあった。原始記録の `diagnostic-test-ir/` に残し、正式 IR の結果と区別する。今回の最終測定は全て `prepare.sh` と同じ正式 IR。入力・JVM・warmup・反復数・core は初期と同じで、負荷が低い run の抜粋はしない。

```sh
# sdkman Oracle JDK 21.0.9 / Maven 3.9.9、RUSTUP_TOOLCHAIN=stable
WORK=$(mktemp -d)
WORK=$WORK benchmarks/scripts/run-java-direct-ast.sh prepare
WORK=$WORK CORE=28 SESSIONS=3 benchmarks/scripts/run-java-direct-ast.sh time
```

AMD Ryzen 9 7950X / WSL2、core 28 固定、単一 thread。Java は `-Xms512m -Xmx2g -Xss16m`。P4 は各 cell warmup 1,000、complex 2,000 / x64 50 反復。Operators 512 atom は warmup 20,000、1,000 反復。各 cell 7 run の中央値・最小・最大・全 sample と ThreadMXBean 割当 byte を記録する。
同一 JVM で Recipe / 直接 / 認識対照を交互実行し、run ごとに巡回方向を反転する。Operators の手書きも同一 JVM に含める。Java P4 → Rust P4 → Java Operators を3セッション。Rust は同入力・core、所有 AST と認識を交互実行する。代表値は3セッションの中央値の中央値。AST 層は各セッションの AST 中央値 − 認識中央値を先に求め、層費用それぞれの代表値の Java / Rust 比を使う。
共有機の他ビルド・CI が並走する。各 run の `/proc/loadavg` と前後 `uptime` の全3値を保存し、以下の load は実計測 run の1分値の範囲。初期の高負荷測定も隠さず残している。

## 最終 A/B 実測

### P4（ms、各7 run の中央値）

| セッション | fixture | 旧 AST | 直接 AST | 直接 / 旧 | 認識対照の直接 / 旧 | load |
|---|---|---:|---:|---:|---:|---|
| S1 | complex | 0.180294 | **0.164443** | 0.912 | 0.909 | 23.47–27.04 |
| S1 | complex-x64 | 17.751057 | **17.101437** | 0.963 | 0.993 | 23.47–27.04 |
| S2 | complex | 0.154037 | **0.161499** | 1.048 | 0.965 | 15.00–16.28 |
| S2 | complex-x64 | 16.084037 | **16.393842** | 1.019 | 0.960 | 15.00–16.28 |
| S3 | complex | 0.259206 | **0.250967** | 0.968 | 1.061 | 18.11–25.65 |
| S3 | complex-x64 | 24.127632 | **18.620285** | 0.772 | 1.032 | 18.11–25.65 |

### AST 層と同じセッションの Rust（ms）

| セッション | fixture | Java 層 | Rust 層 | Java / Rust | Rust load |
|---|---|---:|---:|---:|---|
| S1 | complex | 0.064177 | 0.031295 | 2.051 | 19.99–22.74 |
| S1 | complex-x64 | 6.536186 | 2.902110 | 2.252 | 19.99–22.74 |
| S2 | complex | 0.076969 | 0.040764 | 1.888 | 17.15–19.19 |
| S2 | complex-x64 | 7.358493 | 1.934542 | 3.804 | 17.15–19.19 |
| S3 | complex | 0.099785 | 0.019491 | 5.120 | 23.70–24.97 |
| S3 | complex-x64 | 9.564828 | 4.183630 | 2.286 | 23.70–24.97 |

### Operators（正式 IR、512 atom、ms）

| セッション | 旧 AST | 直接 AST | 手書き AST | 直接 / 手書き | 認識対照比 | load |
|---|---:|---:|---:|---:|---:|---|
| S1 | 0.396422 | **0.391012** | 0.605938 | 0.645 | 1.016 | 19.04–20.78 |
| S2 | 0.514315 | **0.431362** | 0.706948 | 0.610 | 1.039 | 25.83–27.23 |
| S3 | 0.539820 | **0.502580** | 0.728373 | 0.690 | 0.979 | 21.41–22.74 |

### 割当 byte / parse

| fixture | 旧 AST | 直接 AST | 削減率 | 認識 |
|---|---:|---:|---:|---:|
| complex | 249,352 | 240,432 | 3.58% | 136888 |
| complex-x64 | 10,959,160 | 10,404,912 | 5.06% | 4026232 |
| operators | 1,248,128 | 1,194,824 | 4.27% | – |

## 負荷に依存しない constructor 計数

生成物の一時コピーへ constructor 計数を加えた別 binary で、各 fixture を1回解析した。計時には使わない。Event は基底 constructor 回数なので旧列は Recipe の super 呼出しを含む。

| fixture | 経路 | Match | Event | Recipe | Occurrence | Value | field 配列 | 公開 nodeSpans |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| complex | 旧 | 997 | 153 | 153 | 279 | 479 | 142 | 154 |
| complex | 直接 | 997 | 142 | **0** | 279 | 479 | **0** | 154 |
| complex-x64 | 旧 | 60,343 | 9,477 | 9,477 | 17,478 | 29,585 | 8,773 | 9,730 |
| complex-x64 | 直接 | 60,343 | 8,773 | **0** | 17,478 | 29,585 | **0** | 9,730 |

認識専用は complex が Match 658 / Occurrence 13 / Value 13、x64 が 39,844 / 832 / 832（両経路とも Event / Recipe 0）。AST のために増える Match は339 / 20,499、Occurrence は266 / 16,646、Value は466 / 28,753 残る。field 配列と scalar / optional の field list は除いたが、中間表現全体を取り除いた変更ではない。

## 受け入れ条件と次の手

| 条件 | 代表中央値・検証 | 判定 |
|---|---|---|
| Recipe を経由せず直接構築、golden 13 件不変 | Recipe 153 / 9,477 → 0、全13件通過 | 達成 |
| 同値5本、Java corpus、IR golden 不変 | 既存5本＋10,632比較、1,444行の観測一致、Pass 1,316 / Fail 0、IR差分なし | Java無退行を達成。既存IR失敗13件は #33 |
| complex AST ≤0.15 ms | **0.164443 ms** | 未達 |
| x64 AST ≤15 ms | **17.101437 ms** | 未達 |
| Operators AST ≤手書き | **0.431362 ≤ 0.706948 ms** | 達成 |
| AST 層 ≤Rustの2倍 | complex **2.459倍** / x64 **2.536倍** | 未達 |

認識のみの代表中央値は complex **0.100266 ms** / x64 **9.055457 ms**。AST 全体の絶対目標との比較では認識費用と共有負荷を含む。AST 差分は認識対照比と併記し、この3セッションだけで安定した速度改善を断定しない。
残る Match / Occurrence / Value の計数は CPU の時間割合ではない。CPU / 割当プロファイルと突き合わせて `wrap` / `combine`、capture の連続 arena、memo の event handle、読まれない Match の省略から最大項を確定する。未達目標・原因分析・次の手・受け入れ条件は [後続 #35](https://github.com/opaopa6969/ubnfc/issues/35) で追跡する。
今回は field 中間変換の除去を1つの意味単位とする。backtrack 全経路の arena 化や認識構造の変更は混ぜず、観測不変を確認した可逆な範囲を選んだ。merge commit の `git revert -m 1` で戻せる。

## 最終検証

- P4 Java **34テスト通過**。canonical JSON golden **13件を変更せず通過**。既存同値テスト5本（Factor / TwoMode / AstBypass / Predict / Exclusive）を維持。
- `DirectAstEquivalenceTest` は886入力×入口3×AST有無×出現表有無の **10,632比較**に加え、leaf fallback の公開 span 表全体の同値を確認。`DirectAstTest` 3件は ordered / longest の捨てる overflow、捨てた AST の span、選んだ overflow の結果・位置・メッセージ・原因を memo 有無で検証。
- generator 全体 **11,888テスト、失敗0 / error 0 / skip 0**。初期実装時には既存の60秒 timeout 1件が再実行で通ったが、最終版の全実行は再実行なしで通過。
- corpus は現行 front `65a9edd` と同じ pin で旧生成器 / 修正版を実行し、**1,444 case×variant** の `actual / outcome / differences / missing` が全て一致。系統・loader・文法・拒否理由・未観測の集計も一致。両方 Java **Pass 1,316 / 期待拒否 Pass 27 / Known 29 / Fail 0 / 未観測59**。
- **IR 段階失敗13件は両生成器で既存**。harness が `tiny-code-fence` の `LONG` を固定宣言せず `E-RULE-UNDEFINED` になる。run-all は双方 exit 1。Java の生成・build・比較 Fail は0。front / pin を変更せず、受け入れ条件付き [#33](https://github.com/opaopa6969/ubnfc/issues/33) へ切り出した。旧報告は1,212入力、現在の pin は1,231入力で、過去の Pass 1,310 と母集団が異なる。
- `generate.sh --check` で生成物 byte 一致、生成 Java `--release 17`、benchmark driver `--release 17 -Xlint:all -Werror`、benchmark Rust `rustfmt --check`、再現 script `bash -n` / `prepare` を確認。
- workspace の初期検証では既存 CLI target が専用 worktree の兄弟 fixture 不足で1件失敗し、それ以外の全 target は通過。共有パスを変更せず、一時ディレクトリへCIと同じ sibling配置を作り当該2テストを通した。統合後の最終 workspace 判定は PR #36 最新 HEAD の native / wasm CI を参照する。
- 初期 master では `cargo fmt --all --check` が通過。後に取り込んだ #24 の front 等には未整形箇所があり、最終の同コマンドはそこで失敗する。本 PR の Rust benchmark は整形確認済み。範囲外の front は変更しない。

固定 pin: unlaxer-parser `08f2bc1c918a34b84c333a8178c990383fdffb8d` / tinyexpression `f86ce8a5ab0ab7d23fdba2cd477187df8aa7607c`。外部 fixture は `scripts/fetch-external-fixtures.sh` で専用一時ディレクトリへ取得した。最終の generator / front / 原始JSON SHA-256 は `final/corpus-comparison.json`、計測した AST source は `final/source.sha256`、固定IRは `final/ir.sha256` に記録する。
