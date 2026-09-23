# 順序付き選択の到達不能候補の検出（D-037）

> 2026-09-23。`06-ubnf-v2.md` §2 の実装。`crates/ubnfc-front/src/analysis/reachability.rs`。
> 警告のみ。受理・AST・診断・IR は 1 byte も変わらない。`--strict` で既存機械がエラー化する。

## 0. 直したかった弱点

UBNF の `|` は PEG の順序付き選択で、**最初に成功した候補を採り、継続が失敗しても別候補へ戻らない**。
候補順を誤ると候補が黙って死ぬ。ANTLR4 は曖昧性を、LALR は衝突を報告するが、UBNF は何も言わなかった。
実例が P4 の `comparison-heavy`（`docs/reports/2026-09-22-p4-java-bench.md`）で、`Formula` 入口で
拒否される原因は `Expression` の候補順だった。

## 1. 実装した検査と証明

対象は `selection = ordered` の Choice だけ。`longest` は全候補を共通開始状態から試すので順序で
死なず、`predictive` は候補除外の証明側（`analysis/choices.rs`）で扱う。
recovery 適用 site を含む候補、Extern / regex / 複数 trivia policy を跨ぐ経路は**不明**として出さない
（`crates/ubnfc-rust/src/predict.rs` の候補除外と同じ保守性）。

記法: 候補列 `A₁ | … | Aₙ`、選択の開始位置 `p`。PEG なので「`Aₖ` が `p` で成功したら
`A₍ₖ₊₁₎…` は走らない」。よって **`Aₗ` が死ぬ ⟺ `Aₗ` が成功しうる位置で必ず先行候補が成功する**。

### `W-CHOICE-NULLABLE-NOT-LAST`（厳密）

末尾でない候補 `Aₖ` の `zeroConsumeCondition` が `always`。

- `always` は「無条件にゼロ幅成功できる」。`analysis/facts.rs` の `zero_union` / `zero_seq` は
  Optional / `min=0` の Repeat・Separated / `EMPTY` からしか `always` を作らず、Seq は全要素が
  `always` のときだけ、Choice は 1 つでも `always` があるときに `always` になる。
  いずれも PEG で**必ず成功する**構成そのもの。先読み（`predicate`）・EOF（`atEof`）・
  不明（`unknown`）は別の値なので入らない。
- したがって `Aₖ` は全位置で成功し、`A₍ₖ₊₁₎…Aₙ` は一度も走らない。証明は完全（下界でない）。

### `W-CHOICE-PREFIX`（厳密）

`Aₖ` が case-sensitive リテラル `L`（trivia 挿入なし）で、`Aₗ`（`l > k`）の**必須先頭リテラル**
`P` が `L` で始まる。必須先頭リテラルは「その式の全一致が必ずその文字列で始まる」下界で、
リテラル・Group・RuleRef・Seq（先頭からリテラル要素が続く限り連結）・Choice（全候補の最長共通接頭辞）・
`min ≥ 1` の Repeat / Separated だけを辿って求める。途中で trivia 境界を跨いだら出さない。

- `Aₗ` が `p` で成功する ⇒ 入力は `P` で始まる ⇒ `L` で始まる ⇒ `Aₖ` が `p` で成功する。
- よって `Aₗ` は決して選ばれない。この含意は D-002（語境界なし。`ExprNode.literal.wordBoundary` は
  IR v1 で固定値 `false`）に依存する。`06-ubnf-v2.md` §3 が v2 で語境界を既定にしたら、
  境界付きリテラルはこの証明から外す必要がある（境界があれば `'var'` は `variable` に一致しない）。
- `'a' | 'ab'` はこの特別な場合で、**「怪しい」ではなく死んでいる**
  （PEG は `'a'` の成功で確定し、継続の失敗で `'ab'` へ戻らない）。

### `W-CHOICE-UNREACHABLE`（厳密）

次のどちらか。

