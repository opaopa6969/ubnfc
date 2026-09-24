#!/usr/bin/env bash
# 外部パーサ生成系（ANTLR4 / tree-sitter）と JSONTestSuite を benchmarks/external/ に取得する。
#
#   使い方: scripts/fetch-external-tools.sh [--check]
#
#   --check … 取得済みかどうかだけを見る（ダウンロードしない）。全て揃っていれば 0。
#
# 出力（最後の 4 行、`KEY=値` 形式。呼び出し側はこれを eval して使う）:
#   ANTLR_JAR=<antlr4 complete jar の絶対パス>
#   TREE_SITTER_BIN=<tree-sitter CLI の絶対パス>
#   TREE_SITTER_SRC=<tree-sitter C ランタイムのチェックアウト（lib/src, lib/include を含む）>
#   JSON_TEST_SUITE=<JSONTestSuite のチェックアウト>
#
# 取得先とバージョンは全て固定してある（下の PIN 群）。ネットワークが無い環境では
# 「何が足りないか」を述べて非 0 で終わる（黙って別物を使うことはしない）。
# benchmarks/external/ は .gitignore 済みで、この中身はコミットしない。
set -euo pipefail

# ---- PIN（変更するときは sha256 も更新すること） --------------------------------
ANTLR_VERSION=4.13.2
ANTLR_SHA256=eae2dfa119a64327444672aff63e9ec35a20180dc5b8090b7a6ab85125df4d76
ANTLR_URL="https://repo1.maven.org/maven2/org/antlr/antlr4/${ANTLR_VERSION}/antlr4-${ANTLR_VERSION}-complete.jar"

# CLI と C ランタイムは同じバージョンに揃える（生成 parser.c の ABI を合わせるため）。
TREE_SITTER_VERSION=0.25.10
TREE_SITTER_COMMIT=208c6cac1453315e979f05ab34b6d4f7cd0340be   # refs/tags/v0.25.10
TREE_SITTER_REPO=https://github.com/tree-sitter/tree-sitter

JSON_TEST_SUITE_COMMIT=1ef36fa01286573e846ac449e8683f8833c5b26a
JSON_TEST_SUITE_REPO=https://github.com/nst/JSONTestSuite
# ---------------------------------------------------------------------------------

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
EXTERNAL="$ROOT/benchmarks/external"
ANTLR_JAR="$EXTERNAL/antlr-${ANTLR_VERSION}-complete.jar"
TREE_SITTER_BIN="$EXTERNAL/node_modules/.bin/tree-sitter"
TREE_SITTER_SRC="$EXTERNAL/tree-sitter-${TREE_SITTER_VERSION}"
JSON_TEST_SUITE="$EXTERNAL/JSONTestSuite"

CHECK_ONLY=0
[ "${1:-}" = "--check" ] && CHECK_ONLY=1

