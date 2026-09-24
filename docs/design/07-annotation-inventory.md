# UBNF アノテーション棚卸しと v2 統廃合案

> 2026-09-23。`06-ubnf-v2.md` §6 の作業。**棚卸しと提案であって実装ではない**。
> 数字はすべて再現可能な grep/JSON 走査で採った（§1 に採り方）。
>
> **実装状況（2026-09-23 追記）**: P1 / P2 / P3 / P5 と P7 の 1・3 は
> `docs/reports/2026-09-23-annotation-consolidation.md` で実装済み（D-045 〜 D-049）。
> P7-2 は**誤りだった**（Java にも上限検査はある）。P7-4 と P4 / P6 は未着手。
> 各節の先頭に **[済]** / **[誤]** / **[未]** を付けた。

## 0. 要約

- 名前の付いた表層は **26**（rule/element 注釈 20 + grammar 設定 5 + directive 1）。
  加えて `@<任意の識別子>` を黙って受理する catch-all（`Annotation::Simple`）がある。
- **実言語 18 文法**（合成カバレッジ fixture を除く）で使われていないのは **11**:
  `@enum` `@commonField` `@typeof` `@skip` `@doc` `@eval` `@recovery` `@import`
  `@longestChoice` `@predictiveChoice` `@literalBoundary`。
- **どの文法にも 1 度も現れない**のは **3**: `@longestChoice` / `@predictiveChoice` / `@literalBoundary`
  （前 2 つは corpus oracle にだけ存在し、後者は v2 既定で不要）。
- **生成コードに届かない**（front で IR に載るが両 backend が読まない）のは **3**:
  `@doc`、`@precedence` の `level` 値、`@leftAssoc`/`@rightAssoc` の `associativity` 値。
- **片側だけ／宣言と実態がずれている**のは **6**:
  `@interleave` の 2 profile が同一物に落ちる、`@scopeTree(mode=dynamic)` が lexical と同じ解析時 stack、
  `@backref` が文法内の遠い `@scopeTree` で意味を変える、`@precedence` が `@rightAssoc` では `operator` を作らない、
  `@package` を Rust backend が読まない、`@ubnf` の上限検査が Java backend に無い。
- front の注釈ディスパッチは `crates/ubnfc-front/src/syntax/parser.rs:361-482`（規則注釈 19 + catch-all）、
  要素注釈 `@typeof` は `parser.rs:548-561`、grammar 設定はキー自由の `parser.rs:194`、
  `@import` だけ宣言として `parser.rs:150-166`。注釈まわりの検証エラーコードは **40 以上**ある。

## 1. 数え方

| 母集団 | 内容 | 件数 |
|---|---|---|
| 24 fixture | `unlaxer-parser` 配下の `*.ubnf` 全部（`ubnf.ubnf` を含む） | 24 |
| 実言語 | 上のうち、合成カバレッジ fixture 8 件（`annotations` `eval-default` `common-field-space` `elements` `all-tokens` `boundaries` `namespace` `keyword-boundary`）を除いたもの + P4 + P4-v2 | 18 |
| P4 / P4-v2 | tinyexpression の 124 規則文法と、その v2 移植 | 2 |
| corpus oracle | `corpus/oracle/*.json` の inline 文法（Java 1310 / Rust 1292 ケースの出どころ） | 108 |
| 差分 fuzzer | `crates/ubnfc-fuzz/src/gen.rs` が生成する文法 | — |

数える前に `'...'` リテラルと `//` コメントを落としている。`ubnf.ubnf` は UBNF 自身のメタ文法で、
`'@mapping'` のような**注釈名の文字列リテラル**を大量に含むため、これをしないと 2 倍に見える
（`ubnf.ubnf` の `@mapping` は生の grep で 51、リテラル除去後は 49。
`@longestChoice` / `@predictiveChoice` は生の grep で 1 件ずつ出るが、どちらもメタ文法の
リテラルであって使用ではない — これが「実使用ゼロ」の根拠である）。

## 2. 全表

列の意味:
- **front**: `crates/ubnfc-front` が受理して IR のどこへ載せるか
- **Rust** / **Java**: その IR キーを読んで**生成コードを変えるか**（読まないなら「無視」）
- **実言語**: 実言語 18 文法での出現数 / 文法数
- **corpus**: corpus oracle 108 JSON での出現数 / ファイル数

### 2.1 rule 注釈

