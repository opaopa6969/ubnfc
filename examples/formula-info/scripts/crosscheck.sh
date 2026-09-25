#!/usr/bin/env bash
# formula-info の文法を検証し、Rust / Java の 2 backend を生成して 45 本のサンプルを突き合わせる。
#
#   scripts/crosscheck.sh
#
# 生成物は commit しない（examples/json / examples/stax と同じ方針）。
# 作業場所は mktemp -d の専用ディレクトリで、終了時に消す。
# FORMULA_INFO_WORK=<dir> を渡すと再利用する（反復用。build をやり直さない）。
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
root=$(cd "$module/../.." && pwd)

if [ -n "${FORMULA_INFO_WORK:-}" ]; then
  work="$FORMULA_INFO_WORK"
  mkdir -p "$work"
else
  work=$(mktemp -d)
  trap 'rm -rf "$work"' EXIT
fi

echo "== 生成器を build =="
(cd "$root" && cargo build -q --offline --bins)
(cd "$root/ubnfc-java" && mvn -o -q -DskipTests package)
ubnfc="$root/target/debug/ubnfc"

echo "== 文法を検証（--strict） =="
"$ubnfc" validate --grammar "$module/formula-info.ubnf" --strict

echo "== Grammar IR =="
"$ubnfc" ir --grammar "$module/formula-info.ubnf" --out "$work/formula-info.ir.json"

echo "== Rust backend を生成・build =="
rm -rf "$work/rust-out"
"$ubnfc" gen --grammar "$module/formula-info.ubnf" --target rust --out "$work/rust-out" >/dev/null
(cd "$work/rust-out" && cargo build -q --offline --features json --bin driver)

echo "== Java backend を生成・compile =="
rm -rf "$work/java-out" "$work/java-classes"
"$ubnfc" gen --grammar "$module/formula-info.ubnf" --target java --out "$work/java-out" >/dev/null
mkdir -p "$work/java-classes"
javac -encoding UTF-8 -d "$work/java-classes" $(find "$work/java-out/src/main/java" -name '*.java')

echo "== 45 本のサンプルを両 backend へ =="
python3 "$module/scripts/crosscheck.py" \
  "$work/rust-out/target/debug/driver" "$work/java-classes" "$module/samples"
