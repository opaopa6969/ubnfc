# UBNF v2 — パラメータ化ルール（D-043 / D-044）

> 2026-09-23。`06-ubnf-v2.md` §4 の実装報告。
> 対象: `crates/ubnfc-front/src/templates.rs`、`examples/p4-v2/tinyexpression-p4-v2.ubnf`、
> `crates/ubnfc-fuzz`。

## 0. 結論

`Name<P1, ..., Pn> ::= body ;` を**規則ではなく規則本体の雛形**として導入し、
`Name<A1, ..., An>` の位置へ **resolve より前にその場で展開**する（D-039 の左再帰と同じ方法）。

- **新しい規則も新しい名前も作らない**。したがって規則数・AST クラス・capture・
  `rulePath` / `deepestRule` / expected ラベルは**一つも変わらない**。名前のマングリングが要らない。
- 等価性は構成的: 雛形版の IR は、同じ形を手で書いた文法の IR と
  **文法ソース上の範囲だけを除いて完全一致**する（P4 で 117,940 行の IR が 1 行も違わない）。
- P4-v2 に適用: **規則数は 119 のまま、646 → 607 行**、11 雛形・32 実体化。
  v1 parser との応答は **28/28 完全一致**（`examples/p4-v2/scripts/compare.sh`）。
- corpus は **Java Pass 1310 / Fail 0、Rust Pass 1292 / Fail 0**（家族表も契約と一致）。

## 1. 表層構文

```ubnf
@ubnf: 2

VarDecl<Hint, Value> ::=
  ( 'variable' | 'var' ) '$' IDENTIFIER @varName
  [ Hint ]
  [ 'set' [ OnlyIfAbsent @onlyIfAbsent ] Value @value ]
  ';' ;

@declares(symbol=varName, description=desc)
@mapping(NumberVariableDeclarationExpr, params=[varName, onlyIfAbsent, value, desc])
NumberVariableDeclaration ::= VarDecl<NumberTypeHint, NumberExpression> ;
```

### 決めたこと

| 論点 | 決定 | 理由 |
|---|---|---|
| 宣言 | 規則名の直後に `<P1, ..., Pn>` | v1 では規則名の次は `::=` しか来られないので、`<` は**v1 で構文エラーだった位置**。D-038 の「v1 は凍結」を字面で満たす |
| 参照 | 本体の中では**裸の識別子**（`Hint`） | 規則参照と同じ書き味。`$X` のような別記法は UBNF に前例が無い |
| 実引数 | **atom** 1 つ（規則参照・token・リテラル・`( )` `[ ]` `{ }`） | 量指定子や capture 付きの要素を渡せると「実引数の量指定子と実体化側の量指定子」が二重になる。`( X+ )` と括れば同じことができ、括弧は手で書いたときと同じ形で IR に出る |
| 再帰 | **一切禁止**（`E-TEMPLATE-RECURSION`） | §3 |
| 注釈 | 雛形には置けない（`E-TEMPLATE-ANNOTATION`）。実体化する**規則**に書く | AST は実体化した規則のもの。雛形側に `@mapping` を置くと「どの実体化のクラスか」が決まらない |
| capture | 雛形の中に書き、**実体化した規則のフィールドになる** | §2。これが Menhir の `%inline` 側を採る理由そのもの |
| 名前のマングリング | **無い**（規則を作らないので） | 診断・`rulePath`・self-host 経路に新しい名前が 1 つも現れない |
| 版 | `@ubnf: 2` 限定（`E-TEMPLATE-UNSUPPORTED`） | 雛形は**規則宣言の形**を変える。規則宣言の形は自己記述文法 `ubnf.ubnf`（別 repo・読み取り専用）が固定しているので、v1 の表層に足すと「v1 の文法を v1 の自己記述で書けない」ことになる。左再帰（D-039）と同じ線引き |

### なぜ Menhir の `separated_list(sep, X)` 形（新しい非終端を作る）にしなかったか

UBNF は `@mapping(Class, params=[...])` で**規則ごとに AST クラスを決める**。
雛形を非終端にすると、実体化ごとに新しい規則＝新しい AST ノードが生まれ、
呼び出し側の capture はその**ノード 1 つ**を指すようになる。P4 でいえば

```
@mapping(MinExpr, params=[first, rest])
MinFunction ::= 'min' '(' ArgumentExpression @first { ',' ArgumentExpression @rest } ')' ;
```

