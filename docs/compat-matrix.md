# UBNF 互換行列（M0・2026-09-22）

対象は設計 v1 §1・§3・§7（`docs/design/03-design-v1.md:24,55,218`）。独立期待値を優先し、実装同士の一致だけを oracle に格上げしない。
本書は静的照合と既存 `target/debug/ubnfc-harness list` の読取実行による。生成・コンパイル・適合テストの再実行結果ではない。

参照の基点：`U=/home/opa/work/unlaxer-workspace/unlaxer-parser`、`H=crates/ubnfc-harness/src`（ubnfc 内）。
`J=U/unlaxer-dsl/src/main/java/org/unlaxer/dsl/codegen`、`T=U/unlaxer-dsl/src/test/java/org/unlaxer/dsl/codegen`、`C=U/unlaxer-common/src/main/java/org/unlaxer`、`R=U/rust/unlaxer-runtime/src/lib.rs`。
`specs/` は `U/unlaxer-dsl/specs/`、`v1` は `docs/design/03-design-v1.md`。corpus はすべて `U/unlaxer-dsl/src/test/resources/` 相対。

## 1. 機能 × 観測面

| 略号 | corpus ファイル | loader 件数 |
|---|---|---:|
| N | nested-captures/corpus.json | 10 |
| Tt | rule-trivia/corpus.json | 33 |
| S | scope-annotations/corpus.json | 14 |
| P | primitives/corpus.json | 109 |
| L | lexical/corpus.json | 78 |
| SC | semantic-cardinality/corpus.json | 94 |
| PA | semantic-cardinality/pure-alias.json | 69 |
| F | tiny-code-fence/corpus.json | 73 |
| Q | tiny-string-token/corpus.json | 39 |
| M | mixed-values/corpus.json | 23 |
| MD | mixed-values/delimited-corpus.json | 15 |
| MS | mixed-values/shared-corpus.json | 11 |
| E | evolution/conformance.json | 37 |
| D | evolution/diagnostics.json | 8 |
| A | associative/corpus.json | 24 |
| RA | right-associative/corpus.json | 28 |
| K | cardinality/corpus.json | 14 |
| Nu | numeric-capture/conformance.json | 13 |
| O | scope-store/corpus.json | 9 |
| Cat | catalog/corpus.json | 2 |

`P[12]:3件` は JSON の fixture 添字12（0始まり）の全3ケース、`Nu/6` は平坦配列の添字6。`N[0]/inner-before-outer` の実 ID は `nested-text-completion/inner-before-outer`。
fixture 添字から `name`、token 系は `添字:token` を引くと loader ID が復元できる（`H/corpus/mod.rs:174`）。`D/7` は `diagnostics/7`、`Nu/6` は `numeric/6`。件数は参照群の総数で、AST 列はその受理行に適用する。セルは代表群であり加算しない。
「受理」は全入力受理。「prefix」は解析自体の成否で残余入力を許す。空欄相当は **oracle なし** と記す（未実装・無効の断定ではない）。
AST 型は node の `type` と field cardinality、span は code point の半開区間。capture 出現は明示的な完了順・位置列で、field 値だけから補わない。診断列には scope の severity/offset も含む。
P/L の受理時 `Value` 型と root span `[0,入力cp長]` は独立規則（`T/RustConformanceTest.java:561`）。N の cursor は JSON でなくテストの `[true,入力cp長,入力cp長]` 規則（`T/NestedCaptureConformanceTest.java:165,345`）。
SC/PA は型・値・`texts/nodes` と root span を独立検証するが、prefix は Java 実行結果との比較だけ（`T/SemanticCardinalityConformance.java:100,133,149`）。A/RA の全 AST/span の相互比較も独立固定値とは区別する。

