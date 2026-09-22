# P4 の独立 Java 実行例

P4 IR から生成した Java 22 ファイルと、依存ゼロの Extern scanner 3種を収録する。
JDK 21、Maven を使用し、依存は JUnit 5.10.2 の test scope のみ。
生成器・tinyexpression・unlaxer のソースは変更していない。

**既知の不足を含む検証例であり、P4 の完全互換を達成したものではない。**
16 fixture 中、Formula 入口で9件を全消費、意図的な不正入力3件を拒否。
comparison-heavy 系4件は Formula 入口が拒否する。BooleanExpression 入口での成功と
AST golden は別に保存した。文字列の引用符除去と旧生成物の Unicode span にも差がある。
テストの緑はこれらを含む観測結果の固定を意味する。
詳細は [計測報告](../../docs/reports/2026-09-22-p4-java-bench.md) を参照。

## 再現

```bash
cd examples/p4-java
mvn -o -q test
mvn -o -q -DskipTests package
bash scripts/benchmark.sh
bash scripts/depth.sh
```

fixture は tinyexpression の指定ディレクトリから byte を変更せず収録した。
出典と SHA-256 は `evidence/provenance.json`。テストは外部の tinyexpression を必要としない。
`target/` は commit しない。

生成 Java は `src/main/java/org/ubnfc/p4/generated/` に commit する。
再生成は次のスクリプトで行い、テストでも全ファイルの byte 一致を確認する。

```bash
bash scripts/generate.sh --check
bash scripts/generate.sh --write
```

初回生成には元 checkout の master jar を使用した。作業中にその jar が更新されたため、
通常の再生成テストは **同じ checkout の ubnfc-java ソース**を `mktemp -d` 内へコピーして
オフラインでビルドする。同じ22ファイルを byte 一致で生成できることを確認済み。
リポジトリの生成器ソース・target は変更しない。
特定の jar を使う場合は `UBNFC_GENERATOR_JAR=/absolute/path/generator.jar` を指定する。
生成器ソースの版が変わって byte 不一致が出た場合、テストを無効化せず差分を確認する。

## API

```java
var result = TinyExpressionP4Parser.parse(source, P4Scanners.options(true));
// true: AST 付き、false: 認識のみ。両方とも全消費を要求する。
if (result.ok()) {
    System.out.println(Json.canonical(result));
}
```

`P4Scanners.ALL` のキーは `TinyExpressionP4::STRING` / `CODE_START` / `CODE_END`。
scanner は `Input` と `ScanState` だけに依存し、可変な外部状態や旧 ParseContext を持たない。
純粋性と成功・失敗 memo の宣言は、入力 IR の各 token の `memoSafety` に既にある。
`valueSpan` は引用符も含む token 全体。引用符の除去は旧 Mapper の意味変換であり、
今回の scanner では入力文字列を改変しない。

## 計測条件

`Benchmark` は同一プロセスで、入力・モードごとに2,000回の warmup と5 run を実行する。
1 run の回数は complex / comparison-heavy が2,000、x4 が500、x16 が125、x64 が50。
x4/x16 は処理入力総量を概ね揃える設定。結果を volatile に保存し、I/O は計測外に置く。
受理・全消費・AST の存在を事前に確認し、拒否入力を速度として記録しない。
**x64 にも2,000回の warmup があるため、実行には長い時間がかかる。**

`DepthProbe` の既定は括弧深度8/16/32/64/128。`scripts/depth.sh 256` で深度を指定できる。
各 JVM は60秒を上限とし、時間切れは成功と扱わない。

## 旧生成物との比較

ビルド依存に旧実装を追加せず、任意ツール `LegacyProbe` が反射で読み込む。
次のスクリプトは tinyexpression を読み取り専用で使い、unlaxer jar を先に置いて
コアクラスの shadow を避ける。45秒の上限は起動した比較プロセスだけに適用する。

```bash
bash scripts/legacy.sh /path/to/tinyexpression mapper src/test/resources/fixtures/valid-basic.tiny /path/to/result.json
bash scripts/legacy.sh /path/to/tinyexpression facade src/test/resources/fixtures/complex.tiny
python3 scripts/compare-ast.py /path/to/result.json src/test/resources/golden/valid-basic.tiny.json
```

`MAVEN_REPO` と `UNLAXER_VERSION` で jar の探索先を変更できる。既定版は3.0.15。
`mapper` は旧 Formula root の直接比較、`facade` は memo ON の公開入口。
旧 snapshot は `src/test/resources/legacy/`、機械可読の差分は `evidence/*differences.json`。
span の単位や引用符を正規化して差分を消す処理は行わない。
