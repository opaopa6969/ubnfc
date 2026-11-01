#!/usr/bin/env bash
# 生成物・依存・ビルド成果物は専用 WORK に閉じ込める。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
work=${WORK:?mktemp -d で作成した WORK の絶対パスを指定する}
[[ $work = /* && $work != *'"'* && $work != *'\'* ]] || exit 2
image=${IMAGE:?計測用 Docker image を先に build して IMAGE を指定する}
export RUSTUP_TOOLCHAIN=stable CARGO_TARGET_DIR="$work/target"
mkdir -p "$work/ir" "$work/inputs" "$work/bench/src" "$work/classes"
cd "$repo"
cargo build --offline --release --bin ubnfc --bin ubnfc-rust
mkdir -p "$work/java-generator"
cp -R ir/schema "$work/ir/"
tar -C ubnfc-java --exclude=target -cf - . | tar -C "$work/java-generator" -xf -
(cd "$work/java-generator" && mvn -o -q -DskipTests package)
"$work/target/release/ubnfc" ir --grammar examples/json/json.ubnf --out "$work/ir/json.ir.json"
cp ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json "$work/ir/p4.ir.json"
for grammar in p4 json; do
    "$work/target/release/ubnfc-rust" --ir "$work/ir/$grammar.ir.json" --out "$work/$grammar" --no-emit-driver
    sed -i "s/name = \"ubnfc-generated\"/name = \"gen-$grammar\"/" "$work/$grammar/Cargo.toml"
done
mkdir -p "$work/include" "$work/region"
docker run --rm "$image" tar -C /usr/include -cf - valgrind/valgrind.h valgrind/callgrind.h | tar -C "$work/include" -xf -
cc -I"$work/include" -c "$here/region.c" -o "$work/region/region.o"
ar rcs "$work/region/libattribution_region.a" "$work/region/region.o"
cat > "$work/bench/build.rs" <<'RUST'
fn main() {
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rustc-link-search=native={}", root.parent().unwrap().join("region").display());
    println!("cargo:rustc-link-lib=static=attribution_region");
}
RUST
cp "$here/driver.rs" "$work/bench/src/main.rs"
sed 's/ubnfc_generated/gen_p4/g' scanners/rust/scanners.rs > "$work/bench/src/scanners.rs"
cat > "$work/bench/Cargo.toml" <<TOML
[workspace]
[package]
name = "attribution"
version = "0.0.0"
edition = "2021"
[dependencies]
gen-p4 = { path = "$work/p4" }
gen-json = { path = "$work/json" }
[profile.release]
debug = 1
TOML
cargo build --offline --release --manifest-path "$work/bench/Cargo.toml"
java -cp "$work/java-generator/target/ubnfc-java-0.1.0-SNAPSHOT.jar" org.unlaxer.ubnfc.java.Main \
    --ir "$work/ir/json.ir.json" --out "$work/java" --package gen.json --no-emit-driver
find "$work/java" examples/p4-java/src/main/java/org/unlaxer/ubnfc/examples/p4/generated -name '*.java' > "$work/java-sources"
printf '%s\n' "$repo/examples/p4-java/src/main/java/org/unlaxer/ubnfc/examples/p4/P4Scanners.java" "$here/Attribution.java" >> "$work/java-sources"
javac -d "$work/classes" @"$work/java-sources"
# JSONTestSuite は既存比較と同じ commit。専用 WORK にだけ取得する。
if [[ ! -d $work/JSONTestSuite/.git ]]; then
    git init -q "$work/JSONTestSuite"
    git -C "$work/JSONTestSuite" remote add origin https://github.com/nst/JSONTestSuite.git
    git -C "$work/JSONTestSuite" fetch -q --depth 1 origin 1ef36fa01286573e846ac449e8683f8833c5b26a
    git -C "$work/JSONTestSuite" -c advice.detachedHead=false checkout -q --detach FETCH_HEAD
fi
[[ $(git -C "$work/JSONTestSuite" rev-parse HEAD) = 1ef36fa01286573e846ac449e8683f8833c5b26a ]]
python3 benchmarks/scripts/make-json-inputs.py "$work/JSONTestSuite/test_parsing" "$work/inputs"
cp examples/p4-rust/tests/fixtures/complex.tiny "$work/inputs/complex.tiny"
{ git rev-parse HEAD; rustc --version; java -version 2>&1; uname -a; sha256sum "$work/inputs/complex.tiny" "$work/inputs/json-100k.json" "$work/target/release/attribution"; } > "$work/versions.txt"