| 注釈 | 意味 | 決定行 | front | Rust | Java | 実言語 | corpus |
|---|---|---|---|---|---|---|---|
| `@root` | entry 規則 | D-021 | `rules[].annotations` | 実装 `emit.rs:279` | 実装 `JavaEmitter.java:273` | 18/18 | 111/107 |
| `@mapping(C, params=[…])` | 規則→AST 型 | D-006, D-028, D-032, D-039 | `rules[].mappingTypeId` / `mappingParams` / `semantics.astTypes` / `recipes` | 実装 | 実装 | 268/17 | 182/106 |
| `@enum` | 候補列を enum 型に | — | `AstType::Enum` + `RecipeKind::Enum` + `requiredFeatures` | 実装 (`ast_types`) | 実装 | 0/0 | 9/4 |
| `@commonField(a,b)` | sealed interface へ field を引き上げ | — | `semantics.commonFields` | 実装 `mapping.rs` | 実装 | 0/0 | 9/4 |
| `@skip` | AST から除外 | D-032 | `rules[].skip` | 実装 | 実装 | 0/0 | 2/1 |
| `@doc('…')` | 説明文 | D-062 | `rules[].annotations` のみ | **無視** | **無視** | 0/0 | 0/0 |
| `@catalog(context='…')` | 補完カタログ metadata | — | `semantics.catalogs` | 実装 | 実装 | 2/2 | 9/4 |
| `@eval(k='v', …)` | 評価器の種別/戦略 | — | `rules[].eval` | 実装 | 実装 | 0/0 | 19/8 |
| `@backref(name=X)` | scope 参照 **または** 同一 rule 内テキスト一致 | D-029, §3.8 | `scope.effects[].events[].action=use` + `mode` | 実装（`sameRuleText` 含む） | 実装 | 5/3 | 6/5 |
| `@declares(symbol=X[, description=…])` | scope へ宣言登録 | D-029, §3.8 | `scope.effects[].events[].action=declare` | 実装 | 実装 | 19/3 | 13/9 |
| `@scopeTree(mode=lexical\|dynamic)` | 規則境界で scope enter/leave | D-029, §3.8 | `scope.effects[].mode` | 実装（**dynamic は lexical と同じ**） | 同左 | 10/2 | 11/9 |
| `@recovery(sync='…'\|auto\|skip)` | エラー回復 | D-021, D-031, D-017 | `semantics.recovery`（mode/同期集合/consumeSync/noSyncAction を front が確定） | 実装 | 実装 | 0/0 | 8/4 |
| `@whitespace[(style)]` | trivia 有効/無効（rule 上書き） | D-030, §3.5 | `trivia.policies[].rawSettings.whitespace` → `rules[].triviaPolicyId` | 実装 | 実装 | 17/17 | 17/14 |
| `@interleave(profile=…)` | trivia 有効（rule 上書き、旧経路） | D-030, §3.5, compat C10 | 同上 `rawSettings.interleave` | 実装（**whitespace と同一物**） | 同左 | 6/2 | **0/0** |
| `@precedence(level=N)` | 演算子優先度 | D-004, D-039 | `rules[].operator.precedence` | **値は無視** | **値は無視** | 13/5 | 1/1 |
| `@leftAssoc` | 左結合宣言 | D-004, D-039 | `rules[].operator`（この注釈が `operator` を作る唯一の引き金） | 値は無視 | 値は無視 | 12/4 | 1/1 |
| `@rightAssoc` | 右結合宣言 | D-039 | `rules[].annotations`（**`operator` は作られない**） | 実装 `emit.rs:353` | 実装 `JavaEmitter.java:360` | 1/1 | 0/0 |
| `@longestChoice` | 最長一致選択（**意味論**） | §3.4 | `selection=longest`（**規則本体の最上位 Choice だけ**、`lower/expr.rs:134`） | 実装 `expressions.rs:133` | 実装 `JavaEmitter.java:510` | **0/0** | 5/2 |
| `@predictiveChoice` | 予測分岐（**最適化**） | D-037, §3.4 | `selection=predictive` → `plan.choices[].strategy=predict` | 実装 `predict.rs:807` | 実装 `Prediction.java:402` | **0/0** | 12/5 |
| `@typeof(capture)`（要素注釈） | mapping 時の実行時型一致検査 | D-011, §3.6 | `semantics.typeofConstraints` + `requiredFeatures.typeof` | 実装 `mapping.rs:270` | 実装 | 0/0 | 4/1 |
| `@<任意の名前>` | catch-all。黙って metadata 化 | — | `Annotation::Simple` → `annotations[]` | 無視 | 無視 | 1（`@rooted`） | — |

### 2.2 grammar 設定・directive

