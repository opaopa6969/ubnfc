#!/usr/bin/env bash
# ANTLR4 / tree-sitter / ubnfc の 3 系統を、同じ 3 文法（JSON / Operators / Power）で
# 建てるところまでをまとめる（issue #26）。兄弟リポジトリは読むだけで変更しない。
#
#   WORK=$(mktemp -d) benchmarks/scripts/prepare-external.sh
#
# 作るもの（すべて $WORK 配下）:
#   $WORK/antlr/classes      ANTLR 生成 parser + AntlrRunner
#   $WORK/ts/<g>/runner      tree-sitter 生成 parser + C ランタイム + ts_runner
#   $WORK/ubnfc/rust/...     ubnfc Rust backend の生成 crate と計測 bin
#   $WORK/ubnfc/java/classes ubnfc Java backend の生成物 + UbnfcRunner
#   $WORK/inputs/            速度比較の入力（sha256 付き）
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?WORK を指定する}
mkdir -p "$work"

# 0. 外部ツール（未取得なら scripts/fetch-external-tools.sh が何が無いかを述べて落ちる）。
eval "$("$ubnfc/scripts/fetch-external-tools.sh" --check)"
echo "# ANTLR_JAR=$ANTLR_JAR" >&2
echo "# TREE_SITTER_BIN=$TREE_SITTER_BIN ($("$TREE_SITTER_BIN" --version))" >&2
echo "# JSON_TEST_SUITE=$JSON_TEST_SUITE @ $(git -C "$JSON_TEST_SUITE" rev-parse --short HEAD)" >&2

# 1. ANTLR: 3 文法を生成して 1 つの classpath へ。
rm -rf "$work/antlr"; mkdir -p "$work/antlr/src/gen" "$work/antlr/classes"
(cd "$ubnfc/examples/json/antlr" && java -jar "$ANTLR_JAR" -Dlanguage=Java -package gen.json \
   -no-visitor -no-listener -o "$work/antlr/src/gen/json" JSON.g4)
(cd "$ubnfc/examples/arith/antlr" \
  && java -jar "$ANTLR_JAR" -Dlanguage=Java -package gen.operators -no-visitor -no-listener \
       -o "$work/antlr/src/gen/operators" Operators.g4 \
  && java -jar "$ANTLR_JAR" -Dlanguage=Java -package gen.power -no-visitor -no-listener \
       -o "$work/antlr/src/gen/power" Power.g4)
javac -nowarn -cp "$ANTLR_JAR" -d "$work/antlr/classes" \
  $(find "$work/antlr/src" -name '*.java') "$ubnfc/benchmarks/java/AntlrRunner.java"

# 2. tree-sitter: 文法ごとに parser.c を生成し、C ランタイム（-O2）と ts_runner を静的に繋ぐ。
rm -rf "$work/ts"; mkdir -p "$work/ts"
build_ts() { # <name> <grammar.js のあるディレクトリ>
  local name=$1 dir=$2
  mkdir -p "$work/ts/$name"
  (cd "$dir" && "$TREE_SITTER_BIN" generate -o "$work/ts/$name/src" grammar.js)
  gcc -O2 -std=gnu11 \
    -I"$TREE_SITTER_SRC/lib/include" -I"$TREE_SITTER_SRC/lib/src" -I"$work/ts/$name/src" \
    -DLANGUAGE_ENTRY=tree_sitter_"$name" \
    -o "$work/ts/$name/runner" \
    "$ubnfc/benchmarks/c/ts_runner.c" "$work/ts/$name/src/parser.c" "$TREE_SITTER_SRC/lib/src/lib.c"
}
build_ts json "$ubnfc/examples/json/treesitter"
build_ts operators "$ubnfc/examples/arith/treesitter/operators"
build_ts power "$ubnfc/examples/arith/treesitter/power"

