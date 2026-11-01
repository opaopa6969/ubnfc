# LSP / DAP 段階 5（issue #1）

## 1. 完了範囲と選択理由

段階 3 §10 / 段階 4 §8 の残りから、rename、folding range、EvaluatorBridge の実例を完了する。
既存の索引・scope 観測・拡張点を使える三つに範囲を絞り、生成パーサの意味論を変更しない
最小の可逆な変更を選んだ。判断は事前確認で止めず、本書・PR・issue コメントに残す。

**LSP / DAP は Java 側だけの opt-in feature である。Rust backend に LSP / DAP はない。**
Java / Rust のパーサ機能の対称性とは切り離しており、本変更で Rust の生成 API を増やさない。
front のソース、Java の AST 構築、`scripts/external-pins.env` は変更しない。

## 2. rename（D-086）

生成 `LanguageServerImpl` は `renameProvider=true` を通知し、`textDocument/rename` に
`WorkspaceEdit.changes` を返す。definition / references と同じ `Symbols.at` / `named` を使い、
宣言・参照の範囲を位置順で重複なく `TextEdit` にする。code point → UTF-16 は `Positions` に集約する。
scope 観測由来の索引と UBNF の `sameRuleText` の lexical fallback の両方で動く。

範囲は既存索引と同じく **1 文書内・綴り単位**。束縛同一性・shadowing・文書間検索や名前の文法検証は
追加しない。同綴りの独立宣言も対象になり得るため、クライアントで編集を確認して適用する。
未読込・close 済み・記号なし・空白だけの新名では空の編集を返す。
構文不正の場合には、前半だけの索引から一部を変更する事故を避けるため空の編集を返す。
新しい `prepareRename` capability は通知しない。

## 3. folding range（D-087）

生成 `FoldingRanges.java` と `foldingRangeProvider=true` を追加した。
`scope().events()` の enter/leave を stack で対応させ、複数行の範囲だけを返す。
閉じていない enter は推測で EOF まで伸ばさない。半開区間の終端が行頭なら直前の行末へ直す。

trivia は公開 lexical trace に直接現れないため、IR の trivia policy から順序付きの
コメント区切りを `Meta.COMMENT_DELIMITERS` へ生成し、成功した lexical 終端間の隙間だけを読む。
行コメントは読み飛ばし、閉じた複数行 block comment に `kind=comment` を付ける。
文字列などの終端内部を走査せず、未知の文字や閉じていないコメントでは隙間の走査を打ち切る。
区切りはパーサと同じく最初の close で閉じ、入れ子コメントは扱わない。
code point / UTF-16 の走査位置は前の終端から持ち越し、終端ごとに文書の先頭を数え直さない。
複数 policy の区切りは TextMate と同様に集合として生成するため、異なる policy が同じ隙間で競合する
文法について policy 単位の厳密な再現はしない。単一行・同一範囲の重複は除外する。
不正文書は `Analysis` が持つ解析成功の前半だけを使い、前半も無ければ空になる。

## 4. 評価器の実例（D-088）

`examples/stage5-debug/` に小さな文法・IR・`SumEvaluatorBridge`・使い方を置いた。
文法から parser / typed AST / LSP / DAP が生成され、手書き bridge が型付き AST をたどり
全 `let` の数値を `BigDecimal` で加算する。生成物を継承せず、`DebugAdapter(scanners, bridge)`
へ合成する。`EvaluatorBridge.load` は `DapMain --evaluator` と同じクラス名ロード経路である。

`variables` と `evaluate("total")` は `5`、別入力の再 launch 後は `7`。
未知の式は `Optional.empty()` を返し既存の明示エラーに委ねる。
既定 NOOP も同じ未対応エラーのままである。逐次実行・副作用のある評価器は今回実装しない。
`supportsEvaluateForHovers` / `supportsSetVariable` / `supportsStepBack` の通知は変更しない。

## 5. 固定したテストと opt-in 不変条件

`ubnfc-lsp` の `Stage5GoldenTest` が実際の生成物を JDK 21 でコンパイルし、
`Stage5Scenario` の両端に lsp4j launcher を置いて JSON-RPC を往復する。
`src/test/resources/stage5/` の四つの golden は次を固定する。

| golden | 固定する観測 |
|---|---|
| `rename-scope.json` | 宣言・参照から同一編集、無関係位置・空白新名・不正文書・close 済みの空編集 |
| `rename-lexical.json` | UBNF の fallback 索引から宣言 1・参照 2 の編集、同じ行にある BMP 外文字後の UTF-16 座標 |
| `folding.json` | CRLF、BMP 外文字、3 段の scope と block comment、複数行文字列や行コメント内の偽区切りの除外 |
| `evaluator.json` | bridge の variables / evaluate、再 launch の初期化、未知の式と NOOP の明示エラー |

golden とは別に capability 通知・編集数と references の一致・単一行折畳みの除外・未完了コメントの除外も検査する。
更新モードの後に通常モードで照合する。

