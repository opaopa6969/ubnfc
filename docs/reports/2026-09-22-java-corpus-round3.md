Java corpus run-all

元 corpus: 127 ファイル、1212 ケース。文法 variant: 279、hash キャッシュ: 234 build、展開後: 1425 件（文法なし操作列を含む）。jobs=4。

集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。

Java generator SHA-256: `b48bcf99ad7004ec742596709ff04193ae26a5630a99e30186e1cde505154fb1`

front SHA-256: `8b29dda85eb2c481d7a6ce0f7a9704ed64d315bb1396ed745d9e217d59a2ab14`

期待拒否 Pass: 28件（Verdict は Pass）。通常の Pass・IR 失敗とは別列に排他的に計上する。build の実測失敗と診断は拒否理由表に保持する。

| 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 0 | 0 | 0 | 0 | 24 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 70 | 0 | 0 | 0 | 0 |
| catalog | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| evolution | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 66 | 82 |
| lexical | 0 | 0 | 0 | 0 | 78 | 0 | 0 | 0 | 0 |
| mixed-values | 0 | 0 | 0 | 0 | 98 | 0 | 0 | 0 | 0 |
| nested-captures | 0 | 0 | 0 | 0 | 10 | 0 | 0 | 0 | 0 |
| numeric-capture | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 | 0 |
| oracle | 0 | 0 | 0 | 0 | 221 | 4 | 17 | 123 | 33 |
| primitives | 0 | 0 | 0 | 0 | 109 | 0 | 0 | 0 | 0 |
| right-associative | 0 | 0 | 0 | 0 | 112 | 0 | 0 | 0 | 0 |
| rule-trivia | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 0 |
| scope-annotations | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 14 | 0 |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 0 | 0 | 0 | 0 | 131 | 0 | 0 | 32 | 0 |
| tiny-code-fence | 0 | 0 | 0 | 0 | 73 | 0 | 0 | 0 | 0 |
| tiny-string-token | 0 | 0 | 0 | 0 | 20 | 0 | 0 | 19 | 0 |
| 合計 | 0 | 0 | 0 | 0 | 1000 | 28 | 17 | 254 | 126 |

6 系統別（case × variant 単位）

| loader 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 0 | 0 | 0 | 0 | 227 | 24 | 0 | 66 | 82 |
| inline-ast | 0 | 0 | 0 | 0 | 43 | 0 | 0 | 14 | 0 |
| operations | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 11 |
| oracle | 0 | 0 | 0 | 0 | 221 | 4 | 17 | 123 | 33 |
| rule | 0 | 0 | 0 | 0 | 322 | 0 | 0 | 51 | 0 |
| token | 0 | 0 | 0 | 0 | 187 | 0 | 0 | 0 | 0 |
| 合計 | 0 | 0 | 0 | 0 | 1000 | 28 | 17 | 254 | 126 |

文法別（case × variant 単位。文法なしは操作列）

