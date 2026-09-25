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

## `json-v2.ir.json` — この repository の中にある v2 文法（D-063）

`examples/json/json.ubnf`（RFC 8259 の JSON を UBNF v2 で書いたもの）の canonical JSON 出力。
sibling repository に依存しない唯一の golden で、**v2 の code point クラス**が IR の
`TokenSpec.charRange` としてどう落ちるかをここで固定する
（`SAFE_SP` 32..33 / `SAFE_MID` 35..91 / `SAFE_BMP` 93..55295 / `SAFE_SUPP` 57344..1114111）。

```sh
# ubnfc リポジトリのルートから実行する。
cargo run -p ubnfc-front --bin ubnfc -- ir \
  --grammar examples/json/json.ubnf \
  --extern-first-chars scanners/first-chars.json \
  --out ir/examples/json-v2.ir.json
```

照合は `crates/ubnfc-front/tests/pipeline_golden.rs` の
`the_json_v2_grammar_matches_its_committed_ir_golden`。

## `stax.ir.json` — 型の付いた連接言語（issue #66 stage 1）

`examples/stax/stax.ubnf`（Forth 系の自己拡張型言語 stax）の canonical JSON 出力。
sibling repository に依存しない 2 本目の golden で、**scope 注釈**が IR へどう落ちるかを
ここで固定する（`@scopeTree(mode=lexical)` 3 箇所、`@declares` 8 箇所、
`@symbolRef` 4 箇所、`@catalog` 6 箇所、`@doc` 17 箇所、`@mapping` 27 箇所）。

```sh
# ubnfc リポジトリのルートから実行する。extern token は無いので registry は要らない。
cargo run -p ubnfc-front --bin ubnfc -- ir \
  --grammar examples/stax/stax.ubnf \
  --out ir/examples/stax.ir.json
```

照合は `crates/ubnfc-front/tests/pipeline_golden.rs` の
`the_stax_grammar_matches_its_committed_ir_golden`。
同じファイルの `the_stax_grammar_validates_strictly_clean` が `--strict` 警告ゼロを、
`the_stax_samples_all_have_an_expected_verdict` が `examples/stax/samples/` の 20 本と
`expected.json` の対応を固定する。

## `formula-info.ir.json` — 行指向の FormulaInfo 文書（issue #83）

`examples/formula-info/formula-info.ubnf`（tinyexpression の FormulaInfo 文書を UBNF v2 で
書いたもの。出典は tinyexpression issue #180 / PR #191、commit
`3389b056d6de095cc89c645cca34d5ddfe25593b`。tinyexpression → ubnfc の一方向で取込み、
ubnfc 側からは変更しない）の canonical JSON 出力。sibling repository に依存しない 3 本目の
golden で、**`@whitespace: none` の行指向文法**と、規則レベルの否定先読みが無い UBNF で
補集合を書くパターン（`NEGATION` 3 箇所、`NEGATIVE_LOOKAHEAD` 1 箇所、`LOOKAHEAD` 2 箇所）
が IR へどう落ちるかをここで固定する（`@doc` 9 箇所、`@mapping` 7 箇所）。

```sh
# ubnfc リポジトリのルートから実行する。extern token は無いので registry は要らない。
cargo run -p ubnfc-front --bin ubnfc -- ir \
  --grammar examples/formula-info/formula-info.ubnf \
  --out ir/examples/formula-info.ir.json
```

照合は `crates/ubnfc-front/tests/pipeline_golden.rs` の
`the_formula_info_grammar_matches_its_committed_ir_golden`。
同じファイルの `the_formula_info_grammar_validates_strictly_clean` が `--strict` 警告ゼロを、
`the_formula_info_samples_all_have_an_expected_verdict` が `examples/formula-info/samples/`
の 45 本と `expected.json` の対応を固定する。
