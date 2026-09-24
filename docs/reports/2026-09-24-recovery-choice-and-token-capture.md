# `@recovery` の選択内到達（#71）と token の直接 capture（#75）

どちらも stax 文法（#66 stage 1、[`2026-09-24-stax-stage1.md`](2026-09-24-stax-stage1.md) §5）で見つかった。
1 本の PR で 2 つの commit に分けてある（A: #71 / D-073、B: #75 / D-074）。

## A. `@recovery` が順序付き選択の末尾でない候補から到達する（#71 → D-073: 警告）

### 再現（修正前）

stax の `WordDef` に `@recovery(sync=';')` を付け、20 本を両 backend に流した。

- 受理・canonical AST・スコープ診断は **20 本とも両 backend で一致**
- 両 backend とも 06 / 07 / 08 / 09 / 14 / 16 の宣言がスコープに入らない（未定義シンボル警告が出る）
- 両 backend とも `17-bad-syntax.stax` を受理
- `ubnfc validate --strict` は exit 0（何も言わない）

### 判定: 契約の穴ではなく文法の危険な書き方なので警告する

- §3.10 の回復規則は「子 parser が `start` で失敗したとき」だけを条件にしていて、試行の失敗と誤りを区別しない。
  両 backend はこの規則を同じように実装している。backend 間の差でも契約違反でもない。
  受理が変わるのは回復の定義そのもの（corpus の `recovery-sync/1` の `bad;ok;` も回復を通って受理される）。
- 「commit 後だけ回復する」を契約にするには commit 点を定義しなければならないが、UBNF には cut が無い。
  反復 `{ Stmt }` の各回も試行なので、候補だけを特別扱いすると、反復で包む正規の使い方
  （corpus の `recovery-sync` / `recovery-skip`、旧 `SyncPointRecoveryParser`）と一貫しない。
- 旧実装も同じ挙動で、D-017 の「再現しない既存バグ」にも挙がっていない。

**実装**: `crates/ubnfc-front/src/analysis/reachability.rs` の step 4 に `W-RECOVERY-IN-CHOICE`
（category CHOICE）を追加。ordered 選択の末尾でない候補を「試せば必ず同じ位置で最初に呼ばれる」子だけで
辿る（列の先頭要素、ordered / longest 選択の第 1 候補、group、省略可能、UNTIL の無い反復、区切り列の要素、
規則参照の本体）。途中で `@recovery` の適用 site に当たったら警告する。辿る経路は 1 本道で、循環したら打ち切る。
predictive は候補除外で第 1 候補を飛ばすことがあるので辿らない。backend は変更なし。

| 対象 | `W-RECOVERY-IN-CHOICE` |
|---|---|
| stax + `@recovery` on `WordDef` | 1 件（`TopLevel`）、`--strict` で exit 5 |
| repository + unlaxer-parser の `.ubnf` 91 本 | 0 |
| P4（`every_repository_grammar_reports_exactly_the_recorded_choice_warnings` に組込み） | 0 |
| corpus の recovery 系 4 文法（選択なし） | 0 |
| 生成器の乱択文法 v1 帯 / v2 帯 各 1,000 件 | 各 24 件（`@recovery` を含むのは 354 / 362 件） |

D-037 の掃き出し件数（`W-CHOICE-*` だけを数える）は変わらない。正例 3 形と負例 7 形を
`crates/ubnfc-front/tests/choice_reachability.rs` に追加した。

## B. token の直接 capture `NUMBER @x`（#75 → D-074: 両 backend とも原文の文字列）

### 原因

front の `lower/captures.rs` が、数値 token（`NumberParser` / `DigitParser` / `NUMBER`）の**直接** capture だけを
`ValueType::Number { java: int32Exact, rust: lexeme }` にしていた。旧 Java の「数値 token の型境界」
（`generators.md:104-123`）と D-022 をそのまま写したもので、**言語別の型を IR に書いたため、差が仕様扱いになっていた**。
差分ファザーも Java の int を字句へ射影し、`NumberFormatException` を免除していたので、乱択文法の
`T0 @v`（数値 token の直接 capture）が出していた差を消していた。

### 正本

token の直接 capture の値は、数値・識別子・引用符付き文字列・extern scanner のどれでも
**capture 範囲の原文（文字列）**とする。group を挟んだ capture と同じ扱い。型付けは `@mapping` を使う側が行う。
理由は D-074 の行に書いた（数値 token の受理範囲は int より広い、group を挟むかどうかで型が変わるのは文法の読み手から見えない、Rust はもともと字句）。

