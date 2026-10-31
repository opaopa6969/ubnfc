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

## 4. パラメータ化ルール — **実装済み（D-043 / D-044）**

### 採った形（v2）

`Name<P1, ..., Pn> ::= body ;` は**規則ではなく規則本体の雛形**で、`Name<A1, ..., An>` の
位置へ **resolve より前にその場で展開**される（`crates/ubnfc-front/src/templates.rs`）。

```ubnf
VarDecl<Hint, Value> ::=
  ( 'variable' | 'var' ) '$' IDENTIFIER @varName
  [ Hint ]
  [ 'set' [ OnlyIfAbsent @onlyIfAbsent ] Value @value ]
  ';' ;

@declares(symbol=varName, description=desc)
@mapping(NumberVariableDeclarationExpr, params=[varName, onlyIfAbsent, value, desc])
NumberVariableDeclaration ::= VarDecl<NumberTypeHint, NumberExpression> ;
```

- **新しい規則も新しい名前も作らない**。規則数・AST クラス・capture・`rulePath` /
  `deepestRule` / expected ラベルは一つも変わらず、**名前のマングリングが要らない**。
- Menhir の `separated_list(sep, X)`（新しい非終端）**ではなく `%inline` 側**を採った。
  `@mapping` が規則ごとに AST クラスを決める UBNF では、非終端版は**必ず AST を変える**ので
  既存文法へ後付けできない（P4 のカンマ区切り 15 箇所のうち 12 箇所が該当）。
- 実引数は **atom**（規則参照・token・リテラル・`( )` `[ ]` `{ }`）。capture は雛形の中に書き、
  **実体化した規則のフィールド**になる。注釈は雛形に置けず、実体化する規則に書く。
- **再帰は一切禁止**（`E-TEMPLATE-RECURSION`）。その場での置換なのでどんな循環も無限展開になる。
  依存グラフが有限なので判定は**完全**（下界ではない）。再帰が要る形は通常の規則で書ける。

### 実測（`docs/reports/2026-09-23-ubnf-v2-parameterized-rules.md`）

P4-v2 は **規則数 119 のまま、646 → 607 行**、11 雛形・32 実体化。v1 parser との応答は
28/28 一致で、IR は「文法ソース上の範囲」以外 117,940 行が 1 行も違わない。

正直な評価: **`%`（分離リスト）の穴埋めとしての価値はほとんど無い**（カンマ区切りは実在の
25 文法中 P4 に 15・`ubnf.ubnf` に 5・残り 23 文法に 0 で、capture 名がばらばらなので
1 つの `SepList` では括れない）。価値があったのは**型ごとの多重化**のほうで、
`VariableDeclaration` 4 本・`MethodDeclaration` 4 本・8 候補のスライス構文 2 本といった
「AST クラスが違うので規則としては減らせないが、本体は同じ」形が 1 箇所に集まる。
効能は「規則数が減る」ではなく「**同じ形が 1 箇所に書かれる**」である。

**意味述語（ANTLR の `{...}?`）は入れない。** 終端が「(入力, 位置) の純粋述語」である性質に
memo・候補除外・字句キャッシュの健全性が全部乗っているため（D-013、`06` §3）。
文脈依存が要る場面は `@backref`/`@typeof`/`@scopeTree` で宣言的に扱う。

## 5. 言語中立な token 語彙 — **実装済み（D-041 / D-042）**

### 問題
v1 の token 宣言は host 言語のクラス名を書く。Java と Rust の両方を出す言語中立な形式を
名乗る以上これは層の破れで、IR は既に組込み種別へ正規化しているので**残っていたのは表層だけ**だった。

### 採った形

```ubnf
token NUMBER     = NUMBER                        // 言語中立な組込み語彙（36 名前）
token STRING     = extern 'StringLiteralParser'  // host 実装は extern と明示
token EOF        = END_OF_INPUT
```

- **裸の語**（SCREAMING_SNAKE_CASE）。token 宣言の値の位置には利用者の token 名も規則名も
  現れないので衝突し得ず、`ANY` / `EOF` / `EMPTY` / `UNTIL(…)` は v1 から既にこの形である。
  host クラス単純名（`UpperCamelCase`）とも字面が重ならず、重なっても `extern '<id>'` が逃げ道になる。
  名前空間形も `builtin` キーワードも採らなかった（理由は報告 §1）。
