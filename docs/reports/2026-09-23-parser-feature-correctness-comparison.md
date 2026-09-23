# workspace 内パーサ実装の機能・正しさ比較（2026-09-23）

速度は測らない。速度は別報告
[`2026-09-23-parser-speed-comparison.md`](2026-09-23-parser-speed-comparison.md)（別ブランチで作成中）を参照。
本報告は **何ができるか（Part 1）**、**何に対して正しさを検証されているか（Part 2）**、
**同じ入力を実際に流したときに何が食い違うか（Part 3）** の 3 部構成。

セル値は「対応 / 部分的 / 非対応」。根拠を `パス:行` かテスト名か報告書で示す。
根拠を挙げられないものは「不明」と書き、推測で埋めない。

---

## 0. 対象実装

比較対象は 3 リポジトリにまたがる 10 実装。3 つの文法（UBNF / P4 / Operators・Power）で群に分ける。

### UBNF 文法（この workspace の文法記述言語そのもの）

| 記号 | 実装 | 位置 | 種別 |
|---|---|---|---|
| **A** | ubnfc 手書き frontend | `crates/ubnfc-front/src/syntax` | 手書き再帰下降 |
| **B** | ubnfc 自己ホスト生成 frontend | `crates/ubnfc-selfhost`（`src/generated`） | `ubnf.ubnf` から ubnfc が生成 |
| **C** | unlaxer-ubnf | `unlaxer-parser/rust/unlaxer-ubnf` | 手書き Rust、依存ゼロ |
| **D** | Java bootstrap | `unlaxer-parser/unlaxer-dsl/src/main/java/org/unlaxer/dsl/bootstrap/UBNFParsers.java`（1621 行）+ `UBNFMapper` | unlaxer-common combinator |

> **訂正**: 課題文が言う「生成された UBNF パーサ（unlaxer-dsl 側）」は実在しない。
> `unlaxer-dsl/ubnf-vscode/src/main/java/org/unlaxer/dsl/bootstrap/generated/UBNFParsers.java` は
> 9 行の委譲スタブで、本体は手書き bootstrap をそのまま呼ぶ。unlaxer 側の UBNF self-host は
> **文法ファイルとしては存在するが bootstrap の置き換えには至っていない**。
> ubnfc の B はこの意味で unlaxer 側に対応物が無い。

### P4 文法（tinyexpression の式言語、124 規則）

| 記号 | 実装 | 位置 |
|---|---|---|
| **R1** | ubnfc Rust backend の生成物 | `examples/p4-rust/src/generated` |
| **J1** | ubnfc Java backend の生成物 | `examples/p4-java/src/main/java/org/ubnfc/p4/generated` |
| **R2** | unlaxer Rust codegen の生成物 + `unlaxer-runtime` | `tinyexpression/rust/tinyexpression-rs` |
| **J2** | 旧 Java（生成 `TinyExpressionP4Parsers` + `unlaxer-common` combinator、公開 facade `P4PreferredAstMapper`） | `tinyexpression/target/generated-sources/...`、`tinyexpression/src/main/java/org/unlaxer/tinyexpression/p4/P4PreferredAstMapper.java` |

> J2 の生成物はビルド時生成でリポジトリに commit されていない
> （`tinyexpression/pom.xml:127-150` の exec-maven-plugin が `org.unlaxer.dsl.CodegenMain` を走らせる）。

### Operators / Power（結合性の小文法）

`examples/handwritten-target/{rust,java}` の手書き参照実装と、ubnfc の 2 backend 生成物。
手書き参照実装は「生成物が目指す形」を人が書いたもので、`ubnfc-target-rust` に 68 テストがある。

---

## Part 1 — 機能マトリクス

### 1.1 UBNF 文法を読む 4 実装（A/B/C/D）

| 能力 | A ubnfc 手書き | B ubnfc 生成 | C unlaxer-ubnf | D Java bootstrap |
|---|---|---|---|---|
| 型付き AST | 対応 `crates/ubnfc-front/src/syntax/ast.rs:12` | 対応（A と**同じ型**を返す）`crates/ubnfc-selfhost/src/lib.rs:50` | 対応・owned `rust/unlaxer-ubnf/src/ast.rs:1-210` | 対応・record/sealed `UBNFAST.java:13` |
| CST 保持 | 非対応 | 非対応 | 非対応（README「syntax AST のみ」） | **対応** — combinator の `Token` 木 |
| 診断 | 位置 + expected 集合 `syntax/mod.rs` | 位置 + expected 集合 | 位置 + line/column、**expected 集合なし・最遠位置なし・回復なし** `rust/unlaxer-ubnf/src/lib.rs:10-32` | **位置を公開しない**（先頭 80 文字の失敗文のみ） |
| エラー回復 | 非対応 | 非対応 | 非対応（README 明記） | 非対応 |
| 位置の単位 | UTF-8 byte | UTF-8 byte | byte と Unicode scalar の両方 `ast.rs:3-20` | 不明（公開しない） |
| 入力末尾の検査 | する | する | する | **しない**（prefix 受理。Part 3 で実測） |
| 再帰上限 | `maxDepth` 既定 10,000 | 同左 | 128、超過は `NestingLimit` 診断 `src/lib.rs:8` | JVM stack 依存 |
| 構文カバレッジ | UBNF v1 + v2 | v1 の一部（既知拒否 6 件） | v1（token 11 / annotation 18 / element 9） | v1（既知欠陥 4 件） |

**A と B は同じ AST 型を返す**（`ubnfc_selfhost::parse_file -> Result<UbnfFile, SelfhostError>`、
`UbnfFile` は `ubnfc_front::syntax::ast::UbnfFile`）。だから両者は span まで含めて `assert_eq!` で比較できる。
C と D は別々の AST 型だが、`rust/unlaxer-ubnf/tests/support/canonical.rs` の span 抜き正準形で比較できる。

### 1.2 パーサ生成系としての比較（ubnfc 2 backend / unlaxer 2 backend）

行 = 能力、列 = 生成系。**旧 Java** は unlaxer-dsl の Java ジェネレータ群、
**旧 Rust** は unlaxer-dsl の `codegen/rust` + `unlaxer-runtime`。

