# Java backend の一括走査（D-072 の Java 版、2026-09-25、issue #69）

D-072（観測を要求しない終端領域を 1 本の走査へ畳む、`plan.scans`）を Java backend に移植した（D-080）。
front の計画は共有なので `crates/ubnfc-front/**` と Rust backend は触っていない。

## 結論（先に）

- **JSON 1 MB 認識は 0.0386 / 0.0380 → 0.0092 / 0.0098 µs/byte（2 セッションの中央値、比 0.24〜0.26）**。
  #48 の Rust 走査 on（0.0137 µs/byte、別日・別負荷）と同じ桁に入った。
- **JSON 1 MB typed AST は 0.360 / 0.336 → 0.174 / 0.168 µs/byte（比 0.48〜0.50）**。
- **1 parse の割当は認識 3.32 MB → 1.99 MB（0.60）、typed AST 315.7 MB → 161.1 MB（0.51）**。
- **P4 は `plan.scans` が付かないので生成 `Parser.java` は byte 一致**（再生成差分は `Session` / `Driver` / `Input` の template だけ）。
  complex / x4 / x16 / x64 の ast-fast / recognize-fast は 4 周とも雑音帯の中（§3.2）。
- 観測は動いていない: `ScanEquivalenceTest` 9,885 件（走査 on / off × 認識 / AST × 出現表 × 診断 × lexical、
  rollback 解析を含む）、ubnfc-java 全 10,786 件、corpus **Java Pass 1,310 / Fail 0**、`cargo test --workspace` 546 件、
  Rust ⇔ Java cross-backend fuzz campaign（今回から **Java の in-backend 走査 A/B** も回る）。

## 0. 計測環境

| 項目 | 値 |
|---|---|
| 基準（base） | `origin/master` `e1547ee` の生成器で作った生成物 |
| after | 本 PR の生成器。JSON は同じ生成物を `-Dubnfc.scan=false`（off = 参照経路）でも起動した |
| host | AMD Ryzen 9 7950X（32 論理 CPU）、WSL2 Linux 6.18.33.2、Oracle JDK 21.0.9 |
| JVM | `-Xms512m -Xmx2g -Xss16m`、`taskset -c 28`、単一 thread |
| A/B | **JVM 交互起動、2 セッション**。S1 は base → on → off、S2 は off → on → base の順。P4 は各セッション 2 周（S1 base → after、S2 after → base） |
| JSON | `benchmarks/java/UbnfcRunner.java`（`prepare-external.sh` と同じ生成・入力 sha256 は `examples/json/evidence/inputs.sha256` と一致）、runs=7、1 run 0.5 秒、暖機 200 反復 |
| P4 | `examples/p4-java` の `Benchmark`、`bench.warmup=1000`、`bench.runs=7`、ast-fast / recognize-fast × 4 fixture |
| 負荷 | load average 13.4 → 5.4（S1）、7.1 → 6.0（S2）。共有機、別エージェント並走 |
| 生データ | `examples/json/evidence/java-scan-69-json.csv`、`examples/p4-java/evidence/java-scan-69-p4.csv` |

## 1. 畳めた領域（JSON）

`examples/json/json.ubnf` の IR では `plan.scans` の根が 20 個あり、**Java でも 20 個すべてに走査経路が付く**
（`ScanEquivalenceTest.grammarsWithoutScanPlansAreUnchanged` が固定）。

