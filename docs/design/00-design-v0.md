# ubnfc 設計草案 v0

> 状態: draft（codex/astra との往復で改訂する。決定事項は `docs/decisions/` へ切り出す）
> 作成: 2026-09-22

## 0. 一言で

UBNF を入力に、**コンビネータ・ランタイムを持たない**静的パーサ（手書き相当の再帰下降 +
専用スキャン関数 + 予測分岐）を Java / Rust で直接生成する。既存 unlaxer-parser の
生成物（コンビネータ木を組み立てるコード）とは**受理言語・AST・診断が同じ**で、
**内部構造は共有しない**。

## 1. ゴール / 非ゴール

### ゴール（第一段の完了条件 = 「既存 .ubnf 全互換」）

1. unlaxer-parser 内の全 `.ubnf`（24 ファイル、`ubnf.ubnf` 304 行を含む）と
   tinyexpression P4 文法（124 ルール）が生成・コンパイル・実行できる。
2. 既存 conformance corpus（`unlaxer-dsl/src/test/resources/*/corpus.json`：
   grammar + input + 期待 AST(JSON) + captures + declarations）を両バックエンドで通す。
   期待値は既存実装ではなく **corpus の独立期待値**を正とする。
3. self-host: `ubnf.ubnf` から生成した Rust パーサで `ubnf.ubnf` を読み、
   手書きフロントエンドと **同じ IR JSON（byte 一致）** を出す。
4. 性能: tinyexpression P4 の既存ベンチ入力で、既存 Java facade（memo 込み）に対して
   **≥10×**（暫定。codex の棚卸しでベースライン数値を確定してから固定する）。
5. 生成コードは外部依存ゼロ（Java: JDK 21 / Rust: std。serde は feature）。

### 非ゴール（第一段では作らない。理由付き）

- LSP / DAP サーバ生成: パーサ性能と独立で、既存生成物は JVM 専用の大きなサーバ実装。
  新パーサは LSP が要る情報（診断・scope・span）を **データとして**提供するに留める。
- CST（Token 木）の生成: 既存 API のうち CST 依存のもの（`mapParsedToken`, `sourceSpanOf` の
  identity ベース位置）は移植しない。AST + span を一次成果物にする。
  → 決定 D-003 参照（争点）。
- 既存 `CodegenMain` の CLI フラグ互換、Maven plugin: 新 repo なので新 CLI を定義する。
- Java `int` 数値 capture の `Integer.parseInt` 契約、`Optional<List<>>` 入れ子 container の
  mapper 再構築など、既存が「未完了」と明記している差分は **corpus にある範囲**だけ追う。

## 2. 全体構成（IR を境界に、ユーザー決定）

```
                 ubnfc-front (Rust crate)
 .ubnf ──► lex/parse (手書き) ──► GrammarAst
       ──► resolve  (@import, namespace, token class → builtin kind / extern)
       ──► validate (既存 E-*/W-* コード互換)
       ──► lower    ──► Grammar IR (serde; JSON, version 付き)
       ──► analyze  (nullable / FIRST / FOLLOW / 左再帰 / 優先順位 / choice 戦略 /
                     capture cardinality / AST schema / memo 要否 / scope effects)
       ──► IR JSON (解析結果込み。バックエンドは解析をやり直さない)
                        │
          ┌─────────────┴─────────────┐
   ubnfc-java (Java 21, Maven)    ubnfc-rust (Rust crate)
   IR JSON → *.java                IR JSON → *.rs
                        │
              conformance harness (Rust CLI + Java runner)
              corpus.json → 生成 → compile → run → canonical JSON 比較
```

- **front を Rust に置く理由**: 単一バイナリで CI・IDE から呼べる。self-host の対象。
- **バックエンドを各言語で書く理由**: 生成コードの idiom・型システム（record/sealed vs
  enum/Box）をその言語で考えるほうが質が上がる（ユーザー決定）。
- **IR に解析結果を含める理由**: 2 バックエンドが FIRST 集合や左再帰の扱いで食い違うと
  受理言語が割れる。解析は front で 1 回、バックエンドは「IR をコードに写す」だけにする。

## 3. Grammar IR（契約）

版数付き JSON。スキーマは `ir/schema/grammar-ir.v1.json`（JSON Schema）で固定し、
両バックエンドのテストで検証する。概略:

