#!/usr/bin/env python3
"""同一バイナリ内で SWAR の変更前後と対照を交互計測する。"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

VARIANTS = ("scalar", "short", "candidate", "legacy", "placebo")
GRAMMARS = ("p4", "json", "ops")
ROOT = Path(__file__).resolve().parent
while not (ROOT / "benchmarks/src/family_swar.rs").is_file() and ROOT != ROOT.parent:
    ROOT = ROOT.parent
ROOT = Path(os.environ.get("UBNFC_ROOT", ROOT))
RUN = ROOT / "benchmarks"
FIXTURES = ROOT / "examples/p4-rust/tests/fixtures"
P4_INPUTS = [FIXTURES / name for name in (
    "complex.tiny", "complex-x4.tiny", "complex-x16.tiny", "complex-x64.tiny",
    "flat-arithmetic.tiny", "large-match.tiny",
)]


def prepare(work, suite, generators):
    ir = work / "ir"
    ir.mkdir()
    subprocess.run([generators[1].with_name("ubnfc"), "ir", "--grammar",
                    ROOT / "examples/json/json.ubnf", "--out", ir / "json.ir.json"], check=True)
    shutil.copyfile(ROOT / "ir/frozen/tinyexpression-p4-f86ce8a5.ir.json", ir / "p4.ir.json")
    shutil.copyfile(ROOT / "ir/fixtures/unlaxer-dsl__src__test__resources__associative__Operators.ubnf.ir.json",
                    ir / "ops.ir.json")
    for path in ir.glob("*.json"):
        obj = json.loads(path.read_text())
        obj["plan"].pop("ll1", None)
        path.write_text(json.dumps(obj, ensure_ascii=False, indent=2) + "\n")
    inputs = work / "inputs"
    inputs.mkdir()
    subprocess.run(["python3", RUN / "scripts/make-json-inputs.py", suite / "test_parsing",
                    inputs], check=True)
    (inputs / "operators-atoms-512.txt").write_text("+".join(["1"] * 512))

    source = (RUN / "src/family_swar.rs").read_text()
    # 既存の options / scanner / 反復数校正を共有し、版の選択と実行順だけを変える。
    start = source.index("fn before(")
    stop = source.index("// 同じ IR", start)
    before, after = source[start:stop].split("fn after", 1)
    header = source[:start]
    header = header[:header.index("#[allow(dead_code)]")] + header[header.index("const EXTERN"):]
    bench = work / "bench"
    src = bench / "src"
    src.mkdir(parents=True)
    shutil.copyfile(RUN / "src/bench_util.rs", src / "bench_util.rs")
    scanner = (ROOT / "scanners/rust/scanners.rs").read_text()
    dependencies = []
    for variant, generator in zip(VARIANTS, [*generators, generators[1]], strict=True):
        header += f"#[allow(dead_code)] mod sc_{variant};\n"
        entry = before if variant == "legacy" else "fn after" + after
        header += entry.replace("before" if variant == "legacy" else "after", variant)
        (src / f"sc_{variant}.rs").write_text(scanner.replace("ubnfc_generated", f"p4_{variant}"))
        for grammar in GRAMMARS:
            output = work / variant / grammar
            crate = f"{grammar}-{variant}"
            subprocess.run([generator, "--ir", ir / f"{grammar}.ir.json", "--out", output,
                            "--no-emit-driver"], check=True)
            manifest = output / "Cargo.toml"
            manifest.write_text(manifest.read_text().replace('name = "ubnfc-generated"', f'name = "{crate}"'))
            dependencies.append(f'{crate} = {{ path = "../{variant}/{grammar}" }}')
    main = source[stop:].replace(
        "[before(grammar, false, true), after(grammar, false, false, true)]",
        "[scalar(grammar, false, false, true), short(grammar, false, false, true), "
        "candidate(grammar, false, false, true), legacy(grammar, false, true), "
        "placebo(grammar, false, false, true)]",
    ).replace(
        "for k in 0..2 {\n                let v = (round+k)%2;",
        'let offset = (round + env_usize("AB_SESSION", 0)) % 5;\n'
        '            for k in 0..5 { let v = (offset + if round < 5 { k } else { 4-k }) % 5;',
    ).replace('["before", "after"]', '["scalar", "short", "candidate", "legacy", "placebo"]')
    assert "for k in 0..5" in main and "legacy(grammar" in main
    (src / "main.rs").write_text(header + main)
    (bench / "Cargo.toml").write_text(
        '[workspace]\n[package]\nname = "short-swar"\nversion = "0.0.0"\n'
        'edition = "2021"\n[dependencies]\n' + "\n".join(dependencies) + "\n"
    )
    subprocess.run(["cargo", "build", "--offline", "--release", "-j", "8"],
                   cwd=bench, check=True)
    paths = [*generators, *sorted(ir.glob("*.json")), *sorted(inputs.glob("*")),
             *sorted(work.glob("*/*/parser.rs")), *P4_INPUTS,
             bench / "target/release/short-swar"]
    for path in paths:
        print(hashlib.sha256(path.read_bytes()).hexdigest(), path)


def measure(work, session, core):
    inputs = work / "inputs"
    groups = {"p4": P4_INPUTS, "json": [inputs / "json-1m.json"],
              "ops": [inputs / "operators-atoms-512.txt"]}
    env = dict(os.environ, AB_SESSION=str(2 * (session - 1)), AB_RUNS="10", AB_RUN_SECONDS="0.14")
    with (work / f"session-{session}.csv").open("w") as csv:
        subprocess.run(["uptime"], stdout=sys.stderr, check=True)
        for index, (grammar, files) in enumerate(groups.items()):
            rows = subprocess.check_output(["taskset", "-c", str(core), work / "bench/target/release/short-swar",
                                            "time", grammar, *files], text=True, env=env)
            first_row = 1 if index else 0
            csv.writelines(rows.splitlines(keepends=True)[first_row:])
            csv.flush()
        subprocess.run(["uptime"], stdout=sys.stderr, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("work", type=Path)
    commands = parser.add_subparsers(dest="action", required=True)
    build = commands.add_parser("prepare")
    build.add_argument("suite", type=Path)
    build.add_argument("generators", type=Path, nargs=4, metavar="GEN")
    timing = commands.add_parser("time")
    timing.add_argument("session", type=int, choices=(1, 2))
    timing.add_argument("core", type=int)
    args = parser.parse_args()
    os.environ.update(RUSTUP_TOOLCHAIN="stable")
    os.environ.pop("CARGO_TARGET_DIR", None)  # benchmark crate ごとの target
    if args.action == "prepare":
        prepare(args.work.resolve(), args.suite.resolve(), [p.resolve() for p in args.generators])
    else:
        measure(args.work.resolve(), args.session, args.core)
