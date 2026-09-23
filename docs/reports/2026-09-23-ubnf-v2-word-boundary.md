# リテラルの単語境界（D-040） — 実測・規則・移行

> 2026-09-23。`06-ubnf-v2.md` §3 の実装。front `crates/ubnfc-front/src/boundary.rs`、
> 両 backend の literal 実行、`W-CHOICE-PREFIX` の条件付け。
> **v1 は 1 bit も変わらない**（corpus Java 1310 / Rust 1292、IR golden・self-host 固定点すべて不変）。

## 0. 直したかった弱点

D-002: UBNF のリテラルには単語境界がない。`'let'` は `letter` の先頭 3 文字に一致する。
旧実装の `WordParser` が literal の code point 長ぶんだけ比較するからで
（`ParserRuleEmitter.java:1067,1182`）、**仕様ガイド自身は「`WordParser("let")` は識別子を
横取りしない」と書いていて実装と矛盾していた**。ubnfc は互換のため実装側を正とし、
文法作者は毎回手で回避していた。ANTLR の lexer は maximal munch で無料で正しい。

この矛盾が ubnfc 自身にも刺さっている証拠が `known/keyword-boundary.ubnf`
（`grammar Names { @rooted Root ::= 'x'; }`）で、**手書き frontend は `@rooted` を
1 つの annotation として読むのに、`ubnf.ubnf` から生成した frontend は `'@root'` で確定して
byte 24 で拒否する**。self-host の既知差分 6 件の 1 つがこれだった（`05-selfhost.md`）。

## 1. 先に測った（実装前に既定を変えない）

`@literalBoundary` を足したうえで、**全 grammar に `word` を強制**して走らせ、
変わった観測を数えた。強制は `UBNFC_LITERAL_BOUNDARY_DEFAULT=word`（文法が
`@literalBoundary` を書いていれば常にそちらが勝つ。実測専用の既定差し替え）。

### 1.1 静的 — どのリテラルが境界の対象になるか

24 fixture のうち IR まで通る 21 件（`ubnf.ubnf` 自身を含む）と P4。`wordBoundary` 以外は
**IR が 1 byte も変わらない**ことを全件で確認した（`on` と `off` の IR から flag を落とすと完全一致）。

| 文法 | literal 総数 | 境界対象 | 対象の値（抜粋） |
|---|---:|---:|---|
| `tinyexpression-p4.ubnf` | 390 | **130** | `'if'` `'else'` `'true'` `'var'` `'variable'` `'set'` `'as'` `'external'` `'.length'` `'.in'` `'toUpperCase'` … |
| `unlaxer-dsl/grammar/ubnf.ubnf`（self-description） | 131 | **44** | `'@root'` `'@mapping'` `'token'` `'grammar'` `'name'` `'params'` `'CI'` `'REGEX'` … |
| `unlaxer-dsl/examples/tinycalc.ubnf` / `tinycalc-vscode` | 各 10 | 各 3 | `'var'` `'variable'` `'set'` |
| `positive/boundaries.ubnf` | 7 | 6 | `'grammar'` `'root'` `'token'` `'a'` `'b'` `'e'` |
| `positive/elements.ubnf` | 6 | 5 | `'a'` `'b'` `'c'` `'d'` `'x'` |
| `mixed-values/{Shared,Mixed,Delimited}.ubnf` | 6/6/8 | 5/3/3 | `'a'` `'f'` `'s'` `'x'` `'y'` |
| `evolution/{2,3}` | 5/10 | 1/2 | `'neg'` `'if'` |
| `cardinality/Cardinality.ubnf` | 6 | 1 | `'x'` |
| `known/{eval-default,keyword-boundary,common-field-space}` | 各 1 | 各 1 | `'x'` |
| `story/code/calc*.ubnf`、`associative/Operators`、`right-associative/Power`、`evolution/{0,1}` | 6/7/6/6/1/1/2 | **0** | （記号だけ） |
| **合計** | **627** | **209** | |

`CI('select')` は `all-tokens.ubnf` にしか無く、その fixture は resolve エラーで IR に到達しない。

### 1.2 corpus（全 1,425 観測、Java / Rust 両 backend）

| 実行 | Pass | 期待拒否 | Known | **Fail** | 未観測 |
|---|---:|---:|---:|---:|---:|
| Java v1（基準） | 1310 | 28 | 28 | **0** | 59 |
| Java `word` 強制 | 1213 | 28 | 28 | **98** | 58 |
| Rust v1（基準） | 1292 | 30 | 28 | **0** | 75 |
| Rust `word` 強制 | 1197 | 30 | 28 | **98** | 72 |

