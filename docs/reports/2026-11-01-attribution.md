# P4 complex / JSON 100 KB の帰属計測（2026-11-01、issue #6）

計測元は master `66a18ff8e71839b893d2edabdad14825f8436dae`。
手順と定型は [`benchmarks/attribution/README.md`](../../benchmarks/attribution/README.md)。
この変更は計測の道具だけで、front・parser の意味論を変更しない。

## 利用できた手段と条件

| 手段 | 実測した結果 |
|---|---|
| host の perf / valgrind | 未インストール。sudo / apt は使用せず |
| 通常権限の Docker perf | `No permission to enable task-clock event`、exit 255 |
| Docker perf + `PERFMON` + `seccomp=unconfined` | task-clock / cycles / instructions / branches / branch-misses / cache-misses と、`cpu-clock:u` の CPU sampling が利用可能 |
| Docker valgrind | 3.19.0。cachegrind の smoke と、callgrind の関数別命令・cache / branch simulation が利用可能 |
| Java JFR | JDK 21.0.9 の `profile` 設定を API で暖機後だけ録画。`jdk.ExecutionSample` を取得 |
| Java PrintInlining | `-XX:+UnlockDiagnosticVMOptions -XX:+PrintInlining` で inline 判断を取得 |

WSL2 `Linux 6.18.33.2-microsoft-standard-WSL2`、Ryzen 9 7950X、32 logical CPU。
`perf_event_paranoid=2`。host の設定は変更せず、perf 用に作った container だけに capability を付けた。
image は `debian:bookworm-slim` + valgrind / linux-perf / binutils、
作成 image ID は `sha256:c958d6385a07bb67f268aee0bf64a91d819ad3e415e8d8ba809dd3937e4bb92e`。
perf は `6.1.190`、Rust は `stable` = `rustc 1.99.1 (58cf8f9c3 2026-10-13)`、
release の最適化を維持して `debug=1` を足した。Java は `-Xms512m -Xmx2g -Xss16m`。

P4 Rust は commit 済み IR から現行 backend で生成し、Extern は既存の Rust registry。
Java P4 は commit 済み生成物と `P4Scanners`、JSON は両 backend とも現行生成器を使う。
認識は観測 off、AST は所有 typed AST・観測 off。parse 結果の破棄も含める。
Java の sink は直前の結果 1 件を保持する。Rust の P4 は既存の比較と同じく parse ごとに registry を作る。

| 入力 | byte | SHA-256 |
|---|---:|---|
| P4 `complex.tiny` | 332 | `bc7f46e3b93179d67cbdd51b20f6bc74dfe960d8edf6b86e34f46a195fe8e500` |
| `json-100k.json` | 100,039 | `d7a4d32f25212e517f1a4e48cf684326c3874fb49e566f2f541911e9798d2ee1` |

JSONTestSuite は `1ef36fa01286573e846ac449e8683f8833c5b26a`。
既存の `make-json-inputs.py` を使い、`examples/json/evidence/inputs.sha256` と一致した。
外部テスト fixture も `scripts/fetch-external-fixtures.sh` で現在の pin を専用一時ディレクトリへ取得し、
`scripts/external-pins.env` は変更していない。

## 読み方と限界

すべて CPU 30 に固定し、4 ケースを順番に実行した。同じホストには他担当の Rust / Java の
ビルド・テストが並走していた。各ケースの `*.load.txt` に load と process を保存した。
**今回は共有負荷下の現状を 1 回採ったもので、過去の時間値との速度差や改善率は判定しない。**
命令数にもコンパイラ・生成物・allocator の違いがあるため、2026-09-23 / 24 の計数と直接比較しない。

- perf stat と record は別実行。P4 100,000 / JSON 5,000 parse、各 100 parse 暖機。
  stat と sample は起動・暖機も含む（解析回数に対して 0.1% / 2% の暖機）。
  自己時間は `cpu-clock:u`、499 Hz の period / 反復数による標本推定で、精密な関数タイマーではない。
- callgrind は 100 parse 暖機後、10 parse だけを公式 client request で計装する。
  cache は収集開始直後の 1 parse が cold、残りが warm。
  `--collect-atstart=no` と `--toggle-collect` だけでは暖機分の `calls=` が残ることを試走で確認したため、
  **最終採取は `--instr-atstart=no` + 開始／停止 request に修正**した。試走の数値は採用していない。
