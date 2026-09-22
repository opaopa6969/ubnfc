Java corpus run-all

元 corpus: 127 ファイル、1212 ケース。文法 variant: 279、hash キャッシュ: 234 build、展開後: 1425 件（文法なし操作列を含む）。jobs=4。

集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。

Java generator SHA-256: `10cb8b461eddf0ed7c377b97d3c14d0657c95f555cc77c913be1f50a0ab13dff`

front SHA-256: `fd7445b492ffbd6bd54723fff155639bd02f6e70934ad32fa8f0e952c98c540b`

Pass のうち明示的な期待拒否: 1件。build の実測失敗と診断は拒否理由表に保持する。

| 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 24 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 70 | 0 | 0 | 0 |
| catalog | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| evolution | 0 | 0 | 0 | 0 | 8 | 0 | 148 | 0 |
| lexical | 0 | 0 | 0 | 15 | 63 | 0 | 0 | 0 |
| mixed-values | 0 | 0 | 0 | 0 | 98 | 0 | 0 | 0 |
| nested-captures | 0 | 0 | 0 | 0 | 10 | 0 | 0 | 0 |
| numeric-capture | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 |
| oracle | 3 | 0 | 0 | 9 | 113 | 13 | 244 | 16 |
| primitives | 0 | 0 | 0 | 0 | 109 | 0 | 0 | 0 |
| right-associative | 0 | 0 | 0 | 0 | 112 | 0 | 0 | 0 |
| rule-trivia | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 |
| scope-annotations | 0 | 0 | 0 | 0 | 0 | 0 | 14 | 0 |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 101 | 0 | 0 | 0 | 62 | 0 | 0 | 0 |
| tiny-code-fence | 0 | 0 | 0 | 73 | 0 | 0 | 0 | 0 |
| tiny-string-token | 0 | 0 | 0 | 39 | 0 | 0 | 0 | 0 |
| 合計 | 128 | 0 | 0 | 136 | 715 | 13 | 406 | 27 |

6 系統別（case × variant 単位）

| loader 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 24 | 0 | 0 | 0 | 227 | 0 | 148 | 0 |
| inline-ast | 0 | 0 | 0 | 0 | 43 | 0 | 14 | 0 |
| operations | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 11 |
| oracle | 3 | 0 | 0 | 9 | 113 | 13 | 244 | 16 |
| rule | 101 | 0 | 0 | 112 | 160 | 0 | 0 | 0 |
| token | 0 | 0 | 0 | 15 | 172 | 0 | 0 | 0 |
| 合計 | 128 | 0 | 0 | 136 | 715 | 13 | 406 | 27 |

文法別（case × variant 単位。文法なしは操作列）

