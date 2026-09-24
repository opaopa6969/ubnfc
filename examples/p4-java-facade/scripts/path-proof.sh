#!/usr/bin/env bash
# tinyexpression 自身のテスト（無変更）を、facade を差し替えた状態／差し替えない状態で走らせる。
#
#   path-proof.sh legacy   … tinyexpression 本来の P4PreferredAstMapper
#   path-proof.sh shadow   … target/shadow-classes を classpath 先頭に置いた ubnfc 版
#
# tinyexpression 側は target/classes と target/test-classes を読むだけで、書き込まない。
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
mode=${1:-shadow}
tiny=${TINYEXPRESSION_DIR:-$(cd "$module/../../../tinyexpression" && pwd)}
repo=${MAVEN_REPO:-$HOME/.m2/repository}
version=${UNLAXER_VERSION:-3.0.15}
te="$module/target/te/tinyExpression-worktree.jar"
classes="$module/target/classes"
tests="$module/target/test-classes"

jars=(
  "$repo/org/unlaxer/unlaxer-common/$version/unlaxer-common-$version.jar"
  "$repo/org/unlaxer/unlaxer-dsl/$version/unlaxer-dsl-$version.jar"
  "$repo/junit/junit/4.13.2/junit-4.13.2.jar"
  "$repo/org/hamcrest/hamcrest-core/1.3/hamcrest-core-1.3.jar"
  "$repo/org/jetbrains/annotations/24.0.1/annotations-24.0.1.jar"
  "$repo/net/arnx/jsonic/1.3.10/jsonic-1.3.10.jar"
  "$repo/com/fasterxml/jackson/core/jackson-databind/2.18.3/jackson-databind-2.18.3.jar"
  "$repo/com/fasterxml/jackson/core/jackson-core/2.18.3/jackson-core-2.18.3.jar"
  "$repo/com/fasterxml/jackson/core/jackson-annotations/2.18.3/jackson-annotations-2.18.3.jar"
  "$repo/org/junit/jupiter/junit-jupiter-api/5.10.2/junit-jupiter-api-5.10.2.jar"
  "$repo/org/junit/jupiter/junit-jupiter-params/5.10.2/junit-jupiter-params-5.10.2.jar"
  "$repo/org/junit/jupiter/junit-jupiter-engine/5.10.2/junit-jupiter-engine-5.10.2.jar"
  "$repo/org/junit/vintage/junit-vintage-engine/5.10.2/junit-vintage-engine-5.10.2.jar"
  "$repo/org/junit/platform/junit-platform-launcher/1.10.2/junit-platform-launcher-1.10.2.jar"
  "$repo/org/junit/platform/junit-platform-engine/1.10.2/junit-platform-engine-1.10.2.jar"
  "$repo/org/junit/platform/junit-platform-commons/1.10.2/junit-platform-commons-1.10.2.jar"
  "$repo/org/opentest4j/opentest4j/1.3.0/opentest4j-1.3.0.jar"
  "$repo/org/apiguardian/apiguardian-api/1.1.2/apiguardian-api-1.1.2.jar"
)
for jar in "${jars[@]}"; do [[ -f "$jar" ]] || { echo "jar が無い: $jar" >&2; exit 1; }; done

shadow=""
if [[ "$mode" == "shadow" ]]; then
  bash "$module/scripts/build-shadow.sh" >/dev/null
  shadow="$module/target/shadow-classes:"
fi
# 先頭から: shadow（あれば） -> tinyexpression のテストと本体 -> この module -> 依存 jar。
cp="$shadow$tiny/target/test-classes:$te:$classes:$tests"
for jar in "${jars[@]}"; do cp="$cp:$jar"; done

out="$module/target/path-proof/$mode.tsv"
cd "$tiny"   # テストが相対パスで読む資材があるため。書き込みはしない。
exec timeout "${PATH_PROOF_TIMEOUT:-1800}s" java -Xmx2g -Xss16m \
  --add-opens=java.base/java.util=ALL-UNNAMED --add-opens=java.base/java.lang=ALL-UNNAMED \
  -cp "$cp" org.unlaxer.tinyexpression.p4.ubnfc.PathProofRunner \
  "$module/src/test/resources/path-proof-classes.txt" "$out"
