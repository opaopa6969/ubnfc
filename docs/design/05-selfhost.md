# Rust self-host 経路

## 経路

```text
ubnf.ubnf
  │ generated::parse_with_options(lexical = true)
  ▼
typed Ast + captures + structured lexical occurrences (code point span)
  │ ubnfc-selfhost::projection + ubnfc-front::selfhost::convert
  ▼
syntax::ast::UbnfFile (byte span)
  │ pipeline::compile_parsed（import も解析済み ParsedSource）
  ▼
Grammar IR v1 ── ubnfc-rust::emit ──▶ generated Rust parser
```

この経路は `syntax::parse_file` へフォールバックしない。既存の `pipeline::compile` は
手書き parser で `ParsedSource` を作った後、同じ `compile_parsed` 経路を使う。
resolve と lower は渡された main/import AST を使う。`ParsedSource` の AST と元 source が
同じ解析結果であることは呼び出し側の契約であり、source を再解析して検査しない。

## 変換規則

- typed AST は mapping 済みの record と文字列を供給する。未 capture の構文は、採用した
  rule/expression、親 occurrence、code point span、完了順を持つ structured occurrence から読む。
- `AnnotatedElement` は現在の occurrence の直接の子だけを見る。入れ子の group や separator の
  suffix を外側へ持ち出さない。postfix、bounded、separated が併存した場合も AST には全て保持し、
  lower が D-015 の `postfix → bounded → separated` で一つを採用する。
- `A+{n,m}` は外側の `quantifier` に bounded、`postfix_quantifier` に先行 postfix を保持する。
  bounded は採用した `Digits` occurrence から整数を読み、`{n}` / `{n,m}` / `{n,}` を区別する。
- STRING は capture の raw（引用符を含む）を保持する。値は手書き lexer と同じく
  `\n`, `\t`, `\r`, `\\`, `\'` を解決し、未知 escape は backslash と文字を残す。
- 生成 runtime の span は code point 半開区間である。source の全 `char_indices` と EOF から
  対応表を一度作り、全 node/capture/occurrence を UTF-8 byte 半開区間へ変換する。
  範囲外、逆順、親からはみ出す occurrence は丸めず変換エラーにする。
- 空 Sequence の span は occurrence が trivia を消費した後の位置を使う。非空 Sequence と
  Choice は子要素の先頭から末尾までとし、手書き AST の span 規則に合わせる。

## 固定点の検証

`crates/ubnfc-selfhost/tests/regenerate.rs` は次の二つを byte 単位で検査する。

1. 生成 parser で sibling `unlaxer-parser/unlaxer-dsl/grammar/ubnf.ubnf` を読み、
   `compile_parsed` で得た canonical IR が
   `ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json` と一致する。
2. その self-host IR を `ubnfc_rust::emit` へ渡し、全生成ファイルの相対パスと内容が
   `crates/ubnfc-selfhost/src/generated` と一致する。

再現コマンド:

```sh
cargo test -p ubnfc-selfhost --offline --test parity --test regenerate
cargo test -p ubnfc-front --offline --test selfhost_conversion --test selfhost_pipeline
```

## 既知の差分

比較対象25件（24 fixture + P4）のうち、`ubnf.ubnf` が記述する構文だけを使う19件は
`UbnfFile` が全 field/span を含めて一致する。手書き frontend だけの互換拡張を使う次の6件は、
生成 parser が表の byte offset で拒否する。self-host を通すための grammar/IR 加工はしていない。

| source | byte | 手書き frontend の互換拡張 |
|---|---:|---|
| `positive/annotations.ubnf` | 43 | dotted mapping class (`Node.Item`) |
| `known/eval-default.ubnf` | 32 | `@eval` entry の `=` 区切り |
| `known/keyword-boundary.ubnf` | 24 | annotation keyword の識別子境界 (`@rooted`)。**D-040 の語境界そのもの**で、`ubnf.ubnf` を `@ubnf: 2`（または `@literalBoundary: word`）にすると解消する（実測: 受理 19 → 20、他の 5 件の位置は不変。`docs/reports/2026-09-23-ubnf-v2-word-boundary.md` §1.3）。self-description を v2 にするかは別の判断なので、この表は v1 のままの実測値 |
| `known/namespace.ubnf` | 28 | 3成分以上の rule namespace (`a.b.Value`) |
| `unlaxer-dsl/examples/tinycalc.ubnf` | 276 | setting block 内の二重引用符 |
| `tinyexpression-p4.ubnf` | 2595 | `@declares` description の capture 名（非 STRING） |

独立した構文エラー3例は、Unicode を含む場合も手書き/生成変換後の byte span が一致する。
validation error golden 3件では `all-tokens.ubnf` の AST が一致し、`annotations.ubnf` と
`namespace.ubnf` は上記の文法差分として位置を固定している。

`ubnfc-rust` に固定点を妨げる不足はない。将来の結合を単純化する改善として、公開
`Capture` に宣言元 `rule_id` を含めるか、capture site metadata を生成物で公開できる。
現在は unmodified golden IR から作った決定的な site→rule 表を selfhost crate に同梱し、
全 entry が golden と一致することをテストしている。
