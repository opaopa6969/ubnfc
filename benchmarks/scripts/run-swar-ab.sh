#!/usr/bin/env bash
# issue #3（SWAR）の A/B。基準 revision の生成器（before）と作業ツリーの生成器
# （after）で P4 / JSON / Operators の parser を生成し、1 つの計測 bin に並べて測る。
#
#   WORK=$(mktemp -d) BASE=origin/master benchmarks/scripts/run-swar-ab.sh prepare
#   WORK=$WORK CORE=30 RUNS=7 benchmarks/scripts/run-swar-ab.sh time [出力ディレクトリ]
#
# - IR は before / after で同じファイルを使う（生成器だけを比べる）。
# - JSON 入力は JSONTestSuite から make-json-inputs.py で作る（sha256 は
#   examples/json/evidence/inputs.sha256 と一致する）。JSON_TEST_SUITE で場所を変えられる。
# - 作法は benchmarks/README.md と同じ（taskset で 1 コア、交互実行、中央値と最小値、load を併記）。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?WORK を指定する}
base=${BASE:-origin/master}
core=${CORE:-30}
runs=${RUNS:-7}
run_seconds=${RUN_SECONDS:-0.5}
suite=${JSON_TEST_SUITE:-$ubnfc/benchmarks/external/JSONTestSuite}

prepare() {
  mkdir -p "$work"
  # 1. 生成器 2 つ。before は BASE の git archive を別 target で build する（作業ツリーは触らない）。
  rm -rf "$work/base" && mkdir -p "$work/base"
  (cd "$ubnfc" && git archive "$base" crates Cargo.toml Cargo.lock examples/p4-rust \
     examples/handwritten-target scanners ir) | tar -xf - -C "$work/base"
  (cd "$work/base" && CARGO_TARGET_DIR="$work/target-base" cargo build -q --offline --release -p ubnfc-rust --bin ubnfc-rust)
  (cd "$ubnfc" && CARGO_TARGET_DIR="$work/target-after" cargo build -q --offline --release --bin ubnfc-rust --bin ubnfc)
  git -C "$ubnfc" rev-parse "$base" > "$work/base.rev"
  git -C "$ubnfc" rev-parse HEAD > "$work/after.rev"
  git -C "$ubnfc" diff -- crates/ubnfc-rust > "$work/generator.diff"
  sha256sum "$ubnfc/crates/ubnfc-rust/src/scan.rs" "$ubnfc/benchmarks/src/family_swar.rs" > "$work/sources.sha256"
  # 2. IR（共通）と生成物。
  mkdir -p "$work/ir"
  "$work/target-after/release/ubnfc" ir --grammar "$ubnfc/examples/json/json.ubnf" --out "$work/ir/json.ir.json"
  cp "$ubnfc/ir/frozen/tinyexpression-p4-f86ce8a5.ir.json" "$work/ir/p4.ir.json"
  cp "$ubnfc/ir/fixtures/unlaxer-dsl__src__test__resources__associative__Operators.ubnf.ir.json" "$work/ir/ops.ir.json"
  # 後から追加された LL(1) 表は両側から外し、bulk scan 内部だけを比べる。
  python3 - "$work/ir" <<'PY'
import json
import pathlib
import sys
for path in pathlib.Path(sys.argv[1]).glob('*.ir.json'):
    ir = json.loads(path.read_text())
    if 'll1' in ir['plan']:
        del ir['plan']['ll1']
        path.write_text(json.dumps(ir, ensure_ascii=False, indent=2) + '\n')
PY
  for side in before after; do
    local gen=$work/target-$([ "$side" = before ] && echo base || echo after)/release/ubnfc-rust
    rm -rf "$work/$side" && mkdir -p "$work/$side"
    for name in p4 json ops; do
      "$gen" --ir "$work/ir/$name.ir.json" --out "$work/$side/$name" --no-emit-driver
      sed -i "s/^name = \"ubnfc-generated\"/name = \"$name-$side\"/" "$work/$side/$name/Cargo.toml"
    done
  done
  # 3. 計測 bin（ubnfc の cargo workspace には入れない）。P4 の Extern scanner は crate 名を差し替えて 2 つ。
  rm -rf "$work/bench" && mkdir -p "$work/bench/src"
  cp "$ubnfc/benchmarks/src/bench_util.rs" "$work/bench/src/"
  cp "$ubnfc/benchmarks/src/family_swar.rs" "$work/bench/src/main.rs"
  sed 's/ubnfc_generated/p4_before/g' "$ubnfc/scanners/rust/scanners.rs" > "$work/bench/src/sc_before.rs"
  sed 's/ubnfc_generated/p4_after/g' "$ubnfc/scanners/rust/scanners.rs" > "$work/bench/src/sc_after.rs"
  { echo '[workspace]'; echo '[package]'; echo 'name = "family-swar"'; echo 'version = "0.0.0"'
    echo 'edition = "2021"'; echo '[dependencies]'
    for side in before after; do for name in p4 json ops; do
      echo "$name-$side = { path = \"$work/$side/$name\" }"
    done; done; } > "$work/bench/Cargo.toml"
  (cd "$work/bench" && cargo build -q --offline --release)
  # 4. 入力。
  mkdir -p "$work/inputs"
  python3 "$ubnfc/benchmarks/scripts/make-json-inputs.py" "$suite/test_parsing" "$work/inputs" >/dev/null
  python3 -c "open('$work/inputs/operators-atoms-512.txt','w').write('+'.join(['1']*512))"
  echo "# prepare: 完了 ($work, before=$(cat "$work/base.rev"), after=$(cat "$work/after.rev"))" >&2
}

inputs() {
  local fixtures=$ubnfc/examples/p4-rust/tests/fixtures
  echo "p4 $fixtures/complex.tiny $fixtures/complex-x4.tiny $fixtures/complex-x16.tiny $fixtures/complex-x64.tiny $fixtures/flat-arithmetic.tiny $fixtures/large-match.tiny"
  echo "json $work/inputs/json-1k.json $work/inputs/json-10k.json $work/inputs/json-100k.json $work/inputs/json-1m.json"
  echo "ops $work/inputs/operators-atoms-512.txt"
}

measure() {
  local what=$1 out=${2:-$work/results}
  mkdir -p "$out"
  local csv=$out/swar-$what.csv
  echo "# host: $(uname -srm), core=$core, runs=$runs, run_seconds=$run_seconds" >&2
  echo "# before=$(cat "$work/base.rev") after=$(cat "$work/after.rev")" >&2
  uptime >&2
  echo "# load average (開始): $(cut -d' ' -f1-3 /proc/loadavg)" >&2
  : > "$csv"
  while read -r grammar files; do
    # shellcheck disable=SC2086
    AB_RUNS=$runs AB_RUN_SECONDS=$run_seconds taskset -c "$core" \
      "$work/bench/target/release/family-swar" "$what" "$grammar" $files \
      | { if [ -s "$csv" ]; then tail -n +2; else cat; fi; } >> "$csv"
  done < <(inputs)
  uptime >&2
  echo "# load average (終了): $(cut -d' ' -f1-3 /proc/loadavg)" >&2
  echo "# $csv" >&2
}

case "${1:-}" in
  prepare) prepare ;;
  time) measure time "${2:-}" ;;
  *) echo "usage: run-swar-ab.sh prepare|time [出力ディレクトリ]" >&2; exit 2 ;;
esac
