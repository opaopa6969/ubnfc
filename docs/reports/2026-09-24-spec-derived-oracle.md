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

計測は **issue #1 の修正（D-050 / D-051）を取り込んだ後**の `origin/master` + 本 branch。

| 実装 | 一致 | 一致(validate) | 違反 | 別コード | 未定 | 駆動不能 |
|---|---:|---:|---:|---:|---:|---:|
| A ubnfc 手書き frontend | 94 | 8 | **10** | 3 | 11 | 3 |
| B ubnfc 自己ホスト生成 | 83 | 0 | **10** | 0 | 11 | 25 |
| C unlaxer-ubnf（手書き Rust） | 100 | 0 | **1** | 0 | 11 | 17 |
| D Java bootstrap | 109 | 0 | **3** | 3 | 11 | 3 |

- **`一致(validate)`** = 構文解析では受理するが、同じ道具の `validate` 段が ERROR で拒否する場合。
  仕様は「拒否する」としか書いておらず、**どの層で拒否するか**は定めていないので違反と混ぜない。
  8 件はすべて A（CHAR_RANGE 7 件 + `ERROR();`）で、単一パスの C / D は同じ入力を構文段で拒否する。
- 「別コード」= 同じ違反を検出しているが `validation.md` と違うコード名を出した場合（§4）。
- B と C の「駆動不能」が多いのは、独立した validator を持たないため `validate` scope の
  14 件（+ `runtime` 3 件）を駆動できないから。B はさらに、A が validate 段で拒否した 8 件を
  自己ホスト front 経由で駆動する入口が無いので推測で埋めていない。
- 「未定」列が 11 なのは、`undetermined` 14 件のうち 3 件が `runtime` scope で
  「駆動不能」に入るため。
- 残る違反は **ubnfc 10 件、旧実装（D）3 件**。旧実装を基準にしている限り、
  ubnfc が広すぎるのか旧実装が狭すぎるのかは判定できず、既知乖離として並ぶだけになる。

節ごとの違反数:

| 節 | ケース | A | B | C | D |
|---|---:|---:|---:|---:|---:|
| 形式的な文法定義 | 23 | 8 | 8 | 1 | 2 |
| 量指定子とセパレータ | 16 | 2 | 2 | 0 | 0 |
| コメント | 5 | 0 | 0 | 0 | 1 |
| 他 7 節（CHAR_RANGE 含む） | 85 | 0 | 0 | 0 | 0 |

## 3. 実装別の違反一覧（最小入力つき）

### A ubnfc 手書き frontend — 違反 10 件（+ `一致(validate)` 8 件）

**(a) CHAR_RANGE の境界検査 — 7 件、issue #1 の D-050 で解決済み**
（引用: `ubnf-syntax.md`「`CHAR_RANGE('min','max')` — 文字の範囲」。「各境界はエスケープ展開後に
1個の非surrogate BMP文字で、`min <= max` でなければならない。空文字・複数文字・surrogate・
補助文字（例 `😀`）・逆順の範囲は `IllegalArgumentException` で明示的に拒否し、
先頭文字へ切り捨てない。」）

このケース集を作った時点（`origin/master` = 9afb8cd）では 7 件すべてが
**構文でも validate でも通り、`CHAR_RANGE('','a')` は `ubnfc validate` が panic（exit 101）**
していた。issue #1 の D-050 を取り込んだ後の実測:

| ケース | 入力 | A の構文 | `ubnfc validate` |
|---|---|---|---|
| `char-range/empty-min` / `-max` | `token L=CHAR_RANGE('','a')` | 受理 | `E-TOKEN-CHAR-RANGE-BOUNDARY` |
| `char-range/multi-char-min` / `-max` | `CHAR_RANGE('ab','z')` | 受理 | `E-TOKEN-CHAR-RANGE-BOUNDARY` |
| `char-range/supplementary` | `CHAR_RANGE('😀','😇')` | 受理 | `E-TOKEN-CHAR-RANGE-SUPPLEMENTARY` |
| `char-range/reversed` | `CHAR_RANGE('z','a')` | 受理 | `E-TOKEN-CHAR-RANGE-ORDER` |
| `char-range/escaped-bounds-reversed` | `CHAR_RANGE('\n','\t')` | 受理 | `E-TOKEN-CHAR-RANGE-ORDER` |

