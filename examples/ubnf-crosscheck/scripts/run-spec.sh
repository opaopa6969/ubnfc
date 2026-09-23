#!/usr/bin/env bash
# 仕様由来ケース集（corpus/spec/ubnf/*.json）を 4 実装へ流す。
#
#   A ubnfc 手書き frontend      crates/ubnfc-front/src/syntax
#   B ubnfc 自己ホスト生成       crates/ubnfc-selfhost
#   C unlaxer-ubnf（手書き Rust） unlaxer-parser/rust/unlaxer-ubnf（examples/inspect）
#   D Java bootstrap             unlaxer-dsl org.unlaxer.dsl.bootstrap.UBNFMapper
#
# 期待値はすべて unlaxer-dsl/specs/*.md の規範文から導いたもので、
# どの実装の観測値も使っていない（corpus/spec/ubnf/README.md）。
#
# 使い方: scripts/run-spec.sh [OUT_DIR]
# 環境変数: UNLAXER_PARSER_DIR, MAVEN_REPO, UNLAXER_VERSION
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
workspace=$(cd "$module/../.." && pwd)
out=${1:-$module/out-spec}
unlaxer=${UNLAXER_PARSER_DIR:-$(cd "$workspace/../unlaxer-parser" && pwd)}
repo=${MAVEN_REPO:-$HOME/.m2/repository}
version=${UNLAXER_VERSION:-3.0.15}
classpath="$repo/org/unlaxer/unlaxer-common/$version/unlaxer-common-$version.jar:$repo/org/unlaxer/unlaxer-dsl/$version/unlaxer-dsl-$version.jar"
mkdir -p "$out"
work=$(mktemp -d -t ubnf-spec.XXXXXXXX)
trap 'rm -rf -- "$work"' EXIT

cases=$work/cases
mkdir -p "$cases"
# --- ケースをファイルへ展開（id -> ファイル名の対応は manifest.tsv） ---
python3 "$module/scripts/spec_cases.py" "$workspace/corpus/spec/ubnf" "$cases" > "$out/manifest.tsv"

files=()
while IFS= read -r name; do files+=("$cases/$name"); done < <(cd "$cases" && ls | sort)

# --- A/B: ubnfc 手書き + 自己ホスト ---
cargo build --offline --release --manifest-path "$module/Cargo.toml" >&2
"$module/target/release/ubnf-crosscheck" "${files[@]}" > "$out/ubnfc.jsonl"

# --- C: unlaxer-ubnf（rustc で直接ビルドし、参照リポジトリの target を汚さない） ---
rustc --edition=2021 --crate-type=rlib --crate-name=unlaxer_ubnf -O \
  "$unlaxer/rust/unlaxer-ubnf/src/lib.rs" -o "$work/libunlaxer_ubnf.rlib"
rustc --edition=2021 -O "$unlaxer/rust/unlaxer-ubnf/examples/inspect.rs" \
  --extern unlaxer_ubnf="$work/libunlaxer_ubnf.rlib" -o "$work/inspect"
: > "$out/unlaxer-ubnf.jsonl"
for file in "${files[@]}"; do
  printf '{"input":"%s","result":' "$(basename "$file")" >> "$out/unlaxer-ubnf.jsonl"
  "$work/inspect" "$file" | tr -d '\n' >> "$out/unlaxer-ubnf.jsonl"
  printf '}\n' >> "$out/unlaxer-ubnf.jsonl"
done

# --- D: Java bootstrap（構文）と GrammarValidator（診断） ---
javac -cp "$classpath" -d "$work/java" "$module/java/UbnfProbe.java"
java -cp "$work/java:$classpath" UbnfProbe "${files[@]}" > "$out/java-bootstrap.jsonl"
java -cp "$work/java:$classpath" UbnfProbe --validate "${files[@]}" > "$out/java-validate.jsonl"

# --- A の validator（ubnfc validate --format json）---
ubnfc=${UBNFC_BIN:-$workspace/target/debug/ubnfc}
if [ ! -x "$ubnfc" ]; then
  (cd "$workspace" && cargo build --offline -q --bin ubnfc) >&2
fi
: > "$out/ubnfc-validate.jsonl"
for file in "${files[@]}"; do
  name=$(basename "$file")
  status=0
  json=$("$ubnfc" validate --grammar "$file" --format json 2>"$work/err") || status=$?
  if [ -z "$json" ]; then json='null'; fi
  printf '{"input":"%s","exit":%d,"report":%s}\n' "$name" "$status" "$(printf '%s' "$json" | tr -d '\n')" \
    >> "$out/ubnfc-validate.jsonl"
done

python3 "$module/scripts/compare-spec.py" "$workspace/corpus/spec/ubnf" "$out" > "$out/report.md"
echo "書き出し: $out/report.md" >&2