log() { printf '[fetch-external-tools] %s\n' "$*" >&2; }
die() { printf '[fetch-external-tools] ERROR: %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "$1 が必要（$2）"; }

emit() {
  echo "ANTLR_JAR=$ANTLR_JAR"
  echo "TREE_SITTER_BIN=$TREE_SITTER_BIN"
  echo "TREE_SITTER_SRC=$TREE_SITTER_SRC"
  echo "JSON_TEST_SUITE=$JSON_TEST_SUITE"
}

have_all() {
  [ -f "$ANTLR_JAR" ] && [ -x "$TREE_SITTER_BIN" ] \
    && [ -f "$TREE_SITTER_SRC/lib/src/lib.c" ] \
    && [ -d "$JSON_TEST_SUITE/test_parsing" ]
}

if [ "$CHECK_ONLY" = 1 ]; then
  if have_all; then emit; exit 0; fi
  missing=()
  [ -f "$ANTLR_JAR" ] || missing+=("ANTLR jar ($ANTLR_JAR)")
  [ -x "$TREE_SITTER_BIN" ] || missing+=("tree-sitter CLI ($TREE_SITTER_BIN)")
  [ -f "$TREE_SITTER_SRC/lib/src/lib.c" ] || missing+=("tree-sitter C runtime ($TREE_SITTER_SRC)")
  [ -d "$JSON_TEST_SUITE/test_parsing" ] || missing+=("JSONTestSuite ($JSON_TEST_SUITE)")
  die "未取得: ${missing[*]} — ネットワークのある環境で scripts/fetch-external-tools.sh を実行すること"
fi

need curl "ANTLR jar の取得"
need git "tree-sitter / JSONTestSuite の取得"
need npm "tree-sitter-cli の取得"
need sha256sum "ANTLR jar の検証"

mkdir -p "$EXTERNAL"

# --- ANTLR complete jar（Maven Central、sha256 固定） -----------------------------
if [ -f "$ANTLR_JAR" ] && [ "$(sha256sum <"$ANTLR_JAR" | cut -d' ' -f1)" = "$ANTLR_SHA256" ]; then
  log "ANTLR ${ANTLR_VERSION}: 取得済み"
else
  log "ANTLR ${ANTLR_VERSION}: $ANTLR_URL"
  tmp="$ANTLR_JAR.part"
  curl -fsSL --retry 2 --connect-timeout 20 -o "$tmp" "$ANTLR_URL" \
    || die "ANTLR jar を取得できない（ネットワーク不通か URL 変更）: $ANTLR_URL"
  got=$(sha256sum <"$tmp" | cut -d' ' -f1)
  [ "$got" = "$ANTLR_SHA256" ] || { rm -f "$tmp"; die "ANTLR jar の sha256 不一致: expected $ANTLR_SHA256 got $got"; }
  mv "$tmp" "$ANTLR_JAR"
fi

# --- tree-sitter CLI（npm、バージョン固定） ---------------------------------------
if [ -x "$TREE_SITTER_BIN" ] && [ "$("$TREE_SITTER_BIN" --version 2>/dev/null | awk '{print $2}')" = "$TREE_SITTER_VERSION" ]; then
  log "tree-sitter-cli ${TREE_SITTER_VERSION}: 取得済み"
else
  log "tree-sitter-cli ${TREE_SITTER_VERSION}: npm install"
  # --prefix 配下に package.json が無いと npm が上位を探しに行くので先に置く。
  [ -f "$EXTERNAL/package.json" ] || echo '{"name":"ubnfc-external","private":true,"version":"0.0.0"}' >"$EXTERNAL/package.json"
  npm install --no-audit --no-fund --prefix "$EXTERNAL" "tree-sitter-cli@${TREE_SITTER_VERSION}" >&2 \
    || die "tree-sitter-cli を npm から取得できない（ネットワーク不通か registry 不達）"
fi
[ -x "$TREE_SITTER_BIN" ] || die "tree-sitter CLI が $TREE_SITTER_BIN に無い"

# --- tree-sitter C ランタイム（生成 parser.c をリンクするのに要る） ----------------
# npm の tree-sitter-cli にはランタイム C ソースが入らないので、CLI と同じタグを clone する。
if [ -f "$TREE_SITTER_SRC/lib/src/lib.c" ]; then
  log "tree-sitter C runtime ${TREE_SITTER_VERSION}: 取得済み"
else
  log "tree-sitter C runtime ${TREE_SITTER_VERSION}: $TREE_SITTER_REPO @ $TREE_SITTER_COMMIT"
  rm -rf "$TREE_SITTER_SRC.part"
  git init -q "$TREE_SITTER_SRC.part"
  git -C "$TREE_SITTER_SRC.part" remote add origin "$TREE_SITTER_REPO"
  git -C "$TREE_SITTER_SRC.part" fetch -q --depth 1 origin "$TREE_SITTER_COMMIT" \
    || die "tree-sitter を clone できない（ネットワーク不通）: $TREE_SITTER_REPO"
  git -C "$TREE_SITTER_SRC.part" checkout -q FETCH_HEAD
  rm -rf "$TREE_SITTER_SRC"
  mv "$TREE_SITTER_SRC.part" "$TREE_SITTER_SRC"
fi
[ -f "$TREE_SITTER_SRC/lib/src/lib.c" ] || die "tree-sitter の C ランタイムが見つからない: $TREE_SITTER_SRC/lib/src/lib.c"

# --- JSONTestSuite（commit 固定） --------------------------------------------------
if [ -d "$JSON_TEST_SUITE/test_parsing" ] \
   && [ "$(git -C "$JSON_TEST_SUITE" rev-parse HEAD 2>/dev/null)" = "$JSON_TEST_SUITE_COMMIT" ]; then
  log "JSONTestSuite: 取得済み ($JSON_TEST_SUITE_COMMIT)"
else
  log "JSONTestSuite: $JSON_TEST_SUITE_REPO @ $JSON_TEST_SUITE_COMMIT"
  rm -rf "$JSON_TEST_SUITE.part"
  git init -q "$JSON_TEST_SUITE.part"
  git -C "$JSON_TEST_SUITE.part" remote add origin "$JSON_TEST_SUITE_REPO"
  git -C "$JSON_TEST_SUITE.part" fetch -q --depth 1 origin "$JSON_TEST_SUITE_COMMIT" \
    || die "JSONTestSuite を clone できない（ネットワーク不通）: $JSON_TEST_SUITE_REPO"
  git -C "$JSON_TEST_SUITE.part" checkout -q FETCH_HEAD
  rm -rf "$JSON_TEST_SUITE"
  mv "$JSON_TEST_SUITE.part" "$JSON_TEST_SUITE"
fi
[ -d "$JSON_TEST_SUITE/test_parsing" ] || die "JSONTestSuite の test_parsing が無い: $JSON_TEST_SUITE"

log "完了。以下のパスを使う:"
emit
