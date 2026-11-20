#!/usr/bin/env bash
set -euo pipefail
work=$(cd "$(dirname "$0")" && pwd)
repo=${REPO:-"$work/wt"}
out="$work/generated"
jar=ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar
json=ir/examples/json-v2.ir.json
operators=ir/fixtures/unlaxer-dsl__src__test__resources__associative__Operators.ubnf.ir.json
p4=ir/frozen/tinyexpression-p4-f86ce8a5.ir.json
p4_source="$repo/examples/p4-java/src/main/java/org/unlaxer/ubnfc/examples/p4/P4Scanners.java"
fixtures="$repo/examples/p4-java/src/test/resources/fixtures"
p4_package=org.unlaxer.ubnfc.examples.p4.generated

generate() {
  local name=$1 generator=$2
  shift 2
  local dest="$out/$name"
  mkdir -p "$dest"
  java -cp "$generator" org.unlaxer.ubnfc.java.Main --ir "$repo/$json" --out "$dest/src" --package gen.json "$@"
  java -cp "$generator" org.unlaxer.ubnfc.java.Main --ir "$repo/$operators" --out "$dest/src" --package gen.operators "$@"
  java -cp "$generator" org.unlaxer.ubnfc.java.Main --ir "$repo/$p4" --out "$dest/src" --package "$p4_package" "$@"
  find "$dest/src" -name '*.java' | sort > "$dest/sources"
  javac --release 17 -d "$dest/classes" @"$dest/sources" "$p4_source" "$work/ShortScanBench.java"
}

case ${1:?prepare、time または jfr を指定してください} in
prepare)
  generate scalar "$work/before/$jar"
  generate before "$work/before/$jar" --swar
  generate candidate "$repo/$jar" --swar
  generate default "$repo/$jar"
  generate old "$work/old/$jar"
  diff -qr "$out/scalar/src" "$out/default/src"
  for package in gen/operators org; do
    diff -qr "$out/scalar/src/$package" "$out/before/src/$package"
    diff -qr "$out/before/src/$package" "$out/candidate/src/$package"
  done
  grep -q 'long word = scanSwarWord' "$out/before/src/gen/json/JsonParser.java"
  grep -q 'char first = t\[p\]' "$out/candidate/src/gen/json/JsonParser.java"
  (cd "$repo" && sha256sum "$json" "$operators" "$p4" "$fixtures/"*.tiny)
  (cd "$work/inputs" && sha256sum json-1m.json operators-atoms-512.txt)
  for variant in scalar default before candidate old; do
    (cd "$out/$variant/src" && find . -name '*.java' -print0 | sort -z | xargs -0 sha256sum | sha256sum) |
      awk -v variant="$variant" '{print variant, $1}'
  done
  ;;
time)
  session=${2:?セッション番号を指定してください}
  core=${CORE:-28}
  printf 'session,round,variant,input,sample,warmup,iterations,wall_ms,cpu_ms\n'
  printf '# session=%s core=%s\n' "$session" "$core" >&2
  java -version >&2
  uptime >&2
  variants=(scalar before candidate old)
  # 2 セッションで巡回順と逆順を組み合わせる。
  for (( round=1; round<=7; round++ )); do
    for (( slot=0; slot<4; slot++ )); do
      index=$(( (round - 1 + slot) % 4 ))
      if (( (round + session) % 2 == 1 )); then index=$((3 - index)); fi
      variant=${variants[index]}
      extra=()
      if [[ $variant == old ]]; then extra=(json); fi
      printf 'round=%s variant=%s start ' "$round" "$variant" >&2
      uptime >&2
      taskset -c "$core" java -Xms512m -Xmx512m -Xss16m -XX:+UseSerialGC -XX:ActiveProcessorCount=1 \
        -cp "$out/$variant/classes" ShortScanBench "$work/inputs" "$fixtures" "${extra[@]}" |
        sed "s/^/$session,$round,$variant,/"
      printf 'round=%s variant=%s end ' "$round" "$variant" >&2
      uptime >&2
    done
  done
  uptime >&2
  ;;
jfr)
  for variant in scalar before candidate; do
    printf '%s start ' "$variant"
    uptime
    taskset -c "${CORE:-28}" java -Xms512m -Xmx512m -Xss16m -XX:+UseSerialGC -XX:ActiveProcessorCount=1 \
      -XX:FlightRecorderOptions=stackdepth=256 -cp "$out/$variant/classes" ShortScanBench \
      "$work/inputs" "$fixtures" "$work/$variant.jfr"
    printf '%s end ' "$variant"
    uptime
  done
  ;;
*) exit 2 ;;
esac