- 中立名は**旧実装の FQN を既存の分類器へ渡すだけ**なので、`02-facts` §A の受理範囲を
  1 つも変えない（`crates/ubnfc-front/src/vocabulary.rs` がその表そのもの）。
- **`extern '<id>'` の `'…'` は scanner registry の ID** で、クラス名でなくてよい
  （`scanners/first-chars.json`・`scanners/{java,rust}`）。v1 では構文エラーだった位置なので純粋な追加。
- `@ubnf: 2` の文法で `--extern-first-chars` を渡したとき、registry が提供しない extern は
  **生成時に** `E-TOKEN-EXTERN-UNREGISTERED` で落ちる（exit 1）。

### v1 / v2 の方針（D-042）
中立語彙も `extern` も**純粋な追加なので v1 でも使える**。v2 でだけ変わるのは、
旧綴りに `W-TOKEN-HOST-CLASS` が出ること（**拒否はしない**）と、上の registry の門である。
拒否しない理由は「唯一の実利用者（tinyexpression の P4）の文法がこちらから直せない別 repo にあり、
v2 を機能ごとに採れなくなる」「拒否しても `builtin_class` は v1 のために残るので何も減らない」
「強制したい側は `--strict` でエラー化できる」。

証拠と語彙の全表は `docs/reports/2026-09-23-ubnf-v2-token-vocabulary.md`。
P4-v2 の移行では IR の差が `originalDeclaration` / `reason` / extern の registry ID の
**4 キー 20 箇所だけ**（綴りの原文・説明文・extern の registry ID）になり、28 入力の driver 応答は完全に一致した。

## 6. アノテーションの統廃合（v2）— **P1 / P2 / P3 / P5 と P7-1 / P7-3 は実装済み**

**棚卸しと提案は `07-annotation-inventory.md`**（全表・グループ分け・順位付き統廃合案・影響範囲）。
実装の結果は `docs/reports/2026-09-23-annotation-consolidation.md`（D-045 〜 D-049）。
残りは **P4**（`@precedence` 省略可能）・**P6**（型注釈系）・**P7-4**（TriviaPolicy 重複）。
要点だけ:

- 名前の付いた表層は **26**（rule/element 注釈 20 + grammar 設定 5 + directive 1）＋ 未知注釈の catch-all。
  実言語 18 文法で使われていないものが **11**、どの文法にも 1 度も現れないものが **3**
  （`@longestChoice` / `@predictiveChoice` / `@literalBoundary`）。
- 採用した案は 6 つ。順に **P1** 未知注釈の警告（`@rooted` のような綴り間違いが今は素通り）、
  **P2** `@backref` を `@symbolRef` / `@sameText` に割る（今は文法内の遠い `@scopeTree` の有無で意味が変わる）、
  **P3** `@interleave` を `@whitespace` に畳む（2 profile とも同じ区切り集合に落ちる。実在 7 箇所）、
  **P5** `@predictiveChoice` を非推奨（front が自力で `strategy=predict` を 16 箇所選んでいる）、
  **P4** `@precedence(level=N)` を省略可能に（両 backend が値を読んでいない）、
  **P6** `@enum` / `@commonField` を `@mapping` の下位形へ。
  `@longestChoice` は最長一致が導出不能な意味論なので**残す**。
- 統廃合の前に出すべき不具合が 4 件（`@rightAssoc` の span 補正が両 backend で非対称、
  Java に `languageVersion` 上限検査が無い、`@rightAssoc` + `@precedence` が `operator` を作らない、
  同一内容の TriviaPolicy が重複）。**実装時に 2 件が棚卸しの誤りと判明**した:
  `languageVersion` 上限検査は Java にもあり（`ir/Validation.java:43`）、`operator` を作らない
  分かれ目は結合性ではなく演算子 atom の種類（`@leftAssoc` + リテラル演算子も `None`）だった。
  本当の不具合は `@rightAssoc` + 規則参照演算子が `InvalidOperator` で文法ごと落ちること（D-047）。
- P1〜P3・P5 は**警告を足すだけ**で IR も backend も動かないので 1 本にまとめられる。

## 7. code point クラス — **実装済み（D-063）**

### 問題

