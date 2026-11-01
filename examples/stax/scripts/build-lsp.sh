#!/usr/bin/env bash
# stax の既存注釈からパーサ・LSP・TextMate 文法を生成する。
# --vsix のときだけ npm ci とローカル VSIX 梱包まで行う（publish はしない）。
set -euo pipefail
export RUSTUP_TOOLCHAIN=stable
export PYTHONDONTWRITEBYTECODE=1
module=$(cd "$(dirname "$0")/.." && pwd)
root=$(cd "$module/../.." && pwd)
cd "$root"
example="$module/vscode"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mode=${1:-}
[[ -z "$mode" || "$mode" == --vsix ]] || { echo '使い方: build-lsp.sh [--vsix]' >&2; exit 2; }

(cd "$root" && cargo build -q --offline --target-dir "$root/target" --bin ubnfc)
mvn ${MAVEN_FLAGS--o} -q -f "$root/pom.xml" -Plsp -DskipTests package
ubnfc="$root/target/debug/ubnfc"
"$ubnfc" validate --grammar "$module/stax.ubnf" --strict
generated="$example/server/target/generated-sources/ubnfc"
# 自分の生成ディレクトリだけを置換する。
rm -rf "$generated"
mkdir -p "$generated" "$example/syntaxes" "$example/server-dist"
"$ubnfc" ir --grammar "$module/stax.ubnf" --out "$work/stax.ir.json"
java -cp "$root/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar" org.unlaxer.ubnfc.java.Main \
  --ir "$work/stax.ir.json" --out "$generated" --package org.example.stax --no-emit-driver
python3 - "$generated" "$work/parser.json" <<'PY'
import hashlib, json, pathlib, sys
directory = pathlib.Path(sys.argv[1])
pathlib.Path(sys.argv[2]).write_text(json.dumps({str(p.relative_to(directory)):
    hashlib.sha256(p.read_bytes()).hexdigest() for p in directory.rglob('*.java')}))
PY
java -cp "$root/ubnfc-lsp/target/ubnfc-lsp-0.1.0-SNAPSHOT.jar:$root/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar" \
  org.unlaxer.ubnfc.lsp.Main --emit-lsp --ir "$work/stax.ir.json" --out "$generated" \
  --package org.example.stax.lsp --parser-package org.example.stax --language-id stax \
  --server-name 'stax (ubnfc)' --server-version 0.1.0 \
  --emit-tmlanguage "$example/syntaxes/stax.tmLanguage.json"
python3 - "$generated" "$work/parser.json" <<'PY'
import hashlib, json, pathlib, sys
directory = pathlib.Path(sys.argv[1])
for name, digest in json.loads(pathlib.Path(sys.argv[2]).read_text()).items():
    assert hashlib.sha256((directory / name).read_bytes()).hexdigest() == digest, name
PY
python3 "$module/scripts/lsp-manifest.py" "$root" "$work/stax.ir.json" "$generated" \
  > "$example/server-dist/generation.json"
mvn ${MAVEN_FLAGS--o} -q -f "$example/server/pom.xml" package
cp "$example/server/target/ubnfc-stax-lsp-server.jar" "$example/server-dist/"
if [[ "$mode" == --vsix ]]; then
  (cd "$example" && npm ci --silent && npm run --silent compile && mkdir -p target && \
    ./node_modules/.bin/vsce package --allow-missing-repository --out target/ubnfc-stax-0.1.0.vsix)
  python3 "$module/scripts/check-vsix.py" "$example/target/ubnfc-stax-0.1.0.vsix"
fi
echo "生成完了: $example/server-dist/ubnfc-stax-lsp-server.jar"
