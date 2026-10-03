# Declarative lexical conformance

`unlaxer-parser` commit `21b6156` の `spec-corpus/declarative-tokens` と
`spec-corpus/lexical-modules` を共通の入力・期待値として固定した snapshot。
現行 Number/Identifier/quoted/fence 実装との oracle 比較は元 repository のテストが担い、
この repository では同じ定義から生成する standalone Java/Rust の挙動を比較する。

共有 JSON の `value` は旧 unlaxer-dsl mapper の trim/single-unquote 後の値であり、
字句認識の契約ではない。ubnfc の宣言的 token は capture へ raw text を渡すため、
両生成 backend の AST 値は ubnfc mapper の従来の text-field trim のみを適用した値と比較する。
引用符は残る。内部の CAPTURE/SAME_AS、token span は raw のまま、末尾改行も認識範囲に含む。
言語アプリケーションの値変換は token 認識とは別の mapping 層で行う。

ubnfc の既存 general rule import は引き続き利用可能。unlaxer-parser の初期 token-only
loader 制約（`rule-module` / `host-parser-module`）は ubnfc の一般 import を廃止する条件ではない。
