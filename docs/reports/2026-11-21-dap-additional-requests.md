# DAP の停止位置・watch と逆実行の境界（issue #27）

LSP / DAP は Java 側だけの opt-in feature である。Rust backend に LSP / DAP は存在しない。

| 要求 | 動作例と責任 | 対応 |
|---|---|---|
| `breakpointLocations` | `/* 😀 */ let a = 2; let b = 3;` で、adapter が AST の開始位置を UTF-16 の行・列で返す。bridge は介在しない。 | 実装 |
| watch | 停止中の `total` が `5` を返す。bridge が選択フレームの節点で副作用なしに評価し、未対応の式には空を返す。 | 実装 |
| 条件付き breakpoint | `total > 0` が真の候補位置でだけ停止する。bridge が候補位置の状態から副作用なしに条件を評価する。 | 実装（[#76](https://github.com/opaopa6969/ubnfc/issues/76)） |
| hit condition | 同じ breakpoint の 3 回目の到達時に停止する。adapter が回数を数え、bridge はカウントを変更しない。 | 実装（[#77](https://github.com/opaopa6969/ubnfc/issues/77)） |
| logpoint | `total={total}` を出力して停止せずに進む。adapter が出力し、bridge が埋め込み式の値だけを観測する。 | [#78](https://github.com/opaopa6969/ubnfc/issues/78) |
| `setExpression` | 停止中の `counter` に `7` を代入し、watch に反映する。bridge が代入先と型を検証してから更新する。 | [#79](https://github.com/opaopa6969/ubnfc/issues/79) |
| `stepBack` | B の合計が 5、直前の停止 A の合計が 2 なら、A と 2 を一緒に復元する。adapter が停止履歴を管理し、bridge が状態を復元する。 | [#80](https://github.com/opaopa6969/ubnfc/issues/80) |

## 停止位置と watch

位置問い合わせは、正常に解析した現在の入力ファイルに対して `setBreakpoints` が受理できる AST
節点の開始位置を返す。両端を含む検索範囲内で重複を除き、行・列順に整列する。行と列の起点は
initialize のクライアント設定に合わせ、列の UTF-16 変換は既存の `Positions` を使う。
ファイルパス方式の単一ソースを対象とし、別ファイル、解析前・失敗後・終了後、候補がない範囲では
空を返す。列を省いた登録は行単位で、列付きの登録は該当位置で停止する。ログ付きの登録には未検証と理由を返す。別ソースの登録要求は現在のブレークポイントを変更しない。

watch は `evaluate(context="watch")` を使い、独立した initialize capability はない。
停止中の現在または選択中のスタックフレームだけを受け付け、選んだ節点を bridge に渡す。
同じ停止中に繰り返し評価されるため、bridge は式の評価で評価器内外の状態を変えない。
例の `SumEvaluatorBridge` は launch 成功時に初期値を作り、`total` の読み取りだけを行う。
未知の式、停止前、スタック外のフレーム、評価器のない NOOP は失敗応答を返す。
hover の自動評価は通知しない。再 launch は AST、停止位置、ブレークポイント、変数参照を初期化し、
成功時に bridge の `onLaunch` を呼ぶ。

## 条件付き breakpoint

`EvaluatorBridge.supportsConditionalBreakpoints()` が true の場合だけ DAP の capability を有効にする。
その他の bridge で条件付き登録を受けたときは未検証と理由を返す。式の意味は bridge が持つ。

`continue` と `stopOnEntry=false` の初回実行では、AST の前順で開始位置が合う候補を探す。
候補に移動する前に、その節点を `evaluateCondition` に渡す。bridge は候補の節点と現在の状態を
読み取って判断し、候補を試すためにステップ通知や代入を実行しない。同じ位置に複数の登録があれば
要求順に評価し、最初に成立した登録で `stopped(reason="breakpoint")` を返す。
条件が偽ならそのまま次の候補へ進む。未知の式（空の結果）と評価失敗（実行時例外）は
`stderr` に通知し、どちらも停止せず続行する。再 launch では登録を初期化する。

## Hit condition

`supportsHitConditionalBreakpoints` は既定の NOOP を含め有効にする。回数は ASCII の十進数字で
`1` から `2147483647` までを受け付け、先頭のゼロは許容する。空文字、ゼロ、空白、符号、比較演算子、
範囲外の値を指定した登録は未検証と理由を返す。

`continue` と `stopOnEntry=false` の初回実行で、登録位置に合う AST 節点を調べるたびに、登録ごとの
回数を adapter が 1 増やす。開始位置が同じ親と子も別々の候補として数える。複数の登録に合う候補では、
どの登録で停止する場合も各登録を数える。指定の回数に達した候補で通常の条件式を評価し、
成立時に `stopped(reason="breakpoint")` を返す。指定回数を過ぎた候補は通過する。
現在のソースへの `setBreakpoints` は登録を置き換えて回数を初期化し、再 launch は全登録を初期化する。

## 逆実行を有効にする条件

逆実行を実装する bridge は、各停止点の観測時に、値・スコープ・制御状態を再現できる snapshot を
保存する。復元に失敗したら、復元前の状態とステップ位置を保つ。外部 I/O など復元できない
副作用を持つ bridge では capability を有効にしない。履歴は launch ごとに作る。
DAP の同じ capability が対象にする `reverseContinue` の契約も #80 で確定する。
現行の NOOP と手書き bridge は `supportsStepBack=false` を通知し、直接の `stepBack` には
失敗応答を返して停止フレームと評価値を保つ。

## 検証

`Stage5GoldenTest` は生成した Java DAP をコンパイルし、実際の JSON-RPC で capability と
座標、列指定の停止、watch の値・エラー、条件の真偽と失敗、失敗した launch と再 launch、逆実行の拒否を照合する。
`ParserUnchangedTest.dapEmissionOnlyAddsFiles` は feature off と on の parser / LSP 全ファイルを
byte 比較する。VS Code 実機での列ブレークポイントと Watch パネル表示は
[#28](https://github.com/opaopa6969/ubnfc/issues/28) に記録する。

プロトコルの座標・capability の出典は [DAP 仕様の schema](https://github.com/microsoft/debug-adapter-protocol/blob/main/debugAdapterProtocol.json)。