| 文法 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| associative/corpus.json:0[normal] | 0 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| associative/corpus.json:1[reversed-metadata] | 24 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cardinality/corpus.json:0[zero] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 |
| cardinality/corpus.json:1[plus] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 |
| cardinality/corpus.json:2[bounded] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 |
| cardinality/corpus.json:3[unbounded] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 |
| cardinality/corpus.json:4[separated] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 0 |
| catalog/corpus.json:0[deduplicated-sorted-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| catalog/corpus.json:1[rule-order-locality-quantifiers-and-escaped-context] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| evolution/conformance.json:0[0] | 0 | 0 | 0 | 0 | 0 | 0 | 37 | 0 |
| evolution/conformance.json:1[1] | 0 | 0 | 0 | 0 | 0 | 0 | 37 | 0 |
| evolution/conformance.json:2[2] | 0 | 0 | 0 | 0 | 0 | 0 | 37 | 0 |
| evolution/conformance.json:3[3] | 0 | 0 | 0 | 0 | 0 | 0 | 37 | 0 |
| evolution/diagnostics.json:0[3] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 |
| lexical/corpus.json:0[0:IdentifierParser] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 |
| lexical/corpus.json:10[10:SingleQuotedParser] | 0 | 0 | 0 | 2 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:11[11:EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:12[12:org.unlaxer.parser.elementary.EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:13[13:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:14[14:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:15[15:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:16[16:SingleQuotedParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:17[17:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:18[18:SingleQuotedParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:19[19:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:1[1:org.unlaxer.parser.clang.IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:20[20:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| lexical/corpus.json:21[21:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:2[2:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:3[3:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| lexical/corpus.json:4[4:SingleQuotedParser] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 0 |
| lexical/corpus.json:5[5:DoubleQuotedParser] | 0 | 0 | 0 | 9 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser] | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:8[8:SingleQuotedParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| lexical/corpus.json:9[9:DoubleQuotedParser] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| mixed-values/corpus.json:0[two] | 0 | 0 | 0 | 0 | 23 | 0 | 0 | 0 |
| mixed-values/corpus.json:1[one] | 0 | 0 | 0 | 0 | 23 | 0 | 0 | 0 |
| mixed-values/delimited-corpus.json:0[two] | 0 | 0 | 0 | 0 | 15 | 0 | 0 | 0 |
| mixed-values/delimited-corpus.json:1[one] | 0 | 0 | 0 | 0 | 15 | 0 | 0 | 0 |
| mixed-values/shared-corpus.json:0[first] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 |
| mixed-values/shared-corpus.json:1[second] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 |
| nested-captures/corpus.json:0[nested-text-completion] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| nested-captures/corpus.json:1[choice-optional-coexist] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| nested-captures/corpus.json:2[quantifier-items-not-containers] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| nested-captures/corpus.json:3[same-wrapper-and-parser-class] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| nested-captures/corpus.json:4[text-node-and-mixed-values] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| nested-captures/corpus.json:5[recursive-child-boundary] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| nested-captures/corpus.json:6[shared-mapping-nested-and-sequential] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| nested-captures/corpus.json:7[typed-node-list-and-shared-mixed-schema] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| numeric-capture/conformance.json:0[Numeric] | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 |
| oracle/backref.json:0[@backref-no-scope] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| oracle/char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 2 | 0 | 3 | 0 |
| oracle/ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| oracle/ci.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 2 | 0 |
| oracle/comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| oracle/comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| oracle/comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| oracle/common-field-missing.json:0[@commonField] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/common-field-type.json:0[@commonField] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| oracle/conformance-cardinality-bounded.json:0[cardinality] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 |
| oracle/conformance-cardinality-plus.json:0[cardinality] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 |
| oracle/conformance-cardinality-separated.json:0[cardinality] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 |
| oracle/conformance-cardinality-unbounded.json:0[cardinality] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 |
| oracle/conformance-cardinality-zero.json:0[cardinality] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/conformance-catalog-deduplicated-sorted-captures.json:0[catalog] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-catalog-rule-order-locality-quantifiers-and-escaped-context.json:0[catalog] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-mixed-values-one.json:0[mixed-values] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-nested-text-completion.json:0[nested-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-same-wrapper-and-parser-class.json:0[nested-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-text-node-and-mixed-values.json:0[nested-captures] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/conformance-scope-annotations-capture-site-identity.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-failed-choice-rollback.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-nested-lexical-dynamic.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-nested-same-name-captures.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-scope-and-declares-parent-registration.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-annotations-trimmed-group-non-bmp.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/conformance-scope-store.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| oracle/digit.json:0[DigitParser] | 0 | 0 | 0 | 0 | 2 | 0 | 2 | 0 |
| oracle/empty-one-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/empty-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/empty.json:0[EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| oracle/enum-collision.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| oracle/enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/enum.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/eof.json:0[EOF] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| oracle/error.json:0[ERROR] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| oracle/eval-binary_arithmetic.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-conditional.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-literal.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-manual.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-passthrough.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-sum.json:0[@eval] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-unknown.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-variable_ref.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/evolution-v0.json:0[evolution] | 0 | 0 | 0 | 0 | 0 | 0 | 37 | 0 |
| oracle/evolution-v1.json:0[evolution] | 0 | 0 | 0 | 0 | 0 | 0 | 37 | 0 |
| oracle/evolution-v2.json:0[evolution] | 0 | 0 | 0 | 0 | 0 | 0 | 37 | 0 |
| oracle/evolution-v3.json:0[evolution] | 0 | 0 | 0 | 0 | 0 | 0 | 37 | 0 |
| oracle/extern-full.json:0[Extern] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| oracle/extern.json:0[Extern] | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/import-alias.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/import-cycle.json:0[@import] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/import-two-level.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 3 | 0 |
| oracle/lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/lookahead-until.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| oracle/mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| oracle/mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| oracle/negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| oracle/numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 0 |
| oracle/posix.json:0[posix] | 0 | 0 | 0 | 0 | 1 | 0 | 3 | 0 |
| oracle/predictive-extern.json:0[@predictiveChoice] | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| oracle/predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 2 | 0 | 2 | 1 |
| oracle/recovery-auto.json:0[@recovery] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| oracle/recovery-root.json:0[@recovery] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| oracle/recovery-skip.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| oracle/recovery-sync.json:0[@recovery] | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 0 |
| oracle/regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| oracle/regex-backref.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/regex-emoji.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/regex-prefix.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/regex-zero.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/reobserved-catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/reobserved-char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/reobserved-ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/reobserved-ci.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/reobserved-comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| oracle/reobserved-comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/reobserved-comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/reobserved-common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| oracle/reobserved-enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/reobserved-longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/reobserved-lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/reobserved-mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/reobserved-mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/reobserved-mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/reobserved-negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/reobserved-negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/reobserved-numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/reobserved-posix.json:0[posix] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/reobserved-predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/reobserved-predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/reobserved-regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/reobserved-regex-backref.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/repeat-unbounded.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/repeat-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| oracle/separated.json:0[separated] | 0 | 0 | 0 | 0 | 2 | 0 | 3 | 0 |
| oracle/skip.json:0[@skip] | 0 | 0 | 0 | 0 | 2 | 0 | 2 | 0 |
| oracle/typeof.json:0[@typeof] | 0 | 0 | 0 | 0 | 2 | 0 | 3 | 0 |
| oracle/until.json:0[UNTIL] | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 0 |
| primitives/corpus.json:0[0:ANY] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 |
| primitives/corpus.json:10[10:UNTIL('*/')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:11[11:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:12[12:UNTIL('#')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:13[13:UNTIL('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:14[14:LOOKAHEAD('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:15[15:NEGATIVE_LOOKAHEAD('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:16[16:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:17[17:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:18[18:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:19[19:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:1[1:CHAR_RANGE('a','z')] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 |
| primitives/corpus.json:20[20:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:21[21:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:22[22:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:23[23:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:24[24:UNTIL('#')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:25[25:ANY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:26[26:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:27[27:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:28[28:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:29[29:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:2[2:CHAR_RANGE('あ','お')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:30[30:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:31[31:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:32[32:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:33[33:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:34[34:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:35[35:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:36[36:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:37[37:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:38[38:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:39[39:NEGATION('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:3[3:NEGATION('x😀')] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 |
| primitives/corpus.json:40[40:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:41[41:EMPTY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:42[42:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:43[43:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:44[44:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:45[45:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:46[46:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:47[47:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:4[4:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:5[5:EOF] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:6[6:LOOKAHEAD('ab')] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 |
| primitives/corpus.json:7[7:NEGATIVE_LOOKAHEAD('x')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:8[8:LOOKAHEAD('😀')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:9[9:UNTIL('#')] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 |
| right-associative/corpus.json:0[text] | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 |
| right-associative/corpus.json:1[mapped] | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 |
| right-associative/corpus.json:2[unicode] | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 |
| right-associative/corpus.json:3[grouped] | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 |
| rule-trivia/corpus.json:0[global-omitted] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| rule-trivia/corpus.json:10[plain-parent-enabled-default-child] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| rule-trivia/corpus.json:11[root-single-literal-choice] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 |
| rule-trivia/corpus.json:12[nested-constructs-and-rollback] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| rule-trivia/corpus.json:13[left-associative-local-whitespace] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:14[right-associative-local-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:1[global-none] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| rule-trivia/corpus.json:2[global-java-style-case-folded] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:3[rule-whitespace-default] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:4[rule-whitespace-java-style-case-folded] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| rule-trivia/corpus.json:5[interleave-java-style] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| rule-trivia/corpus.json:6[interleave-comments-and-spaces] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| rule-trivia/corpus.json:7[explicit-none-before-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:8[explicit-none-after-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:9[enabled-parent-plain-default-child] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| scope-annotations/corpus.json:0[capture-site-identity] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| scope-annotations/corpus.json:1[nested-lexical-dynamic] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:2[scope-and-declares-parent-registration] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| scope-annotations/corpus.json:3[failed-choice-rollback] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| scope-annotations/corpus.json:4[repeated-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:5[optional-recursive-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| scope-annotations/corpus.json:6[quantified-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:7[nested-same-name-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:8[trimmed-group-non-bmp] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-store/corpus.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality/corpus.json:0[node-one] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:10[mixed-bounded] | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:11[node-separated] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:12[mixed-separated] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:13[optional-many-alias] | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias] | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:1[mixed-one] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:2[node-pair] | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:3[mixed-pair] | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:4[mixed-group] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:5[mixed-alias] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:6[node-optional] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:7[mixed-optional] | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:8[node-repeat] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:9[mixed-repeat] | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:0[pure-node-one-alias] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:1[pure-node-multihop-alias] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:2[pure-node-group-alias] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:3[pure-node-delimiter-alias] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:4[pure-node-direct-multi] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:5[pure-node-multi-alias] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:8[pure-node-alias-optional] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:9[pure-node-alias-many] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:0[start-direct] | 0 | 0 | 0 | 21 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:10[repeat-start] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:11[combined-fence] | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:1[end-direct] | 0 | 0 | 0 | 9 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:2[prefix-vs-full-input-start] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:3[prefix-vs-full-input-end] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:4[start-source-offset-after-crlf] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:5[end-source-offset] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:6[java-style-outer-only] | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:7[p4-zero-field-java-style] | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:8[transparent-aliases] | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:9[optional-start] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:0[direct-eof] | 0 | 0 | 0 | 20 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:1[prefix-vs-full-input] | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:2[comments-and-eof] | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:3[transparent-alias] | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:4[optional] | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:5[repeat] | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:6[mapped-node] | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| 合計 | 128 | 0 | 0 | 136 | 715 | 13 | 406 | 27 |

拒否理由（M3 作業リスト）

同一文法の拒否はキャッシュ単位に数える。ケース数は理由ごとに重複除去する。生成器が最初の拒否で停止するため、後続の未対応機能を網羅する件数ではない。IR 診断が複数あると理由間で重複する。

| 段階 / 機能・診断コード | build 数 | 元ケース数 | 展開後件数 |
|---|---:|---:|---:|
| ir/E-IMPORT-CYCLE | 1 | 1 | 1 |
| ir/E-LOWER-COMMON-FIELD | 2 | 2 | 2 |
| ir/E-LOWER-SUM-EVAL | 1 | 1 | 1 |
| ir/E-PRECEDENCE-ORDER | 1 | 24 | 24 |
| ir/analysis.CaptureCardinality | 17 | 101 | 101 |

Fail 詳細（各実行の差分先頭3行。実測応答・全差分・期待値は JSON に保持）

evolution/conformance.json / evolution/v0/0 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/1 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v0/10 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /nativeFailure/expected/2 | "SignParser" | 欠落 |

evolution/conformance.json / evolution/v0/11 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |

evolution/conformance.json / evolution/v0/12 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v0/13 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/14 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/15 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v0/16 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

evolution/conformance.json / evolution/v0/17 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v0/18 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v0/19 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/2 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/20 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/21 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/22 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/23 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/24 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/25 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/26 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/27 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/28 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/29 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/3 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/30 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/31 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/32 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'\n'" | "'+'" |
| /nativeFailure/expected/1 | "'\r\n'" | 欠落 |
| /nativeFailure/expected/2 | "'\r'" | 欠落 |

evolution/conformance.json / evolution/v0/33 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "WildCardStringParser" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

evolution/conformance.json / evolution/v0/34 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/35 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/36 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v0/4 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/5 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/6 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/7 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/8 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v0/9 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

evolution/conformance.json / evolution/v1/0 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/1 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/10 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /nativeFailure/expected/2 | "SignParser" | 欠落 |

evolution/conformance.json / evolution/v1/11 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |

evolution/conformance.json / evolution/v1/12 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/13 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 1 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/14 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/15 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/16 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

evolution/conformance.json / evolution/v1/17 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/18 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/19 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/2 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/20 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/21 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/22 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/23 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/24 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/25 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/26 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/27 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/28 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/29 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/3 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/30 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/31 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/32 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'\n'" | "'*'" |
| /nativeFailure/expected/1 | "'\r\n'" | "'+'" |
| /nativeFailure/expected/2 | "'\r'" | 欠落 |

evolution/conformance.json / evolution/v1/33 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "WildCardStringParser" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

evolution/conformance.json / evolution/v1/34 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/35 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/36 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v1/4 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/5 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/6 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/7 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/8 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/9 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/0 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/1 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | "NUMBER" |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

evolution/conformance.json / evolution/v2/10 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /nativeFailure/expected/2 | "SignParser" | 欠落 |

evolution/conformance.json / evolution/v2/11 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |

evolution/conformance.json / evolution/v2/12 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/13 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 1 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/14 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/15 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/16 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 7 | 1 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/17 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/18 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/19 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 5 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/2 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/20 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 12 | 5 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/21 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 11 | 10 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

evolution/conformance.json / evolution/v2/22 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 11 | 9 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

evolution/conformance.json / evolution/v2/23 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "')'" |
| /nativeFailure/expected/1 | "')'" | "'*'" |
| /nativeFailure/expected/2 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v2/24 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/25 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/26 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/27 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/28 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/29 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/3 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/30 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

evolution/conformance.json / evolution/v2/31 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/32 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'\n'" | "'*'" |
| /nativeFailure/expected/1 | "'\r\n'" | "'+'" |
| /nativeFailure/expected/2 | "'\r'" | 欠落 |

evolution/conformance.json / evolution/v2/33 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "WildCardStringParser" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

evolution/conformance.json / evolution/v2/34 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/35 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/36 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 5 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/4 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/5 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/6 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/7 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/8 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/9 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/0 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'('" | "'neg'" |
| /nativeFailure/expected/2 | "')'" | "NUMBER" |

evolution/conformance.json / evolution/v3/1 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |
| /nativeFailure/expected/2 | "'.'" | "NUMBER" |

evolution/conformance.json / evolution/v3/10 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /nativeFailure/expected/2 | "SignParser" | 欠落 |

evolution/conformance.json / evolution/v3/11 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |

evolution/conformance.json / evolution/v3/12 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

evolution/conformance.json / evolution/v3/13 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 1 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/14 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/15 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

evolution/conformance.json / evolution/v3/16 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 7 | 1 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

evolution/conformance.json / evolution/v3/17 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

evolution/conformance.json / evolution/v3/18 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

evolution/conformance.json / evolution/v3/19 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 5 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/2 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/20 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 12 | 5 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

evolution/conformance.json / evolution/v3/21 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 11 | 10 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

evolution/conformance.json / evolution/v3/22 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 11 | 9 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

evolution/conformance.json / evolution/v3/23 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "')'" |
| /nativeFailure/expected/1 | "')'" | "'*'" |
| /nativeFailure/expected/2 | "'*'" | "'+'" |

evolution/conformance.json / evolution/v3/24 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'('" | "'neg'" |
| /nativeFailure/expected/2 | "')'" | "NUMBER" |

evolution/conformance.json / evolution/v3/25 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 18 | 14 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

evolution/conformance.json / evolution/v3/26 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 14 | 13 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

evolution/conformance.json / evolution/v3/27 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 17 | 15 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

evolution/conformance.json / evolution/v3/28 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | "','" |

evolution/conformance.json / evolution/v3/29 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/3 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/30 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'('" | "'neg'" |
| /nativeFailure/expected/2 | "')'" | "NUMBER" |

evolution/conformance.json / evolution/v3/31 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/32 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'\n'" | "'*'" |
| /nativeFailure/expected/1 | "'\r\n'" | "'+'" |
| /nativeFailure/expected/2 | "'\r'" | 欠落 |

evolution/conformance.json / evolution/v3/33 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "WildCardStringParser" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

evolution/conformance.json / evolution/v3/34 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/35 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/36 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 5 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

evolution/conformance.json / evolution/v3/4 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/5 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/6 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/7 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/8 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v3/9 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/backref.json / backref/1 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "AlphabetNumericUnderScoreParser" | 欠落 |

oracle/backref.json / backref/2 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "AlphabetNumericUnderScoreParser" | 欠落 |

oracle/backref.json / backref/3 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "AlphabetUnderScoreParser" | "IDENTIFIER" |
| /nativeFailure/expected/1 | "IdentifierParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/backref.json / backref/4 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "AlphabetNumericUnderScoreParser" | 欠落 |

oracle/catalog.json / catalog/3 / oracle/catalog.json:0[@catalog]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'Z'" | "'z'" |
| /nativeFailure/expected/1 | "'z'" | 欠落 |
| /diagnostics/0/farthestExpected/0 | "'Z'" | "'z'" |

oracle/char-range.json / char-range/1 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "RootParser" | "CHAR_RANGE" |
| /nativeFailure/expected/1 | "TParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/char-range.json / char-range/4 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "RootParser" | "CHAR_RANGE" |
| /nativeFailure/expected/1 | "TParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/char-range.json / char-range/5 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "RootParser" | "CHAR_RANGE" |
| /nativeFailure/expected/1 | "TParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/ci-unicode.json / ci-unicode/3 / oracle/ci-unicode.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'é'" | "tokenRef" |
| /diagnostics/0/farthestExpected/0 | "'é'" | "tokenRef" |

oracle/ci.json / ci/2 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,3],"text":"LeT"} | 欠落 |

oracle/ci.json / ci/4 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'let'" | "tokenRef" |
| /diagnostics/0/farthestExpected/0 | "'let'" | "tokenRef" |

oracle/comment-line.json / comment-line/2 / oracle/comment-line.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 4 | 5 |

oracle/comment-whitespace.json / comment-whitespace/1 / oracle/comment-whitespace.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

oracle/comment-whitespace.json / comment-whitespace/2 / oracle/comment-whitespace.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

oracle/comment-whitespace.json / comment-whitespace/3 / oracle/comment-whitespace.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'b'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

oracle/common-field.json / common-field/1 / oracle/common-field.json:0[@commonField]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "AlphabetNumericUnderScoreParser" | 欠落 |

oracle/common-field.json / common-field/2 / oracle/common-field.json:0[@commonField]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 5 | 0 |
| /nativeFailure/expected/0 | "AlphabetNumericUnderScoreParser" | "'a:'" |
| /nativeFailure/matched | 5 | 0 |

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
| /nativeFailure/expected/0 | "RootParser" | "Digit" |
| /nativeFailure/expected/1 | "TParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/digit.json / digit/4 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "RootParser" | "Digit" |
| /nativeFailure/expected/1 | "TParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/empty-one-two.json / empty-one-two/1 / oracle/empty-one-two.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/offset | 1 | 0 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/empty.json / empty/1 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/offset | 1 | 0 |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/empty.json / empty/2 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "MatchOnly" | "ANY" |
| /nativeFailure/expected/1 | "WildCardCharacterParser" | 欠落 |

oracle/empty.json / empty/3 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/offset | 1 | 0 |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/enum-plain.json / enum-plain/2 / oracle/enum-plain.json:0[@enum]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/matched | 3 | 0 |

oracle/eval-binary_arithmetic.json / eval-binary_arithmetic/1 / oracle/eval-binary_arithmetic.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "'.'" | 欠落 |
| /nativeFailure/expected/2 | "DigitParser" | 欠落 |
| /nativeFailure/expected/3 | "EParser" | 欠落 |

oracle/eval-conditional.json / eval-conditional/1 / oracle/eval-conditional.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/eval-literal.json / eval-literal/1 / oracle/eval-literal.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/eval-manual.json / eval-manual/1 / oracle/eval-manual.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/eval-passthrough.json / eval-passthrough/1 / oracle/eval-passthrough.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/eval-unknown.json / eval-unknown/1 / oracle/eval-unknown.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/eval-variable_ref.json / eval-variable_ref/1 / oracle/eval-variable_ref.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "AlphabetNumericUnderScoreParser" | 欠落 |

oracle/evolution-v0.json / evolution/0 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/1 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v0.json / evolution/10 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /nativeFailure/expected/2 | "SignParser" | 欠落 |

oracle/evolution-v0.json / evolution/11 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |

oracle/evolution-v0.json / evolution/12 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v0.json / evolution/13 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/14 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/15 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v0.json / evolution/16 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

oracle/evolution-v0.json / evolution/17 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v0.json / evolution/18 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v0.json / evolution/19 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/2 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/20 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/21 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/22 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/23 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/24 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/25 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/26 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/27 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/28 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/29 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/3 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/30 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/31 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/32 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'\n'" | "'+'" |
| /nativeFailure/expected/1 | "'\r\n'" | 欠落 |
| /nativeFailure/expected/2 | "'\r'" | 欠落 |

oracle/evolution-v0.json / evolution/33 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "WildCardStringParser" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/evolution-v0.json / evolution/34 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/35 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/36 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v0.json / evolution/4 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/5 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/6 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/7 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/8 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v0.json / evolution/9 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'+'" | 欠落 |

oracle/evolution-v1.json / evolution/0 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/1 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/10 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /nativeFailure/expected/2 | "SignParser" | 欠落 |

oracle/evolution-v1.json / evolution/11 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |

oracle/evolution-v1.json / evolution/12 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/13 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 1 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/14 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/15 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/16 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

oracle/evolution-v1.json / evolution/17 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/18 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/19 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/2 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/20 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/21 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/22 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/23 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/24 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/25 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/26 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/27 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/28 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/29 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/3 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/30 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "NUMBER" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/31 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/32 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'\n'" | "'*'" |
| /nativeFailure/expected/1 | "'\r\n'" | "'+'" |
| /nativeFailure/expected/2 | "'\r'" | 欠落 |

oracle/evolution-v1.json / evolution/33 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "WildCardStringParser" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/evolution-v1.json / evolution/34 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/35 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/36 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v1.json / evolution/4 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/5 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/6 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/7 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/8 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/9 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/0 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/1 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | "NUMBER" |
| /nativeFailure/expected/2 | "'.'" | 欠落 |

oracle/evolution-v2.json / evolution/10 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /nativeFailure/expected/2 | "SignParser" | 欠落 |

oracle/evolution-v2.json / evolution/11 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |

oracle/evolution-v2.json / evolution/12 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/13 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 1 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/14 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/15 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/16 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 7 | 1 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/17 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/18 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/19 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 5 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/2 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/20 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 12 | 5 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/21 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 11 | 10 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

oracle/evolution-v2.json / evolution/22 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 11 | 9 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

oracle/evolution-v2.json / evolution/23 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "')'" |
| /nativeFailure/expected/1 | "')'" | "'*'" |
| /nativeFailure/expected/2 | "'*'" | "'+'" |

oracle/evolution-v2.json / evolution/24 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/25 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/26 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/27 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/28 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/29 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/3 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/30 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'('" | "NUMBER" |
| /nativeFailure/expected/2 | "')'" | 欠落 |

oracle/evolution-v2.json / evolution/31 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/32 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'\n'" | "'*'" |
| /nativeFailure/expected/1 | "'\r\n'" | "'+'" |
| /nativeFailure/expected/2 | "'\r'" | 欠落 |

oracle/evolution-v2.json / evolution/33 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "WildCardStringParser" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/evolution-v2.json / evolution/34 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/35 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/36 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 5 | 0 |
| /nativeFailure/expected/0 | "' '" | "'neg'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/4 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/5 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/6 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/7 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/8 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/9 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/0 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'('" | "'neg'" |
| /nativeFailure/expected/2 | "')'" | "NUMBER" |

oracle/evolution-v3.json / evolution/1 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |
| /nativeFailure/expected/2 | "'.'" | "NUMBER" |

oracle/evolution-v3.json / evolution/10 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /nativeFailure/expected/2 | "SignParser" | 欠落 |

oracle/evolution-v3.json / evolution/11 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |

oracle/evolution-v3.json / evolution/12 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

oracle/evolution-v3.json / evolution/13 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 1 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/14 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/15 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

oracle/evolution-v3.json / evolution/16 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 7 | 1 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

oracle/evolution-v3.json / evolution/17 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

oracle/evolution-v3.json / evolution/18 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

oracle/evolution-v3.json / evolution/19 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 8 | 5 |
| /nativeFailure/expected/0 | "' '" | "'+'" |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/2 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/20 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 12 | 5 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

oracle/evolution-v3.json / evolution/21 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 11 | 10 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

oracle/evolution-v3.json / evolution/22 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 11 | 9 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

oracle/evolution-v3.json / evolution/23 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "')'" |
| /nativeFailure/expected/1 | "')'" | "'*'" |
| /nativeFailure/expected/2 | "'*'" | "'+'" |

oracle/evolution-v3.json / evolution/24 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'('" | "'neg'" |
| /nativeFailure/expected/2 | "')'" | "NUMBER" |

oracle/evolution-v3.json / evolution/25 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 18 | 14 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

oracle/evolution-v3.json / evolution/26 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 14 | 13 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

oracle/evolution-v3.json / evolution/27 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 17 | 15 |
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*/'" | "'+'" |

oracle/evolution-v3.json / evolution/28 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | "','" |

oracle/evolution-v3.json / evolution/29 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/3 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/30 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'('" | "'neg'" |
| /nativeFailure/expected/2 | "')'" | "NUMBER" |

oracle/evolution-v3.json / evolution/31 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/32 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'\n'" | "'*'" |
| /nativeFailure/expected/1 | "'\r\n'" | "'+'" |
| /nativeFailure/expected/2 | "'\r'" | 欠落 |

oracle/evolution-v3.json / evolution/33 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "WildCardStringParser" | 欠落 |
| /captures/0 | {"binding":"Literal:0","name":"value","span":[0,1],"text":"1"} | 欠落 |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/evolution-v3.json / evolution/34 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/35 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/36 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 5 | 0 |
| /nativeFailure/expected/0 | "' '" | "'if'" |
| /nativeFailure/expected/1 | "'*/'" | "'neg'" |

oracle/evolution-v3.json / evolution/4 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/5 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/6 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/7 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/8 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/evolution-v3.json / evolution/9 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | "'*'" |
| /nativeFailure/expected/1 | "'*'" | "'+'" |
| /nativeFailure/expected/2 | "'*/'" | 欠落 |

oracle/longest.json / longest/1 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 0 |
| /nativeFailure/matched | 1 | 0 |

oracle/longest.json / longest/3 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"B:0","name":"value","span":[0,2],"text":"ab"} | 欠落 |

oracle/longest.json / longest/4 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/2 | "RootParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "RootParser" | 欠落 |

oracle/lookahead-until.json / lookahead-until/1 / oracle/lookahead-until.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/offset | 2 | 0 |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 2 | 0 |

oracle/lookahead.json / lookahead/1 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/offset | 1 | 0 |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/lookahead.json / lookahead/2 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/offset | 1 | 0 |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/lookahead.json / lookahead/3 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'a'" | "tokenRef" |
| /nativeFailure/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/farthestExpected/0 | "'a'" | "tokenRef" |

oracle/lookahead.json / lookahead/4 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'a'" | "tokenRef" |
| /nativeFailure/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/farthestExpected/0 | "'a'" | "tokenRef" |

oracle/mapping-alias.json / mapping-alias/3 / oracle/mapping-alias.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/mapping-dotted.json / mapping-dotted/2 / oracle/mapping-dotted.json:0[dotted-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 0 |
| /nativeFailure/matched | 1 | 0 |

oracle/mapping-sum.json / mapping-sum/2 / oracle/mapping-sum.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 0 |
| /nativeFailure/matched | 1 | 0 |

oracle/negation-empty.json / negation-empty/1 / oracle/negation-empty.json:0[NEGATION]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "RootParser" | "NEGATION" |
| /nativeFailure/expected/1 | "TParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/negative-lookahead.json / negative-lookahead/2 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

oracle/negative-lookahead.json / negative-lookahead/3 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'a'" | "tokenRef" |
| /nativeFailure/expected/1 | "RootParser" | 欠落 |
| /nativeFailure/matched | 1 | 0 |

oracle/numeric.json / numeric/1 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/numeric.json / numeric/2 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/numeric.json / numeric/3 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/numeric.json / numeric/4 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | 欠落 |
| /nativeFailure/expected/1 | "EParser" | 欠落 |
| /nativeFailure/expected/2 | "ExponentParser" | 欠落 |

oracle/numeric.json / numeric/5 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | 欠落 |
| /nativeFailure/offset | 4 | 5 |

oracle/numeric.json / numeric/6 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/numeric.json / numeric/7 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "DigitParser" | "NUMBER" |
| /nativeFailure/expected/1 | "OneOrMore" | 欠落 |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "NUMBER" |

oracle/posix.json / posix/2 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "RootParser" | "Alphabet" |
| /nativeFailure/expected/1 | "TParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/posix.json / posix/3 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "RootParser" | "Alphabet" |
| /nativeFailure/expected/1 | "TParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |

oracle/posix.json / posix/4 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"A"} | 欠落 |

oracle/predictive-nullable.json / predictive-nullable/1 / oracle/predictive-nullable.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 0 |
| /nativeFailure/matched | 1 | 0 |

oracle/predictive-nullable.json / predictive-nullable/2 / oracle/predictive-nullable.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 0 |
| /nativeFailure/matched | 1 | 0 |

oracle/predictive.json / predictive/3 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 0 |
| /nativeFailure/expected/0 | 欠落 | "'a'" |
| /nativeFailure/matched | 1 | 0 |

oracle/predictive.json / predictive/4 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/2 | "'x'" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "'x'" | 欠落 |

oracle/recovery-auto.json / recovery-auto/1 / oracle/recovery-auto.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 6 | 7 |
| /nativeFailure/matched | 6 | 7 |
| /nativeFailure/offset | 6 | 7 |

oracle/recovery-sync.json / recovery-sync/3 / oracle/recovery-sync.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 2 |
| /nativeFailure/matched | 3 | 2 |
| /nativeFailure/offset | 3 | 2 |

oracle/regex-ascii.json / regex-ascii/1 / oracle/regex-ascii.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,2],"text":"ab"} | 欠落 |

oracle/regex-emoji.json / regex-emoji/1 / oracle/regex-emoji.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 0 | 1 |
| /nativeFailure/expected/0 | "REGEX('😀')" | 欠落 |
| /nativeFailure/matched | 0 | 1 |

oracle/regex-emoji.json / regex-emoji/2 / oracle/regex-emoji.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 2 | 1 |
| /nativeFailure/matched | 2 | 1 |
| /nativeFailure/offset | 2 | 1 |

oracle/regex-prefix.json / regex-prefix/1 / oracle/regex-prefix.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 2 |
| /nativeFailure/expected/0 | "REGEX('a')" | 欠落 |
| /nativeFailure/matched | 1 | 2 |

oracle/regex-prefix.json / regex-prefix/2 / oracle/regex-prefix.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 2 |
| /nativeFailure/expected/0 | "REGEX('a')" | 欠落 |
| /nativeFailure/matched | 1 | 2 |

oracle/regex-zero.json / regex-zero/1 / oracle/regex-zero.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "REGEX('a*')" | "tokenRef" |
| /diagnostics/0/farthestExpected/0 | "REGEX('a*')" | "tokenRef" |

oracle/regex-zero.json / regex-zero/2 / oracle/regex-zero.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "REGEX('a*')" | "tokenRef" |
| /diagnostics/0/farthestExpected/0 | "REGEX('a*')" | "tokenRef" |

oracle/reobserved-comment-whitespace.json / comment-whitespace/1 / oracle/reobserved-comment-whitespace.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

oracle/reobserved-comment-whitespace.json / comment-whitespace/2 / oracle/reobserved-comment-whitespace.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "' '" | 欠落 |
| /nativeFailure/expected/1 | "'*/'" | 欠落 |
| /nativeFailure/expected/2 | "'/*'" | 欠落 |

oracle/reobserved-common-field.json / common-field/1 / oracle/reobserved-common-field.json:0[@commonField]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "AlphabetNumericUnderScoreParser" | 欠落 |

oracle/reobserved-numeric.json / numeric/1 / oracle/reobserved-numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/reobserved-numeric.json / numeric/6 / oracle/reobserved-numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'.'" | 欠落 |
| /nativeFailure/expected/1 | "DigitParser" | 欠落 |
| /nativeFailure/expected/2 | "EParser" | 欠落 |

oracle/repeat-unbounded.json / repeat-unbounded/1 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/2 | "RootBounded0Parser" | "RootParser" |
| /nativeFailure/expected/3 | "RootParser" | 欠落 |
| /diagnostics/0/farthestExpected/2 | "RootBounded0Parser" | "RootParser" |

oracle/repeat-unbounded.json / repeat-unbounded/2 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "RootBounded0Parser" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "RootBounded0Parser" | 欠落 |

oracle/repeat-unbounded.json / repeat-unbounded/3 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "RootBounded0Parser" | 欠落 |

oracle/repeat-unbounded.json / repeat-unbounded/4 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "RootBounded0Parser" | 欠落 |
| /captures/0 | {"binding":"Root:0","name":"values","span":[0,1],"text":"a"} | 欠落 |
| /captures/1 | {"binding":"Root:0","name":"values","span":[1,2],"text":"a"} | 欠落 |

oracle/separated.json / separated/1 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "RootParser" | 欠落 |
| /nativeFailure/expected/2 | "RootSep0Parser" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/separated.json / separated/4 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"values","span":[0,1],"text":"a"} | 欠落 |

oracle/separated.json / separated/5 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "RootParser" | 欠落 |
| /nativeFailure/expected/2 | "RootSep0Parser" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/skip.json / skip/3 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "__CaptureSite" | 欠落 |
| /captures/0 | {"binding":"Item:0","name":"value","span":[0,1],"text":"a"} | 欠落 |
| /captures/1 | {"binding":"Root:0","name":"items","span":[0,1],"text":"a"} | 欠落 |

oracle/skip.json / skip/4 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/1 | "RootParser" | 欠落 |
| /nativeFailure/expected/2 | "__CaptureSite" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/typeof.json / typeof/2 / oracle/typeof.json:0[@typeof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 2 |
| /nativeFailure/matched | 3 | 2 |

oracle/typeof.json / typeof/3 / oracle/typeof.json:0[@typeof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 2 |
| /nativeFailure/matched | 3 | 2 |
| /mappingError | "java.lang.IllegalArgumentException: @typeof constraint violated: right must be same type as left, expected A but got B" | "@typeof constraint violated: right must be same type as left, expected A but got B at Span[start=0, end=3]" |

oracle/typeof.json / typeof/4 / oracle/typeof.json:0[@typeof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 3 | 0 |
| /nativeFailure/matched | 3 | 0 |
| /mappingError | "java.lang.IllegalArgumentException: @typeof constraint violated: right must be same type as left, expected B but got A" | "@typeof constraint violated: right must be same type as left, expected B but got A at Span[start=0, end=3]" |

oracle/until.json / until/1 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'#'" | "#" |
| /nativeFailure/expected/1 | "WildCardStringParser" | "ANY" |

oracle/until.json / until/2 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'#'" | "#" |
| /nativeFailure/expected/1 | "WildCardStringParser" | "ANY" |

oracle/until.json / until/3 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/consumed | 1 | 0 |
| /nativeFailure/expected/0 | "'#'" | "#" |
| /nativeFailure/matched | 2 | 0 |

oracle/until.json / until/4 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/offset | 1 | 0 |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/until.json / until/5 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /nativeFailure/expected/0 | "'#'" | "#" |
| /nativeFailure/expected/1 | "WildCardStringParser" | "ANY" |

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
| /operations | 18 |
| /parseError | 1 |
| /rollback | 4 |

段階失敗の原文（完全な標準出力・標準エラーは JSON）

8f511389f67a6e95-103 / ir: semantic-cardinality/corpus.json:2[node-pair]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

4d2f120fa2e45b33-104 / ir: semantic-cardinality/corpus.json:3[mixed-pair]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

3ab35d2f09300cbb-105 / ir: semantic-cardinality/corpus.json:4[mixed-group]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:146:body/0/group/capture/0", declared: List, inferred: Scalar }

d3cb3d0bdd87d780-106 / ir: semantic-cardinality/corpus.json:5[mixed-alias]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

d0a6ad4342bdea4a-107 / ir: semantic-cardinality/corpus.json:6[node-optional]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: Optional, inferred: Scalar }

767e557a4c92cd65-108 / ir: semantic-cardinality/corpus.json:7[mixed-optional]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: Optional, inferred: Scalar }

c5276f06f813588b-109 / ir: semantic-cardinality/corpus.json:8[node-repeat]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

67fc5dd8209d2abc-110 / ir: semantic-cardinality/corpus.json:9[mixed-repeat]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

5c4beb357ee13378-111 / ir: semantic-cardinality/corpus.json:10[mixed-bounded]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

d5cb7043499ec8fc-112 / ir: semantic-cardinality/corpus.json:11[node-separated]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

a65d3876e659d723-113 / ir: semantic-cardinality/corpus.json:12[mixed-separated]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

fd3cd9445b5e6f12-114 / ir: semantic-cardinality/corpus.json:13[optional-many-alias]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

ae46c408cd26181f-115 / ir: semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

1ab642713828abe9-116 / ir: semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

aa3e1fd30718d13a-125 / ir: semantic-cardinality/pure-alias.json:8[pure-node-alias-optional]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: Optional, inferred: Scalar }

4dc586578b05f624-126 / ir: semantic-cardinality/pure-alias.json:9[pure-node-alias-many]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

143477b0444818f8-127 / ir: semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many]

> 終了 exit status: 1:
> ubnfc: CaptureCardinality { site_id: "expr:__runall__.ubnf:129:135:body/0/ruleRef/capture/0", declared: List, inferred: Scalar }

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

実行エラー: lexical/corpus.json / 10:SingleQuotedParser/1 / lexical/corpus.json:10[10:SingleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::D","id":1}

実行エラー: lexical/corpus.json / 10:SingleQuotedParser/2 / lexical/corpus.json:10[10:SingleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::D","id":2}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/0 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":0}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/1 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":1}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/2 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":2}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/3 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":3}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/4 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":4}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/5 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":5}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/6 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":6}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/7 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":7}

実行エラー: lexical/corpus.json / 5:DoubleQuotedParser/8 / lexical/corpus.json:5[5:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":8}

実行エラー: lexical/corpus.json / 7:org.unlaxer.parser.elementary.DoubleQuotedParser/0 / lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":0}

実行エラー: lexical/corpus.json / 9:DoubleQuotedParser/0 / lexical/corpus.json:9[9:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":0}

実行エラー: lexical/corpus.json / 9:DoubleQuotedParser/1 / lexical/corpus.json:9[9:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":1}

実行エラー: lexical/corpus.json / 9:DoubleQuotedParser/2 / lexical/corpus.json:9[9:DoubleQuotedParser]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T","id":2}

実行エラー: oracle/extern-full.json / extern-full/1 / oracle/extern-full.json:0[Extern]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":0}

実行エラー: oracle/extern-full.json / extern-full/2 / oracle/extern-full.json:0[Extern]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":1}

実行エラー: oracle/extern-full.json / extern-full/3 / oracle/extern-full.json:0[Extern]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":2}

実行エラー: oracle/extern.json / extern/1 / oracle/extern.json:0[Extern]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":0}

実行エラー: oracle/extern.json / extern/2 / oracle/extern.json:0[Extern]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":1}

実行エラー: oracle/extern.json / extern/3 / oracle/extern.json:0[Extern]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":2}

実行エラー: oracle/extern.json / extern/4 / oracle/extern.json:0[Extern]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":3}

実行エラー: oracle/predictive-extern.json / predictive-extern/1 / oracle/predictive-extern.json:0[@predictiveChoice]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":0}

実行エラー: oracle/predictive-extern.json / predictive-extern/2 / oracle/predictive-extern.json:0[@predictiveChoice]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T","id":1}

実行エラー: tiny-code-fence/corpus.json / combined-fence/body-edge-whitespace-trimmed / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":6}

実行エラー: tiny-code-fence/corpus.json / combined-fence/close-not-at-sol / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":11}

実行エラー: tiny-code-fence/corpus.json / combined-fence/close-with-info / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":12}

実行エラー: tiny-code-fence/corpus.json / combined-fence/code-looking-body-is-text / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":5}

実行エラー: tiny-code-fence/corpus.json / combined-fence/cr-lines / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":8}

実行エラー: tiny-code-fence/corpus.json / combined-fence/crlf-lines / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":7}

実行エラー: tiny-code-fence/corpus.json / combined-fence/double-backticks-in-body / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":4}

実行エラー: tiny-code-fence/corpus.json / combined-fence/empty-body / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / combined-fence/inline-triple-backticks / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":10}

実行エラー: tiny-code-fence/corpus.json / combined-fence/lf-body / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-code-fence/corpus.json / combined-fence/missing-close / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":9}

実行エラー: tiny-code-fence/corpus.json / combined-fence/opening-at-eof / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":13}

実行エラー: tiny-code-fence/corpus.json / combined-fence/trailing-nul-before-eol-preserved / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":3}

実行エラー: tiny-code-fence/corpus.json / combined-fence/unicode-internal-nul-body / tiny-code-fence/corpus.json:11[combined-fence]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":2}

実行エラー: tiny-code-fence/corpus.json / end-direct/cr / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":3}

実行エラー: tiny-code-fence/corpus.json / end-direct/crlf / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":2}

実行エラー: tiny-code-fence/corpus.json / end-direct/empty / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":4}

実行エラー: tiny-code-fence/corpus.json / end-direct/eof / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":0}

実行エラー: tiny-code-fence/corpus.json / end-direct/four-backticks / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":5}

実行エラー: tiny-code-fence/corpus.json / end-direct/leading-space / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":8}

実行エラー: tiny-code-fence/corpus.json / end-direct/lf / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":1}

実行エラー: tiny-code-fence/corpus.json / end-direct/trailing-text / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":7}

実行エラー: tiny-code-fence/corpus.json / end-direct/two-backticks / tiny-code-fence/corpus.json:1[end-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":6}

実行エラー: tiny-code-fence/corpus.json / end-source-offset/after-crlf / tiny-code-fence/corpus.json:5[end-source-offset]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":0}

実行エラー: tiny-code-fence/corpus.json / end-source-offset/after-lf / tiny-code-fence/corpus.json:5[end-source-offset]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":1}

実行エラー: tiny-code-fence/corpus.json / end-source-offset/midline / tiny-code-fence/corpus.json:5[end-source-offset]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":2}

実行エラー: tiny-code-fence/corpus.json / java-style-outer-only/internal-comment-not-skipped / tiny-code-fence/corpus.json:6[java-style-outer-only]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":3}

実行エラー: tiny-code-fence/corpus.json / java-style-outer-only/internal-space-not-skipped / tiny-code-fence/corpus.json:6[java-style-outer-only]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":2}

実行エラー: tiny-code-fence/corpus.json / java-style-outer-only/leading-comment-ending-at-sol / tiny-code-fence/corpus.json:6[java-style-outer-only]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / java-style-outer-only/leading-space-breaks-sol / tiny-code-fence/corpus.json:6[java-style-outer-only]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-code-fence/corpus.json / optional-start/absent / tiny-code-fence/corpus.json:9[optional-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / optional-start/malformed / tiny-code-fence/corpus.json:9[optional-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":2}

実行エラー: tiny-code-fence/corpus.json / optional-start/present / tiny-code-fence/corpus.json:9[optional-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-code-fence/corpus.json / p4-zero-field-java-style/closed / tiny-code-fence/corpus.json:7[p4-zero-field-java-style]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / p4-zero-field-java-style/inline-triple-not-trivia / tiny-code-fence/corpus.json:7[p4-zero-field-java-style]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":4}

実行エラー: tiny-code-fence/corpus.json / p4-zero-field-java-style/leading-body-trivia / tiny-code-fence/corpus.json:7[p4-zero-field-java-style]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-code-fence/corpus.json / p4-zero-field-java-style/missing-close / tiny-code-fence/corpus.json:7[p4-zero-field-java-style]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":3}

実行エラー: tiny-code-fence/corpus.json / p4-zero-field-java-style/triple-inside-skipped-comment / tiny-code-fence/corpus.json:7[p4-zero-field-java-style]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":2}

実行エラー: tiny-code-fence/corpus.json / prefix-vs-full-input-end/full / tiny-code-fence/corpus.json:3[prefix-vs-full-input-end]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":0}

実行エラー: tiny-code-fence/corpus.json / prefix-vs-full-input-end/malformed / tiny-code-fence/corpus.json:3[prefix-vs-full-input-end]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":2}

実行エラー: tiny-code-fence/corpus.json / prefix-vs-full-input-end/valid-prefix / tiny-code-fence/corpus.json:3[prefix-vs-full-input-end]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE","id":1}

実行エラー: tiny-code-fence/corpus.json / prefix-vs-full-input-start/full / tiny-code-fence/corpus.json:2[prefix-vs-full-input-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / prefix-vs-full-input-start/malformed / tiny-code-fence/corpus.json:2[prefix-vs-full-input-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":2}

実行エラー: tiny-code-fence/corpus.json / prefix-vs-full-input-start/valid-prefix / tiny-code-fence/corpus.json:2[prefix-vs-full-input-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-code-fence/corpus.json / repeat-start/empty / tiny-code-fence/corpus.json:10[repeat-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / repeat-start/two / tiny-code-fence/corpus.json:10[repeat-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-code-fence/corpus.json / repeat-start/valid-then-midline / tiny-code-fence/corpus.json:10[repeat-start]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":2}

実行エラー: tiny-code-fence/corpus.json / start-direct/ascii-identifiers / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":4}

実行エラー: tiny-code-fence/corpus.json / start-direct/cr / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":2}

実行エラー: tiny-code-fence/corpus.json / start-direct/crlf / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-code-fence/corpus.json / start-direct/dangling-dot / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":15}

実行エラー: tiny-code-fence/corpus.json / start-direct/digit-start-class / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":12}

実行エラー: tiny-code-fence/corpus.json / start-direct/digit-start-scheme / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":8}

実行エラー: tiny-code-fence/corpus.json / start-direct/dollar-class / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":13}

実行エラー: tiny-code-fence/corpus.json / start-direct/dollar-scheme / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":9}

実行エラー: tiny-code-fence/corpus.json / start-direct/empty-class / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":10}

実行エラー: tiny-code-fence/corpus.json / start-direct/empty-dotted-segment / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":14}

実行エラー: tiny-code-fence/corpus.json / start-direct/empty-input / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":5}

実行エラー: tiny-code-fence/corpus.json / start-direct/empty-scheme / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":6}

実行エラー: tiny-code-fence/corpus.json / start-direct/eof / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":3}

実行エラー: tiny-code-fence/corpus.json / start-direct/four-backticks / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":16}

実行エラー: tiny-code-fence/corpus.json / start-direct/internal-comment / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":20}

実行エラー: tiny-code-fence/corpus.json / start-direct/internal-space-after-colon / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":19}

実行エラー: tiny-code-fence/corpus.json / start-direct/internal-space-before-colon / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":18}

実行エラー: tiny-code-fence/corpus.json / start-direct/lf / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / start-direct/two-backticks / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":17}

実行エラー: tiny-code-fence/corpus.json / start-direct/unicode-class / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":11}

実行エラー: tiny-code-fence/corpus.json / start-direct/unicode-scheme / tiny-code-fence/corpus.json:0[start-direct]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":7}

実行エラー: tiny-code-fence/corpus.json / start-source-offset-after-crlf/after-crlf / tiny-code-fence/corpus.json:4[start-source-offset-after-crlf]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / start-source-offset-after-crlf/after-lf / tiny-code-fence/corpus.json:4[start-source-offset-after-crlf]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-code-fence/corpus.json / start-source-offset-after-crlf/midline / tiny-code-fence/corpus.json:4[start-source-offset-after-crlf]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":2}

実行エラー: tiny-code-fence/corpus.json / transparent-aliases/empty-body / tiny-code-fence/corpus.json:8[transparent-aliases]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":0}

実行エラー: tiny-code-fence/corpus.json / transparent-aliases/text-body / tiny-code-fence/corpus.json:8[transparent-aliases]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START","id":1}

実行エラー: tiny-string-token/corpus.json / comments-and-eof/block-and-line / tiny-string-token/corpus.json:2[comments-and-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":0}

実行エラー: tiny-string-token/corpus.json / comments-and-eof/comment-is-raw-inside / tiny-string-token/corpus.json:2[comments-and-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":1}

実行エラー: tiny-string-token/corpus.json / comments-and-eof/comment-only / tiny-string-token/corpus.json:2[comments-and-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":2}

実行エラー: tiny-string-token/corpus.json / direct-eof/bare / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":15}

実行エラー: tiny-string-token/corpus.json / direct-eof/empty-double / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":0}

実行エラー: tiny-string-token/corpus.json / direct-eof/empty-input / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":14}

実行エラー: tiny-string-token/corpus.json / direct-eof/empty-single / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":1}

実行エラー: tiny-string-token/corpus.json / direct-eof/escaped-backslash / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":12}

実行エラー: tiny-string-token/corpus.json / direct-eof/escaped-double / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":9}

実行エラー: tiny-string-token/corpus.json / direct-eof/escaped-newline / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":11}

実行エラー: tiny-string-token/corpus.json / direct-eof/escaped-non-bmp / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":13}

実行エラー: tiny-string-token/corpus.json / direct-eof/escaped-single / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":8}

実行エラー: tiny-string-token/corpus.json / direct-eof/non-bmp / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":4}

実行エラー: tiny-string-token/corpus.json / direct-eof/nul / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":7}

実行エラー: tiny-string-token/corpus.json / direct-eof/open-only / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":16}

実行エラー: tiny-string-token/corpus.json / direct-eof/plain-double / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":2}

実行エラー: tiny-string-token/corpus.json / direct-eof/plain-single / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":3}

実行エラー: tiny-string-token/corpus.json / direct-eof/raw-crlf / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":6}

実行エラー: tiny-string-token/corpus.json / direct-eof/raw-newline / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":5}

実行エラー: tiny-string-token/corpus.json / direct-eof/trailing-backslash / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":18}

実行エラー: tiny-string-token/corpus.json / direct-eof/unclosed / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":17}

実行エラー: tiny-string-token/corpus.json / direct-eof/unknown-escape / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":10}

実行エラー: tiny-string-token/corpus.json / direct-eof/wrong-trailing-input / tiny-string-token/corpus.json:0[direct-eof]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":19}

実行エラー: tiny-string-token/corpus.json / mapped-node/double-node / tiny-string-token/corpus.json:6[mapped-node]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":1}

実行エラー: tiny-string-token/corpus.json / mapped-node/single-node / tiny-string-token/corpus.json:6[mapped-node]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":0}

実行エラー: tiny-string-token/corpus.json / optional/absent / tiny-string-token/corpus.json:4[optional]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":0}

実行エラー: tiny-string-token/corpus.json / optional/double / tiny-string-token/corpus.json:4[optional]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":2}

実行エラー: tiny-string-token/corpus.json / optional/single / tiny-string-token/corpus.json:4[optional]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":1}

実行エラー: tiny-string-token/corpus.json / optional/unclosed / tiny-string-token/corpus.json:4[optional]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":3}

実行エラー: tiny-string-token/corpus.json / prefix-vs-full-input/double-prefix / tiny-string-token/corpus.json:1[prefix-vs-full-input]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":2}

実行エラー: tiny-string-token/corpus.json / prefix-vs-full-input/full / tiny-string-token/corpus.json:1[prefix-vs-full-input]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":0}

実行エラー: tiny-string-token/corpus.json / prefix-vs-full-input/malformed / tiny-string-token/corpus.json:1[prefix-vs-full-input]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":3}

実行エラー: tiny-string-token/corpus.json / prefix-vs-full-input/single-prefix / tiny-string-token/corpus.json:1[prefix-vs-full-input]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":1}

実行エラー: tiny-string-token/corpus.json / repeat/empty / tiny-string-token/corpus.json:5[repeat]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":0}

実行エラー: tiny-string-token/corpus.json / repeat/non-bmp-repeat / tiny-string-token/corpus.json:5[repeat]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":2}

実行エラー: tiny-string-token/corpus.json / repeat/two-kinds / tiny-string-token/corpus.json:5[repeat]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":1}

実行エラー: tiny-string-token/corpus.json / repeat/valid-then-unclosed / tiny-string-token/corpus.json:5[repeat]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":3}

実行エラー: tiny-string-token/corpus.json / transparent-alias/double / tiny-string-token/corpus.json:3[transparent-alias]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":1}

実行エラー: tiny-string-token/corpus.json / transparent-alias/single / tiny-string-token/corpus.json:3[transparent-alias]: {"error":"java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING","id":0}

## Round 2 の比較と判定

基点 `d86f3a3`、ブランチ `fix/java-backend-round2`。旧比較レポートは `2026-09-22-java-corpus-after-triage.json`。実行数は 1195 → 1425（新規 oracle による増加）。元応答・差分・未観測は本 JSON の cases に保持し、B/D 全219件の追跡を round2_analysis.triage_case_tracking に収録した。

P=Pass、F=Fail、U=未観測ケース、R=IR失敗＋生成拒否、E=driver error＋javac/build failure。Known は別集計。

| 系統 | 前 P/F/U/R/E | 後 P/F/U/R/E |
|---|---:|---:|
| associative | 24/0/0/24/0 | 24/0/0/24/0 |
| cardinality | 50/20/0/0/0 | 70/0/0/0/0 |
| catalog | 0/0/2/0/0 | 0/0/2/0/0 |
| evolution | 6/2/148/0/0 | 8/148/0/0/0 |
| lexical | 39/23/0/16/0 | 63/0/0/0/15 |
| mixed-values | 37/39/0/22/0 | 98/0/0/0/0 |
| nested-captures | 0/5/4/1/0 | 10/0/0/0/0 |
| numeric-capture | 13/0/0/0/0 | 13/0/0/0/0 |
| oracle | 1/34/84/38/0 | 113/244/16/3/9 |
| primitives | 93/16/0/0/0 | 109/0/0/0/0 |
| right-associative | 63/49/0/0/0 | 112/0/0/0/0 |
| rule-trivia | 32/1/0/0/0 | 33/0/0/0/0 |
| scope-annotations | 0/2/12/0/0 | 0/14/0/0/0 |
| scope-store | 0/0/9/0/0 | 0/0/9/0/0 |
| semantic-cardinality | 32/30/0/101/0 | 62/0/0/101/0 |
| tiny-code-fence | 0/0/0/73/0 | 0/0/0/0/73 |
| tiny-string-token | 0/0/0/39/0 | 0/0/0/0/39 |
| 合計 | 390/221/259/314/0 | 715/406/27/128/136 |

Known: 11 → 13。旧 Fail221件を同じ corpus/case/variant で追跡すると `{'pass': 192, 'fail': 28, 'unobserved': 1}`。新規 oracle と nativeFailure 等の追加観測により総 Fail は単純比較できない。

### B96件の主因別対応

| 主因 | 対応確認 | 保留 |
|---|---:|---:|
| mapping/list/optional/capture順序 | 37 | 0 |
| A3/A4（empty lookahead・optional matched） | 4 | 0 |
| trivia/C・M・R境界 | 4 | 0 |
| EOF/repeat の主診断 | 6 | 0 |
| trailing_input の主 expected | 17 | 0 |
| 引用符18・backref3・scope文面2・右子span1・predictive空入力1 | 25 | 0 |
| 非BMP REGEX | 0 | 3 |

主因93件のうち30件は基点のレビュー修正を確認、63件は今回修正。主因解消とケース全体の Pass は区別する。B の最終 outcome は {'pass': 71, 'fail': 24, 'unobserved': 1}。prefix captures の比較接続、追加公開した nativeFailure、scope resolved の null キー等が残るため、B 全93ケースの Pass を意味しない。非BMP3件は指定どおり仕様未確定として Fail を維持した。

D123件の主因は評価119件（Power49、semantic30、mixed39、cardinality1）と数値 cause4件に対応。全て runtime AST / mapping例外から観測し、未対応評価はキーを省略する。D の最終 outcome は {'pass': 119, 'fail': 4}。

### 検証した追加契約

- scope `aa|bb|c,c`: a@0,a@1,b@3,b@4,c@6,c@8 の6宣言。repeated は b@6、trimmed 非BMP は宣言位置3・参照位置10/長さ2。
- lexical: `abx` の未captureリテラル a,b,x を3 token出現として保持。rule/expr の構造出現も別に保持。
- 右結合 ` x /*r*/ ^ x `: 右子 `[11,12]`、外側 `[0,13]`。generators.md #138 の「末尾空白・コメントを含むことがある」は全ノード必須ではなく、右再帰子の corpus 契約と両立する。
- DFA があれば code point 遷移と最後の正長受理候補を使用。ordered DFA の a|ab、greedy候補、非BMP、zeroWidth失敗、不正遷移拒否、DFA無しの Pattern.lookingAt を検証。
- syntax完了順の Capture API を維持し、ruleCompletionOrderを追加。captureNames probe では rule完了順を射影する。String/optional/list の値 span は保持した identity source map から得る。
- primary と farthest を分離し、nativeFailure の C/M は診断 snapshot に保存して memo/rollback でも保持。親 rollback は新 session の実parse後に scope/cursor を復元して採取。

### 残件

- nativeFailure/farthestExpectedは現在のruntime診断を公開している。旧Javaの複合token/trivia内部parser名・位置との残差を隠さない。
- prefixCapturesは追加済み。現harnessのOracle比較はcapturesを読み、prefixCapturesへ接続していない。全入力失敗のcapturesをprefix結果で置換していない。
- ScopeStateのresolvedをloaderがdeclared:null付きへ直列化し、宣言有無を要求していないqueryにもnullキーを比較する。driverはquery/symbolの実観測を返し、未要求・未対応項目をnullで補わない。
- scope-store操作列/catalog専用probe、未登録Extern TokenScannerは未対応。入力の再パースや期待値流用をしていない。
- 基点frontのDFAは旧longestMatch metadata。新semantics:leftmostFirstは独立した順序付きDFAテストで検証し、旧metadataも互換受理する。frontの変更は取り込んでいない。

Fail パターン（重複あり、ケース数）: {'nativeFailure': 380, 'diagnostics': 175, 'captures': 70, 'scope_resolved': 20, 'mappingError': 2}。

Extern 等の driver error: [('java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::START', 58), ('java.lang.IllegalArgumentException: Missing TokenScanner: TinyString::STRING', 39), ('java.lang.IllegalArgumentException: Missing TokenScanner: TinyFence::CLOSE', 15), ('java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::T', 13), ('java.lang.IllegalArgumentException: Missing TokenScanner: Probe::T', 9), ('java.lang.IllegalArgumentException: Missing TokenScanner: Primitive::D', 2)]。

### 検証コマンド

```sh
cd /home/opa/work/unlaxer-workspace/ubnfc-wt-java-r2/ubnfc-java
mvn -o -q test  # 135 tests, green
mvn -o -q -DskipTests package
cd /home/opa/work/unlaxer-workspace/ubnfc-wt-java-r2/ubnfc-java/runtime-template
mvn -o -q test  # 77 tests, green
cd /home/opa/work/unlaxer-workspace/ubnfc-wt-java-r2
export TMPDIR=/tmp/tmp.cGqV0QREVb
export UBNFC_JAVA_GENERATOR=/tmp/tmp.cGqV0QREVb/generator-round2.jar
cargo test -p ubnfc-harness --test runner_java_e2e  # 4 tests, green
cargo run -q --bin ubnfc-harness -- run-all --backend java --report docs/reports/2026-09-22-java-corpus-round2.md --json docs/reports/2026-09-22-java-corpus-round2.json
```

run-all は残る Fail/IR失敗/driver error により exit 1。生成 Java は CompilerHarness の全コンパイルで `--release 21 -Xlint:all -Werror` を適用。比較、oracle、期待値、乖離登録は変更していない。push/PR/merge は実施せず、worktree は保持する。
