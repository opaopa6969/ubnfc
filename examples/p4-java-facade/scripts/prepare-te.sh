#!/usr/bin/env bash
# 隣接 tinyexpression checkout の作業ツリー build 出力から、この module が
# コンパイル・実行に使う jar を作る（tinyexpression 側は読むだけ）。
#
# 公開済みの tinyExpression-1.4.15.jar（~/.m2 と target/ の両方）は古く、
# facade が使う mapper API（mapParsedTree / selectParsedTokenWithSourceMap）を持たない。
# そのため作業ツリーの target/classes から jar を作り直す。共有 ~/.m2 には何も書かない。
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
tiny=${TINYEXPRESSION_DIR:-$(cd "$module/../../../tinyexpression" && pwd)}
classes="$tiny/target/classes"
[[ -d "$classes" ]] || { echo "tinyexpression の target/classes が無い: $classes" >&2; exit 1; }
out="$module/target/te"
mkdir -p "$out"
jar --create --file "$out/tinyExpression-worktree.jar" -C "$classes" .
echo "作成: $out/tinyExpression-worktree.jar （元: $classes）"
