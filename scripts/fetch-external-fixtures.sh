#!/usr/bin/env bash
# CI が読む外部 fixture repository（unlaxer-parser / tinyexpression）を、
# scripts/external-pins.env に固定した commit で <取得先> の下へ取得する（issue #105）。
#
#   使い方: scripts/fetch-external-fixtures.sh <取得先ディレクトリ>
#
# 取得先には unlaxer-parser/ と tinyexpression/ ができる。各 commit だけを
# `git fetch --depth 1 origin <sha>` で取り、detached HEAD で展開する。取得先に同じ
# commit の checkout が既にあり作業ツリーが clean ならそのまま使う（それ以外は作り直す）。
#
# 出力（最後の 3 行、`KEY=値` 形式）:
#   UNLAXER_PARSER_DIR=<unlaxer-parser の checkout>
#   UNLAXER_PARSER_ROOT=<同上（generated.rs / handwritten-target が読む名前）>
#   TINYEXPRESSION_DIR=<tinyexpression の checkout>
# GITHUB_ENV が設定されていれば（GitHub Actions）、同じ 3 行をそこへも追記する。
#
# pin の更新手順は docs/ci-external-fixtures.md。
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
PINS=${EXTERNAL_PINS:-"$ROOT/scripts/external-pins.env"}
dest=${1:?"使い方: $0 <取得先ディレクトリ>"}

declare -A pin=()
while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in '' | '#'*) continue ;; esac
  if [[ ! "$line" =~ ^([A-Z_]+)=([^[:space:]\"\'\$]+)$ ]]; then
    echo "$PINS: 解釈できない行: $line" >&2
    exit 2
  fi
  pin[${BASH_REMATCH[1]}]=${BASH_REMATCH[2]}
done <"$PINS"

mkdir -p "$dest"
dest=$(cd "$dest" && pwd)

fetch() {
  local name=$1 key=$2
  local url=${pin[${key}_REPO]:-} commit=${pin[${key}_COMMIT]:-}
  if [ -z "$url" ] || [[ ! "$commit" =~ ^[0-9a-f]{40}$ ]]; then
    echo "$PINS: ${key}_REPO と 40 桁の ${key}_COMMIT が要る（${key}_COMMIT=$commit）" >&2
    exit 2
  fi
  local dir="$dest/$name"
  if [ -d "$dir/.git" ] \
    && [ "$(git -C "$dir" rev-parse HEAD 2>/dev/null)" = "$commit" ] \
    && [ -z "$(git -C "$dir" status --porcelain --untracked-files=no)" ]; then
    echo "$name: $commit（取得済み）" >&2
    return
  fi
  rm -rf -- "$dir"
  git init -q "$dir"
  git -C "$dir" remote add origin "$url"
  git -C "$dir" fetch -q --depth 1 --no-tags origin "$commit"
  git -C "$dir" -c advice.detachedHead=false checkout -q --detach FETCH_HEAD
  local got
  got=$(git -C "$dir" rev-parse HEAD)
  if [ "$got" != "$commit" ]; then
    echo "$name: 取得した commit が pin と違う（$got != $commit）" >&2
    exit 1
  fi
  echo "$name: $commit（$url）" >&2
}

fetch unlaxer-parser UNLAXER_PARSER
fetch tinyexpression TINYEXPRESSION

out=$(printf '%s\n' \
  "UNLAXER_PARSER_DIR=$dest/unlaxer-parser" \
  "UNLAXER_PARSER_ROOT=$dest/unlaxer-parser" \
  "TINYEXPRESSION_DIR=$dest/tinyexpression")
if [ -n "${GITHUB_ENV:-}" ]; then
  printf '%s\n' "$out" >>"$GITHUB_ENV"
fi
printf '%s\n' "$out"
