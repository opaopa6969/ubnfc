# LSP の表示・編集機能（issue #25）

LSP / DAP は Java 側だけの opt-in feature である。Rust backend に LSP / DAP は存在しない。

| capability | 具体例 | 対象外 | 対応 |
|---|---|---|---|
| Inlay hints | `let alpha = 2;` で `alpha` の直前に `name:`、`2` の直前に `value:` を表示 | 型・引数の推論、規則や複合式に付いた capture、skip capture、空の終端、テキスト編集 | 実装 |
| Signature help | `sum(1, \|)` で `sum(left, right)` を表示し、第 2 引数を強調 | 型・引数名の推論、文書をまたぐオーバーロード解決 | [#60](https://github.com/opaopa6969/ubnfc/issues/60) |
| Document formatting | `let  total=5;` を `let total = 5;` に整形 | 範囲整形、入力時整形、構文エラーの修復 | [#61](https://github.com/opaopa6969/ubnfc/issues/61) |
| Code actions | 診断に対応して `let n = 1` の末尾へ `;` を挿入する quick fix | 曖昧な修正、リファクタリング、一括・文書間編集 | [#62](https://github.com/opaopa6969/ubnfc/issues/62) |

直接 token を読む capture は IR と既存の lexical trace の式 ID から特定できる。生成時に
対象式とラベルを表にし、表を持つ LSP サーバだけが `inlayHintProvider: true` を通知する。
ラベルは同じ式に複数あれば文法順にまとめる。サーバは要求範囲の開始を含み、終了を含まない
位置で対象 token を絞り、`Positions` で UTF-16 座標に変換する。未読込・close 済み・構文不正・
回復を含む文書には空の応答を返す。診断時の解析に負荷を加えず、要求時に既存の lexical trace を利用する。

残りの 3 機能は、呼び出しの意味、書式、診断と編集の対応を言語側が決める必要があるため、
それぞれ実装の受け入れ条件を持つ issue に分割した。生成サーバはこれらの capability を通知しない。

## 検証

`Stage5GoldenTest` は生成サーバをコンパイルし、JSON-RPC で capability、正常入力、不正・不完全入力、
close 後の応答と、同じ行の BMP 外文字を経た UTF-16 座標と範囲境界を確認する。
`ParserUnchangedTest` は LSP / DAP の生成前後でパーサ生成物を byte 比較し、
`--emit-lsp` を指定しないと LSP ファイルが出ないことを確認する。
VS Code でのラベル表示と設定の切り替えは [#28](https://github.com/opaopa6969/ubnfc/issues/28) に記録する。
