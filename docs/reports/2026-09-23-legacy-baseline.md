# 旧実装（tinyexpression P4 + unlaxer）の complex.tiny ベースライン確定（2026-09-23）

`docs/design/02-facts-tokens-bench-corpus.md` §B の **complex 12.32 ms / complex-x64 958 ms** と、
`docs/reports/2026-09-22-p4-java-perf-m5.md` §旧実装との比較 の **root 1,928 ms / facade 76,720 ms** が
両立しなかった。原因を特定し、今後の比較に使う 1 本の数値を決める。

**結論**: 12.32 ms 側が正しい。1,928 ms は **共有 tinyexpression の `target/classes` が 2026-08-28 の
ビルドのまま**で、生成 P4 parser に memo marker（`SafeFailureMemoizable` / `SafeSuccessMemoizable`）が
1 個も入っていないために `Memoization.SAFE_FAILURES` が完全に空振りしていた値だった。
tinyexpression のフォーク（`AbstractParser` 等）による shadow は原因ではない（寄与 ≤ 11%、しかも逆方向）。

**推奨ベースライン（complex.tiny 332 byte）: 認識 13.6 ms / AST 14.5 ms**（詳細は §4）。

## 1. 実行時にどのクラスが読まれるか

tinyexpression が `src/main/java/org/unlaxer/**` に持つ java ソースのうち、**unlaxer-common の jar と
同名で衝突する（＝shadow する）のは 4 クラスだけ**:
`org.unlaxer.parser.AbstractParser` / `org.unlaxer.parser.Parser` / `org.unlaxer.ParserTaggable` /
`org.unlaxer.ParserFinderToChild`（`org.unlaxer.parser.Parsers` は tinyexpression PR #161 で削除済みだが、
2026-08-28 の `target/classes` には残っている）。`ParentHolderParser` は tinyexpression 固有で jar に無いため
shadow ではない。フォーク版 `AbstractParser.parse` には memo hook が無い（upstream 側は
`PackratMemoTable.lookup/replay/memoizeFailure` を呼ぶ）。

`LegacyBenchmark` が起動時に `Class.getProtectionDomain().getCodeSource()` を印字する（14 クラス）。設定別:

| クラス | (a) tiny-first（tinyexpression 自身の JMH / test 順） | (b) jar-first（従来の `legacy-bench.sh`） |
|---|---|---|
| `AbstractParser` / `Parser` | tinyexpression `target/classes`（**フォーク**） | unlaxer-common jar |
| `ParentHolderParser` | tinyexpression `target/classes` | tinyexpression `target/classes`（jar に無い） |
| `Chain` / `Choice` / `Occurs` | unlaxer-common jar | unlaxer-common jar |
| `ParseContext` / `PackratMemoTable` / `Memoization` / `SafeFailureMemoizable` | unlaxer-common jar | unlaxer-common jar |
| `SafeSuccessMemoizable` | unlaxer-common jar（master ビルド） | 共有 `~/.m2` の 3.0.15 では **ClassNotFound** |
| 生成 `TinyExpressionP4Parsers` / `TinyExpressionP4Mapper` / `P4PreferredAstMapper` | tinyexpression `target/classes` | tinyexpression `target/classes` |

(c) isolated repo 手順（`benchmarks/run-java-parser-benchmark.sh` + `TINYEXPRESSION_MAVEN_REPO_LOCAL`）は
classpath 順が (a) と同じで、jar だけが isolated repo のものに差し替わる。12.32 ms はこの構成で採られている。

marker の有無も印字する（`root=… safeSuccess=… safeFailure=…`）:

| 生成クラスの出所 | `SafeFailureMemoizable` marker | `SafeSuccessMemoizable` marker | root parser |
|---|---:|---:|---|
| 共有 tinyexpression `target/classes`（2026-08-28 ビルド） | **0 個** | **0 個** | safeFailure=false / safeSuccess=false |
| 本作業で `c70416e1` を再ビルドした `target/classes` | 173 個 | 50 個 | safeFailure=true |

「50 class」は unlaxer-parser `docs/performance-tuning-ja.md` ケース31 の記載と一致する。

## 2. 計測