仕様は「拒否する」としか書いていないので、**構文段で拒否する C / D と validate 段で拒否する
ubnfc はどちらも規範文を満たす**。`一致(validate)` として違反とは別に数える。
`CHAR_RANGE('\t','\n')`（展開後 0x09 <= 0x0A）と `CHAR_RANGE('a','a')` は仕様どおり通るので、
「エスケープ展開後に判定する」「両端を含む」側の期待も同じファイルに入れてある。
`ERROR();` も同様に `E-RULE-UNDEFINED` で validate が拒否する（`ERROR` が規則参照に落ちるため）。

**(b) 空構文 — 8 件**（引用: `UBNFFile ::= { '//' LINE_COMMENT } GrammarDecl+`、
`SequenceBody ::= AnnotatedElement+`）

`(空ファイル)` / `// just a comment` / `R ::= ;` / `R ::= 'a' | ;` / `R ::= | 'a';` /
`R ::= ();` / `R ::= [ ];` / `R ::= { };` の 8 件。C と D は 8 件とも拒否し、
`ubnfc validate` も ERROR を出さない。
→ **意図的。D-052 として記録した**（自己記述文法 `ubnf.ubnf` の `SequenceBody` が
`{ AnnotatedElement }` で、`crates/ubnfc-selfhost/tests/parity.rs` が受理入力として
明示列挙している。狭めると self-host の乖離方向が 8 件増え、実在文法の受理は 1 つも変わらない）。

**(c) 量指定子の併記 — 2 件**（引用: `AnnotatedElement ::= … AtomicElement [Quantifier | '%' AtomicElement] …`）

| ケース | 入力 | A/B | C/D |
|---|---|---|---|
| `quantifier/two-quantifiers` | `grammar G { token V=ANY R ::= V+{2,3}; }` | 受理 | 拒否 |
| `quantifier/quantifier-and-separator` | `grammar G { token V=ANY token S=ANY R ::= V+ % S; }` | 受理 | 拒否 |

→ **意図的な拡張。D-053 として記録した**。

### B ubnfc 自己ホスト生成 — 10 件

A と同じ 8 + 2 件。**ブロックコメントの乖離は解消している**。
このケース集を作った時点では B だけが `grammar G { /* no */ R ::= 'x'; }` と
`grammar G { R ::= /* no */ 'x'; }` を受理していた（2026-09-23 の比較報告が見つけた
7 件目の self-host 乖離）。`comments.json` の 2 ケースは仕様の側から独立にそれを検出し、
issue #1 の D-051 の修正後は A と同じく拒否する。

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
| `rule/dotted-reference-three-segments` | `a.b.Value` の namespace が `a.b` か `a` か。「ドットは外側の文法が処理する」としか書かれていない。A/C/D は受理、B（自己ホスト）は拒否 — 既知の self-host 乖離の 1 つを仕様側から拾った |
| `escape/unknown-sequence` | 表は `\n \t \r \\ \'` の 5 つだけ。`'\q'` を拒否するのか、バックスラッシュごと残すのか、`q` にするのか。4 実装とも受理 |
| `quantifier/reversed-bounds` | `V{9,2}`。`Quantifier` の production は n <= m を要求していない。CHAR_RANGE と違い逆順反復の規範文が無い。4 実装とも構文としては受理 |
| `quantifier/huge-bound` | `'x'{2147483648}`。INTEGER の表現範囲が書かれていない。C/D は構文で拒否、ubnfc は構文で受理し validate が `E-BOUNDED-OVERFLOW`（issue #1 の D-050 で追加）。仕様が範囲を決めていないので、どちらも「違反」とは言えない |
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
| issue #1 が独立に修正した | 9 | CHAR_RANGE の境界検査 7 件（D-050）と self-host のブロックコメント 2 件（D-051）。**本ケース集は同じ 9 件を仕様の側から独立に指し、修正後の受け入れ条件としてそのまま回る** |
| 意図的（仕様より広い受理を維持） | 10 | **D-052**（空構文 8 件）、**D-053**（量指定子併記 2 件）。どちらも受理・AST・IR・corpus 不変の追認 |
| 仕様書の誤り | 3 | 診断コード名（compat-matrix C16）。実装は変えない |
| 他実装の違反 | 4 | C 1 件・D 3 件。ubnfc 側の作業ではない（記録のみ） |