| 機能 | 受理/不受理 | prefix 判定 | consumed | matched | AST 型 | AST 値 | span | capture 出現 | mapping 失敗 | scope 履歴 | 診断 offset/kind | metadata |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| リテラル（接頭辞一致） | N:10件 | N:10件 | N:10件 | N:10件 | N:10件 | N:10件 | N:10件 | N:10件 | **oracle なし** | N[0]:1件 | D/1 | **oracle なし** |
| NumberParser（短名/FQCN） | Nu:13件 | **oracle なし** | **oracle なし** | **oracle なし** | Nu:13件（Java int） | Nu:13件（言語別） | **oracle なし** | **oracle なし** | Nu/6–12:7件 | **oracle なし** | D/5–7:3件 | **oracle なし** |
| IdentifierParser（ASCII） | L[0]:11件 | L[0]:prefix 4件 | L[0]:prefix 4件 | L[0]:prefix 4件 | L[0]:受理3件 | L[0]:受理3件 | L[0]:受理3件 | N[3]:1件 | **oracle なし** | S[0]:2件 | S[0]:2件 | **oracle なし** |
| SingleQuotedParser | L[4]:21件 | L[4]:prefix 2件 | L[4]:prefix 2件 | L[4]:prefix 2件 | L[4]:受理14件 | L[4]:受理14件 | L[4]:受理14件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| DoubleQuotedParser（QuotedParser 系） | L[5]:9件 | L[5]:prefix 1件 | L[5]:prefix 1件 | L[5]:prefix 1件 | L[5]:受理6件 | L[5]:受理6件 | L[5]:受理6件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| DigitParser（直接 capture） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| その他 posix token（AlphabetParser 等） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| ANY | P[0]:5件 | F[4]:3件 | F[4]:3件 | F[4]:3件 | P[0]:受理3件 | P[0]:受理3件 | P[0]:受理3件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| CHAR_RANGE | P[1,2]:7件 | **oracle なし** | **oracle なし** | **oracle なし** | P[1,2]:受理3件 | P[1,2]:受理3件 | P[1,2]:受理3件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| NEGATION | P[3,39]:8件 | **oracle なし** | **oracle なし** | **oracle なし** | P[3,39]:受理4件 | P[3,39]:受理4件 | P[3,39]:受理4件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| CI | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| REGEX（正則部分集合／非正則） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| UNTIL（終端なし EOF を含む） | P[12,13]:5件 | P[12,13]:5件 | P[12,13]:5件 | P[12,13]:5件 | P[12,13]:受理4件 | P[12,13]:受理4件 | P[12,13]:受理4件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| EOF / EndOfSourceParser（ゼロ消費） | P[5]:3件・L[11]:2件 | L[11]:2件 | L[11]:2件 | L[11]:2件 | L[11]/0 | L[11]/0 | L[11]/0 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| EMPTY（ゼロ消費） | P[4]:2件・P[23]:2件 | P[23]/1 | P[23]/1 | P[23]/1 | P[42]/0 | P[42]/0 | P[42]/0 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| LOOKAHEAD（ゼロ消費） | P[6]:4件 | P[21]/0 | P[21]/0 | P[21]/0 | P[6]:受理1件 | P[6]:受理1件 | P[6]:受理1件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| NEGATIVE_LOOKAHEAD（ゼロ消費） | P[7,15]:5件 | **oracle なし** | **oracle なし** | **oracle なし** | P[7,15]:受理2件 | P[7,15]:受理2件 | P[7,15]:受理2件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| 既知 Extern: tiny StringLiteralParser | Q:39件 | Q:39件 | Q:39件 | Q:39件 | Q:39件 | Q:39件 | Q:39件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| 既知 Extern: tiny CodeStart/CodeEndParser | F:73件 | F:73件 | F:73件 | F:73件 | F:73件 | F:73件 | F:73件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| 汎用 Extern token／利用者 scanner | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| 連接 | Tt[0]:3件 | Tt[0]:3件 | Tt[0]:3件 | Tt[0]:3件 | Tt[0]:3件 | Tt[0]:3件 | Tt[0]:3件 | N[0]:1件 | **oracle なし** | S[3]:2件 | D:8件 | **oracle なし** |
| 通常の選択（宣言順） | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | N[1]:2件 | **oracle なし** | S[3]:2件 | D:8件 | **oracle なし** |
| optional（[...] / ?） | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | Tt[12]:3件 | N[1]:2件 | **oracle なし** | S[5]:2件 | **oracle なし** | **oracle なし** |
| repeat（{...} / * / +） | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | **oracle なし** | S[6]:1件 | **oracle なし** | **oracle なし** |
| bounded（{n,m} / {n,}） | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | **oracle なし** | S[6]:1件 | **oracle なし** | **oracle なし** |
| 区切りリスト % | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | **oracle なし** | S[6]:1件 | **oracle なし** | **oracle なし** |
| group (...) | N[0]:1件 | N[0]:1件 | N[0]:1件 | N[0]:1件 | N[0]:1件 | N[0]:1件 | N[0]:1件 | N[0]:1件 | **oracle なし** | N[0]:1件 | **oracle なし** | **oracle なし** |
| capture: scalar | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | **oracle なし** | S[0]:2件 | S[0]:2件 | Cat:2件 |
| capture: optional | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | N[1]:2件 | **oracle なし** | S[5]:2件 | **oracle なし** | Cat:2件 |
| capture: list | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | N[2]:1件 | **oracle なし** | S[4]:1件 | S[4]:1件 | Cat:2件 |
| capture: 同名 nested | N[0,3,6]:3件 | N[0,3,6]:3件 | N[0,3,6]:3件 | N[0,3,6]:3件 | N[0,3,6]:3件 | N[0,3,6]:3件 | N[0,3,6]:3件 | N[0,3,6]:3件 | **oracle なし** | N[0]:1件 | **oracle なし** | **oracle なし** |
| @mapping(Class, params=[...]) | N:10件 | N:10件 | N:10件 | N:10件 | N:10件 | N:10件 | N:10件 | N:10件 | Nu/6–12:7件 | **oracle なし** | **oracle なし** | **oracle なし** |
| @mapping(Class)（0 field の具象型） | F[7]:5件 | F[7]:5件 | F[7]:5件 | F[7]:5件 | F[7]:5件 | F[7]:5件 | F[7]:5件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @mapping 同一 Class の共有 | N[6,7]:2件 | N[6,7]:2件 | N[6,7]:2件 | N[6,7]:2件 | N[6,7]:2件 | N[6,7]:2件 | N[6,7]:2件 | N[6,7]:2件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @mapping(Class)（透明な和型/単一 mapped alias） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @mapping(Parent.Child)（dotted） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @skip | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @enum | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @commonField | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @typeof | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @leftAssoc | Tt[13]:2件 | Tt[13]:2件 | Tt[13]:2件 | Tt[13]:2件 | Tt[13]:2件 | Tt[13]:2件 | Tt[13]:2件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | A:24件付随probe |
| @rightAssoc | Tt[14]:2件 | Tt[14]:2件 | Tt[14]:2件 | Tt[14]:2件 | Tt[14]:2件 | Tt[14]:2件 | Tt[14]:2件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | RA:28件付随probe |
| @precedence | Tt[13,14]:4件 | Tt[13,14]:4件 | Tt[13,14]:4件 | Tt[13,14]:4件 | Tt[13,14]:4件 | Tt[13,14]:4件 | Tt[13,14]:4件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | A:24件/2variants |
| @longestChoice（同長を含む） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @predictiveChoice | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @whitespace | Tt:33件 | Tt:33件 | Tt:33件 | Tt:33件 | Tt:33件 | Tt:33件 | Tt:33件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @interleave | Tt[5,6]:2件 | Tt[5,6]:2件 | Tt[5,6]:2件 | Tt[5,6]:2件 | Tt[5,6]:2件 | Tt[5,6]:2件 | Tt[5,6]:2件 | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @comment（global 設定） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @scopeTree | S:14件 | S:14件 | S:14件 | S:14件 | S:14件 | S:14件 | S:14件 | N[0]:1件 | **oracle なし** | S:14件・O:9 run | S[0]:2件・O:9 run | S:9文法 |
| @declares | S:14件 | S:14件 | S:14件 | S:14件 | S:14件 | S:14件 | S:14件 | N[0]:1件 | **oracle なし** | S:14件・N[0]:1件 | S[0]:2件 | S:9文法 |
| @backref（scope あり） | S[0]:2件 | S[0]:2件 | S[0]:2件 | S[0]:2件 | S[0]:2件 | S[0]:2件 | S[0]:2件 | **oracle なし** | **oracle なし** | S[0]:2件 | S[0]:2件 | S[0]:1文法 |
| @backref（scope なし） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @recovery（SYNC/AUTO/SKIP） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @eval（自動/manual/未知 kind） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| @catalog | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | Cat:2件 |
| @import（単段/多段） | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| ERROR() | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** | **oracle なし** |
| 数値 capture（構文と変換の分離） | Nu:13件 | **oracle なし** | **oracle なし** | **oracle なし** | Nu:13件（Java int） | Nu:13件（言語別） | **oracle なし** | **oracle なし** | Nu/6–12:7件 | **oracle なし** | **oracle なし** | **oracle なし** |

