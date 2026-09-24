#!/usr/bin/env bash
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
for stack in 256k 1m 4m 16m; do
  timeout 60s java -Xmx512m -Xss"$stack" -cp "$module/target/classes" org.unlaxer.ubnfc.examples.p4.DepthProbe "${@}"
done
