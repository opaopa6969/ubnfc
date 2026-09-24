#!/usr/bin/env python3
"""速度比較用の JSON 入力を JSONTestSuite の y_* から決定的に組み立てる（issue #26）。

JSONTestSuite の y_* は最大でも 110 byte しかないので、**大きめの y_* を要素に並べた
JSON 配列**を目標サイズまで伸ばす。要素の順序は (サイズ降順, 名前昇順) で固定、
区切りは `,\n`、末尾は `\n]\n`。同じ commit の JSONTestSuite なら byte 同一になる。

  usage: make-json-inputs.py <JSONTestSuite/test_parsing> <出力ディレクトリ>

出力: json-1k.json / json-10k.json / json-100k.json / json-1m.json と、
各ファイルの sha256 を並べた inputs.sha256。
"""
import hashlib
import json
import pathlib
import sys

TARGETS = [("1k", 1_000), ("10k", 10_000), ("100k", 100_000), ("1m", 1_000_000)]
# 要素に使う y_* の本数（大きい順）。小さすぎるものばかりにならないように絞る。
ELEMENT_COUNT = 24


def main() -> int:
    suite = pathlib.Path(sys.argv[1])
    out = pathlib.Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)

    files = sorted(suite.glob("y_*.json"), key=lambda p: (-p.stat().st_size, p.name))
    elements = []
    for path in files[:ELEMENT_COUNT]:
        text = path.read_bytes().decode("utf-8").strip()
        json.loads(text)  # 要素が本当に JSON 値であることを確かめる（suite の y_ 前提）
        elements.append(text)
    if not elements:
        print("y_* が見つからない", file=sys.stderr)
        return 2

    digests = []
    for name, target in TARGETS:
        parts = []
        size = 2  # "[\n" と "\n]\n" の概算ぶん
        i = 0
        while size < target:
            element = elements[i % len(elements)]
            parts.append(element)
            size += len(element.encode("utf-8")) + 2
            i += 1
        body = "[\n" + ",\n".join(parts) + "\n]\n"
        path = out / f"json-{name}.json"
        path.write_text(body, encoding="utf-8")
        json.loads(body)
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        digests.append(f"{digest}  {path.name}  {path.stat().st_size} bytes  {len(parts)} elements")
        print(digests[-1])
    (out / "inputs.sha256").write_text("\n".join(digests) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