| 能力 | ubnfc Rust | ubnfc Java | 旧 Rust (unlaxer-runtime) | 旧 Java (unlaxer-common) |
|---|---|---|---|---|
| 型付き AST | 対応 `crates/ubnfc-rust/src/ast.rs:26-43` | 対応（sealed interface + record）`ubnfc-java/.../JavaEmitter.java:152` | 対応 `tinyexpression-rs/src/generated/ast.rs` | 対応 |
| **CST / parse tree** | **非対応（明示的な非ゴール）** `docs/design/03-design-v1.md:13` | 同左 | ランタイムに `Node/Tree` はあるが mapper 後は捨てる `unlaxer-runtime/src/lib.rs:338-365` | **対応** — `Token` 木を保持し `Parsed.getRootToken(true)` で取れる |
| capture 出現順 | 対応（IR に `occurrence_order` / `completion_order`）`crates/ubnfc-front/src/ir/types.rs:678` | 同左（front 共有） | 対応 `capture_spans(name) -> &[Span]` `unlaxer-runtime/src/lib.rs:1182-1186` | 対応 |
| 同名のネスト capture | 対応（内側→外側の完了順をテストで固定）`crates/ubnfc-front/tests/lower_semantics.rs:42-82` | 同左 | 対応 `unlaxer-generator/tests/nested_capture_shapes.rs` | 対応 |
| cardinality | `scalar/optional/list` に正規化（D-032）`crates/ubnfc-front/src/ir/types.rs:72-79` | 同左 | 対応 | 対応 |
| separated list `X % sep` | 対応 `crates/ubnfc-front/src/syntax/parser.rs:667-677` | 同左 | 対応 | 対応 |
| **2 カーソル（consumed / matched）** | **対応・契約として明文化** `docs/design/04-cursor-and-diagnostics-contract.md:5-43`、公開 API `consumed_cp`/`matched_cp` | 同左 | 対応 `position()` / `matched_position()` `unlaxer-runtime/src/lib.rs:1142-1148` | 対応 `TokenKind` 4 値 `unlaxer-common/src/main/java/org/unlaxer/TokenKind.java:7-36` |
| 診断: primary offset | 対応 | 対応 | 対応 | 対応（生成 LSP 側） |
| 診断: farthest / expected 集合 / deepest rule | **対応・3 つとも公開** `crates/ubnfc-rust/templates/api.rs:158-169` | 対応 `ubnfc-java/.../api/Diagnostic.java:3-5` | 対応（`Diagnostics` ポリシー） `unlaxer-runtime/src/lib.rs:286-302` | 対応（`ULX-PARSE-001` + 最遠位置 + expected + 最深ルール） |
| **エラー回復** | 対応 `@recovery(sync\|auto\|skip)`（D-031）`docs/design/03-design-v1.md:177-218` | 対応（`Session#recover`） | **非対応** `rust/FULL-SPEC.md`「recovery: 未対応」 | 対応（`@recovery`） |
| 二段診断（fast / detailed 再解析） | 対応・`const DIAG` 単相化つき（D-023/D-036） | 対応（単相化は無し） | 近い機構あり（`DETAILED_ON_FAILURE`） | あり |
| **宣言的スコープ** `@scopeTree`/`@declares` | 対応（enter/leave・宣言・参照・未定義 WARNING、undo ログ、memo hit 時の効果再生）`docs/design/03-design-v1.md:149-161` | **対応かつ Rust より前** — `resolved()`（現在の lookup 表）を返す `ubnfc-java/.../internal/Session.java:912` | 対応 `unlaxer-runtime/src/scope.rs:8-221` | 対応 |
| 未定義シンボル警告 | 対応（WARNING で構文成功） | 対応 | 対応 `SymbolDiagnostic(Severity)` | 対応 |
| `@backref` の意味分割 `@symbolRef`/`@sameText` | **対応（UBNF v2 / D-046）** `docs/design/03-design-v1.md:368` | 同左 | 非対応（`Backref{name}` 一種） | 非対応 |
| **evaluator 生成** | **部分的 — 骨格のみ**。`@eval(strategy=auto)` の型だけ本体生成、他は未実装 callback `crates/ubnfc-rust/src/evaluator.rs:9-40` | 同左 + `debugStrategy` フック `JavaEmitter.java:201-247` | **対応（範囲を明示）** — f32/bool/String、四則・比較・論理・`if`/三項/`match`。変数・関数・object は明示的に非対応 `tinyexpression-rs/src/evaluator.rs:12-140` | **完全** — 変数・関数・日時・UDF・外部呼び出しまで（tinyexpression 本体の Calculator 群） |
| catalog メタデータ | 対応 `crates/ubnfc-rust/src/metadata.rs:4-38` | 対応 `ubnfc-java/.../driver/Driver.java:23` | 対応（静的 `CATALOGS`） | 対応（LSP の補完/ホバーに接続 `LSPServerEmitter.java:432-490`） |
| **LSP サーバ生成** | **非対応（明示的な非ゴール）** `docs/design/03-design-v1.md:38` | 同左 | 非対応 `rust/FULL-SPEC.md` | **対応** `LSPGenerator`/`LSPServerEmitter`/`LSPLauncherGenerator` |
| **DAP（デバッガ）生成** | **非対応（非ゴール）** `docs/design/00-design-v0.md:30` | 同左 | 非対応 | **対応** `DAPGenerator`/`DAPProtocolEmitter`/`DAPRuntimeEmitter` |
| **IDE 統合（VSCode 拡張）** | **無し** | 無し | 無し | **3 本ある** — `unlaxer-dsl/ubnf-vscode`, `unlaxer-dsl/tinycalc-vscode`（DAP smoke test つき）, `tinyexpression/tools/tinyexpression-p4-lsp-vscode`（vsix 同梱） |
| **インクリメンタル parse** | **非対応**（repo 全体に語すら無い） | 非対応 | 非対応 `rust/FULL-SPEC.md` | **対応（生成 LSP のみ）** `IncrementalParseCache` / `parseDocumentIncremental()` `LSPServerEmitter.java:39,206-215` |
| ストリーミング parse | 非対応（入力全体を一度受け取る設計） | 非対応 | 非対応 | 非対応 |
| 直接左再帰 | 対応（限定形、D-016） | 同左 | 対応（canonical 形） | 対応 |
| **v2 左再帰ルール（候補順＝優先順位、ANTLR4 方式）** | **対応（UBNF v2 / D-039）** `docs/design/06-ubnf-v2.md:26-67`、`crates/ubnfc-front/src/desugar.rs` | 同左 | 非対応 | 非対応 |
| 間接左再帰 | 明示拒否 `crates/ubnfc-front/src/lower/recursion.rs:113` | 同左 | 不明 | 不明 |
| precedence climbing | **未実装**（D-004 は M5 で条件付き採用、現状は段別生成） `docs/decisions/D-004-precedence-climbing.md` | 同左 | 非対応 | 非対応 |
| `@precedence(level)` の値を読む | **読まない**（検証と metadata のみ）`docs/design/07-annotation-inventory.md:72` | 同左 | 不明 | 不明 |
| **パラメータ化ルール `Name<P…>`** | **対応（UBNF v2 / D-043・D-044、単相化）** `crates/ubnfc-front/src/templates.rs`（667 行）+ `tests/templates.rs`（370 行） | 同左（front が展開して消すので backend は素通し） | 非対応（UBNF v1 に構文が無い） | 非対応 |
| memo（packrat） | 対応・既定 ON。キー `(ExprId, consumed, matched, mode, stateVersion)` `docs/design/03-design-v1.md:139-148` | 対応 `ubnfc-java/runtime-template/.../MemoTable.java`（boxing なし long[]） | 対応・`SafeFailures` `unlaxer-runtime/src/lib.rs:275-322` | 対応・**既定 OFF**、`SAFE_FAILURES` + 成功メモ化 |
| lexical cache（位置ごとの結合 DFA） | 対応 `crates/ubnfc-rust/templates/lexcache.rs` | **非対応**（grep ヒット 0） | 非対応 | 非対応 |
| **生成物の依存** | **std のみ**、`#![forbid(unsafe_code)]`。`no_std` ではない | **JDK 21（`java.base`）のみ**。`REGEX` 使用時のみ `java.util.regex` | `unlaxer-runtime` crate 1 つ | `unlaxer-common` + `unlaxer-dsl` + jackson + jsonic + JVM |
| ターゲット言語 | Rust / Java の 2 つ | 同左 | — | Java（8 ジェネレータ）+ Rust（実験的 5 モジュール） |
| 入力言語 | UBNF のみ | 同左 | — | UBNF のみ |
| `@import` | front が単一シンボル表へ統合、**backend は無視** `docs/design/07-annotation-inventory.md:90` | 同左 | 保持のみ | 対応 |
| `token X = extern '<id>'` / 言語中立語彙 36 名 | **対応（UBNF v2 / D-041）** `docs/design/06-ubnf-v2.md:187-211` | 同左 | 非対応 | 非対応 |
| 到達不能候補の静的検出（4 警告コード） | **対応（D-037、v1 でも有効）** `crates/ubnfc-front/src/analysis/reachability.rs` | 同左 | 非対応 | 非対応 |
| リテラルの単語境界（既定 `word`） | **対応（UBNF v2 / D-040）** `docs/design/06-ubnf-v2.md:88-134` | 同左 | 非対応 | 非対応 |
| 未知の注釈・設定の警告 | **対応（D-045）** | 同左 | 非対応 | 非対応 |
| 意味述語 `{...}?` | **入れない方針**（memo・候補除外・字句キャッシュの健全性が「終端＝純粋述語」に依存）`docs/design/06-ubnf-v2.md:183-186` | 同左 | 非対応 | 非対応 |
| railroad 図 / UBNF→BNF 変換 | 無し | 無し | — | **対応** `tools/railroad/RailroadMain`、`tools/bnf/UBNFToBNFMain` |
| 生成物の byte 再現性テスト | **対応** — `tests/regenerate.rs`（selfhost / p4-rust）、`scripts/generate.sh --check`（p4-java） | 同左 | `check-generated.sh`（ただし現在 revision 不一致で落ちる） | 不明 |

