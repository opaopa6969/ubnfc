# 仕様由来オラクル — 規範文から期待値を導き、4 実装に当てる（2026-09-24）

issue #5。`docs/reports/2026-09-23-parser-feature-correctness-comparison.md` Part 2 の結論は
**「10 実装のうち独立した仕様から期待値を導いているものはゼロ」**だった。ubnfc の 1,425 件も
出どころは旧 Java 実装の観測で、`corpus/oracle/README.md` 自身が「疑わしい値を望ましい挙動へ
修正していない」と書いている。つまり ubnfc の「正しさ」は**「旧実装と同じ」までしか主張していない**。

UBNF 文法族には規範文書 `unlaxer-parser/unlaxer-dsl/specs/{ubnf-syntax,annotations,validation}.md`
がある。本報告はそこから**実装を一切見ずに**期待値を作り、4 実装へ当てた結果である。

## 0. 何をしたか

- `corpus/spec/ubnf/*.json` — 10 ファイル **129 ケース**。各ケースは引用元
  （ファイル・見出し行・逐語の一文）を持ち、期待値はその文だけから決めた。
- `corpus/spec/p4/*.json` — 4 ファイル **32 ケース**（tinyexpression の規範文書から）。
- `crates/ubnfc-harness/tests/spec_corpus.rs` — 引用の見出しが実在し、引用文が仕様ファイルに
  逐語で含まれることを `cargo test` で検査する（sibling repo が無ければ照合だけ省略）。
- `examples/ubnf-crosscheck/scripts/run-spec.sh` — 129 ケースを 4 実装へ流す。
  `examples/p4-crosscheck/scripts/run-spec.sh` — 32 ケースを P4 2 実装へ流す。
- 証拠: `examples/ubnf-crosscheck/evidence/spec-report.md`、
  `examples/p4-crosscheck/evidence/spec-report.md`。

**規則**: 実装が期待と食い違ったら、まず仕様の読み方を疑う。期待値を実装に合わせて
書き換えた時点でこの corpus の意味が無くなる。仕様が沈黙・矛盾している場合は推測せず
`undetermined` として問いを残した。

## 1. ケース数

### 節（仕様の見出し）ごと

| ファイル | 仕様の節 | ケース |
|---|---|---:|
| `annotations.json` | アノテーション（`ubnf-syntax.md` の形式文法 + `annotations.md` の契約 + `validation.md` のコード表） | 32 |
| `formal-grammar.json` | 形式的な文法定義 | 23 |
| `quantifiers.json` | 量指定子とセパレータ | 16 |
| `char-range.json` | `CHAR_RANGE('min','max')` | 11 |
| `global-settings.json` | グローバル設定 | 10 |
| `rule-declaration.json` | ルール宣言 | 10 |
| `token-declarations.json` | トークン宣言（token） | 10 |
| `literals-escapes.json` | ルール本体の書き方 / リテラル | 9 |
| `comments.json` | コメント | 5 |
| `reserved-words.json` | 予約語 | 3 |
| **合計** | | **129** |

### 期待値の種類と観測層

| 期待 | 件数 | | scope | 件数 | 駆動できる実装 |
|---|---:|---|---|---:|---|
| `accept` | 56 | | `parse` | 112 | 4 実装すべて |
| `reject` | 45 | | `validate` | 14 | ubnfc `validate` と Java `GrammarValidator` |
| `diagnostic` | 14 | | `runtime` | 3 | 無し（仕様文だけ残した） |
| `undetermined` | 14 | | | | |

`accept` のうち 12 件は受理だけでなく**構文木の形**も縛る。内訳は正準 AST の部分文字列を
指定するもの 10 件（`T Root` が 2 要素であること、`'\n'` が実際の改行 1 文字になること等）と、
仕様が等価と述べた 2 綴りの構文木一致を要求するもの 2 件（`X?` ≡ `[ X ]`、`X*` ≡ `{ X }`）。

## 2. 判定（129 ケース × 4 実装）

| 実装 | 一致 | 違反 | 別コード | 未定 | 駆動不能 |
|---|---:|---:|---:|---:|---:|
| A ubnfc 手書き frontend | 94 | **18** | 3 | 11 | 3 |
| B ubnfc 自己ホスト生成 | 81 | **20** | 0 | 11 | 17 |
| C unlaxer-ubnf（手書き Rust） | 100 | **1** | 0 | 11 | 17 |
| D Java bootstrap | 109 | **3** | 3 | 11 | 3 |

