#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C.UTF-8
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
oracle_repo=${1:-/home/opa/work/unlaxer-workspace/unlaxer-parser}
output_dir=${2:-"$script_dir/.."}
if [[ -n "$(git -C "$oracle_repo" status --porcelain)" ]]; then
  echo '参照リポジトリが clean ではないため採取を停止します。' >&2
  exit 1
fi
oracle_revision=$(git -C "$oracle_repo" rev-parse HEAD)
work_dir=$(mktemp -d -t ubnfc-oracle.XXXXXXXX)
trap 'rm -rf -- "$work_dir"' EXIT
# flatten は通常 target/ 外の .flattened-pom.xml を出力するので明示的に無効化。
mvn -o -f "$oracle_repo/pom.xml" -pl unlaxer-common,unlaxer-dsl -DskipTests -Dflatten.skip=true compile > "$work_dir/maven.log" 2>&1 || { cat "$work_dir/maven.log"; exit 1; }
cat "$work_dir/maven.log"
classpath="$oracle_repo/unlaxer-common/target/classes:$oracle_repo/unlaxer-dsl/target/classes:$HOME/.m2/repository/io/vavr/vavr/0.10.4/vavr-0.10.4.jar:$HOME/.m2/repository/com/google/code/gson/gson/2.11.0/gson-2.11.0.jar"
mkdir "$work_dir/driver" "$work_dir/runs"
javac --enable-preview --release 21 -encoding UTF-8 -cp "$classpath" -d "$work_dir/driver" "$script_dir/OracleCollector.java" "$script_dir/ExternalCursorParser.java"
python3 "$script_dir/specifications.py" > "$work_dir/specifications.json"
java --enable-preview -Xmx1g -cp "$work_dir/driver:$classpath" org.unlaxer.dsl.OracleCollector "$work_dir/specifications.json" "$output_dir" "$work_dir/runs" "$oracle_revision"
[[ "$oracle_revision" == "$(git -C "$oracle_repo" rev-parse HEAD)" ]]
[[ -z "$(git -C "$oracle_repo" status --porcelain)" ]]