### 1.3 UBNF v2 で入った機能（決定番号つき）

v1 でも有効なものと v2 限定を分ける。

| 機能 | 決定 | 有効な version |
|---|---|---|
| バージョン門 `@ubnf: 1\|2` + IR の `languageVersion` | D-038 | v2 で門、既定 v1 |
| 到達不能候補の検出（4 警告） | D-037 | **v1 でも有効**（警告のみ） |
| 左再帰ルール（候補順＝優先順位） | D-039 | **v2 のみ** |
| リテラルの単語境界 `@literalBoundary` | D-040 | v2 既定 `word` / v1 既定 `none` |
| 言語中立 token 語彙 36 名 + `extern '<id>'` | D-041 | 純粋追加、v1 でも使える |
| 旧綴り警告 `W-TOKEN-HOST-CLASS` / 未登録 extern の拒否 | D-042 | v2 のみ |
| パラメータ化ルール `Name<P…>` | D-043 | **v2 のみ** |
| `[ X ]` と `X?` を構文木の事実で区別 | D-044 | 全 version（内部） |
| 未知の注釈・設定の警告 | D-045 | 注釈は全 version、未知設定は v2 のみ |
| `@backref` → `@symbolRef` / `@sameText` | D-046 | v2 の新綴り |
| `@rightAssoc` + 規則参照演算子の不具合修正 | D-047 | v1 の不具合修正 |
| `@interleave` の非推奨警告 | D-048 | v2 のみ |
| `@predictiveChoice` の非推奨警告（`@longestChoice` は残す） | D-049 | v2 のみ |

`@ubnf: 2` を宣言しても v2 機能を使わなければ IR は 1 byte も変わらない
（`docs/design/03-design-v1.md:360`、テスト `crates/ubnfc-front/tests/ubnf_v2.rs:276`）。

### 1.4 見出し — 誰が何を独占しているか

- **ubnfc だけが持つもの**: 自己ホスト生成 frontend（B）とその byte 一致固定点、UBNF v2 の
  左再帰ルール / パラメータ化ルール / 言語中立 token 語彙 / 単語境界 / 到達不能候補検出、
  2 カーソルの明文化された契約、生成物の依存ゼロ（std / `java.base` のみ）、
  2 backend 差分ファジング、lexical cache（Rust のみ）。
- **旧 Java スタックだけが持つもの**: **LSP サーバ生成・DAP 生成・VSCode 拡張 3 本・
  インクリメンタル parse・railroad 図・UBNF→BNF 変換・CST（Token 木）**。
  ubnfc はこのどれも持たず、LSP/DAP は明示的な非ゴール。
- **tinyexpression-rs だけが持つもの**: **動く evaluator**。ubnfc は骨格しか生成しない。
- **旧 Java スタックだけが持つもの（evaluator）**: 変数・関数・日時・UDF・外部呼び出しまでの完全な評価器。

---

## Part 2 — それぞれ「何に対して」正しいのか

この節の結論を先に書く。**10 実装のうち、独立に書かれた仕様から期待値を導いているものは 1 つも無い。**
どれも「別の実装の観測値」か「自分で書いた契約テスト」を基準にしている。
違うのは *どの実装を基準にしたか* と *基準にしたことを明記しているか* の 2 点。

### 2.1 ubnfc（front + Rust backend + Java backend）

**テスト本数（実測）**

| 対象 | 件数 |
|---|---:|
| Rust workspace 全体（`cargo test --workspace --offline -- --list`） | **492** |
| うち `ubnfc-front` | 208 |
| うち `ubnfc-harness` | 83 |
| うち `ubnfc-target-rust`（手書き参照実装） | 68 |
| うち `ubnfc-rt-template` | 55 |
| うち `ubnfc-fuzz` | 29 |
| うち `ubnfc-rust` | 26 |
| うち `ubnfc-p4-rust` | 15 |
| うち `ubnfc-selfhost` | 8 |
| Java backend（`ubnfc-java/src/test/java/org/ubnfc/java/`、15 ファイル） | **77** |
| 適合性コーパス（`cargo test` には入らない。`ubnfc-harness run-all`） | 127 ファイル / **1,212 ケース** → 文法 variant 展開後 **1,425 件** |
| うち ubnfc 自前の捕獲物 `corpus/oracle/*.json` | 107 ファイル / **398 ケース** |

**期待値の出どころ — 旧 Java 実装の観測値**

`corpus/oracle/README.md` が自分で書いている:

> `unlaxer-parser` の実装を実際に生成・コンパイル・実行した観測値を固定したもの。
> UBNFC の生成器・runtime・期待値は採取に使用していない。互換性の参照データであり、
> **疑わしい値を望ましい挙動へ修正していない**。

