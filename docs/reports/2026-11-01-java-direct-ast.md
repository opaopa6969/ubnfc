# Java の認識 event から typed AST を直接構築する（issue #4）

## 実装と観測契約

基準は `66a18ff8e71839b893d2edabdad14825f8436dae`（作業開始時の `origin/master`）。
D-079 の `Match → Recipe（field ごとの List<Value>）→ typed AST` から、
規則完了時の `AstEvent`（規則 ID・区間・静的な field plan・認識順の capture 出来事）を
生成された `build` / `bN` が直接読む方式へ変更した。既定経路では **Recipe を生成しない**。
scalar / optional は出来事の先頭値を直に読むため field list も作らない。
list field は選択された出来事から必要なときだけ平坦化する。

Rust の D-035 と同じく、公開する capture 出現表と AST が読む規則直下の capture は区別する。
今回の Java は `Match` / `Occurrence` / `Value` を認識中の出来事として使い、
Rust の連続 arena へ全てを移す変更は行っていない。
`@rightAssoc` の規則局所 trivia による区間補正は、従来と同じ field 射影で最大終端を求める。
この場合の一時的な値列は残るが、Recipe を構築したり値列を event に保持したりはしない。

AST の構築は構文成功・全入力条件が確定してから、選択された event だけを対象にする。
規則完了のたびに AST を作る案は採らなかった。memo や選択で後から捨てられる規則成功があり、
その数値変換失敗や node span を公開すると観測契約が変わるためである。
追加した `DirectAstTest` は ordered / longest の捨てられる overflow、捨てた AST の span、
選ばれた overflow の構文結果・位置・メッセージ・原因を memo 有無も含めて固定する。
`-Dubnfc.directAst=false` / `Session.directAst = false` は A/B と同値検証用の旧 field materialize 経路。
公開 API / IR / canonical JSON は変更していない。

## 負荷に依存しない計数

生成物の一時コピーに constructor 計数を加えた別 binary で、各 fixture を 1 回解析した。
この binary は計時に使用していない。`Event` は共通基底の constructor 回数なので、
旧経路の列には Recipe の super 呼出しも含む。

| fixture | 経路 | Match | Event | Recipe | Occurrence | Value | field 配列 | 公開 nodeSpans |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| complex | 旧 | 997 | 153 | 153 | 279 | 479 | 153 | 154 |
| complex | 直接 | 997 | 153 | **0** | 279 | 479 | **0** | 154 |
| complex-x64 | 旧 | 60,343 | 9,477 | 9,477 | 17,478 | 29,585 | 9,477 | 9,730 |
| complex-x64 | 直接 | 60,343 | 9,477 | **0** | 17,478 | 29,585 | **0** | 9,730 |

認識専用経路の計数は complex が Match 658 / Occurrence 13 / Value 13、
x64 が Match 39,844 / Occurrence 832 / Value 832（Event / Recipe は両方 0）。
したがって直接構築でも AST のために増える Match は **339 / 20,499**、
Occurrence は **266 / 16,646**、Value は **466 / 28,753** 残る。
「128 KB の AST 層全体を取り除いた」という変更ではなく、field 列への中間変換を取り除く変更である。

## 再現手順と集計規約

```sh
# sdkman の Oracle JDK 21.0.9 / Maven 3.9.9、Rust stable
WORK=$(mktemp -d)
WORK=$WORK benchmarks/scripts/run-java-direct-ast.sh prepare
WORK=$WORK CORE=28 SESSIONS=3 benchmarks/scripts/run-java-direct-ast.sh time
```