- 「別コード」= 同じ違反を検出しているが `validation.md` と違うコード名を出した場合（§4）。
- B と C の「駆動不能」が 17 なのは、独立した validator を持たないため `validate` scope の
  14 件を駆動できないから（+ `runtime` 3 件）。
- 「未定」列が 11 なのは、`undetermined` 14 件のうち 3 件が `runtime` scope で
  「駆動不能」に入るため。
- `旧実装と同じ` を目標にしてきた ubnfc が **18 件、旧実装（D）が 3 件**。
  この領域では **旧実装のほうが仕様に近い**。旧実装を基準にしている限り、
  ubnfc が広すぎるのか旧実装が狭すぎるのかは判定できず、既知乖離として並ぶだけになる。

節ごとの違反数:

| 節 | ケース | A | B | C | D |
|---|---:|---:|---:|---:|---:|
| CHAR_RANGE | 11 | 7 | 7 | 0 | 0 |
| 形式的な文法定義 | 23 | 9 | 9 | 1 | 2 |
| 量指定子とセパレータ | 16 | 2 | 2 | 0 | 0 |
| コメント | 5 | 0 | 2 | 0 | 1 |
| 他 6 節 | 74 | 0 | 0 | 0 | 0 |

## 3. 実装別の違反一覧（最小入力つき）

### A ubnfc 手書き frontend — 18 件

**(a) CHAR_RANGE の境界検査が無い — 7 件**（引用: `ubnf-syntax.md` 「`CHAR_RANGE('min','max')` — 文字の範囲」。
「各境界はエスケープ展開後に1個の非surrogate BMP文字で、`min <= max` でなければならない。
空文字・複数文字・surrogate・補助文字（例 `😀`）・逆順の範囲は `IllegalArgumentException` で
明示的に拒否し、先頭文字へ切り捨てない。」）

| ケース | 入力 | A の実測 | `ubnfc validate` |
|---|---|---|---|
| `char-range/empty-min` | `grammar G { token L=CHAR_RANGE('','a') R ::= L; }` | 受理 | **panic exit 101** |
| `char-range/empty-max` | `… CHAR_RANGE('a','') …` | 受理 | **panic exit 101** |
| `char-range/multi-char-min` | `… CHAR_RANGE('ab','z') …` | 受理 | exit 0（先頭 1 文字へ黙って切り捨て） |
| `char-range/multi-char-max` | `… CHAR_RANGE('a','yz') …` | 受理 | exit 0 |
| `char-range/supplementary` | `… CHAR_RANGE('😀','😇') …` | 受理 | exit 0 |
| `char-range/reversed` | `… CHAR_RANGE('z','a') …` | 受理 | exit 0 |
| `char-range/escaped-bounds-reversed` | `… CHAR_RANGE('\n','\t') …` | 受理 | exit 0 |

C と D は 7 件すべてを拒否する。**この 7 件は issue #1（malformed-declaration diagnostics）が
扱う範囲なので、本作業では front を触っていない**（同じファイルを同時に書き換えないため）。
`corpus/spec/ubnf/char-range.json` が受け入れ条件としてそのまま使える。
`CHAR_RANGE('\t','\n')`（展開後 0x09 <= 0x0A）は仕様どおり受理されるので、
「エスケープ展開後に判定する」側の期待も同じファイルに入れてある。

**(b) 空構文 — 9 件**（引用: `UBNFFile ::= { '//' LINE_COMMENT } GrammarDecl+`、
`SequenceBody ::= AnnotatedElement+`）

`(空ファイル)` / `// just a comment` / `R ::= ;` / `R ::= 'a' | ;` / `R ::= | 'a';` /
`R ::= ();` / `R ::= [ ];` / `R ::= { };` / `ERROR();` の 9 件。C と D は 9 件とも拒否。
→ **意図的。D-050 として記録した**（自己記述文法 `ubnf.ubnf` の `SequenceBody` が
`{ AnnotatedElement }` で、`crates/ubnfc-selfhost/tests/parity.rs:56-63` が受理入力として
明示列挙している。狭めると self-host の乖離方向が 9 件増え、実在文法の受理は 1 つも変わらない）。

