#!/usr/bin/env bash
# P4 IR から src/generated を再生成する。--check は byte 比較のみ（tests/regenerate.rs と同じ）。
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
root=$(cd "$module/../.." && pwd)
ir="$root/ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json"
mode=${1:---check}
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT
(cd "$root" && cargo run -q --offline -p ubnfc-rust --bin ubnfc-rust -- --ir "$ir" --out "$out" --no-emit-driver)
rm -f "$out/Cargo.toml"
case "$mode" in
  --check) diff -r "$out" "$module/src/generated" && echo "generated: byte 一致" ;;
  --write) rm -rf "$module/src/generated" && cp -r "$out" "$module/src/generated" && echo "generated: 更新" ;;
  *) echo "usage: generate.sh [--check|--write]" >&2; exit 2 ;;
esac
