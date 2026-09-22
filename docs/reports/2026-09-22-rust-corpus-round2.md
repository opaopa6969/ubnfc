Rust corpus run-all

round2 検証: `cargo test --workspace --offline` は **334 passed / 0 failed**。
変更 crate（ubnfc-rust / ubnfc-rt-template / ubnfc-selfhost）の all-targets clippy `-D warnings`、fmt、selfhost の IR/Rust 固定点も成功。
P4 の metadata 生成・debug/release build・clippy と、追加 M3/レビュー回帰は `crates/ubnfc-rust/tests/round2.rs` で検証した。

前回（2026-09-22-rust-corpus.md）比: Pass **458→469**、Fail **307→351**、生成拒否 **43→9**。
既存 Pass 458 件は全件維持。旧 Fail の9件が Pass、2件が未観測へ移行。
新規実行から19件、旧未観測から36件が Fail へ移行した。後者には公開した deepestRule/rulePath を
旧 oracle の diagnostics 配列が持たない差が含まれる。新規 eval の Rust 数値字句と Java 保存値、
D-018 recovery の公開情報も比較差分に残る。期待値・乖離登録・harness 比較は変更していない。
生成拒否9件は全て IR の DFA 未供給（5文法）。IR 失敗128件、build/実行エラー各0件。

固定した値: scope 宣言 a@0,a@1,b@3,b@4,c@6,c@8、参照 b@6、trim後の宣言3・参照10/長さ2。
深さ28の共有診断DAGは50ms以内。`abq` の主診断は trailing_input@1、farthest=2。
SYNC/AUTO `bad;ok;` は C=M=7・回復[0,4]、root `bad;` は C=M=4・ASTなし・mappingErrorなし。
`😀;x` は prefix C=M=2・回復[0,2]・recovery@0とtrailing_input@2。

右結合の局所例外: RightScoped の右子消費範囲は[11,13]だが AST span は corpus の[11,12]を優先。
generators.md #138 の「mapped rule の消費範囲」という一般規則との差を限定し、global javaStyle の
既存 span と entry [0,13] は維持する。API は selfhost 互換の rule 表 lexical と token 表 tokens に分け、
driver の lexical は両表を含む。Unicode CI はテンプレートの近似で、Rust/JDK の Unicode 版差は残る。

元 corpus: 83 ファイル、871 ケース。文法 variant: 236、hash キャッシュ: 234 build、展開後: 1195 件（文法なし操作列を含む）。jobs=cargo（既定）。

集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。

Rust generator SHA-256: `093c26c6c396bc18f8be2e8487f42fb4a3c1db5948d3284f6aa47ff0972b115e`

front SHA-256: `5d5843a36a1060bc87b3a3454951ffc480350e98135e1e1b3dec6a850e66e151`

Rust 一括 cargo build --release --offline --workspace: 45.645 秒。個別切り分け込み: 45.645 秒、cargo 起動 1 回。生成 207 crate、生成物 hash 重複除去後 206 crate、既存 cache hit 0 件。

JSON の build_failure / builds[].cargo は Rust、javac_failure / builds[].javac は Java の build 段階。表の build 失敗は両者の合計。比較規則は共通で、診断 offset と数値字句は backend ごとの既存 oracle を使う。

Pass のうち明示的な期待拒否: 1件。build の実測失敗と診断は拒否理由表に保持する。

| 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| associative | 24 | 0 | 0 | 0 | 24 | 0 | 0 | 0 |
| cardinality | 0 | 0 | 0 | 0 | 50 | 0 | 20 | 0 |
| catalog | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| evolution | 0 | 0 | 0 | 0 | 6 | 0 | 2 | 148 |
| lexical | 0 | 0 | 0 | 0 | 44 | 0 | 34 | 0 |
| mixed-values | 0 | 0 | 0 | 0 | 49 | 0 | 49 | 0 |
| nested-captures | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 5 |
| numeric-capture | 0 | 0 | 0 | 0 | 13 | 0 | 0 | 0 |
| oracle | 3 | 9 | 0 | 0 | 1 | 18 | 90 | 47 |
| primitives | 0 | 0 | 0 | 0 | 101 | 0 | 8 | 0 |
| right-associative | 0 | 0 | 0 | 0 | 63 | 0 | 49 | 0 |
| rule-trivia | 0 | 0 | 0 | 0 | 33 | 0 | 0 | 0 |
| scope-annotations | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 11 |
| scope-store | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |
| semantic-cardinality | 101 | 0 | 0 | 0 | 32 | 0 | 30 | 0 |
| tiny-code-fence | 0 | 0 | 0 | 0 | 39 | 0 | 34 | 0 |
| tiny-string-token | 0 | 0 | 0 | 0 | 12 | 0 | 27 | 0 |
| 合計 | 128 | 9 | 0 | 0 | 469 | 18 | 351 | 220 |

6 系統別（case × variant 単位）

