# front: capture 付き `{0}` / `{1}` / `{0,1}` を `ubnfc validate` は通し `ubnfc ir` が落ちる

これは runtime の差分ではなく、**生成器の文法が front に拒否された理由**を追って見つけた
front（`crates/ubnfc-front`）側の不整合。fuzzer の finding writer ではなく手で最小化した。

## 再現

```sh
cargo run -q --bin ubnfc -- validate --grammar crates/ubnfc-fuzz/findings/front-capture-cardinality-bounded/grammar.ubnf
# 診断なし、exit 0
cargo run -q --bin ubnfc -- ir --grammar crates/ubnfc-fuzz/findings/front-capture-cardinality-bounded/grammar.ubnf --out /dev/null
# ubnfc: CaptureCardinality { site_id: "...body/0/repeat/capture/0", declared: List, inferred: Scalar }
# exit 1
```

## 観測

| 量指定子 | `ubnfc validate` | `ubnfc ir` |
|---|---|---|
| `'a'{0} @v` | 通る | `CaptureCardinality { declared: List, inferred: Optional }` |
| `'a'{1} @v` | 通る | `CaptureCardinality { declared: List, inferred: Scalar }` |
| `'a'{0,1} @v` | 通る | `CaptureCardinality { declared: List, inferred: Optional }` |
| `'a'{2} @v` / `'a'{0,2} @v` / `'a'{1,} @v` | 通る | 通る |

capture のない同じ量指定子（`'a'{1}`）は `ir` も通る。量指定子が capture site の
**外側**にあっても同じで、`grammar-group.ubnf` の `( 'a' @v ){1}` も同じ理由で落ちる
（`( 'a' @v ){2}` は通る）。campaign の 320 文法のうち 9 件がこの形で discard された。

## 三分類

**(c) front / IR のバグ**。lowering は Bounded 量指定子の capture site を無条件に `List` と
宣言するのに対し、analysis の cardinality 推論は `max <= 1` を `Scalar` / `Optional` と推論し、
front 自身の突き合わせ（`pipeline::checks` の `CaptureCardinality`）で落ちる。
`ubnfc validate` はこの検査を行わないので、**「validate は通るが ir は通らない」文法**が存在する。
`docs/design/03-design-v1.md` §5（IR の意味検証責務に「capture/recipe/typeof/catalog 型整合」を挙げる）
に照らすと、どちらか一方に寄せるべき不整合。修正方針は 2 つあり、本タスクでは選ばない:

1. lowering 側で `max <= 1` の Bounded を Scalar / Optional と宣言する（`{1}` は `X` と、
   `{0,1}` は `X?` と同じ cardinality）。
2. analysis 側で Bounded を常に List と推論する。

corpus では捕まらない: 既存 fixture に capture 付きの `{0}` / `{1}` / `{0,1}` が無い。
