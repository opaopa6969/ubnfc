# UBNF v2 — 言語の弱点を消す

> 2026-09-23 開始。利用者が tinyexpression のみの段階で、互換のために抱えていた欠陥を解消する。

## 0. バージョン門 — **実装済み（D-038）**

grammar 設定 `@ubnf: 1|2`（既定 **1**、grammar ごとに 1 回）。

- **v1 は凍結**。既存の 24 fixture・P4・corpus 1,212 ケース・差分ファジング・self-host 固定点が
  永久にこれを守る。v1 の文法は 1 文字も変えずに通り続ける。
- **v2 でのみ**新機能と挙動変更を有効にする。IR には必須キー `languageVersion` を載せ、
  backend は未知バージョンを明示拒否する。
- 純粋な追加（新しい書き方を足すだけで既存の意味を変えない）は v1 でも使えてよい。
  **既存の受理・AST・診断が変わるものだけ** v2 に閉じる。

実装の要点:

- 表層に足した構文は「v1 では構文エラーだった位置」にしか置かない
  （設定値の数値 `@ubnf: 2`、候補**先頭**の `@leftAssoc` / `@rightAssoc` / `@mapping`）。
  末尾に `@rightAssoc` を置く案は直前の要素の capture と区別が付かないので採らなかった。
- `languageVersion` は IR の必須キー。既存フィールドを変えない追加なので IR を全部再生成した。
- 未知バージョンの拒否は Rust `crates/ubnfc-rust/src/validate.rs`、Java
  `ubnfc-java/.../ir/Validation.java`。
- v2 の表層機能は front が v1 の正規形へ脱糖するので、**backend の実装は門しか増えない**。

## 1. 左再帰 — **実装済み（D-039）**

### 問題
P4 は数値・文字列・真偽で同じ優先順位塔を書いており（式の塔だけで 8 規則）、全体 124 規則。

```ubnf
NumberExpression ::= NumberTerm @left { AddOp @op NumberTerm @right } ;
NumberTerm       ::= NumberFactor @left { MulOp @op NumberFactor @right } ;
NumberFactor     ::= ... ;
```

ANTLR4 なら 1 ルール。可読性・保守性の差は決定的で、UBNF の最大の弱点。

> **訂正（2026-09-23、実装後）**: 本計画の初版は「同じ塔を 4 回書いており」と書いていたが誇張だった。
> 実測は塔 3 つ・式の塔だけで 8 規則で、v2 で書き直しても **124 → 119（−5）**。
> 左再帰の価値は**規則数ではなく「優先順位が宣言的になること」**である
> （`docs/reports/2026-09-23-ubnf-v2-left-recursion.md`）。

### 採った形（v2）
ANTLR4 と同じ「候補順＝優先順位（先頭が最高）」の左再帰ルールを導入し、
**front が優先順位塔へ脱糖する**（`crates/ubnfc-front/src/desugar.rs`）。

```ubnf
@mapping(Binary, params=[left, op, right])
Expr ::=
    @rightAssoc Expr @left '^' @op Expr @right   // 最高優先、右結合
  | Expr @left MulOp @op Expr @right
  | Expr @left AddOp @op Expr @right
  | '(' Expr ')'
  | NUMBER ;
```

- `@leftRec` は要らない。**先頭要素が自分自身**の候補があることが唯一の引き金
  （v1 ではその形は `E-LOWER-D016` で落ちていたので曖昧にならない）。
- 脱糖先は v1 の正規形。入口は元の名前のまま（最低優先の段）、中間段は `Name__lr{n}`、
  基底段は `Name__lrBase`。
- **AST は段ごとに 1 ノード**（v1 と同じ `left + op[] + right[]`）。段ごとに別クラスが要る
  ときは候補先頭の `@mapping` で上書きする。
- 間接左再帰は引き続き拒否（ANTLR4 も同じ）。
- **climbing にしなかった**理由と、第一段で拒否する形は D-039 と
  `docs/reports/2026-09-23-ubnf-v2-left-recursion.md`。