1. **構造同一**: `Aₗ` の式木が `Aₖ` と同型（rule 呼出しは symbol 名で比較して本体へは降りない。
   trivia 挿入計画も一致を要求。capture / 診断ラベルは成功可否を変えないので無視）。
   同じ parser なので `Aₗ` が成功する位置では `Aₖ` も成功する。
2. **左端呼出しが必須**: `Aₖ = RuleRef(R)` で、`Aₗ` の成功が常に「同じ位置での `R` の成功」を含む。
   Seq は全要素の成功が必要なので先頭要素へ、Choice は全候補が必須とする rule の共通部分へ、
   Group / Lookahead は子へ、`min ≥ 1` の Repeat（terminator なし）/ Separated は本体へ降りる。
   Seq は「先行要素が一切消費しない」間だけ次の要素へ進む（`Facts.consumes`）ので、降下中に開始位置は動かない。
   `Aₗ` 成功 ⇒ `R` が `p` で成功 ⇒ `Aₖ` が `p` で成功 ⇒ `Aₗ` は走らない。

### `W-CHOICE-SHADOWED-PATH`（厳密。主張は「経路の死」）

`Aₖ = RuleRef(R)` で、`Aₗ` に「同じ位置で `R` を呼び、その後 `minConsume > 0` を必ず消費する」
**導出経路**がある（Choice の一部の候補経由でよい）。

- 選択はまず `Aₖ` を走らせる。`R` が `p` で成功すれば `Aₖ` に確定し `Aₗ` は走らない。
  `R` が `p` で失敗すれば、`R` から始まる `Aₗ` の導出も失敗する。
  どちらにせよ **`R` から始まる `Aₗ` の導出は選択の結果になりえない**。
- `Aₗ` 全体が死ぬとは主張しない（`R` を通らない導出は生き残る）。だから `UNREACHABLE` と別コード。
- `R` の決定性（同じ入力・同じ位置なら同じ結果）に依存する。UBNF は終端を「(入力, 位置) の純粋述語」
  と定めており（D-013）、失敗時の scope undo も契約なので、2 回目の呼出しは 1 回目と同じ結果になる。

### 位置の同一性と trivia

`R` へ降りる途中で trivia 挿入境界を跨ぐと、`Aₖ` は `p` で、`Aₗ` は `skip(p)` で `R` を呼ぶ。
跨いだ policy がちょうど 1 つで、それが `R` 本体の entry 境界と同じときだけ「同じ位置」と認める。
trivia の読み飛ばしは最長一致で**冪等**（`skip(skip(p)) = skip(p)`）なので、両辺とも
`R` の中身を `skip(p)` から評価することになり、成功・失敗が一致する。
異なる policy・複数 policy・`Aₖ` 側にだけ trivia がある形は不明として出さない。
P4 は全 rule 本体が同じ `trivia:javaStyle:…` を読むのでこの条件を満たす。

## 2. 証明できずに落としたもの

| 案 | 落とした理由 |
|---|---|
| 候補の**言語包含**一般（`06-ubnf-v2.md` の当初 `W-CHOICE-UNREACHABLE`） | PEG の候補は正規でも文脈自由でもない（先読み・順序付き選択・`UNTIL`・Extern）。包含の判定手段が無い。実装したのは「構造同一」と「左端呼出しが必須」という、包含が自明に成り立つ 2 つの特別な場合だけ |
| **FIRST（mayStart）の包含**で死を判定 | `mayStart` は必要条件の**上界**（`analysis/README.md`）。`A` の FIRST が `B` の FIRST を覆っても `A` が成功する保証はまったく無い。これで警告すると実在文法で偽陽性が出る |
| `'a'+` ⊇ `'a'` のような**量指定の包含** | 上と同じ。リテラル接頭辞のように「必ず始まる文字列」へ落とせないと証明にならない |
| `W-CHOICE-SHADOWED-PATH` の**入力の存在証明** | 「その経路を実際に通る入力がある」ことは、PEG の到達可能性＝決定不能。経路が他の理由で死んでいる場合、警告は真だが空虚になる。実測では P4 の 9 件すべてが実在の形だった |
| `'let'` が `letter` を横取りする**語境界**の検出 | 候補順の問題ではなく D-002（境界なし）の帰結。`06-ubnf-v2.md` §3 の v2 既定変更で扱う |

