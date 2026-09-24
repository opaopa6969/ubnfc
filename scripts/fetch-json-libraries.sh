#!/usr/bin/env bash
# JSON 専用ライブラリ（Jackson / Gson）の jar を benchmarks/external/ に取得する（issue #68）。
#
#   使い方: scripts/fetch-json-libraries.sh [--check]
#
#   --check … 取得済みかどうかだけを見る（ダウンロードしない）。全て揃っていれば 0。
#
# 出力（最後の行、`KEY=値` 形式。呼び出し側はこれを eval して使う）:
#   JACKSON_CP=<jackson-core:jackson-databind:jackson-annotations の classpath>
#   GSON_CP=<gson:error_prone_annotations の classpath>
#   JSONLIB_CP=<上の 2 つを結合したもの>
#   JACKSON_VERSION / GSON_VERSION
#
# 取得先とバージョンは全て固定してある（下の PIN 群。sha256 は Maven Central の
# `<jar>.sha256` をそのまま写した）。ネットワークが無い環境では「何が足りないか」を
# 述べて非 0 で終わる（黙って別物を使うことはしない）。
# benchmarks/external/ は .gitignore 済みで、この中身はコミットしない。
#
# Rust 側（serde_json）はここでは扱わない。crates.io は遮断されているので
# `~/.cargo/registry` にあるものを `--offline` で使う（benchmarks/scripts/run-json-libraries.sh）。
set -euo pipefail

# ---- PIN（変更するときは sha256 も更新すること） --------------------------------
# Jackson 2.x 系の最新安定（2026-09-24 時点の Maven Central の <release>）。
# databind は annotations と core に依存する。annotations は 2.x 系の最新が 2.22
# （2.22.3 は core/databind だけのパッチ）。
JACKSON_VERSION=2.22.3
JACKSON_ANNOTATIONS_VERSION=2.22
JACKSON_CORE_SHA256=8a501126a385b25841915d839508f8a66e2a0dbc8a6709d055ef3b3e852b094c
JACKSON_DATABIND_SHA256=556db5439e206114346043f68d200497dc96a0bca62a360a81784092ebd0e0a9
JACKSON_ANNOTATIONS_SHA256=21ddb598807d3a51a876704eb979d9296e1c6a6f47ab1826ff88c6d6a127a2d0

GSON_VERSION=2.14.0
GSON_SHA256=2cbd119bf1961c28788310963dc80ba65f58cdeec1dd139c8bdb1240faa2c36f
# gson の pom は error_prone_annotations を optional にしていない（実行時は無くても
# 動くと pom 自身が書いているが、宣言どおり置いておく）。
ERROR_PRONE_VERSION=2.48.0
ERROR_PRONE_SHA256=b49c5c958316ed67a09c699dda9aa749caf434d51d863dea599ef36a49b9c855
# ---------------------------------------------------------------------------------

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
EXTERNAL="$ROOT/benchmarks/external"
MAVEN=https://repo1.maven.org/maven2

JACKSON_CORE="$EXTERNAL/jackson-core-${JACKSON_VERSION}.jar"
JACKSON_DATABIND="$EXTERNAL/jackson-databind-${JACKSON_VERSION}.jar"
JACKSON_ANNOTATIONS="$EXTERNAL/jackson-annotations-${JACKSON_ANNOTATIONS_VERSION}.jar"
GSON_JAR="$EXTERNAL/gson-${GSON_VERSION}.jar"
ERROR_PRONE_JAR="$EXTERNAL/error_prone_annotations-${ERROR_PRONE_VERSION}.jar"

CHECK_ONLY=0
[ "${1:-}" = "--check" ] && CHECK_ONLY=1

log() { printf '[fetch-json-libraries] %s\n' "$*" >&2; }
die() { printf '[fetch-json-libraries] ERROR: %s\n' "$*" >&2; exit 1; }