AMD Ryzen 9 7950X、WSL2、1 core 固定・単一 thread。
Java は `-Xms512m -Xmx2g -Xss16m`。
P4 の warmup は cell ごとに 1,000 回、complex は各 run 2,000 反復、x64 は 50 反復。
Operators 512 atom は warmup 20,000 回、各 run 1,000 反復。
各 cell 7 run の中央値・最小・最大と全 sample、`ThreadMXBean` の割当 byte を保存する。
同一 JVM で旧 / 直接 / AST を作らない対照を交互実行し、run ごとに巡回順も反転する。
Operators の手書き AST も同一 JVM の巡回に含める。
Java P4 → Rust P4 → Java Operators の順で 3 セッション繰り返す。
Rust は同じ入力・core、所有 typed AST と認識を交互実行し、7 run の中央値を採る。

AST 層は各言語の同一セッションの **AST 中央値 − 認識中央値**。
セッションをまたいだ最速値の差や過去の Rust 値を達成判定には使わない。
3 セッションの各中央値を全て報告し、代表値はその中央値とする。
共有機の他作業や CI が並走しており、`uptime` の前後値と `/proc/loadavg` の各 run 値を残す。

### Operators の予備計測の訂正

初回の Operators だけは誤って `ubnfc-java/src/test/resources/ir/operators.ir.json`
（SHA-256 `a44b290497f96bc0ba5685fe1335150b504214c2c45ebff911f28586605c3d04`）を使った。
`benchmarks/scripts/prepare.sh` の正式な family B は
`ir/fixtures/unlaxer-dsl__src__test__resources__associative__Operators.ubnf.ir.json`
（`5ea4e47b431275ef37868e37a6ff91e89ffdeae4d14eccaaa07c92e51164efeb`）である。
誤った IR は最適化 plan が異なり、生成 AST の割当も約 3.02 MB と正式な文法の比較条件を満たさない。
この予備値（直接 2.844550 ms、手書き 1.399698 ms）は原始記録に残すが達成判定から除外する。
入力・JVM・warmup・反復数を変えず、正式な IR で Operators を 3 セッション計測する。
P4 の IR と入力は初回から正式なもので、全 3 セッションを採用する。

## A/B の実測値

原始記録と集計 JSON は [データディレクトリ](2026-11-01-java-direct-ast-data/README.md)。
時間は ms。中央値は各 7 run、load は**実際の計測 run の 1 分値の範囲**。
`uptime` のセッション前後値も保存した（S1 79.05→58.21、S2 58.21→38.70、S3 38.70→33.88）。
warmup 中には監視上 load 100 超の時間帯もあった。

### P4（同一 JVM の交互 A/B）

| セッション | fixture | 旧 AST | 直接 AST | 直接 / 旧 | 認識対照の直接 / 旧 | load |
|---|---|---:|---:|---:|---:|---|
| S1 | complex | 0.433515 | **0.416256** | 0.960 | 1.387 | 59.91–66.83 |
| S1 | complex-x64 | 50.267013 | **42.414701** | 0.844 | 0.887 | 59.91–66.83 |
| S2 | complex | 0.317418 | **0.315104** | 0.993 | 1.170 | 48.03–51.51 |
| S2 | complex-x64 | 36.316181 | **36.567591** | 1.007 | 1.069 | 48.03–51.51 |
| S3 | complex | 0.238248 | **0.238373** | 1.001 | 1.056 | 27.77–30.69 |
| S3 | complex-x64 | 24.179030 | **23.321909** | 0.965 | 1.044 | 27.77–30.69 |

P4 の経路間の時間差は多くが認識対照の変動帯に収まるため、**安定した P4 の速度改善は主張しない**。
特に S1 complex の対照比 1.387 は大きく、AST 層の差分だけを強い改善の根拠にはできない。
原始記録の最小値も公開するが、達成判定は中央値で行う。

### AST 層（AST 中央値 − 認識中央値）と同じセッションの Rust

