#!/usr/bin/env python3
"""VSIX 内の設定・起動コード・生成サーバ・生成対応表を照合する。"""
import io
import json
import sys
import zipfile

with zipfile.ZipFile(sys.argv[1]) as archive:
    package = json.loads(archive.read('extension/package.json'))
    assert package['name'] == 'ubnfc-stax'
    assert package['activationEvents'] == ['onLanguage:stax']
    assert package['contributes']['languages'][0]['extensions'] == ['.stax']
    assert 'stax' in archive.read('extension/out/extension.js').decode()
    assert json.loads(archive.read('extension/syntaxes/stax.tmLanguage.json'))['scopeName'] == 'source.stax'
    manifest = json.loads(archive.read('extension/server-dist/generation.json'))
    assert 'org/example/stax/lsp/ServerMain.java' in manifest['files']
    with zipfile.ZipFile(io.BytesIO(archive.read('extension/server-dist/ubnfc-stax-lsp-server.jar'))) as jar:
        assert 'org/example/stax/lsp/ServerMain.class' in jar.namelist()
        assert 'org/example/stax/StaxParser.class' in jar.namelist()
    assert not any(p.startswith('extension/server/') or p.startswith('extension/src/') for p in archive.namelist())
print('VSIX の設定・起動コード・生成サーバ・生成対応表を確認')
