#!/usr/bin/env bash
# 速度比較（docs/reports/2026-09-23-parser-speed-comparison.md）の下ごしらえ。
# 兄弟リポジトリのソース・pom・git 状態は一切変更しない。生成物と private maven repo は $WORK 配下に置く。
#
#   WORK=$(mktemp -d) benchmarks/scripts/prepare.sh
#
# 作るもの:
#   $WORK/m2                 unlaxer-parser master を install した private maven repository
#   $WORK/tinyexpression     tinyexpression のソースコピーを $WORK/m2 の jar でビルドしたもの
#   $WORK/gen/{operators,power}  Operators / Power の生成 Rust crate
#   $WORK/genjava/classes    Operators / Power の生成 Java クラス
#   $WORK/bench-{a,b,c}      family ごとの Rust 計測 crate
#   $WORK/benchjava          Java 計測ドライバのクラス
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?WORK を指定する}
unlaxer=${UNLAXER_PARSER:-$(cd "$ubnfc/../unlaxer-parser" && pwd)}
tinyexpression=${TINYEXPRESSION:-$(cd "$ubnfc/../tinyexpression" && pwd)}
seed=${M2_SEED:-}

mkdir -p "$work"

# 1. unlaxer master を private repo へ install（共有 ~/.m2 は触らない）。
if [[ -n "$seed" && -d "$seed" ]]; then cp -r "$seed" "$work/m2"; rm -rf "$work/m2/org/unlaxer"; fi
(cd "$unlaxer" && mvn -B -pl .,unlaxer-common,unlaxer-dsl install -DskipTests -Dgpg.skip=true -Dmaven.repo.local="$work/m2")
(cd "$unlaxer" && mvn -o -q -B -pl unlaxer-dsl -Dmaven.repo.local="$work/m2" \
  org.apache.maven.plugins:maven-dependency-plugin:3.7.0:build-classpath \
  -DincludeScope=runtime -Dmdep.outputFile="$work/dsl-cp.txt")

# 2. tinyexpression をコピーしてその private repo でビルドし直す。
#    共有ツリーの target/classes は 2026-08-28 のままで memo marker が 0 個なので使えない
#    （docs/reports/2026-09-23-legacy-baseline.md §1）。
rm -rf "$work/tinyexpression"; mkdir -p "$work/tinyexpression"
(cd "$tinyexpression" && tar --exclude=./target --exclude=./.git --exclude='*/target' -cf - .) | tar -xf - -C "$work/tinyexpression"
(cd "$work/tinyexpression" && mvn -B -Pparser-benchmark -DskipTests -Dtinyexpression.skipRailroad=true \
  -Dmaven.repo.local="$work/m2" test-compile \
  org.apache.maven.plugins:maven-dependency-plugin:3.7.0:build-classpath \
  -DincludeScope=test -Dmdep.outputFile="$work/tinyexpression/target/jmh-classpath.txt")

# 3. ubnfc の Java 生成器と例題をビルド。
(cd "$ubnfc/ubnfc-java" && mvn -o -q -DskipTests package)
(cd "$ubnfc/examples/p4-java" && mvn -o -q -DskipTests package)
(cd "$ubnfc/examples/handwritten-target/java" && mvn -o -q -DskipTests test-compile)

# 4. Operators / Power を Rust と Java へ生成する。
mkdir -p "$work/gen" "$work/genjava/src" "$work/genjava/classes" "$work/benchjava"
jar=$ubnfc/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar
for pair in operators:associative__Operators power:right-associative__Power; do
  name=${pair%%:*}; file=${pair##*:}
  ir=$ubnfc/ir/fixtures/unlaxer-dsl__src__test__resources__${file}.ubnf.ir.json
  rm -rf "$work/gen/$name"
  (cd "$ubnfc" && cargo run -q --offline --release -p ubnfc-rust --bin ubnfc-rust -- --ir "$ir" --out "$work/gen/$name" --no-emit-driver)
  sed -i "s/^name = \"ubnfc-generated\"/name = \"gen-$name\"/" "$work/gen/$name/Cargo.toml"
  java -cp "$jar" org.unlaxer.ubnfc.java.Main --ir "$ir" --out "$work/genjava/src" --package "gen.$name" --no-emit-driver
done
javac -nowarn -d "$work/genjava/classes" $(find "$work/genjava/src" -name '*.java')

# 5. family ごとの Rust 計測 crate（ubnfc の cargo workspace には入れない）。
mk_crate() {
  local dir=$1 main=$2; shift 2
  rm -rf "$dir"; mkdir -p "$dir/src"
  cp "$ubnfc/benchmarks/src/bench_util.rs" "$dir/src/"
  cp "$ubnfc/benchmarks/src/$main" "$dir/src/main.rs"
  { echo '[workspace]'; echo '[package]'; echo "name = \"$(basename "$dir")\""
    echo 'version = "0.0.0"'; echo 'edition = "2021"'; echo '[dependencies]'; printf '%s\n' "$@"; } > "$dir/Cargo.toml"
  (cd "$dir" && cargo build --offline --release)
}
mk_crate "$work/bench-a" family_a.rs \
  "ubnfc-p4-rust = { path = \"$ubnfc/examples/p4-rust\" }" \
  "tinyexpression-rs = { path = \"$tinyexpression/rust/tinyexpression-rs\" }" \
  'unlaxer-runtime = { git = "https://github.com/opaopa6969/unlaxer-parser.git", rev = "6a4187c73f748282524fac579415bcd092ae9c6e" }'
mk_crate "$work/bench-b" family_b.rs \
  "gen-operators = { path = \"$work/gen/operators\" }" \
  "gen-power = { path = \"$work/gen/power\" }" \
  "ubnfc-target-rust = { path = \"$ubnfc/examples/handwritten-target/rust\" }"
mk_crate "$work/bench-c" family_c.rs \
  "ubnfc-front = { path = \"$ubnfc/crates/ubnfc-front\" }" \
  "ubnfc-selfhost = { path = \"$ubnfc/crates/ubnfc-selfhost\" }" \
  "unlaxer-ubnf = { path = \"$unlaxer/rust/unlaxer-ubnf\" }"

# 6. Java 計測ドライバ。
javac -nowarn -cp "$work/genjava/classes:$ubnfc/examples/handwritten-target/java/target/classes" \
  -d "$work/benchjava" "$ubnfc/benchmarks/java/FamilyBJavaBench.java"
javac -nowarn -cp "$unlaxer/unlaxer-dsl/target/classes:$(cat "$work/dsl-cp.txt")" \
  -d "$work/benchjava" "$ubnfc/benchmarks/java/UbnfLegacyBench.java"
echo "prepare: 完了 ($work)"
