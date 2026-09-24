#!/usr/bin/env bash
# 旧生成物を同じ計測法にかける。使用法: legacy-bench.sh TINYEXPRESSION_REPO root|root-off|root-deferred|mapper|mapper-off|facade [fixture,...]
#
# 環境変数:
#   MAVEN_REPO   unlaxer の jar を取る maven repository（既定 $HOME/.m2/repository）
#   CP_ORDER     jar-first（既定）… unlaxer jar を先に置き、tinyexpression のフォーク（AbstractParser 等）の shadow を避ける
#                tiny-first      … tinyexpression 自身の JMH / test と同じ順（target/test-classes:target/classes:依存 jar）
#   DEPS_CP      tiny-first のとき依存 jar 列を明示する（run-java-parser-benchmark.sh が書く jmh-classpath.txt の中身）
# 既定の CP_ORDER=jar-first は tinyexpression が測っている runtime とは別物になる。
# 旧実装のベースラインを取り直すときは CP_ORDER=tiny-first を使う（docs/reports/2026-09-23-legacy-baseline.md）。
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
if [[ $# -lt 2 ]]; then echo '使用法: legacy-bench.sh TINYEXPRESSION_REPO root|root-off|root-deferred|mapper|mapper-off|facade [fixture,...]' >&2; exit 2; fi
tiny=$(cd "$1" && pwd); kind=$2; fixtures=${3:-complex,complex-x4,complex-x16,complex-x64}
repo=${MAVEN_REPO:-$HOME/.m2/repository}; version=${UNLAXER_VERSION:-3.0.15}
common="$repo/org/unlaxer/unlaxer-common/$version/unlaxer-common-$version.jar"
dsl="$repo/org/unlaxer/unlaxer-dsl/$version/unlaxer-dsl-$version.jar"
case "${CP_ORDER:-jar-first}" in
  jar-first) cp="$module/target/classes:$common:$dsl:$tiny/target/classes" ;;
  tiny-first) cp="$tiny/target/test-classes:$tiny/target/classes:${DEPS_CP:-$common:$dsl}:$module/target/classes" ;;
  *) echo "CP_ORDER は jar-first か tiny-first" >&2; exit 2 ;;
esac
# 全体の上限は BENCH_TIMEOUT（既定 600 秒）。
exec timeout "${BENCH_TIMEOUT:-600}s" java -Xms512m -Xmx2g -Xss16m -Dtinyexpression.p4.memoize=true \
  -Dtinyexpression.p4.parse.timeout.millis="${LEGACY_TIMEOUT_MS:-0}" ${BENCH_JVM_OPTS:-} \
  --add-opens=java.base/java.util=ALL-UNNAMED --add-opens=java.base/java.lang=ALL-UNNAMED \
  -cp "$cp" org.unlaxer.ubnfc.examples.p4.LegacyBenchmark "$kind" "$module/src/test/resources/fixtures" "$fixtures"