- 採取元 revision: `ed4178e673f93e719b62a7bd181f3a219abf8ed1`（全 JSON の `source` に記録）
- 手順: 旧実装の `CodegenMain.run` → `javax.tools.JavaCompiler` → `URLClassLoader` → fresh `ParseContext`。生成 Java を一切修正しない。
- 再現性: 別 scratch で完全再採取し、63 + 44 JSON の **byte 一致**を確認済み。
- `reobserved-*.json`（36 ケース）は **Java/Rust の突き合わせではない**。旧実装を別ドライバで
  再採取した自己整合チェック。README 自身が「採取成功を backend の比較成功とはみなさない」と書いている。

上流コーパス 127 ファイルも unlaxer-parser のテストリソースをそのまま読んでいる。
**ubnfc が独立に導出した期待値は `ubnfc-front` 208 / `ubnfc-rt-template` 55 の自前契約テストと、
D-018 の新契約 4 つ（うち実装済みは `comment-trivia` 1 つだけ）に限られる。**

**最新のコーパス結果**（`docs/reports/2026-09-23-{java,rust}-corpus-contract.md`）

| backend | Pass | 期待拒否 Pass | Known（既知乖離） | Fail | 未観測 |
|---|---:|---:|---:|---:|---:|
| Java | 1,310 | 28 | 28 | **0** | 59 |
| Rust | 1,292 | 30 | 28 | **0** | 75 |

「未観測」は欠落した観測項目であって Pass ではない、と harness が明記している。
`scope-annotations` 系統 14 件は **Rust 側が丸ごと未観測**（Rust runtime が `scope/resolved` を返さない）。

**クロス実装チェック — 本物は差分ファジングだけ**

`ubnfc-harness run-all` は「Java を oracle と比べる」「Rust を oracle と比べる」を別々にやるだけで、
両 backend を直接突き合わせてはいない。しかも D-011（診断 offset）と D-022（数値 capture）では
**backend 別 oracle** を使う。

直接比較は `crates/ubnfc-fuzz` の差分ファジング（`docs/reports/2026-09-23-differential-fuzzing.md`）:

| 項目 | 値 |
|---|---:|
| 生成文法（seed 1..320） | 320 |
| 両 backend で build 通過 | 292（discard 28 = 9%） |
| 比較件数（文法 × 入力） | 7,008 |
| 差分クラス | 92（根本原因で束ねると 12） |
| 所要 | 1,121 秒 |

比較する wire 項目は `accepted`/`allConsumed`/`consumed`/`matched`/`ast`/`captures`/`declarations`/
`references`/`scope`/`diagnostics`/`mappingError`/`recoveries`/`catalogs`/`value` ほか。
**キーの欠落と明示 null を区別する**。最小再現は `crates/ubnfc-fuzz/findings/` に 41 ディレクトリ。

**既知の乖離台帳 — 82 件（30 ケース / 17 ファイル）**

`corpus/oracle/divergences.json`。**すべて「旧 Java 実装 ↔ ubnfc（両 backend とも同じ値）」の乖離**で、
ubnfc の 2 backend 間の乖離ではない。分類（根本原因＝旧実装の疑義番号別）:

| 根本原因 | 件数 | decision |
|---|---:|---|
| 疑義 1: REGEX の UTF-16 / code point 混在 | 23 | follow-spec |
| 疑義 9: `@comment` の line/block 設定が読まれない | 22 | follow-spec 13 + `new-contract:comment-trivia` 9 |
| 疑義 2/3/4/11: recovery（非 BMP 過剰消費・root 無効・AUTO 過剰消費・AST 投影） | 17 | follow-spec 12 + recovery-projection 4 + recovery-diagnostics 1 |
| 疑義 6: import（不正な Java 識別子出力・多段統合欠落・CLI exit 0） | 7 | follow-spec 6 + import-resolution 1 |
| 疑義 7: enum capture の mapper 型不一致 | 3 | follow-spec |
| 疑義 8: frontend が拒否すべきものを javac に落とす | 3 | follow-spec |
| 疑義 5: `@recovery(skip)` の runtime API 不整合 | 2 | follow-spec |
| 疑義 10: predictive choice の空入力 NPE | 2 | follow-spec |

**D-017 は「旧実装のバグは再現しない」**。さらに **D-021 でその逆向きも能動的にテストしている**:
harness は `follow-spec` 登録済み観測が旧実装と一致してしまったら **Fail「既知バグ再現」**として落とす
（`comparison.rs reproduced_follow_spec`）。単なる免除リストより一段強い。

> harness README の免責をそのまま引く:
> 「既知判定は仕様への適合を証明しない。登録済み観測群で oracle と異なることの分類であり、
> 将来の runner の仕様テストと併用する。`undecided` は免除しない。」

**self-host 固定点**（`crates/ubnfc-selfhost/tests/{parity,regenerate}.rs`、8 テスト）

- 手書き frontend と生成 frontend で `ubnf.ubnf` の AST が **全 span 込みで一致**
- 25 ソース中 **19 件が完全一致、6 件は既知拒否**（byte offset まで固定してアサート）
- `regenerate.rs` は真の固定点: self-host IR が `ir/fixtures/*.ir.json` と byte 一致し、
  その IR を `ubnfc_rust::emit` に通した全ファイルが `src/generated` と byte 一致

既知拒否 6 件（`docs/design/05-selfhost.md`）: dotted mapping class / `@eval` の `=` 区切り /
annotation keyword の識別子境界 / 3 成分以上の namespace / setting 内の二重引用符 /
`@declares` の非 STRING capture 名。いずれも **手書き frontend だけの互換拡張**で、
self-host を通すための文法・IR 加工はしていない。

**カバーされていないもの（ubnfc 自身の申告）**

- `docs/compat-matrix.md` に「oracle なし」が **395 箇所**。58 行中 **19 行は全 12 観測面が oracle なし**。
- oracle 採取時に 168 ケース中 **15 ケースが runtime 値を持たない**（生成 Java のコンパイル失敗 14 + 循環 import 1）。
  このとき capture・scope・diagnostics の空配列は「空だった」という実行 oracle ではない。
- 差分ファジングが見ていない範囲（レポート §6 の自己申告）: **両 backend が同じように間違っている場合**、
  左再帰の runtime 差、`REGEX` token、`@import` の非最小形、大入力、`@root` 以外の entry。
- 上の穴と「コーパスが旧実装の写経」であることを重ねると、
  **旧実装と ubnfc の両 backend が同じように間違っている場合は誰も検出できない**。

### 2.2 unlaxer-ubnf（C）

- テスト: `rust/unlaxer-ubnf/tests/frontend.rs` の **10 `#[test]`**。fixture は `positive/` 4 + `known/` 4 + `negative.tsv` 24 行。
- 期待値は **Java bootstrap の観測を基準にしつつ、4 件は意図的に反転**している。
  README の表が「Java bootstrap の観測結果」と「Rust の契約」を 2 列で並べ、差を消していない:

