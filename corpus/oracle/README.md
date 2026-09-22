# Java oracle 採取 corpus

`unlaxer-parser` の実装を実際に生成・コンパイル・実行した観測値を固定したもの。
UBNFC の生成器・runtime・期待値は採取に使用していない。互換性の参照データであり、
疑わしい値を望ましい挙動へ修正していない。

- 採取元: `/home/opa/work/unlaxer-workspace/unlaxer-parser`
- revision: **`ed4178e673f93e719b62a7bd181f3a219abf8ed1`**
- 採取日: 2026-09-22（Asia/Tokyo）
- 環境: Linux amd64、Oracle JDK **21.0.9+7-LTS-338**、Maven **3.9.9**、Python **3.14.7**。
- 依存: `unlaxer-common/target/classes`、`unlaxer-dsl/target/classes`、
  ローカル Maven repository の `vavr-0.10.4.jar`、`gson-2.11.0.jar`。
- 開始・終了時に参照リポジトリの `git status --porcelain` が空で、revision が同じことを確認。
  Maven の offline reactor compile は common 386 / dsl 71 ソースを再コンパイルして成功した。
- §4 の19機能と追加11機能群、**63文法 variant / 168ケース**。
  **153ケースを解析実行**（うち1ケースは実行時例外）、14ケースは生成 Java のコンパイル失敗、
  1ケースは循環 import の解決失敗。未実行のケースを拒否として数えない。

## 再採取と検証

このディレクトリの `tools/` のみで再現できる。参照リポジトリのテストクラスへの依存はない。
JDK 21 と上記 Maven 依存が必要。ネットワーク取得はしない。

```bash
cd /home/opa/work/unlaxer-workspace/ubnfc-wt-oracle
corpus/oracle/tools/collect.sh /home/opa/work/unlaxer-workspace/unlaxer-parser
python3 corpus/oracle/tools/verify.py
```

`collect.sh` は参照リポジトリが dirty なら停止し、次のビルドを実行する。
`flatten.skip` により通常の Maven ビルドが作る `target/` 外の `.flattened-pom.xml` を抑止する。

```bash
mvn -o -pl unlaxer-common,unlaxer-dsl -DskipTests -Dflatten.skip=true compile
```

採取結果を上書きせず、別ディレクトリで完全再採取して比較する手順:

```bash
oracle_check_dir=$(mktemp -d -t ubnfc-oracle-check.XXXXXXXX)
trap 'rm -rf -- "$oracle_check_dir"' EXIT
corpus/oracle/tools/collect.sh /home/opa/work/unlaxer-workspace/unlaxer-parser "$oracle_check_dir/result"
python3 corpus/oracle/tools/verify.py "$oracle_check_dir/result"
```

採取時にこの再実行比較を行い、全63 JSON の byte 単位の一致を確認した。
`verify.py` は入力一覧との対応、19機能と追加項目の網羅、機能群ごとの3〜10件、
canonical AST、capture の code point span と入力原文の一致、段階ごとの値の有無も検証する。
数値や受理結果の期待値をこの検証スクリプトから生成することはない。

### ドライバの実行経路

1. `specifications.py` が inline grammar、入力、必要な import 原文を列挙する。
2. `OracleCollector.java` は `CodegenMain.run`（`main` が呼ぶ同じ CLI 実装）を
   `--grammar ... --output ... --generators parser,ast,mapper` で実行する。
   当該 revision は小文字名を拒否するため、そのエラーを残して
   **`Parser,AST,Mapper`** で再試行する。
3. import の場合、CLI は `UBNFMapper.parse` しか呼ばないため、別途既存の
   `UBNFMapper.parseWithImports` と `ParserGenerator` / `ASTGenerator` /
   `MapperGenerator` を実行する。解決後の rule 名・body と token 名も記録する。
4. `javax.tools.JavaCompiler` で生成された parser・AST・mapper をまとめてコンパイルし、
   variant ごとの `URLClassLoader` からロードする。生成 Java を修正しない。