| 領域 | 根の数 | 中身 |
|---|---:|---|
| `Ws` の参照（`Json` / `Object` / `Member` / `Array` の中） | 12 | 規則参照 `Ws` 1 つが根。中身は下の `Ws` 本体 |
| `Ws` 本体 | 1 | `{ ' ' \| '\t' \| '\n' \| '\r' }`。反復の一括走査表は `complete: true`（本体を呼ばずに表だけで終わる） |
| `String` の中身 | 1 | `{ Char }`。表で SAFE 文字を食べ、表の外（`\`）だけ本体（`Escape`）へ落ちる |
| `Char` 本体 | 1 | `Escape \| SAFE_SP \| SAFE_MID \| SAFE_BMP \| SAFE_SUPP` |
| `Escape` 本体 | 1 | `'\\' ( '"' \| … \| ESC_U HEX{4} )`（有界反復 `HEX{4}` を含む） |
| `Number` の中身 | 1 | `[ '-' ] Int [ Frac ] [ Exp ]`（Int / Frac / Exp を含む） |
| `Int` / `Frac` / `Exp` 本体 | 3 | `ZERO \| ONENINE { DIGIT }` / `'.' DIGIT+` / `( E_LOWER \| E_UPPER ) [ '+' \| '-' ] DIGIT+` |

Operators / Power / P4（`tinyexpression-p4`）は `plan.scans` を持たず、走査関数も走査経路も出ない。

## 2. Java runtime での走査

`ubnfc-java/src/main/java/org/unlaxer/ubnfc/java/emit/Scanning.java`（新規）が Rust の `crates/ubnfc-rust/src/scan.rs` と
1 対 1 に対応する生成を行う。

- 根と、根から規則参照をたどって届く全式に **`private static long scanN(char[] t, int p)`** を生成する。
  Frame・Match・memo・診断・scope・深さ計数を一切触らない純関数で、返り値は `(終了位置 << 1) | 値なし`、失敗は `-1`。
- 各節点の判定は **Java の参照経路（`Session`）の意味そのもの**を写す: 選択は宣言順、反復は「成功したら数えてから 0 進行なら止める」、
  単一 code point の終端は `Session.range` / `builtin` / `any` と同じ式（`CodePointSet` は同じ定数配列を同じ二分探索で引く）、
  literal は `Input.startsWith` と同じ比較（8 文字以下は展開）。code point の読みは `Session.cpAt`（`Input.codePointAt` と同じ。
  孤立 surrogate はその UTF-16 値）。
- 一括走査表（`class`）が付いた無上限反復は、ASCII を `long` 2 本の bitset、U+0080 以上を閉区間の比較で読む内側ループにする。
  `complete: true` なら表が尽きた位置で本体を呼ばない。そうでなければ表の外の文字で 1 周だけ本体を呼んで表へ戻る。
- 根の consumed mode method（`eNC`）の先頭に
  `if (s.scan && !f.invert && f.c == f.m) { Match r = s.scanned(f, scanN(s.chars, f.c)); … return r; }` を置く。
  matchOnly mode（`eNM`）は参照経路のまま。`f.reset` は参照経路が本体評価後に残す値へ合わせる（`setFalse` の伝播）。
- `Session.scanned` は結果を参照経路と同じ `Match` にする。AST 経路で読まれ得るのは区間と「値列が null（plain text）か空列か」だけなので、
  下位 1 bit で後者を返す（空の反復・省略された optional を通ったか。`Session.combine` / `repeat` と同じ規則）。
- 入る条件は Rust と同じ: `Session.scanning`（`-Dubnfc.scan`、既定 true）`&& !diag && !occ && !lexical`（`Session.scan`）と、
  呼出し位置の `!f.invert && f.c == f.m`。
- **farthestOffset / expected（D-027）**: 走査は診断を記録しない解析でしか走らないので、失敗した解析は D-023 により
  診断 session が参照経路で作り直す。公開される主診断は Rust と同じ勘定のまま。
- **rollback 解析**（`parseEntryRollback`）は D-023 の再解析を通らず fast session の到達位置 `reached` を診断位置として返す。
  走査は `reached` を更新しないので、`plan.scans` を持つ文法の生成 parser は rollback の解析では `s.scan = false` にしてから走らせる
  （移植途中の版は `1e` / `"\x"` などで rollback の `offset` が off と食い違っていた。`ScanEquivalenceTest` の観測に rollback を足して固定）。
- 唯一の観測差は資源上限で、走査は資源を減らす方向にしか動かない（`scanOnlyEverReducesResourceUse`: `[1]` を `maxDepth=5` で
  参照経路だけが上限に当たる）。
- harness driver は `options.scan` を Rust 生成 driver と同じ意味で受ける（`docs/harness-driver-protocol.md`）。
  fuzz campaign の in-backend A/B は Rust と Java の両方で回り、`Summary.java_scan_comparisons` に Java の回数が出る。

## 3. A/B

### 3.1 JSON（中央値 µs/byte、S1 / S2）

| 入力 | モード | base | off（`-Dubnfc.scan=false`） | **on（本 PR）** | 比 on / base |
|---|---|---:|---:|---:|---:|
| json-1k | 認識 | 0.0428 / 0.0397 | 0.0413 / 0.0462 | **0.0104 / 0.0092** | 0.24 / 0.23 |
| json-10k | 認識 | 0.0391 / 0.0390 | 0.0463 / 0.0386 | **0.0092 / 0.0102** | 0.24 / 0.26 |
| json-100k | 認識 | 0.0386 / 0.0448 | 0.0394 / 0.0515 | **0.0091 / 0.0097** | 0.24 / 0.22 |
| json-1m | 認識 | 0.0386 / 0.0380 | 0.0386 / 0.0382 | **0.0092 / 0.0098** | **0.24 / 0.26** |
| json-1k | typed AST | 0.1228 / 0.1228 | 0.6053 / 0.2913 | **0.0726 / 0.0717** | 0.59 / 0.58 |
| json-10k | typed AST | 0.1149 / 0.1156 | 0.1168 / 0.1272 | **0.0529 / 0.0480** | 0.46 / 0.42 |
| json-100k | typed AST | 0.1488 / 0.1418 | 0.1433 / 0.1463 | **0.0759 / 0.0699** | 0.51 / 0.49 |
| json-1m | typed AST | 0.3598 / 0.3363 | 0.3266 / 0.3940 | **0.1736 / 0.1681** | **0.48 / 0.50** |

off は base と同じ帯（off の json-1k AST だけ GC の当たりで外れ値）。1 MB typed AST の最小値は base 326 / 311 ms → on 114 / 117 ms。

### 3.2 P4（中央値 ms、4 周: S1 r1 / S1 r2 / S2 r1 / S2 r2）

| fixture | モード | base | after |
|---|---|---|---|
| complex | ast-fast | 0.157 / 0.135 / 0.128 / 0.126 | 0.137 / 0.139 / 0.127 / 0.124 |
| complex | recognize-fast | 0.087 / 0.077 / 0.075 / 0.076 | 0.079 / 0.078 / 0.075 / 0.075 |
| complex-x4 | ast-fast | 0.644 / 0.465 / 0.458 / 0.503 | 0.570 / 0.473 / 0.470 / 0.450 |
| complex-x16 | ast-fast | 2.102 / 1.926 / 1.726 / 1.831 | 1.983 / 1.807 / 1.833 / 1.679 |
| complex-x64 | ast-fast | 11.19 / 11.46 / 9.11 / 10.42 | 12.88 / 10.38 / 10.69 / 8.80 |
| complex-x64 | recognize-fast | 5.15 / 6.40 / 4.40 / 4.47 | 6.24 / 5.36 / 4.76 / 4.04 |

生成 `Parser.java` は byte 一致なので期待値は比 1.0 で、どの行も周ごとの揺れ（x64 で ±20%）の中にある。

### 3.3 1 parse の割当（byte）

| 入力 | base | after | 比 |
|---|---:|---:|---:|
| json-1m 認識 | 3,318,576 | 1,986,872 | **0.60** |
| json-1m typed AST | 315,734,928 | 161,118,776 | **0.51** |
| json-1k typed AST | 300,224 | 145,904 | 0.49 |
| P4 complex ast-fast | 248,032〜248,056 | 247,976〜249,352 | 1.00〜1.005 |
| P4 complex-x64 ast-fast | 10,877,200〜10,877,224 | 10,877,144〜10,959,160 | 1.00〜1.008 |

P4 の after は同じ classes でも JVM ごとに 2 値を取る（S1 r1 だけ base と同値）。生成 parser が同一で `Session` の差は
field 2 個（`boolean` と入力配列の参照、確保なし）なので、JIT の escape analysis の当たり外れと読む。

## 4. 却下案

- **(a) 走査関数を `Session` のインスタンス method にして `Input.codePointAt` / `startsWith` を呼ぶ**: 境界検査と間接参照が残り、
  Rust の `fn s{i}(&self, p)` と違って JIT が `Session` の field 読みを毎回残す。`char[]` と位置だけを受ける static 純関数にした
  （`Input.chars()` は複製しない参照を返す。呼出し側は書き換えない契約）。
- **(b) 走査結果を「走査した範囲ちょうど 1 件の token 出現」として出現表に残し、出現表を要求する解析でも走査する**（#48 報告 §5 の案 4）:
  Java の `Match` 木では出現表あり = capture 出現の組立てがあり、領域内の rule 事象の完了順も数える。Rust と同じく
  出現表を要求する解析では畳まない（`!occ`）に留めた。
- **(c) matchOnly mode（`eNM`）にも走査を入れる**: `f.c != f.m` の frame では記録位置と読み取り位置が分かれ、
  1 本の位置では表せない。consumed mode で `f.c == f.m` の呼出しだけにした（JSON では実質すべての呼出し）。
- **(d) rollback 解析の結果にも走査を許して `reached` を走査関数から返す**: 返り値に最遠失敗位置を足すと全走査関数の返り値が
  2 語になり、非 rollback の速い経路を遅くする。rollback は scope の検査用で頻度が低いので走査を切った。

## 5. 検証

- `cd ubnfc-java && mvn -q -DskipTests package && mvn -q test`: 10,786 件緑（`ScanEquivalenceTest` 9,885）。
- `examples/p4-java/scripts/generate.sh --write` で再生成し、`cd examples/p4-java && mvn -o -q package`（テスト込み）緑。
- corpus: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java --jobs 8` → **Java Pass 1,310 / 期待拒否 Pass 27 /
  Known 29 / Fail 0 / 未観測 59**（D-079 の報告と同じ合計）。
- `cargo test --workspace --no-fail-fast`（546 件、fuzz campaign の Rust ⇔ Java 差分と Java の走査 A/B を含む）/
  `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --check`: 緑。
