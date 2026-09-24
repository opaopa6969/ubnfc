# UBNF 仕様由来オラクル — 4 実装の突き合わせ（自動生成）

期待値は `unlaxer-dsl/specs/*.md` の規範文から導いたもので、
どの実装の観測値も使っていない（`corpus/spec/ubnf/README.md`）。

ケース 129 件。判定は 一致 / 一致(validate) / 違反 / 別コード / 未定（仕様が沈黙）/ 駆動不能。

## 1. 節ごとの件数と違反数

| 節 | ケース | ubnfc-front(手書き) 違反 | ubnfc-selfhost(生成) 違反 | unlaxer-ubnf(Rust) 違反 | Java bootstrap 違反 |
|---|---:|---:|---:|---:|---:|
| `annotations` (アノテーション) | 32 | 0 | 0 | 0 | 0 |
| `char-range` (CHAR_RANGE('min','max')) | 11 | 0 | 0 | 0 | 0 |
| `comments` (コメント) | 5 | 0 | 0 | 0 | 0 |
| `formal-grammar` (形式的な文法定義) | 23 | 8 | 8 | 0 | 0 |
| `global-settings` (グローバル設定) | 10 | 0 | 0 | 0 | 0 |
| `literals-escapes` (ルール本体の書き方 / リテラル) | 9 | 0 | 0 | 0 | 0 |
| `quantifiers` (量指定子とセパレータ) | 16 | 2 | 2 | 0 | 0 |
| `reserved-words` (予約語) | 3 | 0 | 0 | 0 | 0 |
| `rule-declaration` (ルール宣言) | 10 | 0 | 1 | 0 | 0 |
| `token-declarations` (トークン宣言（token）) | 10 | 0 | 0 | 0 | 0 |
| **合計** | **129** | **10** | **11** | **0** | **0** |

## 2. 判定の内訳

| 実装 | 一致 | 一致(validate) | 違反 | 別コード | 未定 | 駆動不能 |
|---|---:|---:|---:|---:|---:|---:|
| ubnfc-front(手書き) | 108 | 8 | 10 | 0 | 0 | 3 |
| ubnfc-selfhost(生成) | 89 | 0 | 11 | 0 | 0 | 29 |
| unlaxer-ubnf(Rust) | 108 | 0 | 0 | 0 | 0 | 21 |
| Java bootstrap | 125 | 0 | 0 | 0 | 0 | 4 |

`一致(validate)` は構文解析では受理するが同じ道具の validate 段が ERROR で拒否する場合。
仕様は「拒否する」としか書いておらず、どの層で拒否するかは定めていない。
`別コード` は validation.md のコード名と違う名前で同じ違反を検出した場合。

## 3. ケース × 実装