5. fresh `ParseContext` で生成 root parser を実行し、cursor・capture・scope を採取する。
   別の fresh context を内部で使用する生成 `Mapper.diagnose` から診断を得る。
   全入力受理の場合に生成 `Mapper.parseWithSourceMap` を呼び、反射で AST を canonical 化する。
6. `@eval` と dotted mapping は既存 `EvaluatorGenerator` の出力もコンパイルし、
   実際にロードしたクラスのメソッド名・abstract 属性を保存する。
   callback を実装しない具象 subclass のコンパイル結果も採取する。評価値は対象外。

生成ソース、class、文法ファイル、ログは `mktemp -d` の専用ディレクトリ内に置き、終了時に削除する。
`generatedSourceSha256` / `evaluator.sourceSha256` は実際に生成したソースの SHA-256。
一時ディレクトリの絶対パスだけを `<scratch>` に置換する。診断の文言、配列順、値は保持する。
JavaCompiler のメッセージは `Locale.ROOT`、反射のメソッド一覧は signature 順、
native expected の集合は既存 mapper と同様にソートする。

## JSON の意味

各 `*.json` は1つの文法 variant。`feature` が同じファイルを合算して機能別件数とする。
`source` は採取元の SHA。import 原文は `imports` に併記してあり、参照ファイルを推測する必要はない。

- `stage`: `executed`、`compilation_failed`、`generation_failed`。
  失敗理由は `generation` または `compilation.diagnostics` に原文で保存する。
- `accepted`: **`Parsed.isSucceeded()`**。prefix 成功を含む。
  全入力の構文受理は `accepted == true && allConsumed == true`。
  `allConsumed` は `ParseContext.allConsumed()` の実測で、空入力の構文失敗でも true になる。
- `consumed` / `matched`: 最終 context の code point cursor。最遠位置ではない。
  `nativeFailure` は最初の解析の最遠 offset / consumed / matched / expected を別々に保存する。
- `ast`: `{type, span:[start,end], fields}`、半開区間・code point 単位。
  文字列・整数・リスト・optional の null を再帰処理し、node span は既存 mapper の
  `sourceSpanOf` をそのまま使用する。prefix 成功・構文失敗では mapping を呼ばず null。
- `mappingError`: 全入力受理後の mapping 例外のクラス名とメッセージ。構文失敗とは区別する。
  parser 自体の例外は `parseError`、診断取得の例外は `diagnosticError`。
- `captures`: 実際の token tree の original children を内側から辿る完了順。
  生成 `__CaptureBinding.captureBindings()` の site ID、capture 名、span、原文を保存する。
  site ID と名前の対応だけは採取元の `CaptureBindingPlan` を反射で読む。
  AST field からの逆算や同値 capture の重複除去は行わない。失敗して rollback された木は出現に含めない。
- `declarations` / `references` / `scopeDepth`: 同じ解析 context の
  `ScopeStore.getAllDeclarations` / `getAllReferences` / `currentScopeDepth`。
  API が保持した履歴であり、独自の scope enter/leave ログを生成したものではない。
- `diagnostics`: `Mapper.diagnose` の record 全項目、および `ScopeStore.getDiagnostics`。
  `origin` で区別する。ScopeStore の `kind` は severity 名で、expected API がないため
  `expected:[]` とする（メッセージから推測しない）。
- `astTypes`: 実際にロードした AST の型・アクセサ・abstract 属性・enum 宣言順。
  `enumValue` / `enumError` は生成 `Mode.fromText(input)` の結果。
  `catalogs` は既存 Java frontend の `RustGrammarLowering.lower(...).rules().catalog()` の値。
  catalog の parser 実行・AST 採取は別途通常の Java 経路で行う。
- 未実行の `accepted` / `allConsumed` / cursor / AST / mappingError は **null**。
  このとき capture・scope・diagnostics の空配列も「空だった」という実行 oracle ではない。
  必ず `stage` を先に読む。`parseError` のある `accepted:null` も false に変換しない。

## 機能別ケース数

「実行」は parser を呼び出した件数。各ケースの variant と入力は JSON を参照。

