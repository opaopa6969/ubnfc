#!/usr/bin/env bash
# examples/p4-vscode の VSIX を「生成物だけ」から組む。
#
#   1. 生成器を組む（LSP は opt-in の Maven profile `lsp`。既定ビルドには入らない）
#   2. ubnfc-java でパーサを生成（LSP feature とは無関係。出力はこの手順で変わらない）
#   3. ubnfc-lsp --emit-lsp で LSP 包みを別パッケージへ生成
#   4. extern token scanner を examples/p4-java から複製（読むだけ。編集しない）
#   5. サーバ fat jar（golden テストもここで走る）
#   6. npm ci + vsce package → examples/p4-vscode/target/*.vsix
#
# 環境変数: MVN（既定 mvn）、MAVEN_FLAGS（既定 -o。依存が未取得なら空にする）、SKIP_VSIX=1 で 6 を飛ばす。
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
example="$repo/examples/p4-vscode"
server="$example/server"
generated="$server/target/generated-sources/ubnfc"
ir="$repo/ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json"
mvn_bin=${MVN:-mvn}
mvn_flags=${MAVEN_FLAGS--o}
version=0.1.0-SNAPSHOT

echo "[1/6] 生成器（-Plsp）"
"$mvn_bin" $mvn_flags -q -f "$repo/pom.xml" -Plsp -DskipTests package

echo "[2/6] パーサ生成"
rm -rf "$generated"
mkdir -p "$generated"
java -cp "$repo/ubnfc-java/target/ubnfc-java-$version.jar" org.ubnfc.java.Main \
  --ir "$ir" --out "$generated" --package org.ubnfc.p4.generated --no-emit-driver

echo "[3/6] LSP 包み生成（opt-in: --emit-lsp）"
java -cp "$repo/ubnfc-lsp/target/ubnfc-lsp-$version.jar:$repo/ubnfc-java/target/ubnfc-java-$version.jar" \
  org.ubnfc.lsp.Main --emit-lsp --ir "$ir" --out "$generated" \
  --package org.ubnfc.p4.lsp --parser-package org.ubnfc.p4.generated \
  --language-id tinyexpression-p4 --scanner-class org.ubnfc.p4.P4Scanners \
  --server-name "TinyExpression P4 (ubnfc)" --server-version 0.1.0

echo "[4/6] extern token scanner"
mkdir -p "$generated/org/ubnfc/p4"
cp "$repo/examples/p4-java/src/main/java/org/ubnfc/p4/P4Scanners.java" "$generated/org/ubnfc/p4/P4Scanners.java"

echo "[5/6] サーバ jar（golden テストを含む）"
"$mvn_bin" $mvn_flags -q -f "$server/pom.xml" package
mkdir -p "$example/server-dist"
cp "$server/target/ubnfc-p4-lsp-server.jar" "$example/server-dist/ubnfc-p4-lsp-server.jar"

if [[ "${SKIP_VSIX:-0}" == "1" ]]; then
  echo "[6/6] SKIP_VSIX=1 のため VSIX は組まない"
  exit 0
fi

echo "[6/6] VSIX"
cd "$example"
if [[ -f package-lock.json ]]; then npm ci --silent; else npm install --silent; fi
npm run --silent compile
mkdir -p target
./node_modules/.bin/vsce package --allow-missing-repository --out "target/ubnfc-tinyexpression-p4-0.1.0.vsix"
echo "できた: $example/target/ubnfc-tinyexpression-p4-0.1.0.vsix"
