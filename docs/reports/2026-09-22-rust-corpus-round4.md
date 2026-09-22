Rust corpus run-all

元 corpus: 127 ファイル、1212 ケース。文法 variant: 279、hash キャッシュ: 234 build、展開後: 1425 件（文法なし操作列を含む）。jobs=cargo（既定）。

集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。

Rust generator SHA-256: `4b1ffda7ebad73ea6cb187e9d84f38742623c6450aeed4ee8ddfe5662e0589c1`

front SHA-256: `71cebb959d34e654103870049c0466c32825840c73662a6fc9dd2e501256184e`

Rust 一括 cargo build --release --offline --workspace: 117.770 秒。個別切り分け込み: 117.770 秒、cargo 起動 1 回。生成 228 crate、生成物 hash 重複除去後 227 crate、既存 cache hit 0 件。

JSON の build_failure / builds[].cargo は Rust、javac_failure / builds[].javac は Java の build 段階。表の build 失敗は両者の合計。比較規則は共通で、診断 offset と数値字句は backend ごとの既存 oracle を使う。

期待拒否 Pass: 28件（Verdict は Pass）。通常の Pass・IR 失敗とは別列に排他的に計上する。build の実測失敗と診断は拒否理由表に保持する。

| 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 0 | 0 | 0 | 0 | 24 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 70 | 0 | 0 | 0 | 0 |
| catalog | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| evolution | 0 | 0 | 0 | 0 | 72 | 0 | 0 | 68 | 16 |
| lexical | 0 | 0 | 0 | 0 | 56 | 0 | 0 | 22 | 0 |
| mixed-values | 0 | 0 | 0 | 0 | 93 | 0 | 0 | 5 | 0 |
| nested-captures | 0 | 0 | 0 | 0 | 10 | 0 | 0 | 0 | 0 |
| numeric-capture | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 | 0 |
| oracle | 0 | 2 | 0 | 0 | 169 | 4 | 18 | 171 | 34 |
| primitives | 0 | 0 | 0 | 0 | 109 | 0 | 0 | 0 | 0 |
| right-associative | 0 | 0 | 0 | 0 | 112 | 0 | 0 | 0 | 0 |
| rule-trivia | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 | 0 |
| scope-annotations | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 11 |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 0 | 0 | 0 | 0 | 163 | 0 | 0 | 0 | 0 |
| tiny-code-fence | 0 | 0 | 0 | 0 | 73 | 0 | 0 | 0 | 0 |
| tiny-string-token | 0 | 0 | 0 | 0 | 39 | 0 | 0 | 0 | 0 |
| 合計 | 0 | 2 | 0 | 0 | 1038 | 28 | 18 | 269 | 70 |

6 系統別（case × variant 単位）

