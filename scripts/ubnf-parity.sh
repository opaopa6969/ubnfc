#!/usr/bin/env bash
# UBNF の VSIX（examples/ubnf-vscode）と旧 unlaxer-dsl/ubnf-vscode の診断を、
# 同じ 24 個の .ubnf 文法（crates/ubnfc-front/tests/pipeline_golden.rs の FILES）で突き合わせる。
#
# sibling repo は読むだけ。旧サーバの jar は scratch へ複製して動かす。
# 出力: examples/ubnf-vscode/evidence/parity-{old,new}.json と fixtures.tsv
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
unlaxer=${UNLAXER_PARSER_DIR:-$(cd "$repo/../unlaxer-parser" && pwd)}
tiny=${TINYEXPRESSION_DIR:-$(cd "$repo/../tinyexpression" && pwd)}
evidence="$repo/examples/ubnf-vscode/evidence"
work=$(mktemp -d -t ubnf-lsp-parity.XXXXXXXX)
trap 'rm -rf -- "$work"' EXIT
mkdir -p "$work/fixtures" "$evidence"

# pipeline_golden.rs の FILES をそのまま資材にする（24 文法）。
python3 - "$repo" "$unlaxer" "$tiny" "$work/fixtures" "$evidence/fixtures.tsv" <<'PY'
import re, sys, pathlib, shutil
repo, unlaxer, tiny, out, manifest = (pathlib.Path(p) for p in sys.argv[1:6])
source = (repo / "crates/ubnfc-front/tests/pipeline_golden.rs").read_text(encoding="utf-8")
block = source[source.index("const FILES"):]
block = block[:block.index("];")]
rows = []
for relative in re.findall(r'"([^"]+)"', block):
    base = tiny if relative.startswith("tools/") else unlaxer
    name = relative.replace("/", "__")
    shutil.copy(base / relative, out / name)
    rows.append(f"{name}\t{base / relative}")
manifest.write_text("\n".join(rows) + "\n", encoding="utf-8")
print(f"{len(rows)} 文法")
PY

old_jar="$unlaxer/unlaxer-dsl/ubnf-vscode/server-dist/ubnf-lsp-server.jar"
if [[ -f "$old_jar" ]]; then
  cp "$old_jar" "$work/old.jar"
  python3 "$repo/scripts/lsp-probe.py" --jar "$work/old.jar" --language-id ubnf --glob '*.ubnf' \
    --fixtures "$work/fixtures" --out "$evidence/parity-old.json"
else
  echo "旧サーバの jar が無いので新側だけ採る: $old_jar" >&2
fi
python3 "$repo/scripts/lsp-probe.py" --jar "$repo/examples/ubnf-vscode/server-dist/ubnfc-ubnf-lsp-server.jar" \
  --language-id ubnf --glob '*.ubnf' --fixtures "$work/fixtures" --out "$evidence/parity-new.json"
echo "できた: $evidence/parity-{old,new}.json"
