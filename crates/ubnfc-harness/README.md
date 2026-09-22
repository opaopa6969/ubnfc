# 適合性 harness

`cargo run -p ubnfc-harness -- list [--root UNLAXER_PARSER_DIR]` で件数を表示する。
既存5系統は指定した unlaxer-parser、第6系統はこの crate と同じ checkout の
`corpus/oracle/` を読む。cwd や `--root` で独自 oracle の参照先は変わらない。
個別の採取先は `corpus::oracle::load_all(path)` で指定できる。

oracle は63ファイル・168ケース。実行153（通常152、parser/diagnose の NPE 1）、
生成 Java のコンパイル失敗14、生成失敗1。台帳 `divergences.json` は件数から除く。
`stage` を先に判定し、未実行を `GenerationFailure`、実行を `Oracle` に正規化する。
NPE の `accepted:null`、prefix 成功と全入力消費、構文受理と mapping 例外を区別する。
採取元 SHA・import 原文・parseError・diagnosticError・追加の観測は `CorpusFile.raw`
に全て保持する。grammar metadata には cases 以外を保持する。

## D-017 expectedDivergence

台帳の `file` は oracle ディレクトリ直下のファイル名、`caseId` は元 JSON の ID。
全エントリの参照と重複を検証し、`follow-spec` のみを `Case.known_divergences` に付ける。
`undecided` は免除しない。ケース単位や機能全体への包括的な免除は行わない。

`divergence::compare(&expected, &actual, &case.known_divergences)` は、runner が返す
同じ `Expectation` 型と比較する。現段階では実行器を含まない。

| observation | 比較対象 |
|---|---|
| accepted | `accepted` と `allConsumed`（prefix と全入力の受理状態） |
| consumed / matched | それぞれの code point cursor |
| ast | canonical AST と構築元の順序付き `captures`（binding/span/text を含む） |
| diagnostics | 公開診断列 |
| mappingError | 構文受理後の mapping 例外 |
| generation | 生成/コンパイルの成否と失敗メッセージ |

`declarations` は常に厳密比較する。生成に失敗した側の null/空配列は runtime oracle
ではないため、生成成功への変化は `/generation` の1差分として扱う。
既存5系統への台帳免除は適用しない。

- 値が一致: `Pass`（登録済みでも差がなければ Pass）。
- 実際の差分がすべて登録済み観測群: `KnownDivergence(diffs)`。
- 未登録の差分を含む: `Fail(diffs)`。既知分も含む全差分を JSON Pointer と値付きで返す。

既知判定は仕様への適合を証明しない。登録済み観測群で oracle と異なることの分類であり、
将来の runner の仕様テストと併用する。採取値自体は変更しない。

## 台帳の判断

README の疑義1〜11を実際の grammar/input/観測と照合し、29ケース・75項目を登録した。
`follow-spec` 60項目、`undecided` 15項目。各項目の根拠は台帳の reason に記載する。
未決定は次の契約が確定していないためで、採取値の修正や推測した期待値の追加はしない。

- 回復した root/statement の AST 表現、および正常に回復した入力の診断契約。
- `@whitespace` がない line comment の改行消費と、javaStyle と明示コメント設定の合成。
- 循環 import は API が正しく失敗する一方 CLI が成功しており、単一 generation 観測での扱い。

`comment-line/2` は // 未設定なので拒否は維持し、不正に // を走査した診断だけを登録。
`comment-block/1` も未設定の /* */ を拒否した対照例なので登録しない。
正常な root recovery 入力、同期文字列がない入力、README 末尾の特異だが既知の契約、
CLI generator 名や短名警告などの運用差を、一律の乖離にはしない。

## 全 corpus の Java 実行

workspace ルートで front と Java 生成器を用意してから実行する。追加の Rust 依存は不要。

~~~sh
cargo build --bin ubnfc
(cd ubnfc-java && mvn -o -q -DskipTests package)
cargo run -q --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-22-java-corpus.md \
  --json docs/reports/2026-09-22-java-corpus.json
~~~

--families associative,oracle または loader の6系統名
inline-ast,token,rule,external-grammar,operations,oracle で対象を絞れる。
--jobs は同時 build 数（既定4、1以上）。--root / UNLAXER_PARSER_DIR は
upstream corpus の参照先。UBNFC_BIN と UBNFC_JAVA_GENERATOR には
front バイナリと Java 生成器の jar/classes のパスを指定できる。
既定はこの checkout の target/debug/ubnfc と
ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar。自動インストールは行わない。