emit() {
  local jackson="$JACKSON_CORE:$JACKSON_DATABIND:$JACKSON_ANNOTATIONS"
  local gson="$GSON_JAR:$ERROR_PRONE_JAR"
  echo "JACKSON_CP=$jackson"
  echo "GSON_CP=$gson"
  echo "JSONLIB_CP=$jackson:$gson"
  echo "JACKSON_VERSION=$JACKSON_VERSION"
  echo "JACKSON_ANNOTATIONS_VERSION=$JACKSON_ANNOTATIONS_VERSION"
  echo "GSON_VERSION=$GSON_VERSION"
}

have_all() {
  [ -f "$JACKSON_CORE" ] && [ -f "$JACKSON_DATABIND" ] && [ -f "$JACKSON_ANNOTATIONS" ] \
    && [ -f "$GSON_JAR" ] && [ -f "$ERROR_PRONE_JAR" ]
}

if [ "$CHECK_ONLY" = 1 ]; then
  if have_all; then emit; exit 0; fi
  missing=()
  for f in "$JACKSON_CORE" "$JACKSON_DATABIND" "$JACKSON_ANNOTATIONS" "$GSON_JAR" "$ERROR_PRONE_JAR"; do
    [ -f "$f" ] || missing+=("$(basename "$f")")
  done
  die "未取得: ${missing[*]} — ネットワークのある環境で scripts/fetch-json-libraries.sh を実行すること"
fi

command -v curl >/dev/null 2>&1 || die "curl が必要（jar の取得）"
command -v sha256sum >/dev/null 2>&1 || die "sha256sum が必要（jar の検証）"
mkdir -p "$EXTERNAL"

fetch_jar() { # <出力パス> <Maven の相対パス> <sha256>
  local dest=$1 rel=$2 want=$3 url="$MAVEN/$2"
  if [ -f "$dest" ] && [ "$(sha256sum <"$dest" | cut -d' ' -f1)" = "$want" ]; then
    log "$(basename "$dest"): 取得済み"
    return
  fi
  log "$(basename "$dest"): $url"
  local tmp="$dest.part"
  curl -fsSL --retry 2 --connect-timeout 20 -o "$tmp" "$url" \
    || die "$(basename "$dest") を取得できない（ネットワーク不通か URL 変更）: $url"
  local got
  got=$(sha256sum <"$tmp" | cut -d' ' -f1)
  [ "$got" = "$want" ] || { rm -f "$tmp"; die "$(basename "$dest") の sha256 不一致: expected $want got $got"; }
  mv "$tmp" "$dest"
}

fetch_jar "$JACKSON_CORE" \
  "com/fasterxml/jackson/core/jackson-core/${JACKSON_VERSION}/jackson-core-${JACKSON_VERSION}.jar" \
  "$JACKSON_CORE_SHA256"
fetch_jar "$JACKSON_DATABIND" \
  "com/fasterxml/jackson/core/jackson-databind/${JACKSON_VERSION}/jackson-databind-${JACKSON_VERSION}.jar" \
  "$JACKSON_DATABIND_SHA256"
fetch_jar "$JACKSON_ANNOTATIONS" \
  "com/fasterxml/jackson/core/jackson-annotations/${JACKSON_ANNOTATIONS_VERSION}/jackson-annotations-${JACKSON_ANNOTATIONS_VERSION}.jar" \
  "$JACKSON_ANNOTATIONS_SHA256"
fetch_jar "$GSON_JAR" \
  "com/google/code/gson/gson/${GSON_VERSION}/gson-${GSON_VERSION}.jar" \
  "$GSON_SHA256"
fetch_jar "$ERROR_PRONE_JAR" \
  "com/google/errorprone/error_prone_annotations/${ERROR_PRONE_VERSION}/error_prone_annotations-${ERROR_PRONE_VERSION}.jar" \
  "$ERROR_PRONE_SHA256"

log "完了。以下のパスを使う:"
emit