**ubnfc のコードは本 branch で 1 行も変えていない。** ケース集を作った時点で ubnfc にあった
18 件のうち 9 件は issue #1 が並行して直し（`crates/ubnfc-front/**` を同時に書き換えないため
こちらでは触らなかった）、残る 10 件は意図的な差として D-052 / D-053 に落ちた。
本 branch の成果物は**仕様由来のケース集 161 件と、それを回すドライバ、決定 2 件、
compat-matrix の 5 行（C16〜C20）**である。

## 8. この作業が示したこと

- **「旧実装と同じ」では仕様との距離が測れない。** ケース集を作った時点で ubnfc は 18 件、
  旧実装は 3 件違反した。この 18 件が「旧実装と食い違っている」ことは以前から見えていた
  （2026-09-23 の比較報告の受理不一致 21 件に含まれる）が、**どちらが仕様に沿っているかは
  問われていなかった**。仕様を当てて初めて「ubnfc が広すぎる 18 件」「旧実装が狭すぎる /
  緩すぎる 3 件」と向きが決まり、9 件が修正に、10 件が意図的な拡張の決定に落ちた。
  逆に `grammar G { }` のように **ubnfc が正しく他 2 実装が誤っている**ことも、
  旧実装を基準にしている限り出てこない。
- **層の違いを違反と混ぜないと決められた。** CHAR_RANGE の 7 件を ubnfc は validate 段で、
  C / D は構文段で拒否する。仕様は「拒否する」としか書いていないのでどちらも適合であり、
  `一致(validate)` として別に数えた。「同じ入力を拒否するか」だけを見る差分比較では、
  この区別は立てられない。
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

## 9. 追記（2026-09-24 その2、issue #11 — unlaxer-parser #279 への追随）

unlaxer-parser #277（PR #279、`4754a3c` で merge 済み）が UBNF 仕様書の欠陥を訂正し、
本報告 §4 の存在しない診断コード3件と §5 の未定14件のうち多くに決定を書いた。
ここではその訂正を `corpus/spec/ubnf/**` に反映し、4実装クロスチェックを取り直す。

### 9.1 引用文が変わった3件

`validation.md` のコード名訂正に追随し、`citation.sentence` と `expectation.code` を更新した。
検出条件（両実装が出す実際のコード）は変わっていない。

| ケース | 旧コード名 → 新コード名 |
|---|---|
| `annotation/mapping-extra-capture` | `E-MAPPING-EXTRA-CAPTURE` → `E-MAPPING-UNLISTED-CAPTURE` |
| `annotation/precedence-without-assoc` | `E-PRECEDENCE-WITHOUT-ASSOC` → `E-PRECEDENCE-NO-ASSOC` |
| `annotation/assoc-without-precedence` | `E-ASSOC-WITHOUT-PRECEDENCE` → `E-ASSOC-NO-PRECEDENCE` |

`cargo test -q -p ubnfc-harness --test spec_corpus --offline`（`UNLAXER_PARSER_DIR` を
unlaxer-parser `931c919` へ向けて実行）は3件とも緑。

### 9.2 リネーム2件の突き合わせ（ubnfc 側の変更は無し）

