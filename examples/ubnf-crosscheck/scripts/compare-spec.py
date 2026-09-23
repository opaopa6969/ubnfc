#!/usr/bin/env python3
"""仕様由来ケース集の期待値と 4 実装の実測を突き合わせ、Markdown 表を出す。

期待値は corpus/spec/ubnf/*.json（規範文から導出）。実装の観測値は期待値に一切使わない。
判定は 5 つ: 一致 / 違反 / 別コード / 未定（仕様が沈黙している）/ 駆動不能。
"""
import json
import sys
from pathlib import Path

NAMES = {"A": "ubnfc-front(手書き)", "B": "ubnfc-selfhost(生成)",
         "C": "unlaxer-ubnf(Rust)", "D": "Java bootstrap"}
MATCH, VIOLATE, OPEN, NOTRUN = "一致", "違反", "未定", "駆動不能"
ALTCODE = "別コード"
# 構文では受理するが、同じ道具の validate 段で ERROR として拒否する場合。
# 仕様は「拒否する」としか言っておらず、どの層で拒否するかは定めていない。
LATEREJECT = "一致(validate)"


def file_name(case_id):
    return case_id.replace("/", "__") + ".ubnf"


def load_cases(root):
    documents = []
    for path in sorted(Path(root).glob("*.json")):
        documents.append((path.stem, json.loads(path.read_text(encoding="utf-8"))))
    return documents