### 変更

- **front だけ変えた**: 特例を削除。**Java / Rust backend のコードは変えていない**。Java の生成物は IR の値型が
  変わった結果、`int` field が `String` になる。`ValueType::Number` は IR schema に残す（旧 IR を読めるようにするため）。
- **生成 API の非互換**: 数値 token を直接 capture していた Java 利用者の field は `int` から `String` に変わる。
  P4 / tinyexpression では該当 0 件。`ubnf.ubnf` では `PrecedenceAnnotation.level` の 1 field。
- **harness**: D-022 の投影を**両 backend に**適用する。numeric-capture 系統も両 backend を `rustLexeme` で比較する。
  `common-field-type/1` は拒否契約から外し、`new-contract:direct-token-capture-text` として登録した（後述）。
- **差分ファザー**: `observe.rs` から Java だけに掛けていた射影と免除を削除し、厳密比較にした。
  generator に直接 capture の seed 帯 `DIRECT_CAPTURE_SEED_BASE = 1_000_000` を追加（v1 帯と v2 帯）。
  この帯は乱数を引かずに token 3 本と
  `Direct ::= DN @dn ':' ( DN ) @dg [ DI @di ] { ',' DN @dl } DS @ds` を足し、根の `@mapping` の field に載せる。
  v2 の奇数 seed では `DS` が `extern 'SingleQuotedParser'` になる。既存の帯の生成物は変わらない。
- **stax**: `NumLit ::= NUMBER @text ;` に戻した。`examples/stax/golden/{02-arith,15-use-vec}.stax.ast.json` を
  `crosscheck.py` が両 backend と突き合わせる。

### 回帰 seed

旧 front（`captures.rs` だけを戻した状態）で `--start-seed 1000001 --seeds 4` を回すと、
**seed 1000002** で次が出た（縮小後の入力は `iあ0:1''` と `iあ1:1,2''`）。

- `diff:/ast/fields/direct/fields/dn`: java=`0` rust=`"0"`
- `diff:/ast/fields/direct/fields/dl/*`: java=`2` rust=`"2"`

固定した回帰は 2 つ。

- `crates/ubnfc-fuzz/tests/direct_token_capture.rs`: 上の形を v1 / v2 / v2 extern の 3 文法で流し、
  符号・先頭ゼロ・小数・指数・int 範囲外も入れる。**値の型（文字列）まで確認**する。
  旧 front では `/ast/fields/direct/fields/dn` で落ちることを確認した。
- `tests/campaign.rs` の g: 直接 capture 帯の固定 seed 4 本（1000001 / 1000003 / 1100001 / 1100002）。

seed 1000002 は乱択部分に別の既存差 `diff:/ast/fields/tail/*` を持つ。直接 capture とは無関係で、
修正後も出るので issue #78 に分けた。この seed は固定帯から外してある。

## 検証

`cd ubnfc-java && mvn -o -q -DskipTests package` を先に実行した。

| 項目 | 結果 |
|---|---|
| corpus Java（`ubnfc-harness run-all --backend java`） | **Pass 1,310** / 期待拒否 Pass 27 / Known 29 / **Fail 0** / 未観測 59 |
| corpus Rust | **Pass 1,292** / 期待拒否 Pass 29 / Known 29 / **Fail 0** / 未観測 75 |
| 変化の内訳 | Pass は不変。`common-field-type/1` だけが「期待拒否 Pass」から Known に移った（Java 28→27、Rust 30→29 / Known 28→29）。数値 capture の oracle 13 件などは、両 backend に字句の投影を掛けたまま Pass |
| canonical JSON golden 13 件（P4、`java_and_rust_goldens_are_identical_without_normalization`） | 不変・緑 |
| IR golden | `ubnf.ubnf` は `PrecedenceAnnotation.level` の値型だけが変わった（number → text、3 箇所）。`ir/examples/stax.ir.json` は `NumLit` の書き換えで変わった。P4 と他の fixture は byte 一致 |
| `examples/stax/scripts/crosscheck.sh` | 20 本とも両 backend 一致、golden 2 本一致 |
| `cargo test --workspace --offline --no-fail-fast` | 緑（self-host の `parity` / `regenerate` を含む） |
| `cargo test -p ubnfc-fuzz`（固定 seed a〜g、直接 capture の targeted、D-037 の掃き出し） | 緑 |
| `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --check` | 緑 |
| `mvn -o test` / `mvn -o -Plsp test` | 緑 |
