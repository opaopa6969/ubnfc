#!/usr/bin/env python3
"""callgrind の自己コストと JFR の main thread 先頭 frame を TSV にする。"""
import collections
import csv
import json
import pathlib
import re
import subprocess
import sys


def callgrind(path, count):
    names = {}
    costs = collections.defaultdict(collections.Counter)
    calls = collections.Counter()
    events = []
    positions = 1
    current = None
    edge = False

    def name(value):
        if value.startswith("("):
            key, _, label = value.partition(")")
            if label.strip():
                names[key] = label.strip()
            return key
        return value

    for line in pathlib.Path(path).read_text().splitlines():
        if line.startswith("events: "):
            events = line.split()[1:]
        elif line.startswith("positions: "):
            positions = len(line.split()) - 1
        elif line.startswith("fn="):
            current = name(line[3:])
        elif line.startswith("cfn="):
            callee = name(line[4:])
        elif line.startswith("calls="):
            calls[callee] += int(line[6:].split()[0])
            edge = True
        elif line and line[0] in "0123456789+*-" and current is not None:
            if not edge:
                values = line.split()[positions:]
                for event, value in zip(events, values):
                    costs[current][event] += int(value)
            edge = False
    totals = sum((c for c in costs.values()), collections.Counter())
    if totals["Ir"] == 0:
        raise ValueError("callgrind の収集命令が 0（計装区間を確認）")
    writer = csv.writer(sys.stdout, delimiter="\t", lineterminator="\n")
    writer.writerow(["関数", "calls/parse", "自己時間ns", "自己Ir%"] + [f"{e}/parse" for e in events])
    writer.writerow(["合計", "—", "未取得", "100"] + [f"{totals[e] / count:.3f}" for e in events])
    for key, cost in sorted(costs.items(), key=lambda item: -item[1]["Ir"]):
        label = names.get(key, key)
        label = subprocess.run(["c++filt", label], check=True, text=True, capture_output=True).stdout.strip()
        writer.writerow([label, f"{calls[key] / count:.3f}", "未取得", f"{100 * cost['Ir'] / max(1, totals['Ir']):.3f}"] + [f"{cost[e] / count:.3f}" for e in events])


def jfr(path):
    data = json.loads(pathlib.Path(path).read_text())
    counts = collections.Counter()
    for event in data["recording"]["events"]:
        values = event["values"]
        thread = values.get("sampledThread") or {}
        if thread.get("javaName") != "main":
            continue
        frames = (values.get("stackTrace") or {}).get("frames", [])
        if frames:
            method = frames[0]["method"]
            label = method["type"]["name"] + "." + method["name"] + method["descriptor"]
            counts[label] += 1
    writer = csv.writer(sys.stdout, delimiter="\t", lineterminator="\n")
    writer.writerow(["関数（先頭 frame）", "自己CPUサンプル", "自己CPUサンプル%", "自己時間ns", "命令数", "分岐ミス", "cache miss"])
    total = counts.total()
    if not total:
        raise ValueError("main thread の JFR CPU サンプルが空")
    writer.writerow(["合計", total, 100, "未取得", "未取得", "未取得", "未取得"])
    for label, count in counts.most_common():
        writer.writerow([label, count, f"{100 * count / max(1, total):.3f}", "未取得", "未取得", "未取得", "未取得"])


def perf(path, count):
    writer = csv.writer(sys.stdout, delimiter="\t", lineterminator="\n")
    writer.writerow(["関数", "自己CPU時間ns/parse（標本推定）", "自己CPU%", "命令数", "分岐ミス", "cache miss"])
    rows = []
    for line in pathlib.Path(path).read_text().splitlines():
        match = re.fullmatch(r"\s*([\d.]+)%\s+(\d+)\s+(.*?)(?:\s+-\s+-)?\s*", line)
        if match:
            percent, period, label = match.groups()
            rows.append((int(period), percent, label))
    if not rows:
        raise ValueError("perf の関数別サンプルが空")
    for period, percent, label in sorted(rows, reverse=True):
        writer.writerow([label, f"{period / count:.3f}", percent, "callgrind参照", "callgrind参照", "callgrind参照"])


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "callgrind" and int(sys.argv[3]) > 0:
        callgrind(sys.argv[2], int(sys.argv[3]))
    elif len(sys.argv) == 4 and sys.argv[1] == "perf" and int(sys.argv[3]) > 0:
        perf(sys.argv[2], int(sys.argv[3]))
    elif len(sys.argv) == 3 and sys.argv[1] == "jfr":
        jfr(sys.argv[2])
    else:
        sys.exit("summarize.py callgrind <file> <反復数> | perf <自己CSV> <反復数> | jfr <json>")
