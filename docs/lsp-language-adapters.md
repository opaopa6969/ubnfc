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

## Document formatting

言語側で生成 LSP パッケージの `DocumentFormatter` を実装する。公開の引数なし
コンストラクターを持つクラスをサーバーのクラスパスに置き、
`java -jar server.jar --formatter 完全修飾クラス名` で指定する。組み込み時は
`LanguageServerImpl(scanners, validator, signatures, formatter)` に渡せる。
設定すると `documentFormattingProvider` を通知する。

`format(Analysis document)` はコメント、文字列、意味を保持した整形後の全文を返す。
適用できない場合は `null`、整形済みの場合は元の文字列を返す。
書式規則は言語側で固定する。生成サーバーは入力の全文解析が回復なしで成功した
ときにだけ呼び出し、出力の全文解析と位置を除いた構文・トークン列の一致を確認する。
空白によって意味が変わる文法では、整形規則がその意味を保持する必要がある。
変更が必要なら文書全体を置換する UTF-16 範囲の編集を 1 件返す。

[Stage5 用のサンプル](../ubnfc-lsp/src/test/resources/stage5/DemoDocumentFormatter.java)では
`let  total=5;` を `let total = 5;` に整形する。文のトークンは半角空白 1 個で
区切り、セミコロンの直前に空白を入れない。文末や波括弧の位置で行を分け、
ブロック 1 段につき半角空白 2 個で字下げする。各コメントをその深さの独立した行に置き、
文字列とコメントは字句の内容と出現順を保持する。字句の外の改行には LF を使う。

## 構文診断の quick fix

言語側で生成 LSP パッケージの `SyntaxQuickFix` を実装する。公開の引数なし
コンストラクターを持つクラスをサーバーのクラスパスに置き、
`java -jar server.jar --syntax-quick-fix 完全修飾クラス名` で指定する。組み込み時は
`LanguageServerImpl(scanners, validator, signatures, formatter, quickFix)` に渡せる。
設定すると `codeActionProvider` に `quickfix` が含まれる。

`forDiagnostic(Analysis document, Diagnostic diagnostic)` は、生成パーサーが出した
主構文診断の種類 `kind`、最深規則 `deepestRule`、期待ラベル `expected`、code point
座標などを使って判定する。挿入が一意に決まる場合は編集の表示名と挿入文字列を
`Insertion` で返し、候補がない場合や複数ある場合は `null` を返す。

クライアントは通知された診断の `data.quickFixDiagnosticId` を code action の
`context.diagnostics` に引き継ぐ。生成サーバーは開いている文書の最新スナップショットで
診断 ID・`source`・`code`・`range` を照合し、要求範囲が診断の開始位置を含むことを
確認する。ID はサーバーのセッション内で文書の更新と再オープンごとに変わる。
候補は現在の本文の診断開始位置に仮挿入し、全文を再解析する。回復なしで構文解析に
成功した候補だけを、同じ文書 URI とバージョンを持つ `documentChanges` の編集として
返す。クライアントにはバージョン付き文書編集のサポートが必要。

[Stage5 用のサンプル](../ubnfc-lsp/src/test/resources/stage5/DemoSyntaxQuickFix.java)は
`let n = 1` のように `Let` の構文診断が末尾を指し、期待ラベルに `';'` がある場合に
セミコロンを提案する。`let n =` のように文の内容が決まらない場合や、閉じ波括弧も不足し
セミコロンの挿入だけでは全文が解析できない場合は編集が空になる。
