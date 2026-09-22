#!/usr/bin/env python3
"""初回 U の選択だけをレポートから読む。期待値は Java 実行からのみ取得する。"""
import collections
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]


def specifications_for(repo):
    resources = pathlib.Path(repo) / 'unlaxer-dsl/src/test/resources'
    report = json.loads((ROOT / 'docs/reports/2026-09-22-java-corpus.json').read_text())
    groups = collections.defaultdict(list)
    for row in report['cases']:
        if row['outcome'] == 'unobserved':
            groups[row['family'], row['corpus'], row['variant']].append(row['case_id'])
    # triage 後に判明した owned mixed-value span も、同じ既存 Java 観測で採取。
    groups['nested-captures', 'nested-captures/corpus.json', 'text-node-and-mixed-values'] = [
        'text-node-and-mixed-values/mixed-text-branch',
        'text-node-and-mixed-values/mixed-node-branch',
    ]
    old_specs = {s['slug']: s for s in json.loads(subprocess.check_output([
        sys.executable, str(ROOT / 'corpus/oracle/tools/specifications.py')]))}
    result = []
    for (family, corpus, variant), ids in sorted(groups.items()):
        spec = dict(feature=family, origin=corpus, variant=variant)
        if family == 'oracle':
            slug = pathlib.Path(corpus).stem
            spec.update(old_specs[slug])
            spec['cases'] = [c for c in spec['cases'] if c['id'] in ids]
            spec.update(slug='reobserved-' + slug, observation='oracle')
        else:
            # Gson が受理する catalog の非標準 \' は期待値側にしかない。
            # 原文を Java へ渡すため、ここでは catalog JSON を decode しない。
            if family == 'catalog':
                spec.update(slug='conformance-catalog-' + variant, observation='catalog',
                            fixtureName=variant, cases=[dict(id=variant, input='')])
                result.append(spec)
                continue
            data = json.loads((resources / corpus).read_text())
            if family == 'evolution':
                path = f'evolution/{variant}/Evolution.ubnf'
                spec.update(slug='evolution-v' + variant, observation='evolution',
                            grammar=(resources / path).read_text(), grammarPath=path,
                            cases=[dict(id=f'evolution/{i}', input=s) for i, s in enumerate(data)])
            elif family in ('cardinality', 'mixed-values'):
                name = 'Cardinality' if family == 'cardinality' else 'Mixed'
                source = (resources / family / (name + '.ubnf')).read_text()
                if family == 'cardinality':
                    quantifier = dict(zero='{ Item }', plus='Item+', bounded='Item{1,2}',
                                      unbounded='Item{1,}', separated="Item % ','")[variant]
                    source = source.replace('{ Item }', quantifier)
                elif variant == 'one':
                    source = source.replace(' | OtherRule', '')
                spec.update(slug=f'conformance-{family}-{variant}', observation=family,
                            grammar=source, cases=[dict(id=f'{name}.ubnf/{i}', input=c['input'])
                                                  for i, c in enumerate(data) if f'{name}.ubnf/{i}' in ids])
            elif family == 'scope-store':
                spec.update(slug='conformance-scope-store', observation='scope-store', cases=[])
                for fixture in data:
                    for run in fixture['runs']:
                        id_ = fixture['name'] + '/' + run['id']
                        if id_ in ids:
                            operations = run['operations']
                            def strip_expected(ops):
                                for op in ops:
                                    op.pop('expected', None)
                                    if 'operations' in op:
                                        strip_expected(op['operations'])
                            strip_expected(operations)
                            spec['cases'].append(dict(id=id_, input=fixture['source'], operations=operations))
            else:
                fixture = next(f for f in data if f['name'] == variant)
                spec.update(slug=f'conformance-{family}-{variant}', observation=family,
                            grammar=fixture['grammar'], cases=[])
                for key in ['queries', 'captureNames', 'valueSpanField', 'valueSpanListField']:
                    if key in fixture:
                        spec[key] = fixture[key]
                # rule 名のみ。metadata の期待値は採取 driver へ渡さない。
                if 'metadata' in fixture:
                    spec['metadata'] = [dict(rule=r['rule']) for r in fixture['metadata']]
                for row in fixture['cases']:
                    id_ = variant + '/' + row['id']
                    if id_ in ids:
                        case = dict(id=id_, input=row['input'])
                        if 'parentRollback' in row:
                            case['parentRollback'] = row['parentRollback']
                        spec['cases'].append(case)
        result.append(spec)
    return result


if __name__ == '__main__':
    print(json.dumps(specifications_for(sys.argv[1]), ensure_ascii=False, indent=2))
