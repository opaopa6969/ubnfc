# P4 仕様由来オラクル — 実装の突き合わせ（自動生成）

期待値は tinyexpression の `spec/SPEC.md` / `specs/language.md` の規範文から導いたもので、
どの実装の観測値も使っていない（`corpus/spec/README.md`）。

ケース 32 件。J1 / J2（Java 2 種）はここでは駆動していない。

## 判定の内訳

| 実装 | 一致 | 違反 | 未定 | 駆動不能 |
|---|---:|---:|---:|---:|
| ubnfc p4-rust | 25 | 0 | 4 | 3 |
| tinyexpression-rs | 25 | 0 | 4 | 3 |

## ケース × 実装

| ケース | scope | 期待 | ubnfc p4-rust | tinyexpression-rs | 引用 |
|---|---|---|---|---|---|
| `p4/if-else` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.7.1 if / else |
| `p4/if-without-else` | parse | reject | 一致（拒否） | 一致（拒否） | spec/SPEC.md #### 2.7.1 if / else |
| `p4/if-space-before-paren` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.7.1 if / else |
| `p4/if-block-comment-head` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.15 コメント |
| `p4/if-block-comment-inside` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.15 コメント |
| `p4/block-comment-anywhere` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.15 コメント |
| `p4/line-comment-not-in-body` | parse | reject | 一致（拒否） | 一致（拒否） | spec/SPEC.md ### 2.15 コメント |
| `p4/match-order` | runtime | undetermined | 駆動不能（駆動していない） | 駆動不能（駆動していない） | spec/SPEC.md #### 2.7.3 match 式 |
| `p4/number-integer` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.1 数値リテラル |
| `p4/number-decimal` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.1 数値リテラル |
| `p4/number-negative` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.1 数値リテラル |
| `p4/number-positive` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.1 数値リテラル |
| `p4/number-exponent-lower` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.1 数値リテラル |
| `p4/number-exponent-upper` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.1 数値リテラル |
| `p4/number-long-suffix` | parse | undetermined | 未定（拒否） | 未定（拒否） | spec/SPEC.md ### 2.17 機能パリティマトリクス（レガシー vs P4 UBNF） |
| `p4/number-float-suffix` | parse | undetermined | 未定（拒否） | 未定（拒否） | spec/SPEC.md ### 2.17 機能パリティマトリクス（レガシー vs P4 UBNF） |
| `p4/string-single-quote` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.2 文字列リテラル |
| `p4/string-double-quote` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.2 文字列リテラル |
| `p4/string-escape-newline` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.2 文字列リテラル |
| `p4/boolean-true` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.3 真偽値リテラル |
| `p4/boolean-false` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md #### 2.1.3 真偽値リテラル |
| `p4/arithmetic` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.3 算術演算子 |
| `p4/comparison` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.4 比較演算子 |
| `p4/comparison-at-top-level` | parse | undetermined | 未定（拒否） | 未定（受理） | spec/SPEC.md ### 2.4 比較演算子 |
| `p4/logical-and` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.5 論理演算子 |
| `p4/logical-not` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.5 論理演算子 |
| `p4/precedence-mul-over-add` | runtime | undetermined | 駆動不能（駆動していない） | 駆動不能（駆動していない） | spec/SPEC.md ### 2.6 演算子の優先順位（高い順） |
| `p4/precedence-xor-and-or-conflict` | parse | undetermined | 未定（受理） | 未定（受理） | spec/SPEC.md ### 2.6 演算子の優先順位（高い順） |
| `p4/variable-dollar` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.2 変数 |
| `p4/variable-without-dollar` | parse | reject | 一致（拒否） | 一致（拒否） | spec/SPEC.md ### 2.2 変数 |
| `p4/variable-case-sensitive` | parse | accept | 一致（受理） | 一致（受理） | spec/SPEC.md ### 2.2 変数 |
| `p4/variable-missing-is-null` | runtime | undetermined | 駆動不能（駆動していない） | 駆動不能（駆動していない） | specs/language.md ## 変数 |

## 未定（仕様が沈黙・矛盾）

- `p4/match-order`: 評価順は実行時の契約で、構文解析では観測できない。また match の具体構文を仕様本文が BNF で与えていない。
- `p4/number-long-suffix`: §2.1.1 の表は `42L` を数値リテラルとして挙げるが、§2.17 は数値具体型のサフィックスを HAND-ONLY（レガシーのみ）に分類している。P4 が受理すべきかが同じ文書の中で決まらない。
- `p4/number-float-suffix`: `42L` と同じ理由（§2.1.1 の表と §2.17 の分類が食い違う）。
- `p4/comparison-at-top-level`: 仕様は式全体（Formula）の production を与えておらず、比較式単独が完結した式として成立するかを述べていない。
- `p4/precedence-mul-over-add`: 優先順位は AST の形に現れるが、この比較ドライバは受理のみを記録する。
- `p4/precedence-xor-and-or-conflict`: spec/SPEC.md §2.6 と docs/language-guide.md は `^` > `&` > `|`、specs/language.md は `&` > `^` > `|` と書いており、同じ repo 内で矛盾している。文法ファイルの `@precedence` は前者と一致するが、規範文だけでは決まらない。
- `p4/variable-missing-is-null`: 未定義変数が null を返すのは評価時の契約で、構文解析では観測できない。
