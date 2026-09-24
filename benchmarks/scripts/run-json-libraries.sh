#!/usr/bin/env bash
# JSON 専用ライブラリ（Jackson / Gson / serde_json）と ubnfc・ANTLR4・tree-sitter を
# 同一入力・同一ホスト・同一セッションで交互に測る（issue #68）。
#
#   WORK=$(mktemp -d) benchmarks/scripts/prepare-external.sh          # ANTLR / tree-sitter / ubnfc
#   WORK=$WORK benchmarks/scripts/run-json-libraries.sh prepare       # Jackson / Gson / serde_json
#   WORK=$WORK benchmarks/scripts/run-json-libraries.sh bench [出力ディレクトリ]
#   WORK=$WORK benchmarks/scripts/run-json-libraries.sh check [出力ディレクトリ]
#   WORK=$WORK benchmarks/scripts/run-json-libraries.sh check-features [出力ディレクトリ]
#
# 作法（#26 の run-external.sh と同じ）:
#   - CPU は taskset -c $CORE（既定 30）で 1 コアに固定、単一 thread。
#   - **入力ごとに実装を回す**（実装ごとに全入力ではない）。ホスト負荷の傾きを
#     1 つの実装へ押しつけないため。
#   - 1 run は RUN_SECONDS（既定 0.5 秒）ぶんの反復、RUNS（既定 7）run の中央値・最小・最大。
#   - JVM は -Xms2g -Xmx2g（ANTLR / ubnfc Java / Jackson / Gson 共通、JDK 21）。
#     JIT 暖機は出力の warmup 列に出る（Jackson/Gson は 500 反復以上かつ 5 秒以上、
#     ANTLR / ubnfc Java は 500 反復・最長 30 秒。JMH は使っていない。理由は
#     benchmarks/java/JsonLibRunner.java の冒頭）。
#   - 「何を作るか」は mode 列にそのまま書いてある:
#       Jackson     … streaming(何も作らない) / readTree(JsonNode 木・span なし) / readValue(Map)
#       Gson        … parseString(JsonElement 木・span なし) / fromJson(Map)
#       serde_json  … IgnoredAny(何も作らない) / serde_json::Value(木・span なし)
#       ANTLR4      … 認識のみ(setBuildParseTree(false)) / parse tree+walk
#       tree-sitter … CST 構築（必ず作られる） / CST+走査
#       ubnfc       … 認識のみ / typed AST（span・診断つき）
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?WORK を指定する}
core=${CORE:-30}
runs=${RUNS:-7}
run_seconds=${RUN_SECONDS:-0.5}
java_warmup=${JAVA_WARMUP:-500}
pin=(taskset -c "$core")
jvm=(-Xms2g -Xmx2g)

prepare() {
  eval "$("$ubnfc/scripts/fetch-json-libraries.sh")"
  { echo "JACKSON_CP=$JACKSON_CP"; echo "GSON_CP=$GSON_CP"; echo "JSONLIB_CP=$JSONLIB_CP"
    echo "JACKSON_VERSION=$JACKSON_VERSION"
    echo "JACKSON_ANNOTATIONS_VERSION=$JACKSON_ANNOTATIONS_VERSION"
    echo "GSON_VERSION=$GSON_VERSION"; } > "$work/json-libraries.env"
  # 1. Jackson / Gson の driver。
  rm -rf "$work/jsonlib/classes"; mkdir -p "$work/jsonlib/classes"
  javac -nowarn -cp "$JSONLIB_CP" -d "$work/jsonlib/classes" "$ubnfc/benchmarks/java/JsonLibRunner.java"
  # 2. serde_json の driver（ubnfc の cargo workspace には入れない）。
  #    crates.io は遮断されているので --offline（~/.cargo/registry のものを使う）。
  rm -rf "$work/bench-jsonlib"; mkdir -p "$work/bench-jsonlib/src"
  cp "$ubnfc/benchmarks/src/bench_util.rs" "$work/bench-jsonlib/src/"
  cp "$ubnfc/benchmarks/src/family_jsonlib.rs" "$work/bench-jsonlib/src/main.rs"
  { echo '[workspace]'; echo '[package]'; echo 'name = "jsonlib-bench"'
    echo 'version = "0.0.0"'; echo 'edition = "2021"'; echo '[dependencies]'
    echo 'serde = "1"'; echo 'serde_json = "1"'; } > "$work/bench-jsonlib/Cargo.toml"
  (cd "$work/bench-jsonlib" && cargo build --offline --release)
  (cd "$work/bench-jsonlib" && grep -A1 '^name = "serde' Cargo.lock) > "$work/serde-json-versions.txt"
  echo "# prepare: 完了 ($work)" >&2
}