| 設定 | 意味 | 決定行 | front | Rust | Java | 実言語 | corpus |
|---|---|---|---|---|---|---|---|
| `@package: …` | 生成 package 名 | — | `grammarUnits[].package` | 該当なし | 実装 | 18/18 | 104/104 |
| `@whitespace: javaStyle\|none` | 既定 trivia | D-030 | `trivia.policies` | 実装 | 実装 | 17/17 | 17/14 |
| `@comment: { line: … blockOpen/blockClose: … }` | コメント区切りの追加 | D-018, D-030 | `effectiveDelimiterSet` | 実装 | 実装 | 2/2 | 20/7 |
| `@memoSafeToken: ALIAS` | token 単位の memo 安全保証 | §3.7 | `tokens[].memoSafety` | 実装 | 実装 | 12/2 | 0/0 |
| `@ubnf: 1\|2` | 表層バージョン門 | D-038 | `languageVersion`（必須キー） | 実装（未知版を拒否） | 実装 | 1/1 | 0/0 |
| `@literalBoundary: word\|none` | 単語境界の既定上書き | D-040 | `literal.wordBoundary` / `TokenSpec.caseInsensitive.wordBoundary` | 実装 | 実装 | **0/0** | 0/0 |
| `@import X from '…'` | 別文法の取込み | D-017, D-018 | 統合後の単一シンボル表 + `importEdges`（来歴 metadata） | **無視** | **無視** | 0/0 | 9/3 |

`@package` は **Rust backend が読まない**（`grammarUnits[].package` の参照 0 件。Rust は CLI 引数で決める）。
`@ubnf` の上限検査は **Rust だけ**にある（`crates/ubnfc-rust/src/validate.rs:13` の `MAX_LANGUAGE_VERSION=2`）。
Java backend に相当する検査が見当たらず、D-038 の「両 backend が未知バージョンを明示拒否する」と食い違う。

**注釈由来で誰も読まない IR キー**: `rules[].mappingParams`（両 backend とも `recipes[].fields` を使う）、
`recovery[].progressRule`、`trivia.policies[].rawSettings`、`rules[].operator.*` の全 field、
`plan.operators[].boundaryExprIds` / `liftRecipeIds` / `proof`。

### 2.3 差分 fuzzer が生成する注釈

`crates/ubnfc-fuzz/src/gen.rs` と `model.rs` は次を生成する:
`@ubnf` `@whitespace`(3 形) `@comment` `@package` `@enum` `@skip` `@mapping`(3 形) `@commonField`
`@interleave`(2 profile) `@recovery`(3 形) `@longestChoice` `@predictiveChoice` `@scopeTree`(2 mode)
`@declares` `@backref` `@catalog` `@doc` `@leftAssoc` `@rightAssoc` `@precedence` `@root` `@eval` `@typeof` `@import`。
**生成しない**のは `@memoSafeToken` と `@literalBoundary` の 2 つだけ。
つまり表層を触る提案はほぼ全部 fuzzer の generator に波及する。

## 3. 関心事によるグループ分け

| 関心 | 注釈 | 数 |
|---|---|---|
| 解析戦略 | `@longestChoice` `@predictiveChoice` `@memoSafeToken` | 3 |
| trivia | `@whitespace`(rule/global) `@interleave` `@comment` | 3 |
| AST / 型 | `@mapping` `@enum` `@commonField` `@skip` | 4 |
| 意味制約 | `@typeof` `@backref`(テキスト側) | 2 |
| scope | `@scopeTree` `@declares` `@backref`(scope 側) | 3 |
| 演算子 | `@precedence` `@leftAssoc` `@rightAssoc` | 3 |
| 評価 | `@eval` | 1 |
| ツーリング metadata | `@doc` `@catalog` | 2 |
| エラー回復 | `@recovery` | 1 |
| 構造・版 | `@root` `@package` `@import` `@ubnf` `@literalBoundary` | 5 |

### 3.1 本物の重複

1. **trivia が 2 系統ある**。`@interleave` は `@whitespace` と同じ `effectiveDelimiterSet` に落ちる。
   P4 の IR で実測すると、`@interleave(profile=javaStyle)` 由来の `trivia:javaStyle` と
   grammar `@whitespace: javaStyle` 由来の `trivia:javaStyle:TinyExpressionP4::CodeBlock` は
   **区切り集合が byte 一致**し、違うのは `rawSettings`（来歴）と ID 文字列だけ。
   さらに 2 つの profile `javaStyle` / `commentsAndSpaces` は front で値の検証だけされ
   （`validate/annotations.rs:75`）、下流でまったく区別されない。
   rule-trivia 契約が「明示 `@whitespace` が `@interleave` に勝つ」と決めているので、
   両方書いた規則では `@interleave` は常に無効である。
   さらに悪いことに、`rawSettings` が違うだけで policy が重複排除されない（`lower/trivia.rs:126`）ため、
   **意味的に同一な TriviaPolicy が 2 つ残り、backend が同内容の `Delimiters` 定数を二重に生成する**。
