#!/usr/bin/env bash
# ANTLR 流（塔を 1 本の expression に統合した）文法との比較（issue #39）。
# 1:1 移植との差だけを見るため、ubnfc / tree-sitter / ANTLR 1:1 の生成物は
# benchmarks/scripts/p4-external.sh prepare が作ったものを同じ $WORK から借りる。
#
#   WORK=$(mktemp -d) benchmarks/scripts/p4-external.sh          prepare
#   WORK=$WORK        benchmarks/scripts/p4-antlr-idiomatic.sh   prepare
#   WORK=$WORK        benchmarks/scripts/p4-antlr-idiomatic.sh   accept    > accept.tsv
#   WORK=$WORK        benchmarks/scripts/p4-antlr-idiomatic.sh   negatives > negatives.tsv
#   WORK=$WORK        benchmarks/scripts/p4-antlr-idiomatic.sh   superset  > superset.tsv
#   WORK=$WORK        benchmarks/scripts/p4-antlr-idiomatic.sh   profile   > profile.tsv
#   WORK=$WORK CORE=26 RUNS=7 benchmarks/scripts/p4-antlr-idiomatic.sh speed > speed.tsv
#
# 生成物（ANTLR の *.java、tree-sitter の src/parser.c）は commit しない。$WORK 配下に作る。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?WORK を指定する}
core=${CORE:-26}
runs=${RUNS:-7}
fixtures=$ubnfc/examples/p4-rust/tests/fixtures
pin=(taskset -c "$core")
antlr_jar() { sed -n 's/^ANTLR_JAR=//p' "$work/external-tools.env"; }
ts_bin()    { sed -n 's/^TREE_SITTER_BIN=//p' "$work/external-tools.env"; }
ts_src()    { sed -n 's/^TREE_SITTER_SRC=//p' "$work/external-tools.env"; }

ACCEPTED=(complex complex-x4 complex-x16 complex-x64 flat-arithmetic large-match
          valid-basic valid-multiline valid-unicode)
REJECTED=(comparison-heavy comparison-heavy-x4 comparison-heavy-x16 comparison-heavy-x64
          complex-half complex-tail invalid-syntax)
BOOLEAN=(comparison-heavy comparison-heavy-x4 comparison-heavy-x16 comparison-heavy-x64)
SPEED=(complex complex-x4 complex-x16 complex-x64)
BOOLEAN_SPEED=(comparison-heavy comparison-heavy-x64)
NEGATIVES=(trailing-plus double-star unclosed-paren unclosed-paren-mul empty
           bad-atom bad-bool bad-xor bad-concat var-decl)
# P4 が構文で拒否し、塔を統合した文法は受理する入力（型の話になったもの）。
SUPERSET=(num-plus-bool str-times-num chained-compare bare-ternary pow-arity
          min-zero-args mixed-match dot-on-number slice-trailing-colon not-number
          decl-type-mismatch tonum-number-arg startswith-number bool-times-bool)