## 3. 全文法の実測

`ubnfc validate` は lower / analysis まで進めて警告を足すようになった（ERROR が無いときだけ）。
`ubnfc ir` / `gen` は `Compiled.warnings` に載る。固定は
`crates/ubnfc-front/tests/choice_reachability.rs` と `crates/ubnfc-fuzz/tests/choice_reachability.rs`。

| 文法 | W-CHOICE-* |
|---|---:|
| 24 fixture のうち 20 件（`unlaxer-dsl/grammar/ubnf.ubnf` 自身を含む） | **0** |
| 同 `positive/annotations.ubnf` | **0**（ERROR 12 件があるので lower へ進まない。従来どおり） |
| 同 `positive/all-tokens.ubnf` / `known/namespace.ubnf` | 評価対象外（resolve がエラーで停止する既知の不正 fixture） |
| `unlaxer-dsl/examples/tinycalc.ubnf` | **1**（`W-CHOICE-PREFIX`） |
| `unlaxer-dsl/tinycalc-vscode/grammar/tinycalc.ubnf` | **1**（`W-CHOICE-PREFIX`） |
| `tinyexpression-p4.ubnf` | **9**（すべて `W-CHOICE-SHADOWED-PATH`） |
| 生成器の乱択文法 1,000 件（seed 0..999、全件 front を通過） | **606**（461 文法。`NULLABLE-NOT-LAST` 594 / `PREFIX` 3 / `UNREACHABLE` 9） |

`ubnf.ubnf`（self-description）が 0 件だったのは良い報せで、悪い報せではない。
self-host 固定点（`crates/ubnfc-selfhost/tests/regenerate.rs`）も不変。

### 3.1 見出しの実例 — P4 の `Expression`

```ubnf
  // NumberExpression first: matches hand-written ExpressionsParser ordering.
  Expression ::=
      NumberExpression @value      // ← 候補 1
    | BooleanExpression @value     // ← 候補 2
    | StringExpression @value      // ← 候補 3
    | ObjectExpression @value      // ← 候補 4
    | MethodInvocation @value
    | '(' Expression @value ')' ;
```

```
alternative 2 of rule Expression has a derivation that starts with NumberExpression
and then consumes more, but alternative 1 is NumberExpression itself,
so that derivation is never reached [code: W-CHOICE-SHADOWED-PATH]
```

導出は `BooleanExpression → BooleanAndExpression → BooleanXorExpression → BooleanFactor`
まで「先頭要素が次の rule 呼出しである Seq」（`X ::= Y @left { op Y @right }` の塔）で降り、
`BooleanFactor` の候補 2 が

```ubnf
  ComparisonExpression ::= NumberExpression @left CompareOp @op NumberExpression @right ;
```

つまり **`NumberExpression` を呼んでから `CompareOp`（`minConsume ≥ 1`）以降を必ず消費する**。

帰結。入力 `1 > 2` に対し `Expression` はまず候補 1 の `NumberExpression` を走らせ、
`1` を消費して成功する。PEG なので確定し、候補 2 は走らない。呼び出し元の `Formula` は
`Expression EOF` を要求するので `> 2` が残って**拒否**される。これが `comparison-heavy` の
拒否の正体で、`BooleanExpression` が「比較式を読めるのに読ませてもらえない」状態だった。

同じ文法の `BranchExpression` は逆に **`ComparisonExpression` を先頭に置いていて正しい**：

```ubnf
  BranchExpression ::=
      ComparisonExpression | StringComparisonExpression | BooleanEqualityExpression
    | NumberExpression | BooleanExpression | StringExpression | ObjectExpression | MethodInvocation ;
```

