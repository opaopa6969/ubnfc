# ubnfc — UBNF パーサジェネレータ（clean-room）

UBNF 文法から、**実行時ライブラリに依存しない** Java パーサと Rust パーサを直接生成する。
`unlaxer-parser` のコンビネータ・ランタイム（ParseContext / トランザクション / Token スタック）は
一切使わず、文法の意味論を Grammar IR へ落としてから各言語のコードを書き下す。

```
.ubnf ──▶ ubnfc-front (Rust) ──▶ Grammar IR v1 (JSON) ──┬─▶ ubnfc-java (Java) ──▶ *.java（JDK 21、依存なし）
          字句/構文/解決/検証/                            └─▶ ubnfc-rust (Rust) ──▶ *.rs（std のみ、serde は optional）
          脱糖/意味論/解析/最適化計画
```

- 入力: `.ubnf`（`unlaxer-parser` と同じ文法言語。既存の全 `.ubnf` が通ることが目標）
- 出力: 生成物だけで完結するパーサ。runtime crate / jar へのリンクは無い
- 設計の正本は [`docs/design/`](docs/design/)、決定は `03-design-v1.md` §8 の表

---

## 1. 何を解く道具か

`unlaxer-parser` は、パーサコンビネータのランタイム（トランザクション・Token スタック・
実行時 memo）の上で `.ubnf` からパーサを生成していた。ubnfc はその **観測可能な振る舞いを
固定したまま**、ランタイムを消す。

- 文法の意味（どこに trivia が入るか、capture の個数、memo が安全か、診断の位置）を
  **front が解析して IR に載せ**、backend はその値をそのまま使う。backend は文字表・
  分岐の形・DFA を選んでよいが、意味を再解釈しない（D-009）。
- 生成物は JDK 21 / Rust std だけに依存する（生成 Rust は `#![forbid(unsafe_code)]`、
  生成 Rust crate の依存は `json` feature のときの serde / serde_json だけ。
  `examples/handwritten-target/java` は同じ目標形を `jdeps -recursive` で `java.base` のみと確認している）。

## 2. 珍しいところ

### 2.1 スコープと記号表が宣言的

`@scopeTree` / `@declares` / `@backref` を文法に書くと、生成パーサが解析中に
スコープの enter/leave、宣言、参照、未定義参照の警告まで面倒を見る。
意味述語（ANTLR の `{...}?`）を入れずにこれを成立させているのが設計の芯で、
終端が「(入力, 位置) の純粋述語」である性質に memo・候補除外・字句キャッシュの
健全性が全部乗っている（`docs/design/06-ubnf-v2.md` §4）。

ロールバックは undo ログ（pop 済み scope・同名再宣言の lookup 上書き・診断 clear は
append の逆ではない）。memo hit のときは効果を同順で再生する（`03-design-v1.md` §3.8）。

### 2.2 独立な 2 バックエンドを機械的に突き合わせる

Java backend と Rust backend は front だけを共有し、生成器も runtime も独立に書かれている。
そこで `crates/ubnfc-fuzz` が **同じ文法・同じ入力を両方で走らせ、観測が食い違うこと自体を
不具合として扱う**（差分ファジング）。共有契約（`farthestOffset` 等）は厳密比較し、
言語別 oracle と決めた項目（`syntax` 診断の主 offset、`expected` の表記）だけを弱める。

### 2.3 self-host の固定点

UBNF 自身のメタ文法（sibling repo の `unlaxer-parser/unlaxer-dsl/grammar/ubnf.ubnf`）から
生成した Rust パーサでその `ubnf.ubnf` を読み直し、
そこから作った IR が golden（`ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json`）と byte 一致し、
その IR から再生成したソースが
`crates/ubnfc-selfhost/src/generated` と byte 一致することをテストが固定している
（`docs/design/05-selfhost.md`）。手書き frontend へのフォールバックはしない。

### 2.4 実行契約が文書で固定されている

二重カーソル（consumed / matched）、mode / invert / R フラグ、反復の進捗規則、
診断の主位置と最遠到達、memo キーの構成が
[`docs/design/04-cursor-and-diagnostics-contract.md`](docs/design/04-cursor-and-diagnostics-contract.md)
に旧実装のファイル:行つきで書かれている。「`EMPTY{2,2}` は失敗、`EMPTY{1,2}` は成功」
のような細部まで契約対象。

### 2.5 front が「死んだ候補」を証明して警告する

PEG の順序付き選択は候補順を誤ると候補が黙って死ぬ。front は**証明できた場合だけ**
4 コードで警告する（D-037）。受理・AST・診断・IR は 1 byte も変わらず、`--strict` で
exit 5 になる。

---

## 3. 現状（2026-09-24 時点）

### 3.1 互換性

「互換」とは **旧実装の記録された振る舞いに 1,212 ケースの corpus 上で一致すること**、
および **2 バックエンド間の差分ファジング**であって、「正しさの証明」ではない。

corpus は `unlaxer-parser` のテスト資産 127 ファイル / 1,212 ケースで、文法 variant を
展開すると 1,425 件（case × variant）になる。

| backend | Pass | 期待拒否 Pass | Known（既知乖離） | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|
| Java | 1,310 | 28 | 28 | **0** | 59 |
| Rust | 1,292 | 30 | 28 | **0** | 75 |

