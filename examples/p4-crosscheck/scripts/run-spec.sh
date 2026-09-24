#!/usr/bin/env bash
# 仕様由来ケース集（corpus/spec/p4/*.json）を P4 実装へ流す（受理のみ）。
#
#   R1 ubnfc p4-rust      examples/p4-rust（--example spec-probe）
#   R2 tinyexpression-rs  tinyexpression/rust（`parse` CLI）
#
# J1（ubnfc Java backend）と J2（旧 Java）は 1 入力あたり JVM 起動が要るので、
# ここでは駆動しない（scripts/run.sh の 16 fixture 比較を参照）。
#
# 使い方: scripts/run-spec.sh [OUT_DIR]
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
examples=$(cd "$module/.." && pwd)
workspace=$(cd "$examples/.." && pwd)
out=${1:-$module/out-spec}
tiny=${TINYEXPRESSION_DIR:-$(cd "$workspace/../tinyexpression" && pwd)}
mkdir -p "$out"
work=$(mktemp -d -t p4-spec.XXXXXXXX)
trap 'rm -rf -- "$work"' EXIT

python3 - "$workspace/corpus/spec/p4" > "$work/cases.tsv" <<'PY'
import json, pathlib, sys
for path in sorted(pathlib.Path(sys.argv[1]).glob("*.json")):
    for case in json.loads(path.read_text(encoding="utf-8"))["cases"]:
        source = case["input"].replace("\t", "\\t").replace("\n", "\\n")
        print(f"{case['id']}\t{source}")
PY

# R1: ubnfc Rust backend の生成物。
cargo run -q --offline --release -p ubnfc-p4-rust --example spec-probe \
  < "$work/cases.tsv" > "$out/r1.tsv"

# R2: tinyexpression-rs の parse CLI（1 ケース 1 ファイル）。
: > "$out/r2.tsv"
if [ -f "$tiny/rust/Cargo.toml" ]; then
  while IFS=$'\t' read -r id source; do
    printf '%b' "${source//\\t/\\t}" > "$work/case.tiny"
    if cargo run -q --locked --offline --release \
         --manifest-path "$tiny/rust/Cargo.toml" -p tinyexpression-rs -- parse "$work/case.tiny" \
         > /dev/null 2>&1; then
      printf '%s\tok\n' "$id" >> "$out/r2.tsv"
    else
      printf '%s\treject\n' "$id" >> "$out/r2.tsv"
    fi
  done < "$work/cases.tsv"
else
  echo "tinyexpression/rust が無いので R2 は省略" >&2
fi

python3 "$module/scripts/compare-spec.py" "$workspace/corpus/spec/p4" "$out" > "$out/report.md"
echo "書き出し: $out/report.md" >&2