bench() {
  local out=${1:-$work/results-jsonlib}
  mkdir -p "$out"
  eval "$(cat "$work/json-libraries.env")"
  eval "$("$ubnfc/scripts/fetch-external-tools.sh" --check)"
  local csv=$out/json-libraries.csv
  echo "# host: $(uname -srm), core=$core, runs=$runs, run_seconds=$run_seconds" >&2
  echo "# load average (開始): $(cut -d' ' -f1-3 /proc/loadavg)" >&2
  echo "impl,input,bytes,mode,iterations,warmup,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s,alloc_bytes_per_parse" > "$csv"
  emit() { grep -v '^impl,' | grep -v '^family,' | grep -v '^#' >> "$csv"; }
  local inputs=("$work/inputs/json-1k.json" "$work/inputs/json-10k.json"
                "$work/inputs/json-100k.json" "$work/inputs/json-1m.json")
  for input in "${inputs[@]}"; do
    echo "# === $(basename "$input") ===" >&2
    # 1. Jackson（3 モード）
    for mode in jackson-stream jackson-tree jackson-map; do
      "${pin[@]}" java "${jvm[@]}" -cp "$JSONLIB_CP:$work/jsonlib/classes" \
        JsonLibRunner bench "$mode" "$runs" "$run_seconds" "$input" | emit
    done
    # 2. Gson（2 モード）
    for mode in gson-element gson-map; do
      "${pin[@]}" java "${jvm[@]}" -cp "$JSONLIB_CP:$work/jsonlib/classes" \
        JsonLibRunner bench "$mode" "$runs" "$run_seconds" "$input" | emit
    done
    # 3. serde_json（2 モード）
    for mode in ignored value; do
      UBNFC_CMP_RUN_SECONDS=$run_seconds UBNFC_CMP_RUNS=$runs \
        "${pin[@]}" "$work/bench-jsonlib/target/release/jsonlib-bench" bench "$mode" "$input" \
        | awk -F, 'NR>1{printf "serde_json,%s,%s,%s,%s,,%s,%s,%s,%s,%s,%.2f,%s\n",$3,$4,$5,$6,$7,$8,$9,$10,$11,$4/($8*1e-3)/1e6,$13}' >> "$csv"
    done
    # 4. ANTLR4（既存 driver。同じ走行で並べ直す）
    for mode in recognize tree; do
      "${pin[@]}" java "${jvm[@]}" -Dbench.warmupIterations="$java_warmup" -Dbench.warmupSeconds=30 \
        -cp "$ANTLR_JAR:$work/antlr/classes" \
        AntlrRunner bench json "$mode" "$runs" "$run_seconds" "$input" | emit
    done
    # 5. tree-sitter（既存 driver）
    for mode in recognize walk; do
      "${pin[@]}" "$work/ts/json/runner" bench "$mode" "$runs" "$run_seconds" "$input" \
        | awk -F, 'NR>1{printf "%s,%s,%s,%s,%s,,%s,%s,%s,%s,%s,%s,\n",$1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11}' >> "$csv"
    done
    # 6. ubnfc Rust（生成）
    for mode in 認識 AST; do
      UBNFC_CMP_RUN_SECONDS=$run_seconds UBNFC_CMP_RUNS=$runs \
        UBNFC_ARENA_ENTRIES=${UBNFC_ARENA_ENTRIES:-200000000} \
        "${pin[@]}" "$work/bench-ext/target/release/ubnfc-cmp-ext" bench json "$mode" "$input" \
        | awk -F, 'NR>1{printf "ubnfc-rust(生成),%s,%s,%s,%s,,%s,%s,%s,%s,%s,%.2f,%s\n",$3,$4,$5,$6,$7,$8,$9,$10,$11,$4/($8*1e-3)/1e6,$13}' >> "$csv"
    done
    # 7. ubnfc Java（生成）
    for mode in 認識 AST; do
      "${pin[@]}" java "${jvm[@]}" -Dbench.warmupIterations="$java_warmup" -Dbench.warmupSeconds=30 \
        -cp "$work/ubnfc/java/classes" \
        UbnfcRunner bench json "$mode" "$runs" "$run_seconds" "$input" | emit
    done
    echo "# load average: $(cut -d' ' -f1-3 /proc/loadavg)" >&2
  done
  echo "# load average (終了): $(cut -d' ' -f1-3 /proc/loadavg)" >&2
  column -s, -t "$csv"
}