def load_observations(out):
    out = Path(out)
    parse = {}
    for line in (out / "ubnfc.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        name = Path(row["input"]).name
        parse.setdefault(name, {})["A"] = {
            "ok": row["handwritten"]["ok"], "ast": row["handwritten"].get("ast"),
            "message": row["handwritten"].get("message")}
        parse[name]["B"] = {
            "ok": row["selfhost"]["ok"], "ast": row["selfhost"].get("ast"),
            "message": row["selfhost"].get("message")}
    for line in (out / "unlaxer-ubnf.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        result = row["result"]
        parse.setdefault(row["input"], {})["C"] = {
            "ok": "ast" in result, "ast": result.get("ast"),
            "message": result.get("error")}
    for line in (out / "java-bootstrap.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        parse.setdefault(Path(row["input"]).name, {})["D"] = {
            "ok": row["ok"], "ast": row.get("ast"), "message": row.get("error")}
    java_validate = {}
    for line in (out / "java-validate.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        java_validate[Path(row["input"]).name] = row
    ubnfc_validate = {}
    for line in (out / "ubnfc-validate.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        ubnfc_validate[row["input"]] = row
    return parse, java_validate, ubnfc_validate


def canonical_text(ast):
    return json.dumps(ast, ensure_ascii=False, separators=(",", ":"))


def parse_verdict(case, entry, by_id, parse, key):
    """構文スコープの 1 実装ぶんの判定と、観測の短い説明を返す。"""
    expectation = case["expectation"]
    verdict = expectation["verdict"]
    if entry is None:
        return NOTRUN, "観測なし"
    observed = "受理" if entry["ok"] else "拒否"
    if verdict == "undetermined":
        return OPEN, observed
    if verdict == "reject":
        return (MATCH if not entry["ok"] else VIOLATE), observed
    if verdict != "accept":
        return NOTRUN, observed
    if not entry["ok"]:
        return VIOLATE, "拒否: " + str(entry.get("message"))[:80]
    contains = expectation.get("canonicalContains")
    if contains:
        text = canonical_text(entry["ast"])
        missing = [fragment for fragment in contains if fragment not in text]
        if missing:
            return VIOLATE, "受理したが構造が違う: " + missing[0]
    other = expectation.get("sameCanonicalAs")
    if other:
        reference = parse.get(file_name(other), {}).get(key)
        if reference is None or not reference["ok"]:
            return NOTRUN, "比較先が観測できない"
        if canonical_text(reference["ast"]) != canonical_text(entry["ast"]):
            return VIOLATE, f"受理したが {other} と構文木が違う"
    return MATCH, observed


def late_codes(row):
    """その入力に対して validator が出した診断コード。駆動できなければ空。"""
    if not row:
        return []
    if "report" in row:
        report = row["report"]
        return [i["code"] for i in report["issues"]] if isinstance(report, dict) else []
    return row.get("codes", []) if row.get("ok") else []


def validate_verdict(case, codes, available):
    expected = case["expectation"]["code"]
    if not available:
        return NOTRUN, "validator を駆動できない"
    if codes is None:
        return NOTRUN, "構文解析で落ちた"
    shown = ",".join(codes) or "診断なし"
    if expected in codes:
        return MATCH, shown
    # 仕様と違うコードでも検出はしている場合を、見落としと区別する。
    if any(code.startswith("E-") for code in codes):
        return ALTCODE, shown
    return VIOLATE, shown


def main():
    case_root, out = sys.argv[1], sys.argv[2]
    documents = load_cases(case_root)
    parse, java_validate, ubnfc_validate = load_observations(out)
    by_id = {c["id"]: c for _, d in documents for c in d["cases"]}

    rows = []
    for section, document in documents:
        for case in document["cases"]:
            name = file_name(case["id"])
            entries = parse.get(name, {})
            verdicts, notes = {}, {}
            for key in "ABCD":
                if case["scope"] == "runtime":
                    verdicts[key], notes[key] = NOTRUN, "生成パーサの実行時挙動"
                elif case["scope"] == "validate":
                    if key == "A":
                        report = ubnfc_validate.get(name, {}).get("report")
                        codes = ([i["code"] for i in report["issues"]]
                                 if isinstance(report, dict) else None)
                        verdicts[key], notes[key] = validate_verdict(case, codes, True)
                    elif key == "D":
                        row = java_validate.get(name, {})
                        codes = row.get("codes") if row.get("ok") else None
                        verdicts[key], notes[key] = validate_verdict(case, codes, True)
                    else:
                        verdicts[key], notes[key] = NOTRUN, "独立した validator を持たない"
                else:
                    verdicts[key], notes[key] = parse_verdict(
                        case, entries.get(key), by_id, parse, key)
            # reject 期待を構文で取りこぼした実装でも、自分の validate 段が ERROR を
            # 出していれば「拒否する」という規範文は満たしている。層の違いを違反と混ぜない。
            for key, source in (("A", ubnfc_validate), ("D", java_validate)):
                if case["scope"] != "parse" or verdicts[key] != VIOLATE:
                    continue
                if case["expectation"]["verdict"] != "reject":
                    continue
                codes = late_codes(source.get(name))
                if any(code.startswith("E-") for code in codes):
                    verdicts[key] = LATEREJECT
                    notes[key] = "構文では受理、validate が拒否: " + ",".join(
                        code for code in codes if code.startswith("E-"))
            # 自己ホスト front は lower/validate を手書き front と共有するが、
            # この経路を単体で駆動する入口が無いので推測で埋めない。
            if verdicts["A"] == LATEREJECT and verdicts["B"] == VIOLATE:
                verdicts["B"] = NOTRUN
                notes["B"] = "validate は手書き front 経由でしか駆動できない"
            rows.append({"section": section, "case": case,
                         "verdicts": verdicts, "notes": notes})

    print("# UBNF 仕様由来オラクル — 4 実装の突き合わせ（自動生成）\n")
    print("期待値は `unlaxer-dsl/specs/*.md` の規範文から導いたもので、")
    print("どの実装の観測値も使っていない（`corpus/spec/ubnf/README.md`）。\n")
    print(f"ケース {len(rows)} 件。判定は 一致 / 一致(validate) / 違反 / 別コード / 未定（仕様が沈黙）/ 駆動不能。\n")

    print("## 1. 節ごとの件数と違反数\n")
    print("| 節 | ケース | " + " | ".join(NAMES[k] + " 違反" for k in "ABCD") + " |")
    print("|---|---:|" + "---:|" * 4)
    for section, document in documents:
        group = [r for r in rows if r["section"] == section]
        cells = [str(sum(1 for r in group if r["verdicts"][k] == VIOLATE)) for k in "ABCD"]
        print(f"| `{section}` ({document['section']}) | {len(group)} | " + " | ".join(cells) + " |")
    totals = [str(sum(1 for r in rows if r["verdicts"][k] == VIOLATE)) for k in "ABCD"]
    print(f"| **合計** | **{len(rows)}** | " + " | ".join(f"**{t}**" for t in totals) + " |")

    counts = {k: {v: sum(1 for r in rows if r["verdicts"][k] == v)
                  for v in (MATCH, LATEREJECT, VIOLATE, ALTCODE, OPEN, NOTRUN)} for k in "ABCD"}
    print("\n## 2. 判定の内訳\n")
    print("| 実装 | 一致 | 一致(validate) | 違反 | 別コード | 未定 | 駆動不能 |")
    print("|---|---:|---:|---:|---:|---:|---:|")
    for k in "ABCD":
        c = counts[k]
        print(f"| {NAMES[k]} | {c[MATCH]} | {c[LATEREJECT]} | {c[VIOLATE]} | {c[ALTCODE]} | "
              f"{c[OPEN]} | {c[NOTRUN]} |")
    print("\n`一致(validate)` は構文解析では受理するが同じ道具の validate 段が ERROR で拒否する場合。")
    print("仕様は「拒否する」としか書いておらず、どの層で拒否するかは定めていない。")
    print("`別コード` は validation.md のコード名と違う名前で同じ違反を検出した場合。")

    print("\n## 3. ケース × 実装\n")
    print("| ケース | scope | 期待 | " + " | ".join(NAMES[k] for k in "ABCD") + " |")
    print("|---|---|---|" + "---|" * 4)
    for row in rows:
        case = row["case"]
        expected = case["expectation"]["verdict"]
        if expected == "diagnostic":
            expected = case["expectation"]["code"]
        cells = " | ".join(row["verdicts"][k] for k in "ABCD")
        print(f"| `{case['id']}` | {case['scope']} | {expected} | {cells} |")

    print("\n## 4. 実装別の違反一覧（最小入力つき）\n")
    for k in "ABCD":
        violations = [r for r in rows if r["verdicts"][k] == VIOLATE]
        print(f"### {NAMES[k]} — {len(violations)} 件\n")
        if not violations:
            print("なし\n")
            continue
        print("| ケース | 入力 | 期待 | 実測 | 引用 |")
        print("|---|---|---|---|---|")
        for row in violations:
            case = row["case"]
            expectation = case["expectation"]
            expected = expectation.get("code", expectation["verdict"])
            source = case["input"].replace("\n", "\\n").replace("|", "\\|")
            if len(source) > 70:
                source = source[:67] + "…"
            citation = case["citation"]
            where = f"{Path(citation['file']).name} {citation['heading']}"
            print(f"| `{case['id']}` | `{source or '(空ファイル)'}` | {expected} | "
                  f"{row['notes'][k].replace('|', '/')} | {where} |")
        print()

    open_rows = [r for r in rows if r["case"]["expectation"]["verdict"] == "undetermined"]
    print(f"## 5. 未定（仕様が沈黙・矛盾） {len(open_rows)} 件\n")
    print("| ケース | 問い | " + " | ".join(NAMES[k] for k in "ABCD") + " |")
    print("|---|---|" + "---|" * 4)
    for row in open_rows:
        case = row["case"]
        cells = " | ".join(row["notes"][k].replace("|", "/") for k in "ABCD")
        question = case["expectation"]["question"].replace("\n", " ")
        print(f"| `{case['id']}` | {question} | {cells} |")

    with open(Path(out) / "verdicts.json", "w", encoding="utf-8") as handle:
        json.dump([{"id": r["case"]["id"], "section": r["section"],
                    "scope": r["case"]["scope"],
                    "expectation": r["case"]["expectation"],
                    "verdicts": r["verdicts"], "observed": r["notes"]}
                   for r in rows], handle, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
