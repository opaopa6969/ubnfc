# 手書き Java 生成物ターゲット

UBNF バックエンドが出力するコードの目標形を、Operators / Power の2文法で固定する。
Java 21、独立 Maven モジュール `org.ubnfc:ubnfc-target-java:0.1.0-SNAPSHOT`。
本体は JDK のみ、依存宣言は test scope の JUnit Jupiter 5.10.2 のみ。
`jdeps -recursive -s target/classes` でも依存先が `java.base` のみであることを確認した。
旧 parser のコードを実装へ転用せず、文法・仕様・テストを挙動の参照に使った。

```sh
cd examples/handwritten-target/java
mvn -o -q test
mvn -o -q test -Dbench=true
```

通常テストは **112件、失敗0・skip 0**（corpus 52 + 意味論42 + 全 rule memo 13 + session 5）。
`Bench.java` は通常の Surefire 検出対象外で、`bench=true` profile と条件 annotation の
両方で明示実行する。profile 実行時はベンチ1件だけを選ぶ。

## 入力資料と今回の契約差

設計はリポジトリの `docs/design/03-design-v1.md` §3・§5・§6、token 範囲は
`docs/design/02-facts-tokens-bench-corpus.md` §A。
corpus と UBNF は `unlaxer-parser` の
`ed4178e673f93e719b62a7bd181f3a219abf8ed1` から無変更でコピーした。
同リポジトリの `docs/rule-trivia.md`、`unlaxer-dsl/specs/generators.md`、
`ParserGenerator.java:495`、`ParserRuleEmitter`、associative 系 runtime tests と
`golden/ast_snapshot.java.txt` を確認した。参照リポジトリの変更は不要。

依頼文の「既存生成物と同じ」と「`(NUMBER) @value` は int」には差がある。
現ソースの `AssociativeMappingRuntimeTest` は、この **group capture を String として扱う**。
`specs/generators.md` の int 契約は NUMBER の **直接 capture** に対するもの。
ここでは明示指定の `Number(int value)` を優先した。group の capture span はそのまま保持し、
数値変換レシピには **NUMBER token 自身の範囲**を別に保持する。
したがって末尾 trivia を含む group 全文へ `parseInt` はかけない。
この型と変換範囲の選択は既存の group→String mapping との差であり、互換確認済みとは扱わない。
生成器に組み込む前に、front の capture 型・変換範囲の契約として確定させる必要がある。

| corpus | 件数 | 評価値一致 | 構文成功・mapping 失敗 | 元文法で拒否 |
|---|---:|---:|---:|---:|
| Operators | 24 | 13 | 1 | 10 |
| Power | 28 | 11 | 0 | 17 |

- Operators の `1e2/4-0.5` は NUMBER として全消費するが、`Integer.parseInt("1e2")` が失敗する。
  corpus の 24.5 を int へ丸めず、`MappingException` を期待する Java oracle として区別する。
- Power は `Atom ::= NUMBER` という非 mapping alias を capture するため、既存どおり
  `Power(String left, List<String> op, List<Power> right)`。`4^0.5` は評価値2になる。
- Power の `modes:["grouped"]` 3件と `modes:["unicode"]` 2件は、上流の
  `RustConformanceTest` が文法を書き換えて使うケース。元の `Power.ubnf` にはその規則がないため、
  **5件ともテストを実行して拒否を確認**する。文法を暗黙に拡張せず、value 比較対象には数えない。
- 演算は double。左結合は出現順の左 fold、右結合は `Math.pow(left, evaluate(right[0]))`。
  Power の alias 文字列は末尾コメントを保持する。評価器は corpus の `probe.rs.txt` と同様、
  最初の `/` より前の数値を読む。AST や capture からコメントを削除するわけではない。

## API とソース構成

```java
var result = OperatorsParser.parse("1+2*3");
assert result.ok();
assert result.consumedCp() == 5 && result.matchedCp() == 5;
double value = Evaluator.evaluate(result.ast().orElseThrow()); // 7.0
String json = Json.canonical(result); // {"type":"Binary","span":[0,5],"fields":{...}}
```

- `src/main/java/target/operators/{OperatorsParser,OperatorsAST}.java`:
  `Binary(OperatorsAST left, List<String> op, List<OperatorsAST> right)` と `Number(int value)`。
  Expression と Term が同じ Binary 型を共有し、優先順位段や括弧内の node を省かない。
- `src/main/java/target/power/{PowerParser,PowerAST}.java`: 右再帰の直接出力形。
- `src/main/java/target/api/`: `ParseOptions`、`ParseResult`、`Span`、`Capture`、
  `Diagnostic`、`MappingException`、`Evaluator`、`Json`。
- `src/main/java/target/internal/`: `Session`、`Recipe`、`Events`、`RepeatProgress`。
  共通のスキャナ・memo・構築命令。汎用 parser VM や parser combinator は使わない。
