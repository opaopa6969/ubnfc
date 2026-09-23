# P4 4 実装クロスチェック（自動生成）

## 受理（AST を得られたか）

| fixture | ubnfc p4-rust | ubnfc p4-java | tinyexpression-rs | 旧Java mapper | 旧Java facade |
|---|---|---|---|---|---|
| `comparison-heavy-x16.tiny` | 拒否/未取得 | 拒否/未取得 | 受理 | 拒否/未取得 | 拒否/未取得 |
| `comparison-heavy-x4.tiny` | 拒否/未取得 | 拒否/未取得 | 受理 | 拒否/未取得 | 拒否/未取得 |
| `comparison-heavy-x64.tiny` | 拒否/未取得 | 拒否/未取得 | 受理 | 拒否/未取得 | 拒否/未取得 |
| `comparison-heavy.tiny` | 拒否/未取得 | 拒否/未取得 | 受理 | 拒否/未取得 | 受理 |
| `complex-half.tiny` | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 |
| `complex-tail.tiny` | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 |
| `complex-x16.tiny` | 受理 | 受理 | 受理 | 受理 | 拒否/未取得 |
| `complex-x4.tiny` | 受理 | 受理 | 受理 | 受理 | 拒否/未取得 |
| `complex-x64.tiny` | 受理 | 受理 | 受理 | 拒否/未取得 | 拒否/未取得 |
| `complex.tiny` | 受理 | 受理 | 受理 | 受理 | 拒否/未取得 |
| `flat-arithmetic.tiny` | 受理 | 受理 | 受理 | 受理 | 受理 |
| `invalid-syntax.tiny` | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 | 拒否/未取得 |
| `large-match.tiny` | 受理 | 受理 | 受理 | 受理 | 拒否/未取得 |
| `valid-basic.tiny` | 受理 | 受理 | 受理 | 受理 | 受理 |
| `valid-multiline.tiny` | 受理 | 受理 | 受理 | 受理 | 受理 |
| `valid-unicode.tiny` | 受理 | 受理 | 受理 | 受理 | 受理 |

## AST 差分（同じ入力を両方が受理した組のみ）


### ubnfc p4-java vs ubnfc p4-rust

| fixture | 差分数 | 内訳 | 例 |
|---|---:|---|---|
| `complex-x16.tiny` | 48 | 引用符=48 | `$.fields.declarations[1].fields.value.fields.left` "'jp'" / 'jp' |
| `complex-x4.tiny` | 12 | 引用符=12 | `$.fields.declarations[1].fields.value.fields.left` "'jp'" / 'jp' |
| `complex-x64.tiny` | 192 | 引用符=192 | `$.fields.declarations[1].fields.value.fields.left` "'jp'" / 'jp' |
| `complex.tiny` | 3 | 引用符=3 | `$.fields.declarations[1].fields.value.fields.left` "'jp'" / 'jp' |
| `flat-arithmetic.tiny` | 0 | — |  |
| `large-match.tiny` | 0 | — |  |
| `valid-basic.tiny` | 0 | — |  |
| `valid-multiline.tiny` | 0 | — |  |
| `valid-unicode.tiny` | 1 | 引用符=1 | `$.fields.expression.fields.value.fields.left` "'こんにちは😀'" / 'こんにちは😀' |

### ubnfc p4-rust vs tinyexpression-rs