が `MinExpr{args: ArgumentExpressionList}` に変わる。P4 の 15 箇所のカンマ区切りのうち
12 箇所が「リストが親規則のフィールドとして展開されている」形なので、**非終端版は
既存文法へ後付けできない**（AST の消費側は tinyexpression 側にあり、こちらから直せない）。
Menhir が `%inline` を別に持っているのは同じ理由である。UBNF は `%inline` 側だけを採った。

## 2. 単相化（lowering）と等価性

`crates/ubnfc-front/src/templates.rs`。`desugar::file` から、**左再帰の脱糖より前**に呼ぶ
（雛形が左再帰ルールの演算子や基底候補を供給できるように）。

1. 雛形を集め、名前の衝突・パラメータの重複・**パラメータが規則 / token を隠していないこと**を検査する。
   最後の条件が、名前による代入が**変数捕獲を起こさない**ための必要十分条件である。
2. 依存グラフ（雛形 → その本体が実体化する雛形）の**循環を検出**する（§3）。
3. 循環が無い＝ DAG なので、**逆トポロジカル順**に雛形本体そのものを先に展開する。
   これで規則本体の展開は 1 段で止まる（停止性の構成的な証明）。
4. 規則本体を展開する。実引数は呼び出し側の環境で先に展開する（call-by-value）。

### 差し込みの規則（手書きと 1 文字も変わらないための条件）

| 実体化の位置 | 展開 |
|---|---|
| 本体（または `( )` `[ ]` `{ }` の中身）**まるごと**が実体化 | 雛形の**候補列をそのまま**その本体にする（括弧を増やさない） |
| 列の途中で、量指定子・区切り・capture・`@typeof` が付いておらず、雛形の候補が 1 つ | **要素列をその場へ差し込む** |
| それ以外（量指定子や capture が付く、候補が複数） | `( ... )` で括る（手で書くときと同じ形） |

### 範囲（span）を実体化の位置へ揃える

展開した木の範囲は**実体化を書いた位置**にする。理由は 2 つ:

- `ExprId` は `expr:<source>:<start>:<end>:<role>` なので、同じ雛形を二箇所で実体化すると
  雛形本体の範囲のままでは**ID が衝突する**（役割の path は規則をまたいで同じになり得る）。
- 文法作者への診断は「どの実体化か」を指すほうが役に立つ。

これに伴い、v1 の lower が「範囲の先頭文字が `[` か」という**原文の覗き見**で
`[ X ]` と `X?` を区別していた箇所を、構文木が持つ事実（`ResolvedAtom::Optional { bracket }`）に
置き換えた。実在の 24 文法・P4 に `[X]?` は 1 つも無く、IR golden・corpus・self-host 固定点は
すべて不変である（両者が食い違う形は `[X]?` の外側 `?` だけ）。

### 等価性の証拠

- `crates/ubnfc-front/tests/templates.rs`（21 件）。中心は
  「雛形版と手書き版で **IR の範囲以外が完全一致**」を 7 つの形（分離リスト・列の途中・
  複数候補・`[ ]`・括弧つき実引数・雛形の入れ子・量指定子つき実体化）で確かめる。
  `ExprId` が重複しないことも固定する。
- **P4-v2 の全体**（132 symbol・1,045 式）: 正規化後 117,940 行の IR が**1 行も違わない**。
  規則名・役割 path・`@mapping`・capture・memo 安全性・解析・計画がすべて一致する。
- **応答**: `examples/p4-v2/scripts/compare.sh` で v1 parser と 28 入力を突き合わせ、
  accepted / 両カーソル / AST / 診断が **28/28 一致**（16 の受理入力と 12 の拒否入力）。

## 3. 拒否するもの

