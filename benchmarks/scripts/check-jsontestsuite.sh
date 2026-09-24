#!/usr/bin/env bash
# JSONTestSuite の test_parsing 全件を 4 実装へ流し、突き合わせ表を作る（issue #26）。
#
#   WORK=... benchmarks/scripts/check-jsontestsuite.sh [出力ディレクトリ]
#
# 前提: WORK は benchmarks/scripts/prepare-external.sh が作ったもの。
# 出力: $out/verdicts.tsv（file, 期待, 4 実装の判定と理由）と $out/summary.md。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
ubnfc=$(cd "$here/../.." && pwd)
work=${WORK:?}
out=${1:-$work/check}
mkdir -p "$out"
eval "$("$ubnfc/scripts/fetch-external-tools.sh" --check)"
suite=$JSON_TEST_SUITE/test_parsing
antlr_cp=$ANTLR_JAR:$work/antlr/classes

files=("$suite"/*.json)
echo "# 対象 ${#files[@]} 件" >&2

"$work/bench-ext/target/release/ubnfc-cmp-ext" check json "${files[@]}" > "$out/ubnfc-rust.tsv"
java -cp "$work/ubnfc/java/classes" UbnfcRunner check json "${files[@]}" > "$out/ubnfc-java.tsv"
java -cp "$antlr_cp" AntlrRunner check json "${files[@]}" > "$out/antlr.tsv"
"$work/ts/json/runner" check "${files[@]}" > "$out/tree-sitter.tsv"

python3 "$here/merge-verdicts.py" "$out" "$suite"