RFC 8259 §7 は「文字列の中に生で置ける文字」を「`"` と `\` と U+0000〜U+001F を除く全 Unicode 文字」と
定める。ANTLR は `~["\\\u0000-\u001F]`、tree-sitter は `/[^"\\\x00-\x1f]/` でそのまま書けるが、
UBNF v2 には**この集合を書く手立てが無かった**（issue #34、
[`../reports/2026-09-24-antlr-treesitter-comparison.md`](../reports/2026-09-24-antlr-treesitter-comparison.md) §1.1 A）。

- `NEGATION('…')` の除外集合はリテラルで、リテラルのエスケープは `\n \t \r \\ \'` の 5 つだけ。
  綴れる制御文字は 3 つしかない。
- 正の範囲で書くと `CHAR_RANGE` の境界が **Java `char` = BMP 限定**（`E-TOKEN-CHAR-RANGE-SUPPLEMENTARY`）
  なので、補助面の生文字を落として**受理必須の `y_string_utf8` 等を拒否**してしまう。
- `REGEX('[^"\\\x00-\x1F]')` は front の正則部分集合パーサが `\x` を拒否する。

そのため `examples/json/json.ubnf` は `token SAFE = NEGATION('"\\')` を採り、**制御文字が通ること**を
受け入れていた。JSONTestSuite で `n_string_unescaped_{ctrl_char,newline,tab}` の 3 件だけ、
ANTLR / tree-sitter と判定が割れていた。

### 採った形（v2）

**新しい構成子を足さない。**既存の 2 つに、v2 限定で 2 つのことを許す。

```ubnf
@ubnf: 2
token SAFE_SP   = CHAR_RANGE('\u0020','\u0021')        // (a) \uXXXX 境界
token SAFE_MID  = CHAR_RANGE('\u0023','\u005B')
token SAFE_BMP  = CHAR_RANGE('\u005D','\u{D7FF}')      // (a) \u{X…} 境界
token SAFE_SUPP = CHAR_RANGE('\u{E000}','\u{10FFFF}')  // (b) BMP 制限を外す
token NOT_CTRL  = NEGATION('"\\\u0000\u{7F}')          // (a) 除外集合でも使える
```

- **(a) エスケープ**: `\uXXXX`（16 進 4 桁ちょうど）と `\u{X…}`（1〜6 桁、U+10FFFF まで）を
  **code point 1 個**へ展開する。効くのは `CHAR_RANGE` の 2 つの境界と `NEGATION` の除外集合**だけ**。
  rule 本体のリテラル・`UNTIL` / `CI` / `LOOKAHEAD` / `NEGATIVE_LOOKAHEAD` / `REGEX` は v1 のまま。
- **(b) 補助面**: `CHAR_RANGE` の境界は U+0000..U+10FFFF の任意の code point 1 個。
  `E-TOKEN-CHAR-RANGE-SUPPLEMENTARY` は **v1 専用**になった。D-050 の残り 3 条件
  （展開後にちょうど 1 個・`min <= max`・surrogate を含まない）は v2 でもそのまま。
- **v1 は門で止める**（`E-TOKEN-ESCAPE-V2-ONLY`）。v1 の `'\u0041'` は「表にないエスケープ」として
  6 文字に読まれ受理されていたので、黙って意味を変えず**明示的に拒否**する。
  実在の v1 文法にこの綴りは 1 件も無い。
