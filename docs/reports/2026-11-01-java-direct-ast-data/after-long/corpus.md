Java corpus run-all

元 corpus: 127 ファイル、1279 ケース。文法 variant: 307、hash キャッシュ: 260 build、展開後: 1492 件（文法なし操作列を含む）。jobs=4。

集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。

Java generator SHA-256: `b14813f35ceb05dfa12114d09b4ea08973d03d421428c2c8ca68c47633014e29`

front SHA-256: `f20a2eb078b509fa8fdce3319765847542a525d01754d877e07f788a2a1a86e4`

期待拒否 Pass: 27件（Verdict は Pass）。通常の Pass・IR 失敗とは別列に排他的に計上する。build の実測失敗と診断は拒否理由表に保持する。

| 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 0 | 0 | 0 | 0 | 24 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 70 | 0 | 0 | 0 | 0 |
| catalog | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| evolution | 0 | 0 | 0 | 0 | 140 | 0 | 0 | 0 | 16 |
| lexical | 0 | 0 | 0 | 0 | 78 | 0 | 0 | 0 | 0 |
| mixed-values | 0 | 0 | 0 | 0 | 98 | 0 | 0 | 0 | 0 |
| nested-captures | 0 | 0 | 0 | 0 | 10 | 0 | 0 | 0 | 0 |
| numeric-capture | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 | 0 |
| oracle | 0 | 0 | 0 | 0 | 332 | 3 | 29 | 0 | 34 |
| primitives | 0 | 0 | 0 | 0 | 113 | 0 | 0 | 2 | 0 |
| right-associative | 0 | 0 | 0 | 0 | 112 | 0 | 0 | 0 | 0 |
| rule-trivia | 44 | 0 | 0 | 0 | 34 | 0 | 0 | 1 | 2 |
| scope-annotations | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 0 | 0 | 0 | 0 | 163 | 0 | 0 | 0 | 0 |
| tiny-code-fence | 0 | 0 | 0 | 0 | 86 | 0 | 0 | 0 | 0 |
| tiny-string-token | 0 | 0 | 0 | 0 | 39 | 0 | 0 | 0 | 0 |
| 合計 | 44 | 0 | 0 | 0 | 1328 | 27 | 29 | 3 | 61 |

6 系統別（case × variant 単位）

