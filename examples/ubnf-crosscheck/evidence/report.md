# UBNF 4 実装クロスチェック（自動生成）

入力 48 件（`corpus.tsv` 参照）。

## 受理・拒否

| 入力 | ubnfc-front(手書き) | ubnfc-selfhost(生成) | unlaxer-ubnf(Rust) | Java bootstrap |
|---|---|---|---|---|
| `negative__block-comment.ubnf` | 拒否@12 | 受理 | 拒否@12 | 拒否 |
| `negative__bound-overflow.ubnf` | 受理 | 受理 | 拒否@22 | 拒否 |
| `negative__double-quantifier.ubnf` | 拒否@22 | 拒否@22 | 拒否@22 | 拒否 |
| `negative__double-quoted-string.ubnf` | 拒否@18 | 拒否@18 | 拒否@18 | 拒否 |
| `negative__empty-choice.ubnf` | 受理 | 受理 | 拒否@24 | 拒否 |
| `negative__empty-file.ubnf` | 受理 | 受理 | 拒否@0 | 拒否 |
| `negative__empty-grammar.ubnf` | 受理 | 受理 | 拒否@11 | 拒否 |
| `negative__empty-group.ubnf` | 受理 | 受理 | 拒否@19 | 拒否 |
| `negative__empty-params.ubnf` | 拒否@31 | 拒否@31 | 拒否@31 | 拒否 |
| `negative__empty-range.ubnf` | 受理 | 受理 | 拒否@31 | 拒否 |
| `negative__empty-sequence.ubnf` | 受理 | 受理 | 拒否@18 | 拒否 |
| `negative__late-import.ubnf` | 拒否@33 | 拒否@33 | 拒否@33 | 拒否 |
| `negative__late-token.ubnf` | 拒否@29 | 拒否@29 | 拒否@29 | 拒否 |
| `negative__long-range.ubnf` | 受理 | 受理 | 拒否@31 | 拒否 |
| `negative__missing-semicolon.ubnf` | 拒否@22 | 拒否@22 | 拒否@22 | 拒否 |
| `negative__precedence-overflow.ubnf` | 受理 | 受理 | 拒否@30 | 拒否 |
| `negative__recovery-mode.ubnf` | 受理 | 受理 | 拒否@22 | 拒否 |
| `negative__reversed-range.ubnf` | 受理 | 受理 | 拒否@20 | 拒否 |
| `negative__supplementary-range.ubnf` | 受理 | 受理 | 拒否@31 | 拒否 |
| `negative__trailing-input.ubnf` | 拒否@25 | 拒否@25 | 拒否@25 | 受理 |
| `negative__unknown-annotation-args.ubnf` | 拒否@20 | 拒否@20 | 拒否@12 | 拒否 |
| `negative__unknown-token-constructor.ubnf` | 拒否@27 | 拒否@27 | 拒否@20 | 拒否 |
| `negative__unterminated-string.ubnf` | 拒否@18 | 拒否@18 | 拒否@18 | 拒否 |
| `rust_unlaxer-ubnf_tests_fixtures_known_common-field-space.ubnf` | 受理 | 受理 | 受理 | 拒否 |
| `rust_unlaxer-ubnf_tests_fixtures_known_eval-default.ubnf` | 受理 | 拒否@32 | 受理 | 受理 |
| `rust_unlaxer-ubnf_tests_fixtures_known_keyword-boundary.ubnf` | 受理 | 拒否@24 | 受理 | 拒否 |
| `rust_unlaxer-ubnf_tests_fixtures_known_namespace.ubnf` | 受理 | 拒否@28 | 受理 | 受理 |
| `rust_unlaxer-ubnf_tests_fixtures_positive_all-tokens.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `rust_unlaxer-ubnf_tests_fixtures_positive_annotations.ubnf` | 受理 | 拒否@43 | 受理 | 受理 |
| `rust_unlaxer-ubnf_tests_fixtures_positive_boundaries.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `rust_unlaxer-ubnf_tests_fixtures_positive_elements.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `story_code_calc-leftrec.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `story_code_calc-var.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `story_code_calc.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `tools_tinyexpression-p4-lsp-vscode_grammar_tinyexpression-p4.ubnf` | 受理 | 拒否@2595 | 受理 | 受理 |
| `unlaxer-dsl_examples_tinycalc.ubnf` | 受理 | 拒否@276 | 拒否@276 | 拒否 |
| `unlaxer-dsl_grammar_ubnf.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_associative_Operators.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_cardinality_Cardinality.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_evolution_0_Evolution.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_evolution_1_Evolution.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_evolution_2_Evolution.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_evolution_3_Evolution.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_mixed-values_Delimited.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_mixed-values_Mixed.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_mixed-values_Shared.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_src_test_resources_right-associative_Power.ubnf` | 受理 | 受理 | 受理 | 受理 |
| `unlaxer-dsl_tinycalc-vscode_grammar_tinycalc.ubnf` | 受理 | 受理 | 受理 | 受理 |