`E-RIGHTASSOC-NON-CANONICAL` → `E-RIGHTASSOC-NONCANONICAL`、
`E-PRECEDENCE-MIXED-LEVEL` → `E-PRECEDENCE-MIXED-ASSOC` の2件は、
`grep -rn "E-RIGHTASSOC\|E-PRECEDENCE-MIXED" crates/ubnfc-front/src` で確認したところ
`crates/ubnfc-front/src/validate/operators.rs` が新コード名（`E-RIGHTASSOC-NONCANONICAL` /
`E-PRECEDENCE-MIXED-ASSOC`）をそのまま出しており、**訂正後の仕様と一致**。
ubnfc 側の corpus にもこの2件に該当するケースが無いため、追随作業は不要だった。

### 9.3 未定14件の昇格

11件を `accept` / `diagnostic` へ昇格し、3件は runtime scope のまま `undetermined` に残した
（理由は下記9.4）。すべて `examples/ubnf-crosscheck` の CLI（`ubnfc validate --format json`）で
実測してから昇格しており、期待値を実装に合わせて書き換えてはいない
（4実装クロスチェックが独立に一致を確認している）。

| ケース | 新しい期待値 | 根拠 |
|---|---|---|
| `rule/lowercase-name` | `accept` | 「大文字始まり」は命名規約でMUSTではない |
| `rule/dotted-reference-three-segments` | `accept` | 3段以上の dotted 参照も1個の IDENTIFIER として受理 |
| `escape/unknown-sequence` | `accept` | 表にないエスケープはエラーにしない（展開結果の文字は実装依存のため未検証） |
| `quantifier/reversed-bounds` | `diagnostic` / `E-BOUNDED-INVALID` | 構文層は受理、バリデーション層で拒否 |
| `quantifier/huge-bound` | `diagnostic` / `E-BOUNDED-OVERFLOW` | INTEGER は Java 32bit 符号付き整数 |
| `error/element-needs-string` | `diagnostic` / `E-RULE-UNDEFINED` | `ERROR(msg)` は `ErrorElement` にならず `RuleRefElement` として解決、`ERROR` 未定義で拒否 |
| `annotation/root-two-rules` | `accept` | `@root` の SHOULD 違反は検出・警告しない |
| `reserved/rule-named-token` | `accept` | 予約語は表の位置以外では通常の識別子 |
| `reserved/rule-named-error` | `accept` | 同上 |
| `reserved/token-named-any` | `accept` | 同上 |
| `spec-contradiction/minilang-leftassoc` | `diagnostic` / `E-ASSOC-NO-PRECEDENCE`（scope を `validate` へ変更） | MiniLang 例の修正で矛盾解消。`@precedence` を伴わない `@leftAssoc` は annotations.md の MUST 違反 |

### 9.4 残した undetermined 3件

いずれも `scope: runtime`。仕様は「観測できない」と明記したが、これは**この corpus 形式が
4実装の構文木・診断の比較でしか判定できない**という性質そのものであり、
accept/reject/diagnostic のいずれにも決められない。既存の `scope: runtime` ケース
（`char-range/inclusive-bounds` を含め本報告作成時から一貫して）はすべて `undetermined` として
残す規約になっており、それを踏襲した。

| ケース | 仕様の文言 |
|---|---|
| `char-range/inclusive-bounds` | 「両端を含む判定は…UBNF を読む処理系自体の入出力からは観測できない（`scope: runtime`）」 |
| `separator/expansion` | 「`elem (sep elem)*` への展開は…UBNF ファイルを読む処理系自体の入出力からは観測できない（`scope: runtime`）」 |
| `error/hint-is-last-alternative` | 「ヒントメッセージが…UBNF を読む処理系自体の入出力や構文木の比較では観測できない（`scope: runtime`）」 |

### 9.5 昇格後の4実装クロスチェック（再実測）

