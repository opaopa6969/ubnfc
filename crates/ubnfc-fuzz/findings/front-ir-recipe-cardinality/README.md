# front: 入れ子の optional を capture すると front 自身の IR 契約検査に落ちる

runtime の差分ではなく、生成した文法が front に拒否された理由。
campaign の 320 文法のうち **19 件**がこの形で discard された（最多の discard 理由）。

## 再現

```sh
cargo run -q --bin ubnfc -- validate --grammar crates/ubnfc-fuzz/findings/front-ir-recipe-cardinality/grammar.ubnf
# 診断なし、exit 0
cargo run -q --bin ubnfc -- ir --grammar crates/ubnfc-fuzz/findings/front-ir-recipe-cardinality/grammar.ubnf --out /dev/null
# ubnfc: 内部エラー: 生成 IR が契約に違反しています
# /semantics/recipes/1/fields/0/cardinality: recipe と capture の意味値 cardinality が不一致
# exit 1
```

seed からの自動最小化: `cargo run -p ubnfc-fuzz -- --minimize-front 15`。

## 観測

| 本体 | `ubnfc ir` |
|---|---|
| `[ N? ] @v`（`N` は `@mapping` を持つ規則） | **落ちる** |
| `[ [ N? ] @v ]` | 落ちる |
| `[ N ] @v` | 通る |
| `[ 'n'? ] @v`（mapped node でなくリテラル） | 通る |

`@mapping` を持つ規則への参照を **optional の二重入れ子**（`[ X? ]`）にして capture すると、
recipe 側の field cardinality と capture 側の意味値 cardinality が食い違う。

## 三分類

**(c) front / IR のバグ**。`ubnfc validate` は通り、`ubnfc ir` が自分で書いた IR を
自分の契約検査（`docs/design/03-design-v1.md` §5「意味検証の責務: … capture/recipe/typeof/catalog 型整合」）
で弾いている。IR が出ないので backend は一切呼ばれず、corpus にもこの形の fixture は無い。