| fixture | Java bootstrap | Rust の契約 |
|---|---|---|
| `eval-default` | `@eval(kind=…, strip_prefix='$')` の strategy を `$` と誤認 | strategy は `default`、`strip_prefix` は独立 param |
| `namespace` | `a.b.Value` を namespace=`a`, name=`b` に切り詰め | namespace=`a.b`, name=`Value` |
| `keyword-boundary` | `@rooted` が `@root` の prefix に奪われ失敗 | `Simple{name:"rooted"}` |
| `common-field-space` | `@commonField(left, right)` のカンマ後空白で失敗 | 通常 trivia として許容 |
| `trailing-input` | 最初の grammar 後の garbage を無視して prefix 成功 | 全入力検査して拒否 |
| 診断位置 | 先頭 80 文字の失敗文のみ、正確位置を非公開 | 位置同値は主張しない |

- **クロス実装チェックあり**: `unlaxer-dsl/src/test/java/org/unlaxer/dsl/codegen/RustUbnfFrontendConformanceTest.java`。
  `rustc` で `unlaxer-ubnf` をビルドし `examples/inspect` を叩き、`UBNFMapper.parse` の正準 JSON と `assertEquals`。
  **ただし既定でスキップ**（`assumeTrue(Boolean.getBoolean("rustConformance"))`、同ファイル L27）。
- 穴: expected 集合・最遠位置・エラー回復が無い。import を解決しない。構文受理は backend 対応を意味しない。

### 2.3 Java bootstrap（D）

- `unlaxer-dsl/src/test/java/org/unlaxer/dsl/` に **1,004 `@Test` / 108 ファイル**。
  UBNF を直接扱うのは `UBNFParsersTest` 57 / `UBNFMapperTest` 35 / `UBNFIdentifierBoundaryTest` 6。
- `unlaxer-common` に 706 テスト / 127 ファイル。
- 期待値は自前（この文法言語の定義実装だから当然）。**ただし上の 4 欠陥 + prefix 受理はここに残っている**。
  つまり「自前の期待値」が欠陥を固定してしまっている領域がある。
- クロス実装チェックは上記 `RustUbnfFrontendConformanceTest`（既定スキップ）のみ。

### 2.4 tinyexpression-rs（R2）

- テスト: `rust/tinyexpression-rs/tests/` の **32**（`parser_cli` 20 / `numeric_evaluator` 9 / `scalar_evaluator` 3）。
  基盤の `unlaxer-runtime` に 8 ファイル 2,112 行、`unlaxer-generator` に 10 ファイル 1,743 行。
- **クロス実装チェックあり・双方向の共有 fixture**。fixture の所在は **Rust 側**で、Java がそれを読む:
  `rust/tinyexpression-rs/tests/fixtures/{numeric-f32.tsv(14), scalar-control.tsv(56), root-expression.tsv(37),
  typed-hint-errors.tsv(8), strict-match-errors.tsv(3), valid-*.tiny, invalid-syntax.tiny}`。
  Java 側の読み手は `tinyexpression/src/test/java/org/unlaxer/tinyexpression/p4/` の
  `P4RustSharedFixtureAcceptanceTest`(2) / `P4RustNumericEvaluatorOracleTest`(1) /
  `P4RustScalarEvaluatorOracleTest`(1) / `P4RustRootExpressionAcceptanceTest`(3)。
- 期待値の性質: README は `numeric-f32.tsv` の期待 f32 raw bits を **独立に書いた値**と主張し、
  Rust library/CLI と Java `P4TypedAstEvaluator` の双方が同じ値に照合される。
  `root-expression.tsv` は Java の semantic root と Rust の Formula expression root を
  **別カラムで**持ち、差を隠していない。
- **ただし受理テスト `P4RustSharedFixtureAcceptanceTest` は既定スキップ**
  （`-Dtinyexpression.rust.shared=true` が要る、同ファイル L31-33）。f32/scalar の oracle テストは常時走る。
- 穴: エラー回復・LSP/DAP・インクリメンタルは未対応と `rust/FULL-SPEC.md` が明記。
  evaluator は変数・関数・object を明示的に非対応。
- **生成物 drift 検査は現在落ちる**: `tinyexpression/rust/check-generated.sh` が期待する
  unlaxer revision は `6a4187c` だが、`unlaxer-parser` の HEAD は `ed4178e`。

### 2.5 旧 Java P4（J2）

- `tinyexpression/src/test/java/` に **634 `@Test` / 149 ファイル**。うち P4 パッケージが 70 / 8 ファイル。
- 期待値は自前。`docs/feature-parity-diff.md`（手書きパーサ vs UBNF P4 の PARITY/HAND-ONLY/UBNF-ONLY/PARTIAL 表）と
  `docs/backend-coverage-matrix.md` で自分の穴を表にしている。
- **これが workspace で唯一「本番で使われている」P4 実装**であり、他の 3 つ（R1/J1/R2）はいずれも
  その互換性の検証例。R1/J1 の README はどちらも「P4 の完全互換を達成したものではない」と冒頭に書いている。

### 2.6 ubnfc P4 例（R1 / J1）

- R1: `examples/p4-rust/tests/` 15 テスト。16 fixture の受理・消費量・canonical AST golden を固定し、
  `tests/regenerate.rs` が IR → emitter の byte 一致を検証。
- J1: `examples/p4-java` の JUnit。`scripts/generate.sh --check` が生成 22 ファイルの byte 一致を検証。
- 旧実装との差は `examples/p4-java/evidence/*differences.json` に機械可読で保存されており、
  **span の単位や引用符を正規化して差分を消す処理は行わない**と README が明記。

---

## Part 3 — 実測クロスチェック

紙の上の比較で終わらせず、同じ入力を実際に 4 実装へ流した。
ドライバは本報告のために新規追加した 2 つで、既存のパーサ・生成器のソースは変更していない。

- `examples/ubnf-crosscheck/` — UBNF を読む 4 実装（A/B/C/D）
- `examples/p4-crosscheck/` — P4 を読む 4 実装（R1/J1/R2/J2）

### 3.1 UBNF: 1 文法・4 パーサ

**入力 48 件** = `.ubnf` ファイル 25 件（unlaxer-parser 配下の全 24 件 + tinyexpression の P4 LSP 文法 1 件）
+ `rust/unlaxer-ubnf/tests/fixtures/negative.tsv` の不正入力 23 件。
`tinyexpression/docs/ubnf/` の草稿 3 件は最終版ではないので入れていない。

**比較の作り方（正規化を隠さない）**

- A と B は同じ `UbnfFile` 型を返すので生でも比較できるが、C/D と並べるため
  3 系統すべてを `rust/unlaxer-ubnf/tests/support/canonical.rs` の span 抜き正準形へ射影した。
- ubnfc の構文木は量指定子を**要素の属性**として持ち、unlaxer 系は**要素種別**として持つ。
  そこだけ機械的に写した（`X?`→`OptionalElement(body[[X]])`、`X*`→`RepeatElement`、
  `X+`→`OneOrMoreElement`、`X{n,m}`→`BoundedRepeatElement`、`X % s`→`SeparatedElement`）。
  **差を消す目的の正規化は他に入れていない。**
