# P4 Java 生成・互換性・性能の検証（2026-09-22）

## 結論

P4 IR（124 rules）から Java の生成・コンパイル・実行が可能だった。
生成拒否された機能はない。独立モジュール `examples/p4-java` に生成物22ファイル、
純粋な Extern scanner 3種、fixture、canonical AST golden、検証・計測ツールを収録した。

**互換性は未達。** Formula 入口は comparison-heavy 系4件を拒否する。
文字列値の引用符除去にも差があり、旧生成物の Unicode span には単位の混在がある。
テストは成功例と既知の不足の両方を固定しており、緑を全互換の根拠にしていない。
性能も互換性と分離して報告する。設計 v1 §1-5 は数値閾値を未固定としているため、
任意の合格閾値を追加しない。

## 使用物と再現

- 作業ブランチ: `feat/p4-java-e2e`。ubnfc 基点は `d86f3a3`。
- 入力: `ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json`。
- 生成器: 元 checkout の `ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar`。
  SHA-256: `1d0e408ff569d4f8a579ef079b6279d380c881acc87e3260c9e38689b697e010`。
- 生成 package: `org.ubnfc.p4.generated`。生成ソースには手を加えていない。
- Java: HotSpot 21.0.9、CPU: AMD Ryzen 9 7950X。単一の計測スレッド。
- 実行時依存: JDK のみ。Maven 依存: JUnit 5.10.2（test scope）のみ。
  `jdeps -s` で jar の依存が java.base / java.management のみであることも確認した。
- tinyexpression / unlaxer-parser は読み取りのみ。ubnfc-java / crates / ir / corpus も無変更。

入力・旧 class ファイル・関連 repo の revision と SHA-256 は
[`provenance.json`](../../examples/p4-java/evidence/provenance.json) に記録した。
旧 `target/classes` は現在のソースとは API が異なり、現在の JMH ソースの
`MappedAst` / `ParseOptions` 入口をそのまま呼べる版ではない。

```bash
cd examples/p4-java
mvn -o -q test
mvn -o -q -DskipTests package
bash scripts/generate.sh --check
bash scripts/benchmark.sh
bash scripts/depth.sh
```

`generate.sh` は同じ checkout の生成器ソースを `mktemp -d` 内にコピーしてオフラインでビルドし、
元の IR から再生成した22ファイルの集合と byte を比較する。生成器ソース自体への変更はない。
初回生成後に共有 master jar が更新され、7ファイルの出力が変化したため、
開始時のソース版（d86f3a3）から byte 一致を確認できる方法へ切り替えた。
この報告の数値は開始時の生成物に対するもので、更新後の master の性能ではない。
`UBNFC_GENERATOR_JAR` を指定すれば、明示した jar でも再生成できる。
fixture は指定された2ディレクトリから内容を変更せず収録した。

## Extern scanner

以下の相対パスは `tinyexpression/src/main/java/org/unlaxer/tinyexpression/`、
`unlaxer-common` は `unlaxer-parser/unlaxer-common/src/main/java/org/unlaxer/` を起点とする。

| token | 移植した受理範囲 | 根拠 |
|---|---|---|
| STRING | 単引用符または二重引用符で囲む。空・改行・補助平面文字も許す。バックスラッシュ＋任意の1 code point を優先し、復号しない。閉じ引用符が必須 | `parser/StringLiteralParser.java:19-25`。unlaxer-common `parser/elementary/QuotedParser.java:55-71`、`EscapeInQuotedParser.java:22-26`、`WildCardStringParser.java:18-20` |
| CODE_START | 行頭の ` ``` `、ASCII 識別子、`:`、ASCII 識別子のドット区切り、行末。空白を暗黙挿入しない | `parser/javalang/CodeStartParser.java:32-40`、`TripleBackTickParser.java:7-8`、`parser/javatype/JavaClassNameParser.java:28-39` |
| CODE_END | 行頭の ` ``` ` と行末 | `parser/javalang/CodeEndParser.java:21-26` |

識別子は `[A-Za-z_][A-Za-z0-9_]*`。`$` や日本語は認めない。
根拠は unlaxer-common `parser/clang/IdentifierParser.java:24-31` と
`parser/posix/AlphabetUnderScoreParser.java:14-15` / `AlphabetNumericUnderScoreParser.java:10-12`。
行頭は offset 0 または直前が CR/LF（`StartOfLineParser.java:40-53`）、
行末は CRLF / CR / LF / EOF（`LineTerminatorParser.java:20-26`）。改行は消費し、EOF はゼロ幅。

scanner の入出力は UTF-16 境界。公開 AST span は生成 runtime により code point へ変換する。
consumed / matched / mode / invert / reset を受け取り、成功時のカーソル・valueSpan を返す。
反転は token 全体の否定ではなく旧 parser の子への伝播として実装した。
状態は呼び出し内に閉じ、effects は空。IR の3 token は既に `purity=pure`、
`success=true`、`failure=true` と宣言されている。IR や生成器への追加変更は不要だった。