細部：optional の `?` は `P[28]:2件`、bounded の停止規則は `P[32–36]:5件`、`{n,}` は K の `unbounded` variant も参照する。SC:94件・PA:69件・M/MD/MS:49件は node/text 混合・alias・同値の別出現を補う。
A/RA付随probeはRustのLeft/Right metadataを独立assertする（`associative/probe.rs.txt:29`、`right-associative/probe.rs.txt:29`）。Aの順序/levelは `T/RustConformanceTest.java:399` に固定値がある。JSON loader はこれらのassertをmetadata期待値へ移していないため、実行harness側へ移植が必要。
A:24件・RA:28件は評価値を使った結合性の oracle（例 `A/1` の5、`RA/2` の512）を持つが、`@eval` 注釈の oracle ではない。`T/RustConformanceTest.java:317,411` は手書き意味処理で評価する。
Tt の javaStyle コメントは global `@comment` 設定の証拠にならない（`U/docs/rule-trivia.md:45`）。Cat は構文入力の解析ではなく静的 metadata の順序・重複除去・context 原文を検証する（`T/RustCatalogConformanceTest.java:48`）。
`QuotedParser` 自体は quote parser 引数を要する基底実装（`C/parser/elementary/QuotedParser.java:41`）。DoubleQuotedParser の corpus だけで任意の quote 設定まで保証しない。

