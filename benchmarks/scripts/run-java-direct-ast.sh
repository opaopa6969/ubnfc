#!/usr/bin/env bash
# #4: WORK=$(mktemp -d) benchmarks/scripts/run-java-direct-ast.sh prepare|time
# 同じ JVM の Recipe/direct と認識対照を交互実行し、同じ core の Rust を挟む。
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
work=${WORK:?WORK に mktemp -d で作った専用ディレクトリを指定する}
export RUSTUP_TOOLCHAIN=stable
case "${1:?prepare または time}" in
prepare)
  mkdir -p "$work/classes" "$work/generated" "$work/rust/src"
  mvn -q -f "$repo/ubnfc-java/pom.xml" -DskipTests package
  UBNFC_GENERATOR_JAR="$repo/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar" bash "$repo/examples/p4-java/scripts/generate.sh" --check
  mvn -q -f "$repo/examples/p4-java/pom.xml" -DskipTests package
  for grammar in operators power; do
    case "$grammar" in operators) ir_name=associative__Operators ;; power) ir_name=right-associative__Power ;; esac
    java -cp "$repo/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar" org.unlaxer.ubnfc.java.Main \
      --ir "$repo/ir/fixtures/unlaxer-dsl__src__test__resources__$ir_name.ubnf.ir.json" --out "$work/generated" --package "gen.$grammar"
  done
  rg --files "$work/generated" -g '*.java' > "$work/sources"
  javac --release 17 -d "$work/classes" @"$work/sources"
  # 手書き目標形は既存 pom と同じ Java 21。生成物の release 17 条件と分ける。
  rg --files "$repo/examples/handwritten-target/java/src/main/java" -g '*.java' > "$work/handwritten-sources"
  javac --release 21 -d "$work/classes" @"$work/handwritten-sources"
  javac --release 17 -Xlint:all -Werror -d "$work/classes" "$repo/benchmarks/java/JavaDirectAstBench.java"
  cp "$repo/benchmarks/src/java_direct_ast.rs" "$work/rust/src/main.rs"
  cat > "$work/rust/Cargo.toml" <<TOML
[workspace]
[package]
name = "java-direct-ast-reference"
version = "0.0.0"
edition = "2021"
[dependencies]
ubnfc-p4-rust = { path = "$repo/examples/p4-rust" }
TOML
  sha256sum "$repo/ir/frozen/tinyexpression-p4-f86ce8a5.ir.json" \
    "$repo/ir/fixtures/unlaxer-dsl__src__test__resources__associative__Operators.ubnf.ir.json" \
    "$repo/ir/fixtures/unlaxer-dsl__src__test__resources__right-associative__Power.ubnf.ir.json" > "$work/ir.sha256"
  cargo build --release --manifest-path "$work/rust/Cargo.toml"
  ;;
time)
  mkdir -p "$work/results"
  cp "$work/ir.sha256" "$work/results/ir.sha256"
  core=${CORE:-28}
  bench_cp="$work/classes:$repo/examples/p4-java/target/classes"
  for session in $(seq 1 "${SESSIONS:-3}"); do
    uptime | tee "$work/results/s$session-load-before.txt"
    taskset -c "$core" java -Xms512m -Xmx2g -Xss16m -Dbench.runs=7 -Dbench.warmup=1000 \
      -cp "$bench_cp" JavaDirectAstBench p4 "$repo/examples/p4-java/src/test/resources/fixtures" \
      > "$work/results/s$session-p4.csv" 2> "$work/results/s$session-p4.log"
    taskset -c "$core" "$work/rust/target/release/java-direct-ast-reference" \
      "$repo/examples/p4-java/src/test/resources/fixtures" > "$work/results/s$session-rust.csv"
    taskset -c "$core" java -Xms512m -Xmx2g -Xss16m -Dbench.runs=7 \
      -cp "$bench_cp" JavaDirectAstBench operators > "$work/results/s$session-operators.csv" 2> "$work/results/s$session-operators.log"
    uptime | tee "$work/results/s$session-load-after.txt"
  done
  ;;
*) echo 'prepare または time を指定する' >&2; exit 2 ;;
esac
