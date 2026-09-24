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

### 7 件目 — 唯一の逆向き乖離（生成側が広い）とその閉じ方

上の 6 件はすべて「**手書き frontend だけが受理する**」向きで、self-host を通すための加工を
していないことの記録だった。2026-09-23 のクロスチェック
（[`../reports/2026-09-23-parser-feature-correctness-comparison.md`](../reports/2026-09-23-parser-feature-correctness-comparison.md) §3.1 (4)）で、
**逆向きが 1 件だけ**見つかった。

| source | 手書き | 生成 | 根本原因 |
|---|---|---|---|
| `grammar G { /* no */ R ::= 'x'; }` | 拒否 @12「UBNF のコメントは // 行コメントのみです」 | **受理**（`/* no */` が無いときと同じ AST） | `ubnf.ubnf:18` の `@whitespace: javaStyle` → `lower/trivia.rs:90-99` が javaStyle の区切り集合へ `BlockComment{/*, */}` を無条件に入れる |

UBNF 自身のコメント構文は `//` 行コメントだけで `/* ... */` は未対応
（`unlaxer-parser/unlaxer-dsl/specs/ubnf-syntax.md`。unlaxer-ubnf と Java bootstrap も拒否する）。
**正しいのは手書き側**で、自己記述文法 `ubnf.ubnf` が UBNF より広い言語を記述している。

`ubnf.ubnf` は読み取り専用の別 repo にあり、javaStyle の中身は corpus
（`corpus/oracle/comment-whitespace.json` の `expected` が `'/*'` / `'*/'` を含む）が固定している。
そこで **D-051** に従い、`ubnfc_selfhost::parse_file` が生成パーサの外でブロックコメントを
拒否する（`crates/ubnfc-selfhost/src/lib.rs` の `block_comment_start`）。手書き lexer と同じ
走査規則を使い、文字列リテラルと `//` の中は見ない。`/*` より手前に別の字句エラーがある入力は
生成パーサに任せるので、失敗位置も手書きと一致する。**再解析ではなく字句の門**なので、
「この経路は `syntax::parse_file` へフォールバックしない」という上の契約は保たれる。
IR・生成物・固定点は 1 byte も変わらない。

回帰は `crates/ubnfc-selfhost/tests/parity.rs` の
`block_comments_are_rejected_by_both_frontends`（拒否位置の一致・文字列リテラル中の `/*`・
先行する字句エラーの優先を固定する）。

**本来の直し方**は `ubnf.ubnf` が UBNF の実際の trivia を宣言すること
（`@whitespace: none` + `@comment: { line: '//' }`、または javaStyle からブロックコメントだけを
外せる表層を足すこと）だが、どちらもこの repo からは行えない。上流で直れば、この門は外せる。

独立した構文エラー3例は、Unicode を含む場合も手書き/生成変換後の byte span が一致する。
validation error golden 3件では `all-tokens.ubnf` の AST が一致し、`annotations.ubnf` と
`namespace.ubnf` は上記の文法差分として位置を固定している。

`ubnfc-rust` に固定点を妨げる不足はない。将来の結合を単純化する改善として、公開
`Capture` に宣言元 `rule_id` を含めるか、capture site metadata を生成物で公開できる。
現在は unmodified golden IR から作った決定的な site→rule 表を selfhost crate に同梱し、
全 entry が golden と一致することをテストしている。

### 8 件目 — 3段以上の dotted ルール参照（`a.b.Value`）は `ubnf.ubnf` 側の文法欠陥

issue #12（`opaopa6969/ubnfc`）の調査。上の表の `known/namespace.ubnf`（byte 28）と
同じ入力形（`grammar G { R ::= a.b.Value; }`）を、`unlaxer-parser` #279 で確定した
仕様（`unlaxer-dsl/specs/ubnf-syntax.md` 「## ルール宣言」290-293行目、
「UBNF 自体は `a.b.Value` 全体を1個の `IDENTIFIER` として扱う（分割しない）」）に
照らして再調査した。

- 手書き frontend（A, `crates/ubnfc-front/src/syntax`）: 受理（namespace=`a.b`, name=`Value`）
- 自己ホスト生成 frontend（B）: **拒否**
  `構文エラー (bytes 21..22): expected + or ? or * or { or % or @ or @typeof or ( or [ or ERROR or STRING or IDENTIFIER or | or ;`
  （`ubnfc_selfhost::parse_file` と `examples/ubnf-crosscheck` の `ubnf-crosscheck` バイナリで
  再現・確認済み。2段の `a.B` は A/B とも受理する）
- unlaxer-ubnf（C）・Java bootstrap（D）: 受理

根本原因は `unlaxer-parser/unlaxer-dsl/grammar/ubnf.ubnf`（読み取り専用）の
`QuantifiedRef` 生成規則そのものにある。

```
@backref(name=name)
QuantifiedRef ::= [ IDENTIFIER @namespace '.' ] IDENTIFIER @name ;
```

これは `IDENTIFIER ['.' IDENTIFIER]`、すなわちドット1個だけの任意分割であり、
段数上限なしの chained dot を最初から想定していない。生成 parser
（`crates/ubnfc-selfhost/src/generated/parser.rs` の `r55_c` / rule 55）はこの文法を
忠実に実装しているだけで、`ubnfc_selfhost::parse_file` や
`crates/ubnfc-front/src/selfhost/convert()` の投影・変換ロジックに欠陥はない
（7件目と異なり、こちらは**生成側が仕様より狭い**方向の乖離で、self-host 経路の外側で
埋める字句ゲートのような閉じ方も取れない — 文法自体が3段以上を受理しない以上、
生成 parser の手前でドットを数えて許可判定をしても、後段の `QuantifiedRef` 本体の
消費規則が2段までしか進まないため無意味）。

したがって直し方は `unlaxer-parser` 側で `QuantifiedRef` を chained dot 対応にし、
`ubnf.ubnf` を再生成することであり、この repo（`ubnfc`）からは行えない。
最小再現・仕様引用・修正方針は
[`unlaxer-parser` issue #284](https://github.com/opaopa6969/unlaxer-parser/issues/284)
で報告済み。`known/namespace.ubnf` の扱い（表の位置固定）は変更しない。上流で直れば、
その修正後の `ubnf.ubnf` を取り込んだ時点でこの表の行と本節は不要になる。