`ScannerTest` は空文字列、escape、改行、Unicode、不完全な引用、接頭辞一致、行頭、
CR/LF/CRLF/EOF、識別子境界、matchOnly、reset、反転、実際の CodeBlock を検証する。
scanner の失敗診断は token 名と開始位置を返す簡略版で、旧 parser の内部 expected 集合や
最遠失敗位置の完全再現は未検証。5状態の全組合せを旧実装 oracle で網羅比較したものでもない。
受理範囲の互換と診断・任意の呼び出し状態の完全互換を混同しない。

## fixture と canonical AST

| fixture | Formula 入口 | 全消費した code point 数 |
|---|---|---:|
| complex | 受理 | 325 |
| complex-x4 | 受理 | 1,293 |
| complex-x16 | 受理 | 5,179 |
| complex-x64 | 受理 | 20,923 |
| flat-arithmetic | 受理 | 1,426 |
| large-match | 受理 | 1,380 |
| valid-basic / valid-multiline / valid-unicode | 受理 | 6 / 40 / 9 |
| comparison-heavy / x4 / x16 / x64 | **拒否、offset 5** | — |
| complex-half / complex-tail | 拒否 | — |
| invalid-syntax | 拒否 | — |

認識のみ・AST 付きの両モードを確認した。成功9件は consumed=matched=入力長。
complex-half / complex-tail は既存 `P4ParserBenchmark.java` の EntryState コメントでも
意図的な不正入力とされているため、`*.tiny` のすべてを正常入力とは扱わない。

comparison-heavy は `1+2*3>=...` の `>=` の手前で停止する。最小例 `1>=1` も同様。
文法 `tinyexpression-p4.ubnf:651-658` の `Expression` は NumberExpression を先に試すため、
先頭の数式だけで選択が確定し、Formula の EOF に達しない。旧 Mapper の Formula 入口も
comparison-heavy を拒否した。これは Extern scanner が呼ばれる前の選択問題である。
`BooleanExpression` 入口なら4件とも全消費（179 / 716 / 2,864 / 11,456 code points）。
その AST は `golden/boolean-entry/` に別保存した。root の成功に読み替えていない。

canonical JSON は node の型・record field・span を保持する。認識成功9件の golden に加え、
complex の構造を手で確認してテスト化した。変数宣言は price(number) と country(string) の2件、
IfExpr 1件、then/else の BranchExpressionExpr 2件、NumberMatchExpr 1件、
case 2件＋default 1件、bonus の NumberMethodDeclarationExpr 1件。
数値葉の `BinaryExpr(left=null, op=[数値文字列], right=[])` は旧 Mapper の出力とも一致した。
拡大 fixture も、宣言数8/32/128、if 数4/16/64、match 数4/16/64、
メソッド数4/16/64と、それぞれの反復数に一致することを確認した。

## 既存 P4 との AST 比較

jar を後ろに置くと AbstractParser は tinyexpression の `target/classes` から読み込まれる。
unlaxer-common / unlaxer-dsl 3.0.15 jar を先に置く経路も試し、読み込み元を記録した。
旧 repo に新しい生成・ビルドは行わず、既存 `target/classes` を使用した。

| 入力 | 比較対象 | 生の canonical JSON の差分 |
|---|---|---|
| valid-basic / valid-multiline / flat-arithmetic / large-match | 旧 Mapper の Formula | 0。型・フィールド・span 一致 |
| complex | 同上 | 4。文字列値3か所、root span 末尾1か所 |
| complex-x4 / complex-x16 | 同上 | 12 / 48。すべて文字列値の引用符 |
| valid-unicode | 同上 | 4。文字列値1か所、span 末尾3か所 |
| complex-x64 | 同上 | **未比較**。45秒の上限で終了、AST 出力なし |
| comparison-heavy | 旧 facade と生成 BooleanExpression 入口 | 0。型・フィールド・span 一致。Formula 同士の比較ではない |
| comparison-heavy-x4 / x16 / x64 | 旧 AST | **未比較** |

旧値 `jp` / `us` / `こんにちは😀` に対し生成側は引用符を含む。
旧 Mapper が `stripQuotes` を適用する一方、今回の IR/runtime は text を字句のまま返す。
`valueSpan` だけを狭めても rule 境界の text 値変換の契約にはならないため、
scanner 側で AST の形を後処理する回避策は加えていない。

旧 span は `tokenStartOffsetCompat` が code point の開始位置を取得し、
`consumedLengthCompat` が `String.length()` を足す
（既存生成物 `TinyExpressionP4Mapper.java:3337-3364`）。
このため valid-unicode の末尾は旧10／生成9、complex の root 末尾は旧326／生成325となる。
差分を消すための単位補正はしていない。両方の snapshot と path 単位の差分を保存した。

## ベンチマーク