系統別（Java）:

| 系統 | Pass(v1) | Pass(word) | Fail(word) |
|---|---:|---:|---:|
| associative | 24 | 24 | 0 |
| cardinality | 70 | 62 | 8 |
| catalog | 2 | 2 | 0 |
| evolution | 140 | 140 | 0 |
| lexical | 78 | 78 | 0 |
| mixed-values | 98 | 79 | 19 |
| nested-captures | 10 | 2 | 8 |
| numeric-capture | 13 | 6 | 7 |
| oracle | 332 | 304 | 29 |
| primitives | 109 | 108 | 1 |
| right-associative | 112 | 112 | 0 |
| rule-trivia | 33 | 25 | 8 |
| scope-annotations | 14 | 12 | 2 |
| scope-store | 0 | 0 | 0 |
| semantic-cardinality | 163 | 147 | 16 |
| tiny-code-fence | 73 | 73 | 0 |
| tiny-string-token | 39 | 39 | 0 |
| **合計** | **1310** | **1213** | **98** |

観測別の内訳（1 ケースが複数の観測を含む）:

| 観測 | 変化した回数 |
|---|---:|
| `/allConsumed` | 88 |
| `/captures/*` | 66 |
| `/value` | 46 |
| `/ast` | 43 |
| `/consumed` | 32 |
| `/matched` | 32 |
| `/accepted` | 30 |
| `/parsedStatus` | 19 |
| `/diagnostics/*` | 16 |
| `/diagnostics/*/farthestExpected/*` | 15 |
| `/scope/declarations/*` | 10 |
| `/mappingError` | 7 |
| `/diagnostics/*/expected/*` | 7 |
| `/diagnostics/*/farthestOffset` | 6 |
| `/diagnostics/*/offset` | 5 |
| `/declarations/*` | 4 |
| `/diagnostics/*/kind` | 3 |

**Java と Rust で落ちるケースは完全に同一の 98 件**（片側だけのものが 0 件）で、
両 backend が同じ規則を実装していることの直接の証拠になっている。

**98 件すべてが同じ 1 つの形**である。影響を受けた 41 build の境界対象リテラル 115 本のうち
**114 本が 1 文字**（`'a'` 34 / `'x'` 21 / `'b'` 19 / `'y'` 14 / …）、残り 1 本が `'ab'`。
corpus の合成文法は `R ::= 'a' 'b'` を入力 `"ab"` で照合するので、`'a'` の直後の `b` が
識別子構成文字になり、境界が効いて落ちる。**キーワード横取りの実例ではなく、
「英数字リテラルを空白なしで連結する試験文法」の帰結**である。

### 1.3 実在の文法 — P4 / `ubnf.ubnf`

| 対象 | 結果 |
|---|---|
| **P4 Rust**（`examples/p4-rust`、境界 on で再生成） | **12/12 緑**。16 fixture の受理・消費量・canonical AST golden・Java golden 突合・predict 参照経路・診断予算、すべて不変 |
| **P4 Java**（`examples/p4-java`、境界 on で再生成） | **28 中 27 緑**。落ちたのは `generatedSourcesAreByteIdentical`（**commit 済み v1 IR から再生成した source と一致するか**という meta 試験）だけで、構文上必ず落ちる。AST・legacy snapshot・scanner・predict 同値はすべて不変 |
| **`ubnf.ubnf`**（self-host、境界 on で再生成） | **改善**。既知の self-host 差分が **6 件 → 5 件**（`known/keyword-boundary.ubnf` の `@rooted` が手書き frontend と一致するようになった）。受理する fixture は 19 → 20、残り 5 件の拒否 byte 位置は不変、self-description 自身の AST・構文エラー位置・validation error golden も不変 |

つまり **実在の 3 文法では 1 つも退行せず、1 つの既知バグが消える**。

### 1.4 開始側の境界も課したら（棄却した案の実測）

「一致の直前の code point も識別子構成文字なら不一致」を足した変種を、Java runtime の
一時計測スイッチで測った（front の flag はそのまま。**末尾が識別子構成文字のリテラルにだけ**
開始側も課す形）。

| 実行 | Pass | Fail | 応答が変わったケース |
|---|---:|---:|---:|
| Java `word`（末尾だけ、採用案） | 1213 | 98 | — |
| Java `word` + 開始側 | 1213 | 98 | **0** |

