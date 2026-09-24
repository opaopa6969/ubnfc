# パリティ入力の出所

`te-formulas.json` は tinyexpression（`c70416e1`）の `src/test/**` ・ `src/jmh/**` ・
`rust/tinyexpression-rs/tests/fixtures/**` ・ `benchmarks/fixtures/**` から、
`P4PreferredAstMapper` の 4 入口に実際に渡される式を集めたもの。324 件（うち
パース失敗を期待するもの 17 件）。重複は `(source, preferredType, preferredAstSimpleName)` で除いた。

各要素:

| 欄 | 意味 |
|---|---|
| `source` | 式そのもの（byte を変えていない） |
| `origin` | 採取元の `path:line` |
| `preferredType` | `ExpressionTypes` の定数名。無指定は `null` |
| `preferredAstSimpleName` | `parseByAstSimpleName*` に渡す名前。無指定は `null` |
| `negative` | テストがパース失敗を期待しているか |

間接呼び出し（`DslGeneratedAstJavaEmitter.tryEmit` → `parseDetailed`、
`AstEvaluatorCalculator` / `GeneratedAstRuntimeProbe` → `parseByAstSimpleNames*`、
`P4ParseProbe` → `parseDetailed`）を経由するテストのフィクスチャも辿って入れてある。

含めなかったもの:

- `P4PreferredAstMapperPrecedenceTest` の式。呼ぶのは候補名 API だけで解析しない。
- `P4RustSharedFixtureAcceptanceTest` の `.tiny`。facade を通らず
  `TinyExpressionP4Parsers` / `TinyExpressionP4Mapper` を直接叩く。
- `formulaInfo-test/**`。どの backend に行くかが実行時の選択次第で静的に決まらない
  （`StageTimingRunner` が別途 `FormulaInfo` 経路として測る）。

`ParityTest` はこれに `examples/p4-java/src/test/resources/fixtures/*.tiny`（16 件）を足して走る。