## 受理判定の不一致 21 件

- `negative__block-comment.ubnf`: 受理 = ubnfc-selfhost(生成) / 拒否 = ubnfc-front(手書き), unlaxer-ubnf(Rust), Java bootstrap
    - ubnfc-front(手書き): UBNF のコメントは // 行コメントのみです (bytes 12..13); expected identifier or single-quoted string or punctuation
    - unlaxer-ubnf(Rust): UBNF block comments are unsupported; use // comments
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: grammar G { /* no */ R ::= 'x'; }
- `negative__bound-overflow.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): integer exceeds 2147483647
    - Java bootstrap: java.lang.NumberFormatException: For input string: "2147483648"
- `negative__empty-choice.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): expected nonempty rule sequence (use an EMPTY token for epsilon)
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: grammar G { R ::= 'a' | ; }
- `negative__empty-file.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): expected at least one grammar
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: 
- `negative__empty-grammar.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): grammar requires at least one rule
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: grammar G {}
- `negative__empty-group.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): expected nonempty rule sequence (use an EMPTY token for epsilon)
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: grammar G { R ::= (); }
- `negative__empty-range.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): CHAR_RANGE boundary requires exactly one non-surrogate BMP character
    - Java bootstrap: java.lang.IllegalArgumentException: CHAR_RANGE token T minimum must be one non-surrogate BMP character
- `negative__empty-sequence.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): expected nonempty rule sequence (use an EMPTY token for epsilon)
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: grammar G { R ::= ; }
- `negative__long-range.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): CHAR_RANGE boundary requires exactly one non-surrogate BMP character
    - Java bootstrap: java.lang.IllegalArgumentException: CHAR_RANGE token T minimum must be one non-surrogate BMP character
- `negative__precedence-overflow.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): integer exceeds 2147483647
    - Java bootstrap: java.lang.NumberFormatException: For input string: "2147483648"
- `negative__recovery-mode.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): expected recovery mode auto, skip or sync
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: grammar G { @recovery(magic) R ::= 'x'; }
- `negative__reversed-range.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): CHAR_RANGE minimum exceeds maximum
    - Java bootstrap: java.lang.IllegalArgumentException: CHAR_RANGE token T requires minimum <= maximum
- `negative__supplementary-range.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成) / 拒否 = unlaxer-ubnf(Rust), Java bootstrap
    - unlaxer-ubnf(Rust): CHAR_RANGE boundary requires exactly one non-surrogate BMP character
    - Java bootstrap: java.lang.IllegalArgumentException: CHAR_RANGE token T minimum must be one non-surrogate BMP character
- `negative__trailing-input.ubnf`: 受理 = Java bootstrap / 拒否 = ubnfc-front(手書き), ubnfc-selfhost(生成), unlaxer-ubnf(Rust)
    - ubnfc-front(手書き): 予期しないトークン: Identifier("garbage") (bytes 25..32); expected grammar
    - ubnfc-selfhost(生成): 構文エラー (bytes 25..26): expected grammar
    - unlaxer-ubnf(Rust): expected grammar
- `rust_unlaxer-ubnf_tests_fixtures_known_common-field-space.ubnf`: 受理 = ubnfc-front(手書き), ubnfc-selfhost(生成), unlaxer-ubnf(Rust) / 拒否 = Java bootstrap
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: grammar Spaces { @commonField(left, right) Root ::= 'x'; }

- `rust_unlaxer-ubnf_tests_fixtures_known_eval-default.ubnf`: 受理 = ubnfc-front(手書き), unlaxer-ubnf(Rust), Java bootstrap / 拒否 = ubnfc-selfhost(生成)
    - ubnfc-selfhost(生成): 構文エラー (bytes 32..33): expected :