**corpus はこの 2 案を区別できない**。全 1,425 ケースの応答が 1 byte も違わなかった。
理由は明快で、corpus で開始側が効く位置（`'a' 'b'` を `ab` で読むときの `'b'`）は、
その手前で `'a'` が**末尾側**の境界に落ちているので到達しないからである。

つまり開始側は corpus では**何も増やさず何も減らさない**。一方で、開始側は
末尾側より厳しい（受理を減らす方向にしか動かない）ので、corpus に無いだけで
`NUMBER 'px'` を `10px` で読む形（`'px'` の直前が `0`）は確実に壊れる。
得るものが測っても 0 で、失うものが実在の形なので採らない。

### 1.5 差分ファジング

`crates/ubnfc-fuzz` の v2 seed 帯（`V2_SEED_BASE = 100_000`）に **末尾が識別子構成文字の
リテラル**（`let` / `letter` / `in` / `int` / `if` / `x1` / `_`）を足した。v2 の文法は
`@literalBoundary: word` が既定なので、生成された文法は必ず境界を踏む。

- 60 seed 中 **59 seed** が境界つきリテラルを含む（`gen::tests::word_boundary_literals_only_appear_in_the_v2_seed_band`）
- v1 の seed 帯（0..199）は **0 件**。seed 0..39 の生成物は master と **byte 一致**
  （`ubnfc-fuzz --dump` の出力を diff。乱数の引き方を変えていない）
- 固定 seed campaign `d_v2_word_boundary_literals_agree_across_backends`
  （seed 100_040..100_042 × 12 入力）で **未登録の差分 0 / driver error 0 / discard 0**。
  Java は UTF-16 単位、Rust は UTF-8 byte 単位で「一致直後の文字」を見るので、
  非 ASCII・非 BMP が続く入力での食い違いが無いことをここで押さえている

## 2. 決めた規則

> **末尾が識別子構成文字のリテラルは、一致の直後の code point も識別子構成文字なら一致しない。**

各論点を明示的に決めた。

| 論点 | 決定 | 根拠 |
|---|---|---|
| 識別子構成文字の集合 | **ASCII `[A-Za-z0-9_]`** | 組込み `IdentifierParser` の継続文字集合と厳密に同じ（`AlphabetNumericUnderScoreParser`。runtime の `identifier()` が `is_ascii_alphanumeric() \|\| '_'`）。横取りする相手がその集合でしか伸びないので、これが必要十分。非 ASCII を識別子に含めない理由は (a) 組込み識別子が非 ASCII を受理しないので `'let'` + `é` は横取りではない、(b) 生成物は**外部依存ゼロ**契約で、Unicode の `XID_Continue` 表を全生成物に埋め込むのは費用が釣り合わない |
| 開始側にも課すか | **課さない** | (a) PEG は「直前の要素が終わった位置」からしかリテラルを試さないので、開始側は構造的に守られている。開始側で守られていないなら、それは前の要素が語の途中で終わったという別のバグで、境界で隠すべきではない。(b) 接尾キーワード（`10px` / `3.5f` / `0xFFu`）を壊す。(c) **実測で得るものが 0**: corpus 1,425 ケースの応答が末尾側だけの場合と 1 byte も違わなかった（§1.4）。厳しくする側の変更で利得が測れないなら入れない |
| 記号で終わるリテラル | **対象外**（`'+='` `'('` `'.in('` `';'`） | 規則が「キーワード横取り」だけに当たるための形の条件。`'+'` の直後に識別子が来るのは正常な文法。§1.1 のとおり 627 本中 418 本がここで落ちる |
| `CI('select')` | **同じ規則を適用**（`TokenSpec.caseInsensitive.wordBoundary`） | `'select'` と `selected` の関係は大文字小文字と無関係。境界の判定は「一致の次の文字」なので大小同一視は関係しない。実装は両 backend 1 行ずつ、IR は追加キー 1 つ |
| `LOOKAHEAD` / `NEGATIVE_LOOKAHEAD` | **対象外** | 消費しないゼロ幅述語で、「次が生の文字列 X か」を覗くのが用途。境界を課すと述語の意味が変わる |
| `UNTIL('#')` の終端、`@recovery` の同期文字列 | **対象外** | 区切り記号であって語ではない。同期は文字列走査であり字句照合ではない |
| `token` 宣言の中と rule 本体の違い | **同じ**（`CI` 以外に literal 形の token 宣言は無い） | UBNF の `TokenValue` に「素のリテラル」は無く、語の形をした消費する終端は rule 本体の literal と `CI` だけ |
| リテラルの長さで例外を作るか | **作らない** | corpus の退行 98 件は 1 文字リテラルが原因（§1.2）なので「1 文字は除外」で消せるが、それは**この corpus への当てはめ**であって規則ではない。`'if'` `'in'` `'do'` `'of'` は 2 文字の実在キーワードで境界が要り、`'e'`（指数）や `'L'`（long 接尾）は 1 文字でも境界が邪魔になる。長さは「キーワードか抽象 token か」を分離しない。分離するのは作者の意図で、それを書く場所が `@literalBoundary` である |
| 境界違反の診断 | **リテラル不一致と完全に同じ** | 同じ位置（一致開始位置）、同じ expected label（`'let'` / 表示名 `WordParser`）、`consume(0)`。新しい診断 kind も新しい候補ラベルも増やさない。実装上も「長さ `None`」に落として既存の primitive 経路へ流すだけなので、`farthestOffset` / `expected` / `deepestRule` の蓄積は自動的に従来どおり |

