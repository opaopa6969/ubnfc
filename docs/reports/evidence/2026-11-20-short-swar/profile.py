#!/usr/bin/env python3
"""固定 JSON の区間を記録し、複製した走査ループの自己 Ir と全体 Ir を採る。"""
import collections
import csv
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

VARIANTS = ("scalar", "short", "swar", "legacy")
BUCKETS = ("0", "1-7", "8+")
ROOT = Path(os.environ.get("UBNFC_ROOT", Path.cwd())).resolve()
REGION = ROOT / "benchmarks/attribution"
PATTERN = re.compile(r'(?:let mut ascii_start=p;\n)?let bytes=self\.text\.as_bytes\(\);while[^\n]*\{')
API = '''
fn options() -> ParseOptions {
    let mut o = ParseOptions { build_ast: false, reuse_buffers: true, ..Default::default() };
    o.limits.arena_entries = 1 << 30;
    o
}
'''
SETUP = '''
    let mut scanner = RejectExtern;
    let session = Session::<false>::new(text, options(), &mut scanner, usize::MAX);
'''


def segments(source):
    for m in PATTERN.finditer(source):
        fn = list(re.finditer(r'fn s(\d+)\(', source[:m.start()]))[-1][1]
        end = m.end()
        depth = 1
        while depth:
            depth += (source[end] == "{") - (source[end] == "}")
            end += 1
        yield fn, m.start(), end


def copy_parser(work, side, mode, profile):
    name = f"{mode}-{side}"
    dest = profile / name
    shutil.copytree(work / side / "json", dest)
    manifest = dest / "Cargo.toml"
    manifest.write_text(manifest.read_text().replace("[workspace]\n", "")
                        .replace(f'name = "json-{side}"', f'name = "{name}"'))
    parser = dest / "parser.rs"
    source = parser.read_text()
    regions = list(segments(source))
    assert regions
    if mode == "trace":
        for fn, start, end in reversed(regions):
            source = (source[:start] + f"let trace_start=p;\n{source[start:end]}\n"
                      + f"MEASURE_TRACE.with(|t| t.borrow_mut().push(({fn}, trace_start, p)));\n"
                      + source[end:])
        source += API + '''
thread_local! {
    static MEASURE_TRACE: std::cell::RefCell<Vec<(usize,usize,usize)>> = Default::default();
}
pub fn measure_trace(text: &str) -> Vec<(usize,usize,usize)> {
    assert!(parse_with_options(text, options()).ok);
    MEASURE_TRACE.with(|t| std::mem::take(&mut *t.borrow_mut()))
}
'''
        export = "measure_trace"
    else:
        bodies = []
        match = []
        for fn, start, end in regions:
            loop = source[start:end]
            counter = "let mut count=0usize;" if "count+=" in loop else "let count=0usize;"
            bodies.append(f"#[inline(never)] fn measure_bulk_{fn}(&self,mut p:usize)"
                          f"->(usize,usize) {{ {counter}\n{loop}\n(p,count) }}")
            match.append(f"{fn} => session.measure_bulk_{fn}(p),")
        source += API + "impl<const DIAG:bool> Session<'_,DIAG> {\n" + "\n".join(bodies) + "}\n"
        source += '''
pub fn measure_replay(text: &str, rows: &[(usize,usize,usize)], active: impl Fn(bool)) {
''' + SETUP + '''
    active(true);
    for &(id,p,end) in rows {
        let (actual,count) = match id {
''' + "\n".join(match) + '''
            _ => unreachable!("bulk id {id}"),
        };
        assert_eq!(actual,end,"bulk id {id} at {p}");
        std::hint::black_box(count);
    }
    active(false);
}
'''
        export = "measure_replay"
    parser.write_text(source)
    mod = dest / "mod.rs"
    mod.write_text(mod.read_text() + f"\npub use parser::{export};\n")
    return f'{name} = {{ path = "{name}" }}'


