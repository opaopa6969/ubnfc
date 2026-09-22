Rust corpus run-all

元 corpus: 83 ファイル、871 ケース。文法 variant: 236、hash キャッシュ: 234 build、展開後: 1195 件（文法なし操作列を含む）。jobs=cargo（既定）。

集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。

Rust generator SHA-256: `379256cc544c0487a2cb2ab22fad1861577ef4abfc342782ee5acdaaa1861b53`

front SHA-256: `a0b32e42d9221ed1f7c6cee26811e0e31f2b3284a4e1854556053c39f436fff5`

Rust 一括 cargo build --release --offline --workspace: 34.599 秒。個別切り分け込み: 34.599 秒、cargo 起動 1 回。生成 189 crate、生成物 hash 重複除去後 189 crate、既存 cache hit 0 件。

JSON の build_failure / builds[].cargo は Rust、javac_failure / builds[].javac は Java の build 段階。表の build 失敗は両者の合計。比較規則は共通で、診断 offset と数値字句は backend ごとの既存 oracle を使う。

Pass のうち明示的な期待拒否: 1件。build の実測失敗と診断は拒否理由表に保持する。

| 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 24 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 50 | 0 | 20 | 0 |
| catalog | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| evolution | 0 | 0 | 0 | 0 | 4 | 0 | 4 | 148 |
| lexical | 0 | 0 | 0 | 0 | 44 | 0 | 34 | 0 |
| mixed-values | 0 | 0 | 0 | 0 | 49 | 0 | 49 | 0 |
| nested-captures | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 5 |
| numeric-capture | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 |
| oracle | 3 | 41 | 0 | 0 | 1 | 10 | 35 | 78 |
| primitives | 0 | 0 | 0 | 0 | 95 | 0 | 14 | 0 |
| right-associative | 0 | 0 | 0 | 0 | 63 | 0 | 49 | 0 |
| rule-trivia | 0 | 0 | 0 | 0 | 32 | 0 | 1 | 0 |
| scope-annotations | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 9 |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 101 | 0 | 0 | 0 | 32 | 0 | 30 | 0 |
| tiny-code-fence | 0 | 0 | 0 | 0 | 39 | 0 | 34 | 0 |
| tiny-string-token | 0 | 0 | 0 | 0 | 12 | 0 | 27 | 0 |
| 合計 | 128 | 43 | 0 | 0 | 458 | 10 | 307 | 249 |

6 系統別（case × variant 単位）

| loader 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 24 | 0 | 0 | 0 | 154 | 0 | 73 | 148 |
| inline-ast | 0 | 0 | 0 | 0 | 32 | 0 | 11 | 14 |
| operations | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 9 |
| oracle | 3 | 41 | 0 | 0 | 1 | 10 | 35 | 78 |
| rule | 101 | 0 | 0 | 0 | 132 | 0 | 140 | 0 |
| token | 0 | 0 | 0 | 0 | 139 | 0 | 48 | 0 |
| 合計 | 128 | 43 | 0 | 0 | 458 | 10 | 307 | 249 |

文法別（case × variant 単位。文法なしは操作列）

