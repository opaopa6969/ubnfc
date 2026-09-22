#!/usr/bin/env python3
"""型・フィールド・span を省略せず比較する。入力順は旧 AST、生成 AST。"""
import json
import sys

def compare(old, new, path="$", differences=None):
    if differences is None:
        differences = []
    if type(old) is not type(new):
        differences.append({"path": path, "legacy": old, "generated": new})
    elif isinstance(old, dict):
        for key in sorted(old.keys() | new.keys()):
            if key not in old or key not in new:
                differences.append({"path": path + "." + key, "legacy": old.get(key, "<missing>"), "generated": new.get(key, "<missing>")})
            else:
                compare(old[key], new[key], path + "." + key, differences)
    elif isinstance(old, list):
        if len(old) != len(new):
            differences.append({"path": path + ".length", "legacy": len(old), "generated": len(new)})
        for index, (left, right) in enumerate(zip(old, new)):
            compare(left, right, path + f"[{index}]", differences)
    elif old != new:
        differences.append({"path": path, "legacy": old, "generated": new})
    return differences

if __name__ == "__main__":
    with open(sys.argv[1], encoding="utf-8") as stream:
        old = json.load(stream)
    with open(sys.argv[2], encoding="utf-8") as stream:
        new = json.load(stream)
    result = compare(old, new)
    print(json.dumps({"differenceCount": len(result), "differences": result}, ensure_ascii=False, indent=2))
    sys.exit(bool(result))