2. **演算子の宣言が 3 つある**のに、生成に効くのは 1 つ半。`@leftAssoc` + `@precedence` が
   揃ったときだけ `rules[].operator` が作られ、`@rightAssoc` + `@precedence` では `operator` が **null**
   になる（`Power.ubnf` の IR で確認）。`operator.precedence` と `operator.associativity` を
   読むコードは**両 backend に 1 行も無い**（`.precedence()` の参照 0 件）。`operator` は
   harness の driver 生成（`driver.rs:64`、`JavaEmitter.java:300`）にしか使われていない。
   `plan.operators` は 14/14 すべて `strategy=tierFunctions` かつ `proof.kind=none`、
   つまり D-004 の precedence climbing は一度も選ばれていない。
3. **選択戦略が「注釈」と「解析結果」で二重**。IR fixture 23 件の `selection` は **82/82 が `ordered`**、
   一方 front が自前で決める `plan.choices[].strategy` は `ordered` 66 / `predict` 16、
   `requiredFeatures` に `predictiveChoice` が載る fixture は 9 件。
   **注釈を 1 つも使っていない文法で予測分岐が 16 箇所選ばれている**＝ front はすでに自力で決めている。
   格上げの条件は `analysis/choices.rs:135`：候補が非 nullable・trivia なしで `mayStart` が
   pairwise disjoint なら `proof.kind=exclusiveCandidates` を添えて `strategy=predict` にする。
   なお `selection` は **規則本体の最上位 Choice にしか付かない**（`lower/expr.rs:134` の `top &&`）。
   入れ子の `( a | b )` には注釈を付ける手段そのものが無く、annotations.md はそれを書いていない。

### 3.2 本物の欠落・約束違反

1. `@backref` の意味が**文法内の遠い `@scopeTree` の有無**で変わる（`validate/mod.rs:75` は
   「文法中のどれか 1 つの規則に `@scopeTree` があるか」を見ている）。
   - scope あり: capture を scope store へ参照登録、未定義は WARNING で構文成功
   - scope なし: 同一 rule 内のテキスト一致比較、不一致は ERROR
   副作用として、**どこか 1 規則に `@scopeTree` を足すと文法中の全 `@backref` の意味が黙って変わる**。
   さらに capture 名の検証 `E-ANNOTATION-BACKREF-CAPTURE` は scope モードでしか発火しない
   （`validate/annotations.rs:99` の `if scope`）ので、テキストモードでは綴り間違いが素通りする。
   corpus は「scope なし `@backref`」の oracle を 1 件も持っていない（`compat-matrix.md:95`）。
2. `@scopeTree(mode=dynamic)` は受理・lowering されるが、仕様自身が「両 mode とも解析時 stack」と
   書いている。実言語での使用は 0（`lexical` のみ 10 件）、corpus に 3 件。
3. `@interleave(profile=commentsAndSpaces)` は検証を通るが `javaStyle` と区別されない。
   実使用は 0（`javaStyle` 6 件 + 合成 fixture の不正値 `comments` 1 件）。
4. `@eval` の引数は**未規定の小言語**。実測で現れた key は `kind` `strategy` `operation` `result`
   `strip_prefix` の 5 つで、未知 key の検証は無い。自動 kind
   （`binary_arithmetic` / `variable_ref` / `conditional` / `passthrough` / `literal`）は
   規則の形に依存するのに、それが型注釈系（`@mapping`）と結び付いていない。
5. **未知の注釈が黙って通る**。`Annotation::Simple`（`lower/annotations.rs:50`）は
   `@<任意の識別子>` を metadata として受理する。実例として
   `rust/unlaxer-ubnf/tests/fixtures/known/keyword-boundary.ubnf` の `@rooted`（`@root` の綴り間違い）が
   **診断ゼロで**通り続けている。しかも `@root` が無いので entry は
   `lower/mod.rs:359` の `.or_else(|| grammar.rules.first())` で「最初の規則」に落ち、
   たまたま正しく動いているだけである（`W-GENERAL-NO-ROOT` は出る）。
   合成 fixture の `@custom` も同じ経路。未知の **grammar 設定**（`@foo: bar`）も同様に素通りする。
