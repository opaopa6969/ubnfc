# UBNF 仕様由来オラクル — 4 実装の突き合わせ（自動生成）

期待値は `unlaxer-dsl/specs/*.md` の規範文から導いたもので、
どの実装の観測値も使っていない（`corpus/spec/ubnf/README.md`）。

ケース 129 件。判定は 一致 / 違反 / 別コード / 未定（仕様が沈黙）/ 駆動不能。

## 1. 節ごとの件数と違反数

| 節 | ケース | ubnfc-front(手書き) 違反 | ubnfc-selfhost(生成) 違反 | unlaxer-ubnf(Rust) 違反 | Java bootstrap 違反 |
|---|---:|---:|---:|---:|---:|
| `annotations` (アノテーション) | 32 | 0 | 0 | 0 | 0 |
| `char-range` (CHAR_RANGE('min','max')) | 11 | 7 | 7 | 0 | 0 |
| `comments` (コメント) | 5 | 0 | 2 | 0 | 1 |
| `formal-grammar` (形式的な文法定義) | 23 | 9 | 9 | 1 | 2 |
| `global-settings` (グローバル設定) | 10 | 0 | 0 | 0 | 0 |
| `literals-escapes` (ルール本体の書き方 / リテラル) | 9 | 0 | 0 | 0 | 0 |
| `quantifiers` (量指定子とセパレータ) | 16 | 2 | 2 | 0 | 0 |
| `reserved-words` (予約語) | 3 | 0 | 0 | 0 | 0 |
| `rule-declaration` (ルール宣言) | 10 | 0 | 0 | 0 | 0 |
| `token-declarations` (トークン宣言（token）) | 10 | 0 | 0 | 0 | 0 |
| **合計** | **129** | **18** | **20** | **1** | **3** |

## 2. 判定の内訳

| 実装 | 一致 | 違反 | 別コード | 未定 | 駆動不能 |
|---|---:|---:|---:|---:|---:|
| ubnfc-front(手書き) | 94 | 18 | 3 | 11 | 3 |
| ubnfc-selfhost(生成) | 81 | 20 | 0 | 11 | 17 |
| unlaxer-ubnf(Rust) | 100 | 1 | 0 | 11 | 17 |
| Java bootstrap | 109 | 3 | 3 | 11 | 3 |

`別コード` は validation.md のコード名と違う名前で同じ違反を検出した場合。

## 3. ケース × 実装