- `rust_unlaxer-ubnf_tests_fixtures_known_keyword-boundary.ubnf`: 受理 = ubnfc-front(手書き), unlaxer-ubnf(Rust) / 拒否 = ubnfc-selfhost(生成), Java bootstrap
    - ubnfc-selfhost(生成): 構文エラー (bytes 24..25): expected : or ::=
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: grammar Names { @rooted Root ::= 'x'; }

- `rust_unlaxer-ubnf_tests_fixtures_known_namespace.ubnf`: 受理 = ubnfc-front(手書き), unlaxer-ubnf(Rust), Java bootstrap / 拒否 = ubnfc-selfhost(生成)
    - ubnfc-selfhost(生成): 構文エラー (bytes 28..29): expected + or ? or * or { or % or @ or @typeof or ( or [ or ERROR or STRING or IDENTIFIER or | or ;
- `rust_unlaxer-ubnf_tests_fixtures_positive_annotations.ubnf`: 受理 = ubnfc-front(手書き), unlaxer-ubnf(Rust), Java bootstrap / 拒否 = ubnfc-selfhost(生成)
    - ubnfc-selfhost(生成): 構文エラー (bytes 43..44): expected , or )
- `tools_tinyexpression-p4-lsp-vscode_grammar_tinyexpression-p4.ubnf`: 受理 = ubnfc-front(手書き), unlaxer-ubnf(Rust), Java bootstrap / 拒否 = ubnfc-selfhost(生成)
    - ubnfc-selfhost(生成): 構文エラー (bytes 2595..2596): expected STRING
- `unlaxer-dsl_examples_tinycalc.ubnf`: 受理 = ubnfc-front(手書き) / 拒否 = ubnfc-selfhost(生成), unlaxer-ubnf(Rust), Java bootstrap
    - ubnfc-selfhost(生成): 構文エラー (bytes 276..277): expected STRING
    - unlaxer-ubnf(Rust): unexpected character '"'
    - Java bootstrap: java.lang.IllegalArgumentException: UBNF パース失敗: // TinyCalc DSL サンプル
// UBNF (Unlaxer BNF) 記法の実証用。
// unlaxer-common の example

## 受理したが AST が食い違う入力 3 件

- `rust_unlaxer-ubnf_tests_fixtures_known_eval-default.ubnf`: 一致群 = A / C / D
    - ubnfc-front(手書き) vs unlaxer-ubnf(Rust): $[1][0][5][0][1][0]: 要素数 2 / 4 — ['EvalAnnotation', [['KeyValuePair', 'kind', 'variable_ref'], ['KeyValuePair', 'strip_prefix', '$']]] vs ['EvalAnnotation', 'variable_ref', 'default', {'strip_prefix': '$'}]
    - unlaxer-ubnf(Rust) vs Java bootstrap: $[1][0][5][0][1][0][2]: 'default' / '$'
- `rust_unlaxer-ubnf_tests_fixtures_known_namespace.ubnf`: 一致群 = A+C / D
    - ubnfc-front(手書き) vs Java bootstrap: $[1][0][5][0][3][1][0][0][1][1]: 'a.b' / 'a'
- `rust_unlaxer-ubnf_tests_fixtures_positive_annotations.ubnf`: 一致群 = A / C+D
    - ubnfc-front(手書き) vs unlaxer-ubnf(Rust): $[1][0][5][0][1][2]: 要素数 2 / 4 — ['EvalAnnotation', [['KeyValuePair', 'kind', 'binary'], ['KeyValuePair', 'strategy', 'manual'], ['KeyValuePair', 'operation', '+'], ['KeyValuePair', 'result', 'number']]] vs ['EvalAnnotation', 'binary', 'manual', {'operation': '+', 'result': 'number'}]

## 拒否位置（byte offset）が食い違う入力

- `negative__unknown-annotation-args.ubnf`: ubnfc-front(手書き)=20, ubnfc-selfhost(生成)=20, unlaxer-ubnf(Rust)=12
- `negative__unknown-token-constructor.ubnf`: ubnfc-front(手書き)=27, ubnfc-selfhost(生成)=27, unlaxer-ubnf(Rust)=20
