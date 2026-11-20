#!/usr/bin/env bash
# WORK は mktemp -d で取得した専用ディレクトリ。生成器と計測には JDK 21 を使う。
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
work=${WORK:?WORK を指定してください}
jar=ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar
json=ir/examples/json-v2.ir.json
operators=ir/fixtures/unlaxer-dsl__src__test__resources__associative__Operators.ubnf.ir.json
p4=ir/frozen/tinyexpression-p4-f86ce8a5.ir.json
p4_src=examples/p4-java/src/main/java/org/unlaxer/ubnfc/examples/p4
fixtures="$repo/examples/p4-java/src/test/resources/fixtures"

generate() {
  local generator=$1 out=$2
  shift 2
  java -cp "$generator" org.unlaxer.ubnfc.java.Main --ir "$repo/$json" --out "$out" --package gen.json "$@"
  java -cp "$generator" org.unlaxer.ubnfc.java.Main --ir "$repo/$operators" --out "$out" --package gen.operators "$@"
  java -cp "$generator" org.unlaxer.ubnfc.java.Main --ir "$repo/ir/fixtures/unlaxer-dsl__src__test__resources__right-associative__Power.ubnf.ir.json" --out "$out" --package gen.power "$@"
  java -cp "$generator" org.unlaxer.ubnfc.java.Main --ir "$repo/$p4" --out "$out" --package org.unlaxer.ubnfc.examples.p4.generated "$@"
}

case "${1:?prepare または time を指定してください}" in
prepare)
  base=${BASE_REF:?BASE_REF に比較基準の commit を指定してください}
  suite=${JSON_TEST_SUITE:?JSON_TEST_SUITE に固定 JSONTestSuite の展開先を指定してください}
  mkdir "$work/base"
  git -C "$repo" archive "$base" | tar -x -C "$work/base"
  git -C "$repo" rev-parse "$base" > "$work/base-commit.txt"
  mvn -q -f "$work/base/pom.xml" -DskipTests package
  mvn -q -f "$repo/pom.xml" -DskipTests package
  python3 "$repo/benchmarks/scripts/make-json-inputs.py" "$suite/test_parsing" "$work/inputs"
  cmp "$repo/examples/json/evidence/inputs.sha256" "$work/inputs/inputs.sha256"
  python3 -c 'print("+".join(["1"] * 512), end="")' > "$work/inputs/operators-atoms-512.txt"
  generate "$work/base/$jar" "$work/before/src"
  generate "$repo/$jar" "$work/default/src"
  generate "$repo/$jar" "$work/after/src" --swar
  diff -qr "$work/before/src" "$work/default/src"
  for package in gen/operators gen/power org; do
    diff -qr "$work/before/src/$package" "$work/after/src/$package"
  done
  grep -q 'long word = scanSwarWord' "$work/after/src/gen/json/JsonParser.java"
  for side in before after; do
    rg --files "$work/$side/src" -g '*.java' | sort > "$work/$side/sources"
    javac --release 17 -d "$work/$side/classes" @"$work/$side/sources" "$repo/$p4_src/"{Benchmark,P4Scanners}.java
    javac -cp "$work/$side/classes" -d "$work/$side/classes" "$repo/benchmarks/java/UbnfcRunner.java"
  done
  (cd "$repo" && sha256sum "$json" "$operators" "$p4" "$fixtures/"*.tiny)
  (cd "$work" && sha256sum inputs/json-1m.json inputs/operators-atoms-512.txt)
  ;;
time)
  core=${CORE:-28}
  rounds=${ROUNDS:-7}
  java -version 2>&1
  uname -r
  printf '# core=%s rounds=%s\n' "$core" "$rounds"
  run() {
    printf '# round=%s side=%s task=%s start ' "$round" "$side" "$task"
    uptime
    taskset -c "$core" java -Xms512m -Xmx512m -Xss16m -XX:+UseSerialGC -XX:ActiveProcessorCount=1 \
      -cp "$work/$side/classes" "$@" 2>&1
    printf '# round=%s side=%s task=%s end ' "$round" "$side" "$task"
    uptime
  }
  for ((round=1; round<=rounds; round++)); do
    order=(before after)
    if ((round % 2 == 0)); then order=(after before); fi
    for task in json operators p4-formula p4-boolean; do
      for side in "${order[@]}"; do
        case "$task" in
          json) run UbnfcRunner bench json 認識 1 0.5 "$work/inputs/json-1m.json" ;;
          operators) run UbnfcRunner bench operators 認識 1 0.5 "$work/inputs/operators-atoms-512.txt" ;;
          p4-formula) run -Dbench.warmup=200 -Dbench.runs=3 -Dbench.modes=recognize-fast \
            -Dbench.fixtures=complex,complex-x4,complex-x16,complex-x64 \
            org.unlaxer.ubnfc.examples.p4.Benchmark "$fixtures" ;;
          p4-boolean) run -Dbench.warmup=200 -Dbench.runs=3 -Dbench.modes=recognize-fast \
            -Dbench.fixtures=comparison-heavy,comparison-heavy-x64 -Dbench.entry=BooleanExpression \
            org.unlaxer.ubnfc.examples.p4.Benchmark "$fixtures" ;;
        esac
      done
    done
  done
  ;;
*) exit 2 ;;
esac