prepare() {
  [[ -f $work/external-tools.env ]] || { echo "先に p4-external.sh prepare を同じ WORK で" >&2; exit 2; }
  local jar; jar=$(antlr_jar)
  # ANTLR 流の文法。
  rm -rf "$work/antlr-idiom"; mkdir -p "$work/antlr-idiom/gen" "$work/antlr-idiom/classes"
  java -jar "$jar" -Dlanguage=Java -o "$work/antlr-idiom/gen" -package p4antlr \
    -no-listener -no-visitor "$ubnfc/examples/p4-antlr/TinyExpressionP4Idiomatic.g4"
  javac -nowarn -cp "$jar" -d "$work/antlr-idiom/classes" \
    "$work"/antlr-idiom/gen/*.java "$ubnfc/examples/p4-antlr/P4AntlrIdiomaticDriver.java"
  # tree-sitter 流の文法。入口 2 本。
  local tsb tss; tsb=$(ts_bin); tss=$(ts_src)
  for entry in Formula Expression; do
    local d=$work/tsgen-Idiom-$entry lang=tree_sitter_tinyexpression_p4_idiomatic
    [[ $entry == Expression ]] && lang=tree_sitter_tinyexpression_p4_idiomatic_expr
    rm -rf "$d"; mkdir -p "$d"
    cp "$ubnfc/examples/p4-treesitter/grammar.idiomatic.js" "$d/grammar.js"
    cp "$ubnfc/examples/p4-treesitter/tree-sitter.json" "$d/"
    (cd "$d" && P4_TS_ENTRY=$entry "$tsb" generate)
    grep -E '^#define (STATE_COUNT|SYMBOL_COUNT)' "$d/src/parser.c" | sed "s/^/# $entry /" >&2
    cc -O2 -std=gnu11 -w -I "$tss/lib/include" -I "$tss/lib/src" -I "$d/src" \
      -DP4_TS_LANG=$lang "$tss/lib/src/lib.c" "$d/src/parser.c" \
      "$ubnfc/examples/p4-treesitter/driver.c" -o "$work/p4ts-idiom-$entry"
  done
  echo "prepare(idiomatic): 完了 ($work)" >&2
}

# P4 が拒否し ANTLR 流が受理する入力。README の「上位集合」表と 1:1 対応。
superset_dir() {
  local d=$work/superset
  mkdir -p "$d"
  printf '%s' '1 + true'                              > "$d/num-plus-bool.tiny"
  printf '%s' "'a' * 2"                               > "$d/str-times-num.tiny"
  printf '%s' '1 == 2 == 3'                           > "$d/chained-compare.tiny"
  printf '%s' '$c ? 1 : 2'                            > "$d/bare-ternary.tiny"
  printf '%s' 'pow(1)'                                > "$d/pow-arity.tiny"
  printf '%s' 'min()'                                 > "$d/min-zero-args.tiny"
  printf '%s' 'match { $a > 1 -> 1, default -> "x" }' > "$d/mixed-match.tiny"
  printf '%s' '(1 + 2).trim()'                        > "$d/dot-on-number.tiny"
  printf '%s' '$a[0:1:]'                              > "$d/slice-trailing-colon.tiny"
  printf '%s' 'not(1)'                                > "$d/not-number.tiny"
  printf '%s' 'var $a as number set "text"; $a'       > "$d/decl-type-mismatch.tiny"
  printf '%s' 'toNum(1, 2)'                           > "$d/tonum-number-arg.tiny"
  printf '%s' '$a.startsWith(1)'                      > "$d/startswith-number.tiny"
  printf '%s' 'true & 1 * 2'                          > "$d/bool-times-bool.tiny"
  echo "$d"
}

antlr_idiom() { java -cp "$(antlr_jar):$work/antlr-idiom/classes" p4antlr.P4AntlrIdiomaticDriver "$@"; }
antlr_1to1()  { java -cp "$(antlr_jar):$work/antlr/classes"       p4antlr.P4AntlrDriver         "$@"; }

accept() {
  echo "# ANTLR 流（BailErrorStrategy, SLL→LL 2 段）"
  antlr_idiom accept "$fixtures"
  echo "# ANTLR 流（SLL のみ）"
  P4_MODE=sll java -Dp4antlr.mode=sll -cp "$(antlr_jar):$work/antlr-idiom/classes" \
    p4antlr.P4AntlrIdiomaticDriver accept "$fixtures"
  echo "# tree-sitter 流（Formula 入口, hasError）"
  "$work/p4ts-idiom-Formula" accept "$fixtures" "${ACCEPTED[@]}" "${REJECTED[@]}"
  echo "# tree-sitter 流（Expression 入口, hasError）"
  "$work/p4ts-idiom-Expression" accept "$fixtures" "${BOOLEAN[@]}"
}

negatives() {
  local d=$work/negatives
  [[ -d $d ]] || { echo "先に p4-external.sh negatives を同じ WORK で" >&2; exit 2; }
  echo "# ANTLR 流"
  antlr_idiom accept "$d" "${NEGATIVES[@]}"
  echo "# tree-sitter 流"
  "$work/p4ts-idiom-Formula" accept "$d" "${NEGATIVES[@]}"
}

superset() {
  local d; d=$(superset_dir)
  echo "# ANTLR 1:1 移植（すべて拒否になるはず）"
  antlr_1to1 accept "$d" "${SUPERSET[@]}"
  echo "# ANTLR 流（すべて受理になるはず）"
  antlr_idiom accept "$d" "${SUPERSET[@]}"
  echo "# tree-sitter 1:1 移植"
  "$work/p4ts-Formula" accept "$d" "${SUPERSET[@]}"
  echo "# tree-sitter 流"
  "$work/p4ts-idiom-Formula" accept "$d" "${SUPERSET[@]}"
}

profile() {
  "${pin[@]}" java -Xms2g -Xmx2g -cp "$(antlr_jar):$work/antlr-idiom/classes" \
    p4antlr.P4AntlrIdiomaticDriver profile "$fixtures" \
    complex complex-x4 complex-x16 complex-x64 flat-arithmetic large-match \
    valid-basic valid-multiline valid-unicode \
    comparison-heavy comparison-heavy:Expression comparison-heavy-x64
}

bench_ubnfc_rust() {  # <fixture>
  UBNFC_CMP_RUNS=$runs UBNFC_CMP_RUN_SECONDS=${RUN_SECONDS:-1.0} \
    "${pin[@]}" "$work/bench-d-scope" bench "$fixtures" "$1"
}
bench_ubnfc_java() {  # <fixture> [entry]
  local entry=${2:-}
  "${pin[@]}" java -Xms2g -Xmx2g -Xss16m -Dbench.warmup="${JAVA_WARMUP:-2000}" -Dbench.runs="$runs" \
    -Dbench.fixtures="$1" -Dbench.modes=recognize-fast,ast-fast \
    ${entry:+-Dbench.entry=$entry} \
    -cp "$ubnfc/examples/p4-java/target/classes" org.unlaxer.ubnfc.examples.p4.Benchmark "$fixtures" 2>/dev/null
}
bench_antlr_1to1() {  # <fixture> [entry]
  local spec=$1; [[ -n ${2:-} ]] && spec="$1:$2"
  "${pin[@]}" java -Xms2g -Xmx2g -Dp4antlr.mode="${ANTLR_MODE:-two-stage}" \
    -cp "$(antlr_jar):$work/antlr/classes" p4antlr.P4AntlrDriver \
    bench "$fixtures" "${ANTLR_RUNS:-$runs}" "${ANTLR_WARMUP:-200}" "$spec"
}
bench_antlr_idiom() {  # <mode> <fixture> [entry]
  local spec=$2; [[ -n ${3:-} ]] && spec="$2:$3"
  "${pin[@]}" java -Xms2g -Xmx2g -Dp4antlr.mode="$1" \
    -cp "$(antlr_jar):$work/antlr-idiom/classes" p4antlr.P4AntlrIdiomaticDriver \
    bench "$fixtures" "$runs" "${ANTLR_IDIOM_WARMUP:-2000}" "$spec"
}
bench_treesitter() {  # <fixture> [entry]
  "${pin[@]}" "$work/p4ts-${2:-Formula}" bench "$fixtures" "$runs" "$1"
}
bench_treesitter_idiom() {  # <fixture> [entry]
  "${pin[@]}" "$work/p4ts-idiom-${2:-Formula}" bench "$fixtures" "$runs" "$1"
}

# 1 入力につき全実装を順に回す（#27 と同じ規約）。
speed() {
  echo "# host: $(uname -srm), core=$core, runs=$runs"
  echo "# load average (開始): $(cut -d' ' -f1-3 /proc/loadavg)"
  local antlr_set=" ${ANTLR_FORMULA_FIXTURES:-complex} "
  for name in "${SPEED[@]}"; do
    echo "## $name (Formula 入口)"
    bench_ubnfc_rust "$name"
    bench_ubnfc_java "$name"
    if [[ $antlr_set == *" $name "* ]]; then bench_antlr_1to1 "$name"; else
      echo "# antlr(1:1): $name は ANTLR_FORMULA_FIXTURES に無いので測らない"; fi
    bench_antlr_idiom two-stage "$name"
    bench_antlr_idiom sll "$name"
    bench_treesitter "$name"
    bench_treesitter_idiom "$name"
    echo "# load average: $(cut -d' ' -f1-3 /proc/loadavg)"
  done
  for name in "${BOOLEAN_SPEED[@]}"; do
    echo "## $name (BooleanExpression / Expression 入口)"
    bench_ubnfc_rust "$name"
    bench_ubnfc_java "$name" BooleanExpression
    bench_antlr_1to1 "$name" BooleanExpression
    bench_antlr_idiom two-stage "$name" Expression
    bench_antlr_idiom sll "$name" Expression
    bench_treesitter "$name" BooleanExpression
    bench_treesitter_idiom "$name" Expression
    echo "# load average: $(cut -d' ' -f1-3 /proc/loadavg)"
  done
  echo "# load average (終了): $(cut -d' ' -f1-3 /proc/loadavg)"
}

case "${1:?prepare|accept|negatives|superset|profile|speed}" in
  prepare)   prepare ;;
  accept)    accept ;;
  negatives) negatives ;;
  superset)  superset ;;
  profile)   profile ;;
  speed)     speed ;;
  *) echo "prepare|accept|negatives|superset|profile|speed" >&2; exit 2 ;;
esac