| loader 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | 期待拒否 Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 0 | 0 | 0 | 0 | 291 | 24 | 0 | 68 | 16 |
| inline-ast | 0 | 0 | 0 | 0 | 43 | 0 | 0 | 3 | 11 |
| operations | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 9 |
| oracle | 0 | 2 | 0 | 0 | 169 | 4 | 18 | 171 | 34 |
| rule | 0 | 0 | 0 | 0 | 368 | 0 | 0 | 5 | 0 |
| token | 0 | 0 | 0 | 0 | 165 | 0 | 0 | 22 | 0 |
| 合計 | 0 | 2 | 0 | 0 | 1038 | 28 | 18 | 269 | 70 |

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
| evolution/conformance.json:0[0] | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 20 | 4 |
| evolution/conformance.json:1[1] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 19 | 4 |
| evolution/conformance.json:2[2] | 0 | 0 | 0 | 0 | 18 | 0 | 0 | 15 | 4 |
| evolution/conformance.json:3[3] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 12 | 4 |
| evolution/diagnostics.json:0[3] | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:0[0:IdentifierParser] | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:10[10:SingleQuotedParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:11[11:EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:12[12:org.unlaxer.parser.elementary.EndOfSourceParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:13[13:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:14[14:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:15[15:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:16[16:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:17[17:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:18[18:SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:19[19:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:1[1:org.unlaxer.parser.clang.IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:20[20:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:21[21:IdentifierParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:2[2:IdentifierParser] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:3[3:IdentifierParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:4[4:SingleQuotedParser] | 0 | 0 | 0 | 0 | 7 | 0 | 0 | 14 | 0 |
| lexical/corpus.json:5[5:DoubleQuotedParser] | 0 | 0 | 0 | 0 | 9 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:6[6:org.unlaxer.parser.elementary.SingleQuotedParser] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| lexical/corpus.json:7[7:org.unlaxer.parser.elementary.DoubleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| lexical/corpus.json:8[8:SingleQuotedParser] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| lexical/corpus.json:9[9:DoubleQuotedParser] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| mixed-values/corpus.json:0[two] | 0 | 0 | 0 | 0 | 20 | 0 | 0 | 3 | 0 |
| mixed-values/corpus.json:1[one] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 2 | 0 |
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
| oracle/catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| oracle/ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/ci.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| oracle/comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 1 | 0 |
| oracle/comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | 0 |
| oracle/comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 | 0 |
| oracle/common-field-missing.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/common-field-type.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
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
| oracle/empty.json:0[EMPTY] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/enum-collision.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| oracle/enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/enum.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/eof.json:0[EOF] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/error.json:0[ERROR] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/eval-binary_arithmetic.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-conditional.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-literal.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-manual.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-passthrough.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-sum.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/eval-unknown.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-variable_ref.json:0[@eval] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/evolution-v0.json:0[evolution] | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 20 | 4 |
| oracle/evolution-v1.json:0[evolution] | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 19 | 4 |
| oracle/evolution-v2.json:0[evolution] | 0 | 0 | 0 | 0 | 18 | 0 | 0 | 15 | 4 |
| oracle/evolution-v3.json:0[evolution] | 0 | 0 | 0 | 0 | 21 | 0 | 0 | 12 | 4 |
| oracle/extern-full.json:0[Extern] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/extern.json:0[Extern] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/import-alias.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/import-cycle.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/import-two-level.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| oracle/lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead-until.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 0 |
| oracle/posix.json:0[posix] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 3 | 0 |
| oracle/predictive-extern.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 |
| oracle/predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| oracle/recovery-auto.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0 |
| oracle/recovery-root.json:0[@recovery] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/recovery-skip.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| oracle/recovery-sync.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 | 0 |
| oracle/regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/regex-backref.json:0[REGEX] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-emoji.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/regex-prefix.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/regex-zero.json:0[REGEX] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/reobserved-catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/reobserved-char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-ci.json:0[CI] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 |
| oracle/reobserved-comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/reobserved-enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/reobserved-longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/reobserved-mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/reobserved-mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/reobserved-negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| oracle/reobserved-negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/reobserved-posix.json:0[posix] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/reobserved-predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/reobserved-regex-ascii.json:0[REGEX] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| oracle/reobserved-regex-backref.json:0[REGEX] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/repeat-unbounded.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 3 | 0 |
| oracle/repeat-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 |
| oracle/separated.json:0[separated] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| oracle/skip.json:0[@skip] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | 0 |
| oracle/typeof.json:0[@typeof] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
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
| tiny-string-token/corpus.json:0[direct-eof] | 0 | 0 | 0 | 0 | 20 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:1[prefix-vs-full-input] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:2[comments-and-eof] | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:3[transparent-alias] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:4[optional] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:5[repeat] | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| tiny-string-token/corpus.json:6[mapped-node] | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| 合計 | 0 | 2 | 0 | 0 | 1038 | 28 | 18 | 269 | 70 |

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

evolution/conformance.json / evolution/v0/19 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

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

evolution/conformance.json / evolution/v0/30 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v0/33 / evolution/conformance.json:0[0]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

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

evolution/conformance.json / evolution/v1/19 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

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

evolution/conformance.json / evolution/v1/30 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

evolution/conformance.json / evolution/v1/33 / evolution/conformance.json:1[1]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

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

evolution/conformance.json / evolution/v2/30 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

evolution/conformance.json / evolution/v2/33 / evolution/conformance.json:2[2]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

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

evolution/conformance.json / evolution/v3/30 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'('" | "neg" |

evolution/conformance.json / evolution/v3/33 / evolution/conformance.json:3[3]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

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

lexical/corpus.json / 16:SingleQuotedParser/0 / lexical/corpus.json:16[16:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/0 | "a" | "'a'" |
| /ast/fields/value/1 | "b" | "'b'" |

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
| /ast/fields/value/0 | "a" | "'a'" |
| /ast/fields/value/1 | "b" | "'b'" |

lexical/corpus.json / 8:SingleQuotedParser/1 / lexical/corpus.json:8[8:SingleQuotedParser]

| 観測 | 期待 | 実測 |
|---|---|---|
| /ast/fields/value/1 | "b" | "'b'" |

mixed-values/corpus.json / Mixed.ubnf/10 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /texts/0/2 | "b" | "'b'" |
| /value | "L:x&#124;T:b&#124;T:a,L:x,T:a" | "L:x&#124;T:'b'&#124;T:a,L:x,T:a" |

mixed-values/corpus.json / Mixed.ubnf/10 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /texts/0/2 | "b" | "'b'" |
| /value | "L:x&#124;T:b&#124;T:a,L:x,T:a" | "L:x&#124;T:'b'&#124;T:a,L:x,T:a" |

mixed-values/corpus.json / Mixed.ubnf/6 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /texts/0/2 | "b" | "'b'" |
| /texts/1/2 | "c" | "'c'" |
| /texts/2/2 | "a" | "'a'" |

mixed-values/corpus.json / Mixed.ubnf/7 / mixed-values/corpus.json:0[two]

| 観測 | 期待 | 実測 |
|---|---|---|
| /texts/0/2 | "😀" | "'😀'" |
| /texts/2/2 | "\\q" | "'\\q'" |
| /value | "T:😀&#124;T:a&#124;T:\\q" | "T:'😀'&#124;T:a&#124;T:'\\q'" |

mixed-values/corpus.json / Mixed.ubnf/7 / mixed-values/corpus.json:1[one]

| 観測 | 期待 | 実測 |
|---|---|---|
| /texts/0/2 | "😀" | "'😀'" |
| /texts/2/2 | "\\q" | "'\\q'" |
| /value | "T:😀&#124;T:a&#124;T:\\q" | "T:'😀'&#124;T:a&#124;T:'\\q'" |

oracle/backref.json / backref/3 / oracle/backref.json:0[@backref-no-scope]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "AlphabetUnderScoreParser" | "ID" |
| /diagnostics/0/farthestExpected/1 | "IdentifierParser" | 欠落 |

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

oracle/evolution-v0.json / evolution/19 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

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

oracle/evolution-v0.json / evolution/30 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v0.json / evolution/33 / oracle/evolution-v0.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | 欠落 |

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

oracle/evolution-v1.json / evolution/19 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

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

oracle/evolution-v1.json / evolution/30 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "NUMBER" |
| /diagnostics/0/farthestExpected/1 | "'*/'" | 欠落 |

oracle/evolution-v1.json / evolution/33 / oracle/evolution-v1.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

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

oracle/evolution-v2.json / evolution/30 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "neg" |
| /diagnostics/0/farthestExpected/1 | "'('" | "NUMBER" |

oracle/evolution-v2.json / evolution/33 / oracle/evolution-v2.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

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

oracle/evolution-v3.json / evolution/30 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Digits" |
| /diagnostics/0/farthestExpected/0 | "' '" | "if" |
| /diagnostics/0/farthestExpected/1 | "'('" | "neg" |

oracle/evolution-v3.json / evolution/33 / oracle/evolution-v3.json:0[evolution]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Evolution::Binary" |
| /diagnostics/0/farthestExpected/0 | "'*/'" | "+" |
| /diagnostics/0/farthestExpected/1 | "WildCardStringParser" | "*" |

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

oracle/recovery-auto.json / recovery-auto/2 / oracle/recovery-auto.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Stmt" |
| /diagnostics/0/farthestExpected/0 | "';'" | "ok" |
| /diagnostics/0/farthestExpected/1 | "'ok'" | 欠落 |

oracle/recovery-root.json / recovery-root/1 / oracle/recovery-root.json:0[@recovery]

| 観測 | 期待 | 実測 |
|---|---|---|
| /parsedStatus | "failed" | "succeeded" |
| /accepted | false | true |
| /allConsumed | false | true |

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
| /parsedStatus | "failed" | "succeeded" |
| /accepted | false | true |
| /allConsumed | false | true |

oracle/regex-emoji.json / regex-emoji/2 / oracle/regex-emoji.json:0[REGEX]

| 観測 | 期待 | 実測 |
|---|---|---|
| /allConsumed | true | false |
| /ast | {"fields":{"value":"😀x"},"span":[0,2],"type":"Value"} | null |
| /captures/0/span/1 | 2 | 1 |

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
| /captures/0 | 欠落 | {"binding":"Root:0","name":"value","span":[1,2],"text":"a"} |

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

未観測項目（展開後件数。欠落を Pass に読み替えない）

| 観測 | 件数 |
|---|---:|
| /diagnosticError | 1 |
| /diagnostics/0/origin | 218 |
| /enumError | 2 |
| /enumValue | 3 |
| /evaluationError | 32 |
| /grammar (操作列の driver protocol 未対応) | 18 |
| /operations | 18 |
| /parseError | 1 |
| /rollback | 10 |
| /scope/resolved | 20 |

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

## Round4 の修正と比較

通常 Pass **582 → 1038**、Fail **639 → 269**（370 件減）、Known **18 → 18**、未観測 **156 → 70**。期待拒否 Pass 28、生成拒否 2 は別集計。build 失敗・driver error は 0。

| 系統 | Pass（round3 → round4） | Fail（round3 → round4） |
|---|---:|---:|
| associative | 24 → 24 | 0 → 0 |
| cardinality | 50 → 70 | 20 → 0 |
| catalog | 2 → 2 | 0 → 0 |
| evolution | 6 → 72 | 130 → 68 |
| lexical | 53 → 56 | 25 → 22 |
| mixed-values | 49 → 93 | 49 → 5 |
| nested-captures | 0 → 10 | 5 → 0 |
| numeric-capture | 13 → 13 | 0 → 0 |
| oracle | 27 → 169 | 236 → 171 |
| primitives | 101 → 109 | 8 → 0 |
| right-associative | 63 → 112 | 49 → 0 |
| rule-trivia | 33 → 33 | 0 → 0 |
| scope-annotations | 0 → 0 | 3 → 3 |
| scope-store | 0 → 0 | 0 → 0 |
| semantic-cardinality | 69 → 163 | 94 → 0 |
| tiny-code-fence | 72 → 73 | 1 → 0 |
| tiny-string-token | 20 → 39 | 19 → 0 |

### 修正した契約違反と出力


- Java driver と同じ Power の葉・右結合、演算子 fold、Negation/Conditional、semantic-cardinality の T:/L:/O:、mixed-values、cardinality を AST から評価。Rust AST の数値字句は保持し、評価時だけ数値へ変換。有限数値と JSON の浮動小数表現を保持する。
- optional/repeat の capture を各要素に付ける。不在を空文字一個にせず、nested capture の完了順と rule 完了順を区別する。prefix captures、binding、parsedStatus、実際の scopeDepth を出力する。
- 意味値の個数と構文要素数を分離。単一値の group/alias の span、空 optional/repeat、多値 alias、separator を除いた各値の span を保持する。採用 AST のパスへ span を確定し、同じ文字列の別出現を区別する。
- texts は Rust の意味値 Text、nodes は Leaf/Other/OtherLeaf、valueSpans/retainedValueSpans は指定 field の保持値から出力する。
- Extern scanner の text span と消費 span を分離。single quote の内容、escape、改行、NUL、alias を保持し、capture は入力の消費範囲を切り出す。tiny-string-token の 19 Fail を解消した。
- 引用符 scanner の失敗では入口の二カーソルを保持する。文字列の trim で NUL を落とさない。同一 rule の backref は最初の対象 capture と比較し、実際の不一致箇所・メッセージ・severity を出す。
- scanner feature 付き生成 driver の重複 allow 属性と、文法に依存する未使用 runtime の clippy エラーを修正した。

### 残る Fail の上位観測

同一ケース内の重複を含む。未出力を Pass にする変更は行っていない。

| 観測 | round3 | round4 |
|---|---:|---:|
| `value` | 336 | 5 |
| `ast` | 92 | 41 |
| `captures` | 11 | 5 |
| `texts` | 0 | 5 |
| `deepestRule` | 215 | 215 |
| `rulePath` | 215 | 215 |
| `farthestExpected` | 196 | 196 |
| `farthestOffset` | 45 | 44 |
| `primaryOffset` | 8 | 6 |
| `mappingError` | 6 | 6 |

### 判断と残存差分

- deepestRule/rulePath の 215 件は未出力ではない。oracle の diagnostics に存在しない追加項目と、Rust の実測ルール情報の差分。情報を消して Pass にせず保持した。farthestExpected 196 件・farthestOffset 44 件も残す。D-014 の主位置 rustOffset と Java の試行失敗位置を混同しない。
- 組込み SingleQuotedParser の字句は自己ホストの front 変換が引用符込みで消費するため維持した。専用組込み経路の lexical 22 件と mixed-values 5 件などは残る。front/自己ホストの手書き投影を変更せず、再生成固定点を維持するための判断。
- Oracle の Java int mapping と Rust 字句保持、recovery、scope の prefix/メッセージ等の差が残る。oracle・rustOffset・期待値・乖離登録・比較器は変更していない。
- Pass から Fail への後退は 0 件。recovery-root/1 は parsedStatus の追加で未承認差分を観測し Known から Fail へ移った。recovery-auto/1 は Fail から Known へ移った。
- 未観測は 156 から 70 へ減少。texts/nodes/valueSpans/retainedValueSpans/capture binding/parsedStatus/scopeDepth の欠落を解消。残る主な欠落は evaluationError、scope/resolved、rollback、操作列の driver protocol。
- Java round3 は Pass 1000 / Fail 254、Rust round4 は Pass 1038 / Fail 269。Rust 固有の Text span も比較するため観測範囲は同一ではない。

生成器と front の固定コピーで指定の run-all コマンドを実行した。バイナリの SHA-256 は本レポート冒頭と JSON に保持する。

### 検証

- `cargo test --workspace --offline`: 成功。
- `cargo clippy --workspace --offline --all-targets -- -D warnings`: 成功。
- `cargo fmt --all -- --check`: 成功。
- `cargo test -p ubnfc-selfhost --offline --test regenerate`: 成功。生成 IR 固定点と全生成バイト一致の 2 テスト
- `cargo run -q --bin ubnfc-harness -- run-all --backend rust --report docs/reports/2026-09-22-rust-corpus-round4.md --json docs/reports/2026-09-22-rust-corpus-round4.json`: レポート生成済み。Fail 269 と生成拒否 2 のため所定の非ゼロ終了。生成器と front は固定コピー、SHA-256 を記録。
- `generated driver: cargo clippy --offline --all-targets --features scanners -- -D warnings`: 成功。driver 回帰テスト内で検証。既存 json feature の clippy も成功。
