# Classic の引数付き規則の corpus（写し）

`cases.json` は [unlaxer-parser](https://github.com/opaopa6969/unlaxer-parser) の
`spec-corpus/parameterized-rules/cases.json` を、commit
`d3ea73e8cfa440df8bdbb70abb29e06259435ba2`（2026-11-01 時点の master）から byte 単位で写したもの
（MIT License、著作権表示は unlaxer-parser の `LICENSE`）。

期待値（受理・拒否、AST、拒否の code）は Classic が手書きで持ち、Classic の
`ParameterizedRuleConformanceTest` と native Rust frontend のテストが満たしている。ubnfc は
これを oracle にして、Classic の綴り（`F(x) ::=` / `F(a)`、issue #11）で書いた文法を ubnfc で
生成した parser が、Classic と同じ入力を受理・拒否し、同じ AST を返すことを確かめる
（`crates/ubnfc-rust/tests/classic_parameterized.rs`）。

外部 fixture の pin（`scripts/external-pins.env`）とは独立に、この写しは CI で常に読む。
Classic 側の corpus が変わったら、同じ手順で写し直してテストを回す。
