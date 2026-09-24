#!/usr/bin/env bash
# stax の文法を検証し、Rust / Java の 2 backend を生成して 20 本のサンプルを突き合わせる。
#
#   scripts/crosscheck.sh
#
# 生成物は commit しない（examples/json / examples/p4-v2 と同じ方針）。
# 作業場所は mktemp -d の専用ディレクトリで、終了時に消す。
# STAX_WORK=<dir> を渡すと再利用する（反復用。build をやり直さない）。
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
root=$(cd "$module/../.." && pwd)

if [ -n "${STAX_WORK:-}" ]; then
  work="$STAX_WORK"
  mkdir -p "$work"
else
  work=$(mktemp -d)
  trap 'rm -rf "$work"' EXIT
fi

echo "== 生成器を build =="
(cd "$root" && cargo build -q --offline --bin ubnfc)
(cd "$root/ubnfc-java" && mvn -o -q -DskipTests package)
ubnfc="$root/target/debug/ubnfc"

echo "== 文法を検証（--strict） =="
"$ubnfc" validate --grammar "$module/stax.ubnf" --strict

echo "== Grammar IR =="
"$ubnfc" ir --grammar "$module/stax.ubnf" --out "$work/stax.ir.json"

echo "== Rust backend を生成・build =="
rm -rf "$work/rust-out"
"$ubnfc" gen --grammar "$module/stax.ubnf" --target rust --out "$work/rust-out" >/dev/null
(cd "$work/rust-out" && cargo build -q --offline --features json --bins)

echo "== Java backend を生成・compile =="
rm -rf "$work/java-out" "$work/java-classes"
"$ubnfc" gen --grammar "$module/stax.ubnf" --target java --out "$work/java-out" >/dev/null
mkdir -p "$work/java-classes"
javac -encoding UTF-8 -d "$work/java-classes" $(find "$work/java-out/src/main/java" -name '*.java')

echo "== 20 本のサンプルを両 backend へ =="
python3 "$module/scripts/crosscheck.py" \
  "$work/rust-out/target/debug/driver" "$work/java-classes" "$module/samples"
