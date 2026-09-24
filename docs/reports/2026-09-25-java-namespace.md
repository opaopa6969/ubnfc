# Java namespace を org.unlaxer.ubnfc へ（issue #82 / D-076）

## 背景

owner が Maven Central で namespace 検証を通せるのは `org.unlaxer` だけで、`org.ubnfc` は持っていない。
`org.ubnfc` の下のものは Central に出せず、生成コードの既定 package をそこに置くと利用者に「公開できない
package」を配ることになる。

## 対応表

| 旧 | 新 |
|---|---|
| groupId `org.ubnfc`（root / `ubnfc-java` / `ubnfc-lsp` / `runtime-template` / examples の pom） | `org.unlaxer`（artifactId・version は不変） |
| `org.ubnfc.java.*`（生成器、`org.ubnfc.java.Main`） | `org.unlaxer.ubnfc.java.*`（`org.unlaxer.ubnfc.java.Main`） |
| `org.ubnfc.lsp.*`（LSP/DAP 生成器） | `org.unlaxer.ubnfc.lsp.*` |
| `org.ubnfc.p4.*`（examples/p4-java、p4-vscode の LSP/DAP） | `org.unlaxer.ubnfc.examples.p4.*` |
| `org.ubnfc.ubnf.*`（examples/ubnf-vscode） | `org.unlaxer.ubnfc.examples.ubnf.*` |
| 生成コードの既定 package `generated.<文法名小文字>` | `org.unlaxer.ubnfc.generated.<文法名小文字>` |

VSIX サーバの Main-Class は `org.unlaxer.ubnfc.examples.{p4,ubnf}.lsp.ServerMain`、DAP は
`….lsp.dap.DapMain`。jar 名（`ubnfc-p4-lsp-server.jar` / `ubnfc-ubnf-lsp-server.jar`）は不変。

## 生成 package の決まり方

`--package` > 文法の package（IR `grammarUnits[].package`）> `org.unlaxer.ubnfc.generated.<grammar>`。
`--package` は従来からあり、動作は不変（tinyexpression のように vendor する側は自分の package を渡す）。
既定値は `JavaEmitter.defaultPackage(grammar)` の 1 箇所で、LSP の `--parser-package` の既定も同じ関数を使う。
`ubnfc gen --target java` / harness は従来どおり `--package target` を明示する。
`PackageOptionTest`（新規 3 件）が 3 段の優先順を固定する。`ParserUnchangedTest` は期待する package 行・
出力パスを新しい名前に書き換えた（検査の強さは不変）。

## 機械的な改名

`scripts/rename-java-namespace.sh` が `git mv`（ファイル単位）+ `sed` で行う。冪等で、並行 PR を merge した後に
再実行すれば旧ディレクトリに増えたファイルも移る。`--check` は残りがあれば非 0。
origin/master（b15d1de）の detached worktree で 2 回実行し、本 PR との差が手作業分（既定 package・
`PackageOptionTest`・docs・D-076）だけであることを確かめた。

## Rust 側

crate 名 `ubnfc-*` は不変。IR / canonical JSON / driver protocol に `org.ubnfc` の literal は無い
（観測可能な出力の契約は不変）。変わるのは Java backend を起動する main class 名だけ
（`crates/ubnfc-front/src/cli.rs`、`crates/ubnfc-harness/src/{runner,runall}/mod.rs`、テスト 2 件）。

## 意図して残した `org.ubnfc`

当時の記録として書き換えない: `docs/reports/**`（スタックトレース・当時のパス）、`docs/reviews/**`、
`examples/p4-java/evidence/**`（`tests.json`、`legacy-comparison-heavy.log`、`speed-match55.csv`）。
ほかに `scripts/rename-java-namespace.sh`（対応表そのもの）と `docs/design/03-design-v1.md` の D-076 行。

## 検証（2026-09-25、WSL 32 core、負荷 10〜15）

| 項目 | 結果 |
|---|---|
| `ubnfc-java` `mvn -o -q test` | 901 件中、`PackageOptionTest` 修正後に全件緑。`ReviewRegressionTest.p4HasOnlyFrontRecipeMismatchesAndAllStatefulExternsCompile` は並列実行＋高負荷で 60 秒 timeout、単独実行で 35 秒で緑（改名と無関係、#73 の範囲） |
| `mvn -o -q -Plsp -pl ubnfc-lsp -am test` | 緑（ubnfc-lsp 14 件） |
| `examples/p4-java` `mvn -o -q test` / `scripts/generate.sh --check` | 32 件緑 / 差分なし（新 package で再生成した結果と一致） |
| `scripts/build-vsix.sh --with-dap all` | P4・UBNF とも VSIX 生成、server golden テスト 30 / 21 件緑、jar 内に `org/ubnfc/` 無し |
| `cargo test --workspace --offline --no-fail-fast` | 536 passed / 0 failed |
| corpus `run-all` | Java Pass 1310 / Rust Pass 1292、Fail 0 |
| JSONTestSuite（`prepare-external.sh` + `check-jsontestsuite.sh`） | ubnfc Java / Rust とも y 95/95・n 188/188、suite との不一致なし |
| tinyexpression `regenerate-ubnfc-parser.sh --check --require-full`（scratch コピー、読むだけ） | 現 pin（6d909d0）で緑 |

tinyexpression が pin を本 branch 以降へ動かすときは、同スクリプトの 4 箇所（main class、
`scanners_source` のパス、P4Scanners の `package` と generated package の sed）を新しい名前へ直して `--write`
する。scratch コピーで確認した結果: `generated/**` は origin/master で `--write` した結果と byte 一致、
`P4Scanners.java` は先頭の由来コメント（ソースパス）1 行だけが変わる。
