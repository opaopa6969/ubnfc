#!/usr/bin/env bash
set -euo pipefail
module=$(cd "$(dirname "$0")/.." && pwd)
repo=$(cd "$module/../.." && pwd)
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
if [[ -n "${UBNFC_GENERATOR_JAR:-}" ]]; then
  jar=$UBNFC_GENERATOR_JAR
else
  # 作業中に共有 jar が更新されても、同じ checkout の生成器で再現する。
  # ビルド出力は一時コピー内へ置く。リポジトリの生成器ソースは変更しない。
  mkdir -p "$scratch/ubnfc-java/runtime-template" "$scratch/ir"
  cp "$repo/ubnfc-java/pom.xml" "$scratch/ubnfc-java/pom.xml"
  cp -R "$repo/ubnfc-java/src" "$scratch/ubnfc-java/src"
  cp -R "$repo/ubnfc-java/runtime-template/src" "$scratch/ubnfc-java/runtime-template/src"
  cp -R "$repo/ir/schema" "$scratch/ir/schema"
  mvn -o -q -f "$scratch/ubnfc-java/pom.xml" -Dmaven.test.skip=true package
  jar="$scratch/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar"
fi
java -cp "$jar" org.ubnfc.java.Main \
  --ir "$repo/ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json" \
  --out "$scratch/generated" --package org.ubnfc.p4.generated
case "${1:---check}" in
  --check) diff -qr "$module/src/main/java/org/ubnfc/p4/generated" "$scratch/generated/org/ubnfc/p4/generated" ;;
  --write) cp -R "$scratch/generated/org/ubnfc/p4/generated/." "$module/src/main/java/org/ubnfc/p4/generated/" ;;
  *) echo '使用法: generate.sh [--check|--write]' >&2; exit 2 ;;
esac
