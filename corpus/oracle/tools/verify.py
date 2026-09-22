#!/usr/bin/env python3
"""採取物の網羅性・形・再採取の一致を検証する（意味の期待値を生成しない）。"""
import collections
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
REQUIRED = {
    'DigitParser', 'posix', 'CI', 'REGEX', 'Extern', 'transparent-mapping',
    'dotted-mapping', '@skip', '@enum', '@commonField', '@typeof', '@longestChoice',
    '@predictiveChoice', '@comment', '@backref-no-scope', '@recovery', '@eval',
    '@import', 'ERROR',
}
EXTRA = {'@catalog', 'UNTIL', 'EMPTY', 'EOF', 'LOOKAHEAD', 'NEGATIVE_LOOKAHEAD',
         'bounded-repeat', 'separated', 'CHAR_RANGE', 'NEGATION', 'numeric-capture'}


def require(test, message):
    if not test:
        raise SystemExit(message)


def canonical(value, context):
    if isinstance(value, dict):
        require(set(value) == {'type', 'span', 'fields'}, f'{context}: canonical node のキー')
        require(isinstance(value['type'], str), f'{context}: type')
        require(len(value['span']) == 2 and all(type(x) is int for x in value['span']), f'{context}: span')
        require(0 <= value['span'][0] <= value['span'][1], f'{context}: span 順序')
        require(isinstance(value['fields'], dict), f'{context}: fields')
        for name, field in value['fields'].items():
            canonical(field, context + '/' + name)
    elif isinstance(value, list):
        for i, item in enumerate(value):
            canonical(item, context + '/' + str(i))
    else:
        require(value is None or type(value) in (str, int), f'{context}: canonical scalar')


def verify(directory):
    specs = json.loads(subprocess.check_output([sys.executable, str(ROOT / 'tools/specifications.py')]))
    expected_files = {s['slug'] + '.json' for s in specs}
    require({p.name for p in directory.glob('*.json')} == expected_files, '不足または余分な JSON ファイル')
    counts = collections.Counter()
    executed = collections.Counter()
    stages = collections.Counter()
    revisions = set()
    ids = set()
    total = 0
    mapping_errors = 0
    parse_errors = 0
    for spec in specs:
        path = directory / (spec['slug'] + '.json')
        d = json.loads(path.read_text())
        require(d['feature'] == spec['feature'] and d['grammar'] == spec['grammar'], f'{path}: 採取仕様と不一致')
        require(d.get('imports') == spec.get('imports'), f'{path}: import の原文')
        require(re.fullmatch('[0-9a-f]{40}', d['source']), f'{path}: revision')
        revisions.add(d['source'])
        require([(c['id'], c['input']) for c in d['cases']] == [(c['id'], c['input']) for c in spec['cases']], f'{path}: 入力の欠落')
        require(d['stage'] in {'executed', 'generation_failed', 'compilation_failed'}, f'{path}: stage')
        stages[d['stage']] += len(d['cases'])
        counts[d['feature']] += len(d['cases'])
        for c in d['cases']:
            total += 1
            require(c['id'] not in ids, f"重複 ID: {c['id']}")
            ids.add(c['id'])
            needed = {'id', 'input', 'accepted', 'allConsumed', 'consumed', 'matched', 'ast', 'captures', 'declarations', 'references', 'diagnostics', 'mappingError'}
            require(needed <= c.keys(), f'{path}: 必須キー不足')
            if d['stage'] == 'executed':
                executed[d['feature']] += 1
                require(type(c['allConsumed']) is bool, f'{path}: allConsumed')
                require(type(c['consumed']) is int and type(c['matched']) is int, f'{path}: cursor')
                require(type(c['accepted']) is bool or 'parseError' in c, f'{path}: accepted 不明の根拠なし')
                require('canonicalError' not in c, f'{path}: canonical 化失敗')
            else:
                require(all(c[k] is None for k in ['accepted', 'allConsumed', 'consumed', 'matched', 'ast', 'mappingError']), f'{path}: 未実行値を捏造')
            canonical(c['ast'], c['id'])
            if c['ast'] is not None:
                require(c['accepted'] and c['allConsumed'] and c['mappingError'] is None, f'{path}: AST と段階が矛盾')
            if c['mappingError'] is not None:
                mapping_errors += 1
                require(c['accepted'] and c['allConsumed'] and c['ast'] is None, f'{path}: mapping 失敗の段階')
            if 'parseError' in c:
                parse_errors += 1
            for capture in c['captures']:
                require({'name', 'binding', 'span', 'text'} <= capture.keys(), f'{path}: capture')
                start, end = capture['span']
                require(c['input'][start:end] == capture['text'], f'{path}: capture の code point span と原文が不一致')
            for diag in c['diagnostics']:
                require(isinstance(diag['kind'], str) and type(diag['offset']) is int and isinstance(diag['expected'], list), f'{path}: 診断')
    require(set(counts) == REQUIRED | EXTRA, '機能網羅の不一致')
    require(len(revisions) == 1, '採取 revision が混在')
    require(all(3 <= counts[f] <= 10 for f in counts), '機能ごとの件数が 3〜10 外')
    print(f"{len(specs)} ファイル / {len(counts)} 機能群 / {total} ケース / {next(iter(revisions))}")
    print('段階:', dict(stages), 'mapping 例外:', mapping_errors, 'parse 例外:', parse_errors)
    for feature, n in counts.items():
        print(f'{feature}: {n} (実行 {executed[feature]})')
    return expected_files


if __name__ == '__main__':
    files = verify(ROOT)
    if len(sys.argv) > 1:
        other = pathlib.Path(sys.argv[1])
        verify(other)
        for name in sorted(files):
            require((ROOT / name).read_bytes() == (other / name).read_bytes(), f'再採取が不一致: {name}')
        print('全 JSON の再採取が byte 単位で一致しました。')
