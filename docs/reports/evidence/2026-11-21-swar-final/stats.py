#!/usr/bin/env python3
"""実時間の分布と同じ周の速度比を集計する。"""
from collections import defaultdict
import csv
from pathlib import Path
from statistics import median
import sys

VARIANTS = ("legacy", "scalar", "short", "candidate", "placebo")


def load(paths):
    inputs = defaultdict(lambda: defaultdict(dict))
    for session, path in enumerate(paths, 1):
        with Path(path).open() as stream:
            for row in csv.DictReader(stream):
                trial = (session, int(row["round"]))
                inputs[row["input"]][trial][row["variant"]] = float(row["ms"])
    return inputs


def spread(values):
    return f"{median(values):.6f} [{min(values):.6f}, {max(values):.6f}]"


def main(paths):
    for name, trials in load(paths).items():
        print(name)
        for variant in VARIANTS:
            values = [times[variant] for times in trials.values()]
            sessions = defaultdict(list)
            for (session, _), times in trials.items():
                sessions[session].append(times[variant])
            medians = [f"{median(values):.6f}" for values in sessions.values()]
            print(f"{variant:9} {len(values):2} {spread(values)}", " / ".join(medians))
        for baseline, trial in (
            ("short", "candidate"),
            ("short", "placebo"),
            ("legacy", "candidate"),
            ("legacy", "short"),
            ("scalar", "candidate"),
        ):
            ratios = [times[baseline] / times[trial] for times in trials.values()]
            wins = sum(ratio > 1 for ratio in ratios)
            print(f"{baseline}/{trial} {spread(ratios)} {wins}/{len(ratios)}")


if __name__ == "__main__":
    main(sys.argv[1:])
