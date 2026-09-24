# JSONTestSuite 突き合わせ

対象 318 件（benchmarks/external/JSONTestSuite/test_parsing）

| 実装 | y_ 受理 / 全 | n_ 拒否 / 全 | i_ 受理 | i_ 拒否 | suite との不一致 |
|---|---:|---:|---:|---:|---|
| ubnfc Rust | 95 / 95 | 188 / 188 | 20 | 15 | なし |
| ubnfc Java | 95 / 95 | 188 / 188 | 20 | 15 | なし |
| ANTLR4 | 95 / 95 | 188 / 188 | 21 | 14 | なし |
| tree-sitter | 95 / 95 | 188 / 188 | 22 | 13 | なし |

## 実装間で判定が割れたファイル（2 件）

| file | 期待 | ubnfc Rust | ubnfc Java | ANTLR4 | tree-sitter |
|---|---|---|---|---|---|
| `i_structure_500_nested_arrays.json` | 実装依存 | reject | reject | accept | accept |
| `i_structure_UTF-8_BOM_empty_object.json` | 実装依存 | reject | reject | reject | accept |
