# D-018: 未決定の乖離 15 観測に対する決定案

状態: 提案。対象は `divergences.json` の `undecided` 15 行のみ（回復 5、コメント 9、import 1）。
根拠は [設計 v1](../design/03-design-v1.md) §3.5・§3.10・§4・§8 D-017、[互換行列](../compat-matrix.md) C03・C06・C14、[oracle の採取規則と疑義](../../corpus/oracle/README.md) 2–4・6・9・11。
既決の D-017（設定無視・root recovery 無効・AUTO/非 BMP 過剰消費等は再現しない）を前提とする。互換行列の古い「暫定参照」「未確定」記述より D-017 を優先する。
以下の「観測」は保存済み JSON、「ソース」は現在の Java ソースの静的確認、「契約」は ubnfc への提案である。再実行と checkout の revision 一致は未確認。JSON の採取元は `ed4178e673f93e719b62a7bd181f3a219abf8ed1`。
参照略記: `U=/home/opa/work/unlaxer-workspace/unlaxer-parser`、`J=U/unlaxer-dsl/src/main/java/org/unlaxer/dsl`、`C=U/unlaxer-common/src/main/java/org/unlaxer`、`S=U/unlaxer-dsl/specs`。位置はすべて Unicode code point、span は半開区間。

## (a) 回復領域の AST と回復成功時の診断

### 確認した挙動と仕様の判定

| oracle のケース・入力 | 保存された値 |
|---|---|
| `recovery-sync/1`・`bad;ok;` | `accepted=true, allConsumed=true, C=M=7`。`Root[0,7].items` は `Stmt[4,7]{value:"ok"}` だけ。captures は `items/Root:0[0,4]="bad;"`, `value/Stmt:0[4,6]="ok"`, `items/Root:1[4,7]="ok;"`。`diagnostics=[]` |
| `recovery-sync/3`・`😀;x` | `accepted=true, allConsumed=true, C=M=3`。`Root[0,3]{items:[]}`、capture `items/Root:0[0,3]="😀;x"`、`diagnostics=[]` |
| `recovery-auto/1`・`bad;ok;` | `accepted=true, allConsumed=false, C=M=6, ast=null`。capture `items/Root:0[0,6]="bad;ok"`、`trailing_input@6` |
| `recovery-root/1`・`bad;` | `accepted=false, allConsumed=false, C=M=0, ast=null, captures=[]`、`syntax@0` |

上表は [recovery-sync.json](../../corpus/oracle/recovery-sync.json)、[recovery-auto.json](../../corpus/oracle/recovery-auto.json)、[recovery-root.json](../../corpus/oracle/recovery-root.json)。いずれも `mappingError=null`。
`C/parser/combinator/SyncPointRecoveryParser.java:102–150` は子の失敗を rollback し、同期文字列までを含めた領域に error marker と Token を作って成功する。`:125–127` は UTF-16 長を `CodePointLength` に渡すため、非 BMP の過剰消費を説明する。
`J/codegen/ParserRuleEmitter.java:302–335,500–543,1312–1315` は FOLLOW（空なら `;`）を AUTO の消費型 wrapper に渡し、参照先を wrapper に切り替える。`J/codegen/ParserGenerator.java:254–256` の root factory は通常 parser を返す。
`J/codegen/MapperRuleEmitter.java:144–156,889–919,993–1005` は通常 rule の parser class を mapping 対象にし、対応する子がない回復 Token から `Stmt` を生成しない。`J/codegen/MapperGenerator.java:184–204` は全入力成功なら診断なし、prefix 成功なら `trailing_input`、失敗なら `syntax` を返す。
`corpus/oracle/tools/OracleCollector.java:205–215` は全入力受理時だけ mapping する。したがって AUTO の `ast=null` は mapping 未実行であり、回復 AST を構築できない証拠ではない。
`U/docs/ubnf-guide.md:377–385` は recovery wrapper を説明するが、AST 投影と回復成功時の公開診断を規定しない。`S/annotations.md`・`S/ubnf-syntax.md` にも該当契約はない。設計 §3.10 も AST 投影を独立契約と明記する。
**判定:** 非 BMP・AUTO・root は D-017 の既存バグ。回復ノードの省略と成功時の診断なしは、ソースどおりだが公開 recovery 契約が不足しており、それだけで仕様違反とは断定しない。