6. `@doc` は IR に載るが、両 backend が `annotations[]` を見るのは `root` と `rightAssoc` の 2 つだけなので
   生成コードに届かない。LSP 用 metadata としては筋が通るが、「生成に効く」とは書けない。
   **追記（2026-09-24、D-062）**: その「LSP 用 metadata」として実際に読む側ができた。
   `ubnfc-lsp` が生成時に `rules[].annotations` の `doc` を読み、hover の説明文の表
   （生成される `Meta.RULE_DOC`）へ畳み込む。**IR にも front にも手を入れていない**
   （`semantics.rules[].doc` のような欄を足すと `ir/fixtures/` の IR golden すべてに `"doc": null` が
   増えるだけで、読める情報は同じ）。パーサ backend の欄が「無視」なのは変わらない。
7. **`@rightAssoc` の span 補正条件が両 backend で非対称**。Rust（`emit.rs:353`）は
   `rightAssoc` **かつ** `interleave` / `whitespace` / `comment` 注釈が同じ規則に同居するときだけ
   span 終端を `significant_end` に切り替えるが、Java（`JavaEmitter.java:360`）は `rightAssoc` の
   真偽だけを見る。trivia 注釈を持たない `@rightAssoc` 規則で node span が食い違い得る。
   corpus の `@rightAssoc` oracle は 0 件なので、この差は現行テストでは光らない。
   これは統廃合の前に**独立した不具合**として起票すべきものである。

## 4. 統廃合案（順位付き）

各案に **採否・置き換える対象・移行・影響範囲**を付ける。
すべて **`@ubnf: 2` 限定**で、v1 は 1 bit も変えない前提（D-038）。

### P1（**[済]** D-045）未知注釈の警告 — `W-ANNOTATION-UNKNOWN`

- **形**: `@ubnf: 2` で、既知でない注釈名に警告を出す。`--strict` でエラー化（D-037 と同じ扱い）。
- **置き換える対象**: `Annotation::Simple` の「黙って受理」。受理自体は残す（拒否しない）。
- **移行**: 無い。既存文法は警告が出るだけ。`@rooted`（keyword-boundary.ubnf）と `@custom`
  （annotations.ubnf）の 2 件が v2 に移行したときだけ光る。
- **影響範囲**: 決定行の追加 1 行（D-045 相当）。IR 変更なし。backend 変更なし。
  fuzzer は `@ubnf: 2` 文法で未知注釈を生成していないので変更不要。corpus は v1 のまま無影響。
- **なぜ 1 位か**: 実際に綴り間違いを 1 件見つけている（`@rooted`。しかも entry の暗黙 fallback で
  隠れていた）、コストが最小、他のどの案よりも先に入れておくと
  以降の非推奨警告（P2〜P5）の受け皿がそのまま使える。
- **合わせて**: 未知の grammar 設定（`@foo: bar`）にも同じ警告を出す。`lower/mod.rs:226` が
  `package` / `comment` / `whitespace` / `interleave` 以外を無検査で `grammarUnits[].annotations` へ
  流している。

### P2（**[済]** D-046）`@backref` を意味ごとに 2 つへ割る

- **形**: `@symbolRef(name=X)`（scope 検索・未定義は WARNING）と `@sameText(name=X)`
  （同一 rule 内テキスト一致・不一致は ERROR）。v2 では `@backref` に
  `W-ANNOTATION-BACKREF-AMBIGUOUS` を出し、上の 2 つへ誘導する。
- **置き換える対象**: 「文法中のどこかに `@scopeTree` があるか」という**非局所な意味決定**
  （`validate/mod.rs:75`、`lower/annotations.rs:267,321`）。
- **移行**: 実在する `@backref` は 6 件（P4 2、P4-v2 2、ubnf.ubnf 1、合成 fixture 1）で、
  **全部が `@scopeTree` を持つ文法**にある。よって全件が `@symbolRef` へ機械置換。
  `@sameText` の実使用は世界にゼロ。
- **影響範囲**:
  - 決定行: §3.8 と D-029 の文言（「scope なし `@backref` は…」）を注釈名で書き直す。
  - IR: **変更なし**。`ScopeEffect.mode` はすでに `lexical` / `dynamic` / `sameRuleText` へ
    解決済みで、front が 2 つの表層名を同じ field へ落とすだけ。backend は 1 行も変わらない。
  - 副次効果: `E-ANNOTATION-BACKREF-CAPTURE` をテキストモードにも効かせられる（今は素通り）。
  - fuzzer: `gen.rs:576` の `@backref` 生成を v2 では `@symbolRef` にする。
- **コスト**: front の parser に 2 つの注釈を足し、既存の `Backref` を残す。小。

### P3（**[済]** D-048）`@interleave` を `@whitespace` に畳む

- **形**: v2 では `@interleave(profile=…)` に `W-ANNOTATION-INTERLEAVE-DEPRECATED` を出し、
  `@whitespace(javaStyle)` へ誘導する。v3 で削除。**v2 でも拒否はしない**（D-042 と同じ理屈で、
  唯一の実利用者 P4 は別 repo にある）。
