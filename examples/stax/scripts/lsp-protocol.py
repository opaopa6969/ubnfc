#!/usr/bin/env python3
"""生成 fat jar と stdio JSON-RPC で通信し、機能と既知境界を golden 照合する。"""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

MODULE = Path(__file__).resolve().parents[1]
ROOT = MODULE.parents[1]
spec = importlib.util.spec_from_file_location('probe', ROOT / 'scripts/lsp-probe.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


def position(text, needle, occurrence=0, within=0):
    start = -1
    for _ in range(occurrence + 1):
        start = text.index(needle, start + 1)
    head = text[:start + within]
    return {'line': head.count('\n'), 'character': len(head.rsplit('\n', 1)[-1].encode('utf-16-le')) // 2}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--jar', required=True)
    parser.add_argument('--record', action='store_true')
    args = parser.parse_args()
    client = probe.Client(['java', '-jar', str(Path(args.jar).resolve())])
    report = {}

    def request(method, params):
        response = client.request(method, params, timeout=30)
        assert response is not None, f'{method}: 応答が無い'
        assert 'error' not in response, response
        return response.get('result')

    def opened(name, text, version=1):
        uri = (MODULE / 'lsp/fixtures' / name).as_uri()
        if version == 1:
            client.notify('textDocument/didOpen', {'textDocument': {
                'uri': uri, 'languageId': 'stax', 'version': version, 'text': text}})
        else:
            client.notify('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version},
                                                    'contentChanges': [{'text': text}]})
        message = client.wait(lambda m: m.get('method') == 'textDocument/publishDiagnostics'
                              and m['params']['uri'] == uri, timeout=30)
        assert message is not None, '診断通知が無い'
        diagnostics = message['params']['diagnostics']
        assert not any(d.get('code') == 'internal' for d in diagnostics), diagnostics
        return uri, diagnostics

    def at(method, uri, pos):
        return request('textDocument/' + method, {'textDocument': {'uri': uri}, 'position': pos})

    try:
        init = request('initialize', {'processId': None, 'rootUri': None, 'capabilities': {}})
        client.notify('initialized', {})
        report['initialize'] = init
        caps = init['capabilities']
        assert caps['textDocumentSync'] == 1
        assert caps['definitionProvider'] and caps['hoverProvider'] and caps['completionProvider']
        text = (MODULE / 'lsp/fixtures/navigation.stax').read_text()
        uri, report['diagnostics'] = opened('navigation.stax', text)
        assert len(report['diagnostics']) == 1 and report['diagnostics'][0]['severity'] == 2
        assert 'missing' in report['diagnostics'][0]['message']
        assert report['diagnostics'][0]['range']['start'] == position(text, 'missing')
        library = (MODULE / 'lsp/fixtures/library.stax').read_text()
        _, lib_diagnostics = opened('library.stax', library)
        assert not lib_diagnostics
        for label, needle, occurrence, within in [
            ('word', 'square', 1, 1), ('type', 'Count', 1, 1),
            ('local', 'local', 1, 1), ('import', 'imported', 1, 1),
            ('field', '.x', 0, 1),
        ]:
            found = at('definition', uri, position(text, needle, occurrence, within))
            assert len(found) == 1, (label, found)
            assert found[0]['uri'] == uri, (label, found)
            report['definition-' + label] = found
        for label, target in [('word', 'square'), ('type', 'Count'), ('local', 'local'),
                              ('import', 'imported'), ('field', 'x:')]:
            assert report['definition-' + label][0]['range']['start'] == position(text, target)
        # use の実ファイルを開いても、ジャンプ先はこの文書の ImportName のまま。
        assert report['definition-import'][0]['range']['start']['line'] == 3
        for label, needle, occurrence in [('word', 'square', 0), ('primitive', 'dup', 0), ('reference', 'square', 1)]:
            hover = at('hover', uri, position(text, needle, occurrence, 1))
            assert hover and hover['contents']['value']
            report['hover-' + label] = hover
        assert '組込み語' in report['hover-primitive']['contents']['value']
        assert '語の定義' in report['hover-word']['contents']['value']

        for label, partial in [('type', ': edit ( '), ('body', ': edit ( -- ) '), ('field', text[:text.index('.x') + 1])]:
            edit_uri, diagnostics = opened('completion-' + label + '.stax', partial)
            items = at('completion', edit_uri, position(partial + '|', '|'))
            if isinstance(items, dict):
                items = items['items']
            report['completion-' + label] = items
        type_labels = {v['label'] for v in report['completion-type']}
        body_labels = {v['label'] for v in report['completion-body']}
        assert 'Int' in type_labels and 'dup' not in type_labels
        assert 'dup' in body_labels and 'Int' not in body_labels

        shadow = (MODULE / 'lsp/fixtures/shadow.stax').read_text()
        shadow_uri, diagnostics = opened('shadow.stax', shadow)
        assert not diagnostics
        ambiguous = at('definition', shadow_uri, position(shadow, 'x drop', 1))
        assert len(ambiguous) == 2, ambiguous
        report['boundary-shadow-definition'] = ambiguous
        outside = at('completion', shadow_uri, position(shadow, ': outside ( -- ) ', within=len(': outside ( -- ) ')))
        assert any(v['label'] == 'x' for v in outside), outside
        report['boundary-outside-completion'] = outside

        fields = (MODULE / 'lsp/fixtures/fields.stax').read_text()
        fields_uri, diagnostics = opened('fields.stax', fields)
        report['boundary-field-diagnostics'] = diagnostics
        field_targets = at('definition', fields_uri, position(fields, '.x', within=1))
        assert len(field_targets) == 2, field_targets
        report['boundary-field-definition'] = field_targets

        broken = ': before ( -- ) ;\n: broken ( '
        edit_uri, report['edit-broken'] = opened('editing.stax', broken)
        assert any(v['severity'] == 1 for v in report['edit-broken'])
        report['edit-prefix-definition'] = at('definition', edit_uri, position(broken, 'before', within=1))
        # 全文が失敗すると、主失敗位置が0となり既存の前半再解析が働かない例。
        assert report['edit-prefix-definition'] == []
        repaired = ': before ( -- ) ;\n: repaired ( -- ) before ;'
        _, report['edit-repaired'] = opened('editing.stax', repaired, version=2)
        assert not report['edit-repaired']
        report['edit-after-definition'] = at('definition', edit_uri, position(repaired, 'before', 1, 1))
        assert report['edit-after-definition']
        client.notify('textDocument/didClose', {'textDocument': {'uri': edit_uri}})
        closed = client.wait(lambda m: m.get('method') == 'textDocument/publishDiagnostics'
                             and m['params']['uri'] == edit_uri, timeout=30)
        assert closed and closed['params']['diagnostics'] == []
        report['after-close-definition'] = at('definition', edit_uri, position(repaired, 'before', 1, 1))
        assert report['after-close-definition'] == []

        # 絶対 URI だけを fixture 名へ正規化し、range・診断・候補は加工しない。
        encoded = json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True)
        encoded = encoded.replace((MODULE / 'lsp/fixtures').as_uri() + '/', 'fixture:///') + '\n'
        golden = MODULE / 'lsp/golden/protocol.json'
        if args.record:
            golden.write_text(encoded)
        else:
            assert encoded == golden.read_text(), 'LSP protocol golden が変化した'
        print('LSP protocol: 定義5種、hover3種、補完3位置、scope境界、編集・close を確認')
    finally:
        client.close()


if __name__ == '__main__':
    main()