`unlaxer-parser` は `931c919`（PR #280 “fix(ubnf): accept empty grammar body, reject trailing
input in Java bootstrap” 併合後）で実行した。共有 `~/.m2/repository` の `unlaxer-dsl-3.0.15.jar`
は PR #280 前のビルドのままだったため、`~/.m2/repository` を隔離コピーへ複製し
（`cp -a ~/.m2/repository $ISO/repository`）、`mvn -o -Dgpg.skip=true
-Dmaven.repo.local=$ISO/repository -pl unlaxer-common,unlaxer-dsl -am -DskipTests install`
で `931c919` のソースから jar を作り直してから `MAVEN_REPO=$ISO/repository
bash examples/ubnf-crosscheck/scripts/run-spec.sh` を実行した（共有 `~/.m2` は汚していない）。

| 実装 | 一致 | 一致(validate) | 違反 | 別コード | 未定 | 駆動不能 |
|---|---:|---:|---:|---:|---:|---:|
| A ubnfc 手書き frontend | 108 | 8 | **10** | 0 | 0 | 3 |
| B ubnfc 自己ホスト生成 | 89 | 0 | **11** | 0 | 0 | 29 |
| C unlaxer-ubnf（手書き Rust） | 108 | 0 | **0** | 0 | 0 | 21 |
| D Java bootstrap | 122 | 0 | **0** | 0 | 0 | 4 |

§2 の初回計測（issue #1 反映後）との差分:

- **別コード 3→0**（A・D とも）: §4 の3コード名訂正を corpus に反映した結果。
- **C の違反 1→0**、**D の違反 3→0**: `grammar/empty-block`・`file/trailing-garbage-rejected`・
  `comment/block-unclosed-rejected` が unlaxer-parser #278（PR #280）で修正され、3件とも一致に変わった。
- **未定 11→0**（駆動不能に吸収された3件を除く）: 11件の昇格による。
- **B の違反 10→11**: 新たに `rule/dotted-reference-three-segments` が違反に加わった（§9.6）。
- A の違反10件（D-052 空構文8件 + D-053 量指定子併記2件）は既存の意図的な差のままで変化なし。

### 9.6 新たな発見: ubnfc-selfhost が3段以上の dotted 参照を拒否する

`rule/dotted-reference-three-segments`（`accept` へ昇格）を4実装へ流したところ、
ubnfc 自己ホスト生成 frontend（B、`crates/ubnfc-front/src/selfhost/**` +
`crates/ubnfc-selfhost/**`）だけが最小入力 `grammar G { R ::= a.b.Value; }` を拒否した
（`構文エラー (bytes 21..22): expected + or ? or * or { or % or @ or @typeof or ( or [ or ERROR
or STRING or IDENTIFIER or | or ;`）。2段の `a.B` は A/B とも受理するため、chained dot が
3段以上になったときだけ selfhost の生成規則が対応していない。