```
GrammarIr {
  version: 1,
  grammars: [ GrammarUnit ],          // @import で複数
  entry: { grammar, rule },           // @root
}
GrammarUnit {
  name, package, settings { whitespace: none|javaStyle, comment{line,blockOpen,blockClose} },
  tokens: [ TokenDef ],
  rules:  [ RuleDef ],
  astSchema: [ AstType ],             // @mapping から導出済み（sealed/record/enum/共通 field）
}
TokenDef {
  id, name,
  kind: Builtin{which: Number|Identifier|SingleQuoted|Quoted|Digit|Alpha|EndOfSource|...}
      | Until{terminator} | Negation{chars} | Lookahead{pat} | NegLookahead{pat}
      | CharRange{min,max} | CaseInsensitive{word} | Regex{pattern} | Any | Eof | Empty
      | Extern{className},            // 未知クラス → ユーザー実装フック
  memoSafe: bool,                     // @memoSafeToken
}
RuleDef {
  id, name, annotations {...全て構造化},
  body: Expr,                         // Seq/Choice/Opt/Repeat{min,max}/Separated/Group/
                                      // Literal/RuleRef/TokenRef/Error/Lookahead...
  trivia: none|javaStyle,             // rule-trivia 契約で解決済みの有効値
  analysis: {
    nullable, first: CharClassSet, follow: CharClassSet,
    leftRec: none|direct{...}|indirect{cycle},
    operator: { prec, assoc, canonicalForm } | null,
    choiceStrategy: per Choice node → Predict{table} | Ordered | Longest,
    memo: none|required{reason},      // (rule,pos) 再試行が起こり得る箇所のみ
    captures: [ {name, sites:[path], cardinality: scalar|optional|list, valueType} ],
    scope: { enter: bool, declares: [...], backref: ... },
    recovery: {...} | null,
  }
}
```

争点（D-001）: `first`/`follow` を **文字クラス**で持つ（scannerless なので token 集合では
なく「次の 1 code point の集合 + EOF」）。ASCII 128 は bitset、非 ASCII は範囲リスト。

## 4. 生成コードの形（性能の核心）

### 4.1 字句（scannerless のまま速くする）

UBNF は scannerless（キーワードはリテラル、token は文字レベル parser、空白は連接境界）。
別レキサを立てると `@whitespace(none)` のルール毎切替・`UNTIL`・`%`・CI 等で意味論が
割れるので、**scannerless を維持しつつスキャンをタイトにする**:

- token 種ごとに専用 `scanX(pos) -> end|-1` を生成（builtin は手書きテンプレート）。
  `IdentifierParser` / `NumberParser` 等の**受理範囲は既存クラスの実装から起こす**
  （codex 棚卸しで確認 → `docs/tokens.md` に表として固定）。
- リテラル: 長さ別 `startsWith` 比較。キーワード境界（`'let'` が `letter` を横取りしない
  規則）は既存の `WordParser` 意味論に従う（争点 D-002: 境界の正確な定義）。
- 空白/コメントスキップ: 生成時に「連接境界」にだけ挿入（既存 DelimitedChain と同位置）。
  同じ位置を backtrack で何度もスキップしないよう、1 スロットの `(pos→afterTrivia)` cache。
- 文字アクセス: Java は `char[]`（`String.toCharArray()` 一回）+ サロゲート境界だけ
  code point 化。Rust は `&[u8]` + ASCII fast path。

### 4.2 構文（静的再帰下降 + 予測分岐）

- ルール = 関数 `parseRule(pos) -> (endPos, node) | FAIL`。状態は `int pos` と
  出力 AST だけ。トランザクション／スナップショット／リスナーは無い。
  backtrack = `pos` を戻す（+ scope ログ長を戻す、下記）。
- Choice: `analysis.choiceStrategy` に従う。
  - `Predict`: FIRST が互いに素なら次文字で `switch`（tableswitch）。ordered PEG と
    受理言語が変わらないことを解析が保証した場合のみ。
  - `Ordered`: FIRST 重複あり → 順に試行（PEG）。ただし「次文字が全選択肢の FIRST に無い」
    ときは即失敗（既存 `@predictiveChoice` と同じ保守的スキップ）。
  - `Longest`: `@longestChoice`。全試行して最長。
- 左再帰・演算子:
  - 直接左再帰 `E ::= E op T | T` → ループ（既存 Java も同じ変換）。
  - `@leftAssoc` 正規形 `Base @left { Op @op Base @right }` → ループ。AST は既存契約どおり
    `left + [op] + [right]` リスト（二分木化しない）。
  - `@rightAssoc` 正規形 → `Base Op Self | Base` の右再帰（既存契約）。
  - 優先順位の**段の潰し込み**（precedence climbing で 16 段の関数呼び出しを 1 ループに）
    は AST 契約（各段の mapping class / span）を保ったまま可能か → 争点 D-004。
    第一段では段ごとの関数のまま（正しさ優先）、ベンチで効くなら D-004 で採る。
- Memoization: 解析で `memo: required` が付いた `(rule)` だけ、`pos → result` の
  packrat スロットを生成（配列 index、HashMap 不使用）。他は memo 無し。
  既存の「安全な失敗の memo」に相当する機能は、予測分岐で backtrack 自体を減らす
  ことで置き換える。tinyexpression の 5 段ネスト if（既存 memo 無しで 4〜6 秒）が
  memo 無しで済むかは M7 で計測して決める。