| ケース | scope | 期待 | ubnfc-front(手書き) | ubnfc-selfhost(生成) | unlaxer-ubnf(Rust) | Java bootstrap |
|---|---|---|---|---|---|---|
| `annotation/root` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/root-two-rules` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `annotation/mapping-class-only` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/mapping-params` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/mapping-empty-params` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `annotation/mapping-missing-capture` | validate | E-MAPPING-MISSING-CAPTURE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/mapping-extra-capture` | validate | E-MAPPING-EXTRA-CAPTURE | 別コード | 駆動不能 | 駆動不能 | 別コード |
| `annotation/mapping-duplicate-param` | validate | E-MAPPING-DUPLICATE-PARAM | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/whitespace-bare` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/whitespace-none` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/whitespace-uppercase` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/whitespace-bad-style` | validate | E-WHITESPACE-RULE-STYLE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/whitespace-rule-duplicate` | validate | E-WHITESPACE-RULE-DUPLICATE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/precedence-level` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/precedence-missing-level-keyword` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `annotation/precedence-negative` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `annotation/precedence-duplicate` | validate | E-PRECEDENCE-DUPLICATE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/precedence-without-assoc` | validate | E-PRECEDENCE-WITHOUT-ASSOC | 別コード | 駆動不能 | 駆動不能 | 別コード |
| `annotation/assoc-without-precedence` | validate | E-ASSOC-WITHOUT-PRECEDENCE | 別コード | 駆動不能 | 駆動不能 | 別コード |
| `annotation/assoc-both` | validate | E-ASSOC-BOTH | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/interleave-profile` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/interleave-missing-keyword` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `annotation/interleave-bad-profile` | validate | E-ANNOTATION-INTERLEAVE-PROFILE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/backref-name` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/scopetree-mode` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/scopetree-bad-mode` | validate | E-ANNOTATION-SCOPETREE-MODE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/doc-string` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/doc-identifier-rejected` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `annotation/skip` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/simple-catch-all` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/simple-with-arguments-rejected` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `spec-contradiction/minilang-leftassoc` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `char-range/valid` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `char-range/equal-bounds` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `char-range/empty-min` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `char-range/empty-max` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `char-range/multi-char-min` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `char-range/multi-char-max` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `char-range/supplementary` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `char-range/reversed` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `char-range/escaped-bounds` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `char-range/escaped-bounds-reversed` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `char-range/inclusive-bounds` | runtime | undetermined | 駆動不能 | 駆動不能 | 駆動不能 | 駆動不能 |
| `comment/line-accepted` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `comment/line-inside-body` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `comment/block-rejected` | parse | reject | 一致 | 違反 | 一致 | 一致 |
| `comment/block-in-body-rejected` | parse | reject | 一致 | 違反 | 一致 | 一致 |
| `comment/block-unclosed-rejected` | parse | reject | 一致 | 一致 | 一致 | 違反 |
| `file/empty-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `file/only-comments-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `file/two-grammars` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `file/leading-comment-then-grammar` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `file/trailing-garbage-rejected` | parse | reject | 一致 | 一致 | 一致 | 違反 |
| `grammar/empty-block` | parse | accept | 一致 | 一致 | 違反 | 違反 |
| `grammar/keyword-required` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `grammar/name-required` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `grammar/unclosed` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `order/token-after-rule` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `order/setting-after-token` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `body/empty-sequence-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `body/empty-alternative-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `body/leading-bar-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `body/empty-group-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `body/empty-optional-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `body/empty-repeat-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `body/choice-and-group` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `element/typeof-prefix` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `error/element` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `error/element-needs-string` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `error/element-needs-argument` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `error/hint-is-last-alternative` | runtime | undetermined | 駆動不能 | 駆動不能 | 駆動不能 | 駆動不能 |
| `setting/package` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `setting/colon-required` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `setting/quoted-scalar-rejected` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `setting/block-value` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `setting/whitespace-javastyle` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `setting/whitespace-none` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `setting/whitespace-bad-style` | validate | E-WHITESPACE-GLOBAL-STYLE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `setting/whitespace-duplicate` | validate | E-WHITESPACE-GLOBAL-DUPLICATE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `setting/whitespace-block-value` | validate | E-WHITESPACE-GLOBAL-STYLE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `setting/unknown-key` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `literal/single-quoted` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `literal/double-quoted-rejected` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `escape/newline` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `escape/tab` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `escape/carriage-return` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `escape/backslash` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `escape/single-quote` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `escape/unknown-sequence` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `escape/unterminated-literal` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `quantifier/plus` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `quantifier/bracket-optional` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `quantifier/question-equals-bracket` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `quantifier/brace-repeat` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `quantifier/star-equals-brace` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `quantifier/exact` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `quantifier/bounded` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `quantifier/open-ended` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `quantifier/missing-lower-bound` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `quantifier/three-bounds` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `quantifier/two-quantifiers` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `quantifier/quantifier-and-separator` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `quantifier/reversed-bounds` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `quantifier/huge-bound` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `separator/rule-reference` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `separator/expansion` | runtime | undetermined | 駆動不能 | 駆動不能 | 駆動不能 | 駆動不能 |
| `reserved/rule-named-token` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `reserved/rule-named-error` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `reserved/token-named-any` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `rule/semicolon-required` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `rule/semicolon-present` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/assign-token-required` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `rule/lowercase-name` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `rule/two-refs-are-two-elements` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/comment-is-identifier-boundary` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/captures-kept-separately` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/space-inside-identifier` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `rule/dotted-reference` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/dotted-reference-three-segments` | parse | undetermined | 未定 | 未定 | 未定 | 未定 |
| `token/class-reference` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `token/equals-required` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `token/unknown-keyword-with-arguments` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `token/any-eof-empty` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `token/until` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `token/ci` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `token/regex` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `token/negation-empty-set` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `token/negation-codepoint-set` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `token/declaration-needs-no-semicolon` | parse | reject | 一致 | 一致 | 一致 | 一致 |

