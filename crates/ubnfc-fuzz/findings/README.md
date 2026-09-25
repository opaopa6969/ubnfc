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
| 回復の観測（issue #41 / D-069、`docs/reports/2026-09-24-fuzz-recovery-observations.md`） | `diff-diagnostics-expected-438fbb32-seed100276`、`diff-recoveries-captureoccurrences-cbee9ef9-seed100276`、`diff-diagnostics-farthestexpected-d88564a1-seed100276` |
| mixed な選択の text 候補を Rust が意味値にしない（issue #78 / D-078、`docs/reports/2026-09-24-nested-capture-78.md`） | `diff-ast-fields-tail-bf11b609-seed1000002` |
| 意味値の領域の中の text だけの規則を Rust が字句の値にする（issue #86、`docs/reports/2026-09-25-fuzz-86.md`） | `diff-ast-fields-tail-bf11b609-seed1000189` |
| node 変換の field で text だけの group の capture を Rust が規則ごとの字句に割る（issue #89、`docs/reports/2026-09-25-fuzz-89.md`。手で縮めた形で seed なし） | `diff-ast-fields-x-issue89` |
| C-1 / B-1 / A-5 / A-8 / A-9 観測契約の一本化（D-024 〜 D-029、`docs/reports/2026-09-23-observation-contract.md`） | `diff-catalogs-dd4854a9-seed1`、`diff-diagnostics-988b1c0e-seed1`、`diff-recoveries-7906c206-seed1`、`diff-scope-declarations-67b97a59-seed1`、`diff-scope-depth-2be9dc99-seed1`、`diff-scope-diagnostics-463a9328-seed1`、`diff-scope-references-3daa5d3e-seed1`、`diff-mappingerror-present-1278776d-seed1`、`diff-value-1e979fc2-seed1` |

seed 1 を元の recipe（`--seeds 1 --inputs 24 --shrink-builds 10`）で回すと、残るのは
`diff-captures-ccc0d8bb-seed1`（A-7、既知 signature）だけになる。
ただし `diff:/value` と `diff:/mappingError/present` は**クラスとしては別 seed に残る**
（seed 6。原因は A-4 で D-028 とは別）ので、消えたのは seed 1 の事例だけである。

これらの slug は jar を package し直さずに campaign を回すと**再び出る**。
driver template は `ubnfc-java` の jar resource なので、古い jar は古い契約の driver を作る。
D-033 でハーネスが jar の鮮度を検査するようになり、いまは差分ではなく
`discard [build:java]` として落ちる（`docs/reports/2026-09-23-fuzz-fixes-catalogs-diagnostics.md`）。

`*-seed100276` の 3 件は **v2 seed 帯の追加分**で、fuzzer の縮小結果をそのまま置いた
（`diff:/diagnostics/*/farthestExpected/*` は**クラスとしては `KNOWN` に残る** — seed 34 / 4001 に
別原因が残っているため。消えたのは seed 100276 の事例だけである）。

回帰試験は `tests/campaign.rs`（`b_recovery_auto_keeps_acceptance_identical_across_backends`、
`f_v2_recovery_diagnostics_agree_across_backends` と固定 seed campaign の `discarded == 0`）、
および `tests/recovery_diagnostics.rs`（文法・入力を固定した回復観測の比較）。

`diff-ast-fields-tail-bf11b609-seed1000002` は直接 capture の帯（#75）の fuzzer 縮小結果をそのまま置いた。
回帰試験は `tests/mixed_choice_text_value.rs`（文法を 4 形に縮めて固定し、値まで確認）と
`tests/campaign.rs` の g（seed 1000002 を固定帯へ戻した）。

`diff-ast-fields-tail-bf11b609-seed1000189` も同じ帯の縮小結果をそのまま置いた（signature は 1000002 と同じだが原因は別）。
回帰試験は `tests/text_rule_value_region.rs`（issue の形と、連接の中の text 規則の形を固定して値まで確認）と
`tests/campaign.rs` の g（seed 1000189 を固定帯に加えた）。