- D の正準化は `RustUbnfFrontendConformanceTest.canonical()` と同じ反射アルゴリズムを
  `examples/ubnf-crosscheck/java/UbnfProbe.java` に写したもの。忠実さの裏付けは
  「C と D が、記録済みの 2 件を除く全入力で完全一致した」こと（下記）。

**結果（1）受理判定の不一致 21 件 / 48**

分類すると 4 群になる。

| 群 | 件数 | 内容 |
|---|---:|---|
| ① ubnfc が構文で受け validate でも落とさない | 11 | 下表 |
| ② ubnfc 自己ホスト（B）だけが落ちる | 5 | 既知の self-host 乖離（`docs/design/05-selfhost.md`） |
| ③ Java bootstrap（D）だけが落ちる／通る | 3 | 既知の Java 欠陥（`rust/unlaxer-ubnf/README.md`） |
| ④ 全員が落ちるが理由が違う | 2 | `tinycalc.ubnf`（二重引用符）ほか |

**① が本報告で見つかった ubnfc 側の穴**。`ubnfc validate --strict` まで通しても診断が出ない:

| 入力 | ubnfc A/B | C unlaxer-ubnf | D Java | `ubnfc validate --strict` |
|---|---|---|---|---|
| `grammar G { token T=CHAR_RANGE('','a') R ::= T; }` | 受理 | 拒否 | 拒否 | **panic（exit 101）** `crates/ubnfc-front/src/lower/tokens.rs:81` — `min.chars().next().unwrap()` が `None` |
| `CHAR_RANGE('ab','z')`（境界が 2 文字） | 受理 | 拒否 | 拒否 | exit 5（`W-GENERAL-NO-ROOT` のみ。**先頭 1 文字を黙って採る**） |
| `CHAR_RANGE('😀','😇')`（非 BMP） | 受理 | 拒否 | 拒否 | exit 5（診断なし。`char as u64` がそのまま通る） |
| `CHAR_RANGE('z','a')`（逆順） | 受理 | 拒否 | 拒否 | exit 5（診断なし） |
| `'x'{2147483648}`（境界が int 超過） | 受理 | 拒否 | 拒否（`NumberFormatException`） | exit 5（診断なし） |
| `@precedence(level=2147483648)` | 受理 | 拒否 | 拒否 | exit 1（`@precedence` の別警告が出るだけで値は検査されない） |
| `@recovery(magic)`（未知の mode） | 受理 | 拒否 | 拒否 | exit 5（`@ubnf: 2` + `--strict` でも診断なし） |
| 空ファイル / `grammar G {}` / `R ::= ;` / `R ::= 'a' \| ;` / `R ::= ();` | 受理 | 拒否 | 拒否 | exit 0〜5、該当診断なし |

最後の 5 つ（空構文）は **ubnfc の意図**とみてよい（`crates/ubnfc-selfhost/tests/parity.rs:56-63` が
`""`, `"grammar G {}"`, `"grammar G { R ::= [ ] | { } | ( ); E ::= ; }"` を通す入力として明示列挙している）。
残る 7 件、特に **`CHAR_RANGE('','a')` の panic は不具合**で、不正入力でプロセスが落ちる。
`@recovery(magic)` と 2 つの整数オーバーフローは、D-045（未知注釈の警告）が導入されたあとも
値の検査が無いまま通る。

**結果（2）受理したのに AST が食い違う入力 3 件 / 48**

| 入力 | 一致群 | 差分 |
|---|---|---|
| `known/eval-default.ubnf` | A / C / D がすべて別 | ① A の `EvalAnnotation` は `[KeyValuePair…]` 形、C/D は `(kind, strategy, params)` 形（**設計差**、射影で埋めていない）。② **C=`default` vs D=`$`** — Java bootstrap が `strip_prefix='$'` を strategy と誤認する既知欠陥を独立に再現 |
| `known/namespace.ubnf` | **A+C** / D | `a.b.Value` の namespace が A=C=`a.b`、**D=`a`** — Java bootstrap の切り詰め欠陥を独立に再現 |
| `positive/annotations.ubnf` | A / **C+D** | 差は ① の `EvalAnnotation` 形だけ |

**A（ubnfc 手書き）と C（unlaxer-ubnf）は、`@eval` の AST 表現という設計差を除けば、
共通に受理した全入力で正準 AST が完全一致した。** 別々に書かれた 2 つの UBNF パーサが
`ubnf.ubnf` を含む 28 文法で構造一致するのは、両者にとって強い結果。

**結果（3）拒否位置の不一致 2 件**

| 入力 | A/B（ubnfc） | C（unlaxer-ubnf） |
|---|---:|---:|
| `grammar G { @unknown(x) R ::= 'x'; }` | byte 20 | byte 12 |
| `grammar G { token T=UNKNOWN('x') R ::= T; }` | byte 27 | byte 20 |

ubnfc は違反した引数・構成子の位置、unlaxer-ubnf は注釈／token 宣言の先頭を指す。
どちらが正しいという契約は無い（D は位置を公開しないので比較不能）。

**結果（4）ubnfc 内部の新しい parity 差 — B がブロックコメントを受理する**

```
入力: grammar G { /* no */ R ::= 'x'; }
A ubnfc 手書き : 拒否 @12「UBNF のコメントは // 行コメントのみです」
B ubnfc 自己ホスト: 受理。しかも /* no */ が無い場合と同じ AST を返す
C unlaxer-ubnf : 拒否「UBNF block comments are unsupported; use // comments」
D Java bootstrap: 拒否
```

`crates/ubnfc-selfhost/tests/parity.rs` の既知乖離 6 件は**すべて「B が拒否し A が受理する」向き**で、
**逆向き（B が受理し A が拒否する）はこれまで記録が無い**。本報告の追加入力で初めて出た。

原因は特定できた。`unlaxer-dsl/grammar/ubnf.ubnf:18` が `@whitespace: javaStyle` を宣言しており、
`crates/ubnfc-front/src/lower/trivia.rs:95-98` は javaStyle に対して
`BlockComment { open: "/*", close: "*/" }` を無条件で delimiter 集合へ入れる。
一方 `crates/ubnfc-front/src/syntax/lexer.rs:89` は `/*` を見たら専用メッセージで拒否する。
つまり **自己記述文法が記述している言語が、それを記述している実装より広い**。
`docs/design/05-selfhost.md` の既知乖離一覧に 7 件目として追記すべき差分で、
直し方は 2 つ（`ubnf.ubnf` の `@whitespace` を `none` + `@comment{line:'//'}` にするか、
手書き lexer をブロックコメント受理に合わせるか）。どちらを選ぶかは self-host の契約の問題。

**結果（5）Java bootstrap の prefix 受理を独立に再現**

`grammar G { R ::= 'x'; } garbage` を D だけが受理する（A/B/C は byte 25 で拒否）。
`rust/unlaxer-ubnf/README.md` が「Java は最初の grammar 後の garbage を無視して prefix 成功」と
書いているとおりで、既存の conformance テストが既定スキップである以上、**この差は日常的には誰も見ていない**。
同様に `@commonField(left, right)`（カンマ後空白）と `@rooted` の keyword boundary も D だけが落ち、
記録済みの Java 欠陥 4 件が 4 件とも再現した。