| §4 の機能 | ケース | 実行 | ファイル接頭辞 |
|---|---:|---:|---|
| DigitParser | 4 | 4 | digit |
| その他 posix（AlphabetParser） | 4 | 4 | posix |
| CI | 7 | 7 | ci |
| REGEX | 9 | 9 | regex |
| 汎用 Extern | 7 | 7 | extern |
| 透明な mapping・多段 alias | 6 | 6 | mapping-sum, mapping-alias |
| dotted mapping | 3 | 3 | mapping-dotted |
| @skip | 4 | 4 | skip |
| @enum | 7 | 3 | enum |
| @commonField | 5 | 3 | common-field |
| @typeof | 5 | 5 | typeof |
| @longestChoice | 4 | 4 | longest |
| @predictiveChoice | 10 | 10 | predictive |
| @comment | 9 | 9 | comment |
| scope なし @backref | 4 | 4 | backref |
| @recovery | 10 | 8 | recovery |
| @eval | 8 | 8 | eval |
| @import | 7 | 0 | import |
| ERROR() | 3 | 3 | error |

| 追加項目 | ケース | ファイル接頭辞 |
|---|---:|---|
| @catalog | 3 | catalog |
| UNTIL | 5 | until |
| EMPTY | 3 | empty（反復 variant を除く） |
| EOF | 3 | eof |
| LOOKAHEAD | 6 | lookahead |
| NEGATIVE_LOOKAHEAD | 3 | negative-lookahead |
| bounded repeat `{0,0}`, `{n,}`, EMPTY 反復 | 9 | empty-zero/two/one-two, repeat |
| `%` 分離 | 5 | separated |
| CHAR_RANGE 境界 | 5 | char-range |
| NEGATION 空集合 | 3 | negation-empty |
| 数値 capture | 7 | numeric |

### 採取範囲と未採取部分

19機能すべてに採取結果がある。ただし次は既存生成物を実行できず、**runtime の値は未採取**。
既存ソースや生成物を修正して成功させる代替経路は使っていない。

- `@import` 7件: alias 付き1段・2段の6件は生成 Java の構文エラー。
  循環1件は `parseWithImports` の例外。循環文言と解決後の構造までは採取済み。
- `@recovery(skip)` 2件: 存在しない runtime API を参照してコンパイル失敗。
- enum capture 3件と重複 enum 名1件: 生成 mapper の型不一致、重複定義によるコンパイル失敗。
  capture しない enum の3件は実行済みで、ON/OFF と fromText の失敗も固定。
- common field の欠損・異型 variant 各1件: 生成 AST が interface の accessor を実装できず
  コンパイル失敗。正常な2 variant の値・abstract accessor は実測済み。
- predictive の空入力1件: parser と diagnose が `NullPointerException`。
  実行済みだが受理/不受理の通常値は得られていない。

Extern はこの corpus 専用の `ExternalCursorParser.java` を使用する。
既存 `Chain` / `WordParser` / `MatchOnly` を組み合わせ、`😀x` で C=1/M=2 を返す。
任意の状態依存 scanner、memo policy 間の等価性を検証したものではない。
parser 不成立後の投機的 token すべてを記録する trace や、評価時 scope は対象外。

### evaluator 観測

| kind / 構成 | evaluator のコンパイル | 生成 callback / helper | 空の具象 subclass |
|---|---|---|---|
| literal | 成功 | evalLiteral | 成功 |
| passthrough | 成功 | evalLiteral | 成功 |
| variable_ref | 成功 | evalVariable, resolveVariable (abstract) | 失敗 |
| binary_arithmetic | 成功 | evalBinary, evalLiteral, evalLeaf / applyBinary (abstract) | 失敗 |
| conditional | 成功 | evalConditional, evalLiteral | 成功 |
| manual | 成功 | evalLiteral (abstract) | 失敗 |
| unknown | 成功 | evalLiteral (abstract) | 失敗 |
| 自動 literal を和型へ指定 | 生成時拒否 | concrete mapping を要求する例外 | 未実行 |