| 文法 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| associative/corpus.json:0[normal] | 0 | 0 | 0 | 0 | 24 | 0 | 0 | 0 | 0 |
| associative/corpus.json:1[reversed-metadata] | 0 | 0 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| cardinality/corpus.json:0[zero] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| cardinality/corpus.json:1[plus] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| cardinality/corpus.json:2[bounded] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| cardinality/corpus.json:3[unbounded] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| cardinality/corpus.json:4[separated] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| catalog/corpus.json:0[deduplicated-sorted-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| catalog/corpus.json:1[rule-order-locality-quantifiers-and-escaped-context] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| evolution/conformance.json:0[0] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 20 | 17 |
| evolution/conformance.json:1[1] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 19 | 18 |
| evolution/conformance.json:2[2] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 15 | 22 |
| evolution/conformance.json:3[3] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 12 | 25 |
| evolution/diagnostics.json:0[3] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:0[0:IdentifierParser] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:10[10:SingleQuotedParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:11[11:EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:12[12:org.unlaxer.parser.elementary.EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:13[13:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:14[14:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:15[15:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:16[16:SingleQuotedParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:17[17:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:18[18:SingleQuotedParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:19[19:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:1[1:org.unlaxer.parser.clang.IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:20[20:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:21[21:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:2[2:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:3[3:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:4[4:SingleQuotedParser] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:5[5:DoubleQuotedParser] | 0 | 0 | 0 | 0 | 9 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:8[8:SingleQuotedParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:9[9:DoubleQuotedParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| mixed-values/corpus.json:0[two] | 0 | 0 | 0 | 0 | 23 | 0 | 0 | 0 | 0 |
| mixed-values/corpus.json:1[one] | 0 | 0 | 0 | 0 | 23 | 0 | 0 | 0 | 0 |
| mixed-values/delimited-corpus.json:0[two] | 0 | 0 | 0 | 0 | 15 | 0 | 0 | 0 | 0 |
| mixed-values/delimited-corpus.json:1[one] | 0 | 0 | 0 | 0 | 15 | 0 | 0 | 0 | 0 |
| mixed-values/shared-corpus.json:0[first] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 | 0 |
| mixed-values/shared-corpus.json:1[second] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:0[nested-text-completion] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:1[choice-optional-coexist] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:2[quantifier-items-not-containers] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:3[same-wrapper-and-parser-class] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:4[text-node-and-mixed-values] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:5[recursive-child-boundary] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:6[shared-mapping-nested-and-sequential] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:7[typed-node-list-and-shared-mixed-schema] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| numeric-capture/conformance.json:0[Numeric] | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 | 0 |
| oracle/backref.json:0[@backref-no-scope] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 1 | 0 |
| oracle/catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| oracle/ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/ci.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| oracle/comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 | 0 |
| oracle/common-field-missing.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/common-field-type.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-bounded.json:0[cardinality] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-plus.json:0[cardinality] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-separated.json:0[cardinality] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-unbounded.json:0[cardinality] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-zero.json:0[cardinality] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/conformance-catalog-deduplicated-sorted-captures.json:0[catalog] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-catalog-rule-order-locality-quantifiers-and-escaped-context.json:0[catalog] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-mixed-values-one.json:0[mixed-values] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-nested-text-completion.json:0[nested-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-same-wrapper-and-parser-class.json:0[nested-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-text-node-and-mixed-values.json:0[nested-captures] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/conformance-scope-annotations-capture-site-identity.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-failed-choice-rollback.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-nested-lexical-dynamic.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-nested-same-name-captures.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-scope-and-declares-parent-registration.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-trimmed-group-non-bmp.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-store.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| oracle/digit.json:0[DigitParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| oracle/empty-one-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/empty-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/empty.json:0[EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/enum-collision.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| oracle/enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/enum.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/eof.json:0[EOF] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/error.json:0[ERROR] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/eval-binary_arithmetic.json:0[@eval] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/eval-conditional.json:0[@eval] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/eval-literal.json:0[@eval] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/eval-manual.json:0[@eval] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/eval-passthrough.json:0[@eval] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/eval-sum.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/eval-unknown.json:0[@eval] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/eval-variable_ref.json:0[@eval] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/evolution-v0.json:0[evolution] | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 20 | 4 |
| oracle/evolution-v1.json:0[evolution] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 19 | 4 |
| oracle/evolution-v2.json:0[evolution] | 0 | 0 | 0 | 0 | 18 | 0 | 0 | 15 | 4 |
| oracle/evolution-v3.json:0[evolution] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 12 | 4 |
| oracle/extern-full.json:0[Extern] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/extern.json:0[Extern] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| oracle/import-alias.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/import-cycle.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/import-two-level.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| oracle/lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/lookahead-until.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 1 | 0 |
| oracle/posix.json:0[posix] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 3 | 0 |
| oracle/predictive-extern.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 1 | 1 |
| oracle/recovery-auto.json:0[@recovery] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | 0 |
| oracle/recovery-root.json:0[@recovery] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/recovery-skip.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| oracle/recovery-sync.json:0[@recovery] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | 0 |
| oracle/regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/regex-backref.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/regex-emoji.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0 |
| oracle/regex-prefix.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/regex-zero.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/reobserved-catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-ci.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/reobserved-comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/reobserved-longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-posix.json:0[posix] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-regex-backref.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/repeat-unbounded.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 3 | 0 |
| oracle/repeat-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/separated.json:0[separated] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| oracle/skip.json:0[@skip] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| oracle/typeof.json:0[@typeof] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 2 | 0 |
| oracle/until.json:0[UNTIL] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 2 | 0 |
| primitives/corpus.json:0[0:ANY] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:10[10:UNTIL('*/')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:11[11:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:12[12:UNTIL('#')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:13[13:UNTIL('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:14[14:LOOKAHEAD('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:15[15:NEGATIVE_LOOKAHEAD('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:16[16:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:17[17:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:18[18:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:19[19:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:1[1:CHAR_RANGE('a','z')] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:20[20:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:21[21:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:22[22:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:23[23:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:24[24:UNTIL('#')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:25[25:ANY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:26[26:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:27[27:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:28[28:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:29[29:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:2[2:CHAR_RANGE('あ','お')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:30[30:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:31[31:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:32[32:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:33[33:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:34[34:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:35[35:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:36[36:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:37[37:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:38[38:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:39[39:NEGATION('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:3[3:NEGATION('x😀')] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:40[40:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:41[41:EMPTY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:42[42:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:43[43:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:44[44:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:45[45:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:46[46:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:47[47:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:4[4:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:5[5:EOF] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:6[6:LOOKAHEAD('ab')] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:7[7:NEGATIVE_LOOKAHEAD('x')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:8[8:LOOKAHEAD('😀')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:9[9:UNTIL('#')] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| right-associative/corpus.json:0[text] | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 | 0 |
| right-associative/corpus.json:1[mapped] | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 | 0 |
| right-associative/corpus.json:2[unicode] | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 | 0 |
| right-associative/corpus.json:3[grouped] | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:0[global-omitted] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:10[plain-parent-enabled-default-child] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:11[root-single-literal-choice] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:12[nested-constructs-and-rollback] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:13[left-associative-local-whitespace] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:14[right-associative-local-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:1[global-none] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:2[global-java-style-case-folded] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:3[rule-whitespace-default] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:4[rule-whitespace-java-style-case-folded] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:5[interleave-java-style] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:6[interleave-comments-and-spaces] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:7[explicit-none-before-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:8[explicit-none-after-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:9[enabled-parent-plain-default-child] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:0[capture-site-identity] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| scope-annotations/corpus.json:1[nested-lexical-dynamic] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:2[scope-and-declares-parent-registration] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| scope-annotations/corpus.json:3[failed-choice-rollback] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| scope-annotations/corpus.json:4[repeated-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:5[optional-recursive-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| scope-annotations/corpus.json:6[quantified-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:7[nested-same-name-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:8[trimmed-group-non-bmp] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-store/corpus.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality/corpus.json:0[node-one] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:10[mixed-bounded] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| semantic-cardinality/corpus.json:11[node-separated] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:12[mixed-separated] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:13[optional-many-alias] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 2 | 0 |
| semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 1 | 0 |
| semantic-cardinality/corpus.json:1[mixed-one] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:2[node-pair] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:3[mixed-pair] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 5 | 0 |
| semantic-cardinality/corpus.json:4[mixed-group] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:5[mixed-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:6[node-optional] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 1 | 0 |
| semantic-cardinality/corpus.json:7[mixed-optional] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 2 | 0 |
| semantic-cardinality/corpus.json:8[node-repeat] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 1 | 0 |
| semantic-cardinality/corpus.json:9[mixed-repeat] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 5 | 0 |
| semantic-cardinality/pure-alias.json:0[pure-node-one-alias] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:1[pure-node-multihop-alias] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:2[pure-node-group-alias] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:3[pure-node-delimiter-alias] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:4[pure-node-direct-multi] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:5[pure-node-multi-alias] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:8[pure-node-alias-optional] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 1 | 0 |
| semantic-cardinality/pure-alias.json:9[pure-node-alias-many] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:0[start-direct] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:10[repeat-start] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:11[combined-fence] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:1[end-direct] | 0 | 0 | 0 | 0 | 9 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:2[prefix-vs-full-input-start] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:3[prefix-vs-full-input-end] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:4[start-source-offset-after-crlf] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:5[end-source-offset] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:6[java-style-outer-only] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:7[p4-zero-field-java-style] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:8[transparent-aliases] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:9[optional-start] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:0[direct-eof] | 0 | 0 | 0 | 0 | 9 | 0 | 0 | 11 | 0 |
| tiny-string-token/corpus.json:1[prefix-vs-full-input] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 1 | 0 |
| tiny-string-token/corpus.json:2[comments-and-eof] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| tiny-string-token/corpus.json:3[transparent-alias] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| tiny-string-token/corpus.json:4[optional] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 1 | 0 |
| tiny-string-token/corpus.json:5[repeat] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| tiny-string-token/corpus.json:6[mapped-node] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| 合計 | 0 | 0 | 0 | 0 | 1000 | 28 | 17 | 254 | 126 |

拒否理由（M3 作業リスト）

同一文法の拒否はキャッシュ単位に数える。ケース数は理由ごとに重複除去する。生成器が最初の拒否で停止するため、後続の未対応機能を網羅する件数ではない。IR 診断が複数あると理由間で重複する。

| 段階 / 機能・診断コード | build 数 | 元ケース数 | 展開後件数 |
|---|---:|---:|---:|
| ir/E-IMPORT-CYCLE | 1 | 1 | 1 |
| ir/E-LOWER-COMMON-FIELD | 2 | 2 | 2 |
| ir/E-LOWER-SUM-EVAL | 1 | 1 | 1 |
| ir/E-PRECEDENCE-ORDER | 1 | 24 | 24 |

Fail 詳細（各実行の差分先頭3行。実測応答・全差分・期待値は JSON に保持）

evolution/conformance.json / evolution/v0/0 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/1 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v0/10 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v0/11 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v0/13 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'+'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/14 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'+'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/15 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /captures/1 | {"binding":"Binary:0","name":"left","span":[0,1],"text":"1"} | 欠落 |
| /captures/2 | {"binding":"Binary:1","name":"op","span":[1,2],"text":"+"} | 欠落 |

evolution/conformance.json / evolution/v0/19 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/20 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/21 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/22 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/23 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/24 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/25 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/26 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/27 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/28 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/29 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,2],"text":"12"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'+'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/30 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/33 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

evolution/conformance.json / evolution/v1/0 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/1 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/10 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v1/11 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v1/14 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v1/15 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /captures/1 | {"binding":"Binary:0","name":"left","span":[0,1],"text":"1"} | 欠落 |
| /captures/2 | {"binding":"Binary:1","name":"op","span":[1,2],"text":"+"} | 欠落 |

evolution/conformance.json / evolution/v1/19 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/20 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/21 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/22 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/23 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/24 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/25 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/26 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/27 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/28 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/29 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,2],"text":"12"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v1/30 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/33 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

evolution/conformance.json / evolution/v2/0 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/1 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v2/10 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v2/11 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v2/14 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v2/15 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /captures/1 | {"binding":"Binary:0","name":"left","span":[0,1],"text":"1"} | 欠落 |
| /captures/2 | {"binding":"Binary:1","name":"op","span":[1,2],"text":"+"} | 欠落 |

evolution/conformance.json / evolution/v2/23 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "')'" |
| /diagnostics/0/farthestExpected/1 | "')'" | "'*'" |
| /diagnostics/0/farthestExpected/2 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v2/24 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/25 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/26 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/27 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/28 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/29 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,2],"text":"12"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v2/30 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/33 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

evolution/conformance.json / evolution/v3/0 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'if'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "'neg'" |
| /diagnostics/0/farthestExpected/2 | "')'" | "NUMBER" |

evolution/conformance.json / evolution/v3/1 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'if'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "'neg'" |
| /diagnostics/0/farthestExpected/2 | "'.'" | "NUMBER" |

evolution/conformance.json / evolution/v3/10 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v3/11 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v3/14 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v3/15 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /captures/1 | {"binding":"Binary:0","name":"left","span":[0,1],"text":"1"} | 欠落 |
| /captures/2 | {"binding":"Binary:1","name":"op","span":[1,2],"text":"+"} | 欠落 |

evolution/conformance.json / evolution/v3/23 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "')'" |
| /diagnostics/0/farthestExpected/1 | "')'" | "'*'" |
| /diagnostics/0/farthestExpected/2 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v3/24 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'if'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "'neg'" |
| /diagnostics/0/farthestExpected/2 | "')'" | "NUMBER" |

evolution/conformance.json / evolution/v3/28 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |
| /diagnostics/0/farthestExpected/2 | "'*/'" | "','" |

evolution/conformance.json / evolution/v3/29 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,2],"text":"12"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v3/30 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'if'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "'neg'" |
| /diagnostics/0/farthestExpected/2 | "')'" | "NUMBER" |

evolution/conformance.json / evolution/v3/33 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/backref.json / backref/3 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "AlphabetUnderScoreParser" | "IDENTIFIER" |
| /diagnostics/0/farthestExpected/1 | "IdentifierParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/catalog.json / catalog/3 / oracle/catalog.json:0[@catalog]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "'Z'" | "'z'" |
| /diagnostics/0/farthestExpected/1 | "'z'" | 欠落 |

oracle/char-range.json / char-range/1 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "RootParser" | "CHAR_RANGE" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/char-range.json / char-range/4 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "RootParser" | "CHAR_RANGE" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/char-range.json / char-range/5 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "RootParser" | "CHAR_RANGE" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/ci-unicode.json / ci-unicode/3 / oracle/ci-unicode.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "'é'" | "tokenRef" |

oracle/ci.json / ci/2 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,3],"text":"LeT"} | 欠落 |

oracle/ci.json / ci/4 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "'let'" | "tokenRef" |

oracle/conformance-scope-annotations-capture-site-identity.json / capture-site-identity/target-site-not-first-parser-class / oracle/conformance-scope-annotations-capture-site-identity.json:0[scope-annotations]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |
| /scope/resolved/2/declared | null | 欠落 |

oracle/conformance-scope-annotations-failed-choice-rollback.json / failed-choice-rollback/committed-first-alternative / oracle/conformance-scope-annotations-failed-choice-rollback.json:0[scope-annotations]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |

oracle/conformance-scope-annotations-nested-lexical-dynamic.json / nested-lexical-dynamic/shadow-then-resolve-outer-after-leave / oracle/conformance-scope-annotations-nested-lexical-dynamic.json:0[scope-annotations]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |

oracle/conformance-scope-annotations-nested-same-name-captures.json / nested-same-name-captures/inner-then-outer-capture-occurrences / oracle/conformance-scope-annotations-nested-same-name-captures.json:0[scope-annotations]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |

oracle/conformance-scope-annotations-scope-and-declares-parent-registration.json / scope-and-declares-parent-registration/declaration-is-visible-after-own-scope-leaves / oracle/conformance-scope-annotations-scope-and-declares-parent-registration.json:0[scope-annotations]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |

oracle/conformance-scope-annotations-trimmed-group-non-bmp.json / trimmed-group-non-bmp/ascii-trim-adjusts-code-point-offset-and-length / oracle/conformance-scope-annotations-trimmed-group-non-bmp.json:0[scope-annotations]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |

oracle/digit.json / digit/3 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "RootParser" | "Digit" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/digit.json / digit/4 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "RootParser" | "Digit" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/empty-one-two.json / empty-one-two/1 / oracle/empty-one-two.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/empty.json / empty/1 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/empty.json / empty/3 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/evolution-v0.json / evolution/0 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/1 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v0.json / evolution/10 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v0.json / evolution/11 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v0.json / evolution/13 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'+'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/14 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'+'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/15 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /captures/1 | {"binding":"Binary:0","name":"left","span":[0,1],"text":"1"} | 欠落 |
| /captures/2 | {"binding":"Binary:1","name":"op","span":[1,2],"text":"+"} | 欠落 |

oracle/evolution-v0.json / evolution/19 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/20 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/21 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/22 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/23 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/24 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/25 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/26 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/27 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/28 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/29 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,2],"text":"12"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'+'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/30 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/33 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/evolution-v1.json / evolution/0 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/1 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/10 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v1.json / evolution/11 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v1.json / evolution/14 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

oracle/evolution-v1.json / evolution/15 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /captures/1 | {"binding":"Binary:0","name":"left","span":[0,1],"text":"1"} | 欠落 |
| /captures/2 | {"binding":"Binary:1","name":"op","span":[1,2],"text":"+"} | 欠落 |

oracle/evolution-v1.json / evolution/19 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/20 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/21 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/22 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/23 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/24 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/25 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/26 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/27 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/28 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/29 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,2],"text":"12"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

oracle/evolution-v1.json / evolution/30 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/33 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/evolution-v2.json / evolution/0 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/1 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "'.'" | 欠落 |

oracle/evolution-v2.json / evolution/10 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v2.json / evolution/11 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v2.json / evolution/14 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

oracle/evolution-v2.json / evolution/15 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /captures/1 | {"binding":"Binary:0","name":"left","span":[0,1],"text":"1"} | 欠落 |
| /captures/2 | {"binding":"Binary:1","name":"op","span":[1,2],"text":"+"} | 欠落 |

oracle/evolution-v2.json / evolution/23 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "')'" |
| /diagnostics/0/farthestExpected/1 | "')'" | "'*'" |
| /diagnostics/0/farthestExpected/2 | "'*'" | "'+'" |

oracle/evolution-v2.json / evolution/24 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/25 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/26 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/27 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/28 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/29 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,2],"text":"12"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

oracle/evolution-v2.json / evolution/30 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'neg'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |
| /diagnostics/0/farthestExpected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/33 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/evolution-v3.json / evolution/0 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'if'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "'neg'" |
| /diagnostics/0/farthestExpected/2 | "')'" | "NUMBER" |

oracle/evolution-v3.json / evolution/1 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'if'" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "'neg'" |
| /diagnostics/0/farthestExpected/2 | "'.'" | "NUMBER" |

oracle/evolution-v3.json / evolution/10 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v3.json / evolution/11 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v3.json / evolution/14 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"2"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

oracle/evolution-v3.json / evolution/15 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /captures/1 | {"binding":"Binary:0","name":"left","span":[0,1],"text":"1"} | 欠落 |
| /captures/2 | {"binding":"Binary:1","name":"op","span":[1,2],"text":"+"} | 欠落 |

oracle/evolution-v3.json / evolution/23 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "')'" |
| /diagnostics/0/farthestExpected/1 | "')'" | "'*'" |
| /diagnostics/0/farthestExpected/2 | "'*'" | "'+'" |

oracle/evolution-v3.json / evolution/24 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'if'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "'neg'" |
| /diagnostics/0/farthestExpected/2 | "')'" | "NUMBER" |

oracle/evolution-v3.json / evolution/28 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |
| /diagnostics/0/farthestExpected/2 | "'*/'" | "','" |

oracle/evolution-v3.json / evolution/29 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,2],"text":"12"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "' '" | "'*'" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "'+'" |

oracle/evolution-v3.json / evolution/30 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "' '" | "'if'" |
| /diagnostics/0/farthestExpected/1 | "'('" | "'neg'" |
| /diagnostics/0/farthestExpected/2 | "')'" | "NUMBER" |

oracle/evolution-v3.json / evolution/33 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/extern-full.json / extern-full/3 / oracle/extern-full.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |

oracle/extern.json / extern/1 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |
| /diagnostics/0/farthestOffset | 2 | 1 |

oracle/extern.json / extern/4 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "'x'" | "'😀'" |
| /diagnostics/0/farthestExpected/1 | "'😀'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "RootParser" | 欠落 |

oracle/longest.json / longest/3 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"B:0","name":"value","span":[0,2],"text":"ab"} | 欠落 |

oracle/longest.json / longest/4 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/2 | "RootParser" | 欠落 |

oracle/lookahead-until.json / lookahead-until/1 / oracle/lookahead-until.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 2 | 0 |

oracle/lookahead.json / lookahead/1 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/lookahead.json / lookahead/2 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/lookahead.json / lookahead/3 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "'a'" | "tokenRef" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/lookahead.json / lookahead/4 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "'a'" | "tokenRef" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/mapping-alias.json / mapping-alias/3 / oracle/mapping-alias.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/negation-empty.json / negation-empty/1 / oracle/negation-empty.json:0[NEGATION]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "RootParser" | "NEGATION" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/negative-lookahead.json / negative-lookahead/2 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

oracle/negative-lookahead.json / negative-lookahead/3 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "'a'" | "tokenRef" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/numeric.json / numeric/7 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/posix.json / posix/2 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "RootParser" | "Alphabet" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/posix.json / posix/3 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "RootParser" | "Alphabet" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/posix.json / posix/4 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"A"} | 欠落 |

oracle/predictive-extern.json / predictive-extern/1 / oracle/predictive-extern.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"A:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |
| /diagnostics/0/farthestOffset | 2 | 1 |

oracle/predictive.json / predictive/4 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/2 | "'x'" | 欠落 |

oracle/regex-ascii.json / regex-ascii/1 / oracle/regex-ascii.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,2],"text":"ab"} | 欠落 |

oracle/regex-emoji.json / regex-emoji/1 / oracle/regex-emoji.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /parsedStatus | "failed" | "succeeded" |
| /accepted | false | true |
| /allConsumed | false | true |

oracle/regex-prefix.json / regex-prefix/1 / oracle/regex-prefix.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /parsedStatus | "failed" | "succeeded" |
| /accepted | false | true |
| /allConsumed | false | true |

oracle/regex-prefix.json / regex-prefix/2 / oracle/regex-prefix.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /parsedStatus | "failed" | "succeeded" |
| /accepted | false | true |
| /consumed | 0 | 2 |

oracle/regex-zero.json / regex-zero/1 / oracle/regex-zero.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "REGEX('a*')" | "tokenRef" |

oracle/regex-zero.json / regex-zero/2 / oracle/regex-zero.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/0 | "REGEX('a*')" | "tokenRef" |

oracle/repeat-unbounded.json / repeat-unbounded/1 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/2 | "RootBounded0Parser" | "RootParser" |
| /diagnostics/0/farthestExpected/3 | "RootParser" | 欠落 |

oracle/repeat-unbounded.json / repeat-unbounded/2 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/1 | "RootBounded0Parser" | 欠落 |

oracle/repeat-unbounded.json / repeat-unbounded/4 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"values","span":[0,1],"text":"a"} | 欠落 |
| /captures/1 | {"binding":"Root:0","name":"values","span":[1,2],"text":"a"} | 欠落 |
| /captures/2 | {"binding":"Root:0","name":"values","span":[2,3],"text":"a"} | 欠落 |

oracle/separated.json / separated/1 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "RootSep0Parser" | 欠落 |

oracle/separated.json / separated/4 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"values","span":[0,1],"text":"a"} | 欠落 |

oracle/separated.json / separated/5 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "RootSep0Parser" | 欠落 |

oracle/skip.json / skip/3 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Item:0","name":"value","span":[0,1],"text":"a"} | 欠落 |
| /captures/1 | {"binding":"Root:0","name":"items","span":[0,1],"text":"a"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "__CaptureSite" | 欠落 |

oracle/skip.json / skip/4 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "__CaptureSite" | 欠落 |

oracle/typeof.json / typeof/3 / oracle/typeof.json:0[@typeof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /mappingError | "java.lang.IllegalArgumentException: @typeof constraint violated: right must be same type as left, expected A but got B" | "@typeof constraint violated: right must be same type as left, expected A but got B at Span[start=0, end=3]" |

oracle/typeof.json / typeof/4 / oracle/typeof.json:0[@typeof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /mappingError | "java.lang.IllegalArgumentException: @typeof constraint violated: right must be same type as left, expected B but got A" | "@typeof constraint violated: right must be same type as left, expected B but got A at Span[start=0, end=3]" |

oracle/until.json / until/3 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |
| /diagnostics/0/farthestExpected/0 | "'#'" | "#" |

oracle/until.json / until/4 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

scope-annotations/corpus.json / capture-site-identity/target-site-not-first-parser-class / scope-annotations/corpus.json:0[capture-site-identity]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |
| /scope/resolved/2/declared | null | 欠落 |

scope-annotations/corpus.json / capture-site-identity/undefined-target-warning / scope-annotations/corpus.json:0[capture-site-identity]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |
| /scope/resolved/2/declared | null | 欠落 |

scope-annotations/corpus.json / failed-choice-rollback/committed-first-alternative / scope-annotations/corpus.json:3[failed-choice-rollback]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |

scope-annotations/corpus.json / failed-choice-rollback/failed-first-alternative-erases-effects / scope-annotations/corpus.json:3[failed-choice-rollback]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |

scope-annotations/corpus.json / nested-lexical-dynamic/shadow-then-resolve-outer-after-leave / scope-annotations/corpus.json:1[nested-lexical-dynamic]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |

scope-annotations/corpus.json / nested-same-name-captures/inner-then-outer-capture-occurrences / scope-annotations/corpus.json:7[nested-same-name-captures]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |

scope-annotations/corpus.json / optional-recursive-boundary/optional-target-absent-does-not-use-child-capture / scope-annotations/corpus.json:5[optional-recursive-boundary]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |
| /scope/resolved/2/declared | null | 欠落 |

scope-annotations/corpus.json / optional-recursive-boundary/parent-target-does-not-cross-recursive-child / scope-annotations/corpus.json:5[optional-recursive-boundary]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |
| /scope/resolved/2/declared | null | 欠落 |

scope-annotations/corpus.json / quantified-capture-occurrences/plus-bounded-and-separated-inner-captures / scope-annotations/corpus.json:6[quantified-capture-occurrences]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |
| /scope/resolved/2/declared | null | 欠落 |

scope-annotations/corpus.json / repeated-capture-occurrences/all-repeat-captures-produce-effects / scope-annotations/corpus.json:4[repeated-capture-occurrences]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |
| /scope/resolved/2/declared | null | 欠落 |

scope-annotations/corpus.json / scope-and-declares-parent-registration/declaration-is-visible-after-own-scope-leaves / scope-annotations/corpus.json:2[scope-and-declares-parent-registration]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |

scope-annotations/corpus.json / scope-and-declares-parent-registration/failed-parent-rule-rolls-back-child-commit / scope-annotations/corpus.json:2[scope-and-declares-parent-registration]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |

scope-annotations/corpus.json / scope-and-declares-parent-registration/valid-prefix-with-trailing-input / scope-annotations/corpus.json:2[scope-and-declares-parent-registration]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |
| /scope/resolved/1/declared | null | 欠落 |

scope-annotations/corpus.json / trimmed-group-non-bmp/ascii-trim-adjusts-code-point-offset-and-length / scope-annotations/corpus.json:8[trimmed-group-non-bmp]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/resolved/0/declared | null | 欠落 |

semantic-cardinality/corpus.json / mixed-alias/0 / semantic-cardinality/corpus.json:5[mixed-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,T:a" | "T:(aa)" |

semantic-cardinality/corpus.json / mixed-alias/1 / semantic-cardinality/corpus.json:5[mixed-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | "L:x" |

semantic-cardinality/corpus.json / mixed-alias/2 / semantic-cardinality/corpus.json:5[mixed-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀,L:x" | "L:x" |

semantic-cardinality/corpus.json / mixed-bounded/0 / semantic-cardinality/corpus.json:10[mixed-bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | "T:[a]" |

semantic-cardinality/corpus.json / mixed-bounded/1 / semantic-cardinality/corpus.json:10[mixed-bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | "L:x" |

semantic-cardinality/corpus.json / mixed-group/0 / semantic-cardinality/corpus.json:4[mixed-group]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | "L:x" |

semantic-cardinality/corpus.json / mixed-group/1 / semantic-cardinality/corpus.json:4[mixed-group]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | "L:x" |

semantic-cardinality/corpus.json / mixed-group/2 / semantic-cardinality/corpus.json:4[mixed-group]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀,T:a" | "T:😀a" |

semantic-cardinality/corpus.json / mixed-optional/0 / semantic-cardinality/corpus.json:7[mixed-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | "T:()" |

semantic-cardinality/corpus.json / mixed-optional/4 / semantic-cardinality/corpus.json:7[mixed-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | "T:( /*c*/ )" |

semantic-cardinality/corpus.json / mixed-pair/1 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,T:a" | "T:aa" |

semantic-cardinality/corpus.json / mixed-pair/2 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | "L:x" |

semantic-cardinality/corpus.json / mixed-pair/3 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | "L:x" |

semantic-cardinality/corpus.json / mixed-pair/5 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀,T:a" | "T:😀a" |

semantic-cardinality/corpus.json / mixed-pair/6 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | "L:x" |

semantic-cardinality/corpus.json / mixed-repeat-delimited-alias/3 / semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a),L:x,T:(😀)" | "L:x" |

semantic-cardinality/corpus.json / mixed-repeat-delimited-alias/4 / semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a) /*c*/,T:(😀)" | "T:(a) /*c*/ (😀)" |

semantic-cardinality/corpus.json / mixed-repeat-delimited-inline/3 / semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a),L:x,T:(😀)" | "L:x" |

semantic-cardinality/corpus.json / mixed-repeat/0 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | "T:[]" |

semantic-cardinality/corpus.json / mixed-repeat/1 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | "T:[a]" |

semantic-cardinality/corpus.json / mixed-repeat/3 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x,T:a" | "L:x" |

semantic-cardinality/corpus.json / mixed-repeat/4 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀,L:x" | "L:x" |

semantic-cardinality/corpus.json / mixed-repeat/5 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | "L:x" |

semantic-cardinality/corpus.json / mixed-separated/0 / semantic-cardinality/corpus.json:12[mixed-separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | "T:[a]" |

semantic-cardinality/corpus.json / mixed-separated/1 / semantic-cardinality/corpus.json:12[mixed-separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x,T:😀" | "L:x" |

semantic-cardinality/corpus.json / mixed-separated/2 / semantic-cardinality/corpus.json:12[mixed-separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | "L:x" |

semantic-cardinality/corpus.json / node-optional/0 / semantic-cardinality/corpus.json:6[node-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / node-repeat/0 / semantic-cardinality/corpus.json:8[node-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / optional-many-alias/0 / semantic-cardinality/corpus.json:13[optional-many-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | "T:()" |

semantic-cardinality/corpus.json / optional-many-alias/1 / semantic-cardinality/corpus.json:13[optional-many-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | "T:(a)" |

semantic-cardinality/corpus.json / optional-many-alias/2 / semantic-cardinality/corpus.json:13[optional-many-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | "L:x" |

semantic-cardinality/pure-alias.json / pure-node-alias-optional/0 / semantic-cardinality/pure-alias.json:8[pure-node-alias-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

tiny-string-token/corpus.json / comments-and-eof/block-and-line / tiny-string-token/corpus.json:2[comments-and-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "x" | "'x'" |

tiny-string-token/corpus.json / comments-and-eof/comment-is-raw-inside / tiny-string-token/corpus.json:2[comments-and-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "/*inside*/" | "'/*inside*/'" |

tiny-string-token/corpus.json / direct-eof/empty-single / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "" | "''" |

tiny-string-token/corpus.json / direct-eof/escaped-backslash / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\\\" | "'\\\\'" |

tiny-string-token/corpus.json / direct-eof/escaped-newline / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\\n" | "'\\\n'" |

tiny-string-token/corpus.json / direct-eof/escaped-non-bmp / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\😀" | "'\\😀'" |

tiny-string-token/corpus.json / direct-eof/escaped-single / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a\\'b" | "'a\\'b'" |

tiny-string-token/corpus.json / direct-eof/non-bmp / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "😀" | "'😀'" |

tiny-string-token/corpus.json / direct-eof/nul / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\u0000" | "'\u0000'" |

tiny-string-token/corpus.json / direct-eof/plain-single / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "single" | "'single'" |

tiny-string-token/corpus.json / direct-eof/raw-crlf / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a\r\nb" | "'a\r\nb'" |

tiny-string-token/corpus.json / direct-eof/raw-newline / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a\nb" | "'a\nb'" |

tiny-string-token/corpus.json / direct-eof/unknown-escape / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\q" | "'\\q'" |

tiny-string-token/corpus.json / mapped-node/single-node / tiny-string-token/corpus.json:6[mapped-node]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/fields/text | "node" | "'node'" |

tiny-string-token/corpus.json / optional/single / tiny-string-token/corpus.json:4[optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "x" | "'x'" |

tiny-string-token/corpus.json / prefix-vs-full-input/full / tiny-string-token/corpus.json:1[prefix-vs-full-input]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "ok" | "'ok'" |

tiny-string-token/corpus.json / repeat/non-bmp-repeat / tiny-string-token/corpus.json:5[repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "😀" | "'😀'" |

tiny-string-token/corpus.json / repeat/two-kinds / tiny-string-token/corpus.json:5[repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "'a'" |

tiny-string-token/corpus.json / transparent-alias/single / tiny-string-token/corpus.json:3[transparent-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "alias" | "'alias'" |

未観測項目（展開後件数。欠落を Pass に読み替えない）

| 観測 | 件数 |
|---|---:|
| /captures/0/binding | 82 |
| /captures/1/binding | 34 |
| /captures/10/binding | 2 |
| /captures/11/binding | 1 |
| /captures/12/binding | 1 |
| /captures/13/binding | 1 |
| /captures/14/binding | 1 |
| /captures/2/binding | 32 |
| /captures/3/binding | 30 |
| /captures/4/binding | 30 |
| /captures/5/binding | 7 |
| /captures/6/binding | 3 |
| /captures/7/binding | 2 |
| /captures/8/binding | 2 |
| /captures/9/binding | 2 |
| /catalogs | 4 |
| /diagnosticError | 1 |
| /enumError | 2 |
| /enumValue | 3 |
| /evaluationError | 32 |
| /grammar (操作列の driver protocol 未対応) | 18 |
| /nodes | 1 |
| /operations | 18 |
| /parseError | 1 |
| /rollback | 4 |

段階失敗の原文（完全な標準出力・標準エラーは JSON）

48c346ec23ef1c06-158 / ir: associative/corpus.json:1[reversed-metadata]

> 終了 exit status: 1:
> ubnfc: rule Expression precedence 30 must be lower than referenced operator rule Term precedence 20 [code: E-PRECEDENCE-ORDER] [hint: Decrease Expression level or increase Term level.]

668cf0ed36645997-179 / ir: oracle/common-field-missing.json:0[@commonField]

> 終了 exit status: 1:
> ubnfc: E-LOWER-COMMON-FIELD: ProbeAST.Node.name: variants の field 型/cardinality が不整合

d8ba3c8d7194ffb2-180 / ir: oracle/common-field-type.json:0[@commonField]

> 終了 exit status: 1:
> ubnfc: E-LOWER-COMMON-FIELD: ProbeAST.Node.name: variants の field 型/cardinality が不整合

4aaea9c4ba5b21d8-197 / ir: oracle/eval-sum.json:0[@eval]

> 終了 exit status: 1:
> ubnfc: E-LOWER-SUM-EVAL: Node: 自動 @eval(literal) は具象型が必要

3f95b76dccbb0235-203 / ir: oracle/import-cycle.json:0[@import]

> 終了 exit status: 1:
> ubnfc: 循環 import を検出しました: &lt;build&gt;/grammar/B.ubnf (スタック: [&lt;build&gt;/grammar/__runall__.ubnf, &lt;build&gt;/grammar/B.ubnf, &lt;build&gt;/grammar/C.ubnf]) [code: E-IMPORT-CYCLE] [hint: Remove the circular import.]


## round3 比較と根拠

D-019 により `nativeFailure` を Verdict/未観測の判定から除外し、期待値と実測値は JSON に保持した。公開診断は従来どおり比較する。

| 記録 | Pass | Fail | 実行エラー | 元ケース | 展開後ケース |
|---|---:|---:|---:|---:|---:|
| round2 | 715 | 406 | 136 | 1212 | 1425 |
| round3 | 1000 | 254 | 0 | 1212 | 1425 |

保存済み round2 と現 checkout の比較。Rust は元ケース871→1212、展開後1195→1425であり、母数と分類の更新を含む。scanner 単独の効果測定には使わない。期待拒否 Pass は通常 Pass と別集計。

round3 の期待拒否 Pass は 28 件。run-all は残る Fail/生成拒否により終了コード1であり、実行基盤の異常終了ではない。

Fail の上位パターン（ケース単位、同じケースは複数パターンに含まれ得る）:

| 比較パス | ケース数 |
|---|---:|
| `/diagnostics/*/farthestExpected` | 163 |
| `/captures/*` | 70 |
| `/value` | 32 |
| `/scope/resolved` | 20 |
| `/ast/fields` | 19 |
| `/diagnostics/*/farthestOffset` | 14 |
| `/accepted` | 3 |
| `/matched` | 3 |

Java round2 の `error` 内の token ID を、その build の `source` に結び付けて得た未登録クラス:

| クラス名 | 実行エラー数 |
|---|---:|
| `DoubleQuotedParser` | 14 |
| `org.unlaxer.parser.elementary.DoubleQuotedParser` | 1 |
| `oracle.ExternalCursorParser` | 9 |
| `org.unlaxer.tinyexpression.parser.javalang.CodeStartParser` | 58 |
| `org.unlaxer.tinyexpression.parser.javalang.CodeEndParser` | 15 |
| `org.unlaxer.tinyexpression.parser.StringLiteralParser` | 39 |

同梱集合は上記に加えて `SingleQuotedParser` と完全名 alias も提供する。各クラスの受理範囲・根拠のファイルと行番号・登録手順は [driver protocol](../harness-driver-protocol.md#extern-scanner-の登録round3) と `scanners/java/Registry.java` / `scanners/rust/scanners.rs` に記載した。外部依存は追加していない。

旧実行エラー136件は113 Pass / 23 Failへ移行し、実行エラーは0件。旧 nativeFailure 差分380件は104 Pass / 173 Fail / 99 未観測 / 4 Knownへ移行した。nativeFailure と公開診断の差分が同居するケースでは Fail が残る。

tiny-string-token の single quote 内容抽出19件は mapper が消費範囲を優先する差分。scanner は内側の value span を返す。

scanner の登録 E2E は direct-eof 文法の double quote/拒否入力で検証。single quote、5状態、Unicode、value span は scanner API の契約テストで検証。

Rust の scanner entry hook は既存 parser の生成結果を driver 内にも配置するため、生成ソース量とコンパイル量は増える。

検証:

- cargo test -p ubnfc-harness: 成功
- mvn -o -q test (ubnfc-java): 135 tests、失敗0、エラー0、skip0
- cargo clippy: 成功
- cargo fmt --check: 成功
- cargo test -p ubnfc-rust --test driver: 4 tests 成功
- cargo test -p ubnfc-harness --test scanners: 分離 directory/jar、環境変数、非root entry、両言語scanner契約テスト成功
- rustfmt --edition 2021 --check scanners/rust/scanners.rs scanners/rust/scanners_tests.rs: 成功

`nativeFailure` 保存: 期待値 485 件、実測値 1379 件。判定差分は0件。scanner の SHA-256 は JSON の `round3_analysis.scanner_sha256` に保存した。
