#!/usr/bin/env bash
# P4 を ANTLR4 / tree-sitter へ移植したものとの受理・速度比較（issue #27）。
# 報告は docs/reports/2026-09-24-p4-antlr-treesitter.md。
#
#   WORK=$(mktemp -d) benchmarks/scripts/p4-external.sh prepare
#   WORK=$WORK benchmarks/scripts/p4-external.sh accept    > accept.tsv
#   WORK=$WORK benchmarks/scripts/p4-external.sh negatives > negatives.tsv
#   WORK=$WORK CORE=26 RUNS=7 benchmarks/scripts/p4-external.sh speed > speed.tsv
#
# 外部ツール（ANTLR jar / tree-sitter CLI と C ランタイム）の取得は #26 の
# scripts/fetch-external-tools.sh に任せる（版と sha256 はあちらに pin。置き場は
# .gitignore 済みの benchmarks/external/）。それ以外の生成物は $WORK 配下にだけ置く。
# 生成した .g4 / grammar.js の出力（ANTLR の *.java、tree-sitter の src/parser.c）は
# commit しない。commit するのは文法 2 本と driver 2 本とこのスクリプトだけ。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?WORK を指定する}
core=${CORE:-26}
runs=${RUNS:-7}
fixtures=$ubnfc/examples/p4-rust/tests/fixtures
pin=(taskset -c "$core")
# prepare が書いた外部ツールのパス（#26 の scripts/fetch-external-tools.sh の出力）。
antlr_jar() { sed -n 's/^ANTLR_JAR=//p' "$work/external-tools.env"; }

ACCEPTED=(complex complex-x4 complex-x16 complex-x64 flat-arithmetic large-match
          valid-basic valid-multiline valid-unicode)
REJECTED=(comparison-heavy comparison-heavy-x4 comparison-heavy-x16 comparison-heavy-x64
          complex-half complex-tail invalid-syntax)
BOOLEAN=(comparison-heavy comparison-heavy-x4 comparison-heavy-x16 comparison-heavy-x64)
SPEED=(complex complex-x4 complex-x16 complex-x64)
BOOLEAN_SPEED=(comparison-heavy comparison-heavy-x64)

