#!/usr/bin/env bash
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
exec java -Xms512m -Xmx2g -Xss16m ${BENCH_JVM_OPTS:-} -cp "$module/target/classes" \
  org.unlaxer.ubnfc.examples.p4.Benchmark "${1:-$module/src/test/resources/fixtures}"
