# P4-v2 — tinyexpression P4 を UBNF v2 の左再帰ルールで書き直したもの

`tinyexpression-p4-v2.ubnf` は、tinyexpression の P4 文法
（`tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf`、**読み取り専用**）を
UBNF v2（`@ubnf: 2`）で書き直したもの。変えたのは次の 2 つだけである。

1. **式の塔 → 左再帰ルール**（D-039、下の表）
2. **token 宣言 → 言語中立な語彙と明示 `extern`**（D-041、`docs/reports/2026-09-23-ubnf-v2-token-vocabulary.md`）

それ以外は 1 文字も変えていない。

| | v1（手書きの塔） | v2（左再帰ルール） |
|---|---:|---:|
| 規則数 | 124 | **119** |
| 式の塔にあたる規則 | 8 | **3** |

書き換えたのは 3 つの塔だけ:

| 塔 | v1 の規則 | v2 |
|---|---|---|
| 数値 | `NumberExpression` / `NumberTerm` / `NumberFactor` | `NumberExpression` 1 本 |
| 文字列 | `StringExpression` / `StringTerm` | `StringExpression` 1 本 |
| 真偽 | `BooleanExpression` / `BooleanAndExpression` / `BooleanXorExpression` | `BooleanExpression` 1 本（`BooleanFactor` は残す） |

真偽の塔は段ごとに AST クラスが違う（`BooleanOrExpr` / `BooleanAndExpr` / `BooleanXorExpr`）ので、
**候補先頭の `@mapping`** で段ごとに指定している。

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
