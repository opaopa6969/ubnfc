# Grammar IR の CLI 生成例

`tinycalc.ir.json` は `ubnfc ir` の canonical JSON 出力そのもの。
生成元は unlaxer-parser の `unlaxer-dsl/examples/tinycalc.ubnf`。
元ソースの CRLF・Unicode は変更せず、sha256 と code point span に反映する。

```sh
# ubnfc リポジトリのルートから実行する。
UNLAXER_PARSER_DIR=${UNLAXER_PARSER_DIR:-../unlaxer-parser}
cargo run -p ubnfc-front --bin ubnfc -- ir \
  --grammar "$UNLAXER_PARSER_DIR/unlaxer-dsl/examples/tinycalc.ubnf" \
  --out ir/examples/tinycalc.ir.json
```

生成元 revision: `ed4178e673f93e719b62a7bd181f3a219abf8ed1`（unlaxer-parser）。
frontend の基点: `76848d4665e974622943cd710209fec60b635013`。
生成器はこの README と同じ `feat/front-cli` の変更。再生成 revision は
`git log -1 --format=%H -- crates/ubnfc-front/src/pipeline.rs` で取得できる。
時刻・Git revision は IR 自体に埋め込まない。

論理ソースルートは main の最寄りの Git ルート（`.git` file/dir）。Git 外では
main の親を使い、import が外にあれば全ソースを含む共通親まで広げる。
同じソースルートを別ディレクトリへ移動しても sourceId と ExprId を維持する。
パスは POSIX、`.` / `..` を字句的に正規化する。
配列の宣言順は保持し、集合・object キーだけを直列化契約順にする。

旧例からの変更:

- source path / ID、semantics は旧例と一致する。
- 葉13式の mayStart を exact に、16式の failure を restore に、18式の
  flagEffect を preserve に精密化。
- AddOp / MulOp の2選択を predict にし、proof と既存3 trie の参照を記録。
- 4 literal の memo を証明付きで省略。2 SCC の memoPolicy と plan.profile を更新。
- 演算子は tierFunctions を維持し、trivia のため climbing を使えない理由を記録。
- requiredFeatures に predictiveChoice を追加。

7 rules、40 expressions、11 captures、27 trivia 境界、3 tries は従来どおり。
`pipeline_golden.rs` は CLI の再実行による byte 一致、別 cwd・絶対パスとの一致、
`ir::from_json` の型往復、`ir::validate_references` を検査する。
JSON Schema validator は使用しない。