- **展開は字句値ではなく原文から**行う。字句は `\\` を 1 個の `\` に畳むので、
  `'\\u0041'`（バックスラッシュ + `u0041`）と `'\u0041'`（エスケープ）が字句値では同じになってしまう。
  AST に引用符の内側の原文を持たせ、`crates/ubnfc-front/src/escape.rs` が 1 度だけ走査する。
  走査規則は v1 と同じで `\u` の枝を足しただけなので、**`\u` を含まない原文では v1 の字句値と
  1 文字も違わない**（`crates/ubnfc-front/tests/escape.rs`）。

### backend と IR

**どちらも増えない。**両 backend の照合はもともと code point 単位（Rust は `Input::cp_at` の
UTF-8 1 scalar 復号、Java は `Character.codePointAt` + `Character.charCount`）で、
`TokenSpec.charRange` は既に `{min,max}` の整数だった。schema の `maximum` を `65535` → `1114111` へ
**広げた**だけで、既存の IR はすべて有効なまま・v1 golden は 1 byte も動かない。

### 実測

`examples/json/json.ubnf` を 4 本の `CHAR_RANGE` へ書き換えて JSONTestSuite 318 件:

| | `y_` 受理 / 95 | `n_` 拒否 / 188 | ANTLR / tree-sitter との不一致 |
|---|---:|---:|---|
| 従来（`NEGATION('"\\')`） | 95 | 185 | 3 件（`n_string_unescaped_*`） |
| **D-063 後** | **95** | **188** | **0 件** |

残る不一致は `i_`（実装依存）の 2 件だけ——先頭 BOM（tree-sitter だけ受理）と
500 重ネスト（issue #35）。証拠は
[`../reports/2026-09-24-v2-codepoint-classes.md`](../reports/2026-09-24-v2-codepoint-classes.md)。

### 残っている宿題

`REGEX` の正則部分集合に `\xHH` / `\uHHHH` / `\u{X…}` とクラス否定（`[^…]`）を足すこと（issue #34 の候補 (c)）。
`REGEX` の文字列は解析器が生の `\` 列を読むので、前段で展開すると意味が二重になる。別 issue で扱う。

## 8. package の import と package 由来の空白 — **実装済み（D-083）**

Classic（unlaxer-parser #401、`docs/ubnf-packages.md`）は、版付きの artifact を
`@import alias from 'pkg:ID'` で読み、その token を `@whitespace: alias.TOKEN` の profile に使える。
tinyexpression P4 は 2026-10 にこの綴りへ移った（`pkg:std/layout` の `SPACES_AND_COMMENTS`）。

### 採った形

- `pkg:ID` の import は、grammar と同じディレクトリの `ubnf.json`（manifest）と `ubnf.lock.json`（lock）で
  解決する。Classic と同じく取得はしない。manifest の版と lock の `roots` が違えば stale lock として拒否する。
- `builtin:` の source は ubnfc に同梱した artifact（`crates/ubnfc-front/packages/`、Classic の同梱物と byte 同一）を
  使い、lock の SHA-256 と照合する。それ以外の source は `.ubnf-cache/packages/<sha256>.json` を読み、内容の
  SHA-256 が lock と一致するときだけ使う。
- resolve は `pkg:ID` を仮想パス `<grammar のディレクトリ>/.ubnf-packages/ID` に写す。IR の `sources[]` には
  この論理パスと entry 本文の SHA-256 が載る。
- `@whitespace` の値（設定・規則注釈とも）には、**std/layout 1.0.0 と構文木が同一の import の
  `SPACES_AND_COMMENTS` だけ**を書ける。これは javaStyle と同じ区切り集合（ASCII space と TAB〜CR、`//` から
  CR / LF の直前まで、`/*` から最初の `*/` まで）なので、IR では javaStyle と同じ `effectiveDelimiterSet` になる。
  Classic も javaStyle の 11 文法・24 入力を明示 import に置き換えて同値を確かめている。
- 失敗はすべて import の位置付きの `E-IMPORT-UNRESOLVED`（hint に理由）。std/layout 以外の token を profile に
  する綴りは、従来どおり `E-WHITESPACE-GLOBAL-STYLE` / `E-WHITESPACE-RULE-STYLE`。

### 範囲外

- 任意の declarative token を空白 profile にすること（IR の delimiter は文字集合・行コメント・block comment の
  3 種だけなので、一般の lexical program は表せない）。
- package の中からの import（package 内の相対 import と推移的な package 依存）。std/layout はどちらも持たない。
- `unlaxer deps resolve` に当たる取得コマンド。lock と cache は Classic の CLI で作る。
## 9. 綴りの正本は Classic — **実装済み（D-084）**

2026-10-23 に所有者が決めた方針: **UBNF の綴りは Classic（unlaxer-parser）を正本とする。**
ubnfc の front は Classic の綴りを受け付ける。ubnfc 独自の綴り（`Name<args>`、D-043）は当面互換のために残し、
非推奨にするかどうかは別の issue で決める。同じ文法ファイルを両方のエンジンに通せることが目的である。

### 受け付ける綴りと、IR に落とせないもの