文法合成は既存 loader を使う。token は RustConformanceTest.tokenCorpus の
T / END / @mapping(Value, params=[value])、semantic-cardinality は
SemanticCardinalityConformance の Grammar.ubnf.txt の置換、tiny 系は各
Tiny*ConformanceTest.grammar に従う。whitespace は false でもフィールドがあれば
@whitespace: javaStyle を付ける。inline は mktemp -d 内の .ubnf に書き、
oracle の import 原文も同じディレクトリに展開する。File は元のパスを直接 front に渡す。
scope-store は文法を持たない操作列なので未観測とし、parser 文法を捏造しない。

全 grammar_ref / variant を展開する。表の単位は case × variant で、元の871行/runとは
区別する。同一文法の IR / Java 生成 / javac / driver 起動は一度だけ。
キャッシュは安定した FNV-1a/64 hash と原文一致で判定し、hash 衝突も分離する。
import 内容も key に含め、File の相対 import は親パスを解決文脈に含める。
並列完了順によらず corpus / case ID / 文法で結果を整列する。

IR、生成、javac の成功・失敗・未実行を独立に保存し、段階失敗後は比較しない。
driver のプロトコル/実行エラーを比較 Fail から分ける。比較は既存 runner と
D-017/D-018 の登録をそのまま利用する。欠落項目だけなら未観測、既知差分があれば
Known、未登録差分があれば Fail。Known/Fail に併存する未観測項目も保存する。
Raw の evolution に結果 oracle を補完しない。

Markdown は系統別・文法別の結果、拒否の機能/診断コード別件数、Fail の差分先頭3行、
未観測項目を出す。JSON には文法原文/import、段階ログ、全ケースの期待値・実測値・
全差分を残す。生成拒否件数は「最初に拒否された機能」であり、潜在する全未対応機能の
件数ではない。拒否表には重複除去した build 数・元ケース数と展開後件数を併記する。

終了コードは、段階失敗・実行エラー・Fail がなければ0、あればレポートを保存して1。
Known と未観測だけでは非ゼロにしない。引数・入出力・準備エラーも1。
associative は通常文法が24 Pass、reversed-metadata は現在の front により
E-PRECEDENCE-ORDER で24件が IR 失敗となる。全文法を対象にするためこの variant も
除外せず、CLI の E2E テストは24 Pass とこの明示拒否を両方確認する。

## 2026-09-22 triage の比較契約

根拠となる既存テストの行番号・分類・driver の追加仕様は
[triage レポート](../../docs/reports/2026-09-22-triage.md) に記載する。
従来の driver protocol 文書との差は以下。

- primitives/lexical の `value` は `Value` AST の field。AST の型・span・全 fields を比較する。
  numeric-capture は `ast.fields.value` の Java int / Rust lexeme、tiny-string も mapped field。
  semantic/mixed/associative/cardinality の top-level `value` は評価値のまま。
- cardinality の簡略 `expected` は canonical AST に展開して比較する。
  不受理 variant に value/texts/nodes/expected を要求しない。
  semantic/mixed の `texts` は既存 Rust probe の観測であり Java には要求しない。
- Oracle の `allConsumed` は prefix cursor の全消費。wire の全入力構文受理とは区別する。
  primary syntax expected は非空、trailing expected は `end of input` を厳密に比較する。
  scope の診断は構文診断と分離し、文面・重大度・位置を維持する。
- `observations` に fixture の captureNames/queries/valueSpanField/valueSpanListField と
  parentRollback の起動条件、family/variant を送る。oracle 値は送らない。
  parentRollback は boolean の成功応答でなく、`rollback` の cursor/scope snapshot を検証する。
- `ExpectedRejection` は import-cycle の `ir/E-IMPORT-CYCLE` だけ。
  元の生成失敗 oracle と build の実測失敗を保持し、別の失敗や予期しない成功を Pass にしない。
  reversed-metadata は既存テストでは正常生成・実行を要求するため IR失敗のまま。
- run-all は `sha256sum` を使い、generator/front の実行前後の SHA を照合してレポートに保存する。
  Java の E2E テストは `UBNFC_JAVA_GENERATOR` があればその既存 jar を使う。
