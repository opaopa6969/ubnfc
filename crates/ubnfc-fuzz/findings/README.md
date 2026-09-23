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

## 修正済みの所見（2026-09-23、`docs/reports/2026-09-23-fuzz-fixes-front-recovery.md`）

再現しなくなったが、経緯として残す。seed を指定した再実行でも該当 signature は出ない。

| 根本原因 | 再現しなくなった slug |
|---|---|
| A-1 `@recovery(AUTO)` の適用（受理そのものが変わる） | `diff-accepted-02147df8-seed269`、`diff-allconsumed-f9a21922-seed269`、`diff-consumed-ce9f4647-seed269`、`diff-matched-5923f95f-seed269`、`diff-parsedstatus-f5944912-seed269`、`diff-captures-span-94600c41-seed269`、`diff-captures-text-004b783b-seed269`、`diff-prefixcaptures-887bb90f-seed269`、`diff-prefixcaptures-span-039546d5-seed269`、`diff-prefixcaptures-text-445a73b7-seed269`、`diff-captures-name-4bfa2e37-seed272`、`diff-prefixcaptures-binding-80d2f5e1-seed272`、`diff-prefixcaptures-name-8f46bda3-seed272` |
| D front が自分の契約に落ちる | `front-ir-recipe-cardinality`、`front-capture-cardinality-bounded`（seed 1..320 の front discard は 28 → 0） |
| C-1 / B-1 / A-5 / A-8 / A-9 観測契約の一本化（D-024 〜 D-029、`docs/reports/2026-09-23-observation-contract.md`） | `diff-catalogs-dd4854a9-seed1`、`diff-diagnostics-988b1c0e-seed1`、`diff-recoveries-7906c206-seed1`、`diff-scope-declarations-67b97a59-seed1`、`diff-scope-depth-2be9dc99-seed1`、`diff-scope-diagnostics-463a9328-seed1`、`diff-scope-references-3daa5d3e-seed1`、`diff-mappingerror-present-1278776d-seed1`、`diff-value-1e979fc2-seed1` |

seed 1 を元の recipe（`--seeds 1 --inputs 24 --shrink-builds 10`）で回すと、残るのは
`diff-captures-ccc0d8bb-seed1`（A-7、既知 signature）だけになる。
ただし `diff:/value` と `diff:/mappingError/present` は**クラスとしては別 seed に残る**
（seed 6。原因は A-4 で D-028 とは別）ので、消えたのは seed 1 の事例だけである。

これらの slug は jar を package し直さずに campaign を回すと**再び出る**。
driver template は `ubnfc-java` の jar resource なので、古い jar は古い契約の driver を作る。
D-033 でハーネスが jar の鮮度を検査するようになり、いまは差分ではなく
`discard [build:java]` として落ちる（`docs/reports/2026-09-23-fuzz-fixes-catalogs-diagnostics.md`）。

回帰試験は `tests/campaign.rs`（`b_recovery_auto_keeps_acceptance_identical_across_backends` と
固定 seed campaign の `discarded == 0`）。
