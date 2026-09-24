#!/usr/bin/env bash
# parse / Java コード生成 / javac / 実行 の 4 段を別々に測る。
#   stage-timing.sh legacy … tinyexpression 本来の P4PreferredAstMapper
#   stage-timing.sh shadow … target/shadow-classes を先頭に置いた ubnfc 版
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
mode=${1:-shadow}
tiny=${TINYEXPRESSION_DIR:-$(cd "$module/../../../tinyexpression" && pwd)}
repo=${MAVEN_REPO:-$HOME/.m2/repository}
version=${UNLAXER_VERSION:-3.0.15}
te="$module/target/te/tinyExpression-worktree.jar"
shadow=""
if [[ "$mode" == "shadow" ]]; then
  bash "$module/scripts/build-shadow.sh" >/dev/null
  shadow="$module/target/shadow-classes:"
fi
cp="$shadow$te:$module/target/classes:$module/target/test-classes"
cp="$cp:$repo/org/unlaxer/unlaxer-common/$version/unlaxer-common-$version.jar"
cp="$cp:$repo/org/unlaxer/unlaxer-dsl/$version/unlaxer-dsl-$version.jar"
exec timeout "${TIMING_TIMEOUT:-1800}s" java -Xmx2g -Xss16m \
  --add-opens=java.base/java.util=ALL-UNNAMED --add-opens=java.base/java.lang=ALL-UNNAMED \
  -Dtiming.fixtures="$module/../p4-java/src/test/resources/fixtures" \
  ${TIMING_JVM_OPTS:-} \
  -cp "$cp" org.unlaxer.tinyexpression.evaluator.javacode.StageTimingRunner \
  "$module/target/timing/$mode.tsv"
