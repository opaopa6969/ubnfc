# Java の反復・capture の割当削減（#35）

PR #36 の D-094 で、Java は構文成功と採用する枝の確定後に `AstEvent` から typed AST を直接構築するようになった。本変更は、その前段で反復や capture を集約するときの割当を減らす。

- 反復は、集約した列と要素単位の capture が読む項目列を一度の `Match` 構築で格納する。区切り付き反復では区切りも集約し、項目列には要素だけを入れる。空反復の意味値は空リストで表す。
- 反復全体に付いた capture は、各項目の局所出現と公開出現を足してから一度だけ `Match` を作る。集約するのはこの二つの列で、同じ項目に複数の site が付く場合は不変の `Value` を共有する。
- `join` は複数の項目が寄与する場合に最終配列へ直接書き込む。寄与元が一つの列はそのリストを返す。完了順の付番は公開出現が有効な場合に行う。

既存の `Match`、`Occurrence`、`Value` と memo の表現の中に収まる変更を選んだ。共有配列へ移す設計には、投機的評価の巻き戻し、左因数分解による site の付け替え、memo から再生する値の生存期間を一緒に扱う必要がある。

## 計測の再現

比較元は `568dca6bb84e9042fa7df54c46ea151e3d575b76`。手順は [ベンチマーク README](../../benchmarks/README.md) の Java 直接 AST A/B を使う。比較元と変更後の Java parser を同一 JVM 内で別々にロードし、両方とも直接 AST を選ぶ。Java は Oracle JDK 21.0.9、`-Xms512m -Xmx2g -Xss16m`。Rust の参照実行は同じ core 28 を使う。ウォームアップ、入力、固定 IR、反復数は #36 と同じ。各セル 7 回を 3 セッション測り、run とセッションごとに Java の先行順を反転する。

代表値はセッション中央値の中央値。AST 層は各セッション内で AST 中央値から認識中央値を引いた後、差の代表値を算出する。Java と Rust の AST 層の比はその代表値同士で取る。計時の allocation byte は main thread の割当カウンタ差分を反復数で割った値。

## 認識中の割当と保持

生成ソースの一時コピーで `Match`、`Occurrence`、`Value`、`AstEvent`、`Recipe` のコンストラクタを計数した。下表は各入力を 1 回解析したときの生成数。公開 node span の件数も一致した。

| 入力 / モード | Match 前 → 後 | Occurrence | Value | AstEvent | Recipe | 公開 node span |
|---|---:|---:|---:|---:|---:|---:|
| complex AST | 997 → 871 | 279 | 479 | 142 | 0 | 154 |
| complex 認識 | 658 → 658 | 13 | 13 | 0 | 0 | 0 |
| complex-x64 AST | 60,343 → 52,909 | 17,478 | 29,585 | 8,773 | 0 | 9,730 |
| complex-x64 認識 | 39,844 → 39,844 | 832 | 832 | 0 | 0 | 0 |

認識完了時に採用した `Match` と memo payload から `Match` の項目・出現・意味値、`AstEvent` の capture、`Value` のノード・要素を辿り、同一実体を一度だけ数えた。参照破棄前後のどちらも集計は前後一致した。

| 入力 / モード | Match（破棄前 → 後） | Occurrence | Value | AstEvent |
|---|---:|---:|---:|---:|
| complex AST | 165 → 1 | 175 → 153 | 351 → 287 | 142 → 120 |
| complex 認識 | 216 → 1 | 0 → 0 | 0 → 0 | 0 → 0 |
| complex-x64 AST | 9,930 → 1 | 11,011 → 9,729 | 21,771 → 18,116 | 8,773 → 7,491 |
| complex-x64 認識 | 13,194 → 1 | 0 → 0 | 0 → 0 | 0 → 0 |

この計数は上記の参照から到達する実体の個数。認識の途中で保持される最大バイト数や JVM 全体の生存ヒープ量は JFR の情報と区別する。

### JFR の自己サンプルと割当元