- cache model は I1 / D1 各 32 KiB・8-way・64 B line、LL 64 MiB・direct mapped。
  **Bcm / Bim / D1 / LL miss は模擬値**。perf の実機 `branch-misses` / 汎用 `cache-misses` event と同じ値ではない。
- JFR は 2,000 parse 暖機後、各 15 秒を録画し、main thread の先頭 frame だけで自己 CPU sample を集計。
  compiler / GC thread を混ぜない。絶対自己時間・命令数・miss 数は JFR では未取得。
- inline 済みの操作回数は関数呼出し数から復元できない。bounds check の費用も単独では未分離。
  次の改善は必ず同じ条件で A/B し、sample の順位をそのまま改善率とは解釈しない。

## Rust の全体像

| 入力 / mode | perf stat 実時間 ms/parse | task-clock µs/parse | 実機 instructions/parse | 実機 branch-misses/parse | 実機 cache-misses/parse |
|---|---:|---:|---:|---:|---:|
| P4 / 認識 | 0.1096 | 109.717 | 830,853.4 | 2,105.3 | 3,463.0 |
| P4 / AST | 0.1373 | 137.399 | 1,086,121.0 | 3,377.9 | 6,400.5 |
| JSON / 認識 | 1.9990 | 2033.154 | 20,016,699.7 | 20,720.4 | 5,970.3 |
| JSON / AST | 14.8635 | 15064.470 | 54,496,215.3 | 55,851.6 | 159,282.1 |

実機カウンタは起動・暖機込み。上の実時間は driver の measured 区間。各 event の実行率は生の stat CSV に残す。

### 回数 × 単価（全体。単位は入力 byte）

| 入力 / mode | 回数 byte/parse | 単価 ns/byte（実時間） | Ir/byte（模擬） | Bcm+Bim/byte（模擬） | D1mr+D1mw/byte（模擬） | ILmr+DLmr+DLmw/byte（模擬） |
|---|---:|---:|---:|---:|---:|---:|
| P4 / 認識 | 332 | 330.259 | 2566.803 | 17.6377 | 14.2867 | 2.3539 |
| P4 / AST | 332 | 413.666 | 3335.708 | 27.2178 | 21.4437 | 2.9994 |
| JSON / 認識 | 100,039 | 19.982 | 195.516 | 0.7191 | 0.0386 | 0.0030 |
| JSON / AST | 100,039 | 148.577 | 535.320 | 3.5247 | 3.2006 | 0.1580 |

旧報告の「runtime 操作 36,772 回」や「19.3 操作/byte」は今回の単位ではない。inline に依存せず再現できる byte 単価と、下の非 inline 関数の呼出し単価を併記する。

### 関数別の上位（自己 CPU 時間の順）

**P4 / 認識**

| 関数 | 自己 CPU % | 自己 CPU µs/parse（推定） |
|---|---:|---:|
| `P4::trivia_1::<false>` | 6.74 | 7.916 |
| `ScopeStore::restore` | 5.94 | 6.974 |
| `Memo::get` | 5.73 | 6.733 |
| `Memo::insert` | 4.35 | 5.110 |
| `__memcmp_evex_movbe` | 2.56 | 3.006 |
| `P4::store` | 2.49 | 2.926 |
| `P4::r121_c` | 2.46 | 2.886 |
| `P4::r98_c` | 1.98 | 2.325 |

**P4 / AST**

| 関数 | 自己 CPU % | 自己 CPU µs/parse（推定） |
|---|---:|---:|
| `P4::trivia_1::<false>` | 5.89 | 7.756 |
| `P4::finish_checked` | 5.86 | 7.715 |
| `ScopeStore::restore` | 4.96 | 6.533 |
| `Memo::get` | 4.90 | 6.453 |
| `Memo::insert` | 3.26 | 4.289 |
| `__memcmp_evex_movbe` | 2.13 | 2.806 |
| `P4::r121_c` | 2.07 | 2.725 |
| `_int_malloc` | 1.78 | 2.345 |

**JSON / 認識**