`mapping-dotted` では `evalNodeA` / `evalNodeB` と親の `evalNode` adapter も記録した。

## 疑義

以下は実測結果から見た不具合・仕様との差の候補。意図された仕様である可能性は残る。
修正を正解とする判断は、この corpus の採取とは分ける。

1. **REGEX の UTF-16 / code point 混在**。
   `regex-emoji/1` は `😀` 単独を拒否し、`/2` の `😀x` は2 code point 全体を capture する。
   `regex-prefix` の `'😀' REGEX('a')` は `😀a` と `😀ab` をともに拒否する。
2. **recovery の同期位置が非 BMP で過大**。
   `recovery-sync/3` の `😀;x` は C=M=3、capture も `😀;x`。
   `;` の後の `x` まで巻き込む。ASCII の `bad;ok;` は `bad;` と次の `ok;` に分かれる。
3. **root の recovery が実行経路に反映されない**。
   `recovery-root/1` の `bad;` は失敗する一方、同じ同期注釈を子 rule に置いた
   `recovery-sync/1` は回復する。
4. **AUTO が後続 statement の先頭まで消費**。
   `recovery-auto/1` の `bad;ok;` は `bad;ok` を capture し、C=M=6、末尾 `;` が残る。
   推定された同期文字列自体を消費するため、次の正常 statement を解析できない。
5. **SKIP の生成 Java と runtime API が不整合**。
   `toParsed()`、`isAtEnd()`、引数なし `getCursor()`、`advanceCursor(int)` が解決できない。
6. **import alias / 多段統合が実行可能にならない**。
   1段でも `b.Leaf` を含む生成 parser が javac の `'{' expected` で失敗。
   2段解決後は `Root` と `b.Leaf` だけで、body 内の `c.Atom` 参照に対応する rule がない。
   CLI 自体は import 解決を行わず、循環入力でも exit 0。API は循環を検出する。
7. **enum capture の mapper 型不一致**。
   `@mapping(Config, params=[mode])` に enum Mode を渡す生成コードが
   `Mode cannot be converted to java.lang.String`。capture しない構成は正常に動作する。
8. **enum 名衝突 / common field 不適合が frontend で拒否されない**。
   `'on'` と `'ON'` は重複 ON を生成。共通 `name` の欠損や String/int 不一致も
   CLI exit 0 のあと javac で失敗する。
9. **`@comment` 設定の値・block 指定が反映されない**。
   `line:'#'` 単独では `a//x\nb` / `a#x\nb` をどちらも拒否。
   `@whitespace: javaStyle` を併用すると前者のみ受理し、`#` は依然拒否する。
   block-only 設定でも `a/*x*/b` / `a(*x*)b` を拒否する。
10. **predictive choice が空入力で例外**。
    `predictive/5` は `PositionResolver.stringIndexInRootFrom(...)` の null に起因する NPE。
    通常の syntax 診断に到達しない。
11. **回復した領域が AST から欠落する**。
    `recovery-sync/1` は capture に `bad;` があるが AST の items は正常な Stmt 1個のみ。
    `recovery-sync/3` は全入力受理で items が空、公開 diagnose にも診断がない。

運用上の差: CLI generator 名は大文字が必須。短名 `NumberParser` / `IdentifierParser` は
生成コードではコンパイルできるが、validator は unresolved の警告を出す。
DigitParser / AlphabetParser は採取 grammar で FQCN を使用した。

既知の特異な契約も保存したが、不具合とは断定しない。
EMPTY の C=0/M=1、EMPTY{0,0} の失敗、LOOKAHEAD→UNTIL の C=0/M=2、
UNTIL の終端なし EOF 成功、`%` の空入力拒否・末尾 separator の未消費、
構文受理と `@typeof` / 数値変換例外の分離、backref 不一致時の受理＋ERROR が該当する。
`lookahead-failure/1` は最終 C=M=0 でも最遠 C=0/M=1、公開 syntax offset=1 であり、
診断位置を最終 consumed と同一視できないことも実測した。
