#!/usr/bin/env bash
# WORK は mktemp -d で作った専用ディレクトリ。入力は既存の JSON/P4 fixture を指定する。
# WORK=$(mktemp -d) benchmarks/scripts/run-ll1-ab.sh json /path/to/json-{1k,10k,100k,1m}.json
# WORK=<同じディレクトリ> benchmarks/scripts/run-ll1-ab.sh p4 /path/to/complex-x64.txt
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
work=${WORK:?mktemp -d で作った WORK を指定する}
export RUSTUP_TOOLCHAIN=stable
mkdir -p "$work/ll1-bench/src"
cargo build --manifest-path "$repo/Cargo.toml" --release --offline --bin ubnfc --bin ubnfc-rust
"$repo/target/release/ubnfc" ir --grammar "$repo/examples/json/json.ubnf" --out "$work/json.ir.json"
"$repo/target/release/ubnfc-rust" --ir "$work/json.ir.json" --out "$work/json" --no-emit-driver
"$repo/target/release/ubnfc-rust" --ir "$repo/ir/frozen/tinyexpression-p4-f86ce8a5.ir.json" --out "$work/p4" --no-emit-driver
cp "$repo/benchmarks/src/family_ll1.rs" "$work/ll1-bench/src/main.rs"
cat > "$work/ll1-bench/Cargo.toml" <<MANIFEST
[workspace]
[package]
name = "ll1-bench"
version = "0.0.0"
edition = "2021"
[dependencies]
gen-json = { package = "ubnfc-generated", path = "$work/json" }
gen-p4 = { package = "ubnfc-p4-rust", path = "$repo/examples/p4-rust" }
MANIFEST
# 同名 package の lockfile 衝突を避けるため生成側の名前を分ける。
sed -i 's/name = "ubnfc-generated"/name = "gen-json"/' "$work/json/Cargo.toml"
sed -i 's/name = "ubnfc-generated"/name = "gen-p4"/' "$work/p4/Cargo.toml"
sed -i 's/package = "ubnfc-generated", //g' "$work/ll1-bench/Cargo.toml"
cargo build --release --offline --manifest-path "$work/ll1-bench/Cargo.toml"
uptime >&2
rustc --version >&2
lscpu | sed -n '/Model name:/p' >&2
# 固定 CPU は呼出し側で CPU=30 のように選ぶ。既定は affinity 内の最小 CPU。
cpu=${CPU:-$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')}
taskset -c "$cpu" "$work/ll1-bench/target/release/ll1-bench" "$@"
uptime >&2