### 2.1 設定と版

`@literalBoundary: word|none`（grammar ごとに 1 回）。

| `@ubnf` | 既定 |
|---|---|
| 1（既定） | `none` — **v1 は凍結** |
| 2 | `word` |

- 適用範囲は「そのリテラルを書いた文法」。import 先の文法は import 先の設定に従う
  （`@whitespace` と同じ、rule の所属で決まる）。
- `E-LITERAL-BOUNDARY-VALUE` / `E-LITERAL-BOUNDARY-DUPLICATE`。検査は desugar 段（resolve より前）。

### 2.2 IR と backend

- `ExprNode.literal.wordBoundary` を**固定値 `false` から真の bool へ**（JSON の形は変わらないので
  v1 の golden は byte 一致）。`TokenSpec.caseInsensitive.wordBoundary` は**追加キー**。
  schema `ir/schema/grammar-ir.v1.schema.json` を両方更新。
- **backend は flag しか見ない**。Rust `Session::literal(..., boundary: bool, ...)`、
  Java `Session.literal(f, word, sensitive, boundary, label)`。
  境界判定は「一致の直後」1 個所の ASCII 判定だけ。Rust は byte（非 ASCII は先頭 byte が
  0x80 以上なので自動的に非識別子）、Java は `codePointAt`（非 ASCII は範囲外）で、
  非 BMP を含めて同じ答えになる。
- **最適化との整合**: Rust の字句 cache（統合 DFA）は「その位置でその語に一致するか」しか答えないので、
  境界判定は cache hit / miss の両経路の**後**に一度だけ掛ける。候補除外（`predict.rs`）が使う
  FIRST は「先頭 1 文字」の上界のままなので境界があっても健全（境界は失敗を増やす方向にしか効かない）。
  共通形の畳み込み（`factor.rs`）の shape 文字列に `wordBoundary` を含めたので、同値でないリテラルが
  同じ helper に潰れることはない。memo キーは (式, カーソル) のままでよい（境界は入力と位置の純粋関数）。

## 3. `W-CHOICE-PREFIX` への波及（D-037）

D-037 の `W-CHOICE-PREFIX` は「先行候補がリテラル `L`、後続候補の必須先頭リテラル `P` が
`L` で始まる ⇒ 後続は死ぬ」だった。この含意は**境界が無いことに依存**している。

境界があると `L` は「直後が識別子構成文字でない」ときにしか成功しないので、
`P` が `L` の続きに識別子構成文字を持つ場合（`'var'` と `'variable'`）は**後続候補が生き残る**。
そこで、先行候補に `wordBoundary` があるときは

- `P` が `L` の直後に**識別子構成文字でない文字**を持つと分かるときだけ警告する（`'x' | 'x+y'` は残る）
- `P == L` で続きが分からないときは**出さない**（後続が語を伸ばして一致しうる）

とした（`crates/ubnfc-front/src/analysis/reachability.rs`）。証明の方向は D-037 のまま下界で、
検出が減る方向にしか変わらないので健全性は保たれる。

実測（`ubnfc validate`）:

| 文法 | v1 | `word` 強制 |
|---|---:|---:|
| `unlaxer-dsl/examples/tinycalc.ubnf` | `W-CHOICE-PREFIX` **1** | **0** |
| `unlaxer-dsl/tinycalc-vscode/grammar/tinycalc.ubnf` | `W-CHOICE-PREFIX` **1** | **0** |
| `tinyexpression-p4.ubnf` | `W-CHOICE-SHADOWED-PATH` **9** | **9**（不変） |
| 24 fixture の残り・`ubnf.ubnf` | 0 | 0 |

