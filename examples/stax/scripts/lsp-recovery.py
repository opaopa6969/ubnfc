#!/usr/bin/env python3
"""原文を変更せず3種類の文法を生成し、回復配置の境界を Java API で実測する。"""
import argparse
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess

MODULE = Path(__file__).resolve().parents[1]
ROOT = MODULE.parents[1]
spec = importlib.util.spec_from_file_location('protocol', MODULE / 'scripts/lsp-protocol.py')
protocol = importlib.util.module_from_spec(spec)
spec.loader.exec_module(protocol)
PROBE = '''
import java.util.*;
import java.nio.file.*;
import org.example.stax.StaxParser;
import org.example.stax.api.*;
public class RecoveryProbe {
  public static void main(String[] args) throws Exception {
    String text = Files.readString(Path.of(args[0]));
    var r = StaxParser.parseEntry("Stax", "Program", text, new ParseOptions(false, true, true, true, Map.of()));
    var out = new LinkedHashMap<String,Object>();
    out.put("ok", r.ok());
    out.put("consumed", r.consumedCp());
    out.put("declarations", r.scope().declarations().stream().map(d -> d.name()).toList());
    out.put("recoveries", r.recoveries().stream().map(v -> Map.of(
      "rule", v.ruleId(), "span", List.of(v.span().start(), v.span().end()),
      "skippedSpan", List.of(v.skippedSpan().start(), v.skippedSpan().end()),
      "mode", v.mode(), "expected", v.diagnostic().expected())).toList());
    out.put("diagnosticKind", r.diagnostics().kind());
    System.out.println(Json.write(out));
  }
}
'''


