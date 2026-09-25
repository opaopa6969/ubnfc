# diff-ast-fields-x-issue89

issue #89。fuzzer の seed ではなく、#86 の調査中に手で縮めた形（seed なし・`finding.json` なし）。
4 帯（各 201 seed × 12 入力）の掃き出しではこの形は生成されていない。

## signature

`diff:/ast/fields/x/*`

## 入力（JSON escape）

```json
"[ijx]"
```

## 文法

`grammar.ubnf`（`P ::= 'i' | 'j'` は写像なし・text だけの規則）。

```ubnf
@root @mapping(Top, params=[x]) Root ::= '[' ( P P ) @x Leaf @x ']';
```

## 差分（修正前）

| backend | `x` |
|---|---|
| Java | `["ij", Leaf]` |
| Rust | `["i", "j", Leaf]` |

`x` は `Leaf` の site を持つので node 変換になり、Rust の `node_values` が capture `( P P )` の子を組み立てて
写像なし規則 `P` ごとに字句を足していた。D-032（site より内側で text だけの領域は一致範囲全体で 1 つの値）
どおり Java が正しい。修正（Rust のみ）後は両 backend とも `["ij", Leaf]`。

回帰試験: `crates/ubnfc-fuzz/tests/text_group_capture_value.rs`。報告: `docs/reports/2026-09-25-fuzz-89.md`。