**(c) 量指定子の併記 — 2 件**（引用: `AnnotatedElement ::= … AtomicElement [Quantifier | '%' AtomicElement] …`）

| ケース | 入力 | A/B | C/D |
|---|---|---|---|
| `quantifier/two-quantifiers` | `grammar G { token V=ANY R ::= V+{2,3}; }` | 受理 | 拒否 |
| `quantifier/quantifier-and-separator` | `grammar G { token V=ANY token S=ANY R ::= V+ % S; }` | 受理 | 拒否 |

→ **意図的な拡張。D-051 として記録した**。

### B ubnfc 自己ホスト生成 — 20 件

A の 18 件に**ブロックコメント 2 件**が加わる。

| ケース | 入力 | A | B | C | D |
|---|---|---|---|---|---|
| `comment/block-rejected` | `grammar G { /* no */ R ::= 'x'; }` | 拒否 | **受理** | 拒否 | 拒否 |
| `comment/block-in-body-rejected` | `grammar G { R ::= /* no */ 'x'; }` | 拒否 | **受理** | 拒否 | 拒否 |

`ubnf-syntax.md`「ルール宣言」の「UBNF ファイル自体のコメント構文は `//` のみで、
`/* ... */` は未対応です。」に反する。2026-09-23 の比較報告が見つけた
**7 件目の self-host 乖離**（`ubnf.ubnf:18` の `@whitespace: javaStyle` が
`lower/trivia.rs` で block comment を delimiter に入れる）を、今回は**仕様の側から**
独立に再現した。B は A の 9 件の空構文と 7 件の CHAR_RANGE も同じく受理する。

### C unlaxer-ubnf（手書き Rust） — 1 件

| ケース | 入力 | 期待 | C の実測 |
|---|---|---|---|
| `grammar/empty-block` | `grammar G { }` | accept | 拒否「grammar requires at least one rule」 |

`GrammarDecl ::= 'grammar' IDENTIFIER '{' { GlobalSetting } { TokenDecl } { RuleDecl } '}'` は
3 つの繰り返しをすべて 0 回にできる。**仕様に無い追加制約**。ubnfc（A/B）が正しい。

### D Java bootstrap — 3 件

| ケース | 入力 | 期待 | D の実測 |
|---|---|---|---|
| `file/trailing-garbage-rejected` | `grammar G { R ::= 'x'; } garbage` | reject | 受理（prefix 成功） |
| `comment/block-unclosed-rejected` | `grammar G { R ::= 'x'; } /* dangling` | reject | 受理（同じ prefix 成功） |
| `grammar/empty-block` | `grammar G { }` | accept | `IllegalArgumentException` |

前 2 件は `rust/unlaxer-ubnf/README.md` に記録済みの prefix 受理欠陥、3 件目は C と同じ
「空 grammar を拒否する」追加制約。

## 4. 仕様書そのものの欠陥 — 存在しない診断コード

`validation.md` のコード表に載っている 3 つのコードは、**ubnfc にも旧 Java `GrammarValidator`
にも存在しない**。両実装が同じ別名を出す。

| `validation.md` が書くコード | 行 | 実際に両実装が出すコード |
|---|---:|---|
| `E-MAPPING-EXTRA-CAPTURE` | 86 | `E-MAPPING-UNLISTED-CAPTURE` |
| `E-ASSOC-WITHOUT-PRECEDENCE` | 94 | `E-ASSOC-NO-PRECEDENCE` |
| `E-PRECEDENCE-WITHOUT-ASSOC` | 113 | `E-PRECEDENCE-NO-ASSOC` |

検出そのものは両実装で一致しているので、**コード名は変えない**（変えると corpus と
生成物の綴りが動く一方、得られるのは仕様書の誤記への追従だけ）。上流 `validation.md` の
修正待ちとして compat-matrix C16 に登録した。

残る 11 の診断コード（`E-MAPPING-MISSING-CAPTURE`、`E-MAPPING-DUPLICATE-PARAM`、
`E-ASSOC-BOTH`、`E-WHITESPACE-GLOBAL-STYLE` / `-GLOBAL-DUPLICATE` / `-RULE-STYLE` /
`-RULE-DUPLICATE`、`E-PRECEDENCE-DUPLICATE`、`E-ANNOTATION-INTERLEAVE-PROFILE`、
`E-ANNOTATION-SCOPETREE-MODE`）は ubnfc・Java とも仕様どおりに出る。

