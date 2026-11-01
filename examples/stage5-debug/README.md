# EvaluatorBridge の差し込み例

`stage5.ubnf` は `let 名前 = 数値;`、`use 名前;`、`{ ... }`、`note "文字列";` だけの小さな文法。
`src/SumEvaluatorBridge.java` が生成される `EvaluatorBridge` を実装し、全 `let` の数値を加算する。
生成された parser / AST / DebugAdapter を継承・改変する必要はない。

```java
var bridge = EvaluatorBridge.load(SumEvaluatorBridge.class.getName());
var adapter = new DebugAdapter(Map.of(), bridge);
```

`onLaunch` は型付き AST をたどって `total` を初期化する。`variables` に `total` が現れ、
Debug Console の `evaluate("total")` が同じ値を返す。未知の式は `Optional.empty()` を返して
既存の明示エラーに委ねる。これは AST 全体を集計する例で、ステップに従う逐次実行器ではない。
`use` は LSP の索引を実演するためだけの構文であり、評価値を変更しない。

生成と動作の再現（repository root、JDK 21 / Maven）:

```sh
mvn -o -Plsp -pl ubnfc-lsp -am test \
  -Dtest=Stage5GoldenTest,ParserUnchangedTest -Dsurefire.failIfNoSpecifiedTests=false
```

テストは同梱の `stage5.ir.json` から parser・LSP・DAP を一時ディレクトリへ生成し、
この bridge と一緒にコンパイルして、実際の JSON-RPC で launch / variables / evaluate を往復する。
入力 `let alpha = 2; { let beta = 3; }` の `total` は `5`、再 launch の
`let single = 7;` では `7`。既定 `NOOP` の evaluate がエラーのままであることも固定する。
生成された `DapMain` の起動時には、このクラスを同じ classpath に含めて
`--evaluator org.unlaxer.ubnfc.examples.stage5.SumEvaluatorBridge` を指定できる
（または `-Dubnfc.dap.evaluator=同じクラス名`）。

IR の再生成（文法を変更した場合だけ）:

```sh
RUSTUP_TOOLCHAIN=stable cargo run -q -p ubnfc-front --bin ubnfc -- \
  ir --grammar examples/stage5-debug/stage5.ubnf --out examples/stage5-debug/stage5.ir.json
```

LSP / DAP は Java 側だけの opt-in feature。Rust backend に LSP / DAP はない。