### 3.2 P4: 1 文法・4 パーサ

入力は `examples/p4-rust/tests/fixtures/*.tiny` の 16 件。
生の結果は `examples/p4-crosscheck/evidence/report.md`。

**結果（1）受理**

| fixture | R1 ubnfc-rust | J1 ubnfc-java | R2 tinyexpression-rs | J2 旧 mapper | J2 旧 facade |
|---|---|---|---|---|---|
| `valid-basic` / `valid-multiline` / `valid-unicode` / `flat-arithmetic` | 受理 | 受理 | 受理 | 受理 | 受理 |
| `complex` / `large-match` | 受理 | 受理 | 受理 | 受理 | **10 秒 deadline 超過** |
| `complex-x4` / `complex-x16` | 受理 | 受理 | 受理 | 受理 | 10 秒 deadline 超過 |
| `complex-x64` | 受理 | 受理 | 受理 | 45 秒 timeout（未取得） | 10 秒 deadline 超過 |
| `comparison-heavy`(×1/x4/x16/x64) | 拒否 | 拒否 | **受理** | 拒否（105 ms、文法的拒否） | x1 のみ受理、他は deadline |
| `invalid-syntax` / `complex-half` / `complex-tail` | 拒否 | 拒否 | 拒否 | 拒否 | 拒否 |

- **意図的な不正入力 3 件は 5 経路すべてが拒否**した。
- **comparison-heavy 4 件で R2 だけが受理する。** これは実装の優劣ではなく **入口の違い**。
  R1/J1/J2-mapper は `Formula` root 固定、R2 は旧 facade と同じ **result family ごとの再解析**
  （number/string/boolean/object の候補 root を公開ディスパッチ順に試す）を持つ。
  R1 は `parse_entry` で `BooleanExpression` を指定すれば 4 件とも全消費する（`examples/p4-rust/README.md`）。
  **ubnfc には「公開入口で root 候補を選ぶ」層が無い**（Part 4 参照）。
- **旧 facade（tinyexpression の公開入口）は `complex.tiny`（325 code point）で自分の 10 秒 deadline を超える**
  （`P4PreferredAstMapper$ParseDeadlineExceededException`、`P4PreferredAstMapper.java:411`）。
  表の「deadline 超過」は文法的拒否ではなく資源の失敗なので、拒否と混ぜて読まないこと。
  `complex-x64` の mapper 欄の「未取得」も 45 秒 timeout（`scripts/legacy.sh` の上限）であって拒否ではない。

**結果（2）canonical AST の差 — 3 種類しか出なかった**

正規化を一切せず全差分を数えた。差は次の 3 クラスに尽きる。

| 比較 | 差分の総数（受理が重なった fixture 上） | 内訳 |
|---|---:|---|
| **J1 ubnfc-java vs R1 ubnfc-rust** | 256 | **引用符 256**。木の形・span の差はゼロ |
| **J2 旧 mapper vs R1 ubnfc-rust** | 4 | **span 4**（`complex` 1 + `valid-unicode` 3）。引用符・木の形の差はゼロ |
| **J2 旧 mapper vs J1 ubnfc-java** | 68 | 引用符 64 + span 4 |
| **J2 旧 mapper vs R2 tinyexpression-rs** | 625 | **木の形 621 + span 4** |
| **R1 ubnfc-rust vs R2 tinyexpression-rs** | 1,022 | **木の形 1,022** |
| J2 旧 facade vs R1 ubnfc-rust（受理が重なるのは 4 fixture のみ） | 19 | 木の形 16 + span 3 |

1. **引用符（ubnfc の 2 backend 間）**: `'jp'` のような文字列リテラルの value を
   Java backend は引用符ごと、Rust backend は引用符を除いて持つ。
   `examples/p4-rust/README.md` が「Rust の scanner 集合は単引用符を value から除く」と書いているとおりで、
   **生成器ではなく Extern scanner の実装差**。旧 mapper は引用符を除く側（= ubnfc Rust と一致）。
   つまり **ubnfc Java backend だけが旧実装と違う**。

2. **span（旧 Java ↔ ubnfc 両 backend）**: 課題文にあった UTF-16 / code point の取り違えを実測で確認・定量化できた。
   `valid-unicode.tiny`（`'こんにちは😀'`）で旧 mapper は末尾 span を **10**、ubnfc は **9** と報告する。
   😀 は UTF-16 で 2 単位・code point で 1 なので、**旧実装は非 BMP 1 文字あたり span を 1 多く数える**。
   影響は `valid-unicode` で 3 span、`complex` で 1 span（ファイル末尾）。
   非 BMP を含まない 6 fixture では **旧 mapper と ubnfc Rust backend の AST は完全一致（差分 0）**。

3. **木の形（tinyexpression-rs だけが違う）**: 二項演算の最内オペランドの包み方が違う。

   ```
   入力 "1 + 2" の最内左辺
   R1 ubnfc / J2 旧 mapper : BinaryExpr{ left: BinaryExpr{left:null, op:["1"], right:[]}, op:[], right:[] }
   R2 tinyexpression-rs    : BinaryExpr{ left:"1", op:[], right:[] }
   ```

   tinyexpression-rs の mapper は中間の `BinaryExpr` を 1 段畳み、数値 capture を `left` へ直に置く。
   **旧 Java 実装の AST を再現しているのは ubnfc の 2 backend のほうで、
   同じ unlaxer 系列の Rust 移植である tinyexpression-rs ではない。**
   非 BMP しか含まない `valid-unicode` では二項演算の連鎖が無いため差分ゼロになり、この差が
   「演算子連鎖の写し方」に限定されることも裏づけられる。

**結果（3）診断の一致**

`invalid-syntax.tiny` で J1 と R2 がどちらも `offset = 4` を報告した（J1 は
`Diagnostic[kind=syntax, offset=4, farthestCp=4, …]`、R2 は
`{"kind":"syntax","offset":4,"expected":[…25 件…],"farthestOffset":4}`）。
別系列の 2 実装が同じ失敗位置を出しているのは、両者の診断契約が実用上そろっている証拠になる。
J2 は位置を同じ形では公開しないので 3 者比較はできない。

### 3.3 駆動できなかったもの

推測で埋めず、動かせなかったものを列挙する。

- **Operators / Power の 4 実装比較**は行っていない。`examples/handwritten-target/{rust,java}` の
  手書き参照実装には比較用の入出力ドライバ（canonical JSON 出力）が無く、
  `ubnfc-target-rust` の 68 テストは内部 API を直接叩く形。共通の入出力を新設すれば比較できるが、
  本報告の範囲では作らなかった。既存の検証は「手書き参照実装が目標形を定義し、
  ubnfc 生成物がそれに合わせる」という一方向。