def run(*args, **kwargs):
    return subprocess.run(list(map(str, args)), check=True, text=True, capture_output=True, timeout=120, **kwargs)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--record', action='store_true')
    args = parser.parse_args()
    work = Path(subprocess.check_output(['mktemp', '-d'], text=True).strip())
    report = {}
    try:
        source = (MODULE / 'stax.ubnf').read_text()
        cases = {
            'valid-struct': 'struct Point { x: Int }\n: main ( -- ) ;\n',
            'broken-then-word': ': broken ( -- ) [ ;\n: after ( -- ) ;\n',
        }
        for name, source_text in cases.items():
            (work / (name + '.stax')).write_text(source_text)
        (work / 'RecoveryProbe.java').write_text(PROBE)
        ubnfc = ROOT / 'target/debug/ubnfc'
        emitter = ROOT / 'ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar'
        for variant, rule in [('original', None), ('word', 'WordDef'), ('top', 'TopLevel')]:
            grammar = source if rule is None else source.replace(
                '  ' + rule + ' ::=', "  @recovery(sync=';')\n  " + rule + ' ::=')
            path = work / (variant + '.ubnf')
            path.write_text(grammar)
            validation = subprocess.run([str(ubnfc), 'validate', '--grammar', str(path), '--strict'],
                                        text=True, capture_output=True, timeout=30)
            report[variant] = {'strictExit': validation.returncode,
                               'choiceWarning': 'W-RECOVERY-IN-CHOICE' in validation.stdout + validation.stderr}
            ir = work / (variant + '.json')
            run(ubnfc, 'ir', '--grammar', path, '--out', ir)
            generated = work / variant
            run('java', '-cp', emitter, 'org.unlaxer.ubnfc.java.Main', '--ir', ir,
                '--out', generated, '--package', 'org.example.stax', '--no-emit-driver')
            files = work / (variant + '-sources.txt')
            files.write_text('\n'.join(str(p) for p in sorted(generated.rglob('*.java')))
                             + '\n' + str(work / 'RecoveryProbe.java') + '\n')
            classes = work / (variant + '-classes')
            classes.mkdir()
            run('javac', '--release', '21', '-encoding', 'UTF-8', '-d', classes, '@' + str(files))
            for name in cases:
                output = run('java', '-cp', classes, 'RecoveryProbe', work / (name + '.stax'))
                report[variant][name] = json.loads(output.stdout)
            if variant == 'top':
                # 同じ IR から LSP も生成し、回復領域が診断へ写るかを調べる。
                jars = ':'.join(str(ROOT / f'{m}/target/{m}-0.1.0-SNAPSHOT.jar')
                                for m in ('ubnfc-lsp', 'ubnfc-java'))
                run('java', '-cp', jars, 'org.unlaxer.ubnfc.lsp.Main', '--emit-lsp',
                    '--ir', ir, '--out', generated, '--package', 'org.example.stax.lsp',
                    '--parser-package', 'org.example.stax', '--language-id', 'stax')
                files.write_text('\n'.join(str(p) for p in sorted(generated.rglob('*.java'))))
                jar = MODULE / 'vscode/server-dist/ubnfc-stax-lsp-server.jar'
                run('javac', '--release', '21', '-encoding', 'UTF-8', '-cp', jar,
                    '-d', classes, '@' + str(files))
                client = protocol.probe.Client(['java', '-cp', str(classes) + ':' + str(jar),
                                                'org.example.stax.lsp.ServerMain'])
                try:
                    initialized = client.request('initialize', {'capabilities': {}}, timeout=30)
                    assert initialized and 'error' not in initialized
                    client.notify('initialized', {})
                    def request(method, uri, pos):
                        response = client.request('textDocument/' + method,
                            {'textDocument': {'uri': uri}, 'position': pos}, timeout=30)
                        assert response is not None and 'error' not in response, response
                        return response['result']

                    def opened(uri, text, version=1):
                        if version == 1:
                            client.notify('textDocument/didOpen', {'textDocument': {'uri': uri,
                                'languageId': 'stax', 'version': version, 'text': text}})
                        else:
                            client.notify('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version},
                                'contentChanges': [{'text': text}]})
                        message = client.wait(lambda m: m.get('method') == 'textDocument/publishDiagnostics'
                            and m['params']['uri'] == uri, 30)
                        assert message is not None
                        return message['params']

                    recovery_cases = {
                        'original': cases['broken-then-word'],
                        'reference': cases['broken-then-word'] + ': use_after ( -- ) after ;\n',
                        'unicode-multiple': ('/* 😀 */ : before ( -- ) ; : broken ( -- ) [ ; '
                            ': after ( -- ) ; : use_after ( -- ) after ;\n'
                            ': broken2 ( -- ) [ ;\n: last ( -- ) ;\n'),
                    }
                    lsp_report = {}
                    for name, text in recovery_cases.items():
                        path = work / ('lsp-' + name + '.stax')
                        path.write_text(text)
                        observed = json.loads(run('java', '-cp', classes, 'RecoveryProbe', path).stdout)
                        assert observed['ok'] and observed['recoveries'], observed
                        uri = 'file:///recovery-' + name + '.stax'
                        published = opened(uri, text)
                        diagnostics = published['diagnostics']
                        assert len(diagnostics) == len(observed['recoveries']), published
                        assert len(diagnostics) == (2 if name == 'unicode-multiple' else 1)
                        for diagnostic, recovery in zip(diagnostics, observed['recoveries']):
                            def at_cp(cp):
                                return protocol.position(text[:cp] + '|', '|')
                            expected_range = dict(zip(('start', 'end'), map(at_cp, recovery['span'])))
                            skipped_range = dict(zip(('start', 'end'), map(at_cp, recovery['skippedSpan'])))
                            assert diagnostic['severity'] == 1 and diagnostic['code'] == 'recovery', diagnostic
                            assert diagnostic['range'] == expected_range
                            assert diagnostic['relatedInformation'][0]['location'] == {'uri': uri, 'range': expected_range}
                            data = diagnostic['data']
                            assert data['origin'] == 'recovery' and data['ruleId'] == recovery['rule']
                            assert data['expected'] == recovery['expected'] and data['expected']
                            assert "']'" in data['expected']
                            assert data['mode'] == recovery['mode'] and data['skippedRange'] == skipped_range
                        target_pos = protocol.position(text, 'after', 2 if name != 'original' else 0, 1)
                        targets = request('definition', uri, target_pos)
                        assert len(targets) == 1 and targets[0]['uri'] == uri
                        assert targets[0]['range']['start'] == protocol.position(text, 'after')
                        # 壊れた定義を削り、同じ URI の全文修正で回復診断を消す。
                        repaired = ': after ( -- ) ;\n: use_after ( -- ) after ;'
                        fixed = opened(uri, repaired, version=2)
                        assert fixed['diagnostics'] == [], fixed
                        fixed_targets = request('definition', uri, protocol.position(repaired, 'after', 2, 1))
                        assert fixed_targets and fixed_targets[0]['range']['start'] == protocol.position(repaired, 'after')
                        # 修正で削除した before / last / broken の宣言が残らない。
                        empty = opened(uri, '', version=3)
                        assert empty['diagnostics'] == []
                        assert request('definition', uri, target_pos) == []
                        client.notify('textDocument/didClose', {'textDocument': {'uri': uri}})
                        closed = client.wait(lambda m: m.get('method') == 'textDocument/publishDiagnostics'
                            and m['params']['uri'] == uri, 30)
                        assert closed and closed['params']['diagnostics'] == []
                        assert request('definition', uri, target_pos) == []
                        lsp_report[name] = {'parse': observed, 'published': published,
                            'definition': targets, 'repaired': fixed, 'repairedDefinition': fixed_targets,
                            'empty': empty, 'closed': closed['params']}
                    report[variant]['lsp'] = lsp_report
                finally:
                    client.close()
        original, word, top = (report[k] for k in ('original', 'word', 'top'))
        assert original['strictExit'] == top['strictExit'] == 0
        assert word['strictExit'] == 5 and word['choiceWarning']
        assert original['valid-struct']['ok'] and original['valid-struct']['declarations']
        assert not original['broken-then-word']['ok']
        assert word['valid-struct']['ok'] and word['valid-struct']['recoveries']
        assert word['valid-struct']['declarations'] != original['valid-struct']['declarations']
        assert top['valid-struct'] == original['valid-struct']
        assert top['broken-then-word']['ok'] and top['broken-then-word']['recoveries']
        assert 'after' in top['broken-then-word']['declarations']
        golden = MODULE / 'lsp/golden/recovery.json'
        encoded = json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + '\n'
        if args.record:
            golden.write_text(encoded)
        else:
            assert encoded == golden.read_text(), '回復の実測 golden が変化した'
        print('回復配置: 原文、WordDef の横取り、TopLevel の全回復診断・UTF-16・後続宣言・修正と close を確認')
    finally:
        shutil.rmtree(work)


if __name__ == '__main__':
    main()
