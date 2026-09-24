# P4 の tree-sitter 移植

`examples/p4-v2/tinyexpression-p4-v2.ubnf`（UBNF v2、119 規則）を tree-sitter へ
**構文だけ**移植したもの。scope 解決（`@scopeTree` / `@declares` / `@backref` / `@catalog`）と
AST 射影（`@mapping`）には対応物が無いので再現していない。

**規則対応表・優先順位・逸脱の一覧は [`../p4-antlr/README.md`](../p4-antlr/README.md) に
ANTLR 版とまとめて 1 つだけ置いてある。** ここには tree-sitter 固有の事情だけを書く。

| ファイル | 中身 |
|---|---|
| `grammar.js` | 文法本体（119 規則 1:1、塔の統合なし） |
| `grammar.idiomatic.js` | 塔を 1 本の `expression` に統合した版（issue #39）。`../p4-antlr/README.md` の「ANTLR / tree-sitter 流に書き直した版」を参照 |
| `tree-sitter.json` | CLI 用の metadata（これが無いと ABI 14 で生成される） |
| `driver.c` | 受理・速度計測の driver（両方の文法で共用） |

生成物（`src/parser.c` など）は commit しない。

## 入口 2 本

tree-sitter の開始規則は 1 本だけなので、`grammar.js` は環境変数で選ぶ。

```sh
P4_TS_ENTRY=Formula           tree-sitter generate   # 既定
P4_TS_ENTRY=BooleanExpression tree-sitter generate
```

`source_file: choice($.formula, $.boolean_expression)` に**合併しなかった**理由:
`formula` はほぼ無条件に `boolean_expression` を導出できるので、合併すると開始記号自体が
全面的に曖昧になり、conflict と状態数が無意味に増える。さらに「どちらの入口で受理されたか」が
観測できなくなり、ツールは和集合を受理するだけになる — 今回まさに測りたい
「Formula では拒否 / BooleanExpression では受理」という ubnfc の差が消える。

## 生成の実測（tree-sitter 0.25.10）

| | Formula 入口 | BooleanExpression 入口 |
|---|---:|---:|
| `tree-sitter generate` | 0.89 s | 0.76 s |
| `src/parser.c` | 1,018,051 byte | 748,372 byte |
| `STATE_COUNT` | 882 | 654 |
| `LARGE_STATE_COUNT` | 103 | 84 |
| `SYMBOL_COUNT` | 238 | 194 |
| `TOKEN_COUNT` | 106 | 94 |
| `EXTERNAL_TOKEN_COUNT` | 0（外部 C scanner を使っていない） | 0 |
| `LANGUAGE_VERSION` | 15（`tree-sitter.json` があるので ABI 15。無いと 14 になる） | 15 |

## `conflicts`（23 件）

`tree-sitter generate` が報告したものだけを、報告された形のまま書いた。
最終状態で「unnecessary conflict」警告は出ない＝23 件とも必要。
すべて P4 の**型別の塔**（`number_expression` / `string_expression` /
`boolean_expression` / `object_expression`）が重なる箇所である。

```
[$.number_expression, $.string_expression, $.boolean_comparable, $.object_expression, $.expression]
[$.number_expression, $.string_expression]
[$.number_expression, $.string_expression, $.boolean_comparable]
[$.object_expression, $.expression]
[$.number_expression, $.string_expression, $.boolean_comparable, $.object_expression]
[$.variable_ref]                                  # Formula 入口だけ
[$.number_expression, $.object_expression, $.expression]
[$.parenthesized_string_expression, $.object_expression, $.expression]
[$.boolean_comparable, $.object_expression, $.expression]
[$.number_expression, $.slice_base_receiver, $.string_expression, $.boolean_comparable, $.object_expression]
[$.argument_expression, $.boolean_factor]
[$.number_variable_declaration, $.string_variable_declaration, $.boolean_variable_declaration, $.object_variable_declaration]
[$.string_typed_variable, $.type_keyword]
[$.number_expression, $.slice_base_receiver, $.string_expression, $.boolean_comparable]
[$.number_expression, $.string_expression, $.boolean_comparable, $.object_expression, $.branch_expression]
[$.object_expression, $.branch_expression]
[$.boolean_factor, $.branch_expression]
[$.slice_base_receiver, $.string_expression]
[$.string_expression, $.boolean_comparable]
[$.slice_base_receiver, $.string_expression, $.boolean_comparable]
[$.number_variable_declaration, $.object_expression]
[$.string_variable_declaration, $.object_expression]
[$.boolean_variable_declaration, $.object_expression]
```