## 5. 未定のまま残したもの — 14 件

仕様が沈黙しているか、仕様内で矛盾しているもの。**推測で埋めていない。**

| ケース | 何が決まらないか |
|---|---|
| `rule/lowercase-name` | 本文は「左辺はルール名（大文字始まりの識別子）」、形式文法は `IDENTIFIER` で大小を制約しない。`r ::= 'x';` を拒否すべきか警告か許容か。4 実装とも受理 |
| `rule/dotted-reference-three-segments` | `a.b.Value` の namespace が `a.b` か `a` か。「ドットは外側の文法が処理する」としか書かれていない。A/C/D は受理、B は拒否 |
| `escape/unknown-sequence` | 表は `\n \t \r \\ \'` の 5 つだけ。`'\q'` を拒否するのか、バックスラッシュごと残すのか、`q` にするのか。4 実装とも受理 |
| `quantifier/reversed-bounds` | `V{9,2}`。`Quantifier` の production は n <= m を要求していない。CHAR_RANGE と違い逆順反復の規範文が無い。4 実装とも構文としては受理 |
| `quantifier/huge-bound` | `'x'{2147483648}`。INTEGER の表現範囲が書かれていない。A/B 受理、C/D 拒否 |
| `error/element-needs-string` | `ERROR(msg)` は `ErrorElement` にならないが、同じ形式文法の `RuleRefElement` + `GroupElement` で導出できる。予約語 `ERROR` を規則参照位置で使えるかが未規定。4 実装とも受理 |
| `annotation/root-two-rules` | 「最大1つのルールに付与すべき（SHOULD）」。違反時に受理か警告かエラーかが定まらない |
| `spec-contradiction/minilang-leftassoc` | `annotations.md:97` は `@leftAssoc` に `@precedence` を MUST とするが、`ubnf-syntax.md:659` の完全版 MiniLang 文法自身が 3 箇所で違反している。**仕様内の矛盾** |
| `reserved/rule-named-token` / `reserved/rule-named-error` / `reserved/token-named-any` | 予約語一覧は「特別な意味を持つ単語」と述べるだけで、識別子として使えないとは書いていない |
| `char-range/inclusive-bounds` / `separator/expansion` / `error/hint-is-last-alternative` | 生成パーサの実行時挙動。UBNF を読む 4 実装では観測できない（`scope: runtime`） |

## 6. P4（tinyexpression）

**結論: P4 にも仕様相当の文書はある。ただし来歴は観測由来である。**

| 文書 | 行 | 性質 |
|---|---:|---|
| `tinyexpression/spec/SPEC.md` | 2194 | ステータス **Accepted**、12 章。MUST/SHOULD を伴う規範文が 41 箇所 |
| `tinyexpression/specs/language.md` | 273 | 「権威的定義」を名乗る（status: draft） |
| `tinyexpression/specs/p4-grammar.md` | 184 | P4 固有（status: draft）。ただし優先順位表が実在しない規則名を挙げている |
| `tinyexpression/docs/language-guide.md` | 407 | 言語リファレンス |

正直に書いておくべき留保が 3 つある。

1. **自己申告で実装由来**。`specs/language.md`「現在の制限事項」は
   「完全な形式文法はレガシーパーサーのソースコードに散在しており、この仕様は実装ベースの記述」
   と書いている。`SPEC.md` §2.1.2 も「完全なエスケープシーケンス仕様は実装ベース（公式形式定義は未確定）」。
   UBNF 側の `ubnf-syntax.md` のような「先に書かれた規範」ではない。
2. **文書間で矛盾している**。論理演算子の優先順位は `SPEC.md` §2.6 と `docs/language-guide.md` が
   `^` > `&` > `|`、`specs/language.md` が `&` > `^` > `|`。文法ファイルの `@precedence` は前者と一致する。
3. **式全体（Formula）の production が無い**。そのため「比較式単独が完結した式か」が決まらない。

それでも、**受理・拒否を独立に決められる文はある**ので小さな集合を分離した
（`corpus/spec/p4/*.json`、32 ケース）。

