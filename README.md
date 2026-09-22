# ubnfc — UBNF parser generator (clean-room)

UBNF 文法から、実行時ライブラリに依存しない高速な Java / Rust パーサを直接生成する
ジェネレータ。unlaxer-parser のコンビネータ・ランタイム（ParseContext /
トランザクション / Token スタック）は一切使わない。

- 入力: `.ubnf`（unlaxer-parser と同じ文法言語。既存の全 `.ubnf` が通ることが目標）
- 出力: `*.java`（JDK 21、依存なし） / `*.rs`（std のみ、serde は optional）
- 構成: `ubnfc-front`（Rust: 字句・構文・検証・解析 → Grammar IR JSON）
  → `ubnfc-java`（Java: IR → Java）/ `ubnfc-rust`（Rust: IR → Rust）

設計は `docs/design/` を参照。