JMH は使わず `System.nanoTime()`。JVM は `-Xms512m -Xmx2g -Xss16m`。
同じ JVM 内で入力ごとに認識のみ→AST 付きの順に実行し、**各入力・各モードで warmup 2,000回**。
計測は5 run の中央値、単位は ms/parse。1 run の回数は complex=2,000、x4=500、
x16=125、x64=50。x4/x16 は入力総量を概ね揃えるための選択。
I/O と canonical JSON 化は計測外、AST 構築と result の生成は AST モードの計測内。
結果は volatile へ保持する。認識モードも runtime の capture/recipe 等を返すコストを含む。

共用ホストで CPU 固定や他プロセスの停止は行っていない。調査・検証用の別 JVM も一部重なった。
run 間の分散があり、JIT/GC の影響を分離していない探索的な計測であり、差が小さいときの速度順位や
「AST 構築が認識より速い」といった解釈はしない。5 run の生値も保存する。

<!-- BENCHMARK_TABLE -->
計測中。完了後にここへ実測表とスケーリングを記録する。
<!-- END_BENCHMARK_TABLE -->

旧 memo ON の公開 facade は complex で10秒の deadline に達した。
通常の classpath と jar 優先の両方で再現した。
Mapper の直接呼び出しは一部動いたが、公開 facade と同一条件のベンチには読み替えない。
comparison-heavy の旧 facade 単発実行は成功したが、生成側 Formula が拒否するため、
同じ入口での性能比較は成立しない。

従って旧 Java の欄は `docs/design/02-facts-tokens-bench-corpus.md` §B の記録
（complex 12.32 ms / complex-x64 958 ms）を参考として併記する。
この2値は AST 付き facade、別 revision・別ハーネスであり、認識のみの基準値ではない。
旧 Rust complex 1.426 ms も別 revision の criterion 記録であり、参考値に限る。
comparison-heavy / x64 の拒否処理時間は正常入力の性能値として出さない。

## スタック依存と深い入力

`'(' * depth + '1' + ')' * depth` を独立 JVM で試した。認識・AST 付きの結果は同じだった。

| -Xss | 深さ8 | 16 | 32 | 64 | 128 |
|---|---|---|---|---|---|
| 256k | 拒否 | 拒否 | 拒否 | 拒否 | 拒否 |
| 1m | 受理 | 受理 | 拒否 | 拒否 | 拒否 |
| 4m | 受理 | 受理 | 受理 | 受理 | 受理 |
| 16m | 受理 | 受理 | 受理 | 受理 | 受理 |

拒否は `maximum parse depth exceeded (JVM stack limit; maxDepth=10000)` の診断となり、
例外を成功扱いにしない。16m の深さ256試行は長時間化し、当該 JVM の開始から約5分後に
自分の調査プロセスのみ停止した。深さ256での成功は確認できていない。
再現用スクリプトは JVM ごとに60秒の上限を設け、既定深度は128までとした。
これは fixture の横方向のスケーリングとは別の問題である。

## 契約不足と提案

1. **Formula と facade の入口選択を契約化する。** comparison-heavy は直接の Formula では旧・新とも拒否し、旧 facade が別入口で受け入れる。生成文法の通常 choice を勝手に longest に変えると他の認識が変わる。文法とは別の facade で選択規則を明示するか、front で互換性を検証した選択計画を持たせる。
2. **文字列の字句 span と AST 値変換を分離する。** 旧 `stripQuotes` に相当する変換を IR に明示し、StringTerm 経由の mixed/text 変換にも伝える。今回の scanner は字句全体を返す契約を維持した。
3. **旧 Java span の単位混在を明示する。** 現在の生成側 code point 契約と旧 artifact の混在した span は同じ値にならない。oracle の revision と単位を固定し、補正を導入するなら明示した別契約とする。
4. **scanner 診断の詳細を定義する。** 現在は token 名・開始位置の簡略診断。受理だけでなく expected 集合や最遠位置まで比較する追加 corpus が必要。
5. **深い入力の資源上限と反復化を検討する。** `maxDepth=10000` は JVM stack の余裕を保証しない。大きな -Xss を互換性の証明にはせず、深度・時間・割り当て量を分けて検証する。
6. **性能改善は未達部分として継続する。** compatibility 差分を保ったまま速さだけで完了とはしない。capture/recipe/診断・memo・scope の費用を次の profiling 候補とする。`buildAst=false` でも `Session.rule` の Recipe 作成と capture 保持は走る（生成 `internal/Session.java:141-159,234-241`）。認識専用の結果契約で何を省略できるかを先に定める。この作業では生成器を変更しない。

## 検証記録

独立モジュールで `mvn -o -q test` を実行し、**27 tests / 0 failures / 0 errors / 0 skipped** を確認した。
生成再現性・scanner 境界・fixture・
AST golden・旧 snapshot・memo ON/OFF の一致を検証する。
生ログと JSON は `examples/p4-java/evidence/` と `src/test/resources/` に収録した。
push / PR / merge は依頼に従い実行しない。