- AST 構築: **CST を作らず、parse 中に typed AST を直接組み立てる**。capture の
  cardinality（scalar/optional/list、同名 nested は内側→外側順）は解析済みなので、
  生成コードは局所変数 / `ArrayList` に直接積む。失敗した枝の部分 AST は捨てる（GC/drop）。
- 例外・Result を制御フローに使わない。Java は `int` 戻り値（`-1` = FAIL）+ 出力は
  フィールド/引数。Rust は `Option<usize>` + `&mut` 出力。

### 4.3 スコープ（@scopeTree / @declares / @backref）

`ScopeStore` = append-only の宣言ログ `Vec<Decl>` + スコープ開始 index のスタック +
参照ログ + 診断ログ。rollback は各ログの **長さを戻すだけ**（O(1)）。lookup は
内側スコープから線形／小 HashMap。既存契約（trim 規則、code point offset、宣言後に参照、
未定義は WARNING で構文は成功、失敗枝の復元）を corpus で検証。

### 4.4 診断

farthest failure（位置）と expected 集合（token/literal/rule の小さな id 列）を、
「今の失敗位置 ≥ 記録済み最遠位置」のときだけ更新。ホットパスの分岐は 1 つ。
LSP 向けに既存 `ULX-PARSE-001` と同等の構造（位置・expected・deepest rule）を返す。

### 4.5 @recovery

既存意味論を確認のうえ（codex 棚卸し項目）: sync token までスキップ → `Error` ノードを
AST に挿入 → 継続。第一段では corpus に有る範囲だけ。

## 5. AST（Java / Rust）

- Java: `sealed interface {G}Ast` + `record`。`@mapping(Node.Item)` の入れ子、
  `@commonField` → interface メソッド、`@enum` → `enum`、`Optional<T>` / `List<T>`、
  数値 capture は `int`（既存契約）。span は `int start, int end`（code point）。
- Rust: `enum Ast { ... }` + 各 variant struct、`Option<Box<_>>` / `Vec<_>`、
  数値は `String`（既存 Rust 契約。Java との差は corpus が既に別 oracle）。
  canonical JSON（serde feature）で corpus 比較。
- span の単位は **code point**（既存契約）。内部 index（UTF-16 / byte）から変換して報告。

## 6. 検証（何をもって「互換」とするか）

1. 既存 corpus（rule-trivia / nested-captures / scope-annotations / scope-store /
   numeric-capture / tiny-string-token / tiny-code-fence / golden / catalog / cardinality）
   を **そのまま読み込み**、両バックエンドで canonical JSON 一致。
2. 全 24 fixture + P4: 生成 → compile（javac / rustc）成功。
3. self-host byte 一致（§1-3）。
4. `analysis` の単体: nullable/FIRST/左再帰の判定を小文法で golden テスト。
5. ベンチ: 既存 `benchmarks/` の入力を流用、JMH（Java）/ criterion（Rust）。
   計測は A/B 2 セッション、`gc.alloc.rate.norm` も採る（既存メモリの教訓）。

## 7. マイルストーン（全互換の中の内部順序）

| M | 内容 | 完了条件 |
|---|---|---|
| M0 | repo / IR schema v1 / corpus loader / harness 骨格 | 既存 corpus を読み込み「未実装」で赤になる |
| M1 | front: UBNF 手書き lexer+parser → GrammarAst → validate → IR | 24 fixture + P4 が IR JSON に落ちる。既存 E-* コード一致 |
| M2 | analysis | golden テスト。P4 の choice 戦略・memo 要否が出る |
| M3 | Rust backend（parser + AST + canonical JSON） | corpus 緑（Rust 側） |
| M4 | Java backend | corpus 緑（Java 側） |
| M5 | scope / backref / recovery / longest / import / enum / eval / commonField / catalog | 全 corpus 緑、24 fixture + P4 compile |
| M6 | self-host | IR byte 一致 |
| M7 | bench + 最適化ループ（D-004 等） | ≥10× or 未達の根拠 |

## 8. 争点（codex/astra と詰める）

- D-001 FIRST を文字クラスで持つ vs 疑似 token 化。非 ASCII と `Extern` token の扱い。
- D-002 キーワード境界の正確な意味論（`WordParser` vs `SingleStringParser` の使い分けを
  既存生成器がどう決めているか）。
- D-003 CST を捨ててよいか。既存 LSP/DAP・`sourceSpanOf`・`mapParsedToken` 利用者の移行。
- D-004 precedence climbing による段の潰し込みと AST 契約の両立。
- D-005 memo の要否判定アルゴリズム（保守的すぎると遅い、緩いと指数）。
- D-006 Java の AST 表現: record 木 vs SoA（int 配列）。まず record、ベンチで判断。
- D-007 span 単位の内部表現と変換コスト（Java UTF-16 → code point）。
- D-008 `Extern` token フックの API（Java interface / Rust trait）と memo 安全性。
- D-009 IR に解析結果を入れる vs バックエンド側で再計算（契約の太さ）。