## 2. 矛盾・言語差・仕様不足（15件）

「正」は既存 corpus がある範囲の互換判断。oracle のない事項は実装を暫定参照とし、拡張や不具合修正を既存互換と混同しない。C04/C05 は明示的な言語別契約、C14 の実行結果は未確認。

| ID | 仕様・設計の記述 | 実装／corpus の観測根拠 | ubnfc が正とするもの |
|---|---|---|---|
| C01 | `specs/ubnf-syntax.md:796` は WordParser が識別子を横取りしないと説明 | `C/parser/elementary/WordParser.java:67` は固定 cp 長の接頭辞比較のみ。`J/ParserRuleEmitter.java:1184` も WordParser。複数文字 keyword 境界の専用 corpus はなし | 境界なし（`v1:68`）。`letter` の先頭 `let` を阻止する規則を追加しない |
| C02 | `U/docs/ubnf-guide.md:311` は common field を default method と説明 | `J/ASTGenerator.java:276,297` は本体なし `Type field();` を中間 interface に生成。独立 corpus なし | 抽象 accessor（`v1:165`）。注釈位置も variant の例を鵜呑みにせず中間 mapping の実装に合わせる |
| C03 | `U/docs/ubnf-guide.md:274` の line 値は設定可能に見える。`v1:91` は block open/close にも言及 | `J/ParserGenerator.java:277` は `line` キーの存在だけを検査し `CPPComment` を追加。`:298` の block は固定 `BlockComment` で、ここに独自 delimiter の読取はない。corpus なし | Java の line 値無視を暫定参照。独自 block delimiter 対応は確認できず、v1 の実装事実として扱わない。旧 Rust は block setting を拒否（`J/rust/RustGrammarLowering.java:46`） |
| C04 | 共通 AST という読み方と Java/Rust の値型は一致しない（`specs/generators.md:104` は差を明記） | Nu/0–5 は Java int、Nu/6–12 は NumberFormatException。Rust は全13字句を保持（`T/NumericCaptureRuntimeTest.java:31,55`） | 構文13件成功、Java mapping 7件失敗、Rust13件成功を別契約として維持（`v1:29,98`） |
| C05 | 単一の診断 offset を両言語で共有できるとは限らない（`v1:129` は基準未確定） | D/7=`neg(1e+)`: Java7/Rust5。`T/RustConformanceTest.java:758` は別々に比較 | D の言語別値と kind を正とし統一しない。syntax/trailing の算出規則は §5 |
| C06 | span/cursor は cp（`v1:105`） | `C/parser/combinator/SyncPointRecoveryParser.java:125` は UTF-16 `indexOf + length` を `CodePointLength` に渡す。corpus なし | cp 契約を正とし、非BMP誤換算の互換再現要否は未確定。既存の不具合を独立 oracle と呼ばない |
| C07 | `specs/ubnf-syntax.md:418` の「ちょうど n 回」だけでは空要素・上限0を説明できない | P[32]/0 は EMPTY{2,2} を拒否、P[33]/0 は {1,2} を受理、P[35,36]/0 は {0,0} を拒否。`C/parser/combinator/Occurs.java:32,60,66,71` は子を試した後に回数・進捗・上限判定 | min/max の算術的展開でなく実際のループ順（`v1:75`）。空成功の1周も数える |
| C08 | `specs/ubnf-syntax.md:194` の EMPTY はゼロ幅だが matched の変化を記述しない | P[23]/1=`ab` は `[true,0,1]`。`C/parser/elementary/EmptyParser.java:25` は Optional(MatchOnly(ANY)) 相当 | ゼロ「消費」であり matched 不変ではない。EMPTY と純粋な空式を同一化しない（`R:1627` にも区別） |
| C09 | `specs/ubnf-syntax.md:131,204,218` は空の pattern/terminator の境界を規定しない | P[14]:2件は LOOKAHEAD('') 拒否、P[15]/0 は負の空先読み成功、P[13]/0 は UNTIL('') が `abc` 全消費。`WordParser.java:70` と `AbstractTokenParser.java:44`（ともに C/parser/elementary）、`R:1635,1657` | 空パターンは空リテラルの自明な成功へ簡約せず、P の結果を維持 |
| C10 | `U/docs/ubnf-guide.md:364` の例は `@interleave(profile=default)` | `J/GrammarValidator.java:855` は javaStyle/commentsAndSpaces だけ許可。Tt[5,6] はこの2つを検証し、default 拒否の JSON はなし | default は生成前拒否。ルール whitespace が interleave より優先（Tt[7,8]、`U/docs/rule-trivia.md:15`） |
| C11 | `specs/scope-tree.md:45` は dynamic を実行時ネストと説明 | S[1]/shadow-then-resolve-outer-after-leave は解析時の enter/leave/lookup。`U/docs/generated-scope-effects.md:21` は両 mode とも解析スタック、評価時環境は未実装と明記 | 解析時 scope 履歴と mode metadata を保存。評価時 dynamic scope の実装を推定しない |
| C12 | `specs/token-resolution.md:75` は NumberParser の例を「符号なし整数」とする | `C/parser/elementary/NumberParser.java:31` は符号・小数・指数を許す。Nu/1=`-17`、Nu/6=`1.5`、Nu/9=`1e2` は構文成功 | alias 名で受理範囲を狭めない。字句は NumberParser、数値変換は mapping 段階 |
| C13 | `U/docs/ubnf-guide.md:224,429`、`specs/ubnf-syntax.md:553` の右結合例は正規形・必須 capture 等を満たさない | `J/GrammarValidator.java:503,531,538` は mapping・left/op/right・repeat・Base { Op Self } を要求。RA:28件の Power.ubnf は正規形 | 検証済み正規形を正とする（`v1:144`）。例の直接右再帰式が注釈付きでもそのまま通るとはしない |
| C14 | `U/docs/ubnf-guide.md:467` の alias 利用、`v1:169` の統合表から多段も完全解決済みとは断定できない | `U/unlaxer-dsl/src/main/java/org/unlaxer/dsl/bootstrap/UBNFMapper.java:159,175,211` は再帰読込後に未統合 GrammarDecl を保存し、直接 import の名前だけ prefix、body は据置き。多段実行 oracle なし | 推移的解決の完成は未確認。ubnfc の qualifiedSymbolId 方針は v1 を採るが、既存との一致は多段テスト追加後に判定 |

