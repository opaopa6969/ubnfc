# 固定した IR（外部 pin から切り離した fixture）

`ir/fixtures/` の golden は、外部 fixture の pin（`scripts/external-pins.env`）が指す文法から作る。
pin を上げると文法が変わり、golden も変わる（`docs/ci-external-fixtures.md`）。

backend の回帰テスト・commit 済みの生成物・ベンチマークは、文法の変化とは無関係に同じ文法を
使い続けたい。そのためにここへ固定した IR を置く。

| ファイル | 元の文法 | 使う側 |
|---|---|---|
| `tinyexpression-p4-f86ce8a5.ir.json` | tinyexpression `f86ce8a5`（#210 の merge）の `tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf`（v1、124 規則）。2026-11-01 に pin を上げる前の `ir/fixtures/` の P4 golden と byte 同一 | `examples/p4-rust`（commit 済み生成物と bench）、`examples/p4-java`、`examples/p4-v2`、`crates/ubnfc-rust/tests/round2.rs`、`crates/ubnfc-front/tests/{memo_plan,ubnf_v2}.rs`、`ubnfc-java` / `ubnfc-lsp` の Java テスト、`scripts/build-vsix.sh`、`benchmarks/scripts/` |

IR の形式（`ir/schema/`）を変えたときは、元の文法から作り直す。

```sh
d=$(mktemp -d) && git -C "$d" init -q && git -C "$d" remote add origin https://github.com/opaopa6969/tinyexpression.git
git -C "$d" fetch -q --depth 1 origin f86ce8a5ab0ab7d23fdba2cd477187df8aa7607c && git -C "$d" checkout -q FETCH_HEAD
cargo build -p ubnfc-front --bin ubnfc
(cd "$d" && "$OLDPWD/target/debug/ubnfc" ir --grammar tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf \
    --extern-first-chars "$OLDPWD/scanners/first-chars.json") > ir/frozen/tinyexpression-p4-f86ce8a5.ir.json
```
