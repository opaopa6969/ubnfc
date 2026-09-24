#!/usr/bin/env bash
# examples/*-vscode の VSIX を「生成物だけ」から組む。
#
#   1. 生成器を組む（LSP は opt-in の Maven profile `lsp`。既定ビルドには入らない）
#   2. ubnfc-java でパーサを生成（LSP feature とは無関係。出力はこの手順で変わらない）
#   3. ubnfc-lsp --emit-lsp で LSP 包みを別パッケージへ生成し、--emit-tmlanguage で TextMate 文法も出す
#      （--with-dap を付けると --emit-dap も渡して文法デバッガ（DAP）を <lsp-package>.dap へ足す）
#   4. extern token scanner を複製（P4 のみ。読むだけ。編集しない）
#   5. サーバ fat jar（golden テストもここで走る）
#   6. npm ci + vsce package → examples/<target>-vscode/target/*.vsix
#
# 使い方: scripts/build-vsix.sh [--with-dap] [p4|ubnf|all]（既定 all）
#   --with-dap: DAP（文法デバッガ）も生成し、server の -Pdap で lsp4j.debug を引いて
#               golden DAP テストも走らせる。付けないと生成物も jar も従来どおり（opt-in、D-067）。
# 環境変数: MVN（既定 mvn）、MAVEN_FLAGS（既定 -o。依存が未取得なら空にする）、SKIP_VSIX=1 で 6 を飛ばす。
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
mvn_bin=${MVN:-mvn}
mvn_flags=${MAVEN_FLAGS--o}
version=0.1.0-SNAPSHOT
with_dap=0
positional=()
for argument in "$@"; do
  case "$argument" in
    --with-dap) with_dap=1 ;;
    -*) echo "不明な引数: $argument" >&2; exit 2 ;;
    *) positional+=("$argument") ;;
  esac
done
targets=${positional[0]:-all}
[[ "$targets" == "all" ]] && targets="p4 ubnf"
dap_emit=()
dap_profile=()
if [[ "$with_dap" == "1" ]]; then dap_emit=(--emit-dap); dap_profile=(-Pdap); fi

echo "[1/6] 生成器（-Plsp）"
"$mvn_bin" $mvn_flags -q -f "$repo/pom.xml" -Plsp -DskipTests package

build_one() {
  local target=$1 ir example lsp_package parser_package language_id scanner_class server_name jar vsix scanner_source
  case "$target" in
    p4)
      ir="$repo/ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json"
      example="$repo/examples/p4-vscode"
      parser_package=org.ubnfc.p4.generated
      lsp_package=org.ubnfc.p4.lsp
      language_id=tinyexpression-p4
      scanner_class=org.ubnfc.p4.P4Scanners
      scanner_source="$repo/examples/p4-java/src/main/java/org/ubnfc/p4/P4Scanners.java"
      server_name="TinyExpression P4 (ubnfc)"
      jar=ubnfc-p4-lsp-server
      vsix=ubnfc-tinyexpression-p4-0.1.0.vsix
      ;;
    ubnf)
      ir="$repo/ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json"
      example="$repo/examples/ubnf-vscode"
      parser_package=org.ubnfc.ubnf.generated
      lsp_package=org.ubnfc.ubnf.lsp
      language_id=ubnf
      scanner_class=""
      scanner_source=""
      server_name="UBNF (ubnfc)"
      jar=ubnfc-ubnf-lsp-server
      vsix=ubnfc-ubnf-0.1.0.vsix
      ;;
    *) echo "不明な target: $target" >&2; return 1 ;;
  esac
  local server="$example/server"
  local generated="$server/target/generated-sources/ubnfc"

  echo "[2/6] パーサ生成（$target）"
  rm -rf "$generated"
  mkdir -p "$generated"
  java -cp "$repo/ubnfc-java/target/ubnfc-java-$version.jar" org.ubnfc.java.Main \
    --ir "$ir" --out "$generated" --package "$parser_package" --no-emit-driver

  echo "[3/6] LSP 包み + TextMate 文法（opt-in: --emit-lsp）"
  mkdir -p "$example/syntaxes"
  local scanner_option=()
  [[ -n "$scanner_class" ]] && scanner_option=(--scanner-class "$scanner_class")
  java -cp "$repo/ubnfc-lsp/target/ubnfc-lsp-$version.jar:$repo/ubnfc-java/target/ubnfc-java-$version.jar" \
    org.ubnfc.lsp.Main --emit-lsp --ir "$ir" --out "$generated" \
    --package "$lsp_package" --parser-package "$parser_package" \
    --language-id "$language_id" "${scanner_option[@]}" "${dap_emit[@]}" \
    --server-name "$server_name" --server-version 0.1.0 \
    --emit-tmlanguage "$example/syntaxes/$language_id.tmLanguage.json"

  echo "[4/6] extern token scanner"
  if [[ -n "$scanner_source" ]]; then
    mkdir -p "$generated/org/ubnfc/p4"
    cp "$scanner_source" "$generated/org/ubnfc/p4/P4Scanners.java"
  else
    echo "  （この文法は extern token を使わない）"
  fi

  echo "[5/6] サーバ jar（golden テストを含む）"
  "$mvn_bin" $mvn_flags -q -f "$server/pom.xml" "${dap_profile[@]}" package
  mkdir -p "$example/server-dist"
  cp "$server/target/$jar.jar" "$example/server-dist/$jar.jar"

  if [[ "${SKIP_VSIX:-0}" == "1" ]]; then
    echo "[6/6] SKIP_VSIX=1 のため VSIX は組まない（$target）"
    return 0
  fi

  echo "[6/6] VSIX（$target）"
  (
    cd "$example"
    if [[ -f package-lock.json ]]; then npm ci --silent; else npm install --silent; fi
    npm run --silent compile
    mkdir -p target
    ./node_modules/.bin/vsce package --allow-missing-repository --out "target/$vsix"
  )
  echo "できた: $example/target/$vsix"
}

for target in $targets; do build_one "$target"; done