def prepare(work, image):
    profile = work / "profile"
    profile.mkdir()
    (profile / "src").mkdir()
    include = profile / "include"
    include.mkdir()
    headers = subprocess.check_output(["docker", "run", "--rm", image, "tar", "-C",
                                       "/usr/include", "-cf", "-", "valgrind"])
    subprocess.run(["tar", "-xf", "-", "-C", include], input=headers, check=True)
    subprocess.run(["cc", "-I", include, "-c", REGION / "region.c", "-o", profile / "region.o"], check=True)
    subprocess.run(["ar", "rcs", profile / "libattribution_region.a", profile / "region.o"], check=True)
    (profile / "build.rs").write_text('''fn main() {
    println!("cargo:rustc-link-search=native={}",env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=static=attribution_region");
}
''')
    dependencies = [copy_parser(work, "scalar", "trace", profile)]
    dependencies.extend(copy_parser(work, side, "replay", profile) for side in VARIANTS)
    dependencies.extend(f'json-{side} = {{ path = "../{side}/json" }}' for side in VARIANTS)
    (profile / "Cargo.toml").write_text(
        '[workspace]\n[package]\nname = "short-swar-profile"\nversion = "0.0.0"\n'
        'edition = "2021"\n[profile.release]\ndebug = 1\n[dependencies]\n' +
        "\n".join(dependencies) + "\n"
    )
    (profile / "src/main.rs").write_text('''
use std::hint::black_box;
extern "C" { fn ubnfc_profile_start(); fn ubnfc_profile_stop(); }
fn region(on: bool) {
    unsafe { if on { ubnfc_profile_start() } else { ubnfc_profile_stop() } }
}
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let input = std::fs::read_to_string(&a[1]).unwrap();
    let text = input.as_str();
    if a[0] == "trace" {
        for (id,p,end) in trace_scalar::measure_trace(text) {
            println!("{id},{p},{end}");
        }
        return;
    }
    let side = a[2].as_str();
    if a[0] == "replay" {
        let file = std::fs::read_to_string(&a[3]).unwrap();
        let rows: Vec<_> = file.lines().map(|row| {
            let v: Vec<usize> = row.split(',').map(|n| n.parse().unwrap()).collect();
            (v[0],v[1],v[2])
        }).collect();
        assert!(!rows.is_empty());
        match side {
''' + "\n".join(f'            "{side}" => replay_{side}::measure_replay(text,&rows,region),' for side in VARIANTS) + '''
            _ => unreachable!(),
        }
        return;
    }
    assert_eq!(a[0],"total");
    macro_rules! parse {
        ($parser:ident) => {{
            let mut o = $parser::ParseOptions { build_ast: false, reuse_buffers: true, ..Default::default() };
            o.limits.arena_entries = 1 << 30;
            let r = $parser::parse_with_options(black_box(text),o);
            assert!(r.ok);
            black_box(r);
        }};
    }
    let parse = || match side {
''' + "\n".join(f'        "{side}" => parse!(json_{side}),' for side in VARIANTS) + '''
        _ => unreachable!(),
    };
    for _ in 0..10 { parse(); }
    region(true);
    for _ in 0..10 { parse(); }
    region(false);
}
''')
    subprocess.run(["cargo", "build", "--offline", "--release", "-j", "6"], cwd=profile, check=True)


def measure(work, image, core):
    profile = work / "profile"
    binary = profile / "target/release/short-swar-profile"
    text = work / "inputs/json-1m.json"
    with (profile / "uptime.txt").open("w") as load:
        def uptime(label):
            print(label, file=load, flush=True)
            subprocess.run(["uptime"], stdout=load, check=True)

        uptime("trace")
        trace = subprocess.check_output(["taskset", "-c", core, binary, "trace", text], text=True)
        input_bytes = text.read_bytes()
        groups = collections.defaultdict(list)
        for row in trace.splitlines():
            _, start, end = map(int, row.split(","))
            bucket = "0" if end == start else "1-7" if end - start < 8 else "8+"
            groups[bucket, input_bytes[end:end+1] == b"\\"].append(row)
        # Escape の byte 値を独立に分け、出現しない組も 0 と記録する。
        with (profile / "buckets.csv").open("w") as summary:
            writer = csv.writer(summary, lineterminator="\n")
            writer.writerow(["variant", "length", "escape", "calls", "self_Ir", "self_Ir_per_call"])
            for side in VARIANTS:
                for bucket in BUCKETS:
                    for escape in (False, True):
                        rows = groups[bucket, escape]
                        label = f"{side}-{bucket}-{str(escape).lower()}"
                        if not rows:
                            writer.writerow([side, bucket, escape, 0, 0, ""])
                            continue
                        bucket_file = profile / f"{bucket}-{escape}.trace"
                        bucket_file.write_text("\n".join(rows) + "\n")
                        stats = collect("replay", side, label, 1, bucket_file, uptime,
                                        image, core, work, binary, text)
                        cost = sum(int(float(row["Ir/parse"])) for row in stats
                                   if "::measure_bulk_" in row["関数"])
                        assert cost > 0, label
                        writer.writerow([side, bucket, escape, len(rows), cost, f"{cost / len(rows):.3f}"])
                collect("total", side, f"total-{side}", 10, None, uptime, image, core, work, binary, text)
                summary.flush()


def collect(mode, side, label, count, bucket, uptime, image, core, work, binary, text):
    profile = work / "profile"
    path = profile / f"{label}.callgrind"
    uptime(label + " start")
    command = ["docker", "run", "--rm", "--cpuset-cpus", core, "--user", f"{os.getuid()}:{os.getgid()}",
               "-v", f"{work}:{work}", image, "valgrind", "--tool=callgrind", "--instr-atstart=no",
               "--cache-sim=yes", "--branch-sim=yes", f"--callgrind-out-file={path}",
               binary, mode, text, side]
    if bucket:
        command.append(bucket)
    with (profile / f"{label}.log").open("w") as log:
        subprocess.run(command, stdout=log, stderr=log, check=True)
    uptime(label + " end")
    tsv = path.with_suffix(".tsv")
    with tsv.open("w") as output:
        subprocess.run(["python3", REGION / "summarize.py", "callgrind", path, str(count)],
                       stdout=output, check=True)
    with tsv.open() as source:
        return list(csv.DictReader(source, delimiter="\t"))


if __name__ == "__main__":
    action, work, image, core = sys.argv[1:]
    os.environ["RUSTUP_TOOLCHAIN"] = "stable"
    os.environ.pop("CARGO_TARGET_DIR", None)
    {"prepare": prepare, "measure": measure}[action](
        *((Path(work).resolve(), image, core) if action == "measure" else (Path(work).resolve(), image))
    )