### 残っている宿題
前置・後置・三項の候補、規則参照を演算子にした右結合、`@scopeTree` / `@recovery` を持つ
左再帰ルール、そして「演算子適用ごとの二項ノード」という別の AST 形。

## 2. 到達不能な候補の検出（低リスク・即効）

PEG の順序付き選択は、候補順を誤ると黙って前の候補が勝つ。P4 の `comparison-heavy` が
`Formula` 入口で拒否されるのはこれが原因だった。

**実装済み（D-037、`crates/ubnfc-front/src/analysis/reachability.rs`）**。証明は設計 v1 §3.4、
実測は `docs/reports/2026-09-23-choice-reachability.md`。

- `W-CHOICE-NULLABLE-NOT-LAST`: 無条件にゼロ幅成功する候補が末尾以外にある（以降が全て死ぬ）
- `W-CHOICE-PREFIX`: A が B の必須先頭リテラルの接頭辞（`'a' | 'ab'` 形）。PEG では A が勝ち B は死ぬ
- `W-CHOICE-UNREACHABLE`: B が A と構造同一、または B の成功が常に「同じ位置での A の成功」を含む
- `W-CHOICE-SHADOWED-PATH`: B に「A を呼んでさらに消費する」導出経路がある（その経路だけが死ぬ）。
  P4 の `Formula` 拒否はこれ。当初案の「B の言語が A に含まれる」一般形は証明できないので入れていない。

**v1 でも出す**（警告のみで受理は変えない）。`--strict` でエラー化。

## 3. リテラルの単語境界 — **実装済み（D-040）**

`'let'` が `letter` の先頭 3 文字に一致する（D-002）。仕様ガイド自身が「識別子を横取りしない」と
書いていて実装と矛盾していた。ubnfc は互換のため実装側に従っていた。

### 決めた規則（最小）

> **末尾が識別子構成文字のリテラルは、一致の直後の code point も識別子構成文字なら一致しない。**

- **識別子構成文字 = ASCII `[A-Za-z0-9_]`**。組込み `IdentifierParser` の継続文字集合と厳密に同じ
  （`AlphabetNumericUnderScoreParser`）。横取りされる相手がその集合でしか伸びないので、これが
  「必要十分」。非 ASCII を識別子扱いしない理由は 2 つ: (a) 組込み識別子が非 ASCII を受理しないので
  `'let'` + `é` は横取りではない、(b) 生成物は依存ゼロ契約なので Unicode 表を持ち込めない。
- **末尾が記号のリテラルには境界を付けない**。`'+='` `'('` `'.in('` は 1 bit も変わらない。
  規則が「実在の問題（キーワード横取り）」だけに当たるようにするための形の条件。
- **開始側の境界は課さない**。PEG は「直前の要素が終わった位置」からしかリテラルを試さないので
  開始側は構造的に守られている。実測でも corpus 1,425 ケースの応答が末尾側だけの場合と
  1 byte も変わらず（利得 0）、一方で `10px` `3.5f` のような接尾キーワードは確実に壊れる。
- **対象は「消費するリテラル形の終端」だけ**: rule 本体のリテラルと `CI('word')`。
  `LOOKAHEAD` / `NEGATIVE_LOOKAHEAD` は消費しないゼロ幅述語（生の接頭辞を覗くのが目的）、
  `UNTIL` の終端と `@recovery` の同期文字列は区切り記号であって語ではないので対象外。
- **診断は不一致とまったく同じ**。境界違反はリテラル不一致と同じ位置・同じ expected label
  （`'let'` / `WordParser`）・`consume(0)` を記録する。新しい診断 kind も新しい候補ラベルも足さない。

### 設定と版

