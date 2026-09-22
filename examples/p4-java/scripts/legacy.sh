#!/usr/bin/env bash
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
if [[ $# -lt 3 ]]; then
  echo '使用法: legacy.sh TINYEXPRESSION_REPO mapper|facade INPUT [OUTPUT_JSON]' >&2
  exit 2
fi
tiny=$(cd "$1" && pwd)
shift
repo=${MAVEN_REPO:-$HOME/.m2/repository}
version=${UNLAXER_VERSION:-3.0.15}
common="$repo/org/unlaxer/unlaxer-common/$version/unlaxer-common-$version.jar"
dsl="$repo/org/unlaxer/unlaxer-dsl/$version/unlaxer-dsl-$version.jar"
# jar を先に置き、tinyexpression が持つコアクラスの shadow を避ける。
exec timeout 45s java -Xmx1g -Xss16m -Dtinyexpression.p4.memoize=true \
  --add-opens=java.base/java.util=ALL-UNNAMED --add-opens=java.base/java.lang=ALL-UNNAMED \
  -cp "$module/target/classes:$common:$dsl:$tiny/target/classes" org.ubnfc.p4.LegacyProbe "$@"