| C15 | `v1:105` は位置をcp、`docs/design/02-facts-tokens-bench-corpus.md:23` は REGEX を lookingAt と規定 | `U/unlaxer-dsl/src/main/java/org/unlaxer/dsl/runtime/RegexTokenParser.java:38,45,50,54` はcpのconsumedをUTF-16 substringの添字に使い、matcherのUTF-16長をCodePointLengthに渡す。非BMPの前置/マッチで単位が混在。`corpus/oracle/regex-emoji.json`・`regex-prefix.json` に実測あり | 決定: D-017 に従い code point 意味論。既存の UTF-16 混在は再現しない。`regex-emoji/1`・`regex-prefix/1,2` の `parsedStatus` も `follow-spec` として台帳に登録（受理・AST/capture・cursor・診断は登録済み） |

## 3. ケース群の分類と loader の読み方

(a) 正常受理、(b) 意図した全入力拒否、(c) 構文成功・Java mapping 失敗を行単位で分類する。(b) には prefix 成功後の trailing input も含め、構文解析そのものの失敗と区別する。
(d) は独立軸：値・成否・offset が明示的に言語別の14行（Nu13+D/7）。a/b/c に重ねて数える。Java/Rust の field 型表記（List/Vec 等）の違いはこの14件へ加算しない。
複数 `modes` の行は「少なくとも1 variant で受理」を (a) とし、全 variant 拒否を (b) とする。variant 展開で703行を水増ししない。

