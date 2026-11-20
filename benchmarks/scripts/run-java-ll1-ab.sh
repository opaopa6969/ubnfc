#!/usr/bin/env bash
# 固定 JSON 1 MB / P4 complex-x64 を同一 CPU の独立した JVM で交互計測する。
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
work=$(realpath "${WORK:?mktemp -d で作った WORK を指定してください}")
json=ir/examples/json-v2.ir.json
p4=ir/frozen/tinyexpression-p4-f86ce8a5.ir.json
p4_root=examples/p4-java/src
p4_input=$repo/$p4_root/test/resources/fixtures/complex-x64.tiny
jar=ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar

case "${1:?prepare または time を指定してください}" in
prepare)
  suite=${JSON_TEST_SUITE:?固定 JSONTestSuite の test_parsing の親を指定してください}
  base=${BASE_REF:-origin/master}
  mkdir -p "$work/base"
  git -C "$repo" archive "$base" | tar -x -C "$work/base"
  git -C "$repo" rev-parse "$base" > "$work/base-commit.txt"
  python3 "$repo/benchmarks/scripts/make-json-inputs.py" "$suite/test_parsing" "$work/inputs"
  (cd "$work/inputs" && awk '$2 == "json-1m.json" { print $1 "  " $2 }' \
    "$repo/examples/json/evidence/inputs.sha256" | sha256sum --check)
  for side in before after; do
    source=$repo
    if [[ $side == before ]]; then source=$work/base; fi
    mvn -q -f "$source/pom.xml" -DskipTests package
    java -cp "$source/$jar" org.unlaxer.ubnfc.java.Main --ir "$repo/$json" --out "$work/$side/src" --package gen.json
    java -cp "$source/$jar" org.unlaxer.ubnfc.java.Main --ir "$repo/$p4" --out "$work/$side/src" \
      --package org.unlaxer.ubnfc.examples.p4.generated
    rg --files "$work/$side/src" -g '*.java' | sort > "$work/$side/sources"
    javac --release 17 -d "$work/$side/classes" @"$work/$side/sources" \
      "$repo/$p4_root/main/java/org/unlaxer/ubnfc/examples/p4/P4Scanners.java"
    javac -cp "$work/$side/classes" -d "$work/$side/classes" "$repo/benchmarks/java/JavaLl1Bench.java"
  done
  sha256sum "$repo/$json" "$repo/$p4" "$p4_input"
  ;;
time)
  core=${CORE:-28}
  rounds=${ROUNDS:-6}
  java -version 2>&1
  printf '# base=%s cpu=%s rounds=%s\n' "$(cat "$work/base-commit.txt")" "$core" "$rounds"
  printf 'task,variant,bytes,warmup,iterations,median_ms,us_per_byte,allocated_bytes_per_parse\n'
  run() {
    local side=after input=$p4_input
    if [[ $variant == before ]]; then side=before; fi
    if [[ $task == json ]]; then input=$work/inputs/json-1m.json; fi
    printf '# round=%s task=%s variant=%s start ' "$round" "$task" "$variant"; uptime
    taskset -c "$core" java -Xms512m -Xmx512m -Xss16m -XX:+UseSerialGC -XX:ActiveProcessorCount=1 \
      -cp "$work/$side/classes" JavaLl1Bench "$task" "$variant" "$input"
    printf '# round=%s task=%s variant=%s end ' "$round" "$task" "$variant"; uptime
  }
  for ((round=1; round<=rounds; round++)); do
    variants=(before off on)
    case $((round % 3)) in
      2) variants=(off on before) ;;
      0) variants=(on before off) ;;
    esac
    for task in json p4; do
      for variant in "${variants[@]}"; do run; done
    done
  done
  ;;
*) exit 2 ;;
esac
