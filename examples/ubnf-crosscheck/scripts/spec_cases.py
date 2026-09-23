#!/usr/bin/env python3
"""corpus/spec/ubnf/*.json のケース入力を 1 ファイルずつ展開し、manifest を標準出力へ出す。

manifest は `ファイル名<TAB>ケース ID<TAB>scope<TAB>verdict` の TSV。
"""
import json
import pathlib
import sys


def file_name(case_id):
    return case_id.replace("/", "__") + ".ubnf"


def main():
    source = pathlib.Path(sys.argv[1])
    target = pathlib.Path(sys.argv[2])
    target.mkdir(parents=True, exist_ok=True)
    for path in sorted(source.glob("*.json")):
        document = json.loads(path.read_text(encoding="utf-8"))
        for case in document["cases"]:
            name = file_name(case["id"])
            (target / name).write_text(case["input"], encoding="utf-8")
            print(f"{name}\t{case['id']}\t{case['scope']}\t{case['expectation']['verdict']}")


if __name__ == "__main__":
    main()