## `prec.dynamic`（27 箇所 / 7 規則）

GLR は複数の解析が同時に成功しうる。P4 の候補順（先頭が最高）を再現するために置いた。
`prec.left` は塔の 6 箇所、`prec.right` は **0 箇所**（P4 に `@rightAssoc` が無い）。

| 規則 | 値（候補順に降順） | 箇所 |
|---|---|---:|
| `expression` | 100/80/60/40/20/0 | 5 |
| `branch_expression` | 70/60/50/40/30/20/10/0 | 7 |
| `variable_declaration` | 30/20/10/0 | 3 |
| `object_expression` | 5/4/3/2/1/0 | 5 |
| `boolean_factor` | 3/2/1/0 | 3 |
| `argument_expression` | 3000/2000/1000/0 | 3 |
| `slice_base_receiver` | `'(' VariableRef ')'` 候補だけ 1 | 1 |

刻み幅を規則ごとに変えてあるのは、dynamic precedence が木全体で**加算**されるためで、
`expression → object_expression → number_expression` のような塔への再入が直接候補を
上回らないようにしている（例: 100 > 40+5）。

これを入れる前と後で実際に選ばれる木が変わったのは次の 3 つ。入れた後は P4 の候補順と一致する。

- `if($c){$a}else{$b}` の branch が `object_expression` → **`number_expression`**
- `($name)[0:6]` の受け手が `parenthesized_string_expression` → **`'(' VariableRef ')'`**
- `var $x set $y;` が **`number_variable_declaration`**

## 認識と木

tree-sitter に「認識のみ」は無い。`ts_parser_parse` は必ず CST を作る。
driver の `recognition(CST)` は parse + `ts_node_has_error` + 解放、`tree(CST+walk)` は
そこに全 node の深さ優先走査を足したもの。**どちらも CST 構築を含む**。

## 流儀で書き直した版（`grammar.idiomatic.js`、#39）

型別の塔を 1 本の `expression`（`prec.left` の梯子）に統合したもの。
入口は `P4_TS_ENTRY=Formula`（既定）/ `Expression`。
受理・拒否は 16 fixture・拒否入力 10 件とも `grammar.js` と完全に一致し、
P4 が型で拒否する 14 件を追加で受理する（上位集合。一覧は `../p4-antlr/README.md`）。

| | `grammar.js`（1:1） | `grammar.idiomatic.js`（流儀） |
|---|---:|---:|
| `conflicts` | 23 件 | **1 件**（`Formula` 入口のみ。`Expression` 入口は 0） |
| `prec.dynamic` | 27 箇所 / 7 規則 | **0 箇所** |
| `STATE_COUNT`（Formula / 第 2 入口） | 882 / 654 | **307 / 183** |
| `LARGE_STATE_COUNT` | 103 / 84 | 3 / 2 |
| `SYMBOL_COUNT` | 238 / 194 | 154 / 122 |
| `TOKEN_COUNT` | 106 / 94 | 106 / 94 |
| `src/parser.c` | 1,018,051 / 748,372 byte | **356,765 / 223,062 byte** |
| `LANGUAGE_VERSION` | 15 | 15 |
| `EXTERNAL_TOKEN_COUNT` | 0 | 0 |

残る 1 件の conflict は型と無関係で、`Formula` 入口で `$a string` が
「型キーワード付きの `variable_ref`」か「`variable_ref` の次に返り値型 `string` の
`method_declaration` が始まる」かが決まらないというもの。`grammar.js` も
`Formula` 入口でだけ同じ `[$.variable_ref]` を要求する。