- **置き換える対象**: trivia の 2 系統目。profile 値 2 つ（`javaStyle` / `commentsAndSpaces`）は
  どちらも同じ区切り集合になるので、情報量はゼロ bit。
- **移行**: `@interleave(profile=X)` → `@whitespace(javaStyle)`。実在は 7 箇所（P4 3、P4-v2 3、
  合成 fixture 1）だけ。同じ規則に `@whitespace` も書いてある場合は `@interleave` を消すだけ
  （rule-trivia 契約により元から無効）。
- **影響範囲**:
  - 決定行: §3.5、D-030、`docs/rule-trivia.md` の優先順位節、`compat-matrix.md` の C10 行。
  - IR: v1 では**変更しない**（`trivia.policies[].rawSettings.interleave` と policy ID の綴りが
    self-host の byte 一致に効くため）。v2 で `@interleave` を書かなくなれば
    `rawSettings.interleave` が null になるだけで、`effectiveDelimiterSet` は変わらない。
    P4-v2 の IR 差分は `rawSettings` と `policyId` の 2 キーに収まる見込み。
  - corpus: `@interleave` の oracle は **0 件**なので corpus は動かない。
  - fuzzer: `gen.rs:539-541` が 2 profile を生成している。v2 分岐で `@whitespace` に寄せる。
- **注意**: 畳んだあと `policyId` の綴りが変わると self-host 固定点の byte 比較に出る。
  policy ID の採番規則は触らないこと。

### P4（**[未]**・採用だが「省略可能」に留める）`@precedence(level=N)` を任意にする

- **形**: v2 では `@precedence` を省略してよい。省略時、front が**塔の参照深さ**（v1 の正規形）または
  **候補順**（v2 左再帰、D-039）から `operator.precedence` を採番する。
  `@leftAssoc` / `@rightAssoc` は候補先頭マーカーとして残す（D-039 がすでにその位置に置いている）。
- **置き換える対象**: `@leftAssoc` は `@precedence` を伴う **MUST**（annotations.md の契約 3/4）。
  実体は検証済み metadata でしかなく、両 backend は値を読んでいない。
- **なぜ「削除」でなく「省略可能」か**: 削除すると `operator` ブロックを作る引き金が消え、
  harness driver の `__OPERATOR_TYPES__` / `operator_ast` 出力が変わる。段階を分ける。
- **検証コストの比**: `@precedence` は front で**三重に検証される**
  （`validate/operators.rs:153-249` の 4 コード、`lower/annotations.rs:222` の範囲検査、
  `analysis/operators.rs:172` の整合再検査）のに、生成物への影響はゼロ。
  棚卸しした中で**最も重い dead surface** である。実際にコードを動かしているのは
  `@leftAssoc`/`@rightAssoc` 由来の `semantics.recipes[].kind="assocLists"` と `leafFallback`
  （`mapping.rs:87` / `JavaEmitter.java:358`）で、こちらは残す。
- **移行**: 既存文法は書いたままで通る（純粋な緩和）。新しい文法は書かなくてよい。
  P4-v2 はすでに `@precedence` を 0 箇所に減らしている（v1 の P4 は 6 箇所）。
- **影響範囲**:
  - 決定行: D-004（precedence climbing の入力が注釈でなく塔の形になる）、D-039 に 1 行追加、
    `annotations.md` の `@precedence` MUST 契約 3〜6。
  - IR: `operator` の中身は同じ。採番規則を front に足すので `analysisVersion` を上げる必要がある
    （既存文法の `precedence` 値が注釈と一致する限り IR は byte 同一に保てる）。
  - corpus: `@precedence` の oracle は 1 ファイル（`eval-binary_arithmetic.json`）だけ。
  - fuzzer: `gen.rs:659` が `@precedence(level=…)` を必ず付けている。省略形も生成させたい。
- **合わせて直すべき非対称**: `@rightAssoc` + `@precedence` が `operator` を作らない件
  （`Power.ubnf` の IR が `operator=None`）。これは案というより不具合に近い。

### P5（**[済]** D-049）`@predictiveChoice` を非推奨にする / `@longestChoice` は残す

- **`@predictiveChoice`**: v2 で `W-ANNOTATION-PREDICTIVE-REDUNDANT`。
  **根拠**: front は注釈と無関係に `plan.choices[].strategy=predict` を 16 箇所で選んでおり
  （`requiredFeatures.predictiveChoice` を持つ fixture 9 件）、一方どの文法もこの注釈を使っていない。
  意味論は `ordered` と同値（§3.4「診断は元の Choice と同値」）なので、消しても受理・AST・診断は変わらない。
  D-037 が「候補順を誤ると黙って死ぬ」を警告するようになったので、この注釈で誤魔化す動機も減った。
