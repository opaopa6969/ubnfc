# 2026-09-22 Java corpus 初回実行の読み方

実測の全結果は [Markdown](2026-09-22-java-corpus.md) と
[JSON](2026-09-22-java-corpus.json) にある。以下の原因は、保存した差分と
既存コードを照合した推定であり、backend 修正の検証結果ではない。

## 実行条件と検証

- ubnfc / front / Java backend の基点: 8e23da691411b7b1a127a986777349e94d40ae10。
- upstream corpus: ed4178e673f93e719b62a7bd181f3a219abf8ed1。
- 専用 worktree: /home/opa/work/unlaxer-workspace/ubnfc-wt-runall。
- Java 21.0.9、rustc 1.98.1。Java jar はこの worktree でソース無変更のまま
  mvn -o -q -DskipTests package でビルドした。
- jar SHA-256: cdb3b98787694ee3496ba953e7154f1816da1beb835aafbb76d4aed5c05b630b。
- 依存追加・期待値変更・乖離登録変更・front/backend ソース変更なし。
- cargo test -p ubnfc-harness: 46テスト成功。
- cargo clippy -p ubnfc-harness -- -D warnings と cargo fmt --check: 成功。
- 集約理由の整理後も全件を再実行し、段階別・Verdict 別の件数が一致した。

~~~sh
cargo build --bin ubnfc
(cd ubnfc-java && mvn -o -q -DskipTests package)
cargo run -q --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-22-java-corpus.md \
  --json docs/reports/2026-09-22-java-corpus.json
~~~

CLI は全結果を保存して終了コード1を返す。これは corpus に段階失敗 / Fail があるためで、
harness のテスト失敗ではない。

## 集計単位

83ファイルの871ケースを、全236文法 variant に対して1195件に展開した。
重複文法は234 build に集約され、22 build が IR 失敗、43 build が生成拒否、
169 build が生成・javac に成功した。文法なし scope-store の9操作列は未観測。

associative は、通常文法の24 Pass と、reversed-metadata の24 IR失敗を含む。
後者は既存 RustConformanceTest の文法変形を loader が保持した variant であり、
front が E-PRECEDENCE-ORDER を出す。通常文法を IR から実走させた24 Passを E2E で確認し、
負の variant も省略せず検証する。全 variant が Pass になったという意味ではない。

## 生成拒否の機能別件数

件数は build ごとの最初の拒否理由。後続の拒否機能は未観測。

| 機能 | build 数 | case × variant |
|---|---:|---:|
| externTokens.inputState | 26 | 137 |
| recipe.conversion | 3 | 23 |
| recovery.legacyMarker | 4 | 10 |
| eval | 7 | 7 |
| typeof | 1 | 5 |
| execution.transaction:error:once | 1 | 3 |
| enum.duplicateName | 1 | 1 |
| 合計 | 43 | 186 |

IR の拒否は analysis.CaptureCardinality が101件、E-PRECEDENCE-ORDER が24件、
E-LOWER-COMMON-FIELD が2件、E-LOWER-SUM-EVAL と E-IMPORT-CYCLE が各1件。
CaptureCardinality は helper の node/list/optional に対する宣言 cardinality と
analysis の推論 Scalar が一致しない。front の調査対象として分けて扱う。

## Fail の主要パターン上位5件

対象は337 Fail実行。各観測パスの差分を含む実行数で数えているため、項目間で重複する。
差分そのものの行数ではない。

| 観測 | 実行数 | 代表例 | 原因の推定・調査対象 |
|---|---:|---|---|
| /value | 236 | primitives の ANY / 入力 a は期待 a、実測 null | 全236件が明示 null。driver は OPERATOR_TYPES に含まれる AST だけを数値評価するため、token の mapped value、semantic/cardinality の独自評価、数値 leaf の結果との対応が不足している。parser の認識失敗と区別する。 |
| /diagnostics | 76 | digit/1 は診断なしを期待し、EOF の syntax 診断が実測 | 成功時に残る試行中の EOF 診断、期待パーサ名と literal/token 表記、診断の種類・オフセットに差がある。driver の診断射影と Session の診断保持を確認する。 |
| /captures | 25 | choice-optional-coexist は item を期待する位置に choice が現れる | 子の capture を結合し外側の capture を追加する順序、capture occurrence の重複、span/text の射影が既存 oracle の順序と異なる可能性。 |
| /allConsumed | 18 | digit/4 の空入力は期待 true、実測 false | oracle の空入力での「全消費」と driver の構文受理を含む result.ok() に定義差がある。ほかに regex の Unicode 境界と LOOKAHEAD/NEGATIVE_LOOKAHEAD の cursor・trivia 差が混在する。 |
| /scope | 8 | undefined-target-warning は日本語の未定義診断を期待し、英語文面と追加キーが実測 | runner の scope 比較は driver の診断列を射影する。scope 専用の診断と構文診断が混在し、診断キー・文面にも差がある。scope状態の正否と wire表現の差を切り分ける。 |

根拠となる既存実装は以下。今回これらは変更していない。

- ubnfc-java/src/main/resources/templates/driver/Driver.java:
  execute の OPERATOR_TYPES 判定、allConsumed、diagnostics、captures、numeric。
- ubnfc-java/src/main/resources/templates/internal/Session.java:
  capture / combine の occurrence 順序、result の全入力判定・診断、applyEffects の scope 処理。
- crates/ubnfc-harness/src/runner/comparison.rs:
  value の明示 null 比較、scope_fields の診断射影、unobserved の欠落検出。

## 未観測と Known の扱い

228件の未観測は Pass に含めていない。evolution は独立した結果 oracle がない148件、
scope-store は driver protocol 未対応の9件。catalog 2件も parser driver が catalog
メタデータを返さないため未観測になる。ほかに texts、nodes、scope状態や oracle の
parsedStatus/nativeFailure 等の欠落がある。

Known 13件と Fail にも欠落観測が併存し得る。これらは未観測列に二重計上せず、
全項目を JSON の missing と Markdown の未観測項目表に保持した。
生成・コンパイルが失敗した upstream oracle の runtime null は成功期待にしない。
D-017/D-018 の既存比較・登録範囲を維持し、今回の失敗に合わせた免除を追加していない。