- host: 32 core / 86 GB、共有（別エージェントの計測が並走）。`taskset -c 30` で 1 コアに固定。
- JDK 21.0.9。`--add-opens=java.base/{util,lang}=ALL-UNNAMED`。
- unlaxer: master `ed4178e`（#273 ケース30 + #274 ケース31 込み）を `mktemp -d` 配下の private maven repo へ install。
- tinyexpression: `c70416e1`（12.32 ms の記録と同じ commit）を同 repo でビルド。共有ツリーは読み取りのみ。
- fixture は tinyexpression `benchmarks/fixtures` と `examples/p4-java/src/test/resources/fixtures` が byte 一致（md5 同一）。
- nanoTime harness = `examples/p4-java` の `LegacyBenchmark`（warmup 300、5 run × 300 iteration、中央値と最小値）。
- 経路: `root` = `getRootParser()` + `Memoization.SAFE_FAILURES`（JMH `parseOnlySafe` 相当）、
  `mapper` = `TinyExpressionP4Mapper.parseWithOptions(src, SAFE_FAILURES)`（JMH `parseAndMapSafe` 相当）、
  `facade` = `P4PreferredAstMapper.parseDetailed`（JMH `publicFacade` 相当）。
  **注意**: 引数 1 つの `TinyExpressionP4Mapper.parse(String)` は `ParseOptions.DEFAULT`＝memo **OFF** なので
  AST モードの比較に使えない（0.9 秒級になる）。本作業で `mapper` を memo 有りへ直し、旧挙動は `mapper-off` に分けた。

### complex.tiny（332 byte）

| 設定 | unlaxer jar | 生成 P4 クラス | CP 順 | heap | root ms（中央/最小） | mapper ms | facade ms | 割当/parse |
|---|---|---|---|---|---:|---:|---:|---:|
| **B3（推奨）** | private m2 master | 再ビルド | tiny-first | 2g | **13.64 / 13.21** | **14.55 / 13.90** | 10.77 / 10.60 | 27.4 / 27.8 / 23.7 MB |
| B | private m2 master | 再ビルド | tiny-first | 6g | 13.20 / 12.45 | 14.30 / 13.67 | 10.31 / 10.07 | 同上 |
| C | private m2 master | 再ビルド | jar-first | 6g | 14.64 / 13.29 | 14.50 / 13.85 | 11.66 / 11.47 | 同上 |
| OFF | private m2 master | 再ビルド | tiny-first | 6g | `Memoization.OFF` 983 / 910 | — | — | 2.57 GB |
| E | private m2 master | **2026-08-28 のまま** | jar-first | 6g | 1,575 / 1,475 | — | — | 2.10 GB |
| D（従来の legacy-bench） | 共有 `~/.m2` 3.0.15 | **2026-08-28 のまま** | jar-first | 6g | 985 / 984 | — | 82,526 / 45,841 | 2.17 GB / 93.8 GB |

JMH（tinyexpression 自身の `P4ParserBenchmark`、設定 (a)、1 fork × 5 warmup × 8〜10 iteration）:

| Bench | 本日 | 既存記録（ケース30/31 の表） |
|---|---:|---:|
| `parseOnlySafe` complex.tiny | 14.96 ms / 15.84 ms（2 回） | 15.049 ms |
| `parseAndMapSafe` complex.tiny | 18.7 ms（±8.7、負荷でばらつく） | — |
| `publicFacade` complex.tiny | 24.2 ms（±45.5）/ 30.1 ms（±23.9） | 12.320 ms |

`parseOnlySafe` は既存記録と **0.6%** 差で一致する。`publicFacade` は 1 iteration あたりの op 数が少なく、
本日の負荷（load 15〜45）では誤差が値を超えたので、nanoTime harness の 10.6〜10.8 ms を採る。

### complex-x64（20,923 byte、`-Xmx6g`）

| 経路 | nanoTime（中央/最小、5 run × 10 iteration） | JMH（1 fork × 5 iteration） | 既存記録 |
|---|---:|---:|---:|
| root / `parseOnlySafe` | 1,603 / 1,581 ms | 1,760 ± 332 ms | 973.8 ms |
| mapper | 1,693 / 1,578 ms | — | — |
| facade / `publicFacade` | 1,549 / 1,348 ms | 1,956 ± 1,766 ms | 958.1 ms |

割当は root 1.68 GB / facade 1.45 GB per parse。既存記録の 1.6〜1.8 倍だが、この経路はヒープ設定と
ホスト負荷に強く依存する（既存記録は JVM 既定ヒープ＝最大 21.6 GB、本日は load 15〜45）。
**x64 は「1.0〜1.8 秒の桁」までしか主張しない。**

## 3. 差の内訳

