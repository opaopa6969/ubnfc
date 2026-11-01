#!/usr/bin/env python3
"""生成元と各生成ファイルの対応を、時刻・絶対パスを含めず記録する。"""
import hashlib
import json
from pathlib import Path
import sys

root, ir, generated = map(Path, sys.argv[1:])
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

manifest = {
    'grammar': 'examples/stax/stax.ubnf',
    'grammarSha256': sha(root / 'examples/stax/stax.ubnf'),
    'irSha256': sha(ir),
    'parserPackage': 'org.example.stax',
    'lspPackage': 'org.example.stax.lsp',
    'files': {str(p.relative_to(generated)): sha(p) for p in sorted(generated.rglob('*.java'))},
    'textmateSha256': sha(root / 'examples/stax/vscode/syntaxes/stax.tmLanguage.json'),
}
print(json.dumps(manifest, ensure_ascii=False, indent=2))
