# 仕様由来オラクル（spec-derived）

`corpus/oracle/` が**旧 Java 実装の観測値**を固定したものであるのに対し、ここは
**規範文書の文から導いた期待値**だけを置く。issue #5 の成果物。

## 絶対規則

1. **どの実装の実行結果も期待値に使わない。** 期待値は引用した規範文だけから決める。
2. **各ケースは規範文を逐語で引用する**（`citation.file` / `citation.heading` /
   `citation.sentence`）。`crates/ubnfc-harness/tests/spec_corpus.rs` が
   引用元の見出しが実在すること、引用文が仕様ファイルに逐語で含まれることを検査する。
3. **仕様が沈黙・矛盾している場合は推測せず `undetermined`** とし、`question` に
   何が決まらないのかを書く。「実装がこうしているから」で埋めない。
4. 実装が期待と食い違ったら、**まず仕様の読み方を疑う**。期待値を実装に合わせて
   書き換えた時点でこの corpus の意味が無くなる。

## 配置

| ディレクトリ | 規範文書 | 備考 |
|---|---|---|
| `ubnf/` | `unlaxer-parser/unlaxer-dsl/specs/{ubnf-syntax,annotations,validation}.md` | UBNF 文法族 |
| `p4/` | `tinyexpression/spec/SPEC.md`、`tinyexpression/specs/language.md`、`tinyexpression/docs/language-guide.md` | P4（TinyExpression）式言語 |

引用先は sibling repository なので、無い環境ではテストは skip する（読み取り専用）。

## スキーマ

```jsonc
{
  "family": "ubnf",
  "section": "コメント",              // 人間向けの節名
  "summary": "…",
  "cases": [
    {
      "id": "comment/block-rejected",  // ファイル横断で一意
      "scope": "parse",                // parse | validate | runtime
      "citation": {
        "file": "unlaxer-dsl/specs/ubnf-syntax.md",   // workspace 相対
        "heading": "## ルール宣言",                    // 実在する見出し行そのもの
        "sentence": "UBNF ファイル自体のコメント構文は …"  // 逐語（空白の畳み込みのみ許す）
      },
      "input": "grammar G { /* no */ R ::= 'x'; }",
      "expectation": { "verdict": "reject" },
      "note": "…"                       // 任意
    }
  ]
}
```

### `scope`

| 値 | 意味 | 駆動できる実装 |
|---|---|---|
| `parse` | UBNF / P4 ソースの受理・拒否・構文木の形 | 4 実装すべて |
| `validate` | 文法の意味制約（診断コード） | ubnfc `validate` と Java `GrammarValidator` のみ |
| `runtime` | 生成パーサの実行時挙動 | このドライバでは駆動不能（仕様文は残す） |

### `expectation`

| verdict | 追加フィールド | 意味 |
|---|---|---|
| `accept` | `canonicalContains`（正準 AST JSON の部分文字列の配列）、`sameCanonicalAs`（別ケース ID） | 受理する。構造まで縛る場合は追加フィールドで書く |
| `reject` | — | 拒否する |
| `diagnostic` | `code` | `validate` で当該診断コードを出す |
| `undetermined` | `question` | 仕様が決めていない。実装の挙動は記録するが合否は付けない |

`sameCanonicalAs` は「`element?` は `[ element ]` と同じ意味です」のように、
仕様が 2 つの綴りの等価性を述べている場合に使う。

## 走らせ方

```bash
bash examples/ubnf-crosscheck/scripts/run-spec.sh   # UBNF 4 実装 × 全ケース
cargo test --offline -p ubnfc-harness --test spec_corpus   # 引用の解決とスキーマ
```

結果は `examples/ubnf-crosscheck/out-spec/report.md`（`evidence/` に実行済みの写しがある）。
分析は `docs/reports/2026-09-24-spec-derived-oracle.md`。
