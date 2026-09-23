#!/usr/bin/env bash
# UBNF 1 文法・4 実装のクロスチェックを走らせて JSON/Markdown を出す。
#
#   A ubnfc 手書き frontend      crates/ubnfc-front/src/syntax
#   B ubnfc 自己ホスト生成       crates/ubnfc-selfhost
#   C unlaxer-ubnf（手書き Rust） unlaxer-parser/rust/unlaxer-ubnf（examples/inspect）
#   D Java bootstrap             unlaxer-dsl org.unlaxer.dsl.bootstrap.UBNFMapper
#
# 使い方: scripts/run.sh [OUT_DIR]
# 環境変数: UNLAXER_PARSER_DIR, TINYEXPRESSION_DIR, MAVEN_REPO, UNLAXER_VERSION
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
out=${1:-$module/out}
unlaxer=${UNLAXER_PARSER_DIR:-$(cd "$module/../../../unlaxer-parser" && pwd)}
tiny=${TINYEXPRESSION_DIR:-$(cd "$module/../../../tinyexpression" && pwd)}
repo=${MAVEN_REPO:-$HOME/.m2/repository}
version=${UNLAXER_VERSION:-3.0.15}
classpath="$repo/org/unlaxer/unlaxer-common/$version/unlaxer-common-$version.jar:$repo/org/unlaxer/unlaxer-dsl/$version/unlaxer-dsl-$version.jar"
mkdir -p "$out"
work=$(mktemp -d -t ubnf-crosscheck.XXXXXXXX)
trap 'rm -rf -- "$work"' EXIT

# --- 入力コーパスを 1 ディレクトリへ集める（sibling repo は読むだけ） ---
cases=$work/cases
mkdir -p "$cases"
manifest=$out/corpus.tsv
: > "$manifest"
while IFS= read -r path; do
  relative=${path#"$unlaxer/"}
  relative=${relative#"$tiny/"}
  id=$(printf '%s' "$relative" | tr '/' '_')
  cp "$path" "$cases/$id"
  printf '%s\t%s\tfile\n' "$id" "$path" >> "$manifest"
done < <(
  find "$unlaxer" -name '*.ubnf' -not -path '*/target/*' | sort
  printf '%s\n' "$tiny/tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf"
)
# 不正入力（unlaxer-ubnf の negative.tsv をそのまま使う。1 行 = id<TAB>source）
while IFS=$'\t' read -r name source; do
  [ -n "$name" ] || continue
  printf '%s' "$source" > "$cases/negative__$name.ubnf"
  printf '%s\t%s\tnegative\n' "negative__$name.ubnf" "$name" >> "$manifest"
done < "$unlaxer/rust/unlaxer-ubnf/tests/fixtures/negative.tsv"

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

# --- D: Java bootstrap ---
javac -cp "$classpath" -d "$work/java" "$module/java/UbnfProbe.java"
java -cp "$work/java:$classpath" UbnfProbe "${files[@]}" > "$out/java-bootstrap.jsonl"

# --- 突き合わせ ---
python3 "$module/scripts/compare.py" "$out" > "$out/report.md"
echo "書き出し: $out/report.md" >&2