- **`@longestChoice`**: **残す**。最長一致は **受理する言語そのものが変わる**選択であって、
  解析から導出できない。使用実績 0・corpus 2 ファイルだが、導出不能なものを消すと表現力が減る。
  ただし「最適化」ではなく「意味論の選択」だと仕様に書き直す（`performance-tuning-ja.md` の
  「一般的な高速化ではない」という注意書きを規範側へ昇格）。
- **影響範囲**: corpus は v1 凍結なので `predictive*.json` 5 ファイルは動かない。
  fuzzer `gen.rs:556,559` は両方生成しているので v2 分岐で `@predictiveChoice` を外す。
  IR 変更なし。決定行は §3.4 に 1 行。

### P6（**[未]**・部分採用）型注釈系の整理

- **採用する部分**: `@enum` と `@commonField` を `@mapping` の下位形にする。
  - `@enum` → `@mapping(enum Mode)`（規則の候補列を enum 型にする、という宣言を型宣言の場所に集める）
  - `@commonField(a,b)` → `@mapping(Node, common=[a,b])`
- **却下する部分**: `@typeof` を型注釈系に入れること。`@typeof(capture)` は型を**宣言**していない。
  「2 つの capture の実行時型が一致すること」を mapping 段で**検査**する制約であり、
  失敗しても構文は成功して mapping 失敗になる（D-011、`mapping.rs:299`）。
  これは `@backref` のテキスト一致（P2 の `@sameText`）と同じ族なので、
  **制約族**として `@sameType(a, b)` へ寄せるほうが筋が通る。要素注釈のまま残してもよい。
- **移行**: 実使用は `@enum` 1・`@commonField` 2・`@typeof` 1（すべて合成 fixture）。
  corpus には `@enum` 9 / `@commonField` 9 / `@typeof` 4 があるが v1 凍結。実質ゼロコスト。
- **影響範囲**: IR は**変更なし**（`AstType::Enum`・`semantics.commonFields`・`typeofConstraints` は
  そのまま）。front の parser のみ。fuzzer `gen.rs:435,478` と `model.rs:177`。
- **なぜ最下位か**: 現状は曖昧ではなく、ただ数が多いだけ。得られるのは概念数 4→2 の見かけの縮小で、
  P1〜P5 のような「意味の曖昧さ」「死んだ表層」の解消ではない。

### P7（統廃合ではないが先に出すべき）棚卸しで見つかった不具合

統廃合の前提を壊すので、別 issue として切り出す。

1. **[済]** **`@rightAssoc` の span 補正が両 backend で非対称**（§3.2-7）。**正しいのは Rust**
   （規則局所の trivia 注釈があるときだけ補正する）で、根拠は旧実装の 2 つの oracle:
   grammar 既定 trivia だけの `RightAssociativeMappingRuntimeTest`（`" 2 ^ 3 ^ 2 "` → 右子 [5,11]・
   葉 [9,11]、**補正なし**）と、規則局所 trivia の `rule-trivia/corpus.json:14` RightScoped
   （`" x /*r*/ ^ x "` → 右子 [11,12]、**補正あり**）。Java 側に同じ門を足し、入れ子判定も
   `depth > 1` に揃えた（旧 Java は「同じ規則の再帰呼出しか」だった）。ただし**実測では
   観測差を作れなかった**: 補正が効くのは `f.c` が最後の field 値の終端より先にある場合だけで、
   それは規則局所 trivia があるとき（＝Rust も補正するとき）に限られる。回帰テストは
   `crates/ubnfc-fuzz/tests/right_assoc_span.rs`（4 形 × 15 入力の Java/Rust 差分）。
2. **[誤]** **`languageVersion` の上限検査が Java backend に無い** — **これは棚卸しの誤り**。
   `ubnfc-java/src/main/java/org/ubnfc/java/ir/Validation.java:43-46` に
   `MAX_LANGUAGE_VERSION = 2` と `unsupported("languageVersion:" + language)` がある（D-038 どおり）。
3. **[済（半分）]** **`@precedence` + `@rightAssoc` が `operator` を作らない** — **前提が誤り**。
   分かれ目は結合性ではなく**演算子 atom の種類**で、`@leftAssoc` + リテラル演算子も
   `operator=None` になる（`lower/annotations.rs` が演算子を規則参照に限っている。実測で確認）。
   本当の非対称は `@rightAssoc` + **規則参照**演算子が `AnalysisError::InvalidOperator` で
   **文法ごと落ちる**ことだった（同じ形の `@leftAssoc` は通る）。D-047 で lowering 側を揃えて
   落ちないようにした。`@rightAssoc` にも `operator` を作る完全対称化は `plan.operators` と
   Power.ubnf の IR が変わるので別作業（IR 不変の枠を出る）。