- `src/main/java/target/rt/`: `Input`、`Cursor`、`ScopeStore`、`Severity`、`MemoTable`、
  `Diagnostics` のソースコピー。元の `ubnfc-java/runtime-template`（基点 `54519c6`）から
  `package ubnfc.rt` を `package target.rt` に変えただけ。runtime jar には依存しない。
- `src/test/java/target/`: corpus、追加意味論、全 rule memo、session の検証、最小 JSON reader、
  golden 候補 writer、独立ベンチ。`src/test/resources/{corpus,golden}/` に入力と固定期待値。

`parse(source)` は全入力を要求する。`ParseOptions.PREFIX` は root の接頭辞成功を返す。
構文失敗・後続入力残りでは `ok=false`、AST/capture/nodeSpans は空。
後続入力残りの場合は root が採用した接頭辞の consumed/matched を保持する。
`ParseOptions.RECOGNIZE` は全入力認識・memo・診断を行い、AST/capture レシピを作らない。
`MappingException.syntax()` は `ok=true` の認識結果と採用 capture、`span()` は変換失敗した
NUMBER の code point 範囲、cause は `NumberFormatException`。

span は半開区間の code point、内部カーソルは UTF-16。`Input` の char[] を通して文字を読む。
`nodeSpans` は immutable な **identity map**。同値な `Number(1)` の別出現を潰さない。
capture は安定した siteId・名前・span の完了順リスト。
この2文法には scope/lexical 出力要求はないため、ScopeStore は埋め込みコピーだけで未使用。
M1 全機能（scope / longest / recovery 等）の縦断完了をこの2文法から主張しない。

## バックエンド生成器へ渡す知見

### trivia の挿入計画

有効 policy は rule whitespace > rule interleave > grammar。今回の全 rule は javaStyle。
`SPACE, child0, SPACE, child1, SPACE, ...` を **chain** に生成する。
choice 自体や全 token の直前へ一律に skip を置いてはいけない。

| 境界 | Operators / Power での具体例 |
|---|---|
| rule chain 先頭・各要素後 | Expression、Term、NumericLiteral、Atom |
| group helper 先頭・各要素後 | `(NUMBER)`。value span は末尾空白・コメントを含む |
| 反復 helper 先頭・各要素後 | `AddOp Term` / `MulOp Factor`。op 後と operand 後にも入る |
| 複数要素 choice 代替の chain | `'(' Expression ')'`、右結合の `Atom '^' Expr` |
| choice 自体は挿入なし | Factor、AddOp、MulOp、右結合の Expr |
| 単一要素代替は子へ委譲 | Factor の NumericLiteral、Expr の fallback Atom |

javaStyle の空白は ASCII の space / TAB / LF / VT / FF / CR の6文字。
`//` は CR / LF / CRLF / EOF まで、block は最初の `*/` までで非入れ子。
未閉鎖 block は skip をロールバックする。NUMBER 内部には trivia を入れない。
` 1 + 2*3 ` の root span は `[0,9]`、左 Term と Number は `[1,3]`、
積 Term は `[5,9]`、その左 Number は `[5,6]`、右 Number は `[7,9]`。
括弧を透過しても内側 Expression の span を括弧まで広げない。

### 結合性と構築レシピ

左結合は `left` 1個と当該段の `op[]` / `right[]` を構築する。operand の内部へ探索しない。
右結合は `Base Op Self | Base` の宣言順 PEG。失敗時は開始状態へ戻して Base を試す。
再帰成功レシピは `Power(left, [op], [childRecipe])`、基底は `Power(left, [], [])`。
採用後に子から record を作り、再 fold しない。各段の right は 0 または 1要素。
右側再帰の capture が完了してから外側 `right` capture を足す。

`Recipe` は型・span・変換範囲・operand レシピとリストだけを持つ小オブジェクト。
parser の各 rule/terminal を並べた CST は作らない。`Events` は capture 列の永続 concat で、
失敗枝では参照だけを捨てる。memo が参照するレシピ領域を truncate/reuse しない。
全入力または明示的な prefix の採用確定後、`build()` だけが公開 record を作る。
`Integer.parseInt` も `build()` 内。`1+(2147483648` の放棄枝は変換しない一方、
採用された `1+2147483648` は MappingException となる。

### テンプレート化できる反復・memo・診断

各 rule は `static int parseX(int consumed, int matched)`。0=成功、-1=失敗、
出力カーソル・レシピ・capture 列は parse 専用 Session の欄。制御フローに例外を使わない。
ThreadLocal は2引数シグネチャの session 受け渡し用で、終了時に必ず元へ戻す。
異なる thread の解析結果や以前の span 表は共有しない。

反復テンプレートは「終端があれば照合 → 子が失敗/停止なら止める → count++ →
consumed の進捗検査 → max 検査 → 終了後 min/max 判定」。ゼロ幅成功の周回も数えるので
`EMPTY{2,2}` は失敗、`EMPTY{1,2}` は1回で成功する。今回の左右反復は min=0/max=MAX。
`RepeatProgress` で停止条件を共有し、EMPTY の境界を session test で固定する。
右結合をこの flat repeat テンプレートへ置換してはいけない。

