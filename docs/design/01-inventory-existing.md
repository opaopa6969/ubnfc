# 既存 unlaxer-parser の棚卸し（codex 調査、2026-09-22）

> 読み取り専用調査の結論。根拠は `unlaxer-parser/` 相対のファイル:行。

## 総括

Java・Rust とも「文法グラフを汎用ランタイムで解釈実行する」方式。専用の解析手続きを
直接生成する経路は存在しない。

## 1. Java 生成物の形

- `XxxParsers.java` に各ルールを `public static class XxxParser extends LazyChain/LazyChoice`
  として配置、`getLazyParsers()` が子を返す（`ParserGenerator.java:154`、`ParserRuleEmitter.java:610`）。
  反復・省略・グループ・分離リストは補助クラス、`@recovery` はラッパー、
  `@longestChoice`/`@predictiveChoice` は継承先変更、scope 系は `TransactionListener` 実装、
  capture 位置は `__CaptureSite`（`ParserRuleEmitter.java:627`、`ParserGenerator.java:200`）。
- 実物: `unlaxer-dsl/src/test/java/org/unlaxer/tinycalc/generated/TinyCalcParsers.java:101`
- ランタイム依存: `Parser/Parsers/ParseContext/Parsed/Token/TokenList/TokenKind/Source/ParserCursor`、
  transaction の begin/rollback/commit（`ChainInterface.java:29`、`TransactionElement.java:18`）。
  `ParseContext.tokenStack: Deque<TransactionElement>`（`ParseContext.java:38`）。
  DSL 固有: `ScopeStore`, `RegexTokenParser`, `SyncPointRecoveryParser`。

## 2. ParserIR は入力に使えない

- `ParserIR`（JSON Schema 2020-12、`irVersion "1.0"`）は各ルールを `RuleDecl` ノード
  (span 0..0) にし注釈と合成 scope イベントを足すだけ。**ルール本体・token 定義・参照グラフを
  出力しない**（`GrammarToParserIrExporter.java:38, :91`）。consumer は validator とテストのみ。
- 実際のコード生成に使う構造 IR は別物: Java `codegen.rust.GrammarIR`、Rust
  `unlaxer_codegen::GrammarIr`（`GrammarIR.java:5`、`rust/unlaxer-codegen/src/ir.rs:1`）。
  **Java 向け生成器は構造 IR を使っていない**。
- Rust `unlaxer-ubnf` は Java 不要の UBNF lexer/parser を持つが、lowering は imports・
  `@typeof`・一部 token/注釈を拒否（`rust/unlaxer-ubnf/src/lib.rs:66`、
  `rust/unlaxer-generator/src/lowering.rs:23`）。
- → ubnfc は UBNF フロントエンドを自前で持つ（既存 IR を継承しない）。

## 3. Rust 側

| クレート | 役割 |
|---|---|
| `unlaxer-ubnf` | lexer/parser/UBNF AST（import ファイル読込は非担当） |
| `unlaxer-generator` | parse → 検証/lowering → codegen、CLI |
| `unlaxer-codegen` | `GrammarIr` → `mod.rs/parser.rs/ast.rs/mapper.rs/evaluator.rs` |
| `unlaxer-runtime` | `Expr/Rule` グラフの実行、CST、capture、診断、scope、memo、checkpoint |

生成 `parser.rs` は `OnceLock<SharedGrammar>` に `Rule { expression: Expr::Choice(...) }` の
ベクタを置き `context.parse_shared_grammar(grammar(), 0, true)` で解釈実行
（`rust/examples/evolution/src/generated/parser.rs:27`、`rust/unlaxer-runtime/src/lib.rs:1596`）。

## 4. ベンチ基盤

- repo 内に JMH / criterion は無い。`benchmarks/` は結果 md/csv のみ。ハーネスは
  `System.nanoTime()` ループ（`unlaxer-common/src/test/java/org/unlaxer/parser/EngineBenchmark.java:103`）
  と Rust `Instant`（`rust/examples/evolution/examples/shared_grammar_benchmark.rs:16`）。
