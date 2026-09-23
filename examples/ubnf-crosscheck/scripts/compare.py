#!/usr/bin/env python3
"""4 実装の受理結果と正準 AST を突き合わせ、Markdown 表と差分一覧を出す。

A ubnfc handwritten / B ubnfc selfhost / C unlaxer-ubnf / D Java bootstrap。
AST は 3 つの Rust/Java 実装が同じ綴り（unlaxer canonical）へ射影済み。
"""
import json
import sys
from pathlib import Path

NAMES = {"A": "ubnfc-front(手書き)", "B": "ubnfc-selfhost(生成)",
         "C": "unlaxer-ubnf(Rust)", "D": "Java bootstrap"}


def load(directory):
    cases = {}
    for line in (directory / "ubnfc.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        name = Path(row["input"]).name
        cases.setdefault(name, {})["A"] = {
            "ok": row["handwritten"]["ok"],
            "ast": row["handwritten"].get("ast"),
            "offset": row["handwritten"].get("offset"),
            "message": row["handwritten"].get("message"),
        }
        cases[name]["B"] = {
            "ok": row["selfhost"]["ok"],
            "ast": row["selfhost"].get("ast"),
            "offset": row["selfhost"].get("offset"),
            "message": row["selfhost"].get("message"),
        }
    for line in (directory / "unlaxer-ubnf.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        result = row["result"]
        cases.setdefault(row["input"], {})["C"] = {
            "ok": "ast" in result,
            "ast": result.get("ast"),
            # inspect の span は [byte_start, byte_end, cp_start, cp_end]
            "offset": result["span"][0] if "span" in result else None,
            "message": result.get("error"),
        }
    for line in (directory / "java-bootstrap.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        cases.setdefault(Path(row["input"]).name, {})["D"] = {
            "ok": row["ok"],
            "ast": row.get("ast"),
            "offset": None,  # Java bootstrap は失敗位置を公開しない
            "message": row.get("error"),
        }
    return cases


def main():
    directory = Path(sys.argv[1])
    cases = load(directory)
    order = sorted(cases)
    acceptance_disagreements = []
    ast_disagreements = []
    for name in order:
        case = cases[name]
        accepted = {key: case[key]["ok"] for key in "ABCD"}
        if len(set(accepted.values())) > 1:
            acceptance_disagreements.append((name, accepted))
        asts = {key: json.dumps(case[key]["ast"], ensure_ascii=False, sort_keys=True)
                for key in "ABCD" if case[key]["ok"]}
        if len(set(asts.values())) > 1:
            groups = {}
            for key, value in asts.items():
                groups.setdefault(value, []).append(key)
            ast_disagreements.append((name, sorted(groups.values()), case))

    print("# UBNF 4 実装クロスチェック（自動生成）\n")
    print(f"入力 {len(order)} 件（`corpus.tsv` 参照）。\n")
    print("## 受理・拒否\n")
    print("| 入力 | " + " | ".join(NAMES[k] for k in "ABCD") + " |")
    print("|---|" + "---|" * 4)
    for name in order:
        cells = []
        for key in "ABCD":
            entry = cases[name][key]
            if entry["ok"]:
                cells.append("受理")
            elif entry["offset"] is not None and entry["offset"] >= 0:
                cells.append(f"拒否@{entry['offset']}")
            else:
                cells.append("拒否")
        print(f"| `{name}` | " + " | ".join(cells) + " |")

    print(f"\n## 受理判定の不一致 {len(acceptance_disagreements)} 件\n")
    for name, accepted in acceptance_disagreements:
        yes = [NAMES[k] for k in "ABCD" if accepted[k]]
        no = [NAMES[k] for k in "ABCD" if not accepted[k]]
        print(f"- `{name}`: 受理 = {', '.join(yes) or 'なし'} / 拒否 = {', '.join(no) or 'なし'}")
        for key in "ABCD":
            if not accepted[key]:
                print(f"    - {NAMES[key]}: {cases[name][key]['message']}")

    print(f"\n## 受理したが AST が食い違う入力 {len(ast_disagreements)} 件\n")
    for name, groups, case in ast_disagreements:
        print(f"- `{name}`: 一致群 = " + " / ".join("+".join(g) for g in groups))
        keys = [g[0] for g in groups]
        for left, right in zip(keys, keys[1:]):
            print(f"    - {NAMES[left]} vs {NAMES[right]}: "
                  + first_difference(case[left]["ast"], case[right]["ast"]))

    print("\n## 拒否位置（byte offset）が食い違う入力\n")
    for name in order:
        case = cases[name]
        offsets = {key: case[key]["offset"] for key in "ABC"
                   if not case[key]["ok"] and case[key]["offset"] is not None}
        if len(set(offsets.values())) > 1:
            print(f"- `{name}`: " + ", ".join(f"{NAMES[k]}={v}" for k, v in offsets.items()))


def first_difference(left, right, path="$"):
    if type(left) is not type(right):
        return f"{path}: 型が違う {left!r} / {right!r}"
    if isinstance(left, list):
        if len(left) != len(right):
            return f"{path}: 要素数 {len(left)} / {len(right)} — {left!r} vs {right!r}"[:400]
        for index, (a, b) in enumerate(zip(left, right)):
            if a != b:
                return first_difference(a, b, f"{path}[{index}]")
        return f"{path}: 一致"
    if left != right:
        return f"{path}: {left!r} / {right!r}"
    return f"{path}: 一致"


if __name__ == "__main__":
    main()