prepare() {
  mkdir -p "$work"
  # 1-2. 外部ツールは #26 の共有スクリプトが取る（ANTLR jar / tree-sitter CLI / tree-sitter の
  #      C ランタイム）。取得先は benchmarks/external/（.gitignore 済み）で、版と sha256 は
  #      あちらに pin されている。ここではその 3 つのパスを受け取るだけ。
  eval "$("$ubnfc/scripts/fetch-external-tools.sh")"
  : "${ANTLR_JAR:?}" "${TREE_SITTER_BIN:?}" "${TREE_SITTER_SRC:?}"
  { echo "ANTLR_JAR=$ANTLR_JAR"; echo "TREE_SITTER_BIN=$TREE_SITTER_BIN"
    echo "TREE_SITTER_SRC=$TREE_SITTER_SRC"; } > "$work/external-tools.env"
  # 3. ANTLR: 生成 + javac。
  rm -rf "$work/antlr"; mkdir -p "$work/antlr/gen" "$work/antlr/classes"
  java -jar "$ANTLR_JAR" -Dlanguage=Java -o "$work/antlr/gen" -package p4antlr \
    -no-listener -no-visitor "$ubnfc/examples/p4-antlr/TinyExpressionP4.g4"
  javac -nowarn -cp "$ANTLR_JAR" -d "$work/antlr/classes" \
    "$work"/antlr/gen/*.java "$ubnfc/examples/p4-antlr/P4AntlrDriver.java"
  # 4. tree-sitter: 入口 2 本を generate して -O2 で driver ごとリンクする。
  for entry in Formula BooleanExpression; do
    local d=$work/tsgen-$entry lang=tree_sitter_tinyexpression_p4
    [[ $entry == BooleanExpression ]] && lang=tree_sitter_tinyexpression_p4_boolean
    rm -rf "$d"; mkdir -p "$d"
    cp "$ubnfc/examples/p4-treesitter/grammar.js" "$ubnfc/examples/p4-treesitter/tree-sitter.json" "$d/"
    (cd "$d" && P4_TS_ENTRY=$entry "$TREE_SITTER_BIN" generate)
    cc -O2 -std=gnu11 -w -I "$TREE_SITTER_SRC/lib/include" -I "$TREE_SITTER_SRC/lib/src" -I "$d/src" \
      -DP4_TS_LANG=$lang "$TREE_SITTER_SRC/lib/src/lib.c" "$d/src/parser.c" \
      "$ubnfc/examples/p4-treesitter/driver.c" -o "$work/p4ts-$entry"
  done
  # 5. ubnfc Java（生成）。
  (cd "$ubnfc/examples/p4-java" && mvn -o -q -DskipTests package)
  # 6. ubnfc Rust（生成）。scope あり / scope 無効化 IR の 2 本。
  #    scope を切る ParseOptions は存在しないので、IR の semantics.scope.effects と
  #    semantics.catalogs を空にして生成し直したものを 2 本目にする。
  local ir=$ubnfc/ir/frozen/tinyexpression-p4-f86ce8a5.ir.json
  mkdir -p "$work/noscope"
  python3 - "$ir" "$work/noscope/ir.json" <<'PY_EOF'
import json, sys
d = json.load(open(sys.argv[1]))
d['semantics']['scope']['effects'] = []
d['semantics']['catalogs'] = []
json.dump(d, open(sys.argv[2], 'w'), separators=(',', ':'), sort_keys=True)
PY_EOF
  (cd "$ubnfc" && cargo run -q --offline -p ubnfc-rust --bin ubnfc-rust -- \
     --ir "$work/noscope/ir.json" --out "$work/noscope/gen" --no-emit-driver)
  rm -f "$work/noscope/gen/Cargo.toml"
  local d=$work/bench-d
  rm -rf "$d"; mkdir -p "$d/src"
  cp "$ubnfc/benchmarks/src/bench_util.rs" "$d/src/"
  cp "$ubnfc/benchmarks/src/family_d.rs" "$d/src/main.rs"
  { echo '[workspace]'; echo '[package]'; echo 'name = "bench-d"'; echo 'version = "0.0.0"'
    echo 'edition = "2021"'; echo '[dependencies]'
    echo "ubnfc-p4-rust = { path = \"$ubnfc/examples/p4-rust\" }"; } > "$d/Cargo.toml"
  (cd "$d" && cargo build --offline --release)
  cp "$d/target/release/bench-d" "$work/bench-d-scope"
  # scope 無効化 IR の生成物へ一時的に差し替えてもう 1 本作る。元に戻すのは trap。
  local keep=$work/generated-backup
  rm -rf "$keep"; cp -r "$ubnfc/examples/p4-rust/src/generated" "$keep"
  restore() { rm -rf "$ubnfc/examples/p4-rust/src/generated"; cp -r "$keep" "$ubnfc/examples/p4-rust/src/generated"; }
  trap restore EXIT
  rm -rf "$ubnfc/examples/p4-rust/src/generated"
  cp -r "$work/noscope/gen" "$ubnfc/examples/p4-rust/src/generated"
  (cd "$d" && cargo build --offline --release)
  cp "$d/target/release/bench-d" "$work/bench-d-noscope"
  restore; trap - EXIT
  echo "prepare: 完了 ($work)" >&2
}

# examples/p4-v2/scripts/compare.py の REJECTS（診断契約を突く 10 個の拒否入力）を
# fixture ファイルとして $WORK/negatives へ書き出す。空入力は名前に困るので empty とする。
negatives_dir() {
  local d=$work/negatives
  mkdir -p "$d"
  printf '%s' '1 + '            > "$d/trailing-plus.tiny"
  printf '%s' '1 * * 2'         > "$d/double-star.tiny"
  printf '%s' '(1 + 2'          > "$d/unclosed-paren.tiny"
  printf '%s' '(1 * 2'          > "$d/unclosed-paren-mul.tiny"
  printf '%s' ''                > "$d/empty.tiny"
  printf '%s' '1 + @'           > "$d/bad-atom.tiny"
  printf '%s' '$a & '           > "$d/bad-bool.tiny"
  printf '%s' '$a ^ '           > "$d/bad-xor.tiny"
  printf '%s' "'a' + "          > "$d/bad-concat.tiny"
  printf '%s' 'var $a set 1 + ; $a' > "$d/var-decl.tiny"
  echo "$d"
}
NEGATIVES=(trailing-plus double-star unclosed-paren unclosed-paren-mul empty
           bad-atom bad-bool bad-xor bad-concat var-decl)

negatives() {
  local d; d=$(negatives_dir)
  echo "# ubnfc Rust（生成）"
  "$work/bench-d-scope" accept "$d" "${NEGATIVES[@]}"
  echo "# ubnfc Java（生成）"
  java -Xss16m -cp "$ubnfc/examples/p4-java/target/classes" \
    org.unlaxer.ubnfc.examples.p4.FixtureProbe "$d" 2>/dev/null | cut -f1-3
  echo "# ANTLR 4"
  java -cp "$(antlr_jar):$work/antlr/classes" p4antlr.P4AntlrDriver accept "$d" "${NEGATIVES[@]}"
  echo "# tree-sitter"
  "$work/p4ts-Formula" accept "$d" "${NEGATIVES[@]}"
}

accept() {
  echo "# ubnfc Rust（生成）"
  "$work/bench-d-scope" accept "$fixtures" "${ACCEPTED[@]}" "${REJECTED[@]}"
  echo "# ubnfc Java（生成）"
  java -Xss16m -cp "$ubnfc/examples/p4-java/target/classes" \
    org.unlaxer.ubnfc.examples.p4.FixtureProbe "$fixtures" 2>/dev/null | cut -f1-3
  echo "# ubnfc Java（生成, BooleanExpression 入口）"
  # FixtureProbe は「入力 / 出力 / 入口」の順。出力を空文字にすると cwd へ JSON を書くので、
  # 捨てる先を $work に作って渡す。
  mkdir -p "$work/probe-out"
  java -Xss16m -cp "$ubnfc/examples/p4-java/target/classes" \
    org.unlaxer.ubnfc.examples.p4.FixtureProbe "$fixtures" "$work/probe-out" BooleanExpression 2>/dev/null | cut -f1-3
  echo "# ANTLR 4（BailErrorStrategy, SLL→LL 2 段）"
  java -cp "$(antlr_jar):$work/antlr/classes" p4antlr.P4AntlrDriver accept "$fixtures"
  echo "# tree-sitter（Formula 入口, hasError）"
  "$work/p4ts-Formula" accept "$fixtures" "${ACCEPTED[@]}" "${REJECTED[@]}"
  echo "# tree-sitter（BooleanExpression 入口, hasError）"
  "$work/p4ts-BooleanExpression" accept "$fixtures" "${BOOLEAN[@]}"
}

# ANTLR 自身の decision profiler。どの決定が LL 全文脈へ落ちているかを見る。
# profiling 自体が 1 桁ぶん高いので、決定間の**配分**だけが読める値。
profile() {
  "${pin[@]}" java -Xms2g -Xmx2g -cp "$(antlr_jar):$work/antlr/classes" \
    p4antlr.P4AntlrDriver profile "$fixtures" complex comparison-heavy:BooleanExpression
}

# 1 入力につき全実装を順に回す（同一セッション・交互実行）。
# ANTLR だけは 1 parse が桁違いに高いので、Formula 入口で測る fixture を
# ANTLR_FORMULA_FIXTURES で絞る（既定は complex のみ。理由は報告 §4）。
bench_ubnfc_rust() {  # <fixture>
  UBNFC_CMP_RUNS=$runs UBNFC_CMP_RUN_SECONDS=${RUN_SECONDS:-1.0} \
    "${pin[@]}" "$work/bench-d-scope" bench "$fixtures" "$1"
  UBNFC_CMP_RUNS=$runs UBNFC_CMP_RUN_SECONDS=${RUN_SECONDS:-1.0} \
    "${pin[@]}" "$work/bench-d-noscope" bench "$fixtures" "$1"
}
bench_ubnfc_java() {  # <fixture> [entry]
  local entry=${2:-}
  "${pin[@]}" java -Xms2g -Xmx2g -Xss16m -Dbench.warmup="${JAVA_WARMUP:-2000}" -Dbench.runs="$runs" \
    -Dbench.fixtures="$1" -Dbench.modes=recognize-fast,ast-fast \
    ${entry:+-Dbench.entry=$entry} \
    -cp "$ubnfc/examples/p4-java/target/classes" org.unlaxer.ubnfc.examples.p4.Benchmark "$fixtures" 2>/dev/null
}
bench_antlr() {  # <fixture> [entry]
  local spec=$1; [[ -n ${2:-} ]] && spec="$1:$2"
  "${pin[@]}" java -Xms2g -Xmx2g -Dp4antlr.mode="${ANTLR_MODE:-two-stage}" \
    -cp "$(antlr_jar):$work/antlr/classes" p4antlr.P4AntlrDriver \
    bench "$fixtures" "${ANTLR_RUNS:-$runs}" "${ANTLR_WARMUP:-200}" "$spec"
}
bench_treesitter() {  # <fixture> [entry]
  "${pin[@]}" "$work/p4ts-${2:-Formula}" bench "$fixtures" "$runs" "$1"
}

speed() {
  echo "# host: $(uname -srm), core=$core, runs=$runs"
  echo "# load average (開始): $(cut -d' ' -f1-3 /proc/loadavg)"
  local antlr_set=" ${ANTLR_FORMULA_FIXTURES:-complex} "
  for name in "${SPEED[@]}"; do
    echo "## $name (Formula 入口)"
    bench_ubnfc_rust "$name"
    bench_ubnfc_java "$name"
    if [[ $antlr_set == *" $name "* ]]; then bench_antlr "$name"; else
      echo "# antlr: $name は ANTLR_FORMULA_FIXTURES に無いので測らない"; fi
    bench_treesitter "$name"
    echo "# load average: $(cut -d' ' -f1-3 /proc/loadavg)"
  done
  # BooleanExpression 入口。ここには Expression / BranchExpression の型別候補が現れないので
  # ANTLR も全 fixture を測れる（報告 §4）。
  for name in "${BOOLEAN_SPEED[@]}"; do
    echo "## $name (BooleanExpression 入口)"
    bench_ubnfc_rust "$name"
    bench_ubnfc_java "$name" BooleanExpression
    bench_antlr "$name" BooleanExpression
    bench_treesitter "$name" BooleanExpression
    echo "# load average: $(cut -d' ' -f1-3 /proc/loadavg)"
  done
  echo "# load average (終了): $(cut -d' ' -f1-3 /proc/loadavg)"
}

case "${1:?prepare|accept|negatives|speed|profile}" in
  prepare)   prepare ;;
  accept)    accept ;;
  negatives) negatives ;;
  speed)     speed ;;
  profile)   profile ;;
  *) echo "prepare|accept|negatives|speed|profile" >&2; exit 2 ;;
esac
