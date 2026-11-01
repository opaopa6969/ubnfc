#!/bin/sh
# 実際の Cargo / rustc エラーを起こす、外部 fixture 不要の合成生成器。
set -eu
out=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --out) out=$2; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out/src/bin"
cat > "$out/Cargo.toml" <<'MANIFEST'
[package]
name = "fuzz-build-failure"
version = "0.1.0"
edition = "2021"
autobins = false
[workspace]
[features]
default = []
scanners = []
[[bin]]
name = "driver"
path = "src/bin/driver.rs"
MANIFEST
cat > "$out/src/bin/driver.rs" <<'RUST'
compile_error!("issue54: 合成 fixture の意図したビルド失敗");
fn main() {}
RUST
