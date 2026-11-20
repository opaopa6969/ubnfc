# 生成 Java LSP の言語アダプター

## Signature help

言語側で、生成 LSP パッケージの `SignatureHelpProvider` を実装する。
公開の引数なしコンストラクターを持つクラスをサーバーのクラスパスに置き、
`java -jar server.jar --signature-help 完全修飾クラス名` で指定する。
サーバーを組み込む場合は `LanguageServerImpl(scanners, validator, provider)` に直接渡せる。
アダプターを指定したサーバーが `signatureHelpProvider` を通知する。
`triggerCharacters()` を実装すると `(` や `,` などの自動表示の起点を指定できる。

`at(Analysis document, int cursorCp)` には編集中の全文スナップショットと
カーソルの code point オフセットが渡る。全文解析に失敗した文書でも呼び出される。
実装はカーソルを含む最内側の呼び出しを選び、次の値を `Call` に入れて返す。

- カーソルを含められる範囲の開始・終了 code point オフセット。両端を含む。
  例えば開始は開き括弧の直後、終了は閉じ括弧の直前とし、末尾で未完なら文書末尾を含める。
- 言語側で定義したシグネチャのラベルと、宣言順の引数ラベル。
- 0 起点の現在の引数番号。引数のない呼び出しでは `-1`。

カーソルが呼び出しの外にある場合や、最内側の呼び出しのシグネチャが未知の場合は
`null` を返す。LSP の UTF-16 座標からの変換は生成サーバーが行う。

[テストで動くアダプター例](../ubnfc-lsp/src/test/resources/stage5/DemoSignatureHelp.java)は、
`sum(1, |` で `sum(left, right)` の第 2 引数を選ぶ。
`sum(1, inc(|2))` では、内側の `inc(value)` の第 1 引数を選ぶ。
