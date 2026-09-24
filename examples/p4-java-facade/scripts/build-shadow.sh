#!/usr/bin/env bash
# shadow 用 P4PreferredAstMapper を target/shadow-classes へ単独でコンパイルする。
# この dir だけを classpath の先頭に置けば、tinyexpression の同名クラスを隠せる。
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
repo=${MAVEN_REPO:-$HOME/.m2/repository}
version=${UNLAXER_VERSION:-3.0.15}
te="$module/target/te/tinyExpression-worktree.jar"
[[ -f "$te" ]] || { echo "先に scripts/prepare-te.sh を実行する" >&2; exit 1; }
[[ -d "$module/target/classes" ]] || { echo "先に mvn -o -DskipTests package を実行する" >&2; exit 1; }
cp="$module/target/classes:$te:$repo/org/unlaxer/unlaxer-common/$version/unlaxer-common-$version.jar:$repo/org/unlaxer/unlaxer-dsl/$version/unlaxer-dsl-$version.jar"
rm -rf "$module/target/shadow-classes" "$module/target/empty-classes"
mkdir -p "$module/target/shadow-classes" "$module/target/empty-classes"
javac --release 21 -encoding UTF-8 -cp "$cp" -d "$module/target/shadow-classes" \
  "$module/src/shadow/java/org/unlaxer/tinyexpression/p4/P4PreferredAstMapper.java"
echo "生成: $module/target/shadow-classes"
ls "$module/target/shadow-classes/org/unlaxer/tinyexpression/p4/"