| loader 系統 | ir 失敗 | 生成拒否 | build 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| external-grammar | 24 | 0 | 0 | 0 | 156 | 0 | 71 | 148 |
| inline-ast | 0 | 0 | 0 | 0 | 33 | 0 | 8 | 16 |
| operations | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 9 |
| oracle | 3 | 9 | 0 | 0 | 1 | 18 | 90 | 47 |
| rule | 101 | 0 | 0 | 0 | 132 | 0 | 140 | 0 |
| token | 0 | 0 | 0 | 0 | 145 | 0 | 42 | 0 |
| 合計 | 128 | 9 | 0 | 0 | 469 | 18 | 351 | 220 |

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
| catalog/corpus.json:0[deduplicated-sorted-captures] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| catalog/corpus.json:1[rule-order-locality-quantifiers-and-escaped-context] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| evolution/conformance.json:0[0] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:1[1] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:2[2] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/conformance.json:3[3] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 37 |
| evolution/diagnostics.json:0[3] | 0 | 0 | 0 | 0 | 6 | 0 | 2 | 0 |
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
| oracle/backref.json:0[@backref-no-scope] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/catalog.json:0[@catalog] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/char-range.json:0[CHAR_RANGE] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/ci-unicode.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/ci.json:0[CI] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| oracle/comment-block.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1 |
| oracle/comment-line.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 1 |
| oracle/comment-whitespace.json:0[@comment] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 2 |
| oracle/common-field-missing.json:0[@commonField] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/common-field-type.json:0[@commonField] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/common-field.json:0[@commonField] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/digit.json:0[DigitParser] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/empty-one-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty-two.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/empty.json:0[EMPTY] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/enum-collision.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| oracle/enum-plain.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/enum.json:0[@enum] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/eof.json:0[EOF] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/error.json:0[ERROR] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/eval-binary_arithmetic.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-conditional.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-literal.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-manual.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-passthrough.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-sum.json:0[@eval] | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/eval-unknown.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/eval-variable_ref.json:0[@eval] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| oracle/extern-full.json:0[Extern] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 |
| oracle/extern.json:0[Extern] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/import-alias.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/import-cycle.json:0[@import] | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| oracle/import-two-level.json:0[@import] | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| oracle/longest.json:0[@longestChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| oracle/lookahead-failure.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead-until.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| oracle/lookahead.json:0[LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| oracle/mapping-alias.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/mapping-dotted.json:0[dotted-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/mapping-sum.json:0[transparent-mapping] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/negation-empty.json:0[NEGATION] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/negative-lookahead.json:0[NEGATIVE_LOOKAHEAD] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |
| oracle/numeric.json:0[numeric-capture] | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 0 |
| oracle/posix.json:0[posix] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| oracle/predictive-extern.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/predictive-nullable.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| oracle/predictive.json:0[@predictiveChoice] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/recovery-auto.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| oracle/recovery-root.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1 |
| oracle/recovery-skip.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| oracle/recovery-sync.json:0[@recovery] | 0 | 0 | 0 | 0 | 0 | 1 | 2 | 0 |
| oracle/regex-ascii.json:0[REGEX] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-backref.json:0[REGEX] | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-emoji.json:0[REGEX] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-prefix.json:0[REGEX] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/regex-zero.json:0[REGEX] | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| oracle/repeat-unbounded.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| oracle/repeat-zero.json:0[bounded-repeat] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| oracle/separated.json:0[separated] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/skip.json:0[@skip] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| oracle/typeof.json:0[@typeof] | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 |
| oracle/until.json:0[UNTIL] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 3 |
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
| primitives/corpus.json:25[25:ANY] | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
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
| scope-annotations/corpus.json:0[capture-site-identity] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| scope-annotations/corpus.json:1[nested-lexical-dynamic] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| scope-annotations/corpus.json:2[scope-and-declares-parent-registration] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 2 |
| scope-annotations/corpus.json:3[failed-choice-rollback] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| scope-annotations/corpus.json:4[repeated-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| scope-annotations/corpus.json:5[optional-recursive-boundary] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| scope-annotations/corpus.json:6[quantified-capture-occurrences] | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
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
| 合計 | 128 | 9 | 0 | 0 | 469 | 18 | 351 | 220 |

拒否理由（M3 作業リスト）

同一文法の拒否はキャッシュ単位に数える。ケース数は理由ごとに重複除去する。生成器が最初の拒否で停止するため、後続の未対応機能を網羅する件数ではない。IR 診断が複数あると理由間で重複する。

| 段階 / 機能・診断コード | build 数 | 元ケース数 | 展開後件数 |
|---|---:|---:|---:|
| generation/regex | 5 | 9 | 9 |
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

oracle/extern-full.json / extern-full/1 / oracle/extern-full.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /accepted | true | false |
| /allConsumed | true | false |
| /ast | {"fields":{"value":"😀"},"span":[0,2],"type":"Value"} | null |

oracle/extern-full.json / extern-full/2 / oracle/extern-full.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "T" |
| /diagnostics/0/farthestOffset | 1 | 0 |

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
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "T" |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/extern.json / extern/3 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "T" |
| /diagnostics/0/farthestOffset | 1 | 0 |

oracle/extern.json / extern/4 / oracle/extern.json:0[Extern]

| 観測 | 期待 | 実測 |
|---|---|---|
| /diagnostics/0/deepestRule | 欠落 | "Probe::Root" |
| /diagnostics/0/farthestExpected/0 | "'x'" | "T" |
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
| /accepted | true | false |
| /captures/0 | {"binding":"A:0","name":"value","span":[0,1],"text":"😀"} | 欠落 |
| /consumed | 1 | 0 |

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
| /captures/0/binding | 64 |
| /captures/1/binding | 17 |
| /captures/2/binding | 5 |
| /captures/3/binding | 3 |
| /captures/4/binding | 2 |
| /captures/5/binding | 1 |
| /diagnosticError | 1 |
| /diagnostics/0/origin | 73 |
| /enumError | 1 |
| /enumValue | 2 |
| /expectation | 148 |
| /grammar (操作列の driver protocol 未対応) | 9 |
| /nativeFailure | 143 |
| /nodes | 14 |
| /operations | 9 |
| /parseError | 1 |
| /parsedStatus | 142 |
| /rollback | 2 |
| /scope/depth | 14 |
| /scope/resolved | 14 |
| /scopeDepth | 143 |
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