`ParserUnchangedTest` は (a) LSP 生成前後の全パーサファイルの byte 一致、
(b) DAP 追加前後の全パーサ / LSP ファイルの byte 一致、(c) パーサに lsp4j 依存が無いこと、
(d) `--emit-lsp` 無指定ではファイルが一つも出ないことを検査する。
既定 Maven ビルドは `ubnfc-lsp` を含まず、追加した lsp4j の依存も `ubnfc-lsp` の test scope だけである。
評価器の例はテスト時のみ合成し、既定 DAP に読み込ませない。

既存 `MetadataTest` の UBNF capture 位置の期待値には古い IR の ID が残っていたため、
現在コミット済みの IR の declare/use site に合わせて修正した（fixture と pin は変更しない）。

CI の native job に `mvn -q -Plsp -pl ubnfc-lsp -am test` を追加し、生成物のコンパイル、
上記の不変条件と golden を継続して検査する。`-Dtest=org.unlaxer.ubnfc.lsp.*Test` と
`-Dsurefire.failIfNoSpecifiedTests=false` で LSP パッケージの全テストを選び、reactor 内の
Java backend テストを重ねて走らせない。既存の Java jar build → Rust workspace test の順序は保つ。

## 6. 検証結果

- `mvn -o -q -Plsp -DskipTests package`: 成功。
- `mvn -o -q -Plsp -pl ubnfc-lsp -am test "-Dtest=org.unlaxer.ubnfc.lsp.*Test" -Dsurefire.failIfNoSpecifiedTests=false`:
  15 テスト、失敗・エラー・skip とも 0。golden 更新後の通常照合で確認。
- `mvn -o -q -Plsp test`: Java backend 11,885 テスト + LSP 15 テスト、すべて成功。
- `SKIP_VSIX=1 scripts/build-vsix.sh --with-dap all`: P4 は30テスト成功。
  UBNF は下記の既存 golden を補正後、`mvn -o -q -f examples/ubnf-vscode/server/pom.xml -Pdap test`
  の通常照合で21テスト成功。VSIX の組立て・実機への導入は実施していない。
- `which code`: exit 1・出力なし。実機確認は実施できない。

既存 UBNF golden の4行（診断候補の残数2行と semantic token の分類2行）にも不一致があった。
`origin/master`（`66a18ff`）を専用の一時ディレクトリへ展開し、旧生成器の実応答でも同じ差分が
出ることを確かめてから、その応答に期待値を合わせた。新旧の UBNF パーサ生成物と DAP 生成物は
`diff -qr` で byte 一致し、新機能を含むサーバの通常 golden も一致した。
今回の実装による既存観測の変更として承認したものではなく、コミット済み IR と期待値のずれを補正した。

pin 済み外部 fixture を使う `RUSTUP_TOOLCHAIN=stable cargo test --workspace --no-fail-fast`
は558成功・1失敗（exit 101）。失敗は `ubnfc-harness --test cli` の既定 fixture 探索であり、
テストが環境変数を意図的に外したとき、worktree の sibling に `unlaxer-parser` が無かったため。
共有ディレクトリにリンクを作らず、専用 `mktemp -d` の中へ同じ commit を展開し、
CI と同じ sibling 配置を作って同じ pin の `cargo test -p ubnfc-harness --test cli` を再実行した。
その2テストは成功した。テストコードや pin の変更は不要だった。

その後 `origin/master`（`763eb5c`）を merge で取り込み、D-083 を保持して
D-086〜D-088 の番号に衝突がないことを確認した。取り込み後も LSP 15テストが成功し、
小文法から再生成した IR が同梱 IR と byte 一致した。統合後の head も PR の CI で検査する。
最終結果は PR の Checks と issue #1 の完了コメントに記録し、CI 成功を merge の条件にする。

## 7. 未着手と理由

| 未着手 | 理由 / 後続 issue |
|---|---|
| inlay hints / signature help / formatting / code actions | 有用な表示や編集の仕様が先に必要。[#25](https://github.com/opaopa6969/ubnfc/issues/25) |
| GrammarValidator 相当の LSP 搭載 / .tecatalog | JVM と front の境界・読込と更新の仕様が必要。[#26](https://github.com/opaopa6969/ubnfc/issues/26) へ |
| 条件付き breakpoint / hit condition / logpoint / watch 式 / setExpression / breakpointLocations / stepBack | 評価時期・副作用と巻戻しの契約が必要。[#27](https://github.com/opaopa6969/ubnfc/issues/27) へ。今回の repl の評価例は watch の実装を意味しない |
| 実機 VS Code 確認 | このマシンの `which code` は exit 1・出力なし。実機へ VSIX を導入できない。[#28](https://github.com/opaopa6969/ubnfc/issues/28) へ |

増分解析は今回の指定範囲外で、既存 #5 / #13 に残る。
