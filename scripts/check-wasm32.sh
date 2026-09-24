#!/usr/bin/env bash
# issue #65 / D-070 wasm: 生成 Rust parser が wasm32-unknown-unknown でビルドできることを確認する。
#
#   scripts/check-wasm32.sh
#
# 対象:
#   - examples/p4-rust     （commit 済みクレート、tinyexpression P4。lib のみ、driver bin は host 前提なので --lib）
#   - crates/ubnfc-selfhost （commit 済みクレート、UBNF self-host。同上 --lib）
#   - examples/json         （committed crate ではない。ubnfc / ubnfc-rust CLI で都度生成する。
#                             JSON Lines driver bin 込みで生成し、実際にリンクされる .wasm のサイズを測る）
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here"

if ! rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then
  echo "# wasm32-unknown-unknown target を追加する" >&2
  rustup target add wasm32-unknown-unknown
fi

echo "== build ubnfc / ubnfc-rust CLI (host) ==" >&2
cargo build --offline --release --bin ubnfc --bin ubnfc-rust

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "== generate examples/json parser (json.ubnf, JSON Lines driver bin 込み) into $work/json ==" >&2
./target/release/ubnfc ir --grammar examples/json/json.ubnf --out "$work/json.ir.json"
./target/release/ubnfc-rust --ir "$work/json.ir.json" --out "$work/json"

echo "== cargo build --release --target wasm32-unknown-unknown (examples/json, generated driver bin) ==" >&2
(cd "$work/json" && cargo build --offline --release --target wasm32-unknown-unknown)
json_wasm=$(find "$work/json/target/wasm32-unknown-unknown/release" -maxdepth 1 -name '*.wasm' | head -1)
if [ -n "${json_wasm:-}" ]; then
  size_default=$(stat -c '%s' "$json_wasm")
  echo "# json driver.wasm (release, opt-level既定): ${size_default} bytes" >&2
  echo "== cargo rustc -C opt-level=z（サイズ比較のみ。profile は変更しない） ==" >&2
  (cd "$work/json" && cargo rustc --offline --release --target wasm32-unknown-unknown --bin driver -- -C opt-level=z)
  size_z=$(stat -c '%s' "$json_wasm")
  echo "# json driver.wasm (release, opt-level=z): ${size_z} bytes" >&2
fi

echo "== cargo build --release --target wasm32-unknown-unknown -p ubnfc-p4-rust --lib ==" >&2
cargo build --offline --release --target wasm32-unknown-unknown -p ubnfc-p4-rust --lib

echo "== cargo build --release --target wasm32-unknown-unknown -p ubnfc-selfhost --lib ==" >&2
cargo build --offline --release --target wasm32-unknown-unknown -p ubnfc-selfhost --lib

echo "== wasm32 build OK ==" >&2