`Expression` にも同じ順序（比較・等値を先に）を入れれば直る。
P4 は読み取り専用なので本 PR では直さない。

### 3.2 P4 の 9 件

| rule | 死ぬ経路の候補 | 覆う先行候補 | 覆う rule |
|---|---:|---:|---|
| `Expression` | 2 (`BooleanExpression`) | 1 | `NumberExpression` |
| `Expression` | 4 (`ObjectExpression`) | 1 | `NumberExpression` |
| `Expression` | 4 (`ObjectExpression`) | 3 | `StringExpression` |
| `ObjectExpression` | 3 (`BooleanExpression`) | 1 | `NumberExpression` |
| `ObjectExpression` | 3 (`BooleanExpression`) | 2 | `StringExpression` |
| `BranchExpression` | 5 (`BooleanExpression`) | 4 | `NumberExpression` |
| `BranchExpression` | 7 (`ObjectExpression`) | 4 | `NumberExpression` |
| `BranchExpression` | 7 (`ObjectExpression`) | 6 | `StringExpression` |
| `BooleanComparable` | 5 (`InMethod`) | 2 | `IfExpression` |

最後の 1 件は `InMethod ::= StringExpression '.in' '(' … ')'` で、`StringExpression → StringTerm`
の候補 2 が `IfExpression`。`if(…){…}else{…}.in('a')` は `BooleanComparable` の候補 2 で
`IfExpression` に確定し、`.in(…)` が残る。
なお P4 は `StringTerm` で `SliceExpression` を `ParenthesizedStringExpression` より前に置く理由を
コメントで説明している（「`($name)` で確定すると slice の suffix が残る」）。
この検査はその手作業を機械化したものにあたる。

### 3.3 tinycalc の 1 件

```ubnf
  VariableDeclaration ::=
    ( 'var' | 'variable' ) @keyword
    IDENTIFIER @name [ 'set' Expression @init ] ';' ;
```

`'var'` は `'variable'` の接頭辞なので、**`'variable'` は決して選ばれない**。
`variable x;` は `'var'` で確定し、`IDENTIFIER` が `iable` を読み、`';'` の位置に `x` が来て拒否される。
D-002（語境界なし）と組み合わさった実在の欠陥。`( 'variable' | 'var' )` に直せば消える
（P4 は同じ場所を正しい順序で書いている）。fixture は corpus の期待値なので本 PR では直さない。

### 3.4 乱択文法での掃き出し

生成器（`crates/ubnfc-fuzz/src/gen.rs`）の seed 0..999 は 1,000 件すべて front を通過し、
461 件に警告が出た。生成器は候補順を無作為に選ぶので、これは期待どおりの分布。
各コードの最初の実例を目視で確認した（`cargo test -p ubnfc-fuzz --test choice_reachability -- --nocapture`）:

- `NULLABLE-NOT-LAST`: `M0 ::= … | [ '('+ @v4 ] | …` — 候補 2 が常に成功し候補 3 が死ぬ
- `UNREACHABLE`: `L2 ::= L0 @v1 | L0 ( … ) '-'` — 候補 2 は必ず `L0` から始まる
- `PREFIX`: `L0 ::= '=' @v1 | '=' 'c'+ T0` — `'='` が `'=c'` の接頭辞

同じ seed で同じ結果になることも固定した（検査は memo 付き DFS で、循環時は空を返して打ち切る。
打ち切りは常に**警告が減る方向**なので健全、走査順は入力だけで決まるので決定的）。

## 4. 診断の配管

4 コードは **ubnfc 固有の追加**で、旧 Java `GrammarValidator` には対応物が無い（他の `E-*` / `W-*` は
Java 実装を正としている）。既存の `ValidationIssue` にそのまま載せ、`category()` は新しい `CHOICE` を返す。
決定的順序（grammar → rule → code → message）、`--strict` のエラー化、JSON の `severityCounts` /
`categoryCounts` は既存機械をそのまま使う。