| コード | 形 | 理由 |
|---|---|---|
| `E-TEMPLATE-UNSUPPORTED` | `@ubnf: 1` で雛形を書いた | v1 は凍結（§1） |
| `E-TEMPLATE-RECURSION` | 雛形が直接・相互に自分を実体化する | **展開はその場での置換なので、どんな循環も無限展開になる**。雛形の依存グラフは有限なので、循環の有無は**完全に判定できる**（下界ではない）。「引数が育つかどうか」を見る必要はなく、循環そのものを禁じれば停止性は構成的に保証される。再帰が要る形は**通常の規則**で書ける（雛形の実引数にその規則を渡すのは循環ではない。試験 `recursion_through_an_ordinary_rule_is_allowed`） |
| `E-TEMPLATE-ARITY` | 実引数の個数が宣言と違う | — |
| `E-TEMPLATE-UNKNOWN` | 宣言の無い名前を実体化した | 規則は実体化できない（規則と雛形は別の名前空間） |
| `E-TEMPLATE-HIGHER-ORDER` | パラメータを雛形として実体化した（`Apply<F, A> ::= F<A>`） | 高階は未対応。名前はいつも書かれたとおりに解決する |
| `E-TEMPLATE-PARAM-SHADOW` | パラメータ名が規則 / token と同じ | 代入は名前で行うので、これを許すと捕獲が起きる |
| `E-TEMPLATE-PARAM-DUPLICATE` | パラメータ名の重複 | — |
| `E-TEMPLATE-NAME-COLLISION` | 雛形名が規則 / token / 他の雛形と重複 | — |
| `E-TEMPLATE-ANNOTATION` | 雛形に注釈を付けた | AST は実体化した規則のもの（§1） |
| `W-TEMPLATE-PARAM-UNUSED` | 本体で使われないパラメータ | 警告 |
| `W-TEMPLATE-UNUSED` | 実体化されない雛形 | 警告 |

展開に失敗した実体化は `ERROR('未展開の雛形 X')` に置き換える。E-* は既に積んであるので
コンパイルは必ず失敗するが、こうしないと resolve が `E-RULE-UNDEFINED` で先に落ちて
**本当の理由を隠す**。

## 4. P4-v2 での実測

`examples/p4-v2/tinyexpression-p4-v2.ubnf` を 11 雛形・32 実体化に書き換えた。

| | 左再帰まで（D-039/D-041） | 雛形も使う（D-043） |
|---|---:|---:|
| 規則数 | 119 | **119**（変わらない） |
| 行数 | 646 | **607** |
| 雛形宣言 | 0 | 11（うち本体 38 行 + 見出し 6 行） |
| 実体化 | 0 | 32 |

**規則は 1 つも減らない**。雛形は規則を作らないので当然で、減るのは
「同じ形を何度も書くこと」だけである。実際に減った形:

| 雛形 | 実体化 | 元の重複 |
|---|---:|---|
| `VarDecl<Hint, Value>` | 4 | `{Number,String,Boolean,Object}VariableDeclaration` が **2 要素しか違わない 9 行**を 4 回 |
| `MethodDecl<Ret, Body>` | 4 | 同上（`{…}MethodDeclaration`、11 行 × 4） |
| `ExternalInvocation<Ret>` | 3 | `External{Boolean,String,Object}Invocation`（Number 版だけ形が違うので残した） |
| `Slice<Recv>` | 2 | **8 候補のスライス構文が丸ごと 2 回**（受け手だけが違う） |
| `MatchBody<Case, Default>` | 3 | `{Number,String,Boolean}MatchExpression` |
| `CaseRule<Value>` / `DefaultCaseRule<Value>` | 3 + 3 | `{…}Case` / `{…}DefaultCase` |
| `ValuesList<X>` | 2 | `MethodParameters` / `Arguments` |
| `NAryNumberFn<Kw>` | 2 | `min` / `max`（**リテラルを実引数に渡す**形） |
| `StringPredFn<Kw>` / `StringPredDot<Kw>` | 3 + 3 | `startsWith` / `endsWith` / `contains` の関数形とドット形 |

### 正直に書いておくこと

- **`%`（分離リスト）の穴埋めとしての価値はほとんど無かった。** カンマ区切りは実在の
  25 文法（unlaxer の 24 + P4）のうち **P4 に 15 箇所、`ubnf.ubnf` に 5 箇所、残り 23 文法には 0 箇所**
  しかない。しかも 20 箇所の capture 名が `@values` `@patterns` `@candidates` `@first`/`@rest`
  `@firstCase`/`@moreCases` と**ばらばら**で、capture 名を雛形に書き込む以上
  「1 つの `SepList` で全部」にはできない（capture 名をパラメータにする案は、
  `@name` を読めば AST のフィールドが分かるという UBNF の性質を壊すので採らなかった）。
  P4 で `ValuesList<X>` として括れたのは 15 箇所中 2 箇所である。
