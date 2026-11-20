#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p results
base="$PWD/base-build/classes:$PWD/base/examples/p4-java/target/classes"
trial="$PWD/timer:$PWD/trial-build/classes:$PWD/trial/examples/p4-java/target/classes"
cmp base-build/ir.sha256 trial-build/ir.sha256
fixtures="$PWD/trial/examples/p4-java/src/test/resources/fixtures"
sha256sum "$fixtures"/complex{,-x64}.tiny | sed "s|$fixtures/||" > results/input.sha256
cp base-build/ir.sha256 results/ir.sha256
for session in 1 2 3; do
  order=()
  if (( session % 2 == 0 )); then order=(-Dbench.reverse=true); fi
  for mode in p4 rust operators; do
    printf 's%d-%s ' "$session" "$mode" >> results/uptime.txt
    uptime >> results/uptime.txt
    if [[ "$mode" == rust ]]; then
      taskset -c 28 "${CARGO_TARGET_DIR:?CARGO_TARGET_DIR を指定}/release/java-direct-ast-reference" "$fixtures" \
        > "results/s$session-rust.csv"
      continue
    fi
    taskset -c 28 java -Xms512m -Xmx2g -Xss16m -Dbench.runs=7 -Dbench.warmup=1000 \
      "-Dbench.baseline=$base" "${order[@]}" -cp "$trial" JavaDirectAstBench "$mode" "$fixtures" \
      > "results/s$session-$mode.csv" 2> "s$session-$mode.log"
  done
  printf 's%d-end ' "$session" >> results/uptime.txt
  uptime >> results/uptime.txt
done
