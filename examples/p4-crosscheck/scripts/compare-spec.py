#!/usr/bin/env python3
"""P4 仕様由来ケースの期待値と実装の受理を突き合わせる。

期待値は corpus/spec/p4/*.json（tinyexpression の規範文書から導出）。
実装の観測値は期待値に一切使わない。
"""
import json
import sys
from pathlib import Path

NAMES = {"R1": "ubnfc p4-rust", "R2": "tinyexpression-rs"}
MATCH, VIOLATE, OPEN, NOTRUN = "一致", "違反", "未定", "駆動不能"


def read_tsv(path):
    values = {}
    if not path.is_file():
        return values
    for line in path.read_text(encoding="utf-8").splitlines():
        parts = line.split("\t")
        if len(parts) >= 2:
            values[parts[0]] = parts[1]
    return values


def main():
    case_root, out = Path(sys.argv[1]), Path(sys.argv[2])
    observed = {"R1": read_tsv(out / "r1.tsv"), "R2": read_tsv(out / "r2.tsv")}
    documents = [(p.stem, json.loads(p.read_text(encoding="utf-8")))
                 for p in sorted(case_root.glob("*.json"))]

    rows = []
    for section, document in documents:
        for case in document["cases"]:
            verdicts, notes = {}, {}
            for key in ("R1", "R2"):
                result = observed[key].get(case["id"])
                if case["scope"] == "runtime" or result is None:
                    verdicts[key], notes[key] = NOTRUN, "駆動していない"
                    continue
                notes[key] = "受理" if result == "ok" else "拒否"
                wanted = case["expectation"]["verdict"]
                if wanted == "undetermined":
                    verdicts[key] = OPEN
                elif wanted == "accept":
                    verdicts[key] = MATCH if result == "ok" else VIOLATE
                elif wanted == "reject":
                    verdicts[key] = MATCH if result != "ok" else VIOLATE
                else:
                    verdicts[key] = NOTRUN
            rows.append({"section": section, "case": case,
                         "verdicts": verdicts, "notes": notes})

    print("# P4 仕様由来オラクル — 実装の突き合わせ（自動生成）\n")
    print("期待値は tinyexpression の `spec/SPEC.md` / `specs/language.md` の規範文から導いたもので、")
    print("どの実装の観測値も使っていない（`corpus/spec/README.md`）。\n")
    print(f"ケース {len(rows)} 件。J1 / J2（Java 2 種）はここでは駆動していない。\n")

    print("## 判定の内訳\n")
    print("| 実装 | 一致 | 違反 | 未定 | 駆動不能 |")
    print("|---|---:|---:|---:|---:|")
    for key in ("R1", "R2"):
        counts = {v: sum(1 for r in rows if r["verdicts"][key] == v)
                  for v in (MATCH, VIOLATE, OPEN, NOTRUN)}
        print(f"| {NAMES[key]} | {counts[MATCH]} | {counts[VIOLATE]} | "
              f"{counts[OPEN]} | {counts[NOTRUN]} |")

    print("\n## ケース × 実装\n")
    print("| ケース | scope | 期待 | " + " | ".join(NAMES[k] for k in ("R1", "R2")) + " | 引用 |")
    print("|---|---|---|---|---|---|")
    for row in rows:
        case = row["case"]
        citation = case["citation"]
        where = f"{citation['file']} {citation['heading']}"
        cells = " | ".join(f"{row['verdicts'][k]}（{row['notes'][k]}）" for k in ("R1", "R2"))
        print(f"| `{case['id']}` | {case['scope']} | {case['expectation']['verdict']} | "
              f"{cells} | {where} |")

    print("\n## 未定（仕様が沈黙・矛盾）\n")
    for row in rows:
        if row["case"]["expectation"]["verdict"] == "undetermined":
            print(f"- `{row['case']['id']}`: {row['case']['expectation']['question']}")

    with open(out / "verdicts.json", "w", encoding="utf-8") as handle:
        json.dump([{"id": r["case"]["id"], "scope": r["case"]["scope"],
                    "expectation": r["case"]["expectation"],
                    "verdicts": r["verdicts"], "observed": r["notes"]}
                   for r in rows], handle, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