| Classic の綴り | ubnfc の扱い | 根拠・検査 |
|---|---|---|
| 引数付き規則 `F(p1, …, pn) ::= body ;` と呼び出し `F(e1, …, en)`（#701） | 受理し、`<...>` の雛形と同じくその場で展開する。版の門（`@ubnf: 2`）は掛けない | 隣接した `F(` が呼び出しになるのは同じ grammar ブロックに定義がある名前だけ。`F (x) ::=`・`F()`・末尾 `,`・引数位置の literal は構文エラー。Classic の受理例 10 文法・26 入力で受理と AST が一致（`crates/ubnfc-rust/tests/classic_parameterized.rs`） |
| 値渡しの引数 `F(=p) ::=`（#704） | 構文は受理し、Classic と同じく `CAPTURE_SCOPE(CAPTURE(F(=p), a) … body')` に展開して、呼び出しの位置で `E-OWNED-CAPTURE-UNSUPPORTED` | IR に owned capture が無い |
| 規則の中の `CAPTURE(name, e)` / `SAME_AS(name)` / `CAPTURE_SCOPE(e)` | その位置で `E-OWNED-CAPTURE-UNSUPPORTED` | 同上 |
| 規則の中の `COMMIT(e[, 'msg'])` / `LABEL(l, e)` / `CATCH(e, alt {, l})`（#700） | その位置で `E-CUT-UNSUPPORTED` | IR に cut が無い。取り除くと受理する言語が変わることがある |
| `@autoCommit`（#702） | 受理し、何も挿入しない | FIRST 集合で「受理と AST を変えない」位置にだけ `COMMIT` を入れる注釈なので、変わるのは失敗の位置と診断だけ。Classic が解析の対象外の文法で注釈を取り除くのと同じ |

関数形式（`CAPTURE(` など）になるのは、名前と `(` が隣接しているときだけである。`CAPTURE ('x')` のように空白・
コメントを挟めば、従来どおり参照と group になる（Classic と同じ）。宣言的 token の中の `CAPTURE` / `SAME_AS`
（D-061）は従来どおり使える。

### `@backref` / `@sameText` と同じ IR になる範囲

**ない。** 規則の中の `SAME_AS(name)` は照合であり、一致しなければその選択肢が失敗して後戻りする。一方
`@sameText(name=…)` は解析が成功した後の検査で、不一致は ERROR の診断になる（入力は受理される）。受理する
言語が違うので、どちらかをもう一方に書き換えることはしない。`@symbolRef` / `@backref` は scope の表引きで、
これも照合ではない。

### Classic の検査コードとの対応

Classic の綴りで書いた雛形には、Classic と同じ `E-PARAM-*` を使う（`<...>` の雛形は従来の `E-TEMPLATE-*`）。
Classic の拒否例 25 件のうち 20 件を、ubnfc も同じ趣旨の code で拒否する（一覧は
`classic_parameterized.rs` の `expected_rejection`）。違いは次のとおり。

- `E-PARAM-NAME-COLLISION`: ubnfc は instance に名前を付けない（その場で展開する）ので起こらない。
- `E-PARAM-UNDEFINED`（定義本体の未定義の名前）: 展開の後の resolve が `E-RULE-UNDEFINED` として拒否する。
- `E-PARAM-RESERVED`: 構文エラー（本文の先頭が `E-PARAM-RESERVED:`）。
- 展開の後の左再帰（CE-002 など）: ubnfc は脱糖できる左再帰（D-039）以外を `E-LOWER-D016` で拒否する。
- `E-PARAM-LIMIT`: 要素数の上限 65536 は Classic と同じ。入れ子は、展開した本体の入れ子を構文解析と同じ
  数え方で 128 までとする。Classic は instance の本体と呼び出しの連鎖も数えるので、Classic が拒否する 4 例
  （平らな連鎖 128 段など）を ubnfc は受理する。この上限は `<...>` の雛形にも当たる（それまでは上限が無く、
  指数的に成長する文法で展開が止まらず、深い入れ子でスタックが溢れた）。

## 実施順

1. ~~§2 到達不能候補の検出（v1 で警告、低リスク）~~ **完了（D-037）**
2. ~~§1 左再帰（最大の価値）~~ **完了（D-039）**
3. ~~§3 単語境界（互換影響の実測 → v2 既定）~~ **完了（D-040）**
4. ~~§5 token 語彙~~ **完了（D-041 / D-042）**
5. ~~§4 パラメータ化ルール~~ **完了（D-043 / D-044）**
6. ~~§6 統廃合の P1 / P2 / P3 / P5 と P7-1 / P7-3~~ **完了（D-045 〜 D-049）**
7. ~~§7 code point クラス~~ **完了（D-063）**
8. §6 の残り（P4 / P6 / P7-4）
9. ~~§8 package の import~~ **完了（D-083）**
10. ~~§9 Classic の綴り~~ **完了（D-084）**