## 4. 実装別の違反一覧（最小入力つき）

### ubnfc-front(手書き) — 18 件

| ケース | 入力 | 期待 | 実測 | 引用 |
|---|---|---|---|---|
| `char-range/empty-min` | `grammar G { token L=CHAR_RANGE('','a') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/empty-max` | `grammar G { token L=CHAR_RANGE('a','') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/multi-char-min` | `grammar G { token L=CHAR_RANGE('ab','z') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/multi-char-max` | `grammar G { token L=CHAR_RANGE('a','yz') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/supplementary` | `grammar G { token L=CHAR_RANGE('😀','😇') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/reversed` | `grammar G { token L=CHAR_RANGE('z','a') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/escaped-bounds-reversed` | `grammar G { token L=CHAR_RANGE('\n','\t') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `file/empty-rejected` | `(空ファイル)` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `file/only-comments-rejected` | `// just a comment\n` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-sequence-rejected` | `grammar G { R ::= ; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-alternative-rejected` | `grammar G { R ::= 'a' \| ; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/leading-bar-rejected` | `grammar G { R ::= \| 'a'; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-group-rejected` | `grammar G { R ::= (); }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-optional-rejected` | `grammar G { R ::= [ ]; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-repeat-rejected` | `grammar G { R ::= { }; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `error/element-needs-argument` | `grammar G { R ::= 'a' \| ERROR(); }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `quantifier/two-quantifiers` | `grammar G { token V=ANY R ::= V+{2,3}; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `quantifier/quantifier-and-separator` | `grammar G { token V=ANY token S=ANY R ::= V+ % S; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |

### ubnfc-selfhost(生成) — 20 件

| ケース | 入力 | 期待 | 実測 | 引用 |
|---|---|---|---|---|
| `char-range/empty-min` | `grammar G { token L=CHAR_RANGE('','a') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/empty-max` | `grammar G { token L=CHAR_RANGE('a','') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/multi-char-min` | `grammar G { token L=CHAR_RANGE('ab','z') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/multi-char-max` | `grammar G { token L=CHAR_RANGE('a','yz') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/supplementary` | `grammar G { token L=CHAR_RANGE('😀','😇') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/reversed` | `grammar G { token L=CHAR_RANGE('z','a') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `char-range/escaped-bounds-reversed` | `grammar G { token L=CHAR_RANGE('\n','\t') R ::= L; }` | reject | 受理 | ubnf-syntax.md #### `CHAR_RANGE('min','max')` — 文字の範囲 |
| `comment/block-rejected` | `grammar G { /* no */ R ::= 'x'; }` | reject | 受理 | ubnf-syntax.md ## ルール宣言 |
| `comment/block-in-body-rejected` | `grammar G { R ::= /* no */ 'x'; }` | reject | 受理 | ubnf-syntax.md ## ルール宣言 |
| `file/empty-rejected` | `(空ファイル)` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `file/only-comments-rejected` | `// just a comment\n` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-sequence-rejected` | `grammar G { R ::= ; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-alternative-rejected` | `grammar G { R ::= 'a' \| ; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/leading-bar-rejected` | `grammar G { R ::= \| 'a'; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-group-rejected` | `grammar G { R ::= (); }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-optional-rejected` | `grammar G { R ::= [ ]; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-repeat-rejected` | `grammar G { R ::= { }; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `error/element-needs-argument` | `grammar G { R ::= 'a' \| ERROR(); }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `quantifier/two-quantifiers` | `grammar G { token V=ANY R ::= V+{2,3}; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `quantifier/quantifier-and-separator` | `grammar G { token V=ANY token S=ANY R ::= V+ % S; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |

### unlaxer-ubnf(Rust) — 1 件

| ケース | 入力 | 期待 | 実測 | 引用 |
|---|---|---|---|---|
| `grammar/empty-block` | `grammar G { }` | accept | 拒否: grammar requires at least one rule | ubnf-syntax.md ## 形式的な文法定義 |

### Java bootstrap — 3 件