| セッション | fixture | Java 直接の層 | Rust の層 | Java / Rust | Rust 計測 load |
|---|---|---:|---:|---:|---|
| S1 | complex | 0.037303 | 0.045633 | 0.817 | 56.77–60.78 |
| S1 | complex-x64 | 19.690263 | 9.533964 | 2.065 | 56.77–60.78 |
| S2 | complex | 0.099424 | 0.040992 | 2.425 | 58.95–61.12 |
| S2 | complex-x64 | 15.277316 | 12.583185 | 1.214 | 58.95–61.12 |
| S3 | complex | 0.107096 | 0.040433 | 2.649 | 27.37–30.16 |
| S3 | complex-x64 | 9.680374 | 5.515263 | 1.755 | 27.37–30.16 |

complex の層費用の代表値は Java **0.099424 ms** / Rust **0.040992 ms** = **2.425 倍**。
x64 は Java **15.277316 ms** / Rust **9.533964 ms** = **1.602 倍**。
これは層費用それぞれの 3 セッション中央値の比で、比自体の中央値とは区別する。
x64 も S1 は 2 倍を超え、共有負荷による散らばりがある。

### Operators（正式 IR、512 atom、同一 JVM の手書きと交互）

O1 / O2 / O3 は原始記録の `s2` / `s3` / `s4`。誤指定した初回の値は上記のとおり判定から除外した。

| セッション | 旧 AST | 直接 AST | 手書き AST | 直接 / 手書き | 認識対照比 | load |
|---|---:|---:|---:|---:|---:|---|
| O1 | 0.599085 | **0.501393** | 0.743436 | 0.674 | 1.076 | 38.70–42.14 |
| O2 | 0.679530 | **0.636637** | 0.948602 | 0.671 | 1.122 | 31.32–33.88 |
| O3 | 0.849965 | **0.678303** | 0.893627 | 0.759 | 0.950 | 27.44–31.86 |

### 割当 byte / parse

| fixture | 旧 AST | 直接 AST | 削減 | 認識（両経路同一） |
|---|---:|---:|---:|---:|
| complex | 249,528 | 241,048 | 3.40% | 136,888 |
| complex-x64 | 10,970,424 | 10,444,336 | 4.80% | 4,026,232 |
| Operators | 1,248,128–1,248,160 | 1,194,824–1,194,856 | 4.27% | 16,696–16,728 |

field 配列と scalar / optional の field list がなくなった分が安定して減る。
AST 層の割当は complex **104,160 B**、x64 **6,418,104 B** 残り、規則完了 event 以外の
`Match` / capture / 列の包み全体が消えたわけではない。

### 受け入れ条件の判定と残りの手

| 条件 | 代表中央値・検証結果 | 判定 |
|---|---|---|
| Recipe を経由しない直接構築・canonical golden 13 件不変 | Recipe 153 / 9,477 → 0、golden 13 件通過 | 達成 |
| 同値テスト 5 本・corpus・IR golden 不変 | 既存 5 本＋直接比較 10,632 件、corpus 1,444 行全一致・Java Pass 1,316 / Fail 0、IR 差分なし | Java の無退行を達成。既存 IR 失敗 13 件は #33 |
| complex AST ≤0.15 ms | **0.315104 ms** | **未達**（3 セッション全て） |
| x64 AST ≤15 ms | **36.567591 ms** | **未達**（3 セッション全て） |
| Operators AST ≤手書き | **0.636637 ≤ 0.893627 ms** | **達成**（3 セッション全て） |
| AST 層 ≤Rust の 2 倍 | complex **2.425 倍**、x64 **1.602 倍** | **complex 未達** |

認識だけの代表中央値が complex **0.215680 ms** / x64 **21.290275 ms** で、既に AST 全体の絶対目標を超える。
この負荷では AST 層をゼロにしても目標に届かず、共有負荷と認識処理の費用を含めた帰属が必要である。
一方で complex の AST 層は低い負荷の S3 でも Rust の 2.649 倍で、event の field 列を除くだけでは不足する。
残る Match / Occurrence / Value の個数は前掲のとおりで、CPU / 割当プロファイルと突き合わせて
`wrap` / `combine`、capture の連続 arena、memo の event handle、読まれない Match の省略から最大項を選ぶ。
現時点でこの個数を特定の method の時間割合だとは断定していない。

