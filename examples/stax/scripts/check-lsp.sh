#!/usr/bin/env bash
# 通常の workspace CI から実行する。VS Code / npm / VSIX の公開は不要。
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
module=$(cd "$(dirname "$0")/.." && pwd)
bash "$module/scripts/build-lsp.sh"
cmp "$module/vscode/syntaxes/stax.tmLanguage.json" "$module/lsp/golden/stax.tmLanguage.json"
python3 "$module/scripts/lsp-protocol.py" --jar "$module/vscode/server-dist/ubnfc-stax-lsp-server.jar"
python3 "$module/scripts/lsp-recovery.py"
