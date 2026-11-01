#!/usr/bin/env python3
"""一般語列の受理と、不変の外部語彙効果・rollback を両 backend で検証する。"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
from crosscheck import canonical

MODULE = Path(__file__).resolve().parents[1]
ROOT = MODULE.parents[1]


def run(*args, **kwargs):
    result = subprocess.run(list(map(str, args)), text=True, capture_output=True, timeout=180, **kwargs)
    assert result.returncode == 0, f'{args}:\n{result.stdout}\n{result.stderr}'
    return result.stdout


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--record', action='store_true')
    args = parser.parse_args()
    work = Path(run('mktemp', '-d').strip())
    report = {'wordStream': [], 'vocabulary': []}
    try:
        ubnfc = ROOT / 'target/debug/ubnfc'
        rust = ROOT / 'target/debug/ubnfc-rust'
        java = ROOT / 'ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar'
        generated = {}
        for grammar in ('word-stream', 'vocabulary'):
            source = MODULE / 'boundaries' / (grammar + '.ubnf')
            run(ubnfc, 'validate', '--grammar', source, '--strict')
            ir = work / (grammar + '.ir.json')
            run(ubnfc, 'ir', '--grammar', source, '--out', ir)
            rust_out, java_out = work / (grammar + '-rust'), work / (grammar + '-java')
            run(rust, '--ir', ir, '--out', rust_out)
            run('java', '-cp', java, 'org.unlaxer.ubnfc.java.Main', '--ir', ir,
                '--out', java_out, '--package', 'org.example.boundary')
            sources = sorted(java_out.rglob('*.java'))
            if grammar == 'vocabulary':
                sources.append(MODULE / 'boundaries/VocabularyProbe.java')
            listing = work / (grammar + '-sources.txt')
            listing.write_text('\n'.join(str(p) for p in sources))
            classes = work / (grammar + '-classes')
            classes.mkdir()
            run('javac', '--release', '21', '-encoding', 'UTF-8', '-d', classes, '@' + str(listing))
            generated[grammar] = (ir, rust_out, classes)

        ir, rust_out, classes = generated['word-stream']
        run('cargo', 'build', '-q', '--offline', '--features', 'json', '--bin', 'driver', cwd=rust_out)
        cases = json.loads((MODULE / 'boundaries/cases.json').read_text())
        inputs = [c['source'] for c in cases['forthReferences']] + ['日本語 >> 2dup!']
        requests = ''.join(json.dumps({'id': n, 'grammar': 'WordStream', 'entry': None,
                       'input': source, 'options': {'lexical': True, 'scope': True}}, ensure_ascii=False)
                       + '\n' for n, source in enumerate(inputs))
        outputs = [run(rust_out / 'target/debug/driver', input=requests),
                   run('java', '-cp', classes, 'org.example.boundary.driver.Driver', input=requests)]
        rows = [[json.loads(line) for line in text.splitlines()] for text in outputs]
        for n, source in enumerate(inputs):
            left, right = rows[0][n], rows[1][n]
            assert left['id'] == right['id'] == n
            assert left['accepted'] and left['allConsumed'] and right['accepted'] and right['allConsumed'], (source, left, right)
            assert left.get('mappingError') is None and right.get('mappingError') is None
            assert canonical(left['ast']) == canonical(right['ast'])
            assert left['ast']['type'] == 'Words'
            words = [v['fields']['text'] for v in left['ast']['fields']['items']]
            assert words == source.split(), (source, words)
            assert left['declarations'] == right['declarations'] == []
            report['wordStream'].append({'input': source, 'words': words, 'declarations': []})

        ir, rust_out, classes = generated['vocabulary']
        data = json.loads(ir.read_text())
        seed = next(t for t in data['semantics']['tokens'] if t['spec']['kind'] == 'extern')
        assert seed['memoSafety']['success'] is False and seed['memoSafety']['failure'] is False
        unsafe = [e for e in data['plan']['memo']['entries'] if e['reason'] == 'unsafeExternClosure']
        assert unsafe and all(not e['cacheSuccess'] and not e['cacheFailure'] for e in unsafe)
        report['externMemoExcluded'] = True
        lib = work / 'libboundary.rlib'
        run('rustc', '--edition=2021', '--crate-name', 'boundary', '--crate-type=rlib',
            rust_out / 'mod.rs', '-o', lib)
        probe = work / 'vocabulary-probe'
        run('rustc', '--edition=2021', MODULE / 'boundaries/vocabulary_probe.rs',
            '--extern', 'boundary=' + str(lib), '-o', probe)
        for vocabulary in ('external', 'other'):
            for source in ('+ external', '+ missing', 'external'):
                input_file = work / 'input.txt'
                input_file.write_text(source)
                expected = {'ok': True, 'declarations': [vocabulary] if source.startswith('+') else [],
                            'scopeDepth': 0, 'warnings': int(not source.startswith('+') or source.split()[-1] != vocabulary)}
                for memo in ('true', 'false'):
                    java_result = json.loads(run('java', '-cp', classes, 'VocabularyProbe', input_file,
                                                 vocabulary, memo, seed['spec']['logicalId']))
                    assert java_result == expected, (source, java_result, expected)
                    for cache in ('true', 'false'):
                        rust_result = json.loads(run(probe, input_file, vocabulary, memo, cache))
                        assert rust_result == expected, (source, rust_result, expected)
                report['vocabulary'].append({'input': source, 'snapshot': vocabulary, **expected})

        golden = MODULE / 'boundaries/generated.json'
        encoded = json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + '\n'
        if args.record:
            golden.write_text(encoded)
        else:
            assert encoded == golden.read_text(), '境界レポートの生成パーサ観測が変化した'
        print('境界: 一般語列4件、外部語彙6件、両backend・memo切替・Rust字句cache切替で一致')
    finally:
        shutil.rmtree(work)


if __name__ == '__main__':
    main()