| 文法 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| associative/corpus.json:0[normal] | 0 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| associative/corpus.json:1[reversed-metadata] | 24 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cardinality/corpus.json:0[zero] | 0 | 0 | 0 | 0 | 8 | 0 | 6 | 0 |
| cardinality/corpus.json:1[plus] | 0 | 0 | 0 | 0 | 10 | 0 | 4 | 0 |
| cardinality/corpus.json:2[bounded] | 0 | 0 | 0 | 0 | 11 | 0 | 3 | 0 |
| cardinality/corpus.json:3[unbounded] | 0 | 0 | 0 | 0 | 10 | 0 | 4 | 0 |
| cardinality/corpus.json:4[separated] | 0 | 0 | 0 | 0 | 11 | 0 | 3 | 0 |
| catalog/corpus.json:0[deduplicated-sorted-captures] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| catalog/corpus.json:1[rule-order-locality-quantifiers-and-escaped-context] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| evolution/conformance.json:0[0] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:1[1] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:2[2] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:3[3] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/diagnostics.json:0[3] | 0 | 0 | 0 | 0 | 4 | 0 | 4 | 0 |
| lexical/corpus.json:0[0:IdentifierParser] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 |
| lexical/corpus.json:10[10:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| lexical/corpus.json:11[11:EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:12[12:org.unlaxer.parser.elementary.EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:13[13:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:14[14:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:15[15:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:16[16:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| lexical/corpus.json:17[17:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:18[18:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:19[19:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:1[1:org.unlaxer.parser.clang.IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| lexical/corpus.json:20[20:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| lexical/corpus.json:21[21:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| lexical/corpus.json:2[2:IdentifierParser] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:3[3:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| lexical/corpus.json:4[4:SingleQuotedParser] | 0 | 0 | 0 | 0 | 7 | 0 | 14 | 0 |
| lexical/corpus.json:5[5:DoubleQuotedParser] | 0 | 0 | 0 | 0 | 3 | 0 | 6 | 0 |
| lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:8[8:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| lexical/corpus.json:9[9:DoubleQuotedParser] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| mixed-values/corpus.json:0[two] | 0 | 0 | 0 | 0 | 10 | 0 | 13 | 0 |
| mixed-values/corpus.json:1[one] | 0 | 0 | 0 | 0 | 15 | 0 | 8 | 0 |
| mixed-values/delimited-corpus.json:0[two] | 0 | 0 | 0 | 0 | 6 | 0 | 9 | 0 |
| mixed-values/delimited-corpus.json:1[one] | 0 | 0 | 0 | 0 | 6 | 0 | 9 | 0 |
| mixed-values/shared-corpus.json:0[first] | 0 | 0 | 0 | 0 | 6 | 0 | 5 | 0 |
| mixed-values/shared-corpus.json:1[second] | 0 | 0 | 0 | 0 | 6 | 0 | 5 | 0 |
| nested-captures/corpus.json:0[nested-text-completion] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| nested-captures/corpus.json:1[choice-optional-coexist] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| nested-captures/corpus.json:2[quantifier-items-not-containers] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:3[same-wrapper-and-parser-class] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| nested-captures/corpus.json:4[text-node-and-mixed-values] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| nested-captures/corpus.json:5[recursive-child-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:6[shared-mapping-nested-and-sequential] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:7[typed-node-list-and-shared-mixed-schema] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| numeric-capture/conformance.json:0[Numeric] | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 |
| oracle/backref.json:0[@backref-no-scope] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| oracle/catalog.json:0[@catalog] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 5 |
| oracle/ci-unicode.json:0[CI] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/ci.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 3 |
| oracle/comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 2 |
| oracle/comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/common-field-missing.json:0[@commonField] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/common-field-type.json:0[@commonField] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/digit.json:0[DigitParser] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 3 |
| oracle/empty-one-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/empty-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/empty-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/empty.json:0[EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/enum-collision.json:0[@enum] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/enum.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/eof.json:0[EOF] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/error.json:0[ERROR] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-binary_arithmetic.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-conditional.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-literal.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-manual.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-passthrough.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-sum.json:0[@eval] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-unknown.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-variable_ref.json:0[@eval] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/extern-full.json:0[Extern] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| oracle/extern.json:0[Extern] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| oracle/import-alias.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/import-cycle.json:0[@import] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/import-two-level.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 3 |
| oracle/lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead-until.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| oracle/mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 0 |
| oracle/posix.json:0[posix] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 3 |
| oracle/predictive-extern.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 4 |
| oracle/recovery-auto.json:0[@recovery] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/recovery-root.json:0[@recovery] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/recovery-skip.json:0[@recovery] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/recovery-sync.json:0[@recovery] | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-ascii.json:0[REGEX] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-backref.json:0[REGEX] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-emoji.json:0[REGEX] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-prefix.json:0[REGEX] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-zero.json:0[REGEX] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/repeat-unbounded.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 3 |
| oracle/repeat-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/separated.json:0[separated] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 4 |
| oracle/skip.json:0[@skip] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 3 |
| oracle/typeof.json:0[@typeof] | 0 | 5 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/until.json:0[UNTIL] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 3 |
| primitives/corpus.json:0[0:ANY] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 |
| primitives/corpus.json:10[10:UNTIL('*/')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:11[11:ANY] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:12[12:UNTIL('#')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:13[13:UNTIL('')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:14[14:LOOKAHEAD('')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| primitives/corpus.json:15[15:NEGATIVE_LOOKAHEAD('')] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
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
| primitives/corpus.json:25[25:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| primitives/corpus.json:26[26:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:27[27:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:28[28:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:29[29:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:2[2:CHAR_RANGE('あ','お')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:30[30:LOOKAHEAD('a')] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
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
| primitives/corpus.json:44[44:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:45[45:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| primitives/corpus.json:46[46:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:47[47:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:4[4:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:5[5:EOF] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:6[6:LOOKAHEAD('ab')] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 |
| primitives/corpus.json:7[7:NEGATIVE_LOOKAHEAD('x')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| primitives/corpus.json:8[8:LOOKAHEAD('😀')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| primitives/corpus.json:9[9:UNTIL('#')] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 |
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
| scope-annotations/corpus.json:2[scope-and-declares-parent-registration] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| scope-annotations/corpus.json:3[failed-choice-rollback] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| scope-annotations/corpus.json:4[repeated-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:5[optional-recursive-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| scope-annotations/corpus.json:6[quantified-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:7[nested-same-name-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-annotations/corpus.json:8[trimmed-group-non-bmp] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
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
| tiny-code-fence/corpus.json:0[start-direct] | 0 | 0 | 0 | 0 | 16 | 0 | 5 | 0 |
| tiny-code-fence/corpus.json:10[repeat-start] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| tiny-code-fence/corpus.json:11[combined-fence] | 0 | 0 | 0 | 0 | 5 | 0 | 9 | 0 |
| tiny-code-fence/corpus.json:1[end-direct] | 0 | 0 | 0 | 0 | 5 | 0 | 4 | 0 |
| tiny-code-fence/corpus.json:2[prefix-vs-full-input-start] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| tiny-code-fence/corpus.json:3[prefix-vs-full-input-end] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| tiny-code-fence/corpus.json:4[start-source-offset-after-crlf] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| tiny-code-fence/corpus.json:5[end-source-offset] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| tiny-code-fence/corpus.json:6[java-style-outer-only] | 0 | 0 | 0 | 0 | 3 | 0 | 1 | 0 |
| tiny-code-fence/corpus.json:7[p4-zero-field-java-style] | 0 | 0 | 0 | 0 | 2 | 0 | 3 | 0 |
| tiny-code-fence/corpus.json:8[transparent-aliases] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| tiny-code-fence/corpus.json:9[optional-start] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 |
| tiny-string-token/corpus.json:0[direct-eof] | 0 | 0 | 0 | 0 | 6 | 0 | 14 | 0 |
| tiny-string-token/corpus.json:1[prefix-vs-full-input] | 0 | 0 | 0 | 0 | 1 | 0 | 3 | 0 |
| tiny-string-token/corpus.json:2[comments-and-eof] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 |
| tiny-string-token/corpus.json:3[transparent-alias] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| tiny-string-token/corpus.json:4[optional] | 0 | 0 | 0 | 0 | 2 | 0 | 2 | 0 |
| tiny-string-token/corpus.json:5[repeat] | 0 | 0 | 0 | 0 | 2 | 0 | 2 | 0 |
| tiny-string-token/corpus.json:6[mapped-node] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| 合計 | 128 | 43 | 0 | 0 | 458 | 10 | 307 | 249 |

拒否理由（M3 作業リスト）

同一文法の拒否はキャッシュ単位に数える。ケース数は理由ごとに重複除去する。生成器が最初の拒否で停止するため、後続の未対応機能を網羅する件数ではない。IR 診断が複数あると理由間で重複する。

| 段階 / 機能・診断コード | build 数 | 元ケース数 | 展開後件数 |
|---|---:|---:|---:|
| generation/caseInsensitive.unicode | 1 | 3 | 3 |
| generation/catalog | 3 | 5 | 5 |
| generation/eval | 7 | 7 | 7 |
| generation/execution.transaction | 1 | 3 | 3 |
| generation/invalid Grammar IR: duplicate enum name/literal: ProbeAST.Mode | 1 | 1 | 1 |
| generation/recovery | 4 | 10 | 10 |
| generation/regex | 5 | 9 | 9 |
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
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 0 | null |

cardinality/corpus.json / Cardinality.ubnf/1 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | "x" | "xxx" |
| /ast/fields/flags/1 | "x" | 欠落 |
| /ast/fields/flags/2 | "x" | 欠落 |

cardinality/corpus.json / Cardinality.ubnf/1 / cardinality/corpus.json:1[plus]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | "x" | "xxx" |
| /ast/fields/flags/1 | "x" | 欠落 |
| /ast/fields/flags/2 | "x" | 欠落 |

cardinality/corpus.json / Cardinality.ubnf/1 / cardinality/corpus.json:2[bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | "x" | "xxx" |
| /ast/fields/flags/1 | "x" | 欠落 |
| /ast/fields/flags/2 | "x" | 欠落 |

cardinality/corpus.json / Cardinality.ubnf/1 / cardinality/corpus.json:3[unbounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | "x" | "xxx" |
| /ast/fields/flags/1 | "x" | 欠落 |
| /ast/fields/flags/2 | "x" | 欠落 |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:1[plus]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:2[bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:3[unbounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/2 / cardinality/corpus.json:4[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/3 / cardinality/corpus.json:4[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 5 | null |

cardinality/corpus.json / Cardinality.ubnf/4 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/4 / cardinality/corpus.json:1[plus]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/4 / cardinality/corpus.json:3[unbounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | 欠落 | "" |
| /ast/fields/tail | null | "" |
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/5 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/tail | null | "" |
| /value | 2 | null |

cardinality/corpus.json / Cardinality.ubnf/6 / cardinality/corpus.json:0[zero]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | "x" | "xx" |
| /ast/fields/flags/1 | "x" | 欠落 |
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/6 / cardinality/corpus.json:1[plus]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | "x" | "xx" |
| /ast/fields/flags/1 | "x" | 欠落 |
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/6 / cardinality/corpus.json:2[bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | "x" | "xx" |
| /ast/fields/flags/1 | "x" | 欠落 |
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/6 / cardinality/corpus.json:3[unbounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/flags/0 | "x" | "xx" |
| /ast/fields/flags/1 | "x" | 欠落 |
| /value | 9 | null |

cardinality/corpus.json / Cardinality.ubnf/7 / cardinality/corpus.json:4[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 8 | null |

evolution/diagnostics.json / diagnostics/2 / evolution/diagnostics.json:0[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /expected/0 | "end of input" | "+" |
| /expected/1 | 欠落 | "*" |
| /kind | "trailing_input" | "syntax" |

evolution/diagnostics.json / diagnostics/3 / evolution/diagnostics.json:0[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /expected/0 | "end of input" | "NUMBER" |
| /kind | "trailing_input" | "syntax" |
| /offset | 1 | 2 |

evolution/diagnostics.json / diagnostics/5 / evolution/diagnostics.json:0[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /offset | 5 | 4 |

evolution/diagnostics.json / diagnostics/6 / evolution/diagnostics.json:0[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /offset | 5 | 4 |

lexical/corpus.json / 10:SingleQuotedParser/0 / lexical/corpus.json:10[10:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a" | "'a'" |

lexical/corpus.json / 10:SingleQuotedParser/1 / lexical/corpus.json:10[10:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"b\""},"span":[0,3],"type":"Value"} | null |

lexical/corpus.json / 14:SingleQuotedParser/0 / lexical/corpus.json:14[14:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"ab"},"span":[0,2],"type":"Value"} | null |

lexical/corpus.json / 16:SingleQuotedParser/0 / lexical/corpus.json:16[16:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "'a''b'" |
| /ast/fields/value/1 | "b" | 欠落 |

lexical/corpus.json / 17:SingleQuotedParser/0 / lexical/corpus.json:17[17:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/fields/text | "a" | "'a'" |

lexical/corpus.json / 18:SingleQuotedParser/0 / lexical/corpus.json:18[18:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | " " | "' '" |

lexical/corpus.json / 18:SingleQuotedParser/1 / lexical/corpus.json:18[18:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\\\\\'x" | "'\\\\\\'x'" |

lexical/corpus.json / 2:IdentifierParser/0 / lexical/corpus.json:2[2:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a b _c9" |
| /ast/fields/value/1 | "b" | 欠落 |
| /ast/fields/value/2 | "_c9" | 欠落 |

lexical/corpus.json / 2:IdentifierParser/1 / lexical/corpus.json:2[2:IdentifierParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a /*c*/b" |
| /ast/fields/value/1 | "b" | 欠落 |

lexical/corpus.json / 4:SingleQuotedParser/0 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "" | "''" |

lexical/corpus.json / 4:SingleQuotedParser/1 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "abc" | "'abc'" |

lexical/corpus.json / 4:SingleQuotedParser/10 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a\\'b" | "'a\\'b'" |

lexical/corpus.json / 4:SingleQuotedParser/11 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\\\" | "'\\\\'" |

lexical/corpus.json / 4:SingleQuotedParser/12 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\\n" | "'\\\n'" |

lexical/corpus.json / 4:SingleQuotedParser/13 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\"" | "'\"'" |

lexical/corpus.json / 4:SingleQuotedParser/2 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "😀あ" | "'😀あ'" |

lexical/corpus.json / 4:SingleQuotedParser/3 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a\nb" | "'a\nb'" |

lexical/corpus.json / 4:SingleQuotedParser/4 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a\r\nb" | "'a\r\nb'" |

lexical/corpus.json / 4:SingleQuotedParser/5 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\u0000" | "'\u0000'" |

lexical/corpus.json / 4:SingleQuotedParser/6 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\q" | "'\\q'" |

lexical/corpus.json / 4:SingleQuotedParser/7 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\n" | "'\\n'" |

lexical/corpus.json / 4:SingleQuotedParser/8 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\u0041" | "'\\u0041'" |

lexical/corpus.json / 4:SingleQuotedParser/9 / lexical/corpus.json:4[4:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "\\😀" | "'\\😀'" |

lexical/corpus.json / 5:DoubleQuotedParser/0 / lexical/corpus.json:5[5:DoubleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"\""},"span":[0,2],"type":"Value"} | null |

lexical/corpus.json / 5:DoubleQuotedParser/1 / lexical/corpus.json:5[5:DoubleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"😀\""},"span":[0,3],"type":"Value"} | null |

lexical/corpus.json / 5:DoubleQuotedParser/2 / lexical/corpus.json:5[5:DoubleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"a\\\"b\""},"span":[0,6],"type":"Value"} | null |

lexical/corpus.json / 5:DoubleQuotedParser/3 / lexical/corpus.json:5[5:DoubleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"\\q\""},"span":[0,4],"type":"Value"} | null |

lexical/corpus.json / 5:DoubleQuotedParser/4 / lexical/corpus.json:5[5:DoubleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"\\😀\""},"span":[0,4],"type":"Value"} | null |

lexical/corpus.json / 5:DoubleQuotedParser/5 / lexical/corpus.json:5[5:DoubleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"a\nb\""},"span":[0,5],"type":"Value"} | null |

lexical/corpus.json / 6:org.unlaxer.parser.elementary.SingleQuotedParser/0 / lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a /*inside*/" | "'a /*inside*/'" |

lexical/corpus.json / 7:org.unlaxer.parser.elementary.DoubleQuotedParser/0 / lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\" a \""},"span":[0,13],"type":"Value"} | null |

lexical/corpus.json / 8:SingleQuotedParser/0 / lexical/corpus.json:8[8:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "'a' /*c*/ 'b'" |
| /ast/fields/value/1 | "b" | 欠落 |

lexical/corpus.json / 8:SingleQuotedParser/1 / lexical/corpus.json:8[8:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "none" | "none 'b'" |
| /ast/fields/value/1 | "b" | 欠落 |

lexical/corpus.json / 9:DoubleQuotedParser/1 / lexical/corpus.json:9[9:DoubleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"a\""},"span":[0,4],"type":"Value"} | null |

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

mixed-values/shared-corpus.json / Shared.ubnf/0 / mixed-values/shared-corpus.json:0[first]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | null |

mixed-values/shared-corpus.json / Shared.ubnf/0 / mixed-values/shared-corpus.json:1[second]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | null |

mixed-values/shared-corpus.json / Shared.ubnf/1 / mixed-values/shared-corpus.json:0[first]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

mixed-values/shared-corpus.json / Shared.ubnf/1 / mixed-values/shared-corpus.json:1[second]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

mixed-values/shared-corpus.json / Shared.ubnf/2 / mixed-values/shared-corpus.json:0[first]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:y" | null |

mixed-values/shared-corpus.json / Shared.ubnf/2 / mixed-values/shared-corpus.json:1[second]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:y" | null |

mixed-values/shared-corpus.json / Shared.ubnf/3 / mixed-values/shared-corpus.json:0[first]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

mixed-values/shared-corpus.json / Shared.ubnf/3 / mixed-values/shared-corpus.json:1[second]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

mixed-values/shared-corpus.json / Shared.ubnf/4 / mixed-values/shared-corpus.json:0[first]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀" | null |

mixed-values/shared-corpus.json / Shared.ubnf/4 / mixed-values/shared-corpus.json:1[second]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀" | null |

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

oracle/backref.json / backref/1 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":[],"kind":"scope","length":1,"message":"backreference mismatch: x","offset":0,"severity":"ERROR"} |
| /diagnostics/1 | 欠落 | {"expected":[],"kind":"scope","length":1,"message":"backreference mismatch: x","offset":2,"severity":"ERROR"} |

oracle/backref.json / backref/2 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/kind | "ERROR" | "scope" |
| /diagnostics/0/message | "back-reference mismatch: expected 'x' but got 'y'" | "backreference mismatch: x" |
| /diagnostics/0/offset | 2 | 0 |

oracle/backref.json / backref/4 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":[],"kind":"scope","length":3,"message":"backreference mismatch: foo","offset":0,"severity":"ERROR"} |
| /diagnostics/1 | 欠落 | {"expected":[],"kind":"scope","length":3,"message":"backreference mismatch: foo","offset":4,"severity":"ERROR"} |

oracle/ci.json / ci/2 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,3],"text":"LeT"} | 欠落 |

oracle/digit.json / digit/1 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 7 | "7" |

oracle/empty.json / empty/1 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

oracle/empty.json / empty/3 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

oracle/extern-full.json / extern-full/1 / oracle/extern-full.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"😀"},"span":[0,2],"type":"Value"} | null |

oracle/extern-full.json / extern-full/2 / oracle/extern-full.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/offset | 1 | 0 |

oracle/extern-full.json / extern-full/3 / oracle/extern-full.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |
| /consumed | 2 | 0 |

oracle/extern.json / extern/1 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |
| /consumed | 1 | 0 |

oracle/extern.json / extern/2 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/offset | 1 | 0 |

oracle/extern.json / extern/3 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/offset | 1 | 0 |

oracle/longest.json / longest/3 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"B:0","name":"value","span":[0,2],"text":"ab"} | 欠落 |
| /declarations/0 | {"name":"ab","sourceOffset":0} | 欠落 |

oracle/lookahead-failure.json / lookahead-failure/1 / oracle/lookahead-failure.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/offset | 1 | 0 |

oracle/lookahead-until.json / lookahead-until/1 / oracle/lookahead-until.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

oracle/lookahead.json / lookahead/1 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

oracle/lookahead.json / lookahead/2 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

oracle/negative-lookahead.json / negative-lookahead/2 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

oracle/numeric.json / numeric/1 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 2147483647 | "2147483647" |

oracle/numeric.json / numeric/2 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast | null | {"fields":{"value":"2147483648"},"span":[0,12],"type":"Value"} |
| /mappingError | "java.lang.NumberFormatException: For input string: \"2147483648\"" | null |

oracle/numeric.json / numeric/3 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast | null | {"fields":{"value":"-2147483649"},"span":[0,13],"type":"Value"} |
| /mappingError | "java.lang.NumberFormatException: For input string: \"-2147483649\"" | null |

oracle/numeric.json / numeric/4 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast | null | {"fields":{"value":"1.5"},"span":[0,5],"type":"Value"} |
| /mappingError | "java.lang.NumberFormatException: For input string: \"1.5\"" | null |

oracle/numeric.json / numeric/5 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast | null | {"fields":{"value":"1e2"},"span":[0,5],"type":"Value"} |
| /mappingError | "java.lang.NumberFormatException: For input string: \"1e2\"" | null |

oracle/numeric.json / numeric/6 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 7 | "+0007" |

oracle/numeric.json / numeric/7 / oracle/numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/offset | 4 | 2 |

oracle/posix.json / posix/4 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"A"} | 欠落 |

oracle/predictive-extern.json / predictive-extern/1 / oracle/predictive-extern.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /captures/0 | {"binding":"A:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |
| /consumed | 1 | 0 |

oracle/predictive-nullable.json / predictive-nullable/3 / oracle/predictive-nullable.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/offset | 1 | 0 |

oracle/predictive.json / predictive/5 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":["a","b"],"kind":"syntax","offset":0} |

oracle/repeat-unbounded.json / repeat-unbounded/4 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"values","span":[0,1],"text":"a"} | 欠落 |
| /captures/1 | {"binding":"Root:0","name":"values","span":[1,2],"text":"a"} | 欠落 |
| /captures/2 | {"binding":"Root:0","name":"values","span":[2,3],"text":"a"} | 欠落 |

oracle/separated.json / separated/4 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"values","span":[0,1],"text":"a"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "a" |
| /diagnostics/0/kind | "trailing_input" | "syntax" |

oracle/skip.json / skip/3 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Item:0","name":"value","span":[0,1],"text":"a"} | 欠落 |
| /captures/1 | {"binding":"Root:0","name":"items","span":[0,1],"text":"a"} | 欠落 |
| /diagnostics/0/expected/0 | "end of input" | "a" |

oracle/until.json / until/3 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |

oracle/until.json / until/4 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /captures/0 | {"binding":"Root:0","name":"value","span":[0,0],"text":""} | 欠落 |

primitives/corpus.json / 14:LOOKAHEAD('')/0 / primitives/corpus.json:14[14:LOOKAHEAD('')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | false | true |

primitives/corpus.json / 15:NEGATIVE_LOOKAHEAD('')/0 / primitives/corpus.json:15[15:NEGATIVE_LOOKAHEAD('')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"a"},"span":[0,1],"type":"Value"} | null |

primitives/corpus.json / 25:ANY/0 / primitives/corpus.json:25[25:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | 欠落 | "" |

primitives/corpus.json / 25:ANY/1 / primitives/corpus.json:25[25:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a😀" |
| /ast/fields/value/1 | "😀" | 欠落 |

primitives/corpus.json / 29:LOOKAHEAD('a')/0 / primitives/corpus.json:29[29:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"ab"},"span":[0,2],"type":"Value"} | null |

primitives/corpus.json / 29:LOOKAHEAD('a')/2 / primitives/corpus.json:29[29:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"ab"},"span":[0,8],"type":"Value"} | null |

primitives/corpus.json / 30:LOOKAHEAD('a')/0 / primitives/corpus.json:30[30:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /matched | 1 | 0 |

primitives/corpus.json / 30:LOOKAHEAD('a')/1 / primitives/corpus.json:30[30:LOOKAHEAD('a')]

| 観測 | 期待 | 実測 |
|---|---|---|
| /matched | 2 | 1 |

primitives/corpus.json / 44:ANY/0 / primitives/corpus.json:44[44:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a b" |
| /ast/fields/value/1 | "b" | 欠落 |

primitives/corpus.json / 44:ANY/1 / primitives/corpus.json:44[44:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a /*c*/b" |
| /ast/fields/value/1 | "b" | 欠落 |

primitives/corpus.json / 45:ANY/0 / primitives/corpus.json:45[45:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a b" |
| /ast/fields/value/1 | "b" | 欠落 |

primitives/corpus.json / 45:ANY/1 / primitives/corpus.json:45[45:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a /*c*/b" |
| /ast/fields/value/1 | "b" | 欠落 |

primitives/corpus.json / 46:ANY/0 / primitives/corpus.json:46[46:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a /*c*/" | "a /*c*/b" |
| /ast/fields/value/1 | "b" | 欠落 |

primitives/corpus.json / 47:ANY/0 / primitives/corpus.json:47[47:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a /*c*/a" |
| /ast/fields/value/1 | "a" | 欠落 |

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
| /scope/diagnostics/0/message | "未定義のシンボル: 'missing'" | "undefined: missing" |

scope-annotations/corpus.json / quantified-capture-occurrences/plus-bounded-and-separated-inner-captures / scope-annotations/corpus.json:6[quantified-capture-occurrences]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/declarations/0/sourceOffset | 0 | 1 |
| /scope/declarations/1/name | "a" | "b" |
| /scope/declarations/1/sourceOffset | 1 | 4 |

scope-annotations/corpus.json / repeated-capture-occurrences/all-repeat-captures-produce-effects / scope-annotations/corpus.json:4[repeated-capture-occurrences]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0/message | "未定義のシンボル: 'missing'" | "undefined: missing" |
| /scope/references/1/length | 1 | 7 |
| /scope/references/1/name | "b" | "missing" |

scope-annotations/corpus.json / scope-and-declares-parent-registration/valid-prefix-with-trailing-input / scope-annotations/corpus.json:2[scope-and-declares-parent-registration]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/declarations/0 | {"name":"kept","sourceOffset":1} | 欠落 |
| /scope/references/0 | {"length":4,"name":"kept","offset":6} | 欠落 |

scope-annotations/corpus.json / trimmed-group-non-bmp/ascii-trim-adjusts-code-point-offset-and-length / scope-annotations/corpus.json:8[trimmed-group-non-bmp]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/declarations/0/sourceOffset | 3 | 1 |
| /scope/references/0/length | 2 | 4 |
| /scope/references/0/offset | 10 | 9 |

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

tiny-code-fence/corpus.json / combined-fence/body-edge-whitespace-trimmed / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 24 | 0 |

tiny-code-fence/corpus.json / combined-fence/code-looking-body-is-text / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 32 | 0 |

tiny-code-fence/corpus.json / combined-fence/cr-lines / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 21 | 0 |

tiny-code-fence/corpus.json / combined-fence/crlf-lines / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 24 | 0 |

tiny-code-fence/corpus.json / combined-fence/double-backticks-in-body / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 20 | 0 |

tiny-code-fence/corpus.json / combined-fence/empty-body / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 15 | 0 |

tiny-code-fence/corpus.json / combined-fence/lf-body / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 22 | 0 |

tiny-code-fence/corpus.json / combined-fence/trailing-nul-before-eol-preserved / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 21 | 0 |

tiny-code-fence/corpus.json / combined-fence/unicode-internal-nul-body / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 19 | 0 |

tiny-code-fence/corpus.json / end-direct/cr / tiny-code-fence/corpus.json:1[end-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 4 | 0 |

tiny-code-fence/corpus.json / end-direct/crlf / tiny-code-fence/corpus.json:1[end-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 5 | 0 |

tiny-code-fence/corpus.json / end-direct/eof / tiny-code-fence/corpus.json:1[end-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 3 | 0 |

tiny-code-fence/corpus.json / end-direct/lf / tiny-code-fence/corpus.json:1[end-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 4 | 0 |

tiny-code-fence/corpus.json / end-source-offset/after-crlf / tiny-code-fence/corpus.json:5[end-source-offset]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 7 | 0 |

tiny-code-fence/corpus.json / end-source-offset/after-lf / tiny-code-fence/corpus.json:5[end-source-offset]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 6 | 0 |

tiny-code-fence/corpus.json / java-style-outer-only/leading-comment-ending-at-sol / tiny-code-fence/corpus.json:6[java-style-outer-only]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 27 | 0 |

tiny-code-fence/corpus.json / optional-start/present / tiny-code-fence/corpus.json:9[optional-start]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 13 | 0 |

tiny-code-fence/corpus.json / p4-zero-field-java-style/closed / tiny-code-fence/corpus.json:7[p4-zero-field-java-style]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 21 | 0 |

tiny-code-fence/corpus.json / p4-zero-field-java-style/leading-body-trivia / tiny-code-fence/corpus.json:7[p4-zero-field-java-style]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 34 | 0 |

tiny-code-fence/corpus.json / p4-zero-field-java-style/triple-inside-skipped-comment / tiny-code-fence/corpus.json:7[p4-zero-field-java-style]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 26 | 0 |

tiny-code-fence/corpus.json / prefix-vs-full-input-end/full / tiny-code-fence/corpus.json:3[prefix-vs-full-input-end]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 4 | 0 |

tiny-code-fence/corpus.json / prefix-vs-full-input-end/valid-prefix / tiny-code-fence/corpus.json:3[prefix-vs-full-input-end]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /consumed | 4 | 0 |
| /matched | 4 | 0 |

tiny-code-fence/corpus.json / prefix-vs-full-input-start/full / tiny-code-fence/corpus.json:2[prefix-vs-full-input-start]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 12 | 0 |

tiny-code-fence/corpus.json / prefix-vs-full-input-start/valid-prefix / tiny-code-fence/corpus.json:2[prefix-vs-full-input-start]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /consumed | 12 | 0 |
| /matched | 12 | 0 |

tiny-code-fence/corpus.json / repeat-start/two / tiny-code-fence/corpus.json:10[repeat-start]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 21 | 0 |

tiny-code-fence/corpus.json / start-direct/ascii-identifiers / tiny-code-fence/corpus.json:0[start-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 15 | 0 |

tiny-code-fence/corpus.json / start-direct/cr / tiny-code-fence/corpus.json:0[start-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 12 | 0 |

tiny-code-fence/corpus.json / start-direct/crlf / tiny-code-fence/corpus.json:0[start-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 13 | 0 |

tiny-code-fence/corpus.json / start-direct/eof / tiny-code-fence/corpus.json:0[start-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 11 | 0 |

tiny-code-fence/corpus.json / start-direct/lf / tiny-code-fence/corpus.json:0[start-direct]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 15 | 0 |

tiny-code-fence/corpus.json / start-source-offset-after-crlf/after-crlf / tiny-code-fence/corpus.json:4[start-source-offset-after-crlf]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 14 | 0 |

tiny-code-fence/corpus.json / start-source-offset-after-crlf/after-lf / tiny-code-fence/corpus.json:4[start-source-offset-after-crlf]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 14 | 0 |

tiny-code-fence/corpus.json / transparent-aliases/empty-body / tiny-code-fence/corpus.json:8[transparent-aliases]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 15 | 0 |

tiny-code-fence/corpus.json / transparent-aliases/text-body / tiny-code-fence/corpus.json:8[transparent-aliases]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /consumed | 22 | 0 |

tiny-string-token/corpus.json / comments-and-eof/block-and-line / tiny-string-token/corpus.json:2[comments-and-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"x"},"span":[0,20],"type":"Value"} | null |

tiny-string-token/corpus.json / comments-and-eof/comment-is-raw-inside / tiny-string-token/corpus.json:2[comments-and-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"/*inside*/"},"span":[0,12],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/empty-double / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"\""},"span":[0,2],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/empty-single / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":""},"span":[0,2],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/escaped-backslash / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\\\\"},"span":[0,4],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/escaped-double / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"a\\\"b\""},"span":[0,6],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/escaped-newline / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\\\n"},"span":[0,4],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/escaped-non-bmp / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\\😀"},"span":[0,4],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/escaped-single / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"a\\'b"},"span":[0,6],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/non-bmp / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"😀"},"span":[0,3],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/nul / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\u0000"},"span":[0,3],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/plain-double / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"double\""},"span":[0,8],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/plain-single / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"single"},"span":[0,8],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/raw-crlf / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"a\r\nb"},"span":[0,6],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/raw-newline / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"a\nb"},"span":[0,5],"type":"Value"} | null |

tiny-string-token/corpus.json / direct-eof/unknown-escape / tiny-string-token/corpus.json:0[direct-eof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\\q"},"span":[0,4],"type":"Value"} | null |

tiny-string-token/corpus.json / mapped-node/double-node / tiny-string-token/corpus.json:6[mapped-node]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":{"fields":{"text":"\"😀\""},"span":[0,3],"type":"Item"}},"span":[0,3],"type":"Value"} | null |

tiny-string-token/corpus.json / mapped-node/single-node / tiny-string-token/corpus.json:6[mapped-node]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":{"fields":{"text":"node"},"span":[0,6],"type":"Item"}},"span":[0,6],"type":"Value"} | null |

tiny-string-token/corpus.json / optional/double / tiny-string-token/corpus.json:4[optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"x\""},"span":[0,4],"type":"Value"} | null |

tiny-string-token/corpus.json / optional/single / tiny-string-token/corpus.json:4[optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"x"},"span":[0,4],"type":"Value"} | null |

tiny-string-token/corpus.json / prefix-vs-full-input/double-prefix / tiny-string-token/corpus.json:1[prefix-vs-full-input]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /consumed | 4 | 0 |
| /matched | 4 | 0 |

tiny-string-token/corpus.json / prefix-vs-full-input/full / tiny-string-token/corpus.json:1[prefix-vs-full-input]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"ok"},"span":[0,4],"type":"Value"} | null |

tiny-string-token/corpus.json / prefix-vs-full-input/single-prefix / tiny-string-token/corpus.json:1[prefix-vs-full-input]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /consumed | 4 | 0 |
| /matched | 4 | 0 |

tiny-string-token/corpus.json / repeat/non-bmp-repeat / tiny-string-token/corpus.json:5[repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":["😀","\"🚀\""]},"span":[0,6],"type":"Value"} | null |

tiny-string-token/corpus.json / repeat/two-kinds / tiny-string-token/corpus.json:5[repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":["a","\"b\""]},"span":[0,6],"type":"Value"} | null |

tiny-string-token/corpus.json / transparent-alias/double / tiny-string-token/corpus.json:3[transparent-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"\"alias\""},"span":[0,7],"type":"Value"} | null |

tiny-string-token/corpus.json / transparent-alias/single / tiny-string-token/corpus.json:3[transparent-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"alias"},"span":[0,7],"type":"Value"} | null |

未観測項目（展開後件数。欠落を Pass に読み替えない）

| 観測 | 件数 |
|---|---:|
| /captures/0/binding | 32 |
| /captures/1/binding | 7 |
| /captures/2/binding | 1 |
| /captures/3/binding | 1 |
| /diagnosticError | 1 |
| /diagnostics/0/farthestExpected | 63 |
| /diagnostics/0/farthestOffset | 63 |
| /diagnostics/0/origin | 64 |
| /enumError | 1 |
| /enumValue | 2 |
| /expectation | 148 |
| /grammar (操作列の driver protocol 未対応) | 9 |
| /nativeFailure | 114 |
| /nodes | 14 |
| /operations | 9 |
| /parseError | 1 |
| /parsedStatus | 113 |
| /rollback | 2 |
| /scope/depth | 14 |
| /scope/resolved | 14 |
| /scopeDepth | 114 |
| /texts | 79 |
| /valueSpans | 10 |

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

918c6ebf78ed62a4-169 / generation: catalog/corpus.json:0[deduplicated-sorted-captures]

> 終了 exit status: 1:
> unsupported IR feature catalog: not implemented in this backend

7be2e8f96b1fbd8f-170 / generation: catalog/corpus.json:1[rule-order-locality-quantifiers-and-escaped-context]

> 終了 exit status: 1:
> unsupported IR feature catalog: not implemented in this backend

58d4a55d5fd2a562-172 / generation: oracle/catalog.json:0[@catalog]

> 終了 exit status: 1:
> unsupported IR feature catalog: not implemented in this backend

ff739bfe1e293efc-174 / generation: oracle/ci-unicode.json:0[CI]

> 終了 exit status: 1:
> unsupported IR feature caseInsensitive.unicode: Probe::T

668cf0ed36645997-179 / ir: oracle/common-field-missing.json:0[@commonField]

> 終了 exit status: 1:
> ubnfc: E-LOWER-COMMON-FIELD: ProbeAST.Node.name: variants の field 型/cardinality が不整合

d8ba3c8d7194ffb2-180 / ir: oracle/common-field-type.json:0[@commonField]

> 終了 exit status: 1:
> ubnfc: E-LOWER-COMMON-FIELD: ProbeAST.Node.name: variants の field 型/cardinality が不整合

0489ef2c71ddce15-187 / generation: oracle/enum-collision.json:0[@enum]

> 終了 exit status: 1:
> invalid Grammar IR: duplicate enum name/literal: ProbeAST.Mode

1ae620c0ffbb124d-191 / generation: oracle/error.json:0[ERROR]

> 終了 exit status: 1:
> unsupported IR feature execution.transaction: expr:__runall__.ubnf:86:101:body/1/error: Once for Error { message: "need a" }

320ab6b6ec2cc8a9-192 / generation: oracle/eval-binary_arithmetic.json:0[@eval]

> 終了 exit status: 1:
> unsupported IR feature eval: not implemented in this backend

000a3a5238bb6cc9-193 / generation: oracle/eval-conditional.json:0[@eval]

> 終了 exit status: 1:
> unsupported IR feature eval: not implemented in this backend

bf0a74f72822971b-194 / generation: oracle/eval-literal.json:0[@eval]

> 終了 exit status: 1:
> unsupported IR feature eval: not implemented in this backend

89160f03adabb722-195 / generation: oracle/eval-manual.json:0[@eval]

> 終了 exit status: 1:
> unsupported IR feature eval: not implemented in this backend

ec991e1179775a0a-196 / generation: oracle/eval-passthrough.json:0[@eval]

> 終了 exit status: 1:
> unsupported IR feature eval: not implemented in this backend

4aaea9c4ba5b21d8-197 / ir: oracle/eval-sum.json:0[@eval]

> 終了 exit status: 1:
> ubnfc: E-LOWER-SUM-EVAL: Node: 自動 @eval(literal) は具象型が必要

92fd086142e9daaa-198 / generation: oracle/eval-unknown.json:0[@eval]

> 終了 exit status: 1:
> unsupported IR feature eval: not implemented in this backend

4e2875aab6b5ab17-199 / generation: oracle/eval-variable_ref.json:0[@eval]

> 終了 exit status: 1:
> unsupported IR feature eval: not implemented in this backend

3f95b76dccbb0235-203 / ir: oracle/import-cycle.json:0[@import]

> 終了 exit status: 1:
> ubnfc: 循環 import を検出しました: &lt;build&gt;/grammar/B.ubnf (スタック: [&lt;build&gt;/grammar/__runall__.ubnf, &lt;build&gt;/grammar/B.ubnf, &lt;build&gt;/grammar/C.ubnf]) [code: E-IMPORT-CYCLE] [hint: Remove the circular import.]

6158dbda03072d94-219 / generation: oracle/recovery-auto.json:0[@recovery]

> 終了 exit status: 1:
> unsupported IR feature recovery: not implemented in this backend

fcabbee7dbead518-220 / generation: oracle/recovery-root.json:0[@recovery]

> 終了 exit status: 1:
> unsupported IR feature recovery: not implemented in this backend

6579df10379cb598-221 / generation: oracle/recovery-skip.json:0[@recovery]

> 終了 exit status: 1:
> unsupported IR feature recovery: not implemented in this backend

ec619df4c9ba410e-222 / generation: oracle/recovery-sync.json:0[@recovery]

> 終了 exit status: 1:
> unsupported IR feature recovery: not implemented in this backend

c30e88edec38b2bc-223 / generation: oracle/regex-ascii.json:0[REGEX]

> 終了 exit status: 1:
> unsupported IR feature regex: Probe::T requires an analyzed DFA; Java Pattern fallback is unavailable

294be563a200330d-224 / generation: oracle/regex-backref.json:0[REGEX]

> 終了 exit status: 1:
> unsupported IR feature regex: Probe::T requires an analyzed DFA; Java Pattern fallback is unavailable

9d4bad6e118b51d6-225 / generation: oracle/regex-emoji.json:0[REGEX]

> 終了 exit status: 1:
> unsupported IR feature regex: Probe::T requires an analyzed DFA; Java Pattern fallback is unavailable

7747bb1a21a1d975-226 / generation: oracle/regex-prefix.json:0[REGEX]

> 終了 exit status: 1:
> unsupported IR feature regex: Probe::T requires an analyzed DFA; Java Pattern fallback is unavailable

ee08200d6b7a5188-227 / generation: oracle/regex-zero.json:0[REGEX]

> 終了 exit status: 1:
> unsupported IR feature regex: Probe::T requires an analyzed DFA; Java Pattern fallback is unavailable

0f54bfb5d29e92fd-232 / generation: oracle/typeof.json:0[@typeof]

> 終了 exit status: 1:
> unsupported IR feature typeof: not implemented in this backend

初回測定の再現情報

生成器・front・corpus loader は作業開始時の master `6040084a6dd894dc3a8bd0fbd8dbd3c79f7b03aa`。並行作業中の Rust 生成器 / oracle loader の変更は含めない。worktree 内で `cargo build -q --offline --bin ubnfc-rust --bin ubnfc` と `cargo build -q --offline --bin ubnfc-harness` を実行し、同 worktree のバイナリで測定した。SHA-256 は冒頭に記載。

```sh
ubnfc-harness run-all --backend rust --report docs/reports/2026-09-22-rust-corpus.md --json docs/reports/2026-09-22-rust-corpus.json
```

Cargo の並列度は既定設定。`--jobs N` 指定時は Cargo に渡す。189 crate を1回でビルドし、build 失敗・実行エラーは0。実装検証は harness 全62テスト、clippy（all-targets / -D warnings）、fmt が成功。associative は24 Pass、反転 metadata variant は既存の IR 拒否24件を保持。失敗 crate の個別切り分け・正常 driver の実行・hash キャッシュは別の故障注入テストで確認。

全 corpus のコマンド終了コードは1（段階失敗または Fail が478件）。期待値・乖離登録は変更していない。数値比較は rust_lexeme、型付き診断は rustOffset を使用する既存の共通比較処理による。

Java after-triage との系統別比較

比較元: [2026-09-22-java-corpus-after-triage.md](2026-09-22-java-corpus-after-triage.md) / [JSON](2026-09-22-java-corpus-after-triage.json)。双方とも83ファイル、871元ケース、1195実行。各セルは **Rust件数（Rust − Java）**。差が0の場合は括弧を省略する。Java の javac 失敗と Rust の build 失敗を同じ列で比較する。

Pass は68件増、生成拒否は143件減、Fail は86件増、Known は1件減、未観測は10件減。実行可能になったケースが増えたため、Fail の増加だけでは回帰と判定しない。

| 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 24 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 50 | 0 | 20 | 0 |
| catalog | 0 | 2 (+2) | 0 | 0 | 0 | 0 | 0 | 0 (-2) |
| evolution | 0 | 0 | 0 | 0 | 4 (-2) | 0 | 4 (+2) | 148 |
| lexical | 0 | 0 (-16) | 0 | 0 | 44 (+5) | 0 | 34 (+11) | 0 |
| mixed-values | 0 | 0 (-22) | 0 | 0 | 49 (+12) | 0 | 49 (+10) | 0 |
| nested-captures | 0 | 0 (-1) | 0 | 0 | 0 | 0 | 5 | 5 (+1) |
| numeric-capture | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 |
| oracle | 3 | 41 (+6) | 0 | 0 | 1 | 10 (-1) | 35 (+1) | 78 (-6) |
| primitives | 0 | 0 | 0 | 0 | 95 (+2) | 0 | 14 (-2) | 0 |
| right-associative | 0 | 0 | 0 | 0 | 63 | 0 | 49 | 0 |
| rule-trivia | 0 | 0 | 0 | 0 | 32 | 0 | 1 | 0 |
| scope-annotations | 0 | 0 | 0 | 0 | 0 | 0 | 5 (+3) | 9 (-3) |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 101 | 0 | 0 | 0 | 32 | 0 | 30 | 0 |
| tiny-code-fence | 0 | 0 (-73) | 0 | 0 | 39 (+39) | 0 | 34 (+34) | 0 |
| tiny-string-token | 0 | 0 (-39) | 0 | 0 | 12 (+12) | 0 | 27 (+27) | 0 |
| 合計 | 128 | 43 (-143) | 0 | 0 | 458 (+68) | 10 (-1) | 307 (+86) | 249 (-10) |


6 loader 系統の比較（同じ集計単位）

| loader 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 24 | 0 | 0 | 0 | 154 (-2) | 0 | 73 (+2) | 148 |
| inline-ast | 0 | 0 (-1) | 0 | 0 | 32 | 0 | 11 (+3) | 14 (-2) |
| operations | 0 | 2 (+2) | 0 | 0 | 0 | 0 | 0 | 9 (-2) |
| oracle | 3 | 41 (+6) | 0 | 0 | 1 | 10 (-1) | 35 (+1) | 78 (-6) |
| rule | 101 | 0 (-134) | 0 | 0 | 132 (+63) | 0 | 140 (+71) | 0 |
| token | 0 | 0 (-16) | 0 | 0 | 139 (+7) | 0 | 48 (+9) | 0 |
| 合計 | 128 | 43 (-143) | 0 | 0 | 458 (+68) | 10 (-1) | 307 (+86) | 249 (-10) |
