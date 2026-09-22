Java corpus run-all

元 corpus: 83 ファイル、871 ケース。文法 variant: 236、hash キャッシュ: 234 build、展開後: 1195 件（文法なし操作列を含む）。jobs=4。

集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。

| 系統 | ir 失敗 | 生成拒否 | javac 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 24 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 30 | 0 | 20 | 20 |
| catalog | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| evolution | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 148 |
| lexical | 0 | 16 | 0 | 0 | 26 | 0 | 36 | 0 |
| mixed-values | 0 | 22 | 0 | 0 | 32 | 0 | 39 | 5 |
| nested-captures | 0 | 1 | 0 | 0 | 0 | 0 | 7 | 2 |
| numeric-capture | 0 | 0 | 0 | 0 | 7 | 0 | 6 | 0 |
| oracle | 4 | 35 | 0 | 0 | 0 | 13 | 80 | 36 |
| primitives | 0 | 0 | 0 | 0 | 48 | 0 | 61 | 0 |
| right-associative | 0 | 0 | 0 | 0 | 63 | 0 | 49 | 0 |
| rule-trivia | 0 | 0 | 0 | 0 | 32 | 0 | 1 | 0 |
| scope-annotations | 0 | 0 | 0 | 0 | 0 | 0 | 8 | 6 |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 101 | 0 | 0 | 0 | 32 | 0 | 30 | 0 |
| tiny-code-fence | 0 | 73 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-string-token | 0 | 39 | 0 | 0 | 0 | 0 | 0 | 0 |
| 合計 | 129 | 186 | 0 | 0 | 302 | 13 | 337 | 228 |

6 系統別（case × variant 単位）

| loader 系統 | ir 失敗 | 生成拒否 | javac 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 24 | 0 | 0 | 0 | 132 | 0 | 75 | 168 |
| inline-ast | 0 | 1 | 0 | 0 | 32 | 0 | 16 | 8 |
| operations | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 11 |
| oracle | 4 | 35 | 0 | 0 | 0 | 13 | 80 | 36 |
| rule | 101 | 134 | 0 | 0 | 64 | 0 | 69 | 5 |
| token | 0 | 16 | 0 | 0 | 74 | 0 | 97 | 0 |
| 合計 | 129 | 186 | 0 | 0 | 302 | 13 | 337 | 228 |

文法別（case × variant 単位。文法なしは操作列）