| fixture | 差分数 | 内訳 | 例 |
|---|---:|---|---|
| `complex-x16.tiny` | 176 | 木の形=176 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'fields': {'left': None, 'op': ['11'], 'right |
| `complex-x4.tiny` | 44 | 木の形=44 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'fields': {'left': None, 'op': ['11'], 'right |
| `complex-x64.tiny` | 401 | 木の形=401 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'fields': {'left': None, 'op': ['11'], 'right |
| `complex.tiny` | 11 | 木の形=11 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'fields': {'left': None, 'op': ['10'], 'right |
| `flat-arithmetic.tiny` | 256 | 木の形=256 | `$.fields.expression.fields.value.fields.left.fields.left` {'fields': {'left': None, 'op': ['1'], 'right': []} |
| `large-match.tiny` | 130 | 木の形=130 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'fields': {'left': None, 'op': ['32'], 'right |
| `valid-basic.tiny` | 2 | 木の形=2 | `$.fields.expression.fields.value.fields.left.fields.left` {'fields': {'left': None, 'op': ['1'], 'right': []} |
| `valid-multiline.tiny` | 2 | 木の形=2 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'fields': {'left': None, 'op': ['10'], 'right |
| `valid-unicode.tiny` | 0 | — |  |

### 旧Java mapper vs ubnfc p4-rust

| fixture | 差分数 | 内訳 | 例 |
|---|---:|---|---|
| `complex-x16.tiny` | 0 | — |  |
| `complex-x4.tiny` | 0 | — |  |
| `complex.tiny` | 1 | span=1 | `$.span[1]` 326 / 325 |
| `flat-arithmetic.tiny` | 0 | — |  |
| `large-match.tiny` | 0 | — |  |
| `valid-basic.tiny` | 0 | — |  |
| `valid-multiline.tiny` | 0 | — |  |
| `valid-unicode.tiny` | 3 | span=3 | `$.fields.expression.fields.value.span[1]` 10 / 9 |

### 旧Java mapper vs ubnfc p4-java

| fixture | 差分数 | 内訳 | 例 |
|---|---:|---|---|
| `complex-x16.tiny` | 48 | 引用符=48 | `$.fields.declarations[1].fields.value.fields.left` 'jp' / "'jp'" |
| `complex-x4.tiny` | 12 | 引用符=12 | `$.fields.declarations[1].fields.value.fields.left` 'jp' / "'jp'" |
| `complex.tiny` | 4 | span=1, 引用符=3 | `$.fields.declarations[1].fields.value.fields.left` 'jp' / "'jp'" |
| `flat-arithmetic.tiny` | 0 | — |  |
| `large-match.tiny` | 0 | — |  |
| `valid-basic.tiny` | 0 | — |  |
| `valid-multiline.tiny` | 0 | — |  |
| `valid-unicode.tiny` | 4 | span=3, 引用符=1 | `$.fields.expression.fields.value.fields.left` 'こんにちは😀' / "'こんにちは😀'" |

### 旧Java mapper vs tinyexpression-rs

| fixture | 差分数 | 内訳 | 例 |
|---|---:|---|---|
| `complex-x16.tiny` | 176 | 木の形=176 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'type': 'BinaryExpr', 'span': [26, 28], 'fiel |
| `complex-x4.tiny` | 44 | 木の形=44 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'type': 'BinaryExpr', 'span': [26, 28], 'fiel |
| `complex.tiny` | 12 | span=1, 木の形=11 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'type': 'BinaryExpr', 'span': [25, 27], 'fiel |
| `flat-arithmetic.tiny` | 256 | 木の形=256 | `$.fields.expression.fields.value.fields.left.fields.left` {'type': 'BinaryExpr', 'span': [0, 1], 'fields': {' |
| `large-match.tiny` | 130 | 木の形=130 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'type': 'BinaryExpr', 'span': [25, 27], 'fiel |
| `valid-basic.tiny` | 2 | 木の形=2 | `$.fields.expression.fields.value.fields.left.fields.left` {'type': 'BinaryExpr', 'span': [0, 1], 'fields': {' |
| `valid-multiline.tiny` | 2 | 木の形=2 | `$.fields.declarations[0].fields.value.fields.left.fields.left` {'type': 'BinaryExpr', 'span': [25, 27], 'fiel |
| `valid-unicode.tiny` | 3 | span=3 | `$.fields.expression.fields.value.span[1]` 10 / 9 |
