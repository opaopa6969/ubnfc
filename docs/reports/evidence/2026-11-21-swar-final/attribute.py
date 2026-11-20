#!/usr/bin/env python3
"""Callgrind のソース位置から固定 JSON の自己 Ir を集計する。"""
import collections
import csv
from pathlib import Path
import re
import sys

from profile import VARIANTS, segments


def lines_of(path, pattern):
    lines = path.read_text().splitlines()
    matches = set()
    end = ""
    for number, line in enumerate(lines, 1):
        if re.match(pattern, line):
            end = line[:len(line) - len(line.lstrip())] + "}"
        if end:
            matches.add(number)
        if end and line == end:
            end = ""
    return matches


def count(work, variant):
    source = work / variant / "json"
    parser = source / "parser.rs"
    text = parser.read_text()
    bulk = set()
    for _, start, end in segments(text):
        bulk.update(range(text.count("\n", 0, start) + 1,
                          text.count("\n", 0, end) + 2))
    scan = lines_of(parser, r"fn s\d+\(")
    rules = lines_of(parser, r"fn (?:[rbe]\d+|guard_e\d+)_[cm]\(")
    input_path = source / "rt/input.rs"
    utf8 = lines_of(input_path, r"    fn cp_at_non_ascii\(")
    if not utf8:
        # 旧ランタイムは ASCII も cp_at 内で UTF-8 検証する。
        utf8 = lines_of(input_path, r"    pub fn cp_at\(")

    def category(file, line):
        if file.endswith("/parser.rs"):
            if line in bulk:
                return "bulk"
            if line in scan:
                return "scan_other"
            if line in rules:
                return "rules"
        if file.endswith("/rt/cursor.rs"):
            return "cursor"
        if file.endswith("/rt/input.rs"):
            return "utf8_decode" if line in utf8 else "input_other"
        if file.endswith(("/str/validations.rs", "/str/converts.rs")):
            return "utf8_core"
        return "other"

    files = {}
    costs = collections.Counter()
    path = work / "profile" / f"total-{variant}.callgrind"
    file = ""
    pos = 0
    edge = False
    rows = path.read_text().splitlines()
    for row in rows:
        definition = re.match(r"^(?:c)?f[ile]=\((\d+)\) (.+)$", row)
        if definition:
            key, value = definition.groups()
            files[f"({key})"] = value
    for row in rows:
        if row.startswith(("fl=", "fi=", "fe=")):
            key = row[3:].split(" ", 1)[0]
            file = files[key] if key.startswith("(") else row[3:]
        elif row.startswith("calls="):
            edge = True
        elif row and row[0] in "0123456789+*-":
            line, ir, *_ = row.split()
            if line != "*":
                pos = pos + int(line) if line.startswith(("+", "-")) else int(line)
            if not edge:
                costs[category(file, pos)] += int(ir)
            edge = False
    return costs


if __name__ == "__main__":
    work = Path(sys.argv[1])
    columns = ["bulk", "scan_other", "rules", "cursor", "utf8_decode", "utf8_core",
               "input_other", "other"]
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(["variant", *columns, "total"])
    for variant in VARIANTS:
        costs = count(work, variant)
        writer.writerow([variant, *(costs[k] / 10 for k in columns), costs.total() / 10])
