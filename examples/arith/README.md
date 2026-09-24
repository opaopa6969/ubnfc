# 算術式（Operators / Power）を ANTLR / tree-sitter で書いたもの

既存の速度比較（[`docs/reports/2026-09-23-parser-speed-comparison.md`](../../docs/reports/2026-09-23-parser-speed-comparison.md)
の family B）が使っている 2 つの文法を、外部ツールとの比較のために `.g4` と `grammar.js` へ
写したもの（issue #26）。UBNF 側の原本は**読み取り専用の兄弟 repo**にある:

- `unlaxer-parser/unlaxer-dsl/src/test/resources/associative/Operators.ubnf`
- `unlaxer-parser/unlaxer-dsl/src/test/resources/right-associative/Power.ubnf`

（`examples/handwritten-target/{java,rust}/README.md` が「生成物の目標形」として同じ 2 文法を扱う。）

```
grammar Operators {                          grammar Power {
  @whitespace: javaStyle                       @whitespace: javaStyle
  token NUMBER = NumberParser                  token NUMBER = NumberParser
  Expression ::= Term { AddOp Term };          Expr ::= Atom { '^' Expr };
  Term ::= Factor { MulOp Factor };            Atom ::= NUMBER;
  AddOp ::= '+' | '-';                       }
  MulOp ::= '*' | '/';
  Factor ::= NumericLiteral | '(' Expression ')';
  NumericLiteral ::= (NUMBER) @value;
}
```

計測入力は既存の比較と同じ **512 atom**（`1+1+…+1`、1,023 byte）と
**power-64**（`2^2^…^2`、127 byte）。

## 3 記法で揃えられなかった点

### 1. NUMBER の先頭符号 — ANTLR 版だけ落としてある

`NumberParser` は `[+-]? ( D+ '.' D+ | D+ '.' | D+ | '.' D+ ) ( [eE] [+-]? D+ )?` を受理する
（`docs/design/02-facts-tokens-bench-corpus.md` §A）。UBNF はスキャナレスで、NUMBER は
「そこで NUMBER が期待される位置」でしか試されないので、`1+2` が
`NUMBER(1)` `+` `NUMBER(2)` になる。

- **tree-sitter は同じ形を保てる。** tree-sitter の字句は**その構文状態で有効な語彙だけ**を
  試すので、`1` の直後の状態では `number` が候補に入らず、`+` が演算子として読まれる。
- **ANTLR は保てない。** ANTLR の lexer は parser の状態を見ない独立の最長一致なので、
  先頭符号を `NUMBER` に入れると `1+2` が `NUMBER(1)` `NUMBER(+2)` になり言語が壊れる。
  意味述語や lexer mode を使わずに直す手が無いので、**`.g4` では先頭符号を落とした**。
  結果として ANTLR 版は `-5` を受理しない（UBNF / tree-sitter は受理する）。
  計測入力には符号が出てこないので、速度の比較には影響しない。

### 2. `Expr ::= Atom { '^' Expr }` の繰り返し

UBNF の `{ }` は 0 回以上の繰り返しだが、内側の `Expr` が右再帰で残りを食い切るため、
生成する言語は `Atom ('^' Expr)?` と同一である。`.g4` / `grammar.js` で `*` / `repeat` の
まま書くと**本当に曖昧**になり、ANTLR には ALL(*) の full-context 予測を、tree-sitter には
GLR の分岐を強いる。これは「外部ツールを不利にする書き方」なので、言語が同じ
`?` / `optional(...)` を使った。

### 3. trivia

UBNF の `@whitespace: javaStyle` の区切り集合は code point 9,10,11,12,13,32 と
`//` 行コメント、`/* */` ブロックコメント（`crates/ubnfc-front/src/lower/trivia.rs:85-99`）。
`.g4` と `grammar.js` はこの 3 つをそのまま写してある。

## 生成と計測

`benchmarks/scripts/prepare-external.sh` が 3 系統をまとめて建てる。
結果は [`docs/reports/2026-09-24-antlr-treesitter-comparison.md`](../../docs/reports/2026-09-24-antlr-treesitter-comparison.md)。