| 関数 | 自己 CPU % | 自己 CPU µs/parse（推定） |
|---|---:|---:|
| `Input::cp_at` | 17.17 | 355.511 |
| `from_utf8` | 15.72 | 325.451 |
| `JSON::r5_c` | 14.95 | 309.419 |
| `JSON::r6_c` | 10.76 | 222.846 |
| `JSON::r2_c` | 7.24 | 149.900 |
| `JSON::r3_c` | 5.83 | 120.641 |
| `JSON::r4_c` | 4.10 | 84.970 |
| `JSON::literal::<false>` | 3.72 | 76.954 |

**JSON / AST**

| 関数 | 自己 CPU % | 自己 CPU µs/parse（推定） |
|---|---:|---:|
| `JSON::finish_checked` | 36.63 | 5528.256 |
| `JSON::build_values_into` | 18.50 | 2792.786 |
| `JSON::semantic_text` | 6.26 | 945.090 |
| `__memmove_avx512_unaligned_erms` | 3.95 | 596.393 |
| `JSON::build_rule` | 3.82 | 576.353 |
| `JSON::r5_c` | 3.72 | 561.523 |
| `JSON::r6_c` | 2.36 | 355.511 |
| `from_utf8` | 2.25 | 339.880 |

### 回数 × 単価（主要関数の自己コスト）

calls は callgrind、自己時間は perf の別実行。再帰文脈に付く `'2` 等は保持し、別関数と二重計上しない。`build_values_into` は再帰文脈を合算して対応し、その他は文字通り一致する symbol の時間だけを対応させた。

| 入力 / mode / 関数 | calls/parse | 自己 CPU ns/call（推定） | Ir/call | Bcm+Bim/call | D1 miss/call | LL miss/call |
|---|---:|---:|---:|---:|---:|---:|
| P4 / 認識 / `P4::trivia_1::<false>` | 912 | 8.680 | 57.751 | 0.2809 | 0.1351 | 0.0044 |
| P4 / 認識 / `ScopeStore::restore` | 2,531 | 2.755 | 24.664 | 0.0554 | 0.0205 | 0.0015 |
| P4 / 認識 / `Memo::get` | 653 | 10.312 | 82.801 | 0.3723 | 0.7830 | 0.0078 |
| P4 / 認識 / `Memo::insert` | 487 | 10.493 | 105.908 | 0.0959 | 1.1862 | 0.1086 |
| JSON / 認識 / `from_utf8` | 87,951 | 3.700 | 37.962 | 0.1181 | 0.0001 | 0.0000 |
| JSON / 認識 / `Input::cp_at` | 87,951 | 4.042 | 31.108 | 0.0166 | 0.0039 | 0.0000 |
| JSON / AST / `JSON::finish_checked` | 1 | 5528256.469 | 11004417.000 | 153882.6000 | 86248.0000 | 388.8000 |
| JSON / AST / `JSON::build_rule'2` | 20,072 | 未分離 | 175.461 | 1.4164 | 0.4426 | 0.0262 |
| JSON / AST / `build_values_into`（再帰文脈合算） | 20,073 | 139.131 | 68.000 | 0.0004 | 1.1242 | 0.0006 |

## Java の自己 CPU サンプルと inline 判断

| 入力 / mode | main thread の標本数 | 上位の先頭 frame（自己 sample %） |
|---|---:|---|
| P4 / 認識 | 481 | `P4/rt/MemoTable.locate` 26.611%、`P4/internal/Session.restore` 10.187%、`P4/internal/Session.sequence` 5.821%、`P4/internal/Session.choice` 3.534%、`java/util/Arrays.binarySearch0` 2.703% |
| P4 / AST | 451 | `P4/internal/Session.rule` 12.639%、`P4/rt/MemoTable.locate` 11.086%、`P4/internal/Session.restore` 6.208%、`java/util/Arrays.binarySearch0` 5.987%、`P4/internal/Session.sequence` 3.991% |
| JSON / 認識 | 788 | `JSON/internal/Session.keep` 12.817%、`JSON/JsonParser.scan58` 11.675%、`java/util/Arrays.binarySearch0` 10.660%、`JSON/internal/Session.rule` 10.025%、`JSON/internal/Session.sequence` 8.629% |
| JSON / AST | 812 | `java/util/IdentityHashMap.put` 19.212%、`JSON/JsonParser.b3` 15.517%、`JSON/internal/Session.rule` 10.222%、`JSON/JsonParser.b2` 8.005%、`JSON/internal/Session.join` 6.897% |

