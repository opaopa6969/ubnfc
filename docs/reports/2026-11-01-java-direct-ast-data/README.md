# Java 直接 AST 構築の計測・検証記録

性能の最終判定は **`final/`** の修正後3セッションを使用する。corpus の最終検証は master #40 統合後の **`integrated-pin/`**（1,492行の観測一致、双方 Pass1,315 / Fail3、既存差分は #46 / #33）を参照する。leaf fallback の参照経路に旧実装にはない空配列確保があったため、修正後に corpus と全 A/B を取り直した。初期の全記録はこのディレクトリ直下に保持し、[当時の報告](initial-report.md) とともに予備記録とする。初期の field 配列計数はこの余分な配列を含むため、最終 constructor 計数を参照する。

`final/` の `s1`〜`s3` は P4 → Rust → Operators を各1回行う同じ番号のセッション。7 run の中央値・全 sample・load と `uptime` を記録する。`source.sha256` は計測した AST 構築ソース、`ir.sha256` は正式な固定 IR。`corpus-comparison.json` は現行 front で旧生成器と修正版を再比較した結果（1,444行差分0）、`corpus.md` は修正版の出力。

## 初期記録の読み方

- `s1`〜`s3` の P4 / Rust は同じ番号のセッション。
- 正式 IR の Operators は `s2` / `s3` / `s4` の 3 回。`s1` は誤指定したテスト用 IR の予備値で、`diagnostic-test-ir/` に分離し、達成判定から除外した。
- Java CSV は先頭の `#` 行に JVM と各 run の load、末尾に全 sample・中央値・最小・最大・割当 byte を記録する。初回 P4 の末尾 sample 列は引用符なしの原始出力なので、固定列を読む場合は `split(',', 9)` で先頭 9 列と sample 列を分ける。値の加工はしていない。
- Rust は各 run の load と全 sample・中央値・最小値。最大値は `metrics.json` に全 sample から集計した。
- `*-load-before.txt` / `*-load-after.txt` は `uptime` の記録。`ir.sha256` は正式な IR。
- `metrics.json` は各セッションの中央値とその中央値による代表値。AST 層はセッションごとの AST 中央値 − 認識中央値を求めてから 3 セッションの中央値を採り、その Java / Rust 比を計算する。
- `corpus.md` は新生成器の harness 出力。旧生成器との差は generator SHA-256 の 1 行だけ。
- `corpus-comparison.json` は前後の全 case の outcome / actual / differences / missing を比較した結果と、case ごとの観測 SHA-256。元 JSON 2 本の SHA-256 も記録する。既存 IR 失敗 13 件を含む 1,444 行で差分 0。
- `allocation-counts-all.csv` は一時コピーの constructor に計数を入れた別 binary。計時には使っていない。

`final/ir.sha256` のP4は測定当時のパス。#40がbyte同一のIRを `ir/frozen/tinyexpression-p4-f86ce8a5.ir.json` に移したため、現在の再現スクリプトはその固定IRを参照する。性能の入力条件は変えていない。
