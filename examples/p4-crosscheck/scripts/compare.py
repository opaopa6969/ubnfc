#!/usr/bin/env python3
"""P4 の 4 実装（ubnfc Rust / ubnfc Java / tinyexpression-rs / 旧 Java）を突き合わせる。

canonical AST はどれも `{"type","span","fields"}` 形だが、mapper の設計が違うので
差分は「引用符」「span 単位」「木の形」の 3 分類に振り分けて数える。正規化はしない。
"""
import json
import sys
from pathlib import Path

LABEL = {"r1": "ubnfc p4-rust", "j1": "ubnfc p4-java",
         "r2": "tinyexpression-rs", "j2-mapper": "旧Java mapper", "j2-facade": "旧Java facade"}


def ast_of(path):
    if not path.is_file():
        return None
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    if isinstance(document, dict) and "ok" in document:
        # tinyexpression-rs CLI 形式。失敗時は診断だけを返すので AST は無い。
        return document.get("ast") if document.get("ok") else None
    if isinstance(document, dict) and "ast" in document:
        return document["ast"]
    return document


def differences(left, right, path="$", found=None):
    if found is None:
        found = []
    if len(found) > 400:
        return found
    if type(left) is not type(right):
        found.append((path, left, right))
    elif isinstance(left, dict):
        for key in sorted(set(left) | set(right)):
            if key in left and key in right:
                differences(left[key], right[key], f"{path}.{key}", found)
            else:
                found.append((f"{path}.{key}", left.get(key, "<欠落>"), right.get(key, "<欠落>")))
    elif isinstance(left, list):
        if len(left) != len(right):
            found.append((f"{path}.length", len(left), len(right)))
        for index, (a, b) in enumerate(zip(left, right)):
            differences(a, b, f"{path}[{index}]", found)
    elif left != right:
        found.append((path, left, right))
    return found


def classify(path, left, right):
    if isinstance(left, str) and isinstance(right, str) and (
            left.strip("'\"") == right.strip("'\"")) and left != right:
        return "引用符"
    if path.endswith("]") and ".span" in path:
        return "span"
    if isinstance(left, int) and isinstance(right, int):
        return "span/個数"
    return "木の形"


def main():
    out = Path(sys.argv[1])
    fixtures = sorted(Path(sys.argv[2]).glob("*.tiny"))
    names = [f.name for f in fixtures]
    kinds = ["r1", "j1", "r2", "j2-mapper", "j2-facade"]
    asts = {kind: {name: ast_of(out / kind / f"{name}.json") for name in names} for kind in kinds}

    print("# P4 4 実装クロスチェック（自動生成）\n")
    print("## 受理（AST を得られたか）\n")
    print("| fixture | " + " | ".join(LABEL[k] for k in kinds) + " |")
    print("|---|" + "---|" * len(kinds))
    for name in names:
        print(f"| `{name}` | " + " | ".join(
            "受理" if asts[k][name] is not None else "拒否/未取得" for k in kinds) + " |")

    print("\n## AST 差分（同じ入力を両方が受理した組のみ）\n")
    pairs = [("j1", "r1"), ("r1", "r2"), ("j2-mapper", "r1"), ("j2-mapper", "j1"),
             ("j2-mapper", "r2")]
    for left, right in pairs:
        print(f"\n### {LABEL[left]} vs {LABEL[right]}\n")
        print("| fixture | 差分数 | 内訳 | 例 |")
        print("|---|---:|---|---|")
        for name in names:
            a, b = asts[left][name], asts[right][name]
            if a is None or b is None:
                continue
            found = differences(a, b)
            counts = {}
            for path, x, y in found:
                counts[classify(path, x, y)] = counts.get(classify(path, x, y), 0) + 1
            sample = ""
            if found:
                path, x, y = found[0]
                sample = f"`{path}` {x!r} / {y!r}"[:110].replace("|", "\\|")
            print(f"| `{name}` | {len(found)} | "
                  + (", ".join(f"{k}={v}" for k, v in sorted(counts.items())) or "—")
                  + f" | {sample} |")


if __name__ == "__main__":
    main()
