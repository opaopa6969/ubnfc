# P4-v2 — tinyexpression P4 を UBNF v2 の左再帰ルールで書き直したもの

`tinyexpression-p4-v2.ubnf` は、tinyexpression の P4 文法
（`tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf`、**読み取り専用**）を
UBNF v2（`@ubnf: 2`）で書き直したもの。変えたのは次の 3 つだけである。

1. **式の塔 → 左再帰ルール**（D-039、下の表）
2. **token 宣言 → 言語中立な語彙と明示 `extern`**（D-041、`docs/reports/2026-09-23-ubnf-v2-token-vocabulary.md`）
3. **繰り返す形 → パラメータ化ルール**（D-043、`docs/reports/2026-09-23-ubnf-v2-parameterized-rules.md`）

それ以外は 1 文字も変えていない。

| | v1（手書き） | v2 |
|---|---:|---:|
| 規則数 | 124 | **119** |
| 式の塔にあたる規則 | 8 | **3** |
| 行数 | 658 | **607** |
| 雛形宣言 / 実体化 | 0 / 0 | **11 / 32** |

書き換えたのは 3 つの塔だけ:

| 塔 | v1 の規則 | v2 |
|---|---|---|
| 数値 | `NumberExpression` / `NumberTerm` / `NumberFactor` | `NumberExpression` 1 本 |
| 文字列 | `StringExpression` / `StringTerm` | `StringExpression` 1 本 |
| 真偽 | `BooleanExpression` / `BooleanAndExpression` / `BooleanXorExpression` | `BooleanExpression` 1 本（`BooleanFactor` は残す） |

真偽の塔は段ごとに AST クラスが違う（`BooleanOrExpr` / `BooleanAndExpr` / `BooleanXorExpr`）ので、
**候補先頭の `@mapping`** で段ごとに指定している。

## パラメータ化ルール（D-043）

雛形は**規則を作らない**。実体化した位置へその場で展開されるので、規則数は 119 のまま、
AST クラス・capture・`rulePath`・expected ラベルも 1 つも変わらない。減るのは
「同じ形を何度も書くこと」だけである。

| 雛形 | 実体化 | 元の重複 |
|---|---:|---|
| `VarDecl<Hint, Value>` | 4 | `{Number,String,Boolean,Object}VariableDeclaration`（2 要素しか違わない 9 行 × 4） |
| `MethodDecl<Ret, Body>` | 4 | `{…}MethodDeclaration`（11 行 × 4） |
| `ExternalInvocation<Ret>` | 3 | `External{Boolean,String,Object}Invocation`（Number 版は形が違うので残した） |
| `Slice<Recv>` | 2 | 8 候補のスライス構文が丸ごと 2 回（受け手だけが違う） |
| `MatchBody<Case, Default>` | 3 | `{Number,String,Boolean}MatchExpression` |
| `CaseRule<Value>` / `DefaultCaseRule<Value>` | 3 + 3 | `{…}Case` / `{…}DefaultCase` |
| `ValuesList<X>` | 2 | `MethodParameters` / `Arguments` |
| `NAryNumberFn<Kw>` | 2 | `min` / `max`（リテラルを実引数に渡す） |
| `StringPredFn<Kw>` / `StringPredDot<Kw>` | 3 + 3 | `startsWith` / `endsWith` / `contains` の関数形とドット形 |

カンマ区切りリストの穴埋めとしての価値はほとんど無かった（P4 の 15 箇所で capture 名が
ばらばらなので `ValuesList<X>` で括れたのは 2 箇所）。効いたのは型ごとの多重化のほうである。
理由と実測は報告の §4。

token 宣言は host 言語のクラス名を書かない形になった:

| token | v1 | v2 |
|---|---|---|
| `NUMBER` / `IDENTIFIER` / `EOF` | `org.unlaxer.parser.…Parser` | `NUMBER` / `IDENTIFIER` / `END_OF_INPUT` |
| `STRING` / `CODE_START` / `CODE_END` | `org.unlaxer.tinyexpression.parser.…` | `extern 'StringLiteralParser'` 等（scanner registry の ID） |
| `CODE_BODY` | `UNTIL('```')` | 同じ（元から中立） |

この差で IR が変わるのは `memoSafety.originalDeclaration` / `memoSafety.reason` /
extern の `originalClassName`・`javaBinding` の 4 キー 20 箇所だけで、`spec` も解析も計画も一致する。

## 比較の走らせ方

```sh
scripts/compare.sh            # 応答（accepted / cursor / AST / 診断）の突き合わせ
scripts/compare.sh --timing   # 上に加えて解析時間
```

両方の Grammar IR を作り、Rust backend で parser を生成して release build し、同じ入力を
両方の driver に投げて応答を比べる。入力は `examples/p4-rust/tests/fixtures/*.tiny`
（tinyexpression の `benchmarks/fixtures/*.tiny` と byte 一致）と、診断契約を突く 12 個の拒否入力。

唯一の正規化は**段の名前の読み替え**（`NumberExpression__lr1` → `NumberTerm` など 1 対 1）で、
AST は読み替えなしで比較する。結果は `docs/reports/2026-09-23-ubnf-v2-left-recursion.md`。

生成物は commit しない（`mktemp -d` の一時ディレクトリに作る）。反復するときは
`P4V2_WORK=<dir> P4V2_REUSE=1` で build を再利用できる。