4. **[未]** **同一内容の TriviaPolicy が重複する**（§3.1-1）。`rawSettings` 差だけで policy が分裂し、
   backend が同じ `Delimiters` 定数を 2 つ出す。P3 を入れると自然に消えるが、
   `lower/trivia.rs:126` の重複排除を `effectiveDelimiterSet` 基準にするだけでも直る
   （ただし policy ID の綴りが変わると self-host の byte 一致に出るので v2 に閉じること）。
   **今回は見送った**: 重複排除キーを変えると policy ID の採番が動き IR が変わるため、
   「IR を変えない」という今回の束の前提に収まらない。

### 却下した案

- **`@doc` の削除**: 生成コードには届かないが、`annotations[]` に載っていて LSP/カタログ側が
  読める意図的な tooling 表層。コストがほぼゼロなので残す。ただし「生成に効かない」と明記する。
  （2026-09-24: 実際に `ubnfc-lsp` の hover が読むようになった。D-062）
- **`@literalBoundary` の削除**: 使用 0 だが、v1 文法が v2 の語境界を個別に切るための唯一の逃げ道
  （D-040）。門を消してはいけない。
- **`@scopeTree(mode=…)` から `dynamic` を消す**: 現状 lexical と同じ挙動なのは事実だが、
  corpus に 3 ファイルの oracle がある。消すと v1 が壊れる。**「未実装の予約値」と仕様に明記**するに留める。
- **`@memoSafeToken` の削除**: 使用 12 件（P4 の 6 token × 2 文法）で、Extern token の
  memo 除外を緩める唯一の手段（§3.7）。実利用者がいる。

## 5. 推奨着手順

| # | 案 | 規模 | v1 への影響 | corpus 再走 | 前提 |
|---|---|---|---|---|---|
| 1 | P1 未知注釈の警告 | 小 | なし（v2 門） | 不要 | — |
| 2 | P2 `@backref` 分割 | 小 | なし（IR 不変） | 不要 | P1（警告の受け皿） |
| 3 | P3 `@interleave` 非推奨 | 小 | なし | P4-v2 の IR 差分確認 | P1 |
| 4 | P5 `@predictiveChoice` 非推奨 | 小 | なし | 不要 | P1 |
| 5 | P4 `@precedence` 省略可能 | 中 | `analysisVersion` を上げる | **要**（両 backend） | D-039 の候補順採番と突き合わせ |
| 6 | P6 型注釈系の整理 | 中 | なし（IR 不変） | 不要 | P2（制約族の置き場所） |
| 0 | P7 不具合 4 件 | 小〜中 | 1・3 は v1 の観測が変わり得る | **要**（1 と 3） | — |

P7 の 1 と 3 は v1 の挙動（span・driver 出力）に触るので、統廃合とは別 PR にし、
先に corpus と差分 fuzzing で現状を固定してから直す。2 と 4 は v2 に閉じられる。

1〜4 は**警告を足すだけ**で IR も backend も動かないので、まとめて 1 本の PR にできる。
5 だけが front の解析に手を入れるので独立させ、`cargo test --workspace` / clippy / fmt /
両 Maven モジュール / self-host 固定点 / `cargo test -p ubnfc-fuzz` を通し、
Java Pass 1310 / Rust Pass 1292 を維持することを完了条件にする（D-033 の jar 再生成を忘れない）。

fuzzer の generator は 1〜6 のどれでも触ることになる（§2.3 のとおり `@memoSafeToken` と
`@literalBoundary` 以外は全部生成している）。`@ubnf: 2` の分岐で新しい綴りを生成し、
v1 の分岐は今の綴りのままにすれば、差分 fuzzing が新旧両方を同時に守る。

## 6. 参照

- `06-ubnf-v2.md` §6（この文書の親）
- `03-design-v1.md` §3.4 選択 / §3.5 trivia / §3.6 capture / §3.7 memo / §3.8 scope / §3.10 recovery / §3.11 演算子 / §8 決定表
- `04-cursor-and-diagnostics-contract.md` §2 診断 / §4 左再帰
- `docs/compat-matrix.md`（注釈別の oracle 有無と C10）
- `unlaxer-parser/docs/rule-trivia.md`（`@whitespace` > `@interleave` > grammar）
- `unlaxer-parser/unlaxer-dsl/specs/annotations.md` / `ubnf-syntax.md` / `scope-tree.md` / `generators.md`