計時と別の JVM で P4 complex の 1,000 回ウォームアップ後に `profile` 設定で 12 秒ずつ収録した。core 27、同じ JDK とヒープ設定で `-XX:TLABSize=128k -XX:-ResizeTLAB` を加えた。main thread の `jdk.ExecutionSample` を 2 ms 間隔で収録し、先頭フレームで集計した。割当は `jdk.ObjectAllocationSample` の weight を型とフレームで集計した。収録開始前の割当を含み得る各収録の最初の main thread 割当サンプルは除いた。JFR の収録中も JIT と他プロセスの負荷の影響がある。

| モード | 解析回数 前 → 後 | 自己 CPU 標本 前 → 後 | 割当標本 前 → 後 | 最大割当型 `Match` の weight 前 → 後 |
|---|---:|---:|---:|---:|
| AST | 206 → 179 | 128 → 115 | 237 → 184 | 10.45 / 37.25 MB → 7.57 / 30.72 MB |
| 認識 | 124 → 168 | 57 → 93 | 42 → 61 | 5.09 / 10.06 MB → 5.88 / 13.32 MB |

変更前の AST で `Match` を確保した上位フレームは `wrap`（2.61 MB）、`rule`（2.35 MB）、`combine`（2.22 MB）。認識では `Match.empty`（4.18 MB）が最大。自己 CPU の標本は AST 変更前の `rule` 6、`MemoTable.locate` と `project` 各 5、変更後の `sequence` 6、`MemoTable.locate` 4。認識の `MemoTable.locate` は前後 6 / 10。確保箇所の順位と CPU 標本の順位は別々に扱う。

この 4 収録では GC pause と main thread の old-object 標本を観測しなかった。別の既定 TLAB の変更前 complex 収録では、GC pause は AST 59.65 ms、認識 83.09 ms（各 1 件）。GC 後の JVM の使用ヒープは 4,503,984 / 4,469,320 byte。競合下の短い収録のため、GC 時間や生存ヒープを速度差の原因と判定できるだけの資料は得られなかった。

変更前 x64 AST の補助収録（既定 TLAB、8 秒）は実行 2 回、自己 CPU 標本 79 件で、`rule` 8、`MemoTable.locate` 6、`replayAll` 4。最初の標本を除くと有効な割当標本はなかった。x64 の割当差には A/B のスレッド割当量とコンストラクタ計数を使う。

## 挙動の検証

`scripts/fetch-external-fixtures.sh` で現行 pin を取得し、`UNLAXER_PARSER_DIR`、`UNLAXER_PARSER_ROOT`、`TINYEXPRESSION_DIR` を設定して検証した。

- `mvn -q -DskipTests package` と P4 生成物の `generate.sh --check` は成功。
- P4 Java のテストは 34 件成功。canonical JSON golden 13 件、既存の同値テスト 5 本、直接構築の同値比較を含む。
- Java 生成器の全テストは 11,888 件中 1 件が高負荷時に既定の 60 秒でタイムアウトした。該当する `ReviewRegressionTest#p4HasOnlyFrontRecipeMismatchesAndAllStatefulExternsCompile` は単独でも同じ制限に達し、ローカル再検証だけ `-Djunit.jupiter.execution.timeout.mode=disabled` を指定すると通過した。残りのテストには投機的な枝の例外・span を確認する `DirectAstTest` の 3 件を含む。
- Java corpus は同一 front・同一 pin・同一 worker 数で比較した。展開後 1,492 件の `actual`、`outcome`、`differences`、`missing` と全区分の集計が一致。両方とも Pass 1,328、期待拒否 Pass 27、Known 29、既存 Fail 3、未観測 61、既存 IR 失敗 44、生成拒否・javac 失敗・build 失敗・driver error は各 0。既存差分により両方とも終了コード 1（[#46](https://github.com/opaopa6969/ubnfc/issues/46)）。
- `cargo test --workspace` の Java/Rust 固定 seed 差分テスト `campaign` は 8 件成功。

計時の全サンプル、`uptime`、目標の判定は [issue #35](https://github.com/opaopa6969/ubnfc/issues/35) の対応 PR に記録する。