`'variable'` が本当に到達可能になるので警告が消えるのが正しい。P4 の 9 件は
`W-CHOICE-SHADOWED-PATH` で rule の決定性にしか依存しないので不変。
`W-CHOICE-UNREACHABLE` の構造同一比較は元から `wordBoundary` を含んでいる。

## 4. 移行メモ（v1 → v2）

`@ubnf: 2` にすると、既定で `@literalBoundary: word` になる。**受理が変わりうるのは
「末尾が識別子構成文字のリテラルの直後に、識別子構成文字が来る入力」だけ**である。

### 4.1 自分の文法が影響を受けるか調べる

```sh
# 境界の対象になるリテラルを列挙する（IR の flag をそのまま読む）。
# 文法がまだ v1 なら UBNFC_LITERAL_BOUNDARY_DEFAULT=word を付けて「もし v2 にしたら」を見る。
UBNFC_LITERAL_BOUNDARY_DEFAULT=word ubnfc ir --grammar g.ubnf --out - \
  | jq -r '.semantics.expressions[].node | select(.kind=="literal" and .wordBoundary) | .value' | sort -u
```

この一覧の各リテラル `L` について、**正しい入力の中で `L` の直後に `[A-Za-z0-9_]` が来る場所が
あるか**だけを見ればよい。典型的に「ある」のは:

1. **空白を挟まず英数字リテラルを連結する規則**（`R ::= 'a' 'b'` を `ab` で読む）。
   corpus の退行 98 件はすべてこれ。
2. **英数字の接頭辞・接尾辞**（`'n' NUMBER` を `n0` で読む、`NUMBER 'px'` を `10px` で読む）。
3. **語の途中で切る規則**（`'variable'` を `'var'` + `IDENTIFIER` で読ませていた回避策）。
   これは元の弱点そのものなので、v2 では回避策を外す側の作業になる。

該当したら、その文法は `@literalBoundary: none` を宣言すれば v1 の意味論のまま v2 の他の機能
（左再帰など）を使える。個別のリテラルだけ外す手段は**用意していない**（§5）。

### 4.2 実在の文法の具体的な結論

| 文法 | v2 にしたとき |
|---|---|
| `tinyexpression-p4.ubnf` | **そのままでよい**。130 本が境界対象になるが、Rust 12/12・Java 27/28 が不変（§1.3）。全 rule が `javaStyle` の trivia を読むので、キーワードの直後は空白・記号・EOF しか来ない |
| `unlaxer-dsl/grammar/ubnf.ubnf` | **そのままでよく、むしろ直る**。`@rooted` のような「keyword + 識別子の続き」を `'@root'` で確定しなくなり、手書き frontend と一致する（既知差分 6 → 5）。self-description の IR も固定点も不変 |
| `tinycalc.ubnf`（2 本） | **そのままでよく、`( 'var' \| 'variable' )` の順序バグが無害になる**。v1 では `variable x;` が `'var'` で確定して壊れていた（D-037 §3.3）。v2 では `'var'` が `variable` に一致しないので `'variable'` に落ちる。`W-CHOICE-PREFIX` も出なくなる |
| corpus の合成文法（`mixed-values` / `nested-captures` / `semantic-cardinality` ほか） | **`@literalBoundary: none` が要る**。`'a' 'b'` を `ab` で読む形なので、境界を入れると受理が変わる。ただし **corpus は v1 のまま凍結**なので、実際には何も起きない |

## 5. 変えなかったもの・入れなかったもの

| 案 | 入れなかった理由 |
|---|---|
| リテラル単位の境界指定（`'let'@word` のような表記） | 表層の構文が増えるわりに、実例（P4 / `ubnf.ubnf` / tinycalc）でどれも要らなかった。必要になってから足すほうが安い。grammar 単位の `none` が退路として十分 |
| 開始側の境界 | §1.4 / §2 の表。構造的に不要で、実測では corpus の応答を 1 byte も変えず（＝利得 0）、`10px` のような接尾キーワードだけを壊す |
| 長さ 1 のリテラルの例外 | §2 の表。corpus への当てはめであって規則ではない |
| `LOOKAHEAD` / `UNTIL` / `@recovery` 同期文字列への適用 | 語ではなく区切り／述語。§2 の表 |
| 非 ASCII の識別子文字（Unicode `XID_Continue`） | 組込み `IdentifierParser` が非 ASCII を受理しないので、横取りの相手がいない。生成物の依存ゼロ契約に表を持ち込む費用が釣り合わない。将来 Unicode 識別子 token を入れるなら、その token と同じ集合を IR に載せて backend へ渡す形になる |
| 「境界で受理が変わりうる位置」の警告 | 有用だが別件。`W-LITERAL-BOUNDARY-*` として FOLLOW 集合から近似できる（`analysis/follow.rs` が持っている）。移行を機械化したくなったときに足す |
| P4 / corpus / oracle の修正 | 読み取り専用・期待値。`Expression` の候補順（D-037 §3.1）も、tinycalc の `( 'var' \| 'variable' )` も直していない |

