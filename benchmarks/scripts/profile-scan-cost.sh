#!/usr/bin/env bash
# run-swar-ab.sh prepare 後、同じ JSON 1 MB の全体命令数と関数別自己コストを採る。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
work=${WORK:?専用 worktree 内の WORK を指定する}
image=${IMAGE:?自分専用の計測 image を指定する}
core=${CORE:-30}
count=${COUNT:-10}
profile=$work/callgrind
mkdir -p "$profile/src" "$work/include" "$work/region"
docker run --rm "$image" tar -C /usr/include -cf - valgrind/valgrind.h valgrind/callgrind.h | tar -C "$work/include" -xf -
cc -I"$work/include" -c "$here/../attribution/region.c" -o "$work/region/region.o"
ar rcs "$work/region/libattribution_region.a" "$work/region/region.o"
cat > "$profile/build.rs" <<'RUST'
fn main() {
    let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rustc-link-search=native={}",root.parent().unwrap().join("region").display());
    println!("cargo:rustc-link-lib=static=attribution_region");
}
RUST
cat > "$profile/Cargo.toml" <<TOML
[workspace]
[package]
name = "scan-attribution"
version = "0.0.0"
edition = "2021"
[dependencies]
json-before = { path = "$work/before/json" }
json-after = { path = "$work/after/json" }
[profile.release]
debug = 1
TOML
cp "$here/../src/scan_attribution.rs" "$profile/src/main.rs"
RUSTUP_TOOLCHAIN=stable CARGO_TARGET_DIR="$profile/target" cargo build --release --offline -j 8 --manifest-path "$profile/Cargo.toml"
for side in before after; do
    uptime > "$profile/$side.load.log"
    docker run --rm --cpuset-cpus="$core" --user "$(id -u):$(id -g)" -v "$work:$work" -w "$work" "$image" \
        valgrind --tool=callgrind --cache-sim=yes --branch-sim=yes --instr-atstart=no \
        --callgrind-out-file="$profile/$side.callgrind" \
        "$profile/target/release/scan-attribution" "$side" "$work/inputs/json-1m.json" "$count" \
        > "$profile/$side.csv" 2> "$profile/$side.log"
    uptime >> "$profile/$side.load.log"
    python3 "$here/../attribution/summarize.py" callgrind "$profile/$side.callgrind" "$count" > "$profile/$side.tsv"
done
