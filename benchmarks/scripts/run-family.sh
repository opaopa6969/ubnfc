#!/usr/bin/env bash
# 1 つの family の全実装を 1 セッションで交互に測る。prepare.sh の後に使う。
#   WORK=... benchmarks/scripts/run-family.sh a|b|c [出力ディレクトリ]
# CPU は CORE（既定 30）へ固定する。結果は CSV で標準出力と $out/family-<x>-*.csv へ。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?}; family=${1:?a|b|c}; out=${2:-$work/results}
unlaxer=${UNLAXER_PARSER:-$(cd "$ubnfc/../unlaxer-parser" && pwd)}
core=${CORE:-30}
mkdir -p "$out"
pin=(taskset -c "$core")
jvm=(-Xms512m -Xmx2g -Xss16m)
deps=$(cat "$work/dsl-cp.txt")
echo "# load average: $(cut -d' ' -f1-3 /proc/loadavg)" >&2

case "$family" in
a)
  fixtures=$ubnfc/examples/p4-rust/tests/fixtures
  UBNFC_CMP_RUN_SECONDS=${RUN_SECONDS:-1.0} UBNFC_CMP_RUNS=7 \
    "${pin[@]}" "$work/bench-a/target/release/ubnfc-cmp-family-a" "$fixtures" \
    complex complex-x4 complex-x16 complex-x64 comparison-heavy comparison-heavy-x64 \
    | tee "$out/family-a-rust.csv"
  # ubnfc Java（生成）。comparison-heavy 系は入口が別なので分ける。
  "${pin[@]}" java "${jvm[@]}" -Dbench.warmup=2000 -Dbench.runs=7 \
    -Dbench.fixtures=complex,complex-x4,complex-x16,complex-x64 \
    -cp "$ubnfc/examples/p4-java/target/classes" org.unlaxer.ubnfc.examples.p4.Benchmark "$fixtures" \
    | tee "$out/family-a-java-ubnfc.csv"
  "${pin[@]}" java "${jvm[@]}" -Dbench.warmup=2000 -Dbench.runs=7 \
    -Dbench.fixtures=comparison-heavy,comparison-heavy-x64 -Dbench.entry=BooleanExpression \
    -cp "$ubnfc/examples/p4-java/target/classes" org.unlaxer.ubnfc.examples.p4.Benchmark "$fixtures" \
    | tee -a "$out/family-a-java-ubnfc.csv"
  # 旧 Java（tinyexpression 生成物 + unlaxer combinator runtime）。
  # 構成は docs/reports/2026-09-23-legacy-baseline.md §4 の B3（private m2 + tiny-first + SAFE_FAILURES）。
  # 1 parse が 14 ms〜1.6 秒なので、fixture ごとに warmup と反復数を落とす。
  for kind in root mapper facade; do
    for stage in complex:200:200 complex-x4:60:40 complex-x16:20:8 complex-x64:5:3; do
      fixture=${stage%%:*}; rest=${stage#*:}; warm=${rest%%:*}; iter=${rest##*:}
      MAVEN_REPO="$work/m2" CP_ORDER=tiny-first DEPS_CP="$(cat "$work/tinyexpression/target/jmh-classpath.txt")" \
        BENCH_TIMEOUT=2400 BENCH_JVM_OPTS="-Dbench.runs=7 -Dbench.warmup=$warm -Dbench.iterations=$iter" \
        "${pin[@]}" bash "$ubnfc/examples/p4-java/scripts/legacy-bench.sh" "$work/tinyexpression" "$kind" "$fixture" \
        | tee -a "$out/family-a-java-legacy.csv"
    done
  done
  ;;
b)
  "${pin[@]}" "$work/bench-b/target/release/ubnfc-cmp-family-b" | tee "$out/family-b-rust.csv"
  "${pin[@]}" java "${jvm[@]}" -Dbench.runs=7 -Dbench.runSeconds="${RUN_SECONDS:-1.0}" \
    -cp "$work/benchjava:$work/genjava/classes:$ubnfc/examples/handwritten-target/java/target/classes" \
    FamilyBJavaBench | tee "$out/family-b-java.csv"
  ;;
c)
  inputs=(
    "ubnf.ubnf=$unlaxer/unlaxer-dsl/grammar/ubnf.ubnf"
    "elements=$unlaxer/rust/unlaxer-ubnf/tests/fixtures/positive/elements.ubnf"
    "boundaries=$unlaxer/rust/unlaxer-ubnf/tests/fixtures/positive/boundaries.ubnf"
    "all-tokens=$unlaxer/rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf"
    "annotations=$unlaxer/rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf"
  )
  "${pin[@]}" "$work/bench-c/target/release/ubnfc-cmp-family-c" "${inputs[@]}" | tee "$out/family-c-rust.csv"
  "${pin[@]}" java "${jvm[@]}" -Dbench.warmup="${UBNF_WARMUP:-100}" -Dbench.runs=7 -Dbench.iterations="${UBNF_ITERATIONS:-20}" \
    --add-opens=java.base/java.util=ALL-UNNAMED --add-opens=java.base/java.lang=ALL-UNNAMED \
    -cp "$work/benchjava:$unlaxer/unlaxer-dsl/target/classes:$deps" UbnfLegacyBench "${inputs[@]}" \
    | tee "$out/family-c-java.csv"
  ;;
*) echo "family は a|b|c" >&2; exit 2 ;;
esac
echo "# load average (終了時): $(cut -d' ' -f1-3 /proc/loadavg)" >&2
