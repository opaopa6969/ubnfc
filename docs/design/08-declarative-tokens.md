# 宣言的 token と lexical module

`@ubnf: 2`（同義の `@ubnf: v2` も可）で、`token NAME ::= expression ;` を使う。
Java/Rust とも式から独立した認識コードを生成する。旧 `NumberParser` 等への
クラス名 mapping や scanner registry はこの宣言には関与しない。

```ubnf
grammar Numbers {
  @ubnf: v2
  token DIGIT ::= CHAR_RANGE('0', '9');
  token NUMBER ::= ('+' | '-')?
      (DIGIT+ ('.' DIGIT*)? | '.' DIGIT+)
      (('e' | 'E') ('+' | '-')? DIGIT+)?;
}
```

```ubnf
grammar Application {
  @import numeric from 'lexical/numbers.ubnf'
  @ubnf: v2
  @root Value ::= numeric.NUMBER;
}
```

import alias が namespace になる。独立した `namespace` 宣言は設けない。
パスは宣言元ファイル基準、参照先は宣言元 module の名前環境で閉じる。
同じ module を別 alias で参照でき、呼び出し側の同名 token には影響されない。
lexical module を含む import 先が複数 grammar を持つ場合は曖昧として拒否する。
ubnfc の既存の一般 rule import は維持する。一方、unlaxer-dsl の最初の lexical
module loader は token 専用であり、両者の module 範囲は同一ではない。

## 式と認識契約

| 語彙 | 意味 |
|---|---|
| `'text'`, `CHAR_RANGE('a','z')`, `NEGATION('chars')`, `ANY` | 固定文字列、Unicode scalar 範囲、除外文字集合、任意の 1 scalar |
| 連接、`a | b`、`(expression)` | 連接、左優先選択、グループ |
| `[expression]`, `{expression}`, `?`, `*`, `+`, `{n}`, `{n,m}`, `{n,}` | 任意・反復。反復は greedy / possessive |
| `LOOKAHEAD(expression)`, `NEGATIVE_LOOKAHEAD(expression)` | 純粋な先読み。入力と capture を変更しない |
| `CAPTURE(name, expression)`, `SAME_AS(name)` | 当該 token 内の raw 部分文字列の保存・完全再照合 |
| `BOF`, `EOF`, `BOL`, `EOL` | 入力端・行端のゼロ幅 assertion。行端は CR/LF 各位置を判定 |
| `NAME`, `alias.NAME` | 別の宣言的 token。参照先ごとに capture scope を分離 |

token 式内には暗黙の空白・コメント・word boundary を挿入しない。rule 側の trivia
設定は token 間に適用される。引用符を含めて token が認識した範囲は raw であり、
STRING の unquote / escape decode / 数値変換はアプリケーションの値変換の責務。
ubnfc の既存の text mapper は周辺 ASCII 空白を trim するため、AST の text 値と
raw capture が同じとは限らない。これは認識コードとは別の既存契約である。

失敗は consumed / matched 両位置と capture を保持し、成功は token 全体を原子的に
確定する。ゼロ幅成功・反転 assertion は二重カーソルを動かさない。選択失敗や
反復試行失敗は保存した capture を復元する。未定義参照・循環参照・束縛が確定
していない再照合・nullable な無限反復を生成前に拒否する。展開は深さ 128、
node 数 4096 に制限する。

IR は `TokenSpec.kind = "declarative"` と閉じた `LexicalExpression` を持ち、
`requiredFeatures` に `declarativeTokens` を宣言する。`REF` は source AST 専用で、
IR では参照を `SCOPE` へ展開済み。Java/Rust の両 generator が IR を検証する。
旧 `token NAME = ...` と `extern` は互換用に残るが新定義には不要である。
unlaxer-dsl の versioned `ADAPTER(...)` registry や `@feature` capability 契約は
この実装の対象外であり、同義の `extern` として暗黙変換しない。

## 検証と自己ホスト

`fixtures/spec-corpus` は unlaxer-parser の同じ入力・期待位置を取り込んだもの。
58 ケースを生成 Java 17 / Rust で実行し、成功・失敗・消費位置・全入力一致と
AST 値・位置を比較する。可変長 fence、CRLF、非 BMP、局所 capture、import の
閉じた名前解決を含む。不正文法と module graph は別テストで拒否を確認する。

`ubnf.ubnf` は 6 宣言的 token / 66 rule のまま IR と Rust parser を生成する。
生成 frontend は採用 occurrence 木から lexical AST を構築する。文字列・反復数の
scalar decode に lexer を使うが、手書き parser への再解析 fallback はしない。
手書きとの AST/span 一致、生成 frontend 由来 IR の byte 固定点、および生成物全
ファイルの byte 固定点をテストする。v2 の word boundary により既知の `@rooted`
差分は解消し、従来の 25 fixture 中 21 件が AST/span まで一致する。

再生成は `UNLAXER_PARSER_DIR=/path/to/unlaxer-parser cargo run -p ubnfc-selfhost
--example regenerate`（1 行）で実行できる。続けて `cargo fmt --all` と
`cargo test -p ubnfc-selfhost` で固定点を確認する。