JFR の自己 sample % は AST 増分の割合でも割当 byte の割合でもない。Java の絶対自己時間は未取得のままとする。

PrintInlining では P4 の `Session::choice`（411 byte）と `sequence`（520 byte）に
`callee is too large` が繰り返し現れた。一方 `MemoTable::locate`（95 byte）は初期には
`callee is too large`、hot な呼出し元では `inline (hot)` が現れた。
**memo の費用を「inline されないこと」だけでは説明できない**。JFR と併せて hash / probe / payload を調べる。
Java JSON の AST は `IdentityHashMap.put` 19.21%、生成 `b3` 15.52%、`b2` 8.00% が上位。
既存の Java AST 改善 issue #4 と合わせて検討する材料になる。

## 主要な事実と次の 3 件

入力の利用頻度が未定なので、ケースを跨ぐ百分率の加算はしない。
各ケースの上位から、相互に異なる機構を改善候補として選んだ。

1. **JSON AST の後処理**: `finish_checked` 36.63% + `build_values_into` 18.50% = 自己 CPU 55.13%。
   `finish_checked` は自己 Ir 20.55% に対し CPU が 36.63%、自己 D1 miss は 86,248/parse。
   cache の模擬値だけから因果は断定できないが、inline された event 走査・木の写像・span を分ける優先度が高い。
   [#22「JSON の AST 後処理と event 走査の自己時間・局所性を改善する」](https://github.com/opaopa6969/ubnfc/issues/22)。
2. **JSON 認識の文字復号**: `cp_at` 17.17% + `from_utf8` 15.72% = 自己 CPU 32.89%、
   自己 Ir は 31.06%。各 87,951 calls/parse で、ASCII も長さ 1 の slice を `from_utf8` に通している。
   次は安全な ASCII 早期 return を小さく A/B できる。
   [#20「cp_at の ASCII 経路で反復する UTF-8 検証の単価を下げる」](https://github.com/opaopa6969/ubnfc/issues/20)。
3. **P4 の memo lookup / 保存**: Rust 認識の get 5.73% + insert 4.35% + store 2.49% = 自己 CPU 12.57%、
   自己 Ir は 16.57%。get 653 / insert 487 calls/parse。Java 認識でも `MemoTable.locate` が 26.61%。
   memo の利益を残し、同じ hit / entry 数をより安く処理する候補を調べる。
   [#21「P4 の memo lookup・保存の単価を hit 数を保って下げる」](https://github.com/opaopa6969/ubnfc/issues/21)。

3 件とも受け入れ条件に観測同値・全体カウンタ・CPU 固定の交互 A/B を書いた。
今回の数字は改善率の予言ではない。P4 の `trivia_1`（認識 6.74%）と scope restore（5.94%）も残るが、
後者を memo の費用に重ねて数えていない。既存の前提を守って 1 PR ずつ revert できる単位にした。

## 採取の検証と証拠

- 準備から Rust / Java 各 4 ケースの全採取を実行し、各反復で受理と AST の有無を検証した。
- perf の event 実行率は全ケース 100%。P4 認識 5,861 / AST 6,568、JSON 認識 5,165 / AST 37,655 samples。
  JSON AST の lost は 10（約 0.027%）、他の 3 ケースは 0。
  JSON AST は `Check IO/CPU overload!` の警告があり、共有負荷の制約として記録する。
- Python 集計テスト 3 件（包含 edge の除外・圧縮関数名、main thread のみ、perf period と空 sample の拒否）、
  Bash 構文、Rustfmt、C wrapper の `-Wall -Wextra -Werror` が成功。
  CI の native job に構文・集計検査を加えた。
- 生データは [`2026-11-01-attribution/`](2026-11-01-attribution/) に保存。
  `rust-*.tsv` は全関数の自己コスト、`perf-*.tsv` は自己 CPU 推定、`java-*.tsv` は自己 sample。
  `*.callgrind.gz` / `*.samples.json.gz` / `*.inlining.txt.gz` は元の計数・CPU sample・inline 出力。
  stat CSV、採取ログ、load、反復数、versions と入力 hash も同梱した。
  gzip を専用一時ディレクトリに展開すれば、`summarize.py` で TSV を再作成できる。

作成した container は `--rm` で削除する運用とし、専用 image と一時ビルド一式は PR の完了時に削除する。
既存の container・image には操作していない。