- `ubnfc validate`: ERROR が 1 件も無いときだけ lower / analysis まで進めて警告を足す
  （lower / analysis が失敗しても validate の契約は変えず、その失敗は `ir` / `gen` が報告する）
- `ubnfc ir` / `ubnfc gen`: `Compiled.warnings` に入り stderr へ出る。`--strict` は exit 5
- IR には載せない（`analysis::analyze_with_choice_issues` の第 3 返値。`analyze` の型は変えていない）

## 5. 警告件数の変化（テストの期待値）

| テスト | 変化 |
|---|---|
| `crates/ubnfc-front/tests/resolve_validate_fixtures.rs` | **変化なし**。`validate()` を直接呼ぶ経路で、lower / analysis を通らない。合計は従来どおり `W-LEFT-RECURSION` 2 / `W-GENERAL-NO-ROOT` 3 |
| `crates/ubnfc-front/tests/pipeline_golden.rs` | **変化なし**。不正 3 fixture は ERROR があるので lower へ進まず `severityCounts.WARNING` は 0 のまま。golden IR も不変 |
| `crates/ubnfc-front/tests/pipeline_api.rs` | **変化なし**（`W-LOWER-SUFFIX-PRIORITY` 1 件の文法に choice 警告は無い） |
| `crates/ubnfc-front/tests/choice_reachability.rs`（新規） | 24 fixture + P4 で `W-CHOICE-PREFIX` **2** / `W-CHOICE-SHADOWED-PATH` **9**、合計 **11** |
| `crates/ubnfc-fuzz/tests/choice_reachability.rs`（新規） | 乱択 1,000 件で `NULLABLE-NOT-LAST` **594** / `PREFIX` **3** / `UNREACHABLE` **9**、合計 **606**（461 文法） |

## 6. corpus（受理・観測の不変）

| backend | 結果 | 比較対象 |
|---|---|---|
| Java `run-all` | **Fail 0**。Pass **1310** / 期待拒否 28 / Known 28 / 未観測 59 | `docs/reports/2026-09-23-java-corpus-contract.md` と**系統表・未観測表・拒否理由表がすべて一致**（差分は generator / front の SHA-256 の 2 行だけ） |
| Rust `run-all` | **Fail 0**。Pass **1292** / 期待拒否 30 / Known 28 / 未観測 75 | `docs/reports/2026-09-23-rust-corpus-contract.md` と同じく**表はすべて一致**（差分は SHA-256 2 行と build 秒数） |

報告本体は `docs/reports/2026-09-23-{java,rust}-corpus-choicecheck.{md,json}`。
元 corpus 127 ファイル・1212 ケース、展開後 1425 件。警告は生成物にも観測にも入らないので、
これは当然の結果だが、`Compiled.warnings` を増やしたことが生成系に漏れていないことの確認になる。

そのほか:

- `cargo test --workspace --offline` 全通過（fuzz campaign の差分ゼロを含む。`ubnfc-java` の jar を
  先に `mvn -o -q -DskipTests package` で更新する D-033 の鮮度検査も満たしている）
- `cargo clippy --workspace --all-targets --offline -- -D warnings` / `cargo fmt --all -- --check` 通過
- `cd ubnfc-java && mvn -o -q test`、`cd examples/p4-java && mvn -o -q test` 通過
- self-host 固定点（`ubnfc-selfhost` の `parity` / `regenerate`）不変。golden IR（`ir/fixtures/**`）も不変

## 7. 再現

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cd ubnfc-java && mvn -o -q -DskipTests package && mvn -o -q test
cd examples/p4-java && mvn -o -q test
cargo run -q --offline --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-23-java-corpus-choicecheck.md --json docs/reports/2026-09-23-java-corpus-choicecheck.json
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust \
  --report docs/reports/2026-09-23-rust-corpus-choicecheck.md --json docs/reports/2026-09-23-rust-corpus-choicecheck.json
# 個別の文法を見る
cargo run -q --offline --bin ubnfc -- validate --grammar <path>.ubnf
```