- 流用できる入力:
  - Rust Evolution `if(1, 3*3, neg(2))`（18 文字・深さ 2、UBNF あり、setup/parse/map 分離）
    → **比較開始点として最適**（`unlaxer-dsl/src/test/resources/evolution/3/Evolution.ubnf`）
  - Java calculator: 短式 22 文字、長式 512 atom / 2,559 文字（手書き `CalculatorParsers` 比較）
  - Java exponential-deep `nested(12,"x")` は**失敗入力**（成功解析速度と分ける）
  - 共通 corpus: Evolution 37 入力、四則 24、右結合 28（括弧深さ ≤2、Unicode 含む）
- 記録数値（1 回あたり）:
  - Java 独自ハーネス 2026-09-20 memo 無効: short 602 µs / long 40,363 µs / deep 172,871 µs
  - Rust Evolution 共有グラフ: setup 2.4 ns / parse 16,747 ns / parse+map 18,057 ns
  - TinyExpression Rust criterion: complex 1.426 ms / comparison-heavy 0.473 ms
  - TinyExpression Java memo 後 facade: complex 12.32 ms / comparison-heavy 9.40 ms / complex-x64 958 ms
  - `.tiny` fixture 本体は tinyexpression repo 側。式長・深さは未確認。

## 5. 静的生成で特別扱いが要る意味論（要点）

| 機能 | 現状の契約 | 必要技法 |
|---|---|---|
| 左再帰 | Java 警告のみ、Rust lowering 拒否 | 直接/間接の検出・除去、演算子は優先順位上昇法 |
| assoc/precedence | 優先順位は参照階層で決まる。右結合 canonical は `Base Op Self \| Base` | 反復/右再帰と列形式 AST の対応 |
| `\|` | PEG 宣言順・最初の成功 | 順序付き選択、証明できる箇所のみ FIRST dispatch |
| `@longestChoice` | 全代替試行・最大消費・同長は宣言順 | 共通開始状態からの試行と敗者破棄 |
| `@predictiveChoice` | 不可能候補のみ除外、全失敗なら全候補再試行 | 保守的 FIRST/nullable、診断互換 fallback |
| `@backref` | scope あり: 未定義 WARNING で構文成功。scope なし: Java テキスト比較 + ERROR 診断、Rust 未対応 | capture 値保存と比較・診断 |
| `@scopeTree` | 解析時ネストスタック、内側優先 lookup、親 rollback まで契約 | ログ + 巻き戻し |
| `@interleave` | rule whitespace 優先、trivia は境界に挿入 | 境界への静的展開 |
| `@recovery` | sync / auto(FOLLOW 推定) / skip のラッパー（`ParserRuleEmitter.java:500`） | 同期点・FOLLOW・進行保証・エラーノード |
| `%` | `elem (sep elem)*` | sep+次要素を一組で巻き戻し、空一致反復検査 |
| `@typeof` | Java mapper が実行時クラス一致検査、Rust 拒否 | AST 構築後の型検査 |
| import/namespace | Java resolver・循環検出・alias 統合あり、Rust 拒否（`UBNFMapper.java:127`） | 生成前の依存解決・ルール ID 化 |
| NEGATION/CHAR_RANGE/ANY | codepoint 単位 / BMP 非 surrogate 境界 | 座標系を混同しない |
| LOOKAHEAD/NEG/EOF/EMPTY | ゼロ消費。consumed/matched cursor が別 | ゼロ幅述語と状態復元 |
| REGEX/CI | Java `Pattern`、ゼロ幅は失敗。Rust 未対応 | 正規部分集合は DFA、他は明示拒否 |
| 任意クラス token | Java クラス参照可、Rust は allowlist | 組込み token の専用 lexer 化 + 拡張 API |

静的化の最大障壁: `@longestChoice`、scope の rollback 契約、任意 Java クラス token。