出典: [`docs/reports/2026-09-23-java-corpus-choicecheck.md`](docs/reports/2026-09-23-java-corpus-choicecheck.md) /
[`docs/reports/2026-09-23-rust-corpus-choicecheck.md`](docs/reports/2026-09-23-rust-corpus-choicecheck.md)。
「未観測」は欠落した観測項目（`/evaluationError`、`/rollback`、`/scope/resolved` 等）で、
Pass に読み替えない。

**既知乖離 28 件は意図的**。D-017 / D-018 で「旧実装のバグは再現しない」と決めた範囲
（`@comment` の値を読まずに行コメントを有効化する挙動、recovery の非 BMP 過剰消費、
root への `@recovery` 無効化など）で、新契約 `recovery-projection` /
`recovery-diagnostics` / `comment-trivia` / `import-resolution` に置き換えてある。
詳細は [`docs/decisions/D-018-undecided-divergences.md`](docs/decisions/D-018-undecided-divergences.md)。

差分ファジング（[`docs/reports/2026-09-23-differential-fuzzing.md`](docs/reports/2026-09-23-differential-fuzzing.md)）:
320 seed から 292 文法が両 backend で build まで通り、7,008 件の入力比較で
**92 の差分クラス（根本原因 12 件）**を検出した。うち 2 件（front が自分の IR 契約に落ちる問題、
`@recovery(AUTO)` の適用範囲が両者で違って受理そのものが変わる問題）は修正済みで、
残りは `crates/ubnfc-fuzz/findings/` に最小再現つきで記録してある。
内部不変条件では `allConsumed == (consumed == 入力長)` が**両 backend で 355 件ずつ破れる**
——これは実装差ではなく protocol の文言の不足で、正しい定義は
`allConsumed == (全入力解析が成功したか)`（D-024）。

