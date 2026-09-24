#!/usr/bin/env bash
# ANTLR4 / tree-sitter / ubnfc(Rust,Java) を同一入力・同一ホストで交互に測る（issue #26）。
#
#   WORK=... benchmarks/scripts/run-external.sh json|arith [出力ディレクトリ]
#
# 作法:
#   - CPU は taskset -c $CORE（既定 30）で 1 コアに固定、単一 thread。
#   - **入力ごとに実装を回す**（実装ごとに全入力ではない）。ホスト負荷の傾きを
#     1 つの実装へ押しつけないため。
#   - 1 run は RUN_SECONDS（既定 0.5 秒）ぶんの反復、RUNS（既定 7）run の中央値・最小・最大。
#   - JVM は -Xms2g -Xmx2g（ANTLR / ubnfc Java 共通）。warmup 反復数は出力の warmup 列に出る
#     （既定 200 反復、最長 20 秒で打ち切り）。
#   - ubnfc Rust は ResourceLimits.arena_entries を既定 2,000,000 から
#     UBNFC_ARENA_ENTRIES（既定 200,000,000）へ上げる。1 MB の JSON を typed AST で
#     解析すると既定では "arena budget exceeded" で失敗するため。入力サイズで設定を
#     変えないよう全サイズに同じ値を使う。認識のみの経路はこの上限に触れない。
#   - 「木」の意味は実装ごとに違う。mode 列にそのまま書いてある:
#       ANTLR4      … parse tree+walk（ANTLR の parse tree を作り no-op listener で走査） / 認識のみ（setBuildParseTree(false)）
#       tree-sitter … CST構築（必ず作られる） / CST+走査（TSTreeCursor で全ノード）
#       ubnfc       … typed AST（@mapping の record 木） / 認識のみ
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?}
family=${1:?json|arith}
out=${2:-$work/results}
core=${CORE:-30}
runs=${RUNS:-7}
run_seconds=${RUN_SECONDS:-0.5}
mkdir -p "$out"
eval "$("$ubnfc/scripts/fetch-external-tools.sh" --check)"
pin=(taskset -c "$core")
jvm=(-Xms2g -Xmx2g)
csv=$out/external-$family.csv

echo "# load average (開始): $(cut -d' ' -f1-3 /proc/loadavg)" >&2
echo "impl,input,bytes,mode,iterations,warmup,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s,alloc_bytes_per_parse" > "$csv"

# 4 列目以降だけを取り出して 13 列へ正規化する（Rust 側は列構成が違う）。
emit() { grep -v '^impl,' | grep -v '^family,' | grep -v '^#' >> "$csv"; }

case "$family" in
json)
  inputs=("$work/inputs/json-1k.json" "$work/inputs/json-10k.json" "$work/inputs/json-100k.json" "$work/inputs/json-1m.json")
  grammar=json
  ;;
arith)
  inputs=("$work/inputs/operators-atoms-512.txt" "$work/inputs/power-64.txt")
  ;;
*) echo "family は json|arith" >&2; exit 2 ;;
esac

for input in "${inputs[@]}"; do
  case "$family" in
    arith)
      case "$(basename "$input")" in
        operators-*) grammar=operators ;;
        power-*)     grammar=power ;;
      esac
      ;;
  esac
  echo "# === $(basename "$input") ($grammar) ===" >&2
  # 1. ANTLR4
  for mode in recognize tree; do
    "${pin[@]}" java "${jvm[@]}" -cp "$ANTLR_JAR:$work/antlr/classes" \
      AntlrRunner bench "$grammar" "$mode" "$runs" "$run_seconds" "$input" | emit
  done
  # 2. tree-sitter
  for mode in recognize walk; do
    "${pin[@]}" "$work/ts/$grammar/runner" bench "$mode" "$runs" "$run_seconds" "$input" \
      | awk -F, 'NR>1{printf "%s,%s,%s,%s,%s,,%s,%s,%s,%s,%s,%s,\n",$1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11}' >> "$csv"
  done
  # 3. ubnfc Rust（生成）
  for mode in 認識 AST; do
    UBNFC_CMP_RUN_SECONDS=$run_seconds UBNFC_CMP_RUNS=$runs \
      UBNFC_ARENA_ENTRIES=${UBNFC_ARENA_ENTRIES:-200000000} \
      "${pin[@]}" "$work/bench-ext/target/release/ubnfc-cmp-ext" bench "$grammar" "$mode" "$input" \
      | awk -F, 'NR>1{printf "ubnfc-rust(生成),%s,%s,%s,%s,,%s,%s,%s,%s,%s,%.2f,%s\n",$3,$4,$5,$6,$7,$8,$9,$10,$11,$4/($8*1e-3)/1e6,$13}' >> "$csv"
  done
  # 4. ubnfc Java（生成）
  for mode in 認識 AST; do
    "${pin[@]}" java "${jvm[@]}" -cp "$work/ubnfc/java/classes" \
      UbnfcRunner bench "$grammar" "$mode" "$runs" "$run_seconds" "$input" | emit
  done
done
echo "# load average (終了): $(cut -d' ' -f1-3 /proc/loadavg)" >&2
column -s, -t "$csv"