## 6. 足したテスト

| 場所 | 内容 |
|---|---|
| `crates/ubnfc-front/tests/literal_boundary.rs` | v1 は常に `false`、v2 は末尾が識別子構成文字のものだけ、設定の両方向上書き、`E-LITERAL-BOUNDARY-*`、`CI` の同規則、`W-CHOICE-PREFIX` の条件（`'var' \| 'variable'` は v2 で消え、`'x' \| 'x+y'` と `'+' \| '+='` は残る） |
| `crates/ubnfc-rt-template/tests/input.rs` | `identifier_byte_at` が ASCII `[A-Za-z0-9_]` だけを真にする（非 ASCII・終端・範囲外） |
| `crates/ubnfc-rust/tests/semantics.rs` + `support/semantic_assertions.rs` | 生成 Rust を実走: `boundary_off` / `boundary_on` / `boundary_symbol` / `boundary_prefix_choice` / `case_insensitive_boundary`。境界違反の診断が不一致と同一であることも含む |
| `ubnfc-java/src/test/java/org/ubnfc/java/WordBoundaryTest.java` | 生成 Java を compile して実走。同じ観点（非 ASCII が続くと一致する、`letA` / `let9` / `let_` は一致しない、診断が不一致と同一、`CI` も同規則） |
| `crates/ubnfc-fuzz`（`gen.rs` / `tests/campaign.rs`） | v2 seed 帯に語境界つきリテラルを足し、`d_v2_word_boundary_literals_agree_across_backends` で両 backend を突き合わせる |

## 7. 不変の確認

| 検査 | 結果 |
|---|---|
| Java `run-all`（v1 既定） | **Fail 0**。Pass **1310** / 期待拒否 28 / Known 28 / 未観測 59 — `docs/reports/2026-09-23-java-corpus-contract.md` と一致 |
| Rust `run-all`（v1 既定） | **Fail 0**。Pass **1292** / 期待拒否 30 / Known 28 / 未観測 75 — `docs/reports/2026-09-23-rust-corpus-contract.md` と一致 |
| IR golden（`ir/fixtures/**`、`ir/examples/tinycalc.ir.json`） | 不変（`wordBoundary` は v1 で常に `false`、JSON の形も同じ） |
| self-host 固定点（`ubnfc-selfhost` の `parity` / `regenerate`） | 不変（既知差分 6 件・受理 19 件のまま） |
| `cargo test --workspace --offline` / `clippy -D warnings` / `fmt --check` | 通過 |
| `cd ubnfc-java && mvn -o -q test`、`cd examples/p4-java && mvn -o -q test` | 通過 |
| `cargo test -p ubnfc-fuzz` | 通過（v2 語境界 campaign を含む） |

生成物の再生成（手編集なし）: `crates/ubnfc-selfhost/src/generated/**`、
`examples/p4-rust/src/generated/**`、`examples/p4-java/src/main/java/org/ubnfc/p4/generated/**`。

## 8. 再現

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cd ubnfc-java && mvn -o -q -DskipTests package && mvn -o -q test
cd examples/p4-java && mvn -o -q test
# 互換影響の実測（既定を強制的に word にする）
UBNFC_LITERAL_BOUNDARY_DEFAULT=word cargo run -q --offline --bin ubnfc-harness -- \
  run-all --backend java --report /tmp/java-word.md --json /tmp/java-word.json
UBNFC_LITERAL_BOUNDARY_DEFAULT=word cargo run -q --offline --bin ubnfc-harness -- \
  run-all --backend rust --report /tmp/rust-word.md --json /tmp/rust-word.json
# 個別の文法で境界対象を見る
UBNFC_LITERAL_BOUNDARY_DEFAULT=word cargo run -q --offline --bin ubnfc -- ir --grammar g.ubnf --out -
```
