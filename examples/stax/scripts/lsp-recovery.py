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
      "skippedSpan", List.of(v.skippedSpan().start(), v.skippedSpan().end()))).toList());
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
                    uri = 'file:///recovery.stax'
                    client.notify('textDocument/didOpen', {'textDocument': {'uri': uri,
                                  'languageId': 'stax', 'version': 1, 'text': cases['broken-then-word']}})
                    message = client.wait(lambda m: m.get('method') == 'textDocument/publishDiagnostics', 30)
                    assert message is not None
                    report[variant]['lspDiagnostics'] = message['params']['diagnostics']
                    assert report[variant]['lspDiagnostics'] == [], message
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
        print('回復配置: 原文、WordDef の横取り、TopLevel の回復と後続宣言を確認')
    finally:
        shutil.rmtree(work)


if __name__ == '__main__':
    main()
