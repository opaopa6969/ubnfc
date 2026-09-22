# 適合性 harness

`cargo run -p ubnfc-harness -- list [--root UNLAXER_PARSER_DIR]` で件数を表示する。
既存5系統は指定した unlaxer-parser、第6系統はこの crate と同じ checkout の
`corpus/oracle/` を読む。cwd や `--root` で独自 oracle の参照先は変わらない。
個別の採取先は `corpus::oracle::load_all(path)` で指定できる。

oracle は63ファイル・168ケース。実行153（通常152、parser/diagnose の NPE 1）、
生成 Java のコンパイル失敗14、生成失敗1。台帳 `divergences.json` は件数から除く。
`stage` を先に判定し、未実行を `GenerationFailure`、実行を `Oracle` に正規化する。
NPE の `accepted:null`、prefix 成功と全入力消費、構文受理と mapping 例外を区別する。
採取元 SHA・import 原文・parseError・diagnosticError・追加の観測は `CorpusFile.raw`
に全て保持する。grammar metadata には cases 以外を保持する。

## D-017 expectedDivergence

台帳の `file` は oracle ディレクトリ直下のファイル名、`caseId` は元 JSON の ID。
全エントリの参照と重複を検証し、`follow-spec` のみを `Case.known_divergences` に付ける。
`undecided` は免除しない。ケース単位や機能全体への包括的な免除は行わない。

`divergence::compare(&expected, &actual, &case.known_divergences)` は、runner が返す
同じ `Expectation` 型と比較する。現段階では実行器を含まない。

| observation | 比較対象 |
|---|---|
| accepted | `accepted` と `allConsumed`（prefix と全入力の受理状態） |
| consumed / matched | それぞれの code point cursor |
| ast | canonical AST と構築元の順序付き `captures`（binding/span/text を含む） |
| diagnostics | 公開診断列 |
| mappingError | 構文受理後の mapping 例外 |
| generation | 生成/コンパイルの成否と失敗メッセージ |

`declarations` は常に厳密比較する。生成に失敗した側の null/空配列は runtime oracle
ではないため、生成成功への変化は `/generation` の1差分として扱う。
既存5系統への台帳免除は適用しない。

- 値が一致: `Pass`（登録済みでも差がなければ Pass）。
- 実際の差分がすべて登録済み観測群: `KnownDivergence(diffs)`。
- 未登録の差分を含む: `Fail(diffs)`。既知分も含む全差分を JSON Pointer と値付きで返す。

既知判定は仕様への適合を証明しない。登録済み観測群で oracle と異なることの分類であり、
将来の runner の仕様テストと併用する。採取値自体は変更しない。

## 台帳の判断

README の疑義1〜11を実際の grammar/input/観測と照合し、29ケース・75項目を登録した。
`follow-spec` 60項目、`undecided` 15項目。各項目の根拠は台帳の reason に記載する。
未決定は次の契約が確定していないためで、採取値の修正や推測した期待値の追加はしない。

- 回復した root/statement の AST 表現、および正常に回復した入力の診断契約。
- `@whitespace` がない line comment の改行消費と、javaStyle と明示コメント設定の合成。
- 循環 import は API が正しく失敗する一方 CLI が成功しており、単一 generation 観測での扱い。

`comment-line/2` は // 未設定なので拒否は維持し、不正に // を走査した診断だけを登録。
`comment-block/1` も未設定の /* */ を拒否した対照例なので登録しない。
正常な root recovery 入力、同期文字列がない入力、README 末尾の特異だが既知の契約、
CLI generator 名や短名警告などの運用差を、一律の乖離にはしない。
