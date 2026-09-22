Rust corpus run-all

元 corpus: 127 ファイル、1212 ケース。文法 variant: 279、hash キャッシュ: 234 build、展開後: 1425 件（文法なし操作列を含む）。jobs=cargo（既定）。

集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。

Rust generator SHA-256: `c0de6518f9c3a8b58390304ff6d810f3c9ba95b7252a5b3507a8df808c97ca2a`

front SHA-256: `8b29dda85eb2c481d7a6ce0f7a9704ed64d315bb1396ed745d9e217d59a2ab14`

Rust 一括 cargo build --release --offline --workspace: 112.539 秒。個別切り分け込み: 112.539 秒、cargo 起動 1 回。生成 228 crate、生成物 hash 重複除去後 227 crate、既存 cache hit 0 件。

JSON の build_failure / builds[].cargo は Rust、javac_failure / builds[].javac は Java の build 段階。表の build 失敗は両者の合計。比較規則は共通で、診断 offset と数値字句は backend ごとの既存 oracle を使う。

期待拒否 Pass: 28件（Verdict は Pass）。通常の Pass・IR 失敗とは別列に排他的に計上する。build の実測失敗と診断は拒否理由表に保持する。

| 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 0 | 0 | 0 | 0 | 24 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 50 | 0 | 0 | 20 | 0 |
| catalog | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| evolution | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 130 | 20 |
| lexical | 0 | 0 | 0 | 0 | 53 | 0 | 0 | 25 | 0 |
| mixed-values | 0 | 0 | 0 | 0 | 49 | 0 | 0 | 49 | 0 |
| nested-captures | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 5 |
| numeric-capture | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 | 0 |
| oracle | 0 | 2 | 0 | 0 | 27 | 4 | 18 | 236 | 111 |
| primitives | 0 | 0 | 0 | 0 | 101 | 0 | 0 | 8 | 0 |
| right-associative | 0 | 0 | 0 | 0 | 63 | 0 | 0 | 49 | 0 |
| rule-trivia | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 0 |
| scope-annotations | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 11 |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 0 | 0 | 0 | 0 | 69 | 0 | 0 | 94 | 0 |
| tiny-code-fence | 0 | 0 | 0 | 0 | 72 | 0 | 0 | 1 | 0 |
| tiny-string-token | 0 | 0 | 0 | 0 | 20 | 0 | 0 | 19 | 0 |
| 合計 | 0 | 2 | 0 | 0 | 582 | 28 | 18 | 639 | 156 |

6 系統別（case × variant 単位）

| loader 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 0 | 0 | 0 | 0 | 156 | 24 | 0 | 199 | 20 |
| inline-ast | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 8 | 16 |
| operations | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 9 |
| oracle | 0 | 2 | 0 | 0 | 27 | 4 | 18 | 236 | 111 |
| rule | 0 | 0 | 0 | 0 | 210 | 0 | 0 | 163 | 0 |
| token | 0 | 0 | 0 | 0 | 154 | 0 | 0 | 33 | 0 |
| 合計 | 0 | 2 | 0 | 0 | 582 | 28 | 18 | 639 | 156 |

文法別（case × variant 単位。文法なしは操作列）