A（手書き frontend）・C（unlaxer-ubnf Rust）・D（Java bootstrap）はいずれも受理する。
selfhost の文法/生成規則は issue #4 の作業範囲（`crates/ubnfc-front/src/selfhost/**` /
`crates/ubnfc-selfhost/**`）のため、本 branch では修正せず
**[opaopa6969/ubnfc#12](https://github.com/opaopa6969/ubnfc/issues/12)** を立てた。

### 9.7 再現手順（追記分）

```bash
# corpus の形と引用の検査
UNLAXER_PARSER_DIR=/path/to/unlaxer-parser \
  cargo test -q -p ubnfc-harness --test spec_corpus --offline

# 4実装クロスチェック（jar を隔離 ~/.m2 で作り直してから）
ISO=$(mktemp -d) && cp -a ~/.m2/repository "$ISO/repository"
(cd /path/to/unlaxer-parser && mvn -o -q -Dgpg.skip=true \
  -Dmaven.repo.local="$ISO/repository" -pl unlaxer-common,unlaxer-dsl -am -DskipTests install)
UNLAXER_PARSER_DIR=/path/to/unlaxer-parser MAVEN_REPO="$ISO/repository" \
  bash examples/ubnf-crosscheck/scripts/run-spec.sh
```

既存 corpus は不変・不退行（Java Pass 1310 / Fail 0、Rust Pass 1292 / Fail 0、下記参照）。

## 追記（2026-09-24 その3、docs 整理 — issue #12 の解消を反映）

§9.6 で報告した `rule/dotted-reference-three-segments`（ubnfc-selfhost だけが `a.b.Value` を
bytes 21..22 で拒否する）は、その後 2 件の PR で解消した。**本節は履歴を書き換えず、追記だけする。**

- 根本原因は `unlaxer-parser` 側の `unlaxer-dsl/grammar/ubnf.ubnf` にあった
  （`QuantifiedRef` がドット1個だけの任意分割で、段数上限なしの chained dot を想定していなかった）。
  [`unlaxer-parser` issue #284](https://github.com/opaopa6969/unlaxer-parser/issues/284) →
  [`unlaxer-parser` PR #285](https://github.com/opaopa6969/unlaxer-parser/pull/285)
  （`QuantifiedRef ::= { IDENTIFIER '.' } @namespace IDENTIFIER @name ;` へ修正）で直った。
- `ubnfc` 側は [PR #20](https://github.com/opaopa6969/ubnfc/pull/20)
  （`fix(selfhost): chained dotted 参照に追随して self-host を再生成する (#12)`）が、直った
  `ubnf.ubnf` から IR fixture と `crates/ubnfc-selfhost/src/generated` を再生成し、
  `@namespace` が `list<text>` になったぶんの投影・変換ロジックを足した。詳細は
  [`docs/design/05-selfhost.md`](../design/05-selfhost.md) の「8 件目」。
  [`opaopa6969/ubnfc#12`](https://github.com/opaopa6969/ubnfc/issues/12) はこの PR で close 済み。

### 再実測（unlaxer-parser `124b484`、2026-09-24）

sibling repo `unlaxer-parser` の現在の master（`124b484`。PR #285 と、無関係な PR #286 を含む）から
§9.7 の手順どおり隔離 `~/.m2` を作り直し、`examples/ubnf-crosscheck/scripts/run-spec.sh` を
本 branch（PR #20 取り込み後）の `ubnfc` に対して再実行した（出力は `mktemp` 配下、repo には
commit していない。再現手順は本ファイル §9.7 と同じ）。

| 実装 | 一致 | 一致(validate) | 違反 | 別コード | 未定 | 駆動不能 |
|---|---:|---:|---:|---:|---:|---:|
| A ubnfc 手書き frontend | 108 | 8 | 10 | 0 | 0 | 3 |
| B ubnfc 自己ホスト生成 | **90**（89 → 90） | 0 | **10**（11 → 10） | 0 | 0 | 29 |
| C unlaxer-ubnf（手書き Rust） | 108 | 0 | 0 | 0 | 0 | 21 |
| D Java bootstrap | 125 | 0 | 0 | 0 | 0 | 4 |

§9.5 との差分は **B の 1 行**（狙った差分）: `rule/dotted-reference-three-segments` が「違反」から
「一致」に変わり、一致 89→90 / 違反 11→10。A・C は §9.5 と一致。**D は 122→125 で、§9.5 の表と
食い違う**——ただし §9.5 の D 行は 122+0+0+0+0+4=126 で 129 件に 3 足りず、算術的に破綻している
（本追記の D 行は 125+4=129 で揃う）。§9.5 の 122 は転記の誤りだった可能性が高く、**D の判定内容
（違反 0 件・駆動不能 4 件）自体は §9.5 から変わっていない**。B の他 10 件の違反は D-052 の空構文
8 件 + D-053 の量指定子併記 2 件で、意図的な既知の差のまま。既存 corpus（Java Pass 1,310 /
Rust Pass 1,292、Fail 0）は本追記のための再測定でも動いていない。
