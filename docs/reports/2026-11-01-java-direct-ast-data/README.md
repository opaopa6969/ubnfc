# Java 直接 AST 構築の計測・検証記録

SWAR追加前の性能・最新corpus検証は **`after-long/`** を使用する。#44統合後の1,492行全一致（双方Pass1,328 / Fail3 / IR44）、共有Rust scanner更新後の全3セッションを収録。#33は解消済み、残るcorpus差分は#46。`post-ll1/` はLONG追加前の履歴。`final/` はleaf修正後・LL(1)統合前の履歴。pin更新直後の corpus 検証は **`integrated-pin/`**（1,492行の観測一致、双方 Pass1,315 / Fail3、既存差分は #46 / #33）を参照する。leaf fallback の参照経路に旧実装にはない空配列確保があったため、修正後に corpus と全 A/B を取り直した。初期の全記録はこのディレクトリ直下に保持し、[当時の報告](initial-report.md) とともに予備記録とする。初期の field 配列計数はこの余分な配列を含むため、最終 constructor 計数を参照する。

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

`post-ll1/java-projection-proof.json` は新pin全260buildについて、LL(1)統合前後で234buildのIR（plan.ll1以外）と各Java生成器の全ファイルが同一、26buildのIRエラーが同一である検証記録。`post-ll1/ir.sha256` は追加メタデータを持つ統合後IRで、生成Javaは統合前とbyte不変。

`after-long/reference-layout.json` はfield offsetとVM alignmentによる旧/参照Recipeの56 B一致。`source.sha256` は共有scanner・P4 lib/build等の依存も含む。`toolchains.txt` は今回の環境。基準生成器は最新master49d1a9dのarchiveからbuildした。

`after-stage5/` はLSP15件とAST/計測ソース不変、`after-stax3/` はstax実行系12件とRust比較binaryのSHA一致を記録する。これらの統合で既存のcorpus/A/B対象は変更していない。

性能の最新判定は **`after-swar/`**。master #32 / 12e5e80を統合し、Rust比較binaryを更新して同条件の交互3セッションを再測定した。Java・front・pin・harness不変なので、corpusはafter-longの全1,492行一致を維持する。

`python3 docs/reports/2026-11-01-java-direct-ast-data/after-swar/metrics.py` で同じディレクトリのCSVから再集計できる。任意の保存先は第1引数で指定する。保存済みmetrics.jsonと再集計結果のbyte一致を確認した。

`after-fuzz-build/` はmaster #55の統合後に、計測対象の不変とfuzzの追加検証10件の成功を確認した記録。性能はafter-swar、corpusはafter-longの結果を維持する。
