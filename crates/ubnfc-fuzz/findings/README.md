# 差分 fuzzing の最小再現例

`docs/reports/2026-09-23-differential-fuzzing.md` の所見。1 ディレクトリが 1 つの
**差分クラス**（JSON Pointer の配列添字を伏せたもの、または不変条件の id）に対応する。

| ファイル | 内容 |
|---|---|
| `grammar.ubnf` | 最小化した文法。`ubnfc ir --grammar grammar.ubnf --out g.ir.json` でそのまま使える |
| `imported.ubnf` | `@import` がある場合の 2 ファイル目 |
| `input.txt` | 入力そのもの（NUL・非 BMP を含み得る） |
| `finding.json` | seed・signature・両 backend の生応答・差分の一覧 |
| `README.md` | 入力の JSON escape 表記、差分、不変条件違反 |

再生成:

```sh
cargo run -q -p ubnfc-fuzz --offline -- --seeds 320 --inputs 24 --jobs 3 \
  --shrink-builds 10 --findings crates/ubnfc-fuzz/findings
```

campaign が出した 92 クラスのうち、**A-2（入れ子ノード span の末尾 trivia、44 クラス）は
同じ原因が入れ子の深さぶん別クラスになる**ので、最小の 1 件
（`diff-ast-fields-left-fields-right-span-3fb5191f-seed5`）だけを残した。
残りは上のコマンドで再生成できる。

`front-*` の 2 件は runtime の差ではなく、生成文法が front に拒否された理由を
`--minimize-front` で追ったもの（手で書いた README を持つ）。