| corpus | 全行 | (a) | (b) | (c) | (d) 重複 | 分類外（根拠なし／非構文） |
|---|---:|---:|---:|---:|---:|---|
| N | 10 | 10 | 0 | 0 | 0 | 0 |
| Tt | 33 | 17 | 16 | 0 | 0 | 0 |
| S | 14 | 12 | 2 | 0 | 0 | 0 |
| P | 109 | 56 | 53 | 0 | 0 | 0 |
| L | 78 | 48 | 30 | 0 | 0 | 0 |
| SC | 94 | 59 | 35 | 0 | 0 | 0 |
| PA | 69 | 35 | 34 | 0 | 0 | 0 |
| F | 73 | 34 | 39 | 0 | 0 | 0 |
| Q | 39 | 27 | 12 | 0 | 0 | 0 |
| M | 23 | 13 | 10 | 0 | 0 | 0 |
| MD | 15 | 9 | 6 | 0 | 0 | 0 |
| MS | 11 | 5 | 6 | 0 | 0 | 0 |
| E | 37 | 0 | 0 | 0 | 0 | 37（入力のみ・期待値未確定） |
| D | 8 | 0 | 8 | 0 | 1 | 0 |
| A | 24 | 14 | 10 | 0 | 0 | 0 |
| RA | 28 | 16 | 12 | 0 | 0 | 0 |
| K | 14 | 8 | 6 | 0 | 0 | 0 |
| Nu | 13 | 6 | 0 | 7 | 13 | 0 |
| O | 9 | 0 | 0 | 0 | 0 | 9（scope 操作） |
| Cat | 2 | 0 | 0 | 0 | 0 | 2（静的 metadata） |
| 合計 | 703 | 369 | 279 | 7 | 14 | 48 |

`369+279+7+37+9+2=703`。Rust 側なら Nu の7行を a に移し、a=376/b=279/c=0。読み込める703件のうち固定 oracle は666件、そのうち解析入力は655件。系統別は AST57/token187/rule324/external124/operations11（`H/lib.rs:25`、既存 list の出力と一致）。
`modes` で受理/拒否が分かれる行は17行（M5/RA5/K7）。行×mode では M=21受理/25拒否、RA=49/63、K=20/50。K の mode 別受理は zero6/plus4/bounded3/unbounded4/separated3。これは上表とは別集計。

- `prefix=[accepted,consumed,matched]` は cp。明示 prefix は188行、うち21行は prefix 成功でも全入力拒否（P5/L6/Tt5/S1/F2/Q2）。N10行の独立 cursor 規則は別枠（`H/corpus/mod.rs:17`、`T/RustConformanceTest.java:553`）。
- `value` 欠落は不受理、明示 `null` は optional 不在で受理。L[3]/0・L[9]/0・Q[4]/absent の3行が該当。空文字・空配列・数値0も受理値（`H/corpus/token.rs:6`、`H/corpus/mod.rs:160`）。
- Nu の `javaInt:null` は7行すべて明示値であり欠落なし。構文 `n<字句>;` は受理、Java だけ mapping 例外、Rust は元の `+`・先頭0・指数を保持（`H/corpus/external.rs:90`、`T/NumericCaptureRuntimeTest.java:39`）。
- `modes` は backend 名ではなく文法 variant の受理集合。欠落なら全 variant、`[]` なら受理なし。`value` の存在と AND を取る（`H/corpus/rule.rs:21,32`）。
- `texts/nodes` は `[start,end,text]` の出現順配列。canonical AST のノード形ではなく、同値の別出現を潰さず比較する。`value` にある L:/T:/O: 等も原文保持（`H/corpus/rule.rs:1,17`、`T/SemanticCardinalityConformance.java:161`）。
- N は `accepted` 省略でも全件受理。N の `declarations`/`valueSpans` 欠落は空列の期待値。S の `parentRollback` はキーの存在で外側 rollback 検証を起動（`H/corpus/ast.rs:19,28,35`、`T/NestedCaptureConformanceTest.java:125,145`、`T/ScopeAnnotationConformanceTest.java:163`）。
- fixture の `whitespace` は真偽値よりキーの存在が意味を持つ。現データは true のみ。Cat の非標準 `\'` だけを正規化し context を trim/再unescape しない（`H/corpus/token.rs:37`、`H/corpus/mod.rs:129`、`H/corpus/operations.rs:116`）。
- E37行は `Expectation::Raw`：Java 実行結果との比較材料であって独立の受理・AST 値ではない（`H/corpus/external.rs:59`、`T/RustConformanceTest.java:820`）。結果を推定して a/b に配分しない。
- `schema/parser-ir` の38 JSON と `golden/cli` の5 JSON も確認したが、703件の loader 対象外（`H/lib.rs:21`）。IR 構造検証・CLI 出力 fixture を生成パーサの runtime oracle と数えない。生成スナップショットも互換根拠にしない（`docs/design/02-facts-tokens-bench-corpus.md:56`）。

## 4. oracle なしの機能と追加テスト案

行列58行中、全12観測面が **oracle なし** の機能は19行。下表は各行に対応する提案で、既存 oracle の実測結果ではない。部分的にしか埋まらない観測面はその外にも残る。

