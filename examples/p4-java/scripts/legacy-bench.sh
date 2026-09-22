#!/usr/bin/env bash
# 旧生成物を同じ計測法にかける。使用法: legacy-bench.sh TINYEXPRESSION_REPO mapper|facade [fixture,...]
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
if [[ $# -lt 2 ]]; then echo '使用法: legacy-bench.sh TINYEXPRESSION_REPO mapper|facade [fixture,...]' >&2; exit 2; fi
tiny=$(cd "$1" && pwd); kind=$2; fixtures=${3:-complex,complex-x4,complex-x16,complex-x64}
repo=${MAVEN_REPO:-$HOME/.m2/repository}; version=${UNLAXER_VERSION:-3.0.15}
common="$repo/org/unlaxer/unlaxer-common/$version/unlaxer-common-$version.jar"
dsl="$repo/org/unlaxer/unlaxer-dsl/$version/unlaxer-dsl-$version.jar"
# jar を先に置き、tinyexpression が持つコアクラスの shadow を避ける。全体の上限は BENCH_TIMEOUT（既定 600 秒）。
exec timeout "${BENCH_TIMEOUT:-600}s" java -Xms512m -Xmx2g -Xss16m -Dtinyexpression.p4.memoize=true \
  -Dtinyexpression.p4.parse.timeout.millis="${LEGACY_TIMEOUT_MS:-0}" ${BENCH_JVM_OPTS:-} \
  --add-opens=java.base/java.util=ALL-UNNAMED --add-opens=java.base/java.lang=ALL-UNNAMED \
  -cp "$module/target/classes:$common:$dsl:$tiny/target/classes" org.ubnfc.p4.LegacyBenchmark "$kind" "$module/src/test/resources/fixtures" "$fixtures"
