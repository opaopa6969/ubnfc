#!/usr/bin/env bash
# P4（v1、手書き優先順位塔）と P4-v2（@ubnf: 2 の左再帰ルール）を同じ入力で比較する。
#
#   scripts/compare.sh [--timing]
#
# 1. 両方の Grammar IR を作り、Rust backend で parser を生成して release build する。
# 2. 同じ fixture を両 driver に投げ、応答（accepted / consumed / ast / diagnostics …）を比較する。
# 3. --timing のときは同じ入力で解析時間も測る（認識のみ / AST 構築）。
#
# 一時ファイルは mktemp -d の専用ディレクトリに置く。生成物は commit しない。
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
root=$(cd "$module/../.." && pwd)
timing=${1:-}

# 既定は専用の一時ディレクトリ。P4V2_WORK を渡すと生成物を再利用する（反復用）。
if [ -n "${P4V2_WORK:-}" ]; then
  work="$P4V2_WORK"
  mkdir -p "$work"
else
  work=$(mktemp -d)
  trap 'rm -rf "$work"' EXIT
fi

echo "== 生成器を build =="
(cd "$root" && cargo build -q --offline --release --bin ubnfc -p ubnfc-front)
(cd "$root" && cargo build -q --offline --release --bin ubnfc-rust -p ubnfc-rust)
ubnfc="$root/target/release/ubnfc"
ubnfc_rust="$root/target/release/ubnfc-rust"

v1_ir="$root/ir/frozen/tinyexpression-p4-f86ce8a5.ir.json"
v2_ir="$work/p4-v2.ir.json"
echo "== P4-v2 の IR =="
(cd "$root" && "$ubnfc" ir --grammar examples/p4-v2/tinyexpression-p4-v2.ubnf \
  --extern-first-chars scanners/first-chars.json --out "$v2_ir")

build_variant() {
  local name=$1 ir=$2
  if [ -x "$work/$name/target/release/driver" ] && [ -n "${P4V2_REUSE:-}" ]; then
    echo "  （再利用）"
    return
  fi
  rm -rf "$work/$name"
  "$ubnfc_rust" --ir "$ir" --out "$work/$name" --emit-driver >/dev/null
  # scanner registry（Extern token）は harness の runner と同じものを写す。
  cp "$root/scanners/rust/scanners.rs" "$work/$name/src/bin/scanners.rs"
  cp "$root/scanners/rust/scanners_tests.rs" "$work/$name/src/bin/scanners_tests.rs"
  cp "$module/scripts/timing.rs" "$work/$name/src/bin/timing.rs"
  python3 "$module/scripts/prepare_manifest.py" "$work/$name/Cargo.toml"
  (cd "$work/$name" && cargo build -q --offline --release --bins)
}

echo "== v1 parser を生成・build =="
build_variant v1 "$v1_ir"
echo "== v2 parser を生成・build =="
build_variant v2 "$v2_ir"

fixtures="$root/examples/p4-rust/tests/fixtures"
echo "== 応答の比較 =="
python3 "$module/scripts/compare.py" \
  "$work/v1/target/release/driver" "$work/v2/target/release/driver" "$fixtures"

if [ "$timing" = "--timing" ]; then
  echo "== 解析時間 =="
  for variant in v1 v2; do
    echo "--- $variant ---"
    "$work/$variant/target/release/timing" \
      "$fixtures/complex.tiny" "$fixtures/complex-x4.tiny" "$fixtures/complex-x16.tiny" \
      "$fixtures/complex-x64.tiny" "$fixtures/flat-arithmetic.tiny" \
      "$fixtures/comparison-heavy.tiny" "$fixtures/large-match.tiny"
  done
fi
