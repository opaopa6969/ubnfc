#!/usr/bin/env bash
# 境界レポートの生成パーサ再現。front / backend 自体は変更しない。
set -euo pipefail
export RUSTUP_TOOLCHAIN=stable
export PYTHONDONTWRITEBYTECODE=1
module=$(cd "$(dirname "$0")/.." && pwd)
root=$(cd "$module/../.." && pwd)
cd "$root"
cargo build -q --offline --target-dir "$root/target" --bin ubnfc --bin ubnfc-rust
(cd ubnfc-java && mvn -o -q -DskipTests package)
python3 "$module/scripts/boundaries.py" "$@"