| loader 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 0 | 0 | 0 | 0 | 359 | 24 | 0 | 0 | 16 |
| inline-ast | 44 | 0 | 0 | 0 | 58 | 0 | 0 | 1 | 2 |
| operations | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 9 |
| oracle | 0 | 0 | 0 | 0 | 332 | 3 | 29 | 0 | 34 |
| rule | 0 | 0 | 0 | 0 | 386 | 0 | 0 | 0 | 0 |
| token | 0 | 0 | 0 | 0 | 191 | 0 | 0 | 2 | 0 |
| 合計 | 44 | 0 | 0 | 0 | 1328 | 27 | 29 | 3 | 61 |

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
| catalog/corpus.json:0[deduplicated-sorted-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| catalog/corpus.json:1[rule-order-locality-quantifiers-and-escaped-context] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| evolution/conformance.json:0[0] | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 4 |
| evolution/conformance.json:1[1] | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 4 |
| evolution/conformance.json:2[2] | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 4 |
| evolution/conformance.json:3[3] | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 4 |
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
| oracle/backref.json:0[@backref-no-scope] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/ci.json:0[CI] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | 0 |
| oracle/comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | 0 |
| oracle/comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 | 0 |
| oracle/common-field-missing.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/common-field-type.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| oracle/common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-bounded.json:0[cardinality] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-plus.json:0[cardinality] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-separated.json:0[cardinality] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-unbounded.json:0[cardinality] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-zero.json:0[cardinality] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/conformance-catalog-deduplicated-sorted-captures.json:0[catalog] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-catalog-rule-order-locality-quantifiers-and-escaped-context.json:0[catalog] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-mixed-values-one.json:0[mixed-values] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-nested-text-completion.json:0[nested-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-same-wrapper-and-parser-class.json:0[nested-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-text-node-and-mixed-values.json:0[nested-captures] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/conformance-scope-annotations-capture-site-identity.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-annotations-failed-choice-rollback.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-scope-annotations-nested-lexical-dynamic.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-annotations-nested-same-name-captures.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-annotations-scope-and-declares-parent-registration.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-scope-annotations-trimmed-group-non-bmp.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-store.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| oracle/digit.json:0[DigitParser] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/empty-one-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/empty-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/empty-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/empty.json:0[EMPTY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
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
| oracle/evolution-v0.json:0[evolution] | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 4 |
| oracle/evolution-v1.json:0[evolution] | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 4 |
| oracle/evolution-v2.json:0[evolution] | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 4 |
| oracle/evolution-v3.json:0[evolution] | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 4 |
| oracle/extern-full.json:0[Extern] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/extern.json:0[Extern] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/import-alias.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/import-cycle.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/import-two-level.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/lookahead-until.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/lookahead.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| oracle/posix.json:0[posix] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/predictive-extern.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 4 | 0 | 1 | 0 | 0 |
| oracle/recovery-auto.json:0[@recovery] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | 0 |
| oracle/recovery-root.json:0[@recovery] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 | 0 |
| oracle/recovery-skip.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| oracle/recovery-sync.json:0[@recovery] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | 0 |
| oracle/regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/regex-backref.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/regex-emoji.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| oracle/regex-prefix.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| oracle/regex-zero.json:0[REGEX] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-ci.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | 0 |
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
| oracle/repeat-unbounded.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/repeat-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/separated.json:0[separated] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/skip.json:0[@skip] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/typeof.json:0[@typeof] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/until.json:0[UNTIL] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
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
| primitives/corpus.json:35[35:EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:36[36:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:37[37:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:38[38:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:39[39:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:3[3:NEGATION('x😀')] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:40[40:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:41[41:NEGATIVE_LOOKAHEAD('x')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:42[42:NEGATION('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:43[43:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:44[44:EMPTY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:45[45:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:46[46:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:47[47:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:48[48:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:49[49:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:4[4:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:50[50:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
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
| rule-trivia/corpus.json:15[NamedGlobal] | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:16[NamedRule] | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:17[imported-layout] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:18[imported-rule-layout] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:19[named-token-atomic] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:1[global-none] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:20[named-child-policy-restoration] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:21[named-overrides-interleave] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:22[BuiltinJavaReserved] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 1 |
| rule-trivia/corpus.json:23[BuiltinNoneReserved] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| rule-trivia/corpus.json:24[standard-layout-global-java-style-case-folded] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:25[standard-layout-rule-whitespace-default] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:26[standard-layout-rule-whitespace-java-style-case-folded] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:27[standard-layout-interleave-java-style] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:28[standard-layout-explicit-none-before-interleave] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:29[standard-layout-explicit-none-after-interleave] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:2[global-java-style-case-folded] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:30[standard-layout-enabled-parent-plain-default-child] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:31[standard-layout-plain-parent-enabled-default-child] | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:32[standard-layout-root-single-literal-choice] | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:33[standard-layout-nested-constructs-and-rollback] | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:34[standard-layout-left-associative-local-whitespace] | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:35[local-package-layout] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:36[transitive-package-layout] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:37[public-personal-project-coexistence] | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:3[rule-whitespace-default] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:4[rule-whitespace-java-style-case-folded] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:5[interleave-java-style] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:6[interleave-comments-and-spaces] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:7[explicit-none-before-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:8[explicit-none-after-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| rule-trivia/corpus.json:9[enabled-parent-plain-default-child] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:0[capture-site-identity] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:1[nested-lexical-dynamic] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:2[scope-and-declares-parent-registration] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:3[failed-choice-rollback] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:4[repeated-capture-occurrences] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:5[optional-recursive-boundary] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:6[quantified-capture-occurrences] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:7[nested-same-name-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| scope-annotations/corpus.json:8[trimmed-group-non-bmp] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| scope-store/corpus.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality/corpus.json:0[node-one] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:10[mixed-bounded] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:11[node-separated] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:12[mixed-separated] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:13[optional-many-alias] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:1[mixed-one] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:2[node-pair] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:3[mixed-pair] | 0 | 0 | 0 | 0 | 10 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:4[mixed-group] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:5[mixed-alias] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:6[node-optional] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:7[mixed-optional] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:8[node-repeat] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:9[mixed-repeat] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:0[pure-node-one-alias] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:1[pure-node-multihop-alias] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:2[pure-node-group-alias] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:3[pure-node-delimiter-alias] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:4[pure-node-direct-multi] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:5[pure-node-multi-alias] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:8[pure-node-alias-optional] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:9[pure-node-alias-many] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:0[long-block] | 0 | 0 | 0 | 0 | 12 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:10[transparent-aliases] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:11[optional-start] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:12[repeat-start] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:13[combined-fence] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:1[long-block-shifted] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:2[start-direct] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:3[end-direct] | 0 | 0 | 0 | 0 | 9 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:4[prefix-vs-full-input-start] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:5[prefix-vs-full-input-end] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:6[start-source-offset-after-crlf] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:7[end-source-offset] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:8[java-style-outer-only] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:9[p4-zero-field-java-style] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:0[direct-eof] | 0 | 0 | 0 | 0 | 20 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:1[prefix-vs-full-input] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:2[comments-and-eof] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:3[transparent-alias] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:4[optional] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:5[repeat] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:6[mapped-node] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| 合計 | 44 | 0 | 0 | 0 | 1328 | 27 | 29 | 3 | 61 |

拒否理由（M3 作業リスト）

同一文法の拒否はキャッシュ単位に数える。ケース数は理由ごとに重複除去する。生成器が最初の拒否で停止するため、後続の未対応機能を網羅する件数ではない。IR 診断が複数あると理由間で重複する。

| 段階 / 機能・診断コード | build 数 | 元ケース数 | 展開後件数 |
|---|---:|---:|---:|
| ir/E-IMPORT-CYCLE | 1 | 1 | 1 |
| ir/E-IMPORT-UNRESOLVED | 15 | 32 | 32 |
| ir/E-LOWER-COMMON-FIELD | 1 | 1 | 1 |
| ir/E-LOWER-SUM-EVAL | 1 | 1 | 1 |
| ir/E-PRECEDENCE-ORDER | 1 | 24 | 24 |
| ir/E-WHITESPACE-GLOBAL-STYLE | 3 | 7 | 7 |
| ir/E-WHITESPACE-RULE-STYLE | 2 | 5 | 5 |

Fail 詳細（各実行の差分先頭3行。実測応答・全差分・期待値は JSON に保持）

primitives/corpus.json / 35:EMPTY/0 / primitives/corpus.json:35[35:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"ab"},"span":[0,2],"type":"Value"} | null |

primitives/corpus.json / 36:ANY/0 / primitives/corpus.json:36[36:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"ab"},"span":[0,2],"type":"Value"} | null |

rule-trivia/corpus.json / BuiltinNoneReserved/reserved-alias-wins / rule-trivia/corpus.json:23[BuiltinNoneReserved]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"a"},"span":[0,2],"type":"Root"} | null |

未観測項目（展開後件数。欠落を Pass に読み替えない）

| 観測 | 件数 |
|---|---:|
| /diagnostic | 2 |
| /diagnosticError | 1 |
| /enumError | 2 |
| /enumValue | 3 |
| /evaluationError | 32 |
| /grammar (操作列の driver protocol 未対応) | 18 |
| /operations | 18 |
| /parseError | 1 |
| /rollback | 4 |

段階失敗の原文（完全な標準出力・標準エラーは JSON）

ee1c07f4a48dcf58-023 / ir: rule-trivia/corpus.json:15[NamedGlobal]

> 終了 exit status: 1:
> ubnfc: global @whitespace style must be javaStyle or none: GAP [code: E-WHITESPACE-GLOBAL-STYLE] [hint: Use '@whitespace: javaStyle' or '@whitespace: none'.]

74e621b9865e2a10-024 / ir: rule-trivia/corpus.json:16[NamedRule]

> 終了 exit status: 1:
> ubnfc: rule Root uses unsupported @whitespace style: GAP (allowed: javaStyle, none) [code: E-WHITESPACE-RULE-STYLE] [hint: Use @whitespace or @whitespace(none).]

100a115568cb0f06-025 / ir: rule-trivia/corpus.json:17[imported-layout]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'layout.ubnf' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/layout.ubnf: No such file or directory (os error 2)]

5b6c2f443482c61d-026 / ir: rule-trivia/corpus.json:18[imported-rule-layout]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'layout.ubnf' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/layout.ubnf: No such file or directory (os error 2)]

04cbeed93c275e6b-027 / ir: rule-trivia/corpus.json:19[named-token-atomic]

> 終了 exit status: 1:
> ubnfc: global @whitespace style must be javaStyle or none: GAP [code: E-WHITESPACE-GLOBAL-STYLE] [hint: Use '@whitespace: javaStyle' or '@whitespace: none'.]

98d07a59569c8566-028 / ir: rule-trivia/corpus.json:20[named-child-policy-restoration]

> 終了 exit status: 1:
> ubnfc: global @whitespace style must be javaStyle or none: GAP [code: E-WHITESPACE-GLOBAL-STYLE] [hint: Use '@whitespace: javaStyle' or '@whitespace: none'.]

9a199968d01aa3af-029 / ir: rule-trivia/corpus.json:21[named-overrides-interleave]

> 終了 exit status: 1:
> ubnfc: rule Root uses unsupported @whitespace style: GAP (allowed: javaStyle, none) [code: E-WHITESPACE-RULE-STYLE] [hint: Use @whitespace or @whitespace(none).]
> rule Root uses @interleave(profile=javaStyle), which resolves to the same delimiter set as @whitespace [code: W-ANNOTATION-INTERLEAVE-DEPRECATED] [hint: Use @whitespace (or @whitespace(javaStyle)) instead; @interleave is removed in a later version.]

522aa57b4b44c927-032 / ir: rule-trivia/corpus.json:24[standard-layout-global-java-style-case-folded]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

95c16131ade9115e-033 / ir: rule-trivia/corpus.json:25[standard-layout-rule-whitespace-default]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

103e774bf76ada8b-034 / ir: rule-trivia/corpus.json:26[standard-layout-rule-whitespace-java-style-case-folded]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

00148edbe5b8dbec-035 / ir: rule-trivia/corpus.json:27[standard-layout-interleave-java-style]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

10a432e0c0fecb6e-036 / ir: rule-trivia/corpus.json:28[standard-layout-explicit-none-before-interleave]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

23b6812b462b310c-037 / ir: rule-trivia/corpus.json:29[standard-layout-explicit-none-after-interleave]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

3d1e117bf9717569-038 / ir: rule-trivia/corpus.json:30[standard-layout-enabled-parent-plain-default-child]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

fe071d9d3b9565d8-039 / ir: rule-trivia/corpus.json:31[standard-layout-plain-parent-enabled-default-child]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

2ee252d188e4abb3-040 / ir: rule-trivia/corpus.json:32[standard-layout-root-single-literal-choice]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

fcd65f758ef82cd0-041 / ir: rule-trivia/corpus.json:33[standard-layout-nested-constructs-and-rollback]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

2ef394199f377e10-042 / ir: rule-trivia/corpus.json:34[standard-layout-left-associative-local-whitespace]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

3e01f79c544ef679-043 / ir: rule-trivia/corpus.json:35[local-package-layout], rule-trivia/corpus.json:36[transitive-package-layout]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:team/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/team/layout: &lt;build&gt;/grammar/ubnf.json: package 'team/layout' の manifest を読めません（No such file or directory (os error 2)）]

6fe10ffcd19b534f-044 / ir: rule-trivia/corpus.json:37[public-personal-project-coexistence]

> 終了 exit status: 1:
> ubnfc: import が解決できません: 'pkg:alice/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/alice/layout: &lt;build&gt;/grammar/ubnf.json: package 'alice/layout' の manifest を読めません（No such file or directory (os error 2)）]
> import が解決できません: 'pkg:std/layout' [code: E-IMPORT-UNRESOLVED] [hint: &lt;build&gt;/grammar/.ubnf-packages/std/layout: &lt;build&gt;/grammar/ubnf.json: package 'std/layout' の manifest を読めません（No such file or directory (os error 2)）]

48c346ec23ef1c06-184 / ir: associative/corpus.json:1[reversed-metadata]

> 終了 exit status: 1:
> ubnfc: rule Expression precedence 30 must be lower than referenced operator rule Term precedence 20 [code: E-PRECEDENCE-ORDER] [hint: Decrease Expression level or increase Term level.]

668cf0ed36645997-205 / ir: oracle/common-field-missing.json:0[@commonField]

> 終了 exit status: 1:
> ubnfc: E-LOWER-COMMON-FIELD: ProbeAST.Node.name: variants の field 型/cardinality が不整合

4aaea9c4ba5b21d8-223 / ir: oracle/eval-sum.json:0[@eval]

> 終了 exit status: 1:
> ubnfc: E-LOWER-SUM-EVAL: Node: 自動 @eval(literal) は具象型が必要

3f95b76dccbb0235-229 / ir: oracle/import-cycle.json:0[@import]

> 終了 exit status: 1:
> ubnfc: 循環 import を検出しました: &lt;build&gt;/grammar/B.ubnf (スタック: [&lt;build&gt;/grammar/__runall__.ubnf, &lt;build&gt;/grammar/B.ubnf, &lt;build&gt;/grammar/C.ubnf]) [code: E-IMPORT-CYCLE] [hint: Remove the circular import.]