- **Java bootstrap（D）の失敗位置**は取れない。`UBNFMapper.parse` は
  `IllegalArgumentException: UBNF パース失敗: <先頭 80 文字>` しか返さず offset を公開しない。
  よって拒否位置の 4 者比較はできず、A/B/C の 3 者に留めた。
- **旧 Java の大入力**（`complex-x64` の mapper、`complex`〜`complex-x64` の facade）は
  資源上限（45 秒 timeout / facade 内蔵 10 秒 deadline）で AST を取れなかった。拒否ではない。
- **ubnfc の Java backend と Rust backend の scope 観測**は 4 者比較の対象にしていない。
  Rust runtime が `scope/resolved` を返さないため、コーパス側でも 14 件が未観測のまま
  （`README.md:200-206`）。ここは差分ファジングでも埋まっていない。
- **evaluator の比較**は行っていない。ubnfc は骨格しか生成しないので、同じ式に対する
  評価値を 4 実装で突き合わせることが原理的にできない（ubnfc 側に評価器の実体が無い）。
  R2 と J2 の間には既に `numeric-f32.tsv` の f32 raw bits oracle がある。

---

## Part 4 — ubnfc が劣っている点（まとめ）

比較が新しい実装を持ち上げるだけでは役に立たないので、劣位を 1 節にまとめる。

1. **LSP サーバ生成・DAP 生成・VSCode 拡張が無い。** 旧 Java スタックは
   `LSPGenerator` / `LSPServerEmitter` / `LSPLauncherGenerator` / `DAPGenerator` /
   `DAPProtocolEmitter` / `DAPRuntimeEmitter` を持ち、VSCode 拡張が 3 本（ubnf / tinycalc / P4）ある。
   ubnfc はこれを**明示的な非ゴール**としている（`docs/design/03-design-v1.md:38`、
   `docs/design/00-design-v0.md:30-31`）。「結果 API に必要な情報は出す」という方針だが、
   実際に LSP を書く手間は利用者に残る。
2. **インクリメンタル parse が無い。** 旧 Java の生成 LSP は `IncrementalParseCache` と
   `parseDocumentIncremental()` を持つ（`LSPServerEmitter.java:39,206-215`）。
   ubnfc は repo 全体に該当する語すら無く、入力全体を一度受け取る前提の設計。
3. **evaluator が骨格どまり。** `@eval(strategy=auto)` の型だけ本体を生成し、他は未実装 callback
   （`crates/ubnfc-rust/src/evaluator.rs:36-40`）。`tinyexpression-rs` は f32/bool/String の
   実際に動く評価器を持ち、旧 Java は変数・関数・日時・UDF・外部呼び出しまで動く。
   コーパスの `/evaluationError` 32 件が未観測なのもここに由来する。
4. **公開入口の root 候補選択が無い。** 旧 facade と tinyexpression-rs は result family ごとに
   再解析して入口を選ぶ。ubnfc は `parse_entry` で利用者が入口名を指定するだけなので、
   3.2 のとおり `comparison-heavy` 系を既定の `Formula` 入口で拒否する。
5. **CST（parse tree）を返さない。** 旧 Java は `Token` 木を保持する。ubnfc は出現表（captures /
   lexical / tokens / nodeSpans）で代替するが、これは同じものではない。
6. **不正な token 宣言で落ちる・黙って通す。** 3.1 の実測:
   `CHAR_RANGE('','a')` で **panic**（`crates/ubnfc-front/src/lower/tokens.rs:81`）、
   `CHAR_RANGE('ab','z')` は先頭 1 文字を黙って採用、`CHAR_RANGE('z','a')`（逆順）・
   非 BMP 境界・`{2147483648}`・`@precedence(level=2147483648)`・`@recovery(magic)` は
   `--strict` でも診断ゼロ。unlaxer-ubnf と Java bootstrap は 7 件とも拒否する。
7. **自己ホスト frontend が手書き frontend より広い言語を受ける。** 3.1（4）のブロックコメント。
   `parity.rs` の既知乖離はこれまで片方向だけ記録されていた。
8. **Java backend が性能目標未達**（`README.md:184-187`。詳細は速度報告）。
   **Java backend に lexical cache が無い**（Rust のみ）。
   **Rust backend は `scope/resolved` を返さない**ため scope 観測 14 件が未観測。
   **Rust backend は非正則 REGEX を生成時エラーにする**（Java は `java.util.regex` へ委譲できる）。
9. **正しさの土台が旧実装の写経である。** コーパス 1,212 ケースも oracle 398 ケースも
   unlaxer-parser 起源で、ubnfc が独立に導出した期待値は front / rt-template の自前契約テストと
   D-018 の新契約 1 つだけ。差分ファジングは「両 backend が同じように間違っている場合」を検出できない。
   ubnfc 自身がこれを繰り返し明記している点は評価できるが、事実としては残る。
10. **railroad 図・UBNF→BNF 変換が無い**（旧 Java は `tools/railroad` / `tools/bnf` を持つ）。
11. **ドキュメントが実装より古い箇所がある。** `README.md:169-170` が
    「パラメータ化ルールは未実装」「アノテーション統廃合は提案のみ」と書いているが、
    D-043〜D-049 は実装済み（`crates/ubnfc-front/src/templates.rs` 667 行 + `tests/templates.rs` 370 行）。

**逆に ubnfc が明確に勝っている点**も 1 箇所にまとめておく:
生成物の依存ゼロ（std / `java.base` のみ）、2 カーソルの明文化された契約と diagnostics の分離公開、
宣言的スコープの undo/replay 契約、UBNF v2（左再帰ルール・パラメータ化ルール・言語中立 token 語彙・
単語境界・到達不能候補検出・未知注釈警告）、自己ホストの byte 一致固定点、2 backend 差分ファジング、
旧バグを「再現しないこと」を能動的にテストする D-021、そして 3.2 で確認した
**旧 Java mapper の AST を（非 BMP span を除いて）完全再現していること**。
最後の点は同じ unlaxer 系列の tinyexpression-rs ができていない。

---

## 再現手順

```bash
# UBNF 4 実装
cd examples/ubnf-crosscheck && bash scripts/run.sh out     # 数分
# P4 4 実装（旧 Java の JVM 起動が 32 回入るので 20 分以上）
cd examples/p4-crosscheck && bash scripts/run.sh out
# ubnfc 側の単発確認
cargo run -q --offline -p ubnfc-front --bin ubnfc -- validate --grammar X.ubnf --strict --format json
```

前提: rustc / cargo（offline 可）、JDK 21、Maven、`~/.m2` の `unlaxer-common` と `unlaxer-dsl` 3.0.15、
`tinyexpression/target/classes` がビルド済みであること。
参照リポジトリ（unlaxer-parser / tinyexpression）は読むだけで、ソースも `target/` も変更しない。

各ドライバの設計と正規化の内訳は
[`examples/ubnf-crosscheck/README.md`](../../examples/ubnf-crosscheck/README.md) と
[`examples/p4-crosscheck/README.md`](../../examples/p4-crosscheck/README.md)。
実行結果は各 `evidence/report.md`。