# 3. ubnfc: front で IR を作り、Rust / Java backend へ通す。
#    Operators / Power は既存の速度比較（family B）と同じ IR fixture を使う。
(cd "$ubnfc" && cargo build --offline --release --bin ubnfc --bin ubnfc-rust)
(cd "$ubnfc/ubnfc-java" && mvn -o -q -DskipTests package)
jar=$ubnfc/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar
rm -rf "$work/ubnfc"; mkdir -p "$work/ubnfc/ir" "$work/ubnfc/rust" "$work/ubnfc/java/src" "$work/ubnfc/java/classes"
"$ubnfc/target/release/ubnfc" ir --grammar "$ubnfc/examples/json/json.ubnf" --out "$work/ubnfc/ir/json.ir.json"
cp "$ubnfc/ir/fixtures/unlaxer-dsl__src__test__resources__associative__Operators.ubnf.ir.json" "$work/ubnfc/ir/operators.ir.json"
cp "$ubnfc/ir/fixtures/unlaxer-dsl__src__test__resources__right-associative__Power.ubnf.ir.json" "$work/ubnfc/ir/power.ir.json"
for name in json operators power; do
  "$ubnfc/target/release/ubnfc-rust" --ir "$work/ubnfc/ir/$name.ir.json" --out "$work/ubnfc/rust/$name" --no-emit-driver
  sed -i "s/^name = \"ubnfc-generated\"/name = \"gen-$name\"/" "$work/ubnfc/rust/$name/Cargo.toml"
  java -cp "$jar" org.unlaxer.ubnfc.java.Main --ir "$work/ubnfc/ir/$name.ir.json" \
    --out "$work/ubnfc/java/src" --package "gen.$name" --no-emit-driver
done
javac -nowarn -d "$work/ubnfc/java/classes" $(find "$work/ubnfc/java/src" -name '*.java')
javac -nowarn -cp "$work/ubnfc/java/classes" -d "$work/ubnfc/java/classes" "$ubnfc/benchmarks/java/UbnfcRunner.java"

# 4. ubnfc Rust の計測 bin（ubnfc の cargo workspace には入れない）。
rm -rf "$work/bench-ext"; mkdir -p "$work/bench-ext/src"
cp "$ubnfc/benchmarks/src/bench_util.rs" "$work/bench-ext/src/"
cp "$ubnfc/benchmarks/src/family_ext.rs" "$work/bench-ext/src/main.rs"
{ echo '[workspace]'; echo '[package]'; echo 'name = "ubnfc-cmp-ext"'
  echo 'version = "0.0.0"'; echo 'edition = "2021"'; echo '[dependencies]'
  echo "gen-json = { path = \"$work/ubnfc/rust/json\" }"
  echo "gen-operators = { path = \"$work/ubnfc/rust/operators\" }"
  echo "gen-power = { path = \"$work/ubnfc/rust/power\" }"; } > "$work/bench-ext/Cargo.toml"
(cd "$work/bench-ext" && cargo build --offline --release)

# 5. 速度比較の入力。
mkdir -p "$work/inputs"
python3 "$ubnfc/benchmarks/scripts/make-json-inputs.py" "$JSON_TEST_SUITE/test_parsing" "$work/inputs"
python3 - "$work/inputs" <<'PY'
import hashlib, pathlib, sys
out = pathlib.Path(sys.argv[1])
# 算術: 既存の速度比較（benchmarks/src/family_b.rs）と同じ 2 入力。
cases = {"operators-atoms-512.txt": "+".join(["1"] * 512), "power-64.txt": "^".join(["2"] * 64)}
lines = []
for name, text in cases.items():
    (out / name).write_bytes(text.encode())
    lines.append(f"{hashlib.sha256(text.encode()).hexdigest()}  {name}  {len(text)} bytes")
    print(lines[-1])
(out / "inputs.sha256").open("a", encoding="utf-8").write("\n".join(lines) + "\n")
PY
echo "# prepare-external.sh 完了: WORK=$work" >&2