### 決定案: `recovery-projection` と `recovery-diagnostics` を新設

型付き AST の既存型を維持し、回復を別表で必ず観測可能にする。任意の record に代替 error node や架空の必須値を埋め込まない。

1. 採用した回復ごとに `recoveredRegions` へ `{ruleId, mode, span:[s,e], syncSpan}` を記録する。`s` は失敗 rule の開始位置、`e` は再開位置。同期なし失敗・ゼロ前進を回復成功にしない。
2. SYNC は同期文字列を含めて消費する。AUTO は推定 FOLLOW の直前で再開し、fallback の `;` は SYNC と同様に消費する。root にも注釈を適用する。これらは D-017 の修正を具体化する。
3. 回復した rule 自体の typed node は作らない。node の list field はその要素を省略、optional は不在にする。必須 scalar が欠ける親も構築せず、同じ規則で上へ伝播する。root が構築不能なら `ast=null` とし、回復欠損自体は `mappingError` にしない。
4. 回復した rule を直接束縛する capture は site ID・完了順・原文・`[s,e]` を保持し、外側の capture は通常どおり自身の全範囲を持つ。失敗した rule 内の capture・意味変換・scope 効果は rollback し、成功したかのように捏造しない。正常 node の値・span は保持し、構築できる親の span は回復領域も含む。
5. `recoveredRegions` に関連 capture の出現番号を付け、省略した node と正常な optional 不在を識別する。表・capture・診断は採用枝に限り公開し、rollback と memo 再生で欠落・二重登録させない。
6. 回復 1 件につき必ず診断を 1 件返す。`kind="recovery", severity="ERROR", offset=s, length=e-s, origin="recovery"`。rule と回復領域を関連付け、`expected/farthestOffset/farthestExpected` は回復走査前の当該失敗試行から取得する。主位置は最遠位置や同期記号の位置ではない。
7. 回復成功は `accepted=true` と両立する。後続の `trailing_input` は別診断として確定 consumed に置く。回復できず全体失敗なら従来の `syntax` 規則を使う。診断順は採用回復の順、終端の失敗診断の順とする。
8. oracle 用 `ast` は採取規則どおり全入力受理時だけ投影し、prefix 成功なら `null`。正常部分の構築レシピ・capture・回復表は保持する。

以下は実測ではなく、この契約と D-017 から導く期待値である。

| ケース | 契約による期待値 |
|---|---|
| `recovery-sync/1` | AST・captures・`C=M=7` は現観測どおり。回復表 `[0,4]` と `recovery@0` を追加 |
| `recovery-auto/1` | 全入力受理、`C=M=7`。AST・captures は `recovery-sync/1` と同形、回復表 `[0,4]`、`recovery@0`。`trailing_input` は消える |
| `recovery-root/1` | 全入力受理、`C=M=4, ast=null, captures=[], mappingError=null`。回復表 `Root[0,4]`、`recovery@0`。`value` は作らない |
| `recovery-sync/3` | prefix 受理、`allConsumed=false, C=M=2, ast=null`。capture は `items/Root:0[0,2]="😀;"`。回復表 `[0,2]`、`recovery@0` と `trailing_input@2` |

## (b) コメントの改行消費と javaStyle の合成

### 確認した挙動と仕様の判定

