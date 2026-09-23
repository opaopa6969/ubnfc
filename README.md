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

## 3. 現状（2026-09-23 時点）

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
`ubnf.ubnf` が記述する構文だけを使う 19 件は `UbnfFile` が全 field / span まで一致する。
残り 6 件は**手書き frontend だけが持つ互換拡張**（dotted mapping class、`@eval` の `=` 区切り、
3 成分以上の namespace、setting block 内の二重引用符、`@declares` description の非 STRING capture、
annotation keyword の識別子境界）で、生成パーサが表の byte offset で拒否する。
self-host を通すための文法 / IR の加工はしていない。

### 3.2 性能

**比較の基準は旧実装（`unlaxer-parser` + `tinyexpression` P4）だけである。**
ANTLR / tree-sitter とのベンチマークは**存在しない**（オフライン環境で入手できないため）。
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
| パラメータ化ルール `List<X, Sep>` | 本 commit 時点では未実装（`crates/ubnfc-front/src/desugar.rs` で作業中） | `06-ubnf-v2.md` §4 |
| アノテーションの統廃合 | 棚卸しと提案 P1〜P7 のみ（**実装ではない**） | `docs/design/07-annotation-inventory.md` |

意味述語（ANTLR の `{...}?`）は v2 でも**入れない**。終端が「(入力, 位置) の純粋述語」である
性質に memo・候補除外・字句キャッシュの健全性が乗っているため。文脈依存が要る場面は
`@backref` / `@typeof` / `@scopeTree` で宣言的に扱う。

左再帰の効果について:「P4 を v2 で書き直すと 124 → 119 規則（−5）」であり、
**価値は規則数ではなく優先順位が宣言的になること**（初版の「同じ塔を 4 回書いている」は
誇張だったと実装後に訂正されている）。

---

## 4. できていないこと

- **Java backend の性能目標は未達**。目標は complex ≤ 0.2 ms（AST）/ x64 ≤ 15 ms で、実測は
  0.64 ms / 44.4 ms。届かせるには Match 木の構築を event arena へ置き換える構造変更（Rust が
  M5 round 1 で行ったもの）が要る。Rust backend は両目標とも達成済み。
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
- **self-host の 6 件の既知乖離**（§3.1）。うち `known/keyword-boundary.ubnf` は D-040 の
  語境界そのもので、`ubnf.ubnf` を `@ubnf: 2` にすると解消する（受理 19 → 20、他 5 件は不変）。
  self-description を v2 にするかは別の判断なので、表は v1 のままの実測値。
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
| `crates/ubnfc-fuzz` | 2 backend の差分ファジング。`findings/` に最小再現 |
| `crates/ubnfc-selfhost` | 生成 Rust frontend による self-host 経路と固定点テスト。`src/generated/` は生成物を commit |
| `crates/ubnfc-rt-template` | 両 backend が写す runtime の原本（input / cursor / scope / memo / diag） |
| `ir/schema`, `ir/fixtures`, `ir/examples` | Grammar IR v1 の JSON Schema、22 の golden IR、解説用の例 |
| `corpus/oracle` | 旧実装から採取した独立 oracle 107 ファイル（+ `divergences.json`・採取ツール）。`ubnfc-harness list` の「oracle」107 ファイル / 398 ケース |
| `scanners/` | Extern token の scanner registry（`first-chars.json` と Java / Rust 実装） |
| `examples/handwritten-target` | 生成物の**目標形**を手書きで固定した Operators / Power（Java・Rust） |
| `examples/p4-java`, `examples/p4-rust` | tinyexpression P4（124 規則）の生成物を commit した実行例とベンチ |
| `examples/p4-v2` | P4 を UBNF v2（左再帰ルール + 中立 token 語彙）で書き直したもの。124 → 119 規則、式の塔 8 → 3 |
| `docs/` | 設計・決定・報告。下記 |

## 7. 設計と決定はどこにあるか

| 文書 | 内容 |
|---|---|
| [`docs/design/03-design-v1.md`](docs/design/03-design-v1.md) | **正本**。全体構成、実行意味論、IR 契約、結果 API、マイルストーン、**§8 の決定表 D-001 … D-042**（本 commit 時点で 41 行。D-034 は採番の欠番。他のエージェントが行を追加中） |
| [`docs/design/04-cursor-and-diagnostics-contract.md`](docs/design/04-cursor-and-diagnostics-contract.md) | 二重カーソルの遷移表、診断の主位置、self-host、左再帰、IR スキーマの要点 |
| [`docs/design/05-selfhost.md`](docs/design/05-selfhost.md) | self-host 経路と固定点、既知の 6 差分 |
| [`docs/design/06-ubnf-v2.md`](docs/design/06-ubnf-v2.md) | 言語の弱点と v2 の計画・実装状況 |
| [`docs/design/07-annotation-inventory.md`](docs/design/07-annotation-inventory.md) | 26 の表層アノテーションの棚卸しと統廃合案（**提案であって実装ではない**） |
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
