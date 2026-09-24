#!/usr/bin/env bash
# P4 文法・4 実装のクロスチェック。16 fixture を同じ入力で走らせ、受理と canonical AST を比べる。
#
#   R1 ubnfc p4-rust     examples/p4-rust（ubnfc Rust backend の生成物）
#   J1 ubnfc p4-java     examples/p4-java（ubnfc Java backend の生成物）
#   R2 tinyexpression-rs tinyexpression/rust（unlaxer Rust codegen + unlaxer-runtime）
#   J2 旧 Java           tinyexpression の生成物 + unlaxer-common combinator（mapper / facade）
#
# 使い方: scripts/run.sh [OUT_DIR]
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
examples=$(cd "$module/.." && pwd)
out=${1:-$module/out}
tiny=${TINYEXPRESSION_DIR:-$(cd "$examples/../../tinyexpression" && pwd)}
mkdir -p "$out"/{r1,j1,r2,j2-mapper,j2-facade}
fixtures=$examples/p4-rust/tests/fixtures

# R1: ubnfc Rust backend。golden 生成と同じ経路（json feature の canonical_json）。
cargo test -q --offline --manifest-path "$examples/p4-rust/Cargo.toml" \
  --features json --test fixtures >/dev/null 2>&1 || true
cp "$examples/p4-rust"/tests/golden/*.json "$out/r1/" 2>/dev/null || true

# J1: ubnfc Java backend。
(cd "$examples/p4-java" && mvn -o -q -DskipTests package >/dev/null)
java -cp "$examples/p4-java/target/classes" org.unlaxer.ubnfc.examples.p4.FixtureProbe "$fixtures" "$out/j1" \
  > "$out/j1-acceptance.tsv"

# R2: tinyexpression-rs の parse CLI。
: > "$out/r2-acceptance.tsv"
for file in "$fixtures"/*.tiny; do
  name=$(basename "$file")
  if cargo run -q --locked --offline --release \
       --manifest-path "$tiny/rust/Cargo.toml" -p tinyexpression-rs -- parse "$file" \
       > "$out/r2/$name.json" 2>/dev/null; then
    printf '%s\tok\n' "$name" >> "$out/r2-acceptance.tsv"
  else
    printf '%s\treject\n' "$name" >> "$out/r2-acceptance.tsv"
  fi
done

# J2: 旧 Java（LegacyProbe を反射で使う既存スクリプト）。
for mode in mapper facade; do
  : > "$out/j2-$mode-acceptance.tsv"
  for file in "$fixtures"/*.tiny; do
    name=$(basename "$file")
    if bash "$examples/p4-java/scripts/legacy.sh" "$tiny" "$mode" "$file" \
         "$out/j2-$mode/$name.json" >/dev/null 2>&1; then
      printf '%s\tok\n' "$name" >> "$out/j2-$mode-acceptance.tsv"
    else
      printf '%s\treject\n' "$name" >> "$out/j2-$mode-acceptance.tsv"
    fi
  done
done

python3 "$module/scripts/compare.py" "$out" "$fixtures" > "$out/report.md"
echo "書き出し: $out/report.md" >&2
