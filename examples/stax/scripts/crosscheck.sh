#!/usr/bin/env bash
# stax の文法を検証し、Rust / Java の 2 backend を生成して 20 本のサンプルを突き合わせる。
#
#   scripts/crosscheck.sh
#
# 生成物は commit しない（examples/json / examples/p4-v2 と同じ方針）。
# 作業場所は mktemp -d の専用ディレクトリで、終了時に消す。
set -euo pipefail
export RUSTUP_TOOLCHAIN=stable
export PYTHONDONTWRITEBYTECODE=1
module=$(cd "$(dirname "$0")/.." && pwd)
root=$(cd "$module/../.." && pwd)

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "== 生成器を build =="
(cd "$root" && cargo build -q --offline --target-dir "$root/target" --bin ubnfc --bin ubnfc-rust)
(cd "$root/ubnfc-java" && mvn -o -q -DskipTests package)
ubnfc="$root/target/debug/ubnfc"
export UBNFC_RUST_BIN="$root/target/debug/ubnfc-rust"
export UBNFC_JAVA_JAR="$root/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar"

echo "== 文法を検証（--strict） =="
"$ubnfc" validate --grammar "$module/stax.ubnf" --strict

echo "== Grammar IR =="
"$ubnfc" ir --grammar "$module/stax.ubnf" --out "$work/stax.ir.json"

echo "== Rust backend を生成・build =="
"$ubnfc" gen --grammar "$module/stax.ubnf" --target rust --out "$work/rust-out" >/dev/null
(cd "$work/rust-out" && cargo build -q --offline --target-dir "$work/rust-target" --features json --bin driver)

echo "== Java backend を生成・compile =="
"$ubnfc" gen --grammar "$module/stax.ubnf" --target java --out "$work/java-out" >/dev/null
mkdir -p "$work/java-classes"
find "$work/java-out/src/main/java" -name '*.java' > "$work/java-sources.txt"
javac --release 17 -encoding UTF-8 -d "$work/java-classes" @"$work/java-sources.txt"

echo "== 受理サンプル 20 本と構文拒否ケースを両 backend へ =="
python3 "$module/scripts/crosscheck.py" \
  "$work/rust-target/debug/driver" "$work/java-classes" "$module/samples"

# 同じ生成パーサで仕様由来の境界ケースを検証する。
python3 "$module/scripts/corpus.py" "$work/rust-target/debug/driver" "$work/java-classes"