| 要因 | 寄与 | 根拠 |
|---|---:|---|
| **生成クラスが古く memo marker が 0 個** | **×75** | D root 985 ms ≒ `Memoization.OFF` 983 ms（差 0.2%）。B3 の 13.6 ms に対し 72 倍。marker 数 0 vs 173/50 |
| ヒープ（旧計測は `-Xmx2g`、1 parse で 2.17 GB 割当） | ×2.0 | 旧記録 1,928 ms → 同設定を `-Xmx6g` で 985 ms |
| jar の版（#273 / #274 の有無） | ×1 未満（説明にならない） | marker が無い状態では jar を master に替えても D 985 → E 1,575 ms と悪化する |
| classpath 順（フォーク shadow） | ≤ 11%、**逆方向** | B 13.20 vs C 14.64 ms（jar-first の方が遅い）。memo は Chain/Choice/Occurs 側で効くので `AbstractParser` フォークは支配的でない |
| 10 秒 deadline（`tinyexpression.p4.parse.timeout.millis`） | 旧 facade 値の前提 | 全計測で 0（無効）。既定のままだと D の facade（82 秒）は `ParseDeadlineExceededException` で中断する。JMH `publicFacade` は既定 10 秒だが 11〜30 ms なので無関係 |
| JIT warmup | ×1.1 程度 | D の初回 1,415 ms → 定常 985 ms。B の初回 226 ms → 定常 13.6 ms（warmup 300 で飽和） |

つまり **1,928 ms ≒ 75（marker 欠落）× 2（ヒープ）× 13.6 ms**。
facade の 76,720 ms も同じ理由で、候補ごとに memo 無しの再 parse が走った結果（本日の再現値 82,526 ms、
割当 93.8 GB/parse）。memo が効く構成では 10.8 ms（**×7,700**）。

`~/.m2` の `unlaxer-common-3.0.15.jar` は 2026-09-21 21:58 ビルドで `SafeSuccessMemoizable` を含まない
（＝#274 以前）。`markMemoizationStateChanged` が存在することは版の新しさの根拠にならない
（#273 はそのメソッドを**呼ばなくする**変更で、メソッド自体は前からある）。

## 4. 推奨ベースライン

> **旧実装 complex.tiny = 認識 13.6 ms（最小 13.2）/ AST 付き 14.5 ms（最小 13.9）、割当 27.4〜27.8 MB/parse**

計測構成（B3）:
- tinyexpression `c70416e1` を、unlaxer master `ed4178e` を install した isolated maven repo でビルドし直した `target/classes`
- classpath 順は tinyexpression 自身の順（`target/test-classes:target/classes:依存 jar`）
- 認識 = `TinyExpressionP4Parsers.getRootParser()` + `ParseOptions.withMemoization(Memoization.SAFE_FAILURES)`
- AST = `TinyExpressionP4Mapper.parseWithOptions(source, 同 options)`
- JVM `-Xms512m -Xmx2g -Xss16m`（`examples/p4-java/scripts/benchmark.sh` と同じ）、warmup 300、5 run × 300 iteration、`taskset -c 30`、開始時 load 7.3
- 再現: `MAVEN_REPO=<isolated> DEPS_CP=<jmh-classpath.txt の中身> CP_ORDER=tiny-first scripts/legacy-bench.sh <tinyexpression> root|mapper complex`

どのモードと比べるか:
- ubnfc の **認識モード** ↔ `root`（13.6 ms）。ubnfc Java 0.60 ms に対し **約 23 倍**。
- ubnfc の **AST モード** ↔ `mapper`（14.5 ms）。ubnfc Java 0.64 ms に対し **約 23 倍**。割当は 27.8 MB vs 546 KB で **約 51 倍**。
- 副次値: 公開 facade `P4PreferredAstMapper.parseDetailed` = **10.8 ms**（候補選択込み・AST 付き）。
  root より速いのは、facade が `Diagnostics.AUTO` で成功経路の失敗記録を作らないため。
  入口の実務コストを見るときだけ使い、認識 / AST の対比には使わない。
- complex-x64（`-Xmx6g`）: 認識 1,603 ms / AST 1,693 ms。ubnfc Java 37.1 / 44.4 ms に対し **約 43 倍 / 38 倍**。
  ヒープ設定が揃っていないので「40 倍前後」までしか主張しない。

旧 Rust（criterion complex 1.426 ms）は本作業では測り直していない。

## 5. 付随して直したもの

- `scripts/legacy-bench.sh` に `CP_ORDER=jar-first|tiny-first` と `DEPS_CP` を追加。既定の `jar-first` は
  tinyexpression が実際に測っている runtime とは別物になるので、ベースラインを取るときは `tiny-first` を使う。
- `LegacyBenchmark`: 主要 14 クラスの `CodeSource` と root parser の memo marker を印字。
  `mapper` を `parseWithOptions(..., SAFE_FAILURES)` に変更（旧＝memo OFF は `mapper-off`）。
  `root-off`（`Memoization.OFF`）を追加。
- 共有 `~/.m2` と共有 tinyexpression `target/classes` は変更していない（読み取りのみ）。
  **共有ツリーの `target/classes` は 2026-08-28 のままなので、そこを指した計測は今後も無効。**
