# UBNF v2 — 言語の弱点を消す

> 2026-09-23 開始。利用者が tinyexpression のみの段階で、互換のために抱えていた欠陥を解消する。

## 0. バージョン門

grammar 設定 `@ubnf: 1|2`（既定 **1**）。

- **v1 は凍結**。既存の 24 fixture・P4・corpus 1,212 ケース・差分ファジング・self-host 固定点が
  永久にこれを守る。v1 の文法は 1 文字も変えずに通り続ける。
- **v2 でのみ**新機能と挙動変更を有効にする。IR には `languageVersion` を載せ、backend は
  未知バージョンを明示拒否する。
- 純粋な追加（新しい書き方を足すだけで既存の意味を変えない）は v1 でも使えてよい。
  **既存の受理・AST・診断が変わるものだけ** v2 に閉じる。

## 1. 左再帰（最優先）

### 問題
P4 は数値・文字列・真偽値・オブジェクトで同じ優先順位塔を 4 回書いており、全体 124 ルール。

```ubnf
NumberExpression ::= NumberTerm @left { AddOp @op NumberTerm @right } ;
NumberTerm       ::= NumberFactor @left { MulOp @op NumberFactor @right } ;
NumberFactor     ::= ... ;
```

ANTLR4 なら 1 ルール。可読性・保守性の差は決定的で、UBNF の最大の弱点。

### 方針（v2）
ANTLR4 と同じ「候補順＝優先順位（先頭が最高）」の左再帰ルールを導入する。

```ubnf
@leftRec
Expr ::=
    Expr '^' @op Expr        @rightAssoc   // 最高優先、右結合
  | Expr ('*'|'/') @op Expr                // 左結合
  | Expr ('+'|'-') @op Expr
  | '(' Expr ')'
  | NUMBER ;
```

- 直接左再帰（先頭が自分自身）と直接右再帰（末尾が自分自身）を優先順位上昇法へ lowering する。
  ubnfc には既に D-004 の climbing 設計、D-031 の直接左再帰書き換え、`analysis/operators.rs` の
  塔解析がある。
- 間接左再帰は引き続き拒否（ANTLR4 も同じ）。
- AST は演算子適用ごとの二項ノード（塔形式の `left + op[] + right[]` より自然）。
  塔形式も v1 として残るので、移行は利用者の選択。
- 未解決: trivia 境界（D-004 の注記）、`@mapping` の型付け、scope/recovery を含む塔の扱い。

## 2. 到達不能な候補の検出（低リスク・即効）

PEG の順序付き選択は、候補順を誤ると黙って前の候補が勝つ。P4 の `comparison-heavy` が
`Formula` 入口で拒否されるのはこれが原因だった。

`crates/ubnfc-rust/src/predict.rs` の trivia 越し FIRST 集合と `analysis` の nullable/minConsume が
既にあるので、検証器に警告を足せる:

- `W-CHOICE-UNREACHABLE`: 候補 B の言語が先行候補 A に完全に含まれる（証明できた場合のみ）
- `W-CHOICE-PREFIX`: A が B の真の接頭辞（`'a' | 'ab'` 形）。PEG では A が勝ち B は事実上死ぬ
- `W-CHOICE-NULLABLE-FIRST`: nullable な候補が末尾以外にある（以降が全て死ぬ）

**v1 でも出す**（警告のみで受理は変えない）。`--strict` でエラー化。

## 3. リテラルの単語境界（v2）

`'let'` が `letter` の先頭 3 文字に一致する（D-002）。仕様書自身が「識別子を横取りしない」と書いて
実装と矛盾していた。

- v2 既定: 英数字で終わるリテラルの直後が識別子構成文字なら不一致（`@literalBoundary: word|none` で上書き）
- v1 既定: 現状維持（`none`）
- 着手前に**互換影響を実測**する（全 fixture + corpus を boundary on で走らせ、変化した観測を数える）

## 4. パラメータ化ルール（v2）

Menhir 風の高階ルールで文法量を減らす。`%` 分離が組込みなのはこの不在の代償。

```ubnf
List<X, Sep> ::= X { Sep X } ;
Args ::= '(' [ List<Expr, ','> ] ')' ;
```

**意味述語（ANTLR の `{...}?`）は入れない。** 終端が「(入力, 位置) の純粋述語」である性質に
memo・候補除外・字句キャッシュの健全性が全部乗っているため（D-013、`06` §3）。
文脈依存が要る場面は `@backref`/`@typeof`/`@scopeTree` で宣言的に扱う。

## 5. 言語中立な token 語彙（v2）

```ubnf
token NUMBER = org.unlaxer.parser.elementary.NumberParser   // v1
token NUMBER = NUMBER                                        // v2（組込み語彙）
token CODE   = extern 'CodeStartParser'                      // host 実装は extern と明示
```

IR には既に組込み種別があるので、表層の語彙を足すだけ。

## 6. アノテーションの統廃合（v2）

- `@interleave(profile=)` は `@whitespace` にほぼ吸収できる（rule-trivia 契約で優先順位が既に決まっている）
- `@backref` がスコープの有無で意味が変わるのを 2 つの注釈に分ける
- `@mapping` / `@enum` / `@commonField` の関係を 1 つの型注釈系に整理

## 実施順

1. §2 到達不能候補の検出（v1 で警告、低リスク）
2. §1 左再帰（最大の価値）
3. §3 単語境界（互換影響の実測 → v2 既定）
4. §5 token 語彙、§4 パラメータ化ルール、§6 統廃合