[comment-line.json](../../corpus/oracle/comment-line.json) `/3` は `line:'#'`、空白設定なし、入力 `a#x\nb` に `accepted=false, allConsumed=false, C=M=0, ast=null, captures=[]`、`syntax@1`（expected は `'//'`, `'b'`）。
同 `/2` の `a//x\nb` も不受理だが `syntax@5`、`nativeFailure.consumed=4, matched=5`。これは改行を消費せず照合だけした事実と整合する。
[comment-whitespace.json](../../corpus/oracle/comment-whitespace.json) `/2` は `javaStyle` と `line:'#'` を併用して `a//x\nb` を全入力受理し、`C=M=6, ast=Value[0,6]{}, captures=[], diagnostics=[]`。同 `/3` の `a#x\nb` は不受理・`syntax@1`。
`J/codegen/ParserGenerator.java:273–310` は `line` の値を読まず `CPPComment` を登録し、javaStyle では `SpaceParser`・`CPPComment`・`BlockComment` を追加する。
`C/parser/clang/CPPComment.java:32–35` は `//` と `WildCardStringTerminatorParser(true, LineTerminatorParser)`。後者の `:31–32` は `MatchOnly` を用い、`LineTerminatorParser.java:20–25` は CRLF・CR・LF・EOF を認識する。`C/parser/posix/SpaceParser.java:11` は空白 6 文字を列挙する。
`S/ubnf-syntax.md:92–109` は javaStyle に `//` が含まれると明記。`S/annotations.md:192–201,225–239` は rule の優先順位と profile の空白・行/block コメントを規定する。guide `:261–280` は空白と comment を別に説明するが、改行消費と明示設定の置換/加算は規定しない。
**判定:** `#` の無視は D-017 のバグ。javaStyle 自体の `//` 受理は仕様に沿う。改行保持と明示コメントとの合成は仕様不足なので、今回の 9 観測を新契約で固定する。

### 決定案: `comment-trivia` を新設

1. 行コメントは設定した開始文字列から CRLF・CR・LF・EOF の直前までを消費し、改行自体は残す。終端は `MatchOnly` 相当で照合し、matched は改行の末尾まで到達する。EOF の照合は前進しない。
2. javaStyle の集合は空白 `U+0009–U+000D, U+0020`、`//`、`/* … */`。明示 `@comment` の line/block は値どおりの区切りをこの集合へ**追加**し、既定集合を置換しない。同一の区切りは重複除去する。
3. 設計 §3.5 の優先順位を適用した有効 policy に javaStyle 相当がなければ、明示コメントだけを有効にする（global `none` でも同様）。ただし rule `@whitespace(none)` はその rule 内の自動 trivia 全体を無効にする。既存の挿入位置を保ち、別 rule の有効化を漏らさない。
4. 改行が必要な文法は明示的に照合できる。この性質と `javaStyle` の既存利用を守るため、改行保持と集合の加算を選ぶ。`rawSettings` から `effectiveDelimiterSet` と `insertionPlan` を front で一意に決定し、両 backend で共有する。

契約上、`comment-line/3` は引き続き不受理・`C=M=0, ast=null, captures=[]`。理由は `#` 無視から未消費改行へ変わり、Java の `syntax.offset` は終端照合を含む `4`（局所 `C=3,M=4`）になる。この診断行は既に `follow-spec` であり、今回の 15 行には含めない。
`comment-whitespace/2` の 5 観測は現値を維持する。対照の `/3` は `#` と改行をそれぞれ消費し、全入力受理・`C=M=5, Value[0,5]{}, diagnostics=[]` になる。これらは契約からの期待値であり、修正版の実行結果ではない。

## (c) 循環 import の CLI 成功と API 拒否

### 確認した挙動と仕様の判定

[import-cycle.json](../../corpus/oracle/import-cycle.json) `/1` は `Probe→B→C→B`、`generation.cliExit=0, cliError=""`、`apiError="java.lang.IllegalStateException: 循環 import を検出しました: B.ubnf (スタック: [C.ubnf, B.ubnf])"`、`stage="generation_failed"`。入力 `a` の実行はなく、`accepted` 等の null と `diagnostics=[]` は runtime の期待値ではない。
`J/CodegenMain.java:98` → `J/CodegenRunner.java:66–67` は `UBNFMapper.parse` のみを使う。API は `J/bootstrap/UBNFMapper.java:127–163` で path をキーに再帰し、処理中集合への再訪を拒否する。`:119–125` の Javadoc も循環拒否を明記する。採取器 `OracleCollector.java:75–106` は CLI と API を別途実行している。
guide `:454–467` は alias 名前空間、設計 §4 は import graph と統合シンボル表を要求する。指定の仕様本文には循環の診断形式・パス正規化・CLI/API 同値性はない。`attachImportedGrammars`（`:167–213`）の直接 prefix だけでは推移的な body 参照の解決を保証しない。
**判定:** API 拒否は明記された契約に一致し維持する。CLI 成功は import を解決しない実装欠落。循環を許可する根拠にはしない。解決経路と診断形式は新たに固定する。