| ケース | scope | 期待 | ubnfc-front(手書き) | ubnfc-selfhost(生成) | unlaxer-ubnf(Rust) | Java bootstrap |
|---|---|---|---|---|---|---|
| `annotation/root` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/root-two-rules` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/mapping-class-only` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/mapping-params` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `annotation/mapping-empty-params` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `annotation/mapping-missing-capture` | validate | E-MAPPING-MISSING-CAPTURE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/mapping-extra-capture` | validate | E-MAPPING-UNLISTED-CAPTURE | 一致 | 駆動不能 | 駆動不能 | 一致 |
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
| `annotation/precedence-without-assoc` | validate | E-PRECEDENCE-NO-ASSOC | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `annotation/assoc-without-precedence` | validate | E-ASSOC-NO-PRECEDENCE | 一致 | 駆動不能 | 駆動不能 | 一致 |
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
| `spec-contradiction/minilang-leftassoc` | validate | E-ASSOC-NO-PRECEDENCE | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `char-range/valid` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `char-range/equal-bounds` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `char-range/empty-min` | parse | reject | 一致(validate) | 駆動不能 | 一致 | 一致 |
| `char-range/empty-max` | parse | reject | 一致(validate) | 駆動不能 | 一致 | 一致 |
| `char-range/multi-char-min` | parse | reject | 一致(validate) | 駆動不能 | 一致 | 一致 |
| `char-range/multi-char-max` | parse | reject | 一致(validate) | 駆動不能 | 一致 | 一致 |
| `char-range/supplementary` | parse | reject | 一致(validate) | 駆動不能 | 一致 | 一致 |
| `char-range/reversed` | parse | reject | 一致(validate) | 駆動不能 | 一致 | 一致 |
| `char-range/escaped-bounds` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `char-range/escaped-bounds-reversed` | parse | reject | 一致(validate) | 駆動不能 | 一致 | 一致 |
| `char-range/inclusive-bounds` | runtime | undetermined | 駆動不能 | 駆動不能 | 駆動不能 | 駆動不能 |
| `comment/line-accepted` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `comment/line-inside-body` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `comment/block-rejected` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `comment/block-in-body-rejected` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `comment/block-unclosed-rejected` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `file/empty-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `file/only-comments-rejected` | parse | reject | 違反 | 違反 | 一致 | 一致 |
| `file/two-grammars` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `file/leading-comment-then-grammar` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `file/trailing-garbage-rejected` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `grammar/empty-block` | parse | accept | 一致 | 一致 | 一致 | 一致 |
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
| `error/element-needs-string` | validate | E-RULE-UNDEFINED | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `error/element-needs-argument` | parse | reject | 一致(validate) | 駆動不能 | 一致 | 一致 |
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
| `escape/unknown-sequence` | parse | accept | 一致 | 一致 | 一致 | 一致 |
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
| `quantifier/reversed-bounds` | validate | E-BOUNDED-INVALID | 一致 | 駆動不能 | 駆動不能 | 一致 |
| `quantifier/huge-bound` | validate | E-BOUNDED-OVERFLOW | 一致 | 駆動不能 | 駆動不能 | 駆動不能 |
| `separator/rule-reference` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `separator/expansion` | runtime | undetermined | 駆動不能 | 駆動不能 | 駆動不能 | 駆動不能 |
| `reserved/rule-named-token` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `reserved/rule-named-error` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `reserved/token-named-any` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/semicolon-required` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `rule/semicolon-present` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/assign-token-required` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `rule/lowercase-name` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/two-refs-are-two-elements` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/comment-is-identifier-boundary` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/captures-kept-separately` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/space-inside-identifier` | parse | reject | 一致 | 一致 | 一致 | 一致 |
| `rule/dotted-reference` | parse | accept | 一致 | 一致 | 一致 | 一致 |
| `rule/dotted-reference-three-segments` | parse | accept | 一致 | 違反 | 一致 | 一致 |
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

### ubnfc-front(手書き) — 10 件

| ケース | 入力 | 期待 | 実測 | 引用 |
|---|---|---|---|---|
| `file/empty-rejected` | `(空ファイル)` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `file/only-comments-rejected` | `// just a comment\n` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-sequence-rejected` | `grammar G { R ::= ; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-alternative-rejected` | `grammar G { R ::= 'a' \| ; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/leading-bar-rejected` | `grammar G { R ::= \| 'a'; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-group-rejected` | `grammar G { R ::= (); }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-optional-rejected` | `grammar G { R ::= [ ]; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-repeat-rejected` | `grammar G { R ::= { }; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `quantifier/two-quantifiers` | `grammar G { token V=ANY R ::= V+{2,3}; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `quantifier/quantifier-and-separator` | `grammar G { token V=ANY token S=ANY R ::= V+ % S; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |

### ubnfc-selfhost(生成) — 11 件

| ケース | 入力 | 期待 | 実測 | 引用 |
|---|---|---|---|---|
| `file/empty-rejected` | `(空ファイル)` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `file/only-comments-rejected` | `// just a comment\n` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-sequence-rejected` | `grammar G { R ::= ; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-alternative-rejected` | `grammar G { R ::= 'a' \| ; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/leading-bar-rejected` | `grammar G { R ::= \| 'a'; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-group-rejected` | `grammar G { R ::= (); }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-optional-rejected` | `grammar G { R ::= [ ]; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `body/empty-repeat-rejected` | `grammar G { R ::= { }; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `quantifier/two-quantifiers` | `grammar G { token V=ANY R ::= V+{2,3}; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `quantifier/quantifier-and-separator` | `grammar G { token V=ANY token S=ANY R ::= V+ % S; }` | reject | 受理 | ubnf-syntax.md ## 形式的な文法定義 |
| `rule/dotted-reference-three-segments` | `grammar G { R ::= a.b.Value; }` | accept | 拒否: 構文エラー (bytes 21..22): expected + or ? or * or { or % or @ or @typeof or ( or [ o | ubnf-syntax.md ## ルール宣言 |

### unlaxer-ubnf(Rust) — 0 件

なし

### Java bootstrap — 0 件

なし

## 5. 未定（仕様が沈黙・矛盾） 3 件

| ケース | 問い | ubnfc-front(手書き) | ubnfc-selfhost(生成) | unlaxer-ubnf(Rust) | Java bootstrap |
|---|---|---|---|---|---|
| `char-range/inclusive-bounds` | unlaxer-parser #279 で仕様書は「観測できない（scope: runtime）」と明記した。ただしこれは runtime scope の性質そのものであり、この corpus 形式が4実装の構文木・診断の比較でしか判定できない以上、accept/reject/diagnostic のいずれにも決められない。他の runtime scope ケースと同じ扱いとして undetermined のまま残す。 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 |
| `error/hint-is-last-alternative` | unlaxer-parser #279 で仕様書は「観測できない（scope: runtime）」と明記した。char-range/inclusive-bounds と同じ理由で、この corpus 形式は runtime scope を accept/reject/diagnostic に決められないため undetermined のまま残す。 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 |
| `separator/expansion` | unlaxer-parser #279 で仕様書は「観測できない（scope: runtime）」と明記した。char-range/inclusive-bounds と同じ理由で、この corpus 形式は runtime scope を accept/reject/diagnostic に決められないため undetermined のまま残す。 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 | 生成パーサの実行時挙動 |
