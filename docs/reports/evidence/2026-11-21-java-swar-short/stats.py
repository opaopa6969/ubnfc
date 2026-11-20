import csv
import re
import sys
from collections import defaultdict
from pathlib import Path
from statistics import median

work = Path(sys.argv[1])
groups = defaultdict(list)
settings = defaultdict(set)
for session in (1, 2):
    with (work / f"session-{session}.csv").open() as stream:
        for row in csv.DictReader(stream):
            key = (row["input"], int(row["session"]), int(row["round"]), row["variant"])
            assert key[1] == session
            groups[key].append((int(row["sample"]), float(row["wall_ms"]), float(row["cpu_ms"])))
            settings[key[0]].add((row["warmup"], row["iterations"]))
    path = work / f"session-{session}.uptime"
    loads = [tuple(map(float, triple)) for triple in
             re.findall(r"load average: ([\d.]+), ([\d.]+), ([\d.]+)", path.read_text())]
    print(f"session {session} ({len(loads)}): {loads[0]} → {loads[-1]}, "
          f"range {tuple(zip(map(min, zip(*loads)), map(max, zip(*loads))))}")
assert all(len(values) == 1 for values in settings.values())
assert all(sorted(row[0] for row in values) == [1, 2, 3] for values in groups.values())

for case in settings:
    variants = ("old", "scalar", "before", "candidate") if case == "json-1m" else ("scalar", "before", "candidate")
    for sessions in ((1,), (2,), (1, 2)):
        per_round = {
            variant: [tuple(median(row[metric] for row in groups[(case, session, round_id, variant)])
                            for metric in (1, 2))
                      for session in sessions for round_id in range(1, 8)]
            for variant in variants
        }
        print(case, sessions)
        for idx, unit in enumerate(("wall", "cpu")):
            values = {variant: [row[idx] for row in per_round[variant]] for variant in variants}
            for variant, series in values.items():
                print(f"  {unit:4} {variant:9} {median(series):.6f} [{min(series):.6f}–{max(series):.6f}]", end="")
                if variant != "candidate":
                    ratios = [base / trial for base, trial in zip(series, values["candidate"], strict=True)]
                    print(f" / candidate: {median(series) / median(values['candidate']):.3f},"
                          f" paired {median(ratios):.3f} [{min(ratios):.3f}–{max(ratios):.3f}]"
                          f" ({sum(ratio > 1 for ratio in ratios)}/{len(ratios)})", end="")
                print()