| 機能 | 追加する入力 → 期待値の形（提案） |
|---|---|
| DigitParser | `T @v EOF` に `7`/`73`/`７` → 受理/拒否/拒否、Java値7・cp span `[0,1]`。単体テスト `T/NumericCaptureRuntimeTest.java:132` を JSON 化し Rust の型契約も固定 |
| その他 posix token | AlphabetParser に `A`/`1`/`あ` → 受理/拒否/拒否、`[ok,c,m]`、成功span `[0,1]`（`C/parser/posix/AlphabetParser.java:15`） |
| CI | `CI('let')` に `LeT!` → prefix `[true,3,3]`・全入力拒否、EOF 付き `LeT` は受理。非ASCII case fold と raw AST 字句は Java oracle を採取 |
| REGEX | `[a-z]+` に `ab!` → prefix長2、`a*` に空入力 → ゼロ幅拒否。非正則 `(a+)\1` に `aaaa` → Java受理/Rust生成時拒否（`v1:70` の目標契約）。前置`😀`の後の`a`とREGEX自体の`😀`一致もcp位置で固定（C15） |
| 汎用 Extern | scanner が `😀` を C+1/M+2 で返す入力 `😀x` → cp の二重 cursor・診断・値spanを保持。状態依存 scanner は memo 対象外、失敗枝の効果は rollback |
| 透明な @mapping 和型 | `@mapping(Node) Root ::= Leaf; @mapping(Item) Leaf ::= 'a';` に `a` → interface Node の具象 Item、span `[0,1]`。2 variant と多段 alias も同形式 |
| dotted @mapping | `Node.A`/`Node.B` の `'a'`/`'b'` → 具象 variant 型・値・span、evaluator callback 名を固定（`specs/ubnf-syntax.md:532`） |
| @skip | 区切り Comma に @skip、`a,a` → 構文上の消費3、AST item2個、Comma nodeなし・各item spanを確認 |
| @enum | `'on'`/`'off'` → ON/OFF の宣言順・名前・literal対応 metadata、`other` → 拒否。重複生成名は生成時診断を固定 |
| @commonField | 共通 name を持つ2 variant → Java interface accessor が abstract、各値を返す。欠損/異型 variant は生成時エラーとする候補を実装照合 |
| @typeof | 2 capture の具象型が同じ/異なる入力 → どちらも構文成功、同型はAST、異型は mapping 失敗。null/cardinality 比較は別行で固定（`J/MapperRuleEmitter.java:844`） |
| @longestChoice | `'a'` と `'ab'` に `ab` → 後者。別 AST の `'a'` と `'a'` に `a` → 宣言順の前者。同長枝で scope 効果・診断が混ざらないことも期待値化 |
| @predictiveChoice | `'a' 'x'` と `'b'` に `az`/`b` → 通常Choiceと同じ拒否/受理・両cursor・診断。全候補失敗時の全分岐再試行と nullable/Extern の保守的候補を検査 |
| @comment | `line:'#'`、本体 `'a' 'b'` に `a//x\nb`/`a#x\nb` → Java は受理/拒否（C03の実装候補）。Rust の旧拒否と ubnfc 拡張を分け、block-only 設定も追加 |
| scope なし @backref | 先行/後続同名 capture の `x:x`/`x:y` → Java の一致/不一致と ERROR severity、構文成否を実測固定。旧Rustは生成拒否（`J/rust/RustGrammarLowering.java:138`） |
| @recovery | 子rule SYNC `;` に `bad;ok;` → 同期記号込みの回復spanと次node、`bad` → 同期なし失敗。AUTO/SKIP・root/参照位置・`😀;x` のcp長2対旧誤換算も別oracle（非BMP採否未確定） |
| @eval | literal/variable_ref/binary_arithmetic/conditional/passthrough を固定入力で評価し値・callback順を保存。manual/未知kindの未実装callbackはcompile拒否、自動指定した和型は生成拒否（`J/EvaluatorGenerator.java:31,103`） |
| @import | A→B→C、各段同名token/rule、入力 `c` → 統合 qualifiedSymbolId とC由来AST/spanを期待。循環・欠落は明示診断、多重aliasは衝突なし（v1の目標。旧多段動作は未確認） |
| ERROR() | `'a'` と `ERROR('need a')` の選択に `x` → syntax失敗、cp offset0、expectedにメッセージ。成功枝や負先読み内での診断抑止も別ケース化 |

全欠落19件とは別の優先追加：keyword `'let'` に `letter` → prefix `[true,3,3]`／全入力拒否、数値変換が失敗枝では起きないこと、診断expectedの具体集合・deepest rule、memo OFF/ON のscope/診断一致、未capture字句出現。
**UNTIL の EOF 到達は既に oracle あり**：P[12]/0=`abc` → 値`abc`・`[true,3,3]`、同/1空入力 → `[true,0,0]`。追加は `😀x`（終端なし）→ `[true,2,2]`、`😀#tail` → `[true,1,2]`・全入力拒否（後者は同/2に既存）とし、EOF成功を拒否へ変えない。