| 文法 | ir 失敗 | 生成拒否 | javac 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| associative/corpus.json:0[normal] | 0 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| associative/corpus.json:1[reversed-metadata] | 24 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cardinality/corpus.json:0[zero] | 0 | 0 | 0 | 0 | 6 | 0 | 6 | 2 |
| cardinality/corpus.json:1[plus] | 0 | 0 | 0 | 0 | 6 | 0 | 4 | 4 |
| cardinality/corpus.json:2[bounded] | 0 | 0 | 0 | 0 | 6 | 0 | 3 | 5 |
| cardinality/corpus.json:3[unbounded] | 0 | 0 | 0 | 0 | 6 | 0 | 4 | 4 |
| cardinality/corpus.json:4[separated] | 0 | 0 | 0 | 0 | 6 | 0 | 3 | 5 |
| catalog/corpus.json:0[deduplicated-sorted-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| catalog/corpus.json:1[rule-order-locality-quantifiers-and-escaped-context] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| evolution/conformance.json:0[0] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:1[1] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:2[2] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:3[3] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/diagnostics.json:0[3] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 0 |
| lexical/corpus.json:0[0:IdentifierParser] | 0 | 0 | 0 | 0 | 8 | 0 | 3 | 0 |
| lexical/corpus.json:10[10:SingleQuotedParser] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:11[11:EndOfSourceParser] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| lexical/corpus.json:12[12:org.unlaxer.parser.elementary.EndOfSourceParser] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| lexical/corpus.json:13[13:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:14[14:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:15[15:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| lexical/corpus.json:16[16:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| lexical/corpus.json:17[17:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:18[18:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:19[19:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| lexical/corpus.json:1[1:org.unlaxer.parser.clang.IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| lexical/corpus.json:20[20:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| lexical/corpus.json:21[21:IdentifierParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:2[2:IdentifierParser] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:3[3:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| lexical/corpus.json:4[4:SingleQuotedParser] | 0 | 0 | 0 | 0 | 7 | 0 | 14 | 0 |
| lexical/corpus.json:5[5:DoubleQuotedParser] | 0 | 9 | 0 | 0 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:8[8:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| lexical/corpus.json:9[9:DoubleQuotedParser] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| mixed-values/corpus.json:0[two] | 0 | 0 | 0 | 0 | 10 | 0 | 13 | 0 |
| mixed-values/corpus.json:1[one] | 0 | 0 | 0 | 0 | 10 | 0 | 8 | 5 |
| mixed-values/delimited-corpus.json:0[two] | 0 | 0 | 0 | 0 | 6 | 0 | 9 | 0 |
| mixed-values/delimited-corpus.json:1[one] | 0 | 0 | 0 | 0 | 6 | 0 | 9 | 0 |
| mixed-values/shared-corpus.json:0[first] | 0 | 11 | 0 | 0 | 0 | 0 | 0 | 0 |
| mixed-values/shared-corpus.json:1[second] | 0 | 11 | 0 | 0 | 0 | 0 | 0 | 0 |
| nested-captures/corpus.json:0[nested-text-completion] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| nested-captures/corpus.json:1[choice-optional-coexist] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| nested-captures/corpus.json:2[quantifier-items-not-containers] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:3[same-wrapper-and-parser-class] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| nested-captures/corpus.json:4[text-node-and-mixed-values] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| nested-captures/corpus.json:5[recursive-child-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:6[shared-mapping-nested-and-sequential] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:7[typed-node-list-and-shared-mixed-schema] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| numeric-capture/conformance.json:0[Numeric] | 0 | 0 | 0 | 0 | 7 | 0 | 6 | 0 |
| oracle/backref.json:0[@backref-no-scope] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/ci.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| oracle/comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 1 |
| oracle/comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 2 |
| oracle/common-field-missing.json:0[@commonField] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/common-field-type.json:0[@commonField] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/digit.json:0[DigitParser] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/empty-one-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty.json:0[EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| oracle/enum-collision.json:0[@enum] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/enum.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/eof.json:0[EOF] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| oracle/error.json:0[ERROR] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-binary_arithmetic.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-conditional.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-literal.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-manual.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-passthrough.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-sum.json:0[@eval] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-unknown.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-variable_ref.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/extern-full.json:0[Extern] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/extern.json:0[Extern] | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/import-alias.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/import-cycle.json:0[@import] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/import-two-level.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| oracle/lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/lookahead-until.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 2 |
| oracle/posix.json:0[posix] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| oracle/predictive-extern.json:0[@predictiveChoice] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/recovery-auto.json:0[@recovery] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/recovery-root.json:0[@recovery] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/recovery-skip.json:0[@recovery] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/recovery-sync.json:0[@recovery] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/regex-backref.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/regex-emoji.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/regex-prefix.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0 |
| oracle/regex-zero.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/repeat-unbounded.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/repeat-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/separated.json:0[separated] | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 0 |
| oracle/skip.json:0[@skip] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/typeof.json:0[@typeof] | 0 | 5 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/until.json:0[UNTIL] | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 0 |
| primitives/corpus.json:0[0:ANY] | 0 | 0 | 0 | 0 | 2 | 0 | 3 | 0 |
| primitives/corpus.json:10[10:UNTIL('*/')] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:11[11:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:12[12:UNTIL('#')] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:13[13:UNTIL('')] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| primitives/corpus.json:14[14:LOOKAHEAD('')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:15[15:NEGATIVE_LOOKAHEAD('')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:16[16:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:17[17:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:18[18:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:19[19:EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:1[1:CHAR_RANGE('a','z')] | 0 | 0 | 0 | 0 | 3 | 0 | 2 | 0 |
| primitives/corpus.json:20[20:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:21[21:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:22[22:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:23[23:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:24[24:UNTIL('#')] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:25[25:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| primitives/corpus.json:26[26:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:27[27:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:28[28:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:29[29:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:2[2:CHAR_RANGE('あ','お')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:30[30:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:31[31:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| primitives/corpus.json:32[32:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:33[33:EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:34[34:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:35[35:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:36[36:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| primitives/corpus.json:37[37:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:38[38:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:39[39:NEGATION('')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:3[3:NEGATION('x😀')] | 0 | 0 | 0 | 0 | 3 | 0 | 3 | 0 |
| primitives/corpus.json:40[40:EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:41[41:EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| primitives/corpus.json:42[42:EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:43[43:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:44[44:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:45[45:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:46[46:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:47[47:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:4[4:EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:5[5:EOF] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| primitives/corpus.json:6[6:LOOKAHEAD('ab')] | 0 | 0 | 0 | 0 | 3 | 0 | 1 | 0 |
| primitives/corpus.json:7[7:NEGATIVE_LOOKAHEAD('x')] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| primitives/corpus.json:8[8:LOOKAHEAD('😀')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:9[9:UNTIL('#')] | 0 | 0 | 0 | 0 | 3 | 0 | 3 | 0 |
| right-associative/corpus.json:0[text] | 0 | 0 | 0 | 0 | 17 | 0 | 11 | 0 |
| right-associative/corpus.json:1[mapped] | 0 | 0 | 0 | 0 | 17 | 0 | 11 | 0 |
| right-associative/corpus.json:2[unicode] | 0 | 0 | 0 | 0 | 15 | 0 | 13 | 0 |
| right-associative/corpus.json:3[grouped] | 0 | 0 | 0 | 0 | 14 | 0 | 14 | 0 |
| rule-trivia/corpus.json:0[global-omitted] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| rule-trivia/corpus.json:10[plain-parent-enabled-default-child] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| rule-trivia/corpus.json:11[root-single-literal-choice] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 |
| rule-trivia/corpus.json:12[nested-constructs-and-rollback] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| rule-trivia/corpus.json:13[left-associative-local-whitespace] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:14[right-associative-local-interleave] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| rule-trivia/corpus.json:1[global-none] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| rule-trivia/corpus.json:2[global-java-style-case-folded] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:3[rule-whitespace-default] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:4[rule-whitespace-java-style-case-folded] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| rule-trivia/corpus.json:5[interleave-java-style] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| rule-trivia/corpus.json:6[interleave-comments-and-spaces] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| rule-trivia/corpus.json:7[explicit-none-before-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:8[explicit-none-after-interleave] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| rule-trivia/corpus.json:9[enabled-parent-plain-default-child] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| scope-annotations/corpus.json:0[capture-site-identity] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| scope-annotations/corpus.json:1[nested-lexical-dynamic] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-annotations/corpus.json:2[scope-and-declares-parent-registration] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| scope-annotations/corpus.json:3[failed-choice-rollback] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| scope-annotations/corpus.json:4[repeated-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:5[optional-recursive-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| scope-annotations/corpus.json:6[quantified-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:7[nested-same-name-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-annotations/corpus.json:8[trimmed-group-non-bmp] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-store/corpus.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality/corpus.json:0[node-one] | 0 | 0 | 0 | 0 | 2 | 0 | 2 | 0 |
| semantic-cardinality/corpus.json:10[mixed-bounded] | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:11[node-separated] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:12[mixed-separated] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:13[optional-many-alias] | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias] | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:1[mixed-one] | 0 | 0 | 0 | 0 | 2 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:2[node-pair] | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:3[mixed-pair] | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:4[mixed-group] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:5[mixed-alias] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:6[node-optional] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:7[mixed-optional] | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:8[node-repeat] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/corpus.json:9[mixed-repeat] | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:0[pure-node-one-alias] | 0 | 0 | 0 | 0 | 4 | 0 | 2 | 0 |
| semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:1[pure-node-multihop-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 3 | 0 |
| semantic-cardinality/pure-alias.json:2[pure-node-group-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 2 | 0 |
| semantic-cardinality/pure-alias.json:3[pure-node-delimiter-alias] | 0 | 0 | 0 | 0 | 5 | 0 | 3 | 0 |
| semantic-cardinality/pure-alias.json:4[pure-node-direct-multi] | 0 | 0 | 0 | 0 | 3 | 0 | 3 | 0 |
| semantic-cardinality/pure-alias.json:5[pure-node-multi-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 4 | 0 |
| semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 4 | 0 |
| semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias] | 0 | 0 | 0 | 0 | 4 | 0 | 4 | 0 |
| semantic-cardinality/pure-alias.json:8[pure-node-alias-optional] | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic-cardinality/pure-alias.json:9[pure-node-alias-many] | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:0[start-direct] | 0 | 21 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:10[repeat-start] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:11[combined-fence] | 0 | 14 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:1[end-direct] | 0 | 9 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:2[prefix-vs-full-input-start] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:3[prefix-vs-full-input-end] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:4[start-source-offset-after-crlf] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:5[end-source-offset] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:6[java-style-outer-only] | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:7[p4-zero-field-java-style] | 0 | 5 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:8[transparent-aliases] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:9[optional-start] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:0[direct-eof] | 0 | 20 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:1[prefix-vs-full-input] | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:2[comments-and-eof] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:3[transparent-alias] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:4[optional] | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:5[repeat] | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:6[mapped-node] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| 合計 | 129 | 186 | 0 | 0 | 302 | 13 | 337 | 228 |

拒否理由（M3 作業リスト）

同一文法の拒否はキャッシュ単位に数える。ケース数は理由ごとに重複除去する。生成器が最初の拒否で停止するため、後続の未対応機能を網羅する件数ではない。IR 診断が複数あると理由間で重複する。

| 段階 / 機能・診断コード | build 数 | 元ケース数 | 展開後件数 |
|---|---:|---:|---:|
| generation/enum.duplicateName | 1 | 1 | 1 |
| generation/eval | 7 | 7 | 7 |
| generation/execution.transaction:error:once | 1 | 3 | 3 |
| generation/externTokens.inputState | 26 | 137 | 137 |
| generation/recipe.conversion | 3 | 12 | 23 |
| generation/recovery.legacyMarker | 4 | 10 | 10 |
| generation/typeof | 1 | 5 | 5 |
| ir/E-IMPORT-CYCLE | 1 | 1 | 1 |
| ir/E-LOWER-COMMON-FIELD | 2 | 2 | 2 |
| ir/E-LOWER-SUM-EVAL | 1 | 1 | 1 |
| ir/E-PRECEDENCE-ORDER | 1 | 24 | 24 |
| ir/analysis.CaptureCardinality | 17 | 101 | 101 |

Fail 詳細（各実行の差分先頭3行。実測応答・全差分・期待値は JSON に保持）

cardinality/corpus.json / Cardinality.ubnf/0 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0 | null |

cardinality/corpus.json / Cardinality.ubnf/1 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 10 | null |

cardinality/corpus.json / Cardinality.ubnf/1 / cardinality/corpus.json:1[plus]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 10 | null |

cardinality/corpus.json / Cardinality.ubnf/1 / cardinality/corpus.json:2[bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 10 | null |

cardinality/corpus.json / Cardinality.ubnf/1 / cardinality/corpus.json:3[unbounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 10 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:1[plus]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:2[bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:3[unbounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:4[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/3 / cardinality/corpus.json:4[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5 | null |

cardinality/corpus.json / Cardinality.ubnf/4 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/4 / cardinality/corpus.json:1[plus]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/4 / cardinality/corpus.json:3[unbounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/5 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/6 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/6 / cardinality/corpus.json:1[plus]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/6 / cardinality/corpus.json:2[bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/6 / cardinality/corpus.json:3[unbounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/7 / cardinality/corpus.json:4[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 8 | null |

lexical/corpus.json / 0:IdentifierParser/0 / lexical/corpus.json:0[0:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

lexical/corpus.json / 0:IdentifierParser/1 / lexical/corpus.json:0[0:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "_A09z" | null |

lexical/corpus.json / 0:IdentifierParser/2 / lexical/corpus.json:0[0:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "if" | null |

lexical/corpus.json / 11:EndOfSourceParser/0 / lexical/corpus.json:11[11:EndOfSourceParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

lexical/corpus.json / 12:org.unlaxer.parser.elementary.EndOfSourceParser/0 / lexical/corpus.json:12[12:org.unlaxer.parser.elementary.EndOfSourceParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

lexical/corpus.json / 14:SingleQuotedParser/0 / lexical/corpus.json:14[14:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "ab" | null |

lexical/corpus.json / 15:IdentifierParser/0 / lexical/corpus.json:15[15:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a"] | null |

lexical/corpus.json / 16:SingleQuotedParser/0 / lexical/corpus.json:16[16:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","b"] | null |

lexical/corpus.json / 17:SingleQuotedParser/0 / lexical/corpus.json:17[17:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | {"fields":{"text":"a"},"span":[1,4],"type":"Item"} | null |

lexical/corpus.json / 18:SingleQuotedParser/0 / lexical/corpus.json:18[18:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | " " | null |

lexical/corpus.json / 18:SingleQuotedParser/1 / lexical/corpus.json:18[18:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\\\\\\'x" | null |

lexical/corpus.json / 19:IdentifierParser/0 / lexical/corpus.json:19[19:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a:b" | null |

lexical/corpus.json / 1:org.unlaxer.parser.clang.IdentifierParser/0 / lexical/corpus.json:1[1:org.unlaxer.parser.clang.IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "hello_2" | null |

lexical/corpus.json / 20:IdentifierParser/0 / lexical/corpus.json:20[20:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

lexical/corpus.json / 20:IdentifierParser/1 / lexical/corpus.json:20[20:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "!" | null |

lexical/corpus.json / 21:IdentifierParser/0 / lexical/corpus.json:21[21:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | {"fields":{},"span":[1,6],"type":"Item"} | null |

lexical/corpus.json / 2:IdentifierParser/0 / lexical/corpus.json:2[2:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","b","_c9"] | null |

lexical/corpus.json / 2:IdentifierParser/1 / lexical/corpus.json:2[2:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","b"] | null |

lexical/corpus.json / 3:IdentifierParser/1 / lexical/corpus.json:3[3:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

lexical/corpus.json / 4:SingleQuotedParser/0 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

lexical/corpus.json / 4:SingleQuotedParser/1 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "abc" | null |

lexical/corpus.json / 4:SingleQuotedParser/10 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a\\'b" | null |

lexical/corpus.json / 4:SingleQuotedParser/11 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\\\\" | null |

lexical/corpus.json / 4:SingleQuotedParser/12 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\\\n" | null |

lexical/corpus.json / 4:SingleQuotedParser/13 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\"" | null |

lexical/corpus.json / 4:SingleQuotedParser/2 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "😀あ" | null |

lexical/corpus.json / 4:SingleQuotedParser/3 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a\nb" | null |

lexical/corpus.json / 4:SingleQuotedParser/4 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a\r\nb" | null |

lexical/corpus.json / 4:SingleQuotedParser/5 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\u0000" | null |

lexical/corpus.json / 4:SingleQuotedParser/6 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\\q" | null |

lexical/corpus.json / 4:SingleQuotedParser/7 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\\n" | null |

lexical/corpus.json / 4:SingleQuotedParser/8 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\\u0041" | null |

lexical/corpus.json / 4:SingleQuotedParser/9 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "\\😀" | null |

lexical/corpus.json / 6:org.unlaxer.parser.elementary.SingleQuotedParser/0 / lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a /*inside*/" | null |

lexical/corpus.json / 8:SingleQuotedParser/0 / lexical/corpus.json:8[8:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","b"] | null |

lexical/corpus.json / 8:SingleQuotedParser/1 / lexical/corpus.json:8[8:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["none","b"] | null |

mixed-values/corpus.json / Mixed.ubnf/0 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;-&#124;" | null |

mixed-values/corpus.json / Mixed.ubnf/0 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;-&#124;" | null |

mixed-values/corpus.json / Mixed.ubnf/1 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;-&#124;" | null |

mixed-values/corpus.json / Mixed.ubnf/1 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;-&#124;" | null |

mixed-values/corpus.json / Mixed.ubnf/10 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;T:b&#124;T:a,L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/10 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;T:b&#124;T:a,L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/11 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;-&#124;L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/11 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;-&#124;L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/12 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;-&#124;L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/12 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;-&#124;L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/2 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:y&#124;-&#124;" | null |

mixed-values/corpus.json / Mixed.ubnf/3 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;L:x&#124;T:a,L:x,O:y" | null |

mixed-values/corpus.json / Mixed.ubnf/4 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;T:a&#124;O:y,T:a,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/5 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀&#124;O:y&#124;T:😀,L:x,T:😀" | null |

mixed-values/corpus.json / Mixed.ubnf/6 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:b&#124;-&#124;T:c,L:x,O:y,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/7 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀&#124;T:a&#124;T:\\q" | null |

mixed-values/corpus.json / Mixed.ubnf/7 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀&#124;T:a&#124;T:\\q" | null |

mixed-values/corpus.json / Mixed.ubnf/8 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;L:x&#124;T:a,L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/8 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a&#124;L:x&#124;T:a,L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/9 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀&#124;L:x&#124;T:😀,L:x,T:a" | null |

mixed-values/corpus.json / Mixed.ubnf/9 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀&#124;L:x&#124;T:😀,L:x,T:a" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/0 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)&#124;-&#124;" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/0 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)&#124;-&#124;" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/1 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;-&#124;" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/1 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;-&#124;" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/2 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)&#124;L:x&#124;T:(a),L:x" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/2 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)&#124;L:x&#124;T:(a),L:x" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/3 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;T:(a)&#124;L:x,T:(a)" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/3 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x&#124;T:(a)&#124;L:x,T:(a)" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/4 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(😀)&#124;-&#124;T:(😀),L:x" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/4 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(😀)&#124;-&#124;T:(😀),L:x" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/5 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:('b')&#124;-&#124;T:('c')" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/5 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:('b')&#124;-&#124;T:('c')" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/6 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:( a )&#124;L:x&#124;T:(😀),T:( a )" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/6 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:( a )&#124;L:x&#124;T:(😀),T:( a )" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/7 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a/*c*/)&#124;-&#124;L:x,T:(a)" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/7 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a/*c*/)&#124;-&#124;L:x,T:(a)" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/8 / mixed-values/delimited-corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)&#124;-&#124;T:(a)/*😀*/,L:x" | null |

mixed-values/delimited-corpus.json / Delimited.ubnf/8 / mixed-values/delimited-corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)&#124;-&#124;T:(a)/*😀*/,L:x" | null |

nested-captures/corpus.json / choice-optional-coexist/exclusive-scalar-and-optional-absent / nested-captures/corpus.json:1[choice-optional-coexist]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/1/name | "item" | "choice" |
| /captures/1/span/0 | 3 | 0 |
| /captures/1/span/1 | 4 | 1 |

nested-captures/corpus.json / choice-optional-coexist/second-choice-and-optional-present / nested-captures/corpus.json:1[choice-optional-coexist]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/1/name | "value" | "choice" |
| /captures/1/span/0 | 2 | 0 |
| /captures/1/span/1 | 3 | 1 |

nested-captures/corpus.json / quantifier-items-not-containers/every-item-has-inner-then-outer-values / nested-captures/corpus.json:2[quantifier-items-not-containers]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/bounded/fields/value/1 | "bB" | "b" |
| /ast/fields/bounded/fields/value/2 | "b" | "bBbB" |
| /ast/fields/bounded/fields/value/3 | "bB" | 欠落 |

nested-captures/corpus.json / recursive-child-boundary/absent-parent-does-not-borrow-recursive-name / nested-captures/corpus.json:5[recursive-child-boundary]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0/span/0 | 6 | 0 |
| /captures/0/span/1 | 11 | 5 |
| /captures/0/text | "grand" | "child" |

nested-captures/corpus.json / shared-mapping-nested-and-sequential/shared-list-schema-retains-each-rule-order / nested-captures/corpus.json:6[shared-mapping-nested-and-sequential]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/2/name | "value" | "items" |
| /captures/2/span/0 | 2 | 0 |
| /captures/2/span/1 | 3 | 2 |

nested-captures/corpus.json / text-node-and-mixed-values/mixed-node-branch / nested-captures/corpus.json:4[text-node-and-mixed-values]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0/span/0 | 1 | 0 |
| /captures/0/span/1 | 2 | 1 |
| /captures/0/text | "x" | "a" |

nested-captures/corpus.json / text-node-and-mixed-values/mixed-text-branch / nested-captures/corpus.json:4[text-node-and-mixed-values]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0/span/0 | 1 | 0 |
| /captures/0/span/1 | 2 | 1 |
| /captures/0/text | "x" | "a" |

numeric-capture/conformance.json / numeric/0 / numeric-capture/conformance.json:0[Numeric]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0 | null |

numeric-capture/conformance.json / numeric/1 / numeric-capture/conformance.json:0[Numeric]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -17 | null |

numeric-capture/conformance.json / numeric/2 / numeric-capture/conformance.json:0[Numeric]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 17 | null |

numeric-capture/conformance.json / numeric/3 / numeric-capture/conformance.json:0[Numeric]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2147483647 | null |

numeric-capture/conformance.json / numeric/4 / numeric-capture/conformance.json:0[Numeric]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -2147483648 | null |

numeric-capture/conformance.json / numeric/5 / numeric-capture/conformance.json:0[Numeric]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4 | null |

oracle/backref.json / backref/1 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":[],"kind":"scope","length":1,"message":"Reference mismatch: x","offset":0,"severity":"ERROR"} |
| /diagnostics/1 | 欠落 | {"expected":[],"kind":"scope","length":1,"message":"Reference mismatch: x","offset":2,"severity":"ERROR"} |

oracle/backref.json / backref/2 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/kind | "ERROR" | "scope" |
| /diagnostics/0/message | "back-reference mismatch: expected 'x' but got 'y'" | "Reference mismatch: x" |
| /diagnostics/0/offset | 2 | 0 |

oracle/backref.json / backref/3 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "AlphabetUnderScoreParser" | "IDENTIFIER" |
| /diagnostics/0/expected/1 | "IdentifierParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/backref.json / backref/4 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":[],"kind":"scope","length":3,"message":"Reference mismatch: foo","offset":0,"severity":"ERROR"} |
| /diagnostics/1 | 欠落 | {"expected":[],"kind":"scope","length":3,"message":"Reference mismatch: foo","offset":4,"severity":"ERROR"} |

oracle/catalog.json / catalog/3 / oracle/catalog.json:0[@catalog]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "'Z'" | "'z'" |
| /diagnostics/0/expected/1 | "'z'" | 欠落 |

oracle/char-range.json / char-range/1 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "RootParser" | "CHAR_RANGE" |
| /diagnostics/0/expected/1 | "TParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/char-range.json / char-range/4 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "RootParser" | "CHAR_RANGE" |
| /diagnostics/0/expected/1 | "TParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/char-range.json / char-range/5 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "RootParser" | "CHAR_RANGE" |
| /diagnostics/0/expected/1 | "TParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/ci-unicode.json / ci-unicode/3 / oracle/ci-unicode.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "'é'" | "é" |

oracle/ci.json / ci/2 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,3],"text":"LeT"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/ci.json / ci/4 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "'let'" | "let" |

oracle/common-field.json / common-field/2 / oracle/common-field.json:0[@commonField]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'a:'"],"kind":"syntax","offset":0} |

oracle/digit.json / digit/1 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["EOF"],"kind":"syntax","offset":1} |

oracle/digit.json / digit/2 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "EndOfSourceParser" | 欠落 |

oracle/digit.json / digit/3 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "RootParser" | "Digit" |
| /diagnostics/0/expected/1 | "TParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/digit.json / digit/4 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0/expected/0 | "RootParser" | "Digit" |
| /diagnostics/0/expected/1 | "TParser" | 欠落 |

oracle/empty-one-two.json / empty-one-two/1 / oracle/empty-one-two.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/empty-two.json / empty-two/1 / oracle/empty-two.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "Repeat" | 欠落 |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/offset | 0 | 1 |

oracle/empty-zero.json / empty-zero/1 / oracle/empty-zero.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "Repeat" | 欠落 |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/offset | 0 | 1 |

oracle/empty.json / empty/1 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/empty.json / empty/2 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["ANY"],"kind":"syntax","offset":0} |

oracle/empty.json / empty/3 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/enum-plain.json / enum-plain/2 / oracle/enum-plain.json:0[@enum]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'on'"],"kind":"syntax","offset":0} |

oracle/eof.json / eof/1 / oracle/eof.json:0[EOF]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["EOF"],"kind":"syntax","offset":0} |

oracle/eof.json / eof/2 / oracle/eof.json:0[EOF]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "EndOfSourceParser" | 欠落 |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/eof.json / eof/3 / oracle/eof.json:0[EOF]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "EndOfSourceParser" | 欠落 |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/longest.json / longest/1 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'ab'"],"kind":"syntax","offset":0} |

oracle/longest.json / longest/3 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"B:0","name":"value","span":[0,2],"text":"ab"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/longest.json / longest/4 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/2 | "RootParser" | 欠落 |

oracle/lookahead-until.json / lookahead-until/1 / oracle/lookahead-until.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/lookahead.json / lookahead/1 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/lookahead.json / lookahead/2 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/lookahead.json / lookahead/3 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "'a'" | "a" |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |

oracle/lookahead.json / lookahead/4 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0/expected/0 | "'a'" | "a" |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |

oracle/mapping-alias.json / mapping-alias/2 / oracle/mapping-alias.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/mapping-alias.json / mapping-alias/3 / oracle/mapping-alias.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |

oracle/mapping-dotted.json / mapping-dotted/2 / oracle/mapping-dotted.json:0[dotted-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'a'"],"kind":"syntax","offset":0} |

oracle/mapping-sum.json / mapping-sum/2 / oracle/mapping-sum.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'a'"],"kind":"syntax","offset":0} |

oracle/negation-empty.json / negation-empty/1 / oracle/negation-empty.json:0[NEGATION]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0/expected/0 | "RootParser" | "NEGATION" |
| /diagnostics/0/expected/1 | "TParser" | 欠落 |

oracle/negative-lookahead.json / negative-lookahead/2 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/negative-lookahead.json / negative-lookahead/3 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "'a'" | "a" |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |

oracle/numeric.json / numeric/2 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /mappingError | "java.lang.NumberFormatException: For input string: \"2147483648\"" | "NUMBER cannot be mapped to int at Span[start=1, end=11]" |

oracle/numeric.json / numeric/3 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /mappingError | "java.lang.NumberFormatException: For input string: \"-2147483649\"" | "NUMBER cannot be mapped to int at Span[start=1, end=12]" |

oracle/numeric.json / numeric/4 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /mappingError | "java.lang.NumberFormatException: For input string: \"1.5\"" | "NUMBER cannot be mapped to int at Span[start=1, end=4]" |

oracle/numeric.json / numeric/5 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /mappingError | "java.lang.NumberFormatException: For input string: \"1e2\"" | "NUMBER cannot be mapped to int at Span[start=1, end=4]" |

oracle/numeric.json / numeric/7 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "DigitParser" | "NUMBER" |
| /diagnostics/0/expected/1 | "OneOrMore" | 欠落 |

oracle/posix.json / posix/2 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "RootParser" | "Alphabet" |
| /diagnostics/0/expected/1 | "TParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/posix.json / posix/3 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "RootParser" | "Alphabet" |
| /diagnostics/0/expected/1 | "TParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/posix.json / posix/4 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"A"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/predictive-nullable.json / predictive-nullable/1 / oracle/predictive-nullable.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'a'"],"kind":"syntax","offset":0} |

oracle/predictive-nullable.json / predictive-nullable/2 / oracle/predictive-nullable.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'a'","'x'"],"kind":"syntax","offset":0} |

oracle/predictive.json / predictive/3 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'a'"],"kind":"syntax","offset":0} |

oracle/predictive.json / predictive/4 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/2 | "'x'" | 欠落 |

oracle/predictive.json / predictive/5 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0 | 欠落 | {"expected":["'a'","'b'"],"kind":"syntax","offset":0} |

oracle/regex-ascii.json / regex-ascii/1 / oracle/regex-ascii.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,2],"text":"ab"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/regex-emoji.json / regex-emoji/1 / oracle/regex-emoji.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | false | true |
| /allConsumed | false | true |
| /ast | null | {"fields":{"value":"😀"},"span":[0,1],"type":"Value"} |

oracle/regex-emoji.json / regex-emoji/2 / oracle/regex-emoji.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"😀x"},"span":[0,2],"type":"Value"} | null |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,2],"text":"😀x"} | 欠落 |

oracle/regex-prefix.json / regex-prefix/1 / oracle/regex-prefix.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | false | true |
| /allConsumed | false | true |
| /ast | null | {"fields":{"value":"a"},"span":[0,2],"type":"Value"} |

oracle/regex-zero.json / regex-zero/1 / oracle/regex-zero.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0/expected/0 | "REGEX('a*')" | "a*" |

oracle/regex-zero.json / regex-zero/2 / oracle/regex-zero.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "REGEX('a*')" | "a*" |

oracle/repeat-unbounded.json / repeat-unbounded/1 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0/expected/1 | "Repeat" | 欠落 |
| /diagnostics/0/expected/2 | "RootBounded0Parser" | 欠落 |

oracle/repeat-unbounded.json / repeat-unbounded/2 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/1 | "RootBounded0Parser" | 欠落 |

oracle/repeat-unbounded.json / repeat-unbounded/3 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'a'"],"kind":"syntax","offset":2} |

oracle/repeat-unbounded.json / repeat-unbounded/4 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"values","span":[0,1],"text":"a"} | 欠落 |
| /captures/1 | {"binding":"Root:0","name":"values","span":[1,2],"text":"a"} | 欠落 |
| /captures/2 | {"binding":"Root:0","name":"values","span":[2,3],"text":"a"} | 欠落 |

oracle/repeat-zero.json / repeat-zero/1 / oracle/repeat-zero.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["'a'"],"kind":"syntax","offset":0} |

oracle/repeat-zero.json / repeat-zero/2 / oracle/repeat-zero.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/0 | "'a'" | 欠落 |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/offset | 0 | 1 |

oracle/separated.json / separated/1 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/expected/2 | "RootSep0Parser" | 欠落 |

oracle/separated.json / separated/2 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["','"],"kind":"syntax","offset":1} |

oracle/separated.json / separated/3 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["','"],"kind":"syntax","offset":3} |

oracle/separated.json / separated/4 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"values","span":[0,1],"text":"a"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "'a'" |

oracle/separated.json / separated/5 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/expected/2 | "RootSep0Parser" | 欠落 |

oracle/skip.json / skip/1 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["','"],"kind":"syntax","offset":3} |

oracle/skip.json / skip/2 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["','"],"kind":"syntax","offset":1} |

oracle/skip.json / skip/3 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Item:0","name":"value","span":[0,1],"text":"a"} | 欠落 |
| /captures/1 | {"binding":"Root:0","name":"items","span":[0,1],"text":"a"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "'a'" |

oracle/skip.json / skip/4 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /diagnostics/0/expected/1 | "RootParser" | 欠落 |
| /diagnostics/0/expected/2 | "__CaptureSite" | 欠落 |

oracle/until.json / until/1 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["#","ANY"],"kind":"syntax","offset":3} |

oracle/until.json / until/2 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["#","ANY"],"kind":"syntax","offset":2} |

oracle/until.json / until/3 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/until.json / until/4 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "EOF" |

oracle/until.json / until/5 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["#","ANY"],"kind":"syntax","offset":0} |

primitives/corpus.json / 0:ANY/0 / primitives/corpus.json:0[0:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

primitives/corpus.json / 0:ANY/1 / primitives/corpus.json:0[0:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "😀" | null |

primitives/corpus.json / 0:ANY/2 / primitives/corpus.json:0[0:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "あ" | null |

primitives/corpus.json / 10:UNTIL('*/')/0 / primitives/corpus.json:10[10:UNTIL('*/')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a*b" | null |

primitives/corpus.json / 10:UNTIL('*/')/1 / primitives/corpus.json:10[10:UNTIL('*/')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 11:ANY/0 / primitives/corpus.json:11[11:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

primitives/corpus.json / 11:ANY/1 / primitives/corpus.json:11[11:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "😀" | null |

primitives/corpus.json / 12:UNTIL('#')/0 / primitives/corpus.json:12[12:UNTIL('#')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "abc" | null |

primitives/corpus.json / 12:UNTIL('#')/1 / primitives/corpus.json:12[12:UNTIL('#')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 13:UNTIL('')/0 / primitives/corpus.json:13[13:UNTIL('')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "abc" | null |

primitives/corpus.json / 13:UNTIL('')/1 / primitives/corpus.json:13[13:UNTIL('')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 14:LOOKAHEAD('')/0 / primitives/corpus.json:14[14:LOOKAHEAD('')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | false | true |

primitives/corpus.json / 15:NEGATIVE_LOOKAHEAD('')/0 / primitives/corpus.json:15[15:NEGATIVE_LOOKAHEAD('')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /value | "a" | null |

primitives/corpus.json / 16:LOOKAHEAD('a')/0 / primitives/corpus.json:16[16:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "ab" | null |

primitives/corpus.json / 19:EMPTY/0 / primitives/corpus.json:19[19:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "ab" | null |

primitives/corpus.json / 1:CHAR_RANGE('a','z')/0 / primitives/corpus.json:1[1:CHAR_RANGE('a','z')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

primitives/corpus.json / 1:CHAR_RANGE('a','z')/1 / primitives/corpus.json:1[1:CHAR_RANGE('a','z')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "z" | null |

primitives/corpus.json / 20:LOOKAHEAD('a')/0 / primitives/corpus.json:20[20:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

primitives/corpus.json / 22:LOOKAHEAD('a')/0 / primitives/corpus.json:22[22:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 23:EMPTY/0 / primitives/corpus.json:23[23:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 24:UNTIL('#')/0 / primitives/corpus.json:24[24:UNTIL('#')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "abc" | null |

primitives/corpus.json / 24:UNTIL('#')/1 / primitives/corpus.json:24[24:UNTIL('#')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 25:ANY/0 / primitives/corpus.json:25[25:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | [] | null |

primitives/corpus.json / 25:ANY/1 / primitives/corpus.json:25[25:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","😀"] | null |

primitives/corpus.json / 26:EMPTY/0 / primitives/corpus.json:26[26:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 27:LOOKAHEAD('a')/0 / primitives/corpus.json:27[27:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

primitives/corpus.json / 28:LOOKAHEAD('a')/0 / primitives/corpus.json:28[28:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

primitives/corpus.json / 29:LOOKAHEAD('a')/0 / primitives/corpus.json:29[29:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /value | "ab" | null |

primitives/corpus.json / 29:LOOKAHEAD('a')/2 / primitives/corpus.json:29[29:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /value | "ab" | null |

primitives/corpus.json / 2:CHAR_RANGE('あ','お')/0 / primitives/corpus.json:2[2:CHAR_RANGE('あ','お')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "う" | null |

primitives/corpus.json / 30:LOOKAHEAD('a')/0 / primitives/corpus.json:30[30:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /matched | 1 | 0 |

primitives/corpus.json / 30:LOOKAHEAD('a')/1 / primitives/corpus.json:30[30:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /matched | 2 | 1 |

primitives/corpus.json / 31:LOOKAHEAD('a')/0 / primitives/corpus.json:31[31:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "ab" | null |

primitives/corpus.json / 31:LOOKAHEAD('a')/1 / primitives/corpus.json:31[31:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "ab" | null |

primitives/corpus.json / 33:EMPTY/0 / primitives/corpus.json:33[33:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "ab" | null |

primitives/corpus.json / 37:LOOKAHEAD('a')/0 / primitives/corpus.json:37[37:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | false | true |

primitives/corpus.json / 38:LOOKAHEAD('a')/0 / primitives/corpus.json:38[38:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | false | true |

primitives/corpus.json / 39:NEGATION('')/0 / primitives/corpus.json:39[39:NEGATION('')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "😀" | null |

primitives/corpus.json / 3:NEGATION('x😀')/0 / primitives/corpus.json:3[3:NEGATION('x😀')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

primitives/corpus.json / 3:NEGATION('x😀')/1 / primitives/corpus.json:3[3:NEGATION('x😀')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "あ" | null |

primitives/corpus.json / 3:NEGATION('x😀')/2 / primitives/corpus.json:3[3:NEGATION('x😀')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "🚀" | null |

primitives/corpus.json / 40:EMPTY/0 / primitives/corpus.json:40[40:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | [""] | null |

primitives/corpus.json / 41:EMPTY/0 / primitives/corpus.json:41[41:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | [""] | null |

primitives/corpus.json / 41:EMPTY/1 / primitives/corpus.json:41[41:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["",""] | null |

primitives/corpus.json / 41:EMPTY/2 / primitives/corpus.json:41[41:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["","",""] | null |

primitives/corpus.json / 42:EMPTY/0 / primitives/corpus.json:42[42:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | {"fields":{"text":""},"span":[1,1],"type":"Item"} | null |

primitives/corpus.json / 43:LOOKAHEAD('a')/0 / primitives/corpus.json:43[43:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "ab" | null |

primitives/corpus.json / 44:ANY/0 / primitives/corpus.json:44[44:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","b"] | null |

primitives/corpus.json / 44:ANY/1 / primitives/corpus.json:44[44:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","b"] | null |

primitives/corpus.json / 45:ANY/0 / primitives/corpus.json:45[45:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","b"] | null |

primitives/corpus.json / 45:ANY/1 / primitives/corpus.json:45[45:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","b"] | null |

primitives/corpus.json / 46:ANY/0 / primitives/corpus.json:46[46:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a /*c*/","b"] | null |

primitives/corpus.json / 47:ANY/0 / primitives/corpus.json:47[47:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | ["a","a"] | null |

primitives/corpus.json / 4:EMPTY/0 / primitives/corpus.json:4[4:EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 5:EOF/0 / primitives/corpus.json:5[5:EOF]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

primitives/corpus.json / 6:LOOKAHEAD('ab')/0 / primitives/corpus.json:6[6:LOOKAHEAD('ab')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "ab" | null |

primitives/corpus.json / 7:NEGATIVE_LOOKAHEAD('x')/0 / primitives/corpus.json:7[7:NEGATIVE_LOOKAHEAD('x')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "a" | null |

primitives/corpus.json / 8:LOOKAHEAD('😀')/0 / primitives/corpus.json:8[8:LOOKAHEAD('😀')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "😀" | null |

primitives/corpus.json / 9:UNTIL('#')/0 / primitives/corpus.json:9[9:UNTIL('#')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "abc" | null |

primitives/corpus.json / 9:UNTIL('#')/1 / primitives/corpus.json:9[9:UNTIL('#')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "😀" | null |

primitives/corpus.json / 9:UNTIL('#')/2 / primitives/corpus.json:9[9:UNTIL('#')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

right-associative/corpus.json / Power.ubnf/0 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/0 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/0 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/0 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/1 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 8 | null |

right-associative/corpus.json / Power.ubnf/1 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 8 | null |

right-associative/corpus.json / Power.ubnf/1 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 8 | null |

right-associative/corpus.json / Power.ubnf/1 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 8 | null |

right-associative/corpus.json / Power.ubnf/10 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/10 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/10 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/10 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/11 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 64 | null |

right-associative/corpus.json / Power.ubnf/12 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/13 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 8 | null |

right-associative/corpus.json / Power.ubnf/14 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/15 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/2 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/2 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/2 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/2 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/3 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 81 | null |

right-associative/corpus.json / Power.ubnf/3 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 81 | null |

right-associative/corpus.json / Power.ubnf/3 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 81 | null |

right-associative/corpus.json / Power.ubnf/3 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 81 | null |

right-associative/corpus.json / Power.ubnf/4 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/4 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/4 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/4 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/5 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1 | null |

right-associative/corpus.json / Power.ubnf/5 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1 | null |

right-associative/corpus.json / Power.ubnf/5 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1 | null |

right-associative/corpus.json / Power.ubnf/5 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1 | null |

right-associative/corpus.json / Power.ubnf/6 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0 | null |

right-associative/corpus.json / Power.ubnf/6 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0 | null |

right-associative/corpus.json / Power.ubnf/6 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0 | null |

right-associative/corpus.json / Power.ubnf/6 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0 | null |

right-associative/corpus.json / Power.ubnf/7 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/7 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/7 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/7 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2 | null |

right-associative/corpus.json / Power.ubnf/8 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/8 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/8 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/8 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/9 / right-associative/corpus.json:0[text]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/9 / right-associative/corpus.json:1[mapped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/9 / right-associative/corpus.json:2[unicode]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

right-associative/corpus.json / Power.ubnf/9 / right-associative/corpus.json:3[grouped]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 512 | null |

rule-trivia/corpus.json / right-associative-local-interleave/local-policy-survives-right-rewrite / rule-trivia/corpus.json:14[right-associative-local-interleave]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/right/0/span/1 | 12 | 13 |

scope-annotations/corpus.json / capture-site-identity/undefined-target-warning / scope-annotations/corpus.json:0[capture-site-identity]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0/expected | 欠落 | [] |
| /scope/diagnostics/0/kind | 欠落 | "scope" |
| /scope/diagnostics/0/message | "未定義のシンボル: 'missing'" | "Undefined reference: missing" |

scope-annotations/corpus.json / failed-choice-rollback/failed-first-alternative-erases-effects / scope-annotations/corpus.json:3[failed-choice-rollback]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0 | 欠落 | {"expected":["'!'"],"kind":"syntax","offset":5} |

scope-annotations/corpus.json / optional-recursive-boundary/optional-target-absent-does-not-use-child-capture / scope-annotations/corpus.json:5[optional-recursive-boundary]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0 | 欠落 | {"expected":["'/'"],"kind":"syntax","offset":11} |

scope-annotations/corpus.json / optional-recursive-boundary/parent-target-does-not-cross-recursive-child / scope-annotations/corpus.json:5[optional-recursive-boundary]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0 | 欠落 | {"expected":["'/'"],"kind":"syntax","offset":18} |

scope-annotations/corpus.json / quantified-capture-occurrences/plus-bounded-and-separated-inner-captures / scope-annotations/corpus.json:6[quantified-capture-occurrences]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0 | 欠落 | {"expected":["','"],"kind":"syntax","offset":9} |

scope-annotations/corpus.json / repeated-capture-occurrences/all-repeat-captures-produce-effects / scope-annotations/corpus.json:4[repeated-capture-occurrences]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0/expected | 欠落 | ["','"] |
| /scope/diagnostics/0/kind | 欠落 | "syntax" |
| /scope/diagnostics/0/length | 7 | 欠落 |

scope-annotations/corpus.json / scope-and-declares-parent-registration/failed-parent-rule-rolls-back-child-commit / scope-annotations/corpus.json:2[scope-and-declares-parent-registration]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0 | 欠落 | {"expected":["IDENTIFIER"],"kind":"syntax","offset":6} |

scope-annotations/corpus.json / scope-and-declares-parent-registration/valid-prefix-with-trailing-input / scope-annotations/corpus.json:2[scope-and-declares-parent-registration]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0 | 欠落 | {"expected":["EOF"],"kind":"trailing_input","offset":10} |

semantic-cardinality/corpus.json / mixed-one/0 / semantic-cardinality/corpus.json:1[mixed-one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)" | null |

semantic-cardinality/corpus.json / mixed-one/1 / semantic-cardinality/corpus.json:1[mixed-one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / mixed-one/2 / semantic-cardinality/corpus.json:1[mixed-one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(😀)" | null |

semantic-cardinality/corpus.json / node-one/0 / semantic-cardinality/corpus.json:0[node-one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / node-one/1 / semantic-cardinality/corpus.json:0[node-one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y" | null |

semantic-cardinality/pure-alias.json / pure-node-delimiter-alias/0 / semantic-cardinality/pure-alias.json:3[pure-node-delimiter-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-delimiter-alias/1 / semantic-cardinality/pure-alias.json:3[pure-node-delimiter-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y" | null |

semantic-cardinality/pure-alias.json / pure-node-delimiter-alias/2 / semantic-cardinality/pure-alias.json:3[pure-node-delimiter-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-direct-multi/0 / semantic-cardinality/pure-alias.json:4[pure-node-direct-multi]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-direct-multi/1 / semantic-cardinality/pure-alias.json:4[pure-node-direct-multi]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:z" | null |

semantic-cardinality/pure-alias.json / pure-node-direct-multi/2 / semantic-cardinality/pure-alias.json:4[pure-node-direct-multi]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:🚀" | null |

semantic-cardinality/pure-alias.json / pure-node-group-alias/0 / semantic-cardinality/pure-alias.json:2[pure-node-group-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-group-alias/1 / semantic-cardinality/pure-alias.json:2[pure-node-group-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-alias/0 / semantic-cardinality/pure-alias.json:5[pure-node-multi-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-alias/1 / semantic-cardinality/pure-alias.json:5[pure-node-multi-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-alias/2 / semantic-cardinality/pure-alias.json:5[pure-node-multi-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:z" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-alias/3 / semantic-cardinality/pure-alias.json:5[pure-node-multi-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:🚀" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-delimiter-alias/0 / semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-delimiter-alias/1 / semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:z" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-delimiter-alias/2 / semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:🚀" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-delimiter-alias/3 / semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:🚀" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-multihop-group-alias/0 / semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-multihop-group-alias/1 / semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:z" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-multihop-group-alias/2 / semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:🚀" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-multihop-group-alias/3 / semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:z" | null |

semantic-cardinality/pure-alias.json / pure-node-multihop-alias/0 / semantic-cardinality/pure-alias.json:1[pure-node-multihop-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-multihop-alias/1 / semantic-cardinality/pure-alias.json:1[pure-node-multihop-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y" | null |

semantic-cardinality/pure-alias.json / pure-node-multihop-alias/2 / semantic-cardinality/pure-alias.json:1[pure-node-multihop-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-one-alias/0 / semantic-cardinality/pure-alias.json:0[pure-node-one-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-one-alias/1 / semantic-cardinality/pure-alias.json:0[pure-node-one-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y" | null |

未観測項目（展開後件数。欠落を Pass に読み替えない）

| 観測 | 件数 |
|---|---:|
| /captures/0/binding | 37 |
| /captures/1/binding | 8 |
| /captures/2/binding | 1 |
| /captures/3/binding | 1 |
| /catalogs | 2 |
| /diagnosticError | 1 |
| /diagnostics/0/farthestExpected | 63 |
| /diagnostics/0/farthestOffset | 63 |
| /diagnostics/0/origin | 64 |
| /enumError | 1 |
| /enumValue | 2 |
| /expectation | 148 |
| /expected | 40 |
| /grammar (操作列の driver protocol 未対応) | 9 |
| /nativeFailure | 120 |
| /nodes | 14 |
| /operations | 9 |
| /parentRollback | 2 |
| /parseError | 1 |
| /parsedStatus | 119 |
| /scope/depth | 14 |
| /scope/resolved | 14 |
| /scopeDepth | 120 |
| /texts | 74 |
| /valueSpans | 9 |

段階失敗の原文（完全な標準出力・標準エラーは JSON）

2f47c7425e70d0c7-007 / generation: nested-captures/corpus.json:7[typed-node-list-and-shared-mixed-schema]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Recipe conversion mismatch: recipe:Text.value
> 	at org.ubnfc.java.ir.Validation.require(Validation.java:24)

ae6bdbdfb30f2a9c-085 / generation: lexical/corpus.json:5[5:DoubleQuotedParser]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:Primitive::T
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

fdb7ca179a878fee-087 / generation: lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:Primitive::T
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

792ec6c908e208db-089 / generation: lexical/corpus.json:9[9:DoubleQuotedParser]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:Primitive::T
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

f17ded7005262c50-090 / generation: lexical/corpus.json:10[10:SingleQuotedParser]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:Primitive::D
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

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

d6e3fd9c8937578f-128 / generation: tiny-code-fence/corpus.json:0[start-direct]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

fdd521c12b635a1f-129 / generation: tiny-code-fence/corpus.json:1[end-direct]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

5441ae4a5100def4-130 / generation: tiny-code-fence/corpus.json:2[prefix-vs-full-input-start]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

67093033f25f8c44-131 / generation: tiny-code-fence/corpus.json:3[prefix-vs-full-input-end]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

472dea76680fd30e-132 / generation: tiny-code-fence/corpus.json:4[start-source-offset-after-crlf]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

6b8d2cca553f70db-133 / generation: tiny-code-fence/corpus.json:5[end-source-offset]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

4df5f8955e5e3bfb-134 / generation: tiny-code-fence/corpus.json:6[java-style-outer-only]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

936d4546f53f9c12-135 / generation: tiny-code-fence/corpus.json:7[p4-zero-field-java-style]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

f273d1b663b90b51-136 / generation: tiny-code-fence/corpus.json:8[transparent-aliases]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

38e17ae9cee10518-137 / generation: tiny-code-fence/corpus.json:9[optional-start]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

fd6cb6c3b5388d8f-138 / generation: tiny-code-fence/corpus.json:10[repeat-start]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

a1b1c138c0fa90db-139 / generation: tiny-code-fence/corpus.json:11[combined-fence]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyFence::START
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

266b34d2c878021e-140 / generation: tiny-string-token/corpus.json:0[direct-eof]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyString::STRING
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

4b3813d28e07d2b9-141 / generation: tiny-string-token/corpus.json:1[prefix-vs-full-input]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyString::STRING
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

082a886e9fe7d162-142 / generation: tiny-string-token/corpus.json:2[comments-and-eof]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyString::STRING
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

1d83999d5b935530-143 / generation: tiny-string-token/corpus.json:3[transparent-alias]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyString::STRING
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

8bce993cf0c67dd3-144 / generation: tiny-string-token/corpus.json:4[optional]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyString::STRING
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

0a2ab1488382b82c-145 / generation: tiny-string-token/corpus.json:5[repeat]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyString::STRING
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

1073c661e033ad07-146 / generation: tiny-string-token/corpus.json:6[mapped-node]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:TinyString::STRING
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

6470655b1bf28b1a-151 / generation: mixed-values/shared-corpus.json:0[first]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Recipe conversion mismatch: recipe:Second.value
> 	at org.ubnfc.java.ir.Validation.require(Validation.java:24)

736325e6c69f9ed2-152 / generation: mixed-values/shared-corpus.json:1[second]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Recipe conversion mismatch: recipe:Second.value
> 	at org.ubnfc.java.ir.Validation.require(Validation.java:24)

48c346ec23ef1c06-158 / ir: associative/corpus.json:1[reversed-metadata]

> 終了 exit status: 1:
> ubnfc: rule Expression precedence 30 must be lower than referenced operator rule Term precedence 20 [code: E-PRECEDENCE-ORDER] [hint: Decrease Expression level or increase Term level.]

668cf0ed36645997-179 / ir: oracle/common-field-missing.json:0[@commonField]

> 終了 exit status: 1:
> ubnfc: E-LOWER-COMMON-FIELD: ProbeAST.Node.name: variants の field 型/cardinality が不整合

d8ba3c8d7194ffb2-180 / ir: oracle/common-field-type.json:0[@commonField]

> 終了 exit status: 1:
> ubnfc: E-LOWER-COMMON-FIELD: ProbeAST.Node.name: variants の field 型/cardinality が不整合

0489ef2c71ddce15-187 / generation: oracle/enum-collision.json:0[@enum]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Duplicate enum name: ProbeAST.Mode
> 	at org.ubnfc.java.ir.Validation.require(Validation.java:24)

1ae620c0ffbb124d-191 / generation: oracle/error.json:0[ERROR]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: execution.transaction:error:once
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

320ab6b6ec2cc8a9-192 / generation: oracle/eval-binary_arithmetic.json:0[@eval]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported requiredFeatures: eval
> 	at org.ubnfc.java.ir.IrReader.read(IrReader.java:28)

000a3a5238bb6cc9-193 / generation: oracle/eval-conditional.json:0[@eval]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported requiredFeatures: eval
> 	at org.ubnfc.java.ir.IrReader.read(IrReader.java:28)

bf0a74f72822971b-194 / generation: oracle/eval-literal.json:0[@eval]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported requiredFeatures: eval
> 	at org.ubnfc.java.ir.IrReader.read(IrReader.java:28)

89160f03adabb722-195 / generation: oracle/eval-manual.json:0[@eval]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported requiredFeatures: eval
> 	at org.ubnfc.java.ir.IrReader.read(IrReader.java:28)

ec991e1179775a0a-196 / generation: oracle/eval-passthrough.json:0[@eval]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported requiredFeatures: eval
> 	at org.ubnfc.java.ir.IrReader.read(IrReader.java:28)

4aaea9c4ba5b21d8-197 / ir: oracle/eval-sum.json:0[@eval]

> 終了 exit status: 1:
> ubnfc: E-LOWER-SUM-EVAL: Node: 自動 @eval(literal) は具象型が必要

92fd086142e9daaa-198 / generation: oracle/eval-unknown.json:0[@eval]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported requiredFeatures: eval
> 	at org.ubnfc.java.ir.IrReader.read(IrReader.java:28)

4e2875aab6b5ab17-199 / generation: oracle/eval-variable_ref.json:0[@eval]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported requiredFeatures: eval
> 	at org.ubnfc.java.ir.IrReader.read(IrReader.java:28)

c6e53876beadbbe7-200 / generation: oracle/extern-full.json:0[Extern]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:Probe::T
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

bc7461d8cc57ad13-201 / generation: oracle/extern.json:0[Extern]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:Probe::T
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

3f95b76dccbb0235-203 / ir: oracle/import-cycle.json:0[@import]

> 終了 exit status: 1:
> ubnfc: 循環 import を検出しました: &lt;build&gt;/grammar/B.ubnf (スタック: [&lt;build&gt;/grammar/__runall__.ubnf, &lt;build&gt;/grammar/B.ubnf, &lt;build&gt;/grammar/C.ubnf]) [code: E-IMPORT-CYCLE] [hint: Remove the circular import.]

6bf1db86b87e3cd7-216 / generation: oracle/predictive-extern.json:0[@predictiveChoice]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: externTokens.inputState:Probe::T
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

6158dbda03072d94-219 / generation: oracle/recovery-auto.json:0[@recovery]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: recovery.legacyMarker
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

fcabbee7dbead518-220 / generation: oracle/recovery-root.json:0[@recovery]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: recovery.legacyMarker
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

6579df10379cb598-221 / generation: oracle/recovery-skip.json:0[@recovery]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: recovery.legacyMarker
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

ec619df4c9ba410e-222 / generation: oracle/recovery-sync.json:0[@recovery]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported IR feature: recovery.legacyMarker
> 	at org.ubnfc.java.ir.Validation.unsupported(Validation.java:23)

0f54bfb5d29e92fd-232 / generation: oracle/typeof.json:0[@typeof]

> 終了 exit status: 1:
> Exception in thread "main" java.lang.IllegalArgumentException: Unsupported requiredFeatures: typeof
> 	at org.ubnfc.java.ir.IrReader.read(IrReader.java:28)
