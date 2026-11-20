#!/usr/bin/env bash
# run-swar-ab.sh prepare 後の専用 WORK に、計装した JSON crate を別途作る。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
work=${WORK:?専用 worktree 内の WORK を指定する}
side=${PROFILE_SIDE:-after}
[[ $side = before || $side = after ]] || exit 2
profile=$work/profile-$side
mkdir -p "$profile/src"
[[ ! -e $profile/json ]] || { echo '既存の profile/json があるため停止' >&2; exit 2; }
cp -R "$work/$side/json" "$profile/json"
sed -i 's/^name = "json-before"/name = "json-after"/' "$profile/json/Cargo.toml"
python3 "$here/profile-scans.py" "$profile/json" "$work/ir/json.ir.json"
sed -i '/^\[workspace\]$/d' "$profile/json/Cargo.toml"
cp "$here/../src/profile_scans.rs" "$profile/src/main.rs"
cat > "$profile/Cargo.toml" <<'TOML'
[workspace]
[package]
name = "scan-profile"
version = "0.0.0"
edition = "2021"
[dependencies]
json-after = { path = "json" }
TOML
RUSTUP_TOOLCHAIN=stable CARGO_TARGET_DIR="$profile/target" cargo build --release --offline -j 8 --manifest-path "$profile/Cargo.toml"
uptime > "$profile/load.log"
taskset -c "${CORE:-30}" "$profile/target/release/scan-profile" "$work/inputs/json-1m.json" > "$profile/counts.csv"
uptime >> "$profile/load.log"