self-host（[`docs/design/05-selfhost.md`](docs/design/05-selfhost.md)）: 比較対象 25 件のうち
`ubnf.ubnf` が記述する構文だけを使う 20 件は `UbnfFile` が全 field / span まで一致する。
残り**5 件**は**手書き frontend だけが持つ互換拡張**（dotted mapping class、`@eval` の `=` 区切り、
annotation keyword の識別子境界、setting block 内の二重引用符、`@declares` description の非 STRING capture）で、
生成パーサが表の byte offset で拒否する。self-host を通すための文法 / IR の加工はしていない。
これとは別に**唯一の逆向き乖離**（生成側だけが広い）があり、`ubnf.ubnf` 自身が `@whitespace: javaStyle`
経由で UBNF が定義していない `/* ... */` ブロックコメントを受理してしまう問題を、
`ubnfc_selfhost::parse_file` が生成パーサの外でブロックコメントを拒否する字句の門（**D-051**）で閉じている。
以前は「8 件目」として記録していた**3 段以上の dotted ルール参照（`a.b.Value`）は解消済み**
（[`unlaxer-parser` PR #285](https://github.com/opaopa6969/unlaxer-parser/pull/285) が
`ubnf.ubnf` の `QuantifiedRef` を chained dot 対応へ直し、ubnfc 側は
[PR #20](https://github.com/opaopa6969/ubnfc/pull/20) が self-host の生成物を再生成した）。
詳細は `docs/design/05-selfhost.md`「7 件目」「8 件目」と
[`docs/reports/2026-09-24-spec-derived-oracle.md`](docs/reports/2026-09-24-spec-derived-oracle.md) §9.6・追記。

**仕様由来オラクル**: 旧実装の記録された振る舞いとの一致とは別に、`unlaxer-dsl/specs/*.md` の規範文だけから
期待値を導いた corpus（`corpus/spec/` — UBNF 129 ケース、P4 32 ケース、JSON 283 ケース。
どの実装の観測値も使っていない）を
4 実装（ubnfc 手書き frontend、ubnfc 自己ホスト生成、`unlaxer-ubnf`、Java bootstrap）へ流している
（PR [#10](https://github.com/opaopa6969/ubnfc/pull/10) が初回計測、PR
[#23](https://github.com/opaopa6969/ubnfc/pull/23) が `unlaxer-parser` #279 の仕様訂正に追随）。
`docs/compat-matrix.md` の「期待値の出所（provenance）」列が、旧実装から採取した項目と仕様由来の項目
（`Sp` / `Sp4`）を区別する。詳細は
[`docs/reports/2026-09-24-spec-derived-oracle.md`](docs/reports/2026-09-24-spec-derived-oracle.md)。

**この workspace の 10 実装比較**: UBNF 文法・P4 文法・Operators/Power 文法のそれぞれで、
このワークスペースにある実装を横並びにした報告が 2 本ある——機能面
（[`docs/reports/2026-09-23-parser-feature-correctness-comparison.md`](docs/reports/2026-09-23-parser-feature-correctness-comparison.md)、
3 リポジトリにまたがる 10 実装を対象）と速度面
（[`docs/reports/2026-09-23-parser-speed-comparison.md`](docs/reports/2026-09-23-parser-speed-comparison.md)）。
後者には、この README の性能比較（§3.2）が言及しない参照点として**`tinyexpression-rs`**
（旧世代の Rust コード生成 + `unlaxer-runtime`。P4 complex で 1.569 ms、CST 構築込み）が載っている。
ubnfc はこれと直接の比較対象ではないが、旧世代 Rust 生成物の実測値として記録に残す。

### 3.2 性能

**この repo での最適化の基準は旧実装（`unlaxer-parser` + `tinyexpression` P4）である。**
2026-09-24 に **ANTLR4 4.13.2 / tree-sitter 0.25.10 との比較**を加えた
（[`docs/reports/2026-09-24-antlr-treesitter-comparison.md`](docs/reports/2026-09-24-antlr-treesitter-comparison.md)、
issue #26）。同じ言語を 3 記法（`.ubnf` / `.g4` / `grammar.js`）で書き、
[JSONTestSuite](https://github.com/nst/JSONTestSuite) 318 件と同一入力の速度で突き合わせた結果は:

- **正しさ**: `y_` 95 件は 4 実装すべて受理、`n_` 188 件は ANTLR / tree-sitter が全件拒否、
  **ubnfc は 185 件**（文字列中の生の制御文字 3 件を受理してしまう。UBNF v2 に
  「U+0000–U+001F を除く code point 集合」を書く手立てが無いため。文法の書き方の問題ではない）。
  実装間で判定が割れたのは 318 件中 **5 件だけ**。
- **速度（JSON 1 MB、認識のみ）**: ANTLR4 11.2 ms（89.6 MB/s）、tree-sitter 47.3 ms（CST 込み）、
  **ubnfc Rust 234.5 ms（4.3 MB/s）**。ubnfc は入力が大きいほど µs/byte が悪化し、
  原因は **memo の `defaultSafe` 判定**と特定した（memo を切ると 1 MB が 51 ms・µs/byte が平ら）。
- **速度（512 atom の算術式、認識のみ）**: ANTLR4 0.045 ms、ubnfc Rust 0.119 ms、tree-sitter 0.403 ms。
  木込みでは ubnfc Rust の typed AST 0.260 ms が ANTLR4 の parse tree 0.209 ms に近い。

ANTLR4 の適応的予測と**エラー回復**、tree-sitter の**増分解析**はどちらも測っていない
（ubnfc が目指していないものでもある）。報告の §5 に「何が同条件でないか」を並べてある。

同じ日に **実用文法 P4（119 規則）でも同じ 4 実装を比べた**
（[`docs/reports/2026-09-24-p4-antlr-treesitter.md`](docs/reports/2026-09-24-p4-antlr-treesitter.md)、
issue #27。文法は `examples/p4-antlr/TinyExpressionP4.g4` と `examples/p4-treesitter/grammar.js`）:

- **受理**: 16 fixture 中 12 件で 4 実装が完全一致。残り 4 件（`comparison-heavy` 系）は
  P4 が PEG の順序つき選択で先頭候補にコミットして拒否するのに対し、ALL(\*) と GLR が
  後続文脈を見て別候補で受理する差。追加の拒否入力 10 件も 4 実装すべてで拒否。
- **速度（`complex` 332 B、認識のみ）**: tree-sitter 0.0415 ms（CST 込み）、
  **ubnfc Rust 0.0696 ms**、ubnfc Java 0.257 ms、**ANTLR4 645 ms**。
  ANTLR が 4 桁遅いのは P4 の型別の順序つき選択（`Expression ::= NumberExpression | BooleanExpression | …`）で
  ALL(\*) が毎回 LL 全文脈へ落ちるためで、`BooleanExpression` 入口なら同じ parser が 0.99 ms（650 倍速い）。
- **その 645 ms は文法の書き方の問題だった**（issue #39、報告の「追補」節）。同じ言語を ANTLR の流儀
  （型別の塔を 1 本の左再帰 `expression` に統合）で書き直すと、**同じ回転の中で 993 ms → 1.83 ms（542 倍）**、
  LL 全文脈への転落は 12 fixture すべてで **0 回**、SLL だけで 16 fixture 全部が通る。
  tree-sitter 側も同じ統合で `conflicts` 23 → 1 件、`prec.dynamic` 27 → 0 箇所、`STATE_COUNT` 882 → 307。
  ただし受理する言語は P4 の**上位集合**になる（`1 + true` が型エラーになり構文エラーでなくなる）し、
  流儀版でもなお ubnfc Rust の 17 倍・tree-sitter の 52 倍かかる。**「ANTLR が遅い」のではなく
  「PEG 前提の型別の塔を ALL(\*) にそのまま持ち込むと遅い」**。
- **scope 解決の代金**: ubnfc は `@scopeTree` / `@declares` / `@backref` を常に実行する。
  その効果を IR から外して生成し直すと認識が 17〜29% 速くなる（確保回数は 10 倍減）。
  ANTLR と tree-sitter はこの仕事を一切していない。

`examples/handwritten-target` にある手書きパーサは「生成物の目標形」を固定したもので、
その README 自身が「既存 parser と同一条件の比較計測はしておらず、速度比は主張しない」と書いている。

旧実装のベースラインは 2026-09-23 に確定させた
（[`docs/reports/2026-09-23-legacy-baseline.md`](docs/reports/2026-09-23-legacy-baseline.md)）。
以前この repo に載っていた root 1,928 ms / facade 76,720 ms は**無効**で、原因は共有
`target/classes` が 2026-08-28 のビルドのままで memo marker が 1 個も入っておらず
`Memoization.SAFE_FAILURES` が空振りしていたこと（×75）とヒープ設定（×2）だった。

文法は tinyexpression P4（124 規則）、入力は `complex.tiny`（332 byte）と
`complex-x64`（20,923 byte）。ホストは 32 core / WSL2（共有機。並走負荷あり）。

| 経路 | complex 認識 | complex AST | x64 認識 | x64 AST |
|---|---:|---:|---:|---:|
| 旧 Java 実装（基準） | 13.6 ms | 14.5 ms | 1,603 ms※ | 1,693 ms※ |
| ubnfc Java 生成 | **0.60 ms** | **0.64 ms** | **37.1 ms** | **44.4 ms** |
| ubnfc Rust 生成 | **0.066 ms** | **0.089 ms** | **4.52 ms** | **5.86 ms** |

- 旧 Java の計測条件: tinyexpression `c70416e1` を unlaxer master `ed4178e` の isolated
  maven repo で再ビルド、tinyexpression 自身の classpath 順、`Memoization.SAFE_FAILURES`、
  JDK 21.0.9、`-Xms512m -Xmx2g -Xss16m`（x64 は `-Xmx6g`）、warmup 300、5 run × 300 iteration、
  `taskset -c 30`。割当は 27.4〜27.8 MB/parse。
  ※ x64 の旧実装値はヒープ設定とホスト負荷に強く依存するので、報告自身が
  **「1.0〜1.8 秒の桁」までしか主張しない**と書いている。表の値はその中の実測中央値。
- ubnfc Java: [`docs/reports/2026-09-22-p4-java-perf-m5.md`](docs/reports/2026-09-22-p4-java-perf-m5.md)。
  割当は complex（AST）546 KB、x64 35.7 MB。旧比は **約 23 倍 / 約 40 倍前後**
  （x64 はヒープ設定が揃っていないのでこれ以上細かく主張しない）。
- ubnfc Rust: [`docs/reports/2026-09-23-p4-rust-floor.md`](docs/reports/2026-09-23-p4-rust-floor.md)。
  rustc 1.98.1 `--release`、`taskset -c 27`、全 variant を同一セッションで交互に 9 round、
  中央値と全 run の最小値。表の値は最小値。割当は complex（AST）509 回 / 77 KB。
- 旧 Rust 実装は criterion で complex 1.426 ms の記録があるが、**本作業では測り直していない**。
- 公開 facade `P4PreferredAstMapper.parseDetailed` の 10.8 ms は候補選択込みの副次値で、
  認識 / AST の対比には使わない。

**Java backend の追加ラウンド（issue #3 系列。上表は 2026-09-22 の M5 第 1 ラウンド時点、
以下は 2026-09-24 の 3 ラウンドで更新）**: 手書き目標形との比較
（[`2026-09-23-parser-speed-comparison.md`](docs/reports/2026-09-23-parser-speed-comparison.md) §7-1）
から出発し、生成物が観測（Match 木・capture 出現・診断）を要求されたときだけ作るよう
二段構えにした（D-023 の Java 版）。**認識専用の経路は手書き目標形を追い越した**が、
**AST 経路は M5 の目標に届いていない**——正直に分けて書く。

| ラウンド | 変更 | 結果 | 出典 |
|---|---|---|---|
| PR #14（issue #3） | `Session.combine` の逐次連結を線形化 | Operators 512 atom 認識 1.82→0.76 ms、割当 10.5→2.01 MB（手書き 2.09 MB を下回る）。**P4 complex/x64 は未達のまま** | `2026-09-24-java-backend-gap.md`（D-055） |
| PR #18（issue #15） | 認識専用経路で Match 木・出現表・診断観測を作らない | **Operators 512 atom 認識 0.207 ms（手書き目標形 0.350 ms の 0.59 倍、目標 ≤0.4 ms 達成）**。**P4 complex-x64 認識 13.71 ms（目標 ≤15 ms 達成）**。complex 認識 0.213 ms（目標 0.2 ms に 6% 届かず） | `2026-09-24-java-two-mode.md`（D-056） |
| PR #21（issue #19） | AST 経路にも観測の opt-in を広げる（出現表を作る条件を Match 木の条件から分離） | **Operators 512 atom AST 0.631 ms（issue の目標 0.693 ms を達成、手書き 0.560 ms の 1.13 倍）**。P4 complex AST 0.564→0.309 ms、x64 AST 50.0→28.0 ms。**M5 の AST 目標（complex ≤0.2 ms / x64 ≤15 ms）は未達**——同条件の認識専用が complex 0.220 ms / x64 17.5 ms なので、AST 層を 0 円にしても届かない | `2026-09-24-java-ast-path.md`（D-057） |
| PR #24（issue #17） | rule 本体の memo を「同じ位置へ再入し得る」規則だけに絞る（Operators は全 rule で外れ、P4 の塔は残る） | Operators 512 atom 認識専用 0.204→0.160 ms、割当 465,664→350,168 B。**Rust も同判定で 0.176→0.115 ms、P4 complex-x64 認識 5.67→4.68 ms（最小値）**。P4 は両 backend で悪化なし | `2026-09-24-memo-plan.md`（D-058） |

corpus（Java Pass 1,310 / Rust Pass 1,292、Fail 0）はこの 4 ラウンドいずれでも系統別表が既存報告と
byte 一致、IR golden も `plan.memo.entries` 以外は不変。**結論**: Java backend の性能目標は
「認識は達成、AST は未達」であり、§4 の記載もこれに合わせて分けてある。

### 3.3 言語の進化（UBNF v2）

`@ubnf: 1|2`（既定 1）でバージョンを門にした（D-038）。**v1 は凍結**で、既存の 24 fixture・
P4・corpus 1,212 ケース・差分ファジング・self-host 固定点が永久にこれを守る。
v2 の表層機能は front が v1 の正規形へ脱糖するので、backend には門しか増えない。

| 機能 | 状態 | 決定 |
|---|---|---|
| 到達不能候補の検出（v1 でも警告） | 実装済み | D-037 |
| 左再帰ルール（候補順＝優先順位、ANTLR4 方式） | 実装済み | D-038 / D-039 |
| リテラルの単語境界（v2 既定） | 実装済み | D-040 |
| 言語中立な token 語彙（36 名前）＋ 明示 `extern` | 実装済み | D-041 / D-042 |
| パラメータ化ルール `Name<P1,...,Pn>`（規則本体の雛形。resolve 前にその場で展開、`%inline` 方式） | 実装済み | D-043 / D-044 |
| アノテーションの統廃合（P1 未知注釈警告・P2 `@backref` 分割・P3 `@interleave` 統合・P5 `@predictiveChoice` 非推奨・P7-1/P7-3） | 実装済み。**P7-2 は誤りと判明**（Java にも上限検査があった）。**P4・P6・P7-4 は未着手** | D-045〜D-049。棚卸し全体は `docs/design/07-annotation-inventory.md`（提案 P1〜P7 のうち実装済みの範囲を明記） |

意味述語（ANTLR の `{...}?`）は v2 でも**入れない**。終端が「(入力, 位置) の純粋述語」である
性質に memo・候補除外・字句キャッシュの健全性が乗っているため。文脈依存が要る場面は
`@backref` / `@typeof` / `@scopeTree` で宣言的に扱う。

左再帰の効果について:「P4 を v2 で書き直すと 124 → 119 規則（−5）」であり、
**価値は規則数ではなく優先順位が宣言的になること**（初版の「同じ塔を 4 回書いている」は
誇張だったと実装後に訂正されている）。

### 3.4 堅牢性

- **不正な token 宣言・注釈値を診断するようにした**（**D-050**、
  [PR #6](https://github.com/opaopa6969/ubnfc/pull/6)）。2026-09-23 のクロスチェック
  （`2026-09-23-parser-feature-correctness-comparison.md` §3.1 ①）で見つかった、
  ubnfc だけが受理し他の 2 実装は拒否する不正宣言 7 件のうち `CHAR_RANGE('','a')` は
  `lower/tokens.rs` の `unwrap()` で **panic（exit 101）していた**。新しい `E-TOKEN-*` 診断
  6 種で拒否し、`unwrap` 自体も除いて利用者入力で panic しないことを二重に保証している。
  corpus・IR golden・self-host 固定点は 1 byte も動いていない。
- **深い文法連鎖でスタックオーバーフローしない**（**D-054**、
  [PR #9](https://github.com/opaopa6969/ubnfc/pull/9)）。規則参照の連鎖長がそのまま深さになる
  4 つの再帰的走査（SCC・scope 効果の要約・順序付き選択の左端・必須リテラル接頭辞）が
  16,000 段の連鎖で **exit 134** していたのを、すべて明示スタックへ置き換えた（上限は設けない）。
  `validate::operators::find_cycles` も互いに左端参照する規則で階乗時間になっていたのを
  Tarjan の SCC + 成分内 BFS に直し、120 秒超だったものを規則ごと 1 件の線形時間にした。
  IR golden・self-host 固定点・corpus は不変（`W-LEFT-RECURSION` の**件数**だけ変わり得るが、
  文言・severity・順序は同じ）。詳細は
  [`docs/reports/2026-09-24-deep-grammar-robustness.md`](docs/reports/2026-09-24-deep-grammar-robustness.md)。

---

## 4. できていないこと

- **Java backend の AST 性能目標は未達**（§3.2 の 3 ラウンドを経てもなお）。**認識専用経路は
  目標を達成済み**（Operators 512 atom 0.207 ms、P4 complex-x64 13.71 ms ≤15 ms。PR #18/D-056）が、
  AST 経路は complex 0.309 ms（目標 ≤0.2 ms）、x64 28.0 ms（目標 ≤15 ms）で未達
  （PR #21/D-057、`2026-09-24-java-ast-path.md`）。**測定で分かった理由は AST の組み立てではなく
  「認識そのもの」**——同条件の認識専用が complex 0.220 ms / x64 17.5 ms なので、AST 層を 0 円に
  しても届かない。残作業（`MemoTable` の slot 配置、`ScopeStore` の取消し記録、trivia の速い経路）は
  [issue #22](https://github.com/opaopa6969/ubnfc/issues/22) に切り出し済み（本 PR 時点で別エージェントが
  `ubnfc-java/**` で作業中）。Rust backend は両目標とも達成済み。
- **性能の内訳が割り切れていない**。Rust の complex 認識 66 µs のうち約 55 µs は
  「切ると受理が変わる機構の束」で、**このホストに `perf` が無いので分解できていない**
  （`2026-09-23-p4-rust-floor.md` §6）。以後の改善は計数（呼出し回数・確保回数）を
  一次証拠にしている。
- **計測して却下したもの**: 認識での event arena 削減（候補3、complex −1.5〜−3.0%・AST 0%）、
  AST の arena / SoA 化（候補5。公開型 `Ast` の破壊的変更と引き換えに取れるのは 23 µs の一部だけ）。
  `const DIAG` の単相化は一度「別 binary にする価値なし」と見送ったあと、床を詰める段で
  採用し直している（D-036）。判断が変わった経緯も報告に残っている。
- **差分ファジングの残り 10 根本原因**（入れ子ノード span の末尾 trivia、`@enum` を演算子の
  operand にしたときの Rust の mapping 失敗、成功解析での Java の投機的 `syntax` 診断、
  driver の wire キーの不一致など）は記録済みで未修正。
- **差分ファジングが見ていない範囲**: 両 backend が同じように間違っている場合、左再帰の
  runtime 差、`REGEX` token、`@import` の非最小形、大きな入力、`@root` 以外の entry。
- **self-host の 5 件の既知乖離 + ブロックコメントの字句の門**（§3.1）。うち
  `known/keyword-boundary.ubnf` は D-040 の語境界そのもので、`ubnf.ubnf` を `@ubnf: 2` にすると
  解消する（受理 19 → 20、他 4 件は不変）。self-description を v2 にするかは別の判断なので、
  表は v1 のままの実測値。3 段以上の dotted 参照（旧「8 件目」）は
  [unlaxer-parser #285](https://github.com/opaopa6969/unlaxer-parser/pull/285) +
  [ubnfc #20](https://github.com/opaopa6969/ubnfc/pull/20) で解消済み。
- **corpus の未観測項目**（Java 59 / Rust 75）。`/evaluationError` 32、`/scope/resolved` 20、
  `/operations` 18、`/rollback` 10 ほか。欠落であって一致ではない。
  とくに **`scope-annotations` 系統 14 件は Rust 側が丸ごと未観測**（Java は 14 件とも Pass）で、
  欠けているのは `/scope/resolved`（現在の lookup 表）20 件。Rust runtime の `scope` は
  `{mode, events}` しか持たない。`declarations` / `references` / `diagnostics` は top-level へ
  出ているので harness が fallback するが、wire 形の不一致は残っている（差分ファジング B-1）。
- **corpus のうち IR / 生成の段階で拒否されるもの**: `E-PRECEDENCE-ORDER` 24 ケース、
  `E-LOWER-COMMON-FIELD` 2、`E-LOWER-SUM-EVAL` 1、`E-IMPORT-CYCLE` 1、
  Rust backend の非正則 `REGEX` 2（Java は `java.util.regex` へ委譲できるが Rust は生成時エラー）。
  多くは「拒否すること自体が期待値」だが、その分だけ実行時の観測が無い。
- **P4 の完全互換は達成していない**。16 fixture 中、Formula 入口で 9 件を全消費、
  意図的な不正入力 3 件と comparison-heavy 系 4 件を拒否する（両 backend で同じ）。
  comparison-heavy は `BooleanExpression` 入口なら全消費する。
- **P4 の期待値は（32 件の仕様由来ケースを除き）観測由来のまま**。tinyexpression の
  `SPEC.md` 等は自己申告で「実装ベースの記述」であり（UBNF の `ubnf-syntax.md` のような
  先に書かれた規範ではない）、文書間で優先順位の矛盾もある。判定可能な 32 件のうち
  25 件は R1（ubnfc p4-rust）・R2（tinyexpression-rs）とも仕様と一致、残り 4 件は文書間の
  矛盾で未定、3 件は駆動不能（[`docs/reports/2026-09-24-spec-derived-oracle.md`](docs/reports/2026-09-24-spec-derived-oracle.md) §6）。
- **インクリメンタル解析は無い**。生成物は毎回全文を読み直す 1 回きりの解析器で、
  差分再解析・部分木の再利用は設計にも実装にも無い。
- **LSP は opt-in の feature**（D-060、issue #28）。既定のビルドには入らない（Maven profile `lsp`）。
  有効にすると `ubnfc-lsp --emit-lsp` が生成パーサを包む LSP サーバを**別パッケージへ追加**する。
  生成パーサ本体は依存ゼロのままで、lsp4j に依存するのは包み側だけ。**段階 2 の capability は診断のみ**
  （構文エラーと未定義参照。completion / hover / definition / references は未実装）。
  打鍵ごとに全文を再解析する（[`docs/reports/2026-09-24-lsp-vsix-stage1.md`](docs/reports/2026-09-24-lsp-vsix-stage1.md)
  §6 に実測。P4 の 325 code point で 0.45 ms、20,923 code point で 15.6 ms）。DAP は未実装。
- `@doc`、`@precedence` の `level` 値、`@leftAssoc`/`@rightAssoc` の `associativity` 値は
  front が IR に載せるが**どちらの backend も読まない**（`07-annotation-inventory.md`）。

---

## 5. ビルドと実行

前提: Rust 1.85 以上（実測 1.98.1）、JDK 21 以上、Maven。すべてオフラインで動く。

### 5.1 ジェネレータを建てる

```sh
# Rust 側（front CLI と Rust backend）→ target/debug/{ubnfc,ubnfc-rust}
cargo build --offline --bin ubnfc --bin ubnfc-rust

# Java backend の jar → ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar
(cd ubnfc-java && mvn -o -q -DskipTests package)

export PATH="$PWD/target/debug:$PATH"   # 以下 `ubnfc` を直に呼ぶ
```

`ubnfc gen --target java` は `ubnfc-java/target/ubnfc-java-*.jar` を自動で探す
（`UBNFC_JAVA_JAR` / `UBNFC_RUST_BIN` で明示指定も可能）。

> **D-033**: Java backend の driver テンプレートと emitter は **jar の resource** なので、
> `ubnfc-java/` を触ったら必ず `mvn -o -q -DskipTests package` をやり直すこと。
> 古い jar のまま差分ファジングを回すと「Java だけが契約を破っている」ようにしか見えない
> 偽陽性が出る。ハーネスはこれを差分ではなく discard として落とす鮮度検査を持っている。

### 5.2 文法をコンパイルする

```sh
ubnfc validate --grammar X.ubnf [--strict] [--format text|json]
ubnfc ir       --grammar X.ubnf [--out FILE|-] [--strict] [--extern-first-chars scanners/first-chars.json]
ubnfc gen      --grammar X.ubnf --target java|rust --out DIR
```

exit code: `0` 成功 / `1` 文法エラー / `2` CLI・入出力エラー / `3` backend が見つからない /
`5` `--strict` で警告あり。

手を動かす手順は [`docs/getting-started.md`](docs/getting-started.md)。

### 5.3 テスト

```sh
cargo test --workspace --offline          # Rust 側すべて
cd ubnfc-java && mvn -o -q test           # Java backend
cd examples/p4-java && mvn -o -q test     # P4 Java 実行例
```

self-host の固定点だけを見るなら:

```sh
cargo test -p ubnfc-selfhost --offline --test parity --test regenerate
cargo test -p ubnfc-front    --offline --test selfhost_conversion --test selfhost_pipeline
```

### 5.4 適合性ハーネス（corpus）

corpus は sibling の `unlaxer-parser` から読む（既定は `../unlaxer-parser`、
`UNLAXER_PARSER_DIR` で上書き）。

```sh
cargo build --offline --bin ubnfc-harness
./target/debug/ubnfc-harness list
./target/debug/ubnfc-harness run-all --backend java --report out.md --json out.json [--jobs N]
./target/debug/ubnfc-harness run-all --backend rust --report out.md --json out.json
```

### 5.5 差分ファジング

**先に `ubnfc-java` の jar を package し直すこと**（D-033）。

```sh
cargo build --offline --bin ubnfc --bin ubnfc-rust
(cd ubnfc-java && mvn -o -q -DskipTests package)

cargo test -p ubnfc-fuzz --offline        # CI 用の固定 seed campaign（数十秒）
cargo run -q -p ubnfc-fuzz --offline -- \
  --seeds 320 --inputs 24 --jobs 3 --shrink-builds 10 --minutes 60 \
  --findings crates/ubnfc-fuzz/findings --report out.md --json out.json
cargo run -q -p ubnfc-fuzz --offline -- --start-seed 1 --dump 8   # 生成文法だけ見る
```

seed から文法と入力を完全に再現する。

### 5.6 ベンチ

```sh
cargo run -q --offline --release -p ubnfc-p4-rust --example bench
cd examples/p4-java && mvn -o -q -DskipTests package && bash scripts/benchmark.sh
```

旧実装のベースラインを取り直す手順は `2026-09-23-legacy-baseline.md` §4。

---

## 6. リポジトリの構成

| パス | 中身 |
|---|---|
| `crates/ubnfc-front` | `.ubnf` → 字句 / 構文 / 解決 / 検証 / 脱糖 / 意味論 lowering / 解析 / 最適化計画 → IR JSON。`ubnfc` CLI もここ |
| `crates/ubnfc-rust` | IR → Rust ソース。`ubnfc-rust` CLI |
| `ubnfc-java` | IR → Java ソース。Maven モジュール。runtime テンプレートと driver テンプレートを resource に持つ |
| `crates/ubnfc-harness` | corpus loader、生成 → build → 実行 → 比較の runner、canonical 差分 |
| `ubnfc-lsp` | IR → 生成パーサを包む LSP サーバのソース。**opt-in**（root `pom.xml` の Maven profile `lsp`）。`--emit-lsp` を明示したときだけ出す（D-060） |
| `crates/ubnfc-fuzz` | 2 backend の差分ファジング。`findings/` に最小再現 |
| `crates/ubnfc-selfhost` | 生成 Rust frontend による self-host 経路と固定点テスト。`src/generated/` は生成物を commit |
| `crates/ubnfc-rt-template` | 両 backend が写す runtime の原本（input / cursor / scope / memo / diag） |
| `ir/schema`, `ir/fixtures`, `ir/examples` | Grammar IR v1 の JSON Schema、22 の golden IR、解説用の例 |
| `corpus/oracle` | 旧実装から採取した独立 oracle 107 ファイル（+ `divergences.json`・採取ツール）。`ubnfc-harness list` の「oracle」107 ファイル / 398 ケース |
| `corpus/spec` | 規範文書から導いた期待値（UBNF 129 ケース / P4 32 ケース / JSON 283 ケース）。実装の観測値を使わない。`corpus/spec/README.md`、issue #5・#26 |
| `scanners/` | Extern token の scanner registry（`first-chars.json` と Java / Rust 実装） |
| `examples/handwritten-target` | 生成物の**目標形**を手書きで固定した Operators / Power（Java・Rust） |
| `examples/p4-java`, `examples/p4-rust` | tinyexpression P4（124 規則）の生成物を commit した実行例とベンチ |
| `examples/p4-vscode` | P4 の VS Code 拡張（生成パーサ + 生成 LSP サーバ + VSIX）。`scripts/build-vsix.sh` で組む |
| `examples/p4-v2` | P4 を UBNF v2（左再帰ルール + 中立 token 語彙）で書き直したもの。124 → 119 規則、式の塔 8 → 3 |
| `examples/json`, `examples/arith` | 同じ言語を `.ubnf` / ANTLR `.g4` / tree-sitter `grammar.js` の 3 記法で書いたもの（JSON = RFC 8259、算術式 = Operators / Power）。外部比較の材料。issue #26 |
| `examples/p4-antlr`, `examples/p4-treesitter` | **実用文法** P4（119 規則）を ANTLR `.g4` / tree-sitter `grammar.js` へ移植したもの。規則対応表と逸脱の一覧は `examples/p4-antlr/README.md`。issue #27。同じ言語を各ツールの流儀で書き直した `TinyExpressionP4Idiomatic.g4` / `grammar.idiomatic.js` も同じディレクトリ（issue #39） |
| `benchmarks/` | 速度比較のドライバ。`scripts/prepare-external.sh` 以下が ANTLR4 / tree-sitter との比較、`scripts/p4-external.sh` が P4 での同じ比較（取得はどちらも `scripts/fetch-external-tools.sh`） |
| `docs/` | 設計・決定・報告。下記 |

## 7. 設計と決定はどこにあるか

| 文書 | 内容 |
|---|---|
| [`docs/design/03-design-v1.md`](docs/design/03-design-v1.md) | **正本**。全体構成、実行意味論、IR 契約、結果 API、マイルストーン、**§8 の決定表 D-001 … D-058**（本 commit 時点で 57 行。D-034 は採番の欠番。行は増え続けている） |
| [`docs/design/04-cursor-and-diagnostics-contract.md`](docs/design/04-cursor-and-diagnostics-contract.md) | 二重カーソルの遷移表、診断の主位置、self-host、左再帰、IR スキーマの要点 |
| [`docs/design/05-selfhost.md`](docs/design/05-selfhost.md) | self-host 経路と固定点、既知の 5 差分 + ブロックコメントの字句の門 |
| [`docs/design/06-ubnf-v2.md`](docs/design/06-ubnf-v2.md) | 言語の弱点と v2 の計画・実装状況 |
| [`docs/design/07-annotation-inventory.md`](docs/design/07-annotation-inventory.md) | 26 の表層アノテーションの棚卸しと統廃合案（順位付き P1〜P7）。**P1/P2/P3/P5 と P7-1/P7-3 は実装済み（D-045〜D-049）、P7-2 は誤りと判明、P4/P6/P7-4 は未着手**（§3.3） |
| [`docs/design/02-facts-tokens-bench-corpus.md`](docs/design/02-facts-tokens-bench-corpus.md) | 組込み token の受理範囲、ベンチ fixture、corpus の形式 |
| [`docs/compat-matrix.md`](docs/compat-matrix.md) | 機能 × 観測面の oracle 対応表。「oracle なし」を空欄と区別する |
| [`docs/harness-driver-protocol.md`](docs/harness-driver-protocol.md) | 生成物と手書き目標が共有する JSON Lines の driver 契約 |
| [`docs/decisions/`](docs/decisions/) | 疑似コードや根拠が長い決定（D-004 precedence climbing、D-018 未決乖離 15 件） |
| [`docs/reports/`](docs/reports/) | 全計測・全 corpus 実行の記録。**性能を語るときはここの数字と条件を引く** |
| [`docs/reviews/`](docs/reviews/) | 両 backend への敵対的レビュー |

report は日付 + 話題で名前が付いている。性能の出発点は
`2026-09-23-legacy-baseline.md`（旧実装の確定ベースライン）、
`2026-09-23-p4-rust-floor.md`（Rust の到達点）、
`2026-09-22-p4-java-perf-m5.md`（Java の到達点）の 3 本。

---

## 8. 用語

| 語 | 意味 |
|---|---|
| consumed / matched | 二重カーソル。matched は「照合が到達した位置」で、**最遠到達位置ではない**（消費や transaction 開始で後退する） |
| Known（既知乖離） | 旧実装のバグを意図的に再現しないと決めた差。`new-contract:<名>` で登録する |
| 期待拒否 Pass | 文法を拒否することが期待値のケース。Pass とは別列に排他的に数える |
| 未観測 | 比較に差分は無いが観測項目が欠けている状態。Pass に読み替えない |
| fast mode | `ParseOptions.diagnostics = false`（既定）。診断を記録せず、必要なら同じ入力を決定的に再解析する（D-023） |
