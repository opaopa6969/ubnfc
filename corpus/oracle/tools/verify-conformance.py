#!/usr/bin/env python3
"""追加実走値の形・入力網羅・既存記録との一致と、別実行との byte 一致を検証する。"""
import collections
import json
import pathlib
import re
import sys

sys.dont_write_bytecode = True
from conformance_specifications import specifications_for
from verify import canonical, require

ROOT = pathlib.Path(__file__).resolve().parent.parent
REPO = pathlib.Path('/home/opa/work/unlaxer-workspace/unlaxer-parser')
COUNTS = {'evolution': 148, 'cardinality': 20, 'mixed-values': 5, 'scope-store': 9,
          'catalog': 2, 'scope-annotations': 6, 'nested-captures': 4, 'oracle': 36}


def observations(operations):
    for op in operations:
        if op['op'] == 'observe':
            yield op
        yield from observations(op.get('operations', []))


def verify(directory, repo):
    specs = specifications_for(repo)
    names = {s['slug'] + '.json' for s in specs}
    actual_names = {p.name for p in directory.glob('*.json')
                    if p.name != 'divergences.json'
                    and json.loads(p.read_text()).get('collection') == 'triage-2026-09-22'}
    require(actual_names == names, '追加採取ファイルの不足/過剰')
    counts = collections.Counter()
    accepted = collections.Counter()
    snapshots = 0
    for spec in specs:
        path = directory / (spec['slug'] + '.json')
        data = json.loads(path.read_text())
        kind = spec['observation']
        require(data['stage'] == 'executed', f'{path}: 未実行')
        require(data['observation'] == kind, f'{path}: 観測種別')
        require(re.fullmatch('[0-9a-f]{40}', data['source']), f'{path}: SHA')
        require(data['origin'] == spec['origin'] and data['variant'] == spec['variant'], f'{path}: 来歴')
        if 'grammar' in spec:
            require(data['grammar'] == spec['grammar'], f'{path}: 文法の改変')
        require([(r['id'], r['input']) for r in data['cases']] ==
                [(r['id'], r['input']) for r in spec['cases']], f'{path}: 入力列')
        if kind == 'oracle':
            original = json.loads((ROOT / pathlib.Path(spec['origin']).name).read_text())
            original_rows = {c['id']: c for c in original['cases']}
            for row in data['cases']:
                require(row == original_rows[row['id']], f'{path}: 既存観測との差（補正せず調査する）')
        for row in data['cases']:
            counts[kind] += 1
            require(not any(k in row for k in ['canonicalError', 'parseError', 'diagnosticError']), f'{path}: 採取例外')
            if kind == 'scope-store':
                require(set(row) == {'id', 'input', 'operations'}, f'{path}: 操作列に未観測値')
                for op in observations(row['operations']):
                    snapshots += 1
                    state = op['expected']
                    require(set(state) == {'depth', 'cursor', 'resolved', 'declaredCurrent',
                                           'allDeclarations', 'references', 'diagnostics', 'retained'}, f'{path}: snapshot')
            elif kind == 'catalog':
                require(set(row) == {'id', 'input', 'catalogs'}, f'{path}: catalog に解析値')
                for c in row['catalogs']:
                    require(set(c) == {'rule', 'context', 'captures'}, f'{path}: catalog の項目')
            else:
                canonical(row.get('ast'), row['id'])
                require(type(row['accepted']) is bool, f'{path}: accepted')
                accepted[kind] += row['accepted'] and row.get('allConsumed', True)
                if kind == 'cardinality':
                    require(set(row) == {'id', 'input', 'accepted'}, f'{path}: 不受理時の未観測項目')
                elif kind == 'mixed-values':
                    require(set(row) == {'id', 'input', 'accepted', 'prefix'}, f'{path}: 不受理時の未観測項目')
                elif kind == 'nested-captures':
                    require(row['valueSpans'] == row['retainedValueSpans'], f'{path}: retained span 不一致')
                elif kind == 'evolution':
                    full = row['accepted'] and row['allConsumed']
                    require((row['ast'] is not None) == full, f'{path}: AST/受理')
                    require(row['mappingError'] is None, f'{path}: mappingError')
                    if full:
                        require(('value' in row) != ('evaluationError' in row), f'{path}: 評価の欠落/二重化')
                        require(not row['diagnostics'], f'{path}: 成功時診断')
                    else:
                        require(len(row['diagnostics']) == 1, f'{path}: primary 診断')
                        require(not {'value', 'evaluationError'} & row.keys(), f'{path}: 不受理時の評価')
    require(counts == COUNTS, f'機能別件数: {dict(counts)}')
    require(snapshots == 17, f'ScopeStore snapshot 件数: {snapshots}')
    print(f'{len(names)} ファイル / {sum(counts.values())} ケース: {dict(counts)}')
    print(f'全入力受理: {dict(accepted)} / ScopeStore snapshot: {snapshots}')
    return names


if __name__ == '__main__':
    repo = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else REPO
    names = verify(ROOT, repo)
    if len(sys.argv) > 1:
        other = pathlib.Path(sys.argv[1])
        verify(other, repo)
        for name in sorted(names):
            require((ROOT / name).read_bytes() == (other / name).read_bytes(), f'再採取 byte 不一致: {name}')
        print('追加採取の全 JSON が byte 単位で一致しました。')