### 決定案: `import-resolution` を新設

1. CLI と生成 API は同じ resolver を必須とし、未参照の import も含め全依存を生成前に解決・検証する。構文 AST だけを返す `parse` と生成入口は区別する。
2. 相対パスは import を記述した source の親を基準とする。CLI は実在ファイルの正規化実パスを `sourceId` にし、API resolver は `(importerSourceId, path)` から同一資源で安定した `sourceId` と本文を返す。alias は資源同一性とは別に扱う。
3. 入口も処理中に登録し、宣言順の DFS で処理中の `sourceId` への辺を循環として拒否する。完了済み資源の再利用は許す。名前を alias で束縛し、body 内の参照も定義元の環境で推移的に `qualifiedSymbolId` へ解決して単一シンボル表へ統合する。
4. 循環は `kind="import_cycle"`、主位置は循環を閉じた辺の `@import` 先頭（sourceId と cp offset）、経路は訪問順で報告する。本例なら `C.ubnf` の offset `12`、循環部分 `B.ubnf→C.ubnf→B.ubnf`。未解決は同じ位置規則で `import_not_found`。
5. 両経路とも生成段階で失敗し、生成物を公開しない。CLI は非ゼロ終了、API は構造化エラーを返す。runtime 拒否へ置き換えない。旧 Java 例外のクラス名や HashSet の表示順は新契約には含めない。
6. 既存 `generation` 観測は正しい API 拒否と誤った CLI 成功を混在させているため、行全体を `reproduce` や無条件の `follow-spec` 免除にしない。新契約では CLI/API の成否・失敗段階・kind・位置を別々に照合し、API 成功への退行は必ず失敗にする。

## 台帳への適用案（15 行）

`id` は一意な `caseId#observation` 表記。ファイル名は caseId の `/` より前に `.json` を付けたもの。

| id | 新しい decision |
|---|---|
| `recovery-auto/1#ast` | `new-contract:recovery-projection` |
| `recovery-root/1#ast` | `new-contract:recovery-projection` |
| `recovery-sync/1#ast` | `new-contract:recovery-projection` |
| `recovery-sync/1#diagnostics` | `new-contract:recovery-diagnostics` |
| `recovery-sync/3#ast` | `new-contract:recovery-projection` |
| `comment-line/3#accepted` | `new-contract:comment-trivia` |
| `comment-line/3#ast` | `new-contract:comment-trivia` |
| `comment-line/3#consumed` | `new-contract:comment-trivia` |
| `comment-line/3#matched` | `new-contract:comment-trivia` |
| `comment-whitespace/2#accepted` | `new-contract:comment-trivia` |
| `comment-whitespace/2#ast` | `new-contract:comment-trivia` |
| `comment-whitespace/2#consumed` | `new-contract:comment-trivia` |
| `comment-whitespace/2#diagnostics` | `new-contract:comment-trivia` |
| `comment-whitespace/2#matched` | `new-contract:comment-trivia` |
| `import-cycle/1#generation` | `new-contract:import-resolution` |

## 適用時の条件

現行 [divergence.rs](../../crates/ubnfc-harness/src/divergence.rs) の `Decision` は `follow-spec` と `undecided` のみ。`undecided` の差分は `Fail`、一致なら `Pass` であり、この文書を作っても判定は変わらない。表は移行案であって、現行 JSON へそのまま投入できる値ではない。
適用には名前付き契約の読込・独立期待値による検証を先に実装する。`new-contract` を観測群の一括免除にせず、契約適合を必須にする。旧値と同じ場合も契約の検証を省かない。
`ast` 観測は captures も含む。`corpus/oracle.rs:136–158` は generation 全体を失敗メッセージへ保存し、`divergence.rs:137–158` は生成成功への変化まで同じ観測にまとめるため、import の経路別検証が必要である。
受入条件は上記の期待値を Java/Rust 両 backend で照合し、回復の root/list/optional/scalar・rollback/memo、LF/CRLF/CR/EOF、コメント合成、循環・非循環の共有 import・相対パス別名を追加検証すること。本提案では実装・台帳・oracle を変更していない。
