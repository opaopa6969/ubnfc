# fuzz build 失敗の詳細保持（issue #54）

## 元の CI と再現試行

[PR #52](https://github.com/opaopa6969/ubnfc/pull/52) の head
`2a03b207d4326ecbc9103ccfbc0a0119c3728ea8`、
[CI run 41298379082](https://github.com/opaopa6969/ubnfc/actions/runs/41298379082)
attempt 1 の失敗ログを取得した。固定 seed campaign の a / d / e / f が
`front の discard: {"build:rust": 1}` で失敗した。Cargo / rustc の本文と失敗 seed は
このログには含まれず、資源不足・タイムアウト・生成コード不具合のいずれとも断定できない。

2026-11-01 に専用 worktree の基点 `06fa27b` と本修正で再試行した。
この基点と上記 head の間で、front / Rust 生成器 / fuzz / harness/src /
Java 生成器 / Cargo.lock / 外部 fixture pin に差分がないことを確認した。
外部 fixture は `scripts/fetch-external-fixtures.sh` で専用の一時ディレクトリへ取得した。

- `RUSTUP_TOOLCHAIN=stable`
- rustc `1.99.1 (58cf8f9c3 2026-10-13)`（元の CI と同一）
- cargo `1.99.1 (5f94df478 2026-08-27)`（元の CI と同一）
- unlaxer-parser: `d3ea73e8cfa440df8bdbb70abb29e06259435ba2`
- tinyexpression: `c736c2c66e4569ebc5e22f1f0c5125a045679d35`
- `UNLAXER_PARSER_DIR` / `UNLAXER_PARSER_ROOT` / `TINYEXPRESSION_DIR` を取得先に設定
- `CARGO_TARGET_DIR=<専用 worktree>/target/issue54`（gitignore 対象）
- Java 生成器を `mvn -q -DskipTests package` で事前に構築
- `cargo build --bin ubnfc-rust --bin ubnfc-fuzz`
- `cargo test -p ubnfc-fuzz --test campaign`（テストスレッド数は Cargo の既定）

固定 seed campaign 全 8 件が成功し、元の a / d / e / f の build 失敗は再現しなかった。
したがって原因を推測した生成器修正や build 失敗の無視は加えない。
今回の詳細保持により、再発時には実エラーと seed を同じ CI ログから取得できる。

## 変更と検証

`Engine::prepare` が返す `Discard.message` と harness の stderr / stdout 取得は
そのまま使い、失われていた campaign 集計への受け渡しを追加した。
理由別件数、機能別件数、discard ゼロの要求、比較・既知差分の判定は変更していない。
保存形式と一時パスの扱いは
[driver protocol](../harness-driver-protocol.md#fuzz-campaign-の-build-失敗を追跡するissue-54)
に記載した。

合成 Rust 生成器は依存なし crate の `compile_error!` により実際のビルドを失敗させる。
Cargo の stderr（`could not compile`）と stdout の rustc JSON 診断
（`compiler-message` と独自エラー本文）の両方を確認した。javac stub の別ケースでも
標準エラーを確認した。2 ケースとも外部開発者 tree に依存せず、seed・文法名・digest・
backend・本文を JSON / Markdown / stderr から取得でき、一時ディレクトリ削除後にも
保存した report が残ることを検証した。discard=1、compiled=0、comparisons=0、findings=[]
も確認している。

- `cargo test -p ubnfc-fuzz --test build_discard -- --nocapture`: 2 件成功
- `cargo test -p ubnfc-fuzz --lib`: 33 件成功
- `cargo test -p ubnfc-fuzz --test campaign`: 8 件成功
- `cargo clippy -p ubnfc-fuzz --all-targets -- -D warnings`: 成功
- 変更した Rust ファイルの `rustfmt --check` と `git diff --check`: 成功

いずれも stable Rust を明示した。CLI の既存の終了コード規則と一時ファイル削除は維持した。