# JSONTestSuite 318 件を Jackson 3 変種 / Gson 2 変種 / serde_json へ流す。
# ubnfc / ANTLR / tree-sitter の判定は benchmarks/scripts/check-jsontestsuite.sh が出す
# （同じ $WORK / 同じ suite commit を使うので、そのまま横に並べられる）。
check() {
  local out=${1:-$work/check-jsonlib}
  mkdir -p "$out"
  eval "$(cat "$work/json-libraries.env")"
  eval "$("$ubnfc/scripts/fetch-external-tools.sh" --check)"
  local suite=$JSON_TEST_SUITE/test_parsing
  local files=("$suite"/*.json)
  echo "# 対象 ${#files[@]} 件" >&2
  for variant in jackson-default jackson-strict jackson-lenient gson-default gson-strict; do
    java -Xss16m -cp "$JSONLIB_CP:$work/jsonlib/classes" \
      JsonLibRunner check "$variant" "${files[@]}" > "$out/$variant.tsv"
  done
  "$work/bench-jsonlib/target/release/jsonlib-bench" check "${files[@]}" > "$out/serde_json.tsv"
  echo "# 書き出し: $out" >&2
}

# Jackson の寛容 feature を 1 つずつ既定へ足し、受理される n_ を feature 別に出す（帰属の切り分け）。
check_features() {
  local out=${1:-$work/check-jsonlib}
  mkdir -p "$out"
  eval "$(cat "$work/json-libraries.env")"
  eval "$("$ubnfc/scripts/fetch-external-tools.sh" --check)"
  local suite=$JSON_TEST_SUITE/test_parsing f
  for f in ALLOW_JAVA_COMMENTS ALLOW_YAML_COMMENTS ALLOW_BACKSLASH_ESCAPING_ANY_CHARACTER \
           ALLOW_SINGLE_QUOTES ALLOW_UNESCAPED_CONTROL_CHARS ALLOW_UNQUOTED_FIELD_NAMES \
           ALLOW_RS_CONTROL_CHAR ALLOW_LEADING_DECIMAL_POINT_FOR_NUMBERS \
           ALLOW_LEADING_PLUS_SIGN_FOR_NUMBERS ALLOW_LEADING_ZEROS_FOR_NUMBERS \
           ALLOW_NON_NUMERIC_NUMBERS ALLOW_TRAILING_DECIMAL_POINT_FOR_NUMBERS \
           ALLOW_MISSING_VALUES ALLOW_TRAILING_COMMA; do
    java -cp "$JSONLIB_CP:$work/jsonlib/classes" JsonLibRunner check "jackson+$f" "$suite"/n_*.json \
      | awk -F'\t' -v f="$f" '$2=="accept"{n=$1; sub(/.*\//,"",n); print f"\t"n}'
  done > "$out/jackson-features.tsv"
  echo "# 書き出し: $out/jackson-features.tsv" >&2
}

case "${1:?prepare|bench|check|check-features}" in
  prepare) prepare ;;
  bench)   bench "${2:-}" ;;
  check)   check "${2:-}" ;;
  check-features) check_features "${2:-}" ;;
  *) echo "prepare|bench|check|check-features" >&2; exit 2 ;;
esac