| 実装 | 一致 | 違反 | 未定 | 駆動不能 |
|---|---:|---:|---:|---:|
| R1 ubnfc p4-rust | 25 | **0** | 4 | 3 |
| R2 tinyexpression-rs | 25 | **0** | 4 | 3 |

**両実装とも判定可能な 25 件すべてで仕様と一致した**（数値・文字列・真偽値リテラル、
`$` 変数、`if`/`else` 必須、`if` とパーレンの間の空白・ブロックコメント、
式本文の `/* */`、`#` が式本文のコメントにならないこと）。
J1（ubnfc Java backend）と J2（旧 Java）は 1 入力あたり JVM 起動が要るのでここでは駆動していない
（`examples/p4-crosscheck/scripts/run.sh` の 16 fixture 比較を参照）。

未定 4 件: `42L` / `3.14f`（§2.1.1 の表は数値リテラルとして挙げるが、§2.17 は数値具体型の
サフィックスを HAND-ONLY に分類しており同じ文書内で食い違う。両実装とも拒否）、
`$age >= 18` 単独（Formula の production が無い。両実装とも拒否）、
論理演算子の優先順位（文書間の矛盾）。

**P4 の残りの期待値は観測由来のまま**である。`docs/feature-parity-diff.md`（PARITY 68 /
HAND-ONLY 30 / UBNF-ONLY 16 / PARTIAL 14）は仕様ではなく実装差分の棚卸しで、
`examples/p4-rust/tests/golden/` も生成物の固定である。

## 7. 対応

| 分類 | 件数 | 対応 |
|---|---:|---|
| 意図的（仕様より広い受理を維持） | 11 | **D-050**（空構文 9 件）、**D-051**（量指定子併記 2 件）。どちらも受理・AST・IR・corpus 不変の追認 |
| issue #1 が扱う範囲 | 7 | CHAR_RANGE の境界検査。`corpus/spec/ubnf/char-range.json` を受け入れ条件として使える。本作業では front を変更していない |
| 仕様書の誤り | 3 | 診断コード名（compat-matrix C16）。実装は変えない |
| 他実装の違反 | 4 | C 1 件・D 3 件。ubnfc 側の作業ではない（記録のみ） |

**ubnfc のコードは 1 行も変えていない。** 見つかった 18 件はすべて
「意図的（11）」「issue #1 の担当（7）」のどちらかで、本作業で直すべき残りは無い。
決定 2 件と compat-matrix の 5 行（C16〜C20）が成果物である。

## 8. この作業が示したこと

- **「旧実装と同じ」では仕様との距離が測れない。** 今回 ubnfc は 18 件、旧実装は 3 件
  違反した。この 18 件は「旧実装と食い違っている」ことは以前から見えていた
  （2026-09-23 の比較報告の受理不一致 21 件に含まれる）が、**どちらが仕様に沿っているかは
  問われていなかった**。仕様を当てて初めて「ubnfc が広すぎる 18 件」「旧実装が狭すぎる /
  緩すぎる 3 件」と向きが決まり、11 件を意図的な拡張として決定に落とせた。
  逆に `grammar G { }` のように **ubnfc が正しく他 2 実装が誤っている**ことも、
  旧実装を基準にしている限り出てこない。
- **仕様の側にも欠陥がある。** 存在しない診断コード 3 つ、自分の例が自分の MUST を破る
  箇所 1 つ、production が実態より狭い箇所 2 つ。仕様由来オラクルは仕様のレビューにもなる。
- **「仕様が決めていない」を件数で持てるようになった。** 129 件中 14 件（10.9%）は
  仕様から答えが出ない。これは実装の自由度であって、どの実装も「正しい」と言えない領域である。

## 再現手順

```bash
# ケース集の形と引用の検査（sibling repo が無ければ照合は省略）
cargo test --offline -q -p ubnfc-harness --test spec_corpus

# UBNF 129 ケース × 4 実装（JDK 21 と ~/.m2 の unlaxer-common / unlaxer-dsl が要る）
bash examples/ubnf-crosscheck/scripts/run-spec.sh
# → examples/ubnf-crosscheck/out-spec/report.md

# P4 32 ケース × 2 実装
bash examples/p4-crosscheck/scripts/run-spec.sh
# → examples/p4-crosscheck/out-spec/report.md
```

既存 corpus は不変・不退行（Java Pass 1310 / Fail 0、Rust Pass 1292 / Fail 0）。