- **価値があったのは「型ごとの多重化」のほう**だった。P4 は number / string / boolean / object で
  同じ形を 3〜4 回書いており（`VariableDeclaration` 4 本、`MethodDeclaration` 4 本、
  `ExternalInvocation` 4 本、match 系 3 × 3 本）、それぞれ AST クラスが違うので**規則としては
  減らせない**が、**本体は 1 回書けば済む**。`Slice<Recv>` の 8 候補 × 2 本が一番分かりやすい。
- したがって「規則数が減る」ではなく「**同じ形が 1 箇所に書かれる**」が唯一の効能である。
  これは左再帰（D-039）の「規則数 124→119 より、優先順位が宣言的になることが価値」と同じ形の結論である。
- `ubnf.ubnf` は別 repo の読み取り専用なので書き換えていない。5 箇所のカンマ区切りは
  すべて separator も capture も違うので、雛形にしても 1 箇所も括れない。

## 5. 差分ファジングの被覆

`crates/ubnfc-fuzz`。v2 の seed 帯（`V2_SEED_BASE = 100_000` 以降）で

```ubnf
Tpl<A, S> ::= A @head { S A @tail } ;
@mapping(TplExpr, params=[head, tail])
TplWrap ::= '(' Tpl<Op, ','> ')' | Tpl<Op, ';'> ;
```

を必ず生成し、`TplWrap` を根の被演算子にする。要点:

- **モデルは展開後の形を持ち、原文だけが雛形の綴り**である。つまり生成器は
  「front の展開はこの木と一致するはずだ」と主張しており、外れれば**両 backend が同時に
  モデルと食い違って赤くなる**。
- 実体化は 2 箇所あり、**列の途中への差し込み**（`'(' Tpl<…> ')'`）と
  **候補まるごと**の両方を踏む。実引数は規則参照とリテラルの両方を使う。
- **乱数を 1 つも引かない**ので、既存 seed（seed 269 の A-1 回帰、4001..4004 の固定 campaign）の
  生成物は 1 バイトも変わらない。v1 の帯には雛形が 1 つも出ない
  （`parameterized_rules_only_appear_in_the_v2_seed_band`）。
- 縮小は「雛形の綴りをやめて展開後を直接書く」候補を 1 手として持つので、
  乖離が展開に由来するかどうかを切り分けられる。規則本体を触る縮小はその規則の綴りを外す。
- 固定 seed campaign は 4 本とも緑（`discarded == 0`、未登録の乖離なし）。

## 6. 検証

| 対象 | 結果 |
|---|---|
| `cargo test --workspace --offline` | 緑（`templates.rs` 21 件を含む） |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | 緑 |
| `cargo fmt --all -- --check` | 緑 |
| `cd ubnfc-java && mvn -o -q test` | 緑 |
| `cd examples/p4-java && mvn -o -q test` | 緑 |
| self-host 固定点（`cargo test -p ubnfc-selfhost`） | 緑（`generated_frontend_reaches_the_ir_and_rust_fixed_point`、`unmodified_golden_regenerates_every_committed_byte`） |
| corpus Rust | **Pass 1292 / Fail 0**、家族表は `2026-09-23-rust-corpus-contract.md` と一致 |
| corpus Java | **Pass 1310 / Fail 0**、家族表は `2026-09-23-java-corpus-contract.md` と一致 |
| IR golden | 追加フィールドのみ（`GrammarDecl.templates` は展開後に必ず空なので JSON に出ない） |
| P4-v2 応答（`scripts/compare.sh`） | **28/28 一致**（AST は読み替えなしの生比較でも一致） |

## 7. 残っている宿題

- **高階パラメータ**（`Apply<F, A> ::= F<A>`）。今は拒否している。入れるなら
  「雛形を値として渡す」ことになり、循環検出が引数の流れを追う必要がある。
- **capture 名をパラメータにする案**。`SepList<X, Sep, @head, @tail>` にすれば P4 の
  カンマ区切り 15 箇所を 1 つに括れるが、`@name` を読めば AST のフィールドが分かるという
  性質が壊れるので採らなかった。`%` の穴が本当に痛くなったときに再考する。
- 雛形の**別ファイル共有**（`@import` した grammar の雛形を使う）。今は grammar 内に閉じている。