宣言 rule と生成 helper の全12関数で成功・失敗 memo を使う。
キーは `(rule/ExprId, consumed, matched, mode, stateVersion=0)`。
mode は認識のみ0 / AST付き1。`MemoTable` は **ExprId 24bit、mode 8bit** の制約を持つ。
payload handle はレシピ・capture 列・局所診断の組を指す。
Power の Base fallback は、実際に Atom の success/failure memo を再訪する。

診断は失敗した terminal の consumed 位置を基本とし、NUMBER 内部の不足桁や未閉鎖 comment は
その走査到達位置を記録する。`1+` は接頭辞 consumed=1、farthest=2、
`1e+` は consumed=1、farthest=3。この farthest を EOF エラーの consumed 位置で上書きしない。
expected は宣言順で重複除去、同位置は最深 stack、同深さは先着。
生成 helper 名も stack に残す（例 `NumericLiteral.group`）。成功 rule 内の失敗も保持する。
memo hit は保存 rule を enter する前に局所診断を merge し、呼出元 stack を付け替える。
診断表示は文法 terminal（NUMBER、リテラル、EOF、`*/`）を単位に固定している。
上流 runtime 内部の全 primitive hint と同一という主張ではなく、設計 D-011 の最終 oracle は別途必要。

この参照経路は全 memo・スカラー走査・直接分岐。FIRST 予測、memo 省略、climbing、
SIMD、trivia cache は未適用。将来の最適化はこの canonical AST/capture/span を保つ必要がある。
右再帰と括弧再帰は Java call stack を使うため、任意の深さに対する stack 耐性は保証しない。

## golden の固定と確認

`golden/operators.json` と `golden/power.json` は全52入力の ok、両カーソル、mapping 成否、
canonical AST、capture 列、診断を固定する。AST は `{type,span:[s,e],fields:{...}}`、
field 順は `left,op,right` または `value`。未受理・mapping 失敗時は AST null。
文字列 writer は quote、backslash、制御文字、surrogate を escape する。

52行の成否・カーソル・診断、および全24 AST の型・構造・全 node span を展開して目視確認した。
特に左 fold、右再帰の `[0,5]→[2,5]→[4,5]`、空の op/right、括弧内の node、
絵文字コメントの code point 範囲、同名 capture の内側→外側順を確認し固定した。
主要 span と計算値は golden と別の手書き assertion でも検証する。

golden は通常テストで自動更新しない。変更候補を作る場合だけ、モジュールのディレクトリで:

```sh
mvn -o -q test-compile
golden_tmp=$(mktemp -d)
java -cp target/classes:target/test-classes target.GoldenWriter "$golden_tmp"
diff -u src/test/resources/golden/operators.json "$golden_tmp/operators.json"
diff -u src/test/resources/golden/power.json "$golden_tmp/power.json"
```

## ベンチ

2026-09-22、Oracle JDK 21.0.9 / HotSpot 64-bit、Linux amd64
(WSL2 `6.18.33.2-microsoft-standard-wsl2`)、AMD Ryzen 9 7950X (32 logical CPU)。
`mvn -o -q test -Dbench=true`、GC・heap・CPU affinity の追加指定なし。
各入力・各モードで warmup **20,000回**、**200,000回 × 5 run** の ns/op の中央値。
毎回 Input/session/memo を作り、volatile sink に ParseResult を保存する。
認識のみも memo と診断を含む。AST付きは capture 列・identity span 表の実体化も含む。
JSON 出力と評価器は計測範囲外。単一 JVM の連続 run であり、独立 fork の統計ではない。

| 入力 | 文字数 | 認識のみ (µs/op) | AST付き (µs/op) |
|---|---:|---:|---:|
| `1+2*3` | 5 | 2.129 | 2.288 |
| 括弧8段 + `1+2*3` | 21 | 14.163 | 15.811 |
| `1+` × 511 + `1`（512 atom） | 1023 | 408.711 | 572.780 |

全5 run の ns/op（実行順）:

```text
1+2*3      recognize 3885.9, 2129.3, 1388.3, 3021.6, 1397.7
1+2*3      ast       2545.7, 2287.7, 2281.4, 2257.8, 2299.2
depth-8    recognize 13543.1, 14163.1, 13529.5, 14866.1, 15045.2
depth-8    ast       15661.0, 15957.1, 15811.4, 16038.8, 15676.4
512-atoms  recognize 398472.8, 409755.9, 404334.1, 408711.0, 414482.1
512-atoms  ast       599376.0, 592175.3, 572780.3, 570449.9, 572232.4
```

短い入力の最初の数 run は JIT/GC 等の揺れがあるため、中央値と全 run を併記する。
既存 parser と同一条件の比較計測はしておらず、速度比や性能目標達成は主張しない。
EOF まで全消費したことをベンチ各周回で確認する。