| 文法 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| associative/corpus.json:0[normal] | 0 | 0 | 0 | 0 | 24 | 0 | 0 | 0 | 0 |
| associative/corpus.json:1[reversed-metadata] | 0 | 0 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| cardinality/corpus.json:0[zero] | 0 | 0 | 0 | 0 | 8 | 0 | 0 | 6 | 0 |
| cardinality/corpus.json:1[plus] | 0 | 0 | 0 | 0 | 10 | 0 | 0 | 4 | 0 |
| cardinality/corpus.json:2[bounded] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 3 | 0 |
| cardinality/corpus.json:3[unbounded] | 0 | 0 | 0 | 0 | 10 | 0 | 0 | 4 | 0 |
| cardinality/corpus.json:4[separated] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 3 | 0 |
| catalog/corpus.json:0[deduplicated-sorted-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| catalog/corpus.json:1[rule-order-locality-quantifiers-and-escaped-context] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| evolution/conformance.json:0[0] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 32 | 5 |
| evolution/conformance.json:1[1] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 32 | 5 |
| evolution/conformance.json:2[2] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 32 | 5 |
| evolution/conformance.json:3[3] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 32 | 5 |
| evolution/diagnostics.json:0[3] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:0[0:IdentifierParser] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:10[10:SingleQuotedParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:11[11:EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:12[12:org.unlaxer.parser.elementary.EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:13[13:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:14[14:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:15[15:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:16[16:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:17[17:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:18[18:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:19[19:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:1[1:org.unlaxer.parser.clang.IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:20[20:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:21[21:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:2[2:IdentifierParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:3[3:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:4[4:SingleQuotedParser] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 14 | 0 |
| lexical/corpus.json:5[5:DoubleQuotedParser] | 0 | 0 | 0 | 0 | 9 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:8[8:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:9[9:DoubleQuotedParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| mixed-values/corpus.json:0[two] | 0 | 0 | 0 | 0 | 10 | 0 | 0 | 13 | 0 |
| mixed-values/corpus.json:1[one] | 0 | 0 | 0 | 0 | 15 | 0 | 0 | 8 | 0 |
| mixed-values/delimited-corpus.json:0[two] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 9 | 0 |
| mixed-values/delimited-corpus.json:1[one] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 9 | 0 |
| mixed-values/shared-corpus.json:0[first] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 5 | 0 |
| mixed-values/shared-corpus.json:1[second] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 5 | 0 |
| nested-captures/corpus.json:0[nested-text-completion] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| nested-captures/corpus.json:1[choice-optional-coexist] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| nested-captures/corpus.json:2[quantifier-items-not-containers] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:3[same-wrapper-and-parser-class] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| nested-captures/corpus.json:4[text-node-and-mixed-values] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| nested-captures/corpus.json:5[recursive-child-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:6[shared-mapping-nested-and-sequential] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| nested-captures/corpus.json:7[typed-node-list-and-shared-mixed-schema] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| numeric-capture/conformance.json:0[Numeric] | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 | 0 |
| oracle/backref.json:0[@backref-no-scope] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/ci.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| oracle/comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1 |
| oracle/comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 1 |
| oracle/comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 2 |
| oracle/common-field-missing.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/common-field-type.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/conformance-cardinality-bounded.json:0[cardinality] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-plus.json:0[cardinality] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-separated.json:0[cardinality] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-unbounded.json:0[cardinality] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| oracle/conformance-cardinality-zero.json:0[cardinality] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/conformance-catalog-deduplicated-sorted-captures.json:0[catalog] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-catalog-rule-order-locality-quantifiers-and-escaped-context.json:0[catalog] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/conformance-mixed-values-one.json:0[mixed-values] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 |
| oracle/conformance-nested-captures-nested-text-completion.json:0[nested-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-nested-captures-same-wrapper-and-parser-class.json:0[nested-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-nested-captures-text-node-and-mixed-values.json:0[nested-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/conformance-scope-annotations-capture-site-identity.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-annotations-failed-choice-rollback.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-annotations-nested-lexical-dynamic.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-annotations-nested-same-name-captures.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-annotations-scope-and-declares-parent-registration.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-annotations-trimmed-group-non-bmp.json:0[scope-annotations] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/conformance-scope-store.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| oracle/digit.json:0[DigitParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/empty-one-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty.json:0[EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/enum-collision.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| oracle/enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/enum.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/eof.json:0[EOF] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/error.json:0[ERROR] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/eval-binary_arithmetic.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-conditional.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-literal.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-manual.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-passthrough.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-sum.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/eval-unknown.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-variable_ref.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/evolution-v0.json:0[evolution] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 32 | 5 |
| oracle/evolution-v1.json:0[evolution] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 32 | 5 |
| oracle/evolution-v2.json:0[evolution] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 32 | 5 |
| oracle/evolution-v3.json:0[evolution] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 32 | 5 |
| oracle/extern-full.json:0[Extern] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/extern.json:0[Extern] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/import-alias.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/import-cycle.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/import-two-level.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| oracle/lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead-until.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 0 |
| oracle/posix.json:0[posix] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| oracle/predictive-extern.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/recovery-auto.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/recovery-root.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1 |
| oracle/recovery-skip.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| oracle/recovery-sync.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 | 0 |
| oracle/regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/regex-backref.json:0[REGEX] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-emoji.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/regex-prefix.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/regex-zero.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/reobserved-catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/reobserved-char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/reobserved-ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/reobserved-ci.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/reobserved-comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/reobserved-comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/reobserved-comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/reobserved-common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/reobserved-enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/reobserved-longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/reobserved-lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/reobserved-mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/reobserved-mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/reobserved-mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/reobserved-negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| oracle/reobserved-negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/reobserved-numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/reobserved-posix.json:0[posix] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/reobserved-predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/reobserved-predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/reobserved-regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/reobserved-regex-backref.json:0[REGEX] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/repeat-unbounded.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| oracle/repeat-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/separated.json:0[separated] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/skip.json:0[@skip] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| oracle/typeof.json:0[@typeof] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/until.json:0[UNTIL] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 3 |
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
| primitives/corpus.json:25[25:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
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
| primitives/corpus.json:44[44:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| primitives/corpus.json:45[45:ANY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| primitives/corpus.json:46[46:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:47[47:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| primitives/corpus.json:4[4:EMPTY] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:5[5:EOF] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:6[6:LOOKAHEAD('ab')] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:7[7:NEGATIVE_LOOKAHEAD('x')] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:8[8:LOOKAHEAD('😀')] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| primitives/corpus.json:9[9:UNTIL('#')] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 |
| right-associative/corpus.json:0[text] | 0 | 0 | 0 | 0 | 17 | 0 | 0 | 11 | 0 |
| right-associative/corpus.json:1[mapped] | 0 | 0 | 0 | 0 | 17 | 0 | 0 | 11 | 0 |
| right-associative/corpus.json:2[unicode] | 0 | 0 | 0 | 0 | 15 | 0 | 0 | 13 | 0 |
| right-associative/corpus.json:3[grouped] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 14 | 0 |
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
| scope-annotations/corpus.json:0[capture-site-identity] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| scope-annotations/corpus.json:1[nested-lexical-dynamic] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-annotations/corpus.json:2[scope-and-declares-parent-registration] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| scope-annotations/corpus.json:3[failed-choice-rollback] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| scope-annotations/corpus.json:4[repeated-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:5[optional-recursive-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| scope-annotations/corpus.json:6[quantified-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-annotations/corpus.json:7[nested-same-name-captures] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-annotations/corpus.json:8[trimmed-group-non-bmp] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-store/corpus.json:文法なし | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality/corpus.json:0[node-one] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| semantic-cardinality/corpus.json:10[mixed-bounded] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| semantic-cardinality/corpus.json:11[node-separated] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 2 | 0 |
| semantic-cardinality/corpus.json:12[mixed-separated] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:13[optional-many-alias] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 5 | 0 |
| semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 4 | 0 |
| semantic-cardinality/corpus.json:1[mixed-one] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:2[node-pair] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 4 | 0 |
| semantic-cardinality/corpus.json:3[mixed-pair] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 7 | 0 |
| semantic-cardinality/corpus.json:4[mixed-group] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:5[mixed-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:6[node-optional] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| semantic-cardinality/corpus.json:7[mixed-optional] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 5 | 0 |
| semantic-cardinality/corpus.json:8[node-repeat] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 4 | 0 |
| semantic-cardinality/corpus.json:9[mixed-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 6 | 0 |
| semantic-cardinality/pure-alias.json:0[pure-node-one-alias] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 2 | 0 |
| semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| semantic-cardinality/pure-alias.json:1[pure-node-multihop-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 3 | 0 |
| semantic-cardinality/pure-alias.json:2[pure-node-group-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 2 | 0 |
| semantic-cardinality/pure-alias.json:3[pure-node-delimiter-alias] | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 3 | 0 |
| semantic-cardinality/pure-alias.json:4[pure-node-direct-multi] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 3 | 0 |
| semantic-cardinality/pure-alias.json:5[pure-node-multi-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 4 | 0 |
| semantic-cardinality/pure-alias.json:6[pure-node-multi-multihop-group-alias] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 4 | 0 |
| semantic-cardinality/pure-alias.json:7[pure-node-multi-delimiter-alias] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 4 | 0 |
| semantic-cardinality/pure-alias.json:8[pure-node-alias-optional] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| semantic-cardinality/pure-alias.json:9[pure-node-alias-many] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 4 | 0 |
| tiny-code-fence/corpus.json:0[start-direct] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:10[repeat-start] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-code-fence/corpus.json:11[combined-fence] | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 1 | 0 |
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
| 合計 | 0 | 2 | 0 | 0 | 582 | 28 | 18 | 639 | 156 |

拒否理由（M3 作業リスト）

同一文法の拒否はキャッシュ単位に数える。ケース数は理由ごとに重複除去する。生成器が最初の拒否で停止するため、後続の未対応機能を網羅する件数ではない。IR 診断が複数あると理由間で重複する。

| 段階 / 機能・診断コード | build 数 | 元ケース数 | 展開後件数 |
|---|---:|---:|---:|
| generation/regex | 1 | 2 | 2 |
| ir/E-IMPORT-CYCLE | 1 | 1 | 1 |
| ir/E-LOWER-COMMON-FIELD | 2 | 2 | 2 |
| ir/E-LOWER-SUM-EVAL | 1 | 1 | 1 |
| ir/E-PRECEDENCE-ORDER | 1 | 24 | 24 |

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

evolution/conformance.json / evolution/v0/0 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/1 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/10 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v0/11 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

evolution/conformance.json / evolution/v0/12 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

evolution/conformance.json / evolution/v0/13 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/14 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/15 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestExpected/0 | "' '" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/16 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

evolution/conformance.json / evolution/v0/19 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/2 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.0 | null |

evolution/conformance.json / evolution/v0/20 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/21 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/22 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/23 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/24 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/25 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/26 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/27 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/28 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/29 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/3 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 12.0 | null |

evolution/conformance.json / evolution/v0/30 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/31 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

evolution/conformance.json / evolution/v0/33 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

evolution/conformance.json / evolution/v0/36 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 3.0 | null |

evolution/conformance.json / evolution/v0/4 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -3.0 | null |

evolution/conformance.json / evolution/v0/5 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4.0 | null |

evolution/conformance.json / evolution/v0/6 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.5 | null |

evolution/conformance.json / evolution/v0/7 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

evolution/conformance.json / evolution/v0/8 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 150.0 | null |

evolution/conformance.json / evolution/v0/9 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.02 | null |

evolution/conformance.json / evolution/v1/0 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/1 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/10 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

evolution/conformance.json / evolution/v1/11 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

evolution/conformance.json / evolution/v1/12 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

evolution/conformance.json / evolution/v1/13 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 6.0 | null |

evolution/conformance.json / evolution/v1/14 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

evolution/conformance.json / evolution/v1/15 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestExpected/0 | "' '" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/16 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

evolution/conformance.json / evolution/v1/19 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/2 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.0 | null |

evolution/conformance.json / evolution/v1/20 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/21 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/22 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/23 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/24 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/25 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/26 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/27 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/28 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/29 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

evolution/conformance.json / evolution/v1/3 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 12.0 | null |

evolution/conformance.json / evolution/v1/30 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/31 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

evolution/conformance.json / evolution/v1/33 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

evolution/conformance.json / evolution/v1/36 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 3.0 | null |

evolution/conformance.json / evolution/v1/4 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -3.0 | null |

evolution/conformance.json / evolution/v1/5 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4.0 | null |

evolution/conformance.json / evolution/v1/6 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.5 | null |

evolution/conformance.json / evolution/v1/7 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

evolution/conformance.json / evolution/v1/8 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 150.0 | null |

evolution/conformance.json / evolution/v1/9 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.02 | null |

evolution/conformance.json / evolution/v2/0 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

evolution/conformance.json / evolution/v2/1 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "NUMBER" |

evolution/conformance.json / evolution/v2/10 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

evolution/conformance.json / evolution/v2/11 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

evolution/conformance.json / evolution/v2/12 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

evolution/conformance.json / evolution/v2/13 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 6.0 | null |

evolution/conformance.json / evolution/v2/14 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

evolution/conformance.json / evolution/v2/15 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Negation" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v2/16 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

evolution/conformance.json / evolution/v2/19 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -6.0 | null |

evolution/conformance.json / evolution/v2/2 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.0 | null |

evolution/conformance.json / evolution/v2/20 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -5.0 | null |

evolution/conformance.json / evolution/v2/21 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -2.0 | null |

evolution/conformance.json / evolution/v2/22 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2.0 | null |

evolution/conformance.json / evolution/v2/23 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "')'" | "*" |

evolution/conformance.json / evolution/v2/24 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

evolution/conformance.json / evolution/v2/25 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

evolution/conformance.json / evolution/v2/26 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

evolution/conformance.json / evolution/v2/27 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

evolution/conformance.json / evolution/v2/28 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

evolution/conformance.json / evolution/v2/29 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

evolution/conformance.json / evolution/v2/3 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 12.0 | null |

evolution/conformance.json / evolution/v2/30 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

evolution/conformance.json / evolution/v2/31 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

evolution/conformance.json / evolution/v2/33 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

evolution/conformance.json / evolution/v2/36 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 3.0 | null |

evolution/conformance.json / evolution/v2/4 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -3.0 | null |

evolution/conformance.json / evolution/v2/5 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4.0 | null |

evolution/conformance.json / evolution/v2/6 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.5 | null |

evolution/conformance.json / evolution/v2/7 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

evolution/conformance.json / evolution/v2/8 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 150.0 | null |

evolution/conformance.json / evolution/v2/9 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.02 | null |

evolution/conformance.json / evolution/v3/0 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'('" | "neg" |

evolution/conformance.json / evolution/v3/1 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "neg" |

evolution/conformance.json / evolution/v3/10 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

evolution/conformance.json / evolution/v3/11 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

evolution/conformance.json / evolution/v3/12 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

evolution/conformance.json / evolution/v3/13 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 6.0 | null |

evolution/conformance.json / evolution/v3/14 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

evolution/conformance.json / evolution/v3/15 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Conditional" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "neg" |

evolution/conformance.json / evolution/v3/16 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

evolution/conformance.json / evolution/v3/19 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -6.0 | null |

evolution/conformance.json / evolution/v3/2 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.0 | null |

evolution/conformance.json / evolution/v3/20 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -5.0 | null |

evolution/conformance.json / evolution/v3/21 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -2.0 | null |

evolution/conformance.json / evolution/v3/22 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2.0 | null |

evolution/conformance.json / evolution/v3/23 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "')'" | "*" |

evolution/conformance.json / evolution/v3/24 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'('" | "neg" |

evolution/conformance.json / evolution/v3/25 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9.0 | null |

evolution/conformance.json / evolution/v3/26 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -2.0 | null |

evolution/conformance.json / evolution/v3/27 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2.0 | null |

evolution/conformance.json / evolution/v3/28 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

evolution/conformance.json / evolution/v3/29 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

evolution/conformance.json / evolution/v3/3 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 12.0 | null |

evolution/conformance.json / evolution/v3/30 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'('" | "neg" |

evolution/conformance.json / evolution/v3/31 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

evolution/conformance.json / evolution/v3/33 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

evolution/conformance.json / evolution/v3/36 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 3.0 | null |

evolution/conformance.json / evolution/v3/4 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -3.0 | null |

evolution/conformance.json / evolution/v3/5 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4.0 | null |

evolution/conformance.json / evolution/v3/6 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.5 | null |

evolution/conformance.json / evolution/v3/7 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

evolution/conformance.json / evolution/v3/8 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 150.0 | null |

evolution/conformance.json / evolution/v3/9 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.02 | null |

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

lexical/corpus.json / 6:org.unlaxer.parser.elementary.SingleQuotedParser/0 / lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | "a /*inside*/" | "'a /*inside*/'" |

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

oracle/backref.json / backref/3 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "AlphabetUnderScoreParser" | "ID" |
| /diagnostics/0/farthestExpected/1 | "IdentifierParser" | 欠落 |

oracle/backref.json / backref/4 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"expected":[],"kind":"scope","length":3,"message":"backreference mismatch: foo","offset":0,"severity":"ERROR"} |
| /diagnostics/1 | 欠落 | {"expected":[],"kind":"scope","length":3,"message":"backreference mismatch: foo","offset":4,"severity":"ERROR"} |

oracle/catalog.json / catalog/2 / oracle/catalog.json:0[@catalog]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'Z'" | "Z" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/catalog.json / catalog/3 / oracle/catalog.json:0[@catalog]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'Z'" | "z" |
| /diagnostics/0/farthestExpected/1 | "'z'" | 欠落 |

oracle/char-range.json / char-range/1 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "RootParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |

oracle/char-range.json / char-range/4 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "RootParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |

oracle/char-range.json / char-range/5 / oracle/char-range.json:0[CHAR_RANGE]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "RootParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |

oracle/ci-unicode.json / ci-unicode/3 / oracle/ci-unicode.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'é'" | "T" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/ci.json / ci/2 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/ci.json / ci/4 / oracle/ci.json:0[CI]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'let'" | "T" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/comment-block.json / comment-block/1 / oracle/comment-block.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'b'" | "b" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/common-field.json / common-field/3 / oracle/common-field.json:0[@commonField]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a:'" | "a:" |
| /diagnostics/0/farthestExpected/1 | "'b:'" | "b:" |

oracle/digit.json / digit/1 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 7 | "7" |

oracle/digit.json / digit/2 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "EndOfSourceParser" | "E" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/digit.json / digit/3 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "RootParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |

oracle/digit.json / digit/4 / oracle/digit.json:0[DigitParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "RootParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |

oracle/empty-one-two.json / empty-one-two/1 / oracle/empty-one-two.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 1 | 0 |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/empty-two.json / empty-two/1 / oracle/empty-two.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "Repeat" | "expression" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/empty-zero.json / empty-zero/1 / oracle/empty-zero.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "Repeat" | "expression" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/empty.json / empty/1 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 1 | 0 |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/empty.json / empty/3 / oracle/empty.json:0[EMPTY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 1 | 0 |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/enum-plain.json / enum-plain/3 / oracle/enum-plain.json:0[@enum]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Mode" |
| /diagnostics/0/farthestExpected/0 | "'off'" | "on" |
| /diagnostics/0/farthestExpected/1 | "'on'" | "off" |

oracle/eof.json / eof/2 / oracle/eof.json:0[EOF]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "EndOfSourceParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/eof.json / eof/3 / oracle/eof.json:0[EOF]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "EndOfSourceParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/error.json / error/2 / oracle/error.json:0[ERROR]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/error.json / error/3 / oracle/error.json:0[ERROR]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/eval-binary_arithmetic.json / eval-binary_arithmetic/1 / oracle/eval-binary_arithmetic.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/left/fields/value | 1 | "1" |
| /ast/fields/right/0/fields/value | 2 | "2" |

oracle/eval-conditional.json / eval-conditional/1 / oracle/eval-conditional.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/condition/fields/value | 1 | "1" |
| /ast/fields/elseExpr/fields/value | 3 | "3" |
| /ast/fields/thenExpr/fields/value | 2 | "2" |

oracle/eval-literal.json / eval-literal/1 / oracle/eval-literal.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 7 | "7" |

oracle/eval-manual.json / eval-manual/1 / oracle/eval-manual.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 7 | "7" |

oracle/eval-passthrough.json / eval-passthrough/1 / oracle/eval-passthrough.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 7 | "7" |

oracle/eval-unknown.json / eval-unknown/1 / oracle/eval-unknown.json:0[@eval]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 7 | "7" |

oracle/evolution-v0.json / evolution/0 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/1 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/10 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v0.json / evolution/11 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/evolution-v0.json / evolution/12 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

oracle/evolution-v0.json / evolution/13 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/14 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/15 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestExpected/0 | "' '" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/16 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

oracle/evolution-v0.json / evolution/19 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/2 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.0 | null |

oracle/evolution-v0.json / evolution/20 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/21 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/22 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/23 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/24 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/25 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/26 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/27 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/28 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/29 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/3 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 12.0 | null |

oracle/evolution-v0.json / evolution/30 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/31 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

oracle/evolution-v0.json / evolution/33 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

oracle/evolution-v0.json / evolution/36 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 3.0 | null |

oracle/evolution-v0.json / evolution/4 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -3.0 | null |

oracle/evolution-v0.json / evolution/5 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4.0 | null |

oracle/evolution-v0.json / evolution/6 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.5 | null |

oracle/evolution-v0.json / evolution/7 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

oracle/evolution-v0.json / evolution/8 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 150.0 | null |

oracle/evolution-v0.json / evolution/9 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.02 | null |

oracle/evolution-v1.json / evolution/0 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/1 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/10 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

oracle/evolution-v1.json / evolution/11 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

oracle/evolution-v1.json / evolution/12 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

oracle/evolution-v1.json / evolution/13 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 6.0 | null |

oracle/evolution-v1.json / evolution/14 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

oracle/evolution-v1.json / evolution/15 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestExpected/0 | "' '" | 欠落 |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/16 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

oracle/evolution-v1.json / evolution/19 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/2 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.0 | null |

oracle/evolution-v1.json / evolution/20 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/21 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/22 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/23 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/24 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/25 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/26 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/27 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/28 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/29 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

oracle/evolution-v1.json / evolution/3 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 12.0 | null |

oracle/evolution-v1.json / evolution/30 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/31 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

oracle/evolution-v1.json / evolution/33 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

oracle/evolution-v1.json / evolution/36 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 3.0 | null |

oracle/evolution-v1.json / evolution/4 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -3.0 | null |

oracle/evolution-v1.json / evolution/5 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4.0 | null |

oracle/evolution-v1.json / evolution/6 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.5 | null |

oracle/evolution-v1.json / evolution/7 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

oracle/evolution-v1.json / evolution/8 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 150.0 | null |

oracle/evolution-v1.json / evolution/9 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.02 | null |

oracle/evolution-v2.json / evolution/0 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

oracle/evolution-v2.json / evolution/1 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "NUMBER" |

oracle/evolution-v2.json / evolution/10 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

oracle/evolution-v2.json / evolution/11 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

oracle/evolution-v2.json / evolution/12 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

oracle/evolution-v2.json / evolution/13 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 6.0 | null |

oracle/evolution-v2.json / evolution/14 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

oracle/evolution-v2.json / evolution/15 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Negation" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v2.json / evolution/16 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

oracle/evolution-v2.json / evolution/19 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -6.0 | null |

oracle/evolution-v2.json / evolution/2 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.0 | null |

oracle/evolution-v2.json / evolution/20 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -5.0 | null |

oracle/evolution-v2.json / evolution/21 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -2.0 | null |

oracle/evolution-v2.json / evolution/22 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2.0 | null |

oracle/evolution-v2.json / evolution/23 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "')'" | "*" |

oracle/evolution-v2.json / evolution/24 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

oracle/evolution-v2.json / evolution/25 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

oracle/evolution-v2.json / evolution/26 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

oracle/evolution-v2.json / evolution/27 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

oracle/evolution-v2.json / evolution/28 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

oracle/evolution-v2.json / evolution/29 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

oracle/evolution-v2.json / evolution/3 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 12.0 | null |

oracle/evolution-v2.json / evolution/30 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

oracle/evolution-v2.json / evolution/31 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

oracle/evolution-v2.json / evolution/33 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

oracle/evolution-v2.json / evolution/36 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 3.0 | null |

oracle/evolution-v2.json / evolution/4 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -3.0 | null |

oracle/evolution-v2.json / evolution/5 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4.0 | null |

oracle/evolution-v2.json / evolution/6 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.5 | null |

oracle/evolution-v2.json / evolution/7 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

oracle/evolution-v2.json / evolution/8 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 150.0 | null |

oracle/evolution-v2.json / evolution/9 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.02 | null |

oracle/evolution-v3.json / evolution/0 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'('" | "neg" |

oracle/evolution-v3.json / evolution/1 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "neg" |

oracle/evolution-v3.json / evolution/10 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

oracle/evolution-v3.json / evolution/11 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | "+" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | "*" |

oracle/evolution-v3.json / evolution/12 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

oracle/evolution-v3.json / evolution/13 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 6.0 | null |

oracle/evolution-v3.json / evolution/14 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

oracle/evolution-v3.json / evolution/15 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Conditional" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | "neg" |

oracle/evolution-v3.json / evolution/16 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 5.0 | null |

oracle/evolution-v3.json / evolution/19 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -6.0 | null |

oracle/evolution-v3.json / evolution/2 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.0 | null |

oracle/evolution-v3.json / evolution/20 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -5.0 | null |

oracle/evolution-v3.json / evolution/21 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -2.0 | null |

oracle/evolution-v3.json / evolution/22 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2.0 | null |

oracle/evolution-v3.json / evolution/23 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "')'" | "*" |

oracle/evolution-v3.json / evolution/24 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'('" | "neg" |

oracle/evolution-v3.json / evolution/25 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 9.0 | null |

oracle/evolution-v3.json / evolution/26 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -2.0 | null |

oracle/evolution-v3.json / evolution/27 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 2.0 | null |

oracle/evolution-v3.json / evolution/28 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

oracle/evolution-v3.json / evolution/29 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "' '" | "+" |
| /diagnostics/0/farthestExpected/1 | "'*'" | "*" |

oracle/evolution-v3.json / evolution/3 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 12.0 | null |

oracle/evolution-v3.json / evolution/30 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'('" | "neg" |

oracle/evolution-v3.json / evolution/31 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

oracle/evolution-v3.json / evolution/33 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

oracle/evolution-v3.json / evolution/36 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 3.0 | null |

oracle/evolution-v3.json / evolution/4 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | -3.0 | null |

oracle/evolution-v3.json / evolution/5 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 4.0 | null |

oracle/evolution-v3.json / evolution/6 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.5 | null |

oracle/evolution-v3.json / evolution/7 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 1.0 | null |

oracle/evolution-v3.json / evolution/8 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 150.0 | null |

oracle/evolution-v3.json / evolution/9 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | 0.02 | null |

oracle/extern-full.json / extern-full/2 / oracle/extern-full.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/extern-full.json / extern-full/3 / oracle/extern-full.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/extern.json / extern/1 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 2 | 1 |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/extern.json / extern/2 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/extern.json / extern/3 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/extern.json / extern/4 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "'😀'" |
| /diagnostics/0/farthestExpected/1 | "'😀'" | 欠落 |

oracle/longest.json / longest/3 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /declarations/0 | {"name":"ab","sourceOffset":0} | 欠落 |
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/longest.json / longest/4 / oracle/longest.json:0[@longestChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "'ab'" | "ab" |

oracle/lookahead-failure.json / lookahead-failure/1 / oracle/lookahead-failure.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "x" |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/lookahead-until.json / lookahead-until/1 / oracle/lookahead-until.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 2 | 0 |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/lookahead.json / lookahead/1 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 1 | 0 |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/lookahead.json / lookahead/2 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 1 | 0 |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/lookahead.json / lookahead/3 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "T" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/lookahead.json / lookahead/4 / oracle/lookahead.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "T" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/mapping-alias.json / mapping-alias/2 / oracle/mapping-alias.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/mapping-alias.json / mapping-alias/3 / oracle/mapping-alias.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Leaf" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/mapping-dotted.json / mapping-dotted/3 / oracle/mapping-dotted.json:0[dotted-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "'b'" | "b" |

oracle/mapping-sum.json / mapping-sum/3 / oracle/mapping-sum.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "'b'" | "b" |

oracle/negation-empty.json / negation-empty/1 / oracle/negation-empty.json:0[NEGATION]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "RootParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |

oracle/negative-lookahead.json / negative-lookahead/2 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/negative-lookahead.json / negative-lookahead/3 / oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "T" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

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
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "DigitParser" | ";" |
| /diagnostics/0/farthestExpected/1 | "OneOrMore" | 欠落 |

oracle/posix.json / posix/2 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "RootParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |

oracle/posix.json / posix/3 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "RootParser" | "T" |
| /diagnostics/0/farthestExpected/1 | "TParser" | 欠落 |

oracle/posix.json / posix/4 / oracle/posix.json:0[posix]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/predictive-extern.json / predictive-extern/1 / oracle/predictive-extern.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 2 | 1 |
| /diagnostics/0/rulePath | 欠落 | [] |

oracle/predictive-nullable.json / predictive-nullable/3 / oracle/predictive-nullable.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'b'" | "a" |
| /diagnostics/0/farthestExpected/1 | 欠落 | "x" |

oracle/predictive.json / predictive/2 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "x" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root","Probe::A"] |

oracle/predictive.json / predictive/4 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "'b'" | "b" |

oracle/predictive.json / predictive/5 / oracle/predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"deepestRule":"Probe::A","expected":["a","b"],"farthestExpected":["a","b"],"farthestOffset":0,"kind":"syntax","offset":0,"rulePath":["Probe::Root","Probe::A"]} |

oracle/recovery-auto.json / recovery-auto/1 / oracle/recovery-auto.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | false | true |
| /ast | null | {"fields":{"items":[{"fields":{"value":"ok"},"span":[4,7],"type":"Stmt"}]},"span":[0,7],"type":"Root"} |
| /captures/0/span/1 | 6 | 4 |

oracle/recovery-auto.json / recovery-auto/2 / oracle/recovery-auto.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Stmt" |
| /diagnostics/0/farthestExpected/0 | "';'" | "ok" |
| /diagnostics/0/farthestExpected/1 | "'ok'" | 欠落 |

oracle/recovery-root.json / recovery-root/3 / oracle/recovery-root.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "';'" | "ok" |
| /diagnostics/0/farthestExpected/1 | "'ok'" | 欠落 |

oracle/recovery-sync.json / recovery-sync/2 / oracle/recovery-sync.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Stmt" |
| /diagnostics/0/farthestExpected/0 | "';'" | "ok" |
| /diagnostics/0/farthestExpected/1 | "'ok'" | 欠落 |

oracle/recovery-sync.json / recovery-sync/3 / oracle/recovery-sync.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"items":[]},"span":[0,3],"type":"Root"} | null |
| /captures/0/span/1 | 3 | 2 |

oracle/regex-ascii.json / regex-ascii/1 / oracle/regex-ascii.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/rulePath | 欠落 | [] |

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
| /captures/0/span/1 | 2 | 1 |

oracle/regex-prefix.json / regex-prefix/1 / oracle/regex-prefix.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | false | true |
| /allConsumed | false | true |
| /ast | null | {"fields":{"value":"a"},"span":[0,2],"type":"Value"} |

oracle/regex-prefix.json / regex-prefix/2 / oracle/regex-prefix.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | false | true |
| /captures/0 | 欠落 | {"name":"value","span":[1,2],"text":"a"} |
| /consumed | 0 | 2 |

oracle/regex-zero.json / regex-zero/1 / oracle/regex-zero.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "REGEX('a*')" | "T" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/regex-zero.json / regex-zero/2 / oracle/regex-zero.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "REGEX('a*')" | "T" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/reobserved-catalog.json / catalog/2 / oracle/reobserved-catalog.json:0[@catalog]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'Z'" | "Z" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/reobserved-comment-block.json / comment-block/1 / oracle/reobserved-comment-block.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'b'" | "b" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/reobserved-comment-block.json / comment-block/2 / oracle/reobserved-comment-block.json:0[@comment]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'b'" | "b" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/reobserved-common-field.json / common-field/3 / oracle/reobserved-common-field.json:0[@commonField]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a:'" | "a:" |
| /diagnostics/0/farthestExpected/1 | "'b:'" | "b:" |

oracle/reobserved-enum-plain.json / enum-plain/3 / oracle/reobserved-enum-plain.json:0[@enum]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Mode" |
| /diagnostics/0/farthestExpected/0 | "'off'" | "on" |
| /diagnostics/0/farthestExpected/1 | "'on'" | "off" |

oracle/reobserved-lookahead-failure.json / lookahead-failure/1 / oracle/reobserved-lookahead-failure.json:0[LOOKAHEAD]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "x" |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/reobserved-mapping-dotted.json / mapping-dotted/3 / oracle/reobserved-mapping-dotted.json:0[dotted-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "'b'" | "b" |

oracle/reobserved-mapping-sum.json / mapping-sum/3 / oracle/reobserved-mapping-sum.json:0[transparent-mapping]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "'b'" | "b" |

oracle/reobserved-numeric.json / numeric/1 / oracle/reobserved-numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 2147483647 | "2147483647" |

oracle/reobserved-numeric.json / numeric/6 / oracle/reobserved-numeric.json:0[numeric-capture]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value | 7 | "+0007" |

oracle/reobserved-predictive-nullable.json / predictive-nullable/3 / oracle/reobserved-predictive-nullable.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'b'" | "a" |
| /diagnostics/0/farthestExpected/1 | 欠落 | "x" |

oracle/reobserved-predictive.json / predictive/2 / oracle/reobserved-predictive.json:0[@predictiveChoice]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "x" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root","Probe::A"] |

oracle/repeat-unbounded.json / repeat-unbounded/1 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "Repeat" | "expression" |

oracle/repeat-unbounded.json / repeat-unbounded/2 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "RootBounded0Parser" | 欠落 |

oracle/repeat-unbounded.json / repeat-unbounded/4 / oracle/repeat-unbounded.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "RootBounded0Parser" | 欠落 |

oracle/repeat-zero.json / repeat-zero/2 / oracle/repeat-zero.json:0[bounded-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "expression" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/separated.json / separated/1 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/separated.json / separated/4 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/rulePath | 欠落 | ["Probe::Root"] |

oracle/separated.json / separated/5 / oracle/separated.json:0[separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/skip.json / skip/3 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Item" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "__CaptureSite" | 欠落 |

oracle/skip.json / skip/4 / oracle/skip.json:0[@skip]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Item" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "RootParser" | 欠落 |

oracle/typeof.json / typeof/3 / oracle/typeof.json:0[@typeof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"deepestRule":null,"expected":[],"farthestExpected":[],"farthestOffset":3,"kind":"mapping","offset":3,"rulePath":[]} |
| /mappingError | "java.lang.IllegalArgumentException: @typeof constraint violated: right must be same type as left, expected A but got B" | "@typeof constraint violated: right must be same type as left, expected A but got B" |

oracle/typeof.json / typeof/4 / oracle/typeof.json:0[@typeof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0 | 欠落 | {"deepestRule":null,"expected":[],"farthestExpected":[],"farthestOffset":3,"kind":"mapping","offset":3,"rulePath":[]} |
| /mappingError | "java.lang.IllegalArgumentException: @typeof constraint violated: right must be same type as left, expected B but got A" | "@typeof constraint violated: right must be same type as left, expected B but got A" |

oracle/typeof.json / typeof/5 / oracle/typeof.json:0[@typeof]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::A" |
| /diagnostics/0/farthestExpected/0 | "'a'" | "a" |
| /diagnostics/0/farthestExpected/1 | "'b'" | "b" |

oracle/until.json / until/3 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestExpected/0 | "'#'" | 欠落 |
| /diagnostics/0/farthestOffset | 0 | 1 |

oracle/until.json / until/4 / oracle/until.json:0[UNTIL]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | null |
| /diagnostics/0/farthestOffset | 1 | 0 |
| /diagnostics/0/rulePath | 欠落 | [] |

primitives/corpus.json / 25:ANY/0 / primitives/corpus.json:25[25:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | 欠落 | "" |

primitives/corpus.json / 25:ANY/1 / primitives/corpus.json:25[25:ANY]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "a😀" |
| /ast/fields/value/1 | "😀" | 欠落 |

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

scope-annotations/corpus.json / capture-site-identity/undefined-target-warning / scope-annotations/corpus.json:0[capture-site-identity]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0/message | "未定義のシンボル: 'missing'" | "undefined: missing" |

scope-annotations/corpus.json / repeated-capture-occurrences/all-repeat-captures-produce-effects / scope-annotations/corpus.json:4[repeated-capture-occurrences]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/diagnostics/0/message | "未定義のシンボル: 'missing'" | "undefined: missing" |

scope-annotations/corpus.json / scope-and-declares-parent-registration/valid-prefix-with-trailing-input / scope-annotations/corpus.json:2[scope-and-declares-parent-registration]

| 観測 | 期待 | 実測 |
|---|---|---|
| /scope/declarations/0 | {"name":"kept","sourceOffset":1} | 欠落 |
| /scope/references/0 | {"length":4,"name":"kept","offset":6} | 欠落 |

semantic-cardinality/corpus.json / mixed-alias/0 / semantic-cardinality/corpus.json:5[mixed-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,T:a" | null |

semantic-cardinality/corpus.json / mixed-alias/1 / semantic-cardinality/corpus.json:5[mixed-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | null |

semantic-cardinality/corpus.json / mixed-alias/2 / semantic-cardinality/corpus.json:5[mixed-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀,L:x" | null |

semantic-cardinality/corpus.json / mixed-bounded/0 / semantic-cardinality/corpus.json:10[mixed-bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | null |

semantic-cardinality/corpus.json / mixed-bounded/1 / semantic-cardinality/corpus.json:10[mixed-bounded]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | null |

semantic-cardinality/corpus.json / mixed-group/0 / semantic-cardinality/corpus.json:4[mixed-group]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | null |

semantic-cardinality/corpus.json / mixed-group/1 / semantic-cardinality/corpus.json:4[mixed-group]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | null |

semantic-cardinality/corpus.json / mixed-group/2 / semantic-cardinality/corpus.json:4[mixed-group]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀,T:a" | null |

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

semantic-cardinality/corpus.json / mixed-optional/0 / semantic-cardinality/corpus.json:7[mixed-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / mixed-optional/1 / semantic-cardinality/corpus.json:7[mixed-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)" | null |

semantic-cardinality/corpus.json / mixed-optional/2 / semantic-cardinality/corpus.json:7[mixed-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / mixed-optional/3 / semantic-cardinality/corpus.json:7[mixed-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(😀)" | null |

semantic-cardinality/corpus.json / mixed-optional/4 / semantic-cardinality/corpus.json:7[mixed-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / mixed-pair/0 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:x" | null |

semantic-cardinality/corpus.json / mixed-pair/1 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,T:a" | null |

semantic-cardinality/corpus.json / mixed-pair/2 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | null |

semantic-cardinality/corpus.json / mixed-pair/3 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | null |

semantic-cardinality/corpus.json / mixed-pair/4 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:y" | null |

semantic-cardinality/corpus.json / mixed-pair/5 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀,T:a" | null |

semantic-cardinality/corpus.json / mixed-pair/6 / semantic-cardinality/corpus.json:3[mixed-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-alias/0 / semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-alias/1 / semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-alias/2 / semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-alias/3 / semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a),L:x,T:(😀)" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-alias/4 / semantic-cardinality/corpus.json:14[mixed-repeat-delimited-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a) /*c*/,T:(😀)" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-inline/0 / semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-inline/1 / semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a)" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-inline/2 / semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / mixed-repeat-delimited-inline/3 / semantic-cardinality/corpus.json:15[mixed-repeat-delimited-inline]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:(a),L:x,T:(😀)" | null |

semantic-cardinality/corpus.json / mixed-repeat/0 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / mixed-repeat/1 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | null |

semantic-cardinality/corpus.json / mixed-repeat/2 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / mixed-repeat/3 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x,T:a" | null |

semantic-cardinality/corpus.json / mixed-repeat/4 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:😀,L:x" | null |

semantic-cardinality/corpus.json / mixed-repeat/5 / semantic-cardinality/corpus.json:9[mixed-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x" | null |

semantic-cardinality/corpus.json / mixed-separated/0 / semantic-cardinality/corpus.json:12[mixed-separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | null |

semantic-cardinality/corpus.json / mixed-separated/1 / semantic-cardinality/corpus.json:12[mixed-separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a,L:x,T:😀" | null |

semantic-cardinality/corpus.json / mixed-separated/2 / semantic-cardinality/corpus.json:12[mixed-separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | null |

semantic-cardinality/corpus.json / node-one/0 / semantic-cardinality/corpus.json:0[node-one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / node-one/1 / semantic-cardinality/corpus.json:0[node-one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y" | null |

semantic-cardinality/corpus.json / node-optional/0 / semantic-cardinality/corpus.json:6[node-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / node-optional/1 / semantic-cardinality/corpus.json:6[node-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / node-optional/2 / semantic-cardinality/corpus.json:6[node-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y" | null |

semantic-cardinality/corpus.json / node-pair/0 / semantic-cardinality/corpus.json:2[node-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:x" | null |

semantic-cardinality/corpus.json / node-pair/1 / semantic-cardinality/corpus.json:2[node-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:y" | null |

semantic-cardinality/corpus.json / node-pair/2 / semantic-cardinality/corpus.json:2[node-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:y,L:x" | null |

semantic-cardinality/corpus.json / node-pair/3 / semantic-cardinality/corpus.json:2[node-pair]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:y" | null |

semantic-cardinality/corpus.json / node-repeat/0 / semantic-cardinality/corpus.json:8[node-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / node-repeat/1 / semantic-cardinality/corpus.json:8[node-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / node-repeat/2 / semantic-cardinality/corpus.json:8[node-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:y,L:x" | null |

semantic-cardinality/corpus.json / node-repeat/3 / semantic-cardinality/corpus.json:8[node-repeat]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:x" | null |

semantic-cardinality/corpus.json / node-separated/0 / semantic-cardinality/corpus.json:11[node-separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/corpus.json / node-separated/1 / semantic-cardinality/corpus.json:11[node-separated]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:y" | null |

semantic-cardinality/corpus.json / optional-many-alias/0 / semantic-cardinality/corpus.json:13[optional-many-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/corpus.json / optional-many-alias/1 / semantic-cardinality/corpus.json:13[optional-many-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "T:a" | null |

semantic-cardinality/corpus.json / optional-many-alias/2 / semantic-cardinality/corpus.json:13[optional-many-alias]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,T:a" | null |

semantic-cardinality/pure-alias.json / pure-node-alias-many/0 / semantic-cardinality/pure-alias.json:9[pure-node-alias-many]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/pure-alias.json / pure-node-alias-many/1 / semantic-cardinality/pure-alias.json:9[pure-node-alias-many]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-alias-many/2 / semantic-cardinality/pure-alias.json:9[pure-node-alias-many]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-alias-many/3 / semantic-cardinality/pure-alias.json:9[pure-node-alias-many]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,L:y" | null |

semantic-cardinality/pure-alias.json / pure-node-alias-optional/0 / semantic-cardinality/pure-alias.json:8[pure-node-alias-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/pure-alias.json / pure-node-alias-optional/1 / semantic-cardinality/pure-alias.json:8[pure-node-alias-optional]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-alias-optional/2 / semantic-cardinality/pure-alias.json:8[pure-node-alias-optional]

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

semantic-cardinality/pure-alias.json / pure-node-multi-alias-many/0 / semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-alias-many/1 / semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "L:x,O:🚀,L:x" | null |

semantic-cardinality/pure-alias.json / pure-node-multi-alias-many/2 / semantic-cardinality/pure-alias.json:10[pure-node-multi-alias-many]

| 観測 | 期待 | 実測 |
|---|---|---|
| /value | "O:z,L:x" | null |

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

tiny-code-fence/corpus.json / combined-fence/trailing-nul-before-eol-preserved / tiny-code-fence/corpus.json:11[combined-fence]

| 観測 | 期待 | 実測 |
|---|---|---|
| /fields/body | "tail\u0000" | "tail" |

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
| /captures/0/binding | 301 |
| /captures/1/binding | 94 |
| /captures/10/binding | 4 |
| /captures/11/binding | 2 |
| /captures/12/binding | 2 |
| /captures/13/binding | 2 |
| /captures/14/binding | 2 |
| /captures/2/binding | 77 |
| /captures/3/binding | 71 |
| /captures/4/binding | 70 |
| /captures/5/binding | 15 |
| /captures/6/binding | 6 |
| /captures/7/binding | 4 |
| /captures/8/binding | 4 |
| /captures/9/binding | 4 |
| /diagnosticError | 1 |
| /diagnostics/0/origin | 219 |
| /enumError | 2 |
| /enumValue | 3 |
| /evaluationError | 40 |
| /grammar (操作列の driver protocol 未対応) | 18 |
| /nodes | 24 |
| /operations | 18 |
| /parseError | 1 |
| /parsedStatus | 481 |
| /retainedValueSpans | 4 |
| /rollback | 10 |
| /scope/depth | 20 |
| /scope/resolved | 20 |
| /scopeDepth | 482 |
| /texts | 143 |
| /valueSpans | 14 |

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

294be563a200330d-224 / generation: oracle/regex-backref.json:0[REGEX], oracle/reobserved-regex-backref.json:0[REGEX]

> 終了 exit status: 1:
> unsupported IR feature regex: Probe::T requires an analyzed DFA; Java Pattern fallback is unavailable


## round3 比較と根拠

D-019 により `nativeFailure` を Verdict/未観測の判定から除外し、期待値と実測値は JSON に保持した。公開診断は従来どおり比較する。

| 記録 | Pass | Fail | 実行エラー | 元ケース | 展開後ケース |
|---|---:|---:|---:|---:|---:|
| round2 | 469 | 351 | 0 | 871 | 1195 |
| round3 | 582 | 639 | 0 | 1212 | 1425 |

保存済み round2 と現 checkout の比較。Rust は元ケース871→1212、展開後1195→1425であり、母数と分類の更新を含む。scanner 単独の効果測定には使わない。期待拒否 Pass は通常 Pass と別集計。

round3 の期待拒否 Pass は 28 件。run-all は残る Fail/生成拒否により終了コード1であり、実行基盤の異常終了ではない。

Fail の上位パターン（ケース単位、同じケースは複数パターンに含まれ得る）:

| 比較パス | ケース数 |
|---|---:|
| `/value` | 336 |
| `/diagnostics/*/rulePath` | 215 |
| `/diagnostics/*/deepestRule` | 215 |
| `/diagnostics/*/farthestExpected` | 196 |
| `/ast/fields` | 82 |
| `/diagnostics/*/farthestOffset` | 45 |
| `/captures/*` | 11 |
| `/ast` | 10 |

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

`nativeFailure` 保存: 期待値 485 件、実測値 0 件。判定差分は0件。scanner の SHA-256 は JSON の `round3_analysis.scanner_sha256` に保存した。