## 5. D-011：診断 offset の独立確認

共通テストは `T/RustConformanceTest.java:724`。Java は Mapper.diagnose、Rust は parse_tree_detailed の結果について、kind・各backend offset・expected非空を検証し、trailingだけ expected も一致を要求（`:755–765`）。deepest rule の固定期待値はDにない。
Java の syntax は `ParseFailureDiagnostics.getFarthestOffset()`（`C/context/ParseFailureDiagnostics.java:194`）。DTO 自体で算出せず、`C/context/ParseContext.java:900` が失敗候補の最遠値（候補なしなら最大到達）を設定する。
失敗候補は frame の **max(過去最大consumed,過去最大matched)**（同`:1042,1315,1349`）。cp の start/end/consume/matchOnly を記録し（同`:832,845,995`）、rollback後の最終cursorに置き換えない。したがって「常にconsumed」でも「最終matched」でもない。
Java の trailing は成功 token の消費長を `source.codePointCount` で変換（`J/MapperGenerator.java:196–204`）。Rust は trailing に position のcp変換、syntax に記録済み farthest のcp変換を使う（`R:1003–1045,1444`）。
Rust `fail()` は consumed側の `position` を記録（`R:1466`）。ただし `JavaLookahead` の失敗は `fail_at(matched_position)`（`R:1644`）なので、Rust全体をconsumedのみと一般化してはいけない。D の文法にはこの明示先読みはない（`evolution/3/Evolution.ubnf:1`）。

| D ケース id / 入力 | kind | javaOffset の意味 | rustOffset の意味 | 単位・根拠 |
|---|---|---|---|---|
| diagnostics/0 `neg(/*😀*/)` | syntax | 9：失敗frame最大 C/M、`)` 手前 | 9：失敗記録position、`)` 手前 | 両方cp。D/0、Java max規則／`R:2239` |
| diagnostics/1 `neg(/*😀*/2` | syntax | 10：失敗frame最大 C/M、EOF | 10：失敗記録position、EOF | 両方cp。D/1、閉じ括弧失敗／`R:1466` |
| diagnostics/2 `1/*😀*/x` | trailing_input | 6：成功prefixのconsumed終端 | 6：成功prefixのposition終端 | 両方cp。D/2、`J/MapperGenerator.java:201`／`R:1014` |
| diagnostics/3 `1+` | trailing_input | 1：成功prefixのconsumed終端 | 1：成功prefixのposition終端 | 両方cp。D/3。投機的な最遠失敗とは別値 |
| diagnostics/4 `if(1,2,)` | syntax | 7：失敗frame最大 C/M、第3引数開始 | 7：失敗記録position、第3引数開始 | 両方cp。D/4、Java max規則／`R:2239` |
| diagnostics/5 `neg(-)` | syntax | 5：符号後に到達した失敗frame最大 C/M | 5：number が符号を進めたpositionで失敗 | 両方cp。D/5、`C/parser/elementary/NumberParser.java:32`／`R:2231–2242` |
| diagnostics/6 `neg(.)` | syntax | 5：小数点後に到達した失敗frame最大 C/M | 5：number が小数点を進めたpositionで失敗 | 両方cp。D/6、NumberParser.java:50／`R:2235–2242` |
| diagnostics/7 `neg(1e+)` | syntax | 7：指数のdigit失敗をrollback後も保持 | 5：不完全指数を戻した後、`e`位置の失敗記録 | 両方cp。D/7、`C/parser/elementary/ExponentParser.java:18`／`R:2244–2255` |

D/0–2 の非BMP入力と変換実装が cp 単位を裏付ける。Java各行の `farthestConsumedOffset` / `farthestMatchedOffset` の**個別実測値は未確認**（既存テストも比較しない）。数字の一致だけから片方専用の基準を決めない。
D-011 の結論：**kind別・backend別に契約化**し、Java syntax の max(C履歴,M履歴)、Rust の明示failure位置、両言語 trailing の確定consumedを保持する。Java7/Rust5の差は単位変換の差ではなく不完全指数の失敗記録の差である。
不足する識別実験：`LOOKAHEAD('a')` の後で `'x'` を要求し入力`a`を与え、最終cursor・両farthest・公開offsetを別々に保存する。期待値は上の実装規則から採取し、consumed≠matched の失敗例として固定する（D8件だけでは未識別）。
