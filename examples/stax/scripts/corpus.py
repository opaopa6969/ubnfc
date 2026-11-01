#!/usr/bin/env python3
"""仕様から手書きした期待値を、両 backend の共有契約と照合する。"""
import json
import pathlib
import sys
from crosscheck import canonical, run, scope_warnings


def nodes(value):
    if isinstance(value, dict):
        if set(value) == {'type', 'span', 'fields'}:
            yield value
        for child in value.values():
            yield from nodes(child)
    elif isinstance(value, list):
        for child in value:
            yield from nodes(child)


def main():
    rust_driver, java_classes = sys.argv[1:3]
    cases = json.loads((pathlib.Path(__file__).parent.parent / 'corpus/cases.json').read_text())
    assert cases and len({c['id'] for c in cases}) == len(cases)
    requests = ''.join(json.dumps(dict(id=i, grammar='Stax', entry=None,
                         input=c['source'], options=dict(lexical=True, scope=True)),
                         ensure_ascii=False) + '\n' for i, c in enumerate(cases))
    outputs = [run([rust_driver], requests),
               run(['java', '-cp', java_classes, 'target.driver.Driver'], requests)]
    for out in outputs:
        assert set(out) == set(range(len(cases))), '要求と応答 id が一致しない'
    failures = []
    for i, case in enumerate(cases):
        r, j = (out[i] for out in outputs)
        assert (r['accepted'], r['allConsumed'], canonical(r.get('ast')), scope_warnings(r)) == (
            j['accepted'], j['allConsumed'], canonical(j.get('ast')), scope_warnings(j)), case['id']
        for backend, response in zip(('Rust', 'Java'), (r, j)):
            label = f"{case['id']}/{backend}"
            accept = case['parse'] == 'accept'
            assert response['accepted'] is accept and response['allConsumed'] is accept, label
            assert response.get('mappingError') is None, label
            if not accept:
                assert any(d.get('kind') == 'syntax' for d in response.get('diagnostics', [])), label
                continue
            ast = response.get('ast')
            assert ast and ast['type'] == 'Program' and ast['fields']['items'], label
            actual = list(nodes(ast))
            source = case['source']
            for check in case['checks']:
                find = source.rindex if check.get('occurrence') == 'last' else source.index
                start = find(check['slice'])
                span = [start, start + len(check['slice'])]
                if not any(n['type'] == check['type'] and n['span'] == span and
                           all(n['fields'].get(k) == v for k, v in check['fields'].items())
                           for n in actual):
                    failures.append(f'{label}: {check}, span={span}')
            expected = sorted((w['offset'], f"未定義のシンボル: '{w['name']}'")
                              for w in case['warnings'])
            if scope_warnings(response) != expected:
                failures.append(f'{label}: {scope_warnings(response)} != {expected}')
        print(f"corpus {case['id']}: {case['parse']} / {case['typecheck']}（型検査は段階3）照合終了")
    assert not failures, '\n'.join(failures)
    print(f'仕様由来 corpus {len(cases)} 件: 受理・AST 型/値/code point 位置・スコープ診断が成功')


if __name__ == '__main__':
    main()