`@literalBoundary: word|none`（grammar ごと、1 回）。既定は `@ubnf: 1` で `none`、`@ubnf: 2` で `word`。
IR は既存キー `ExprNode.literal.wordBoundary` を固定値 `false` から真の bool に変え、
`TokenSpec.caseInsensitive` に同名キーを足した（追加キー）。**backend は flag しか見ない**。

### 実測した互換影響

`docs/reports/2026-09-23-ubnf-v2-word-boundary.md`。要点:

- 24 fixture のうち IR まで通る 21 件（`ubnf.ubnf` を含む）+ P4 の全リテラル 627 本のうち
  **209 本**が境界対象。
  `wordBoundary` 以外の IR は 1 byte も変わらない。
- corpus に `@literalBoundary: word` を全文法へ強制すると **Java 1310 → 1213**、
  **Rust 1292 → 1197**（Fail は**両 backend で完全に同じ 98 件**）。ただし **98 件すべてが
  「英数字リテラルを空白なしで連結する合成試験文法」の帰結**で（影響した 41 build の境界対象
  115 本のうち 114 本が 1 文字リテラル `'a'` `'x'` `'b'` `'y'` …）、キーワード横取りの実例ではない。
  corpus は v1 のまま凍結なので実際には何も起きない。
- **実在の 3 文法は退行ゼロ**。P4 は Rust 12/12 緑・Java 28 中 27 緑（落ちるのは
  「commit 済み v1 IR から再生成した source と一致するか」の meta 試験だけ）。
- `ubnf.ubnf` は**既知の self-host 差分が 6 → 5 に減る**（`known/keyword-boundary.ubnf` の `@rooted`）。
  手書き frontend は最初から境界を持っており、生成 frontend だけが持っていなかった。

### W-CHOICE-PREFIX への波及

D-037 の証明は「`L` が一致するなら先行候補も一致する」に依存していた。境界があると成り立たない
ので、先行候補に `wordBoundary` があるときは**後続候補の必須接頭辞が `L` の直後に識別子構成文字で
ない文字を持つと分かるときだけ**警告する（`03-design-v1.md` §3.4）。実測では tinycalc の 2 本が
v2 で消え（`'variable'` が本当に到達可能になる）、P4 の 9 件（`W-CHOICE-SHADOWED-PATH`）は不変。

## 4. パラメータ化ルール（v2）

Menhir 風の高階ルールで文法量を減らす。`%` 分離が組込みなのはこの不在の代償。

```ubnf
List<X, Sep> ::= X { Sep X } ;
Args ::= '(' [ List<Expr, ','> ] ')' ;
```

**意味述語（ANTLR の `{...}?`）は入れない。** 終端が「(入力, 位置) の純粋述語」である性質に
memo・候補除外・字句キャッシュの健全性が全部乗っているため（D-013、`06` §3）。
文脈依存が要る場面は `@backref`/`@typeof`/`@scopeTree` で宣言的に扱う。

## 5. 言語中立な token 語彙（v2）

```ubnf
token NUMBER = org.unlaxer.parser.elementary.NumberParser   // v1
token NUMBER = NUMBER                                        // v2（組込み語彙）
token CODE   = extern 'CodeStartParser'                      // host 実装は extern と明示
```

IR には既に組込み種別があるので、表層の語彙を足すだけ。

## 6. アノテーションの統廃合（v2）

- `@interleave(profile=)` は `@whitespace` にほぼ吸収できる（rule-trivia 契約で優先順位が既に決まっている）
- `@backref` がスコープの有無で意味が変わるのを 2 つの注釈に分ける
- `@mapping` / `@enum` / `@commonField` の関係を 1 つの型注釈系に整理

## 実施順

1. ~~§2 到達不能候補の検出（v1 で警告、低リスク）~~ **完了（D-037）**
2. ~~§1 左再帰（最大の価値）~~ **完了（D-039）**
3. ~~§3 単語境界（互換影響の実測 → v2 既定）~~ **完了（D-040）**
4. §5 token 語彙、§4 パラメータ化ルール、§6 統廃合