| ケース | 入力 | 期待 | 実測 | 引用 |
|---|---|---|---|---|
| `comment/block-unclosed-rejected` | `grammar G { R ::= 'x'; } /* dangling` | reject | 受理 | ubnf-syntax.md ## ルール宣言 |
| `file/trailing-garbage-rejected` | `grammar G { R ::= 'x'; } garbage` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `grammar/empty-block` | `grammar G { }` | accept | 拒否: java.lang.IllegalArgumentException: UBNF パース失敗: grammar G { } | ubnf-syntax.md ## 形式的な文法定義 |

## 5. 未定（仕様が沈黙・矛盾） 14 件

| ケース | 問い | ubnfc-front(手書き) | ubnfc-selfhost(生成) | unlaxer-ubnf(Rust) | Java bootstrap |
|---|---|---|---|---|---|
| `annotation/root-two-rules` | SHOULD なので違反時に受理するか警告するかエラーにするかが定まらない。 | 受理 | 受理 | 受理 | 受理 |
| `spec-contradiction/minilang-leftassoc` | ubnf-syntax.md の完全版 MiniLang 文法は `@leftAssoc` を `@precedence` 無しで使うが、annotations.md は「`@leftAssoc` を使用するルールは `@precedence` も宣言する（MUST）」、validation.md は E-ASSOC-WITHOUT-PRECEDENCE と書く。仕様内で矛盾している。 | 受理 | 受理 | 受理 | 受理 |
| `char-range/inclusive-bounds` | 両端を含むことは生成パーサの実行時挙動であり、UBNF を読む 4 実装では観測できない。 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 |
| `error/element-needs-string` | `ErrorElement ::= 'ERROR' '(' STRING ')'` は満たさないが、同じ形式文法の `RuleRefElement ::= IDENTIFIER` と `GroupElement ::= '(' RuleBody ')'` で `ERROR` + `(msg)` として導出できてしまう。予約語 `ERROR` を規則参照位置で 使えるかを仕様が決めていない。 | 受理 | 受理 | 受理 | 受理 |
| `error/hint-is-last-alternative` | ヒントが診断に現れることは生成パーサの実行時挙動で、構文木の比較では観測できない。 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 |
| `escape/unknown-sequence` | 表は `\n` `\t` `\r` `\\` `\'` の 5 つだけを挙げ、表に無いエスケープ （例 `\q`）を拒否するのか、バックスラッシュごと残すのか、`q` にするのかを述べていない。 | 受理 | 受理 | 受理 | 受理 |
| `quantifier/reversed-bounds` | `Quantifier ::= … '{' INTEGER ',' [INTEGER] '}'` は n <= m を要求していない。CHAR_RANGE と違って逆順の反復に関する規範文が無い。 | 受理 | 受理 | 受理 | 受理 |
| `quantifier/huge-bound` | INTEGER の表現範囲（32bit か否か）を仕様が定めていない。 | 受理 | 受理 | 拒否 | 拒否 |
| `separator/expansion` | `elem (sep elem)*` への展開は生成パーサの構造であり、構文木の比較では観測できない。 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 |
| `reserved/rule-named-token` | 予約語一覧は「特別な意味を持つ単語」と述べるだけで、それらをルール名・トークン名に使えないとは書いていない。 | 受理 | 受理 | 受理 | 受理 |
| `reserved/rule-named-error` | `ERROR` は予約語だが、`ERROR('...')` 以外の位置での参照が禁じられているとは書かれていない。 | 受理 | 受理 | 受理 | 受理 |
| `reserved/token-named-any` | トークンキーワードと同名のトークン宣言に関する規範文が無い。 | 受理 | 受理 | 受理 | 受理 |
| `rule/lowercase-name` | 本文は「大文字始まりの識別子」と書くが、形式的な文法定義の `RuleDecl ::= { Annotation } IDENTIFIER '::=' RuleBody ';'` は大小を制約しない。小文字始まりのルール名を拒否すべきか、警告すべきか、許すかが仕様から決まらない。 | 受理 | 受理 | 受理 | 受理 |
| `rule/dotted-reference-three-segments` | ドットを外側の文法が処理するとしか書かれておらず、namespace が何段まで許されるか、`a.b.Value` の namespace が `a.b` か `a` かが仕様から決まらない。 | 受理 | 拒否 | 受理 | 受理 |