未達目標・測定条件・分析・次の手と受け入れ条件を
[後続 issue #35](https://github.com/opaopa6969/ubnfc/issues/35) に切り出した。
本 PR は field 列への中間変換の除去を 1 つの意味単位とし、backtrack 全経路の arena 化や
認識側の構造変更を混ぜない。観測を変えず、割当削減を確認でき、1 PR の revert で戻せるためこの範囲を採った。

## 検証結果

- canonical JSON golden **13 件**は変更せず通過（span を含む）。P4 Java **33 テスト**が通過。
- 既存同値テスト **5 本**（Factor / TwoMode / AstBypass / Predict / Exclusive）が通過。
  新設 `DirectAstEquivalenceTest` は同じ全 fixture・切り詰め・破壊入力 × 入口 3 × AST 有無 × 出現表有無で
  **10,632 比較**。数値変換失敗・span の投機性を確認する `DirectAstTest` **3 件**も通過。
- `ubnfc-java` 全体 **11,885 件**は既存テスト 1 件が最初に 60 秒 timeout。
  同じ上限で当該テストを再実行して通過（42.33 秒）。追加の回帰テスト 3 件も通過。
- IR golden、`crates/ubnfc-front`、外部 pin は本変更の差分に含まない。生成 Java は `--release 17` で検査し、
  `examples/p4-java/scripts/generate.sh --check` と生成物が一致する。
- corpus は前後とも **Java Pass 1,316 / 期待拒否 Pass 27 / Known 29 / Fail 0 / 未観測 59**。
  `actual / outcome / differences / missing` を case のキーで比較し、**1,444 case×variant 全て完全一致**。
  系統・loader・文法・拒否理由・未観測の集計も全一致。報告書の差分は生成器 SHA-256 の 1 行だけ。
- ただし corpus の **IR 段階失敗 13 件は両生成器で既存**。`tiny-code-fence` の `LONG` に対して
  harness の固定宣言が `START / CLOSE / BODY / CHAR / END` しか出さず、`E-RULE-UNDEFINED` で止まる。
  Java の生成・build・比較 Fail は 0 だが、`run-all` 自体はこのため **exit 1**。
  意味論を緩めたり front / pin を変更したりせず、受け入れ条件付きの
  [後続 issue #33](https://github.com/opaopa6969/ubnfc/issues/33) へ切り出した。
  旧報告の入力は 1,212 ケース、今回の固定 pin は 1,231 ケースで、過去の Pass 1,310 と母集団が異なる。
- workspace の検証中、環境変数を除去して兄弟 checkout を参照する既存 CLI テストが、
  専用 worktree の兄弟に fixture が無いため 1 件失敗した。共有兄弟パスを作らず、
  自分の `mktemp -d` 配下へ未変更の Rust ソースと CI と同じ sibling symlink を置き、
  `cargo test -p ubnfc-harness --test cli` の **2 件が通過**。
  `cargo test --workspace --no-fail-fast` の失敗はこの CLI target だけで、その他の全 target は通過した。
  統合後の最終判定は、当 PR の `native workspace tests` / `wasm32-unknown-unknown build` の CI 記録を参照する。

固定 pin は unlaxer-parser `08f2bc1c918a34b84c333a8178c990383fdffb8d`、
tinyexpression `f86ce8a5ab0ab7d23fdba2cd477187df8aa7607c`。
外部 fixture は `scripts/fetch-external-fixtures.sh` で専用の一時ディレクトリへ取得した。


検査は `cargo fmt --all --check`、benchmark Rust の `rustfmt --check`、生成 benchmark の javac（`--release 17 -Xlint:all -Werror`）、
スクリプトの `bash -n`、専用ディレクトリでの再現スクリプト `prepare`、生成物の byte 一致を確認した。
