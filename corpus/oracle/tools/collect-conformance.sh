#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C.UTF-8
export PYTHONDONTWRITEBYTECODE=1
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
oracle_repo=${1:-/home/opa/work/unlaxer-workspace/unlaxer-parser}
output_dir=${2:-"$script_dir/.."}
[[ -z "$(git -C "$oracle_repo" status --porcelain)" ]] || { echo '参照リポジトリが dirty です。' >&2; exit 1; }
oracle_revision=$(git -C "$oracle_repo" rev-parse HEAD)
work_dir=$(mktemp -d -t ubnfc-conformance.XXXXXXXX)
trap 'rm -rf -- "$work_dir"' EXIT
mvn -o -q -f "$oracle_repo/pom.xml" -pl unlaxer-common,unlaxer-dsl -DskipTests -Dflatten.skip=true compile > "$work_dir/maven.log" 2>&1 || { cat "$work_dir/maven.log"; exit 1; }
classpath="$oracle_repo/unlaxer-common/target/classes:$oracle_repo/unlaxer-dsl/target/classes:$HOME/.m2/repository/io/vavr/vavr/0.10.4/vavr-0.10.4.jar:$HOME/.m2/repository/com/google/code/gson/gson/2.11.0/gson-2.11.0.jar:$HOME/.m2/repository/junit/junit/4.13.2/junit-4.13.2.jar:$HOME/.m2/repository/org/hamcrest/hamcrest-core/1.3/hamcrest-core-1.3.jar"
test_sources="$oracle_repo/unlaxer-dsl/src/test/java"
mkdir "$work_dir/driver" "$work_dir/runs"
# test source は read-only。javac の出力と TemporaryFolder は mktemp 配下に限定。
javac --enable-preview --release 21 -encoding UTF-8 -cp "$classpath" -sourcepath "$test_sources" -d "$work_dir/driver" \
  "$script_dir/OracleCollector.java" "$script_dir/ConformanceCollector.java" \
  "$test_sources/org/unlaxer/dsl/codegen/RustConformanceTest.java" \
  "$test_sources/org/unlaxer/dsl/codegen/ScopeAnnotationConformanceTest.java" \
  "$test_sources/org/unlaxer/dsl/codegen/ScopeStoreConformanceTest.java" \
  "$test_sources/org/unlaxer/dsl/codegen/NestedCaptureConformanceTest.java"
python3 "$script_dir/conformance_specifications.py" "$oracle_repo" > "$work_dir/specifications.json"
java --enable-preview -Xmx1g -Djava.io.tmpdir="$work_dir/runs" \
  -cp "$work_dir/driver:$classpath:$oracle_repo/unlaxer-dsl/src/test/resources" \
  org.unlaxer.dsl.ConformanceCollector "$work_dir/specifications.json" "$output_dir" "$work_dir/runs" "$oracle_revision" "$oracle_repo"
[[ "$oracle_revision" == "$(git -C "$oracle_repo" rev-parse HEAD)" ]]
[[ -z "$(git -C "$oracle_repo" status --porcelain)" ]]
