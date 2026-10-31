# Java 直接 AST 構築の計測・検証記録

- `s1`〜`s3` の P4 / Rust は同じ番号のセッション。
- 正式 IR の Operators は `s2` / `s3` / `s4` の 3 回。`s1` は誤指定したテスト用 IR の予備値で、`diagnostic-test-ir/` に分離し、達成判定から除外した。
- Java CSV は先頭の `#` 行に JVM と各 run の load、末尾に全 sample・中央値・最小・最大・割当 byte を記録する。初回 P4 の末尾 sample 列は引用符なしの原始出力なので、固定列を読む場合は `split(',', 9)` で先頭 9 列と sample 列を分ける。値の加工はしていない。
- Rust は各 run の load と全 sample・中央値・最小値。最大値は `metrics.json` に全 sample から集計した。
- `*-load-before.txt` / `*-load-after.txt` は `uptime` の記録。`ir.sha256` は正式な IR。
- `metrics.json` は各セッションの中央値とその中央値による代表値。AST 層はセッションごとの AST 中央値 − 認識中央値を求めてから 3 セッションの中央値を採り、その Java / Rust 比を計算する。
- `corpus.md` は新生成器の harness 出力。旧生成器との差は generator SHA-256 の 1 行だけ。
- `corpus-comparison.json` は前後の全 case の outcome / actual / differences / missing を比較した結果と、case ごとの観測 SHA-256。元 JSON 2 本の SHA-256 も記録する。既存 IR 失敗 13 件を含む 1,444 行で差分 0。
- `allocation-counts-all.csv` は一時コピーの constructor に計数を入れた別 binary。計時には使っていない。
