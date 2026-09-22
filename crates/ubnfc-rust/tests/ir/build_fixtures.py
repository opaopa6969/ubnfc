"""手で記述した意味木を IR v1 に直列化する fixture 作成補助。front の lowering は使わない。"""
import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
BASE = json.loads((ROOT / 'ir/examples/tinycalc.ir.json').read_text())


def lit(s): return ('literal', s)
def ref(s): return ('ruleRef', s)
def token(): return ('tokenRef', 'NUMBER')
def seq(*items): return ('seq', items)
def choice(*items): return ('choice', items)
def repeat(child): return ('repeat', child)
def optional(child): return ('optional', child)
def cap(child, name, card='scalar', conv='node'): return ('capture', child, name, card, conv)
def field(name, card='scalar', kind='node'):
    return dict(name=name, cardinality=card, valueType={'kind': kind, **({'typeId': 'Node'} if kind == 'node' else {})})


class Fixture:
    def __init__(self, name, definitions, types):
        self.name = name
        self.source = f'fixtures/{name.lower()}.ubnf'
        self.text = '\n'.join(f'{name}::{r} ::= {expr!r};' for r, expr, *_ in definitions) + '\n'
        self.span = dict(sourceId=self.source, start=0, end=len(self.text))
        self.j = copy.deepcopy(BASE)
        j = self.j
        j['sources'] = [dict(id=self.source, path=self.source, codePointLength=len(self.text), sha256=hashlib.sha256(self.text.encode()).hexdigest())]
        unit = j['grammarUnits'][0]
        unit.update(id=f'grammar:{name}', name=name, sourceId=self.source, source=self.span, annotations=[])
        j['entry'] = dict(grammarId=unit['id'], ruleId=self.rule_id(definitions[0][0]))
        j['importEdges'] = []
        s = j['semantics']
        for key in ('astTypes', 'captures', 'commonFields', 'expressions', 'recipes', 'rules', 'symbols', 'tokens'):
            s[key] = []
        s['trivia']['insertionPlan'] = []
        s['diagnostics']['expectedDisplay'] = []
        j['analysis'] = dict(expressions=[], prefixTries=[], rules=[], sccs=[])
        j['plan']['choices'] = []
        j['plan']['operators'] = []
        j['plan']['memo']['entries'] = []
        j['plan']['lexical'] = [dict(tokenId=self.rule_id('NUMBER'), strategy='scalar', dfa=None)]
        j['requiredFeatures'] = sorted(['backendDiagnostics', 'captures', 'cursorFrameFlag', 'deferredMapping', 'dualCursor', 'memoReplay', 'operators', 'orderedChoice', 'structuredOccurrences', 'triviaInsertion'])
        t = copy.deepcopy(BASE['semantics']['tokens'][0]); t['symbolId'] = self.rule_id('NUMBER'); s['tokens'].append(t)
        self.symbol('NUMBER', 'token')
        s['astTypes'].append(dict(id='Node', name='Node', kind='transparentUnion', enclosingTypeId=None, variants=[t[0] for t in types]))
        for ty, fields in types:
            s['astTypes'].append(dict(id=ty, name=ty, kind='record', enclosingTypeId=None, fields=fields, implements=['Node']))
        for r, expr, mapped, operator in definitions:
            self.symbol(r, 'rule')
            first = len(s['captures'])
            body = self.expression(expr, r, 'body')
            captures = s['captures'][first:]
            rule = dict(symbolId=self.rule_id(r), body=body, annotations=[], eval=None, mappingParams=[], mappingTypeId=mapped, operator=None, skip=False, triviaPolicyId='trivia:javaStyle')
            if mapped:
                fields = next(fields for ty, fields in types if ty == mapped)
                recipe_fields = []
                for f in fields:
                    sites = [c for c in captures if c['name'] == f['name']]
                    recipe_fields.append(dict(field=f['name'], cardinality=f['cardinality'], conversion=sites[0]['conversion'], siteIds=[c['siteId'] for c in sites]))
                s['recipes'].append(dict(id=f'recipe:{r}', ruleId=self.rule_id(r), kind='assocLists' if operator else 'construct', resultTypeId=mapped, fields=recipe_fields, leafFallback=None, onFailure='mappingFailure', phase='selectedMapping', transformId=None))
                rule['mappingParams'] = [f['name'] for f in fields]
            if operator:
                assoc, base, op = operator
                sites = {c['name']: c['siteId'] for c in captures}
                rule['operator'] = dict(associativity=assoc, baseRuleId=self.rule_id(base), operatorRuleId=self.rule_id(op), canonicalForm='leftOpRightLists' if assoc == 'left' else 'baseOpSelfOrBase', leftSiteId=sites['left'], opSiteId=sites['op'], rightSiteId=sites['right'], precedence=10 if r.endswith('Expr') else 20)
                j['plan']['operators'].append(dict(ruleIds=[self.rule_id(r)], strategy='tierFunctions', liftRecipeIds=[f'recipe:{r}'], boundaryExprIds=[body], proof=None))
            s['rules'].append(rule)
            j['analysis']['rules'].append(dict(ruleId=self.rule_id(r), sccId='scc:all', followSyncPatterns=None, leftRecursion='none', progressProven=True))
        # 宣言順に安定な SCC。FIRST は unknown の保守上界、memo は全式有効。
        rule_ids = [r['symbolId'] for r in s['rules']]
        edges = {r: set() for r in rule_ids}
        for e in s['expressions']:
            if e['node']['kind'] == 'ruleRef': edges[e['ruleId']].add(e['node']['symbolId'])
        reachable = {}
        for r in rule_ids:
            seen, pending = set(), list(edges[r])
            while pending:
                child = pending.pop()
                if child in seen: continue
                seen.add(child)
                pending.extend(edges[child])
            reachable[r] = seen
        assigned = set()
        j['analysis']['sccs'] = []
        for r in rule_ids:
            if r in assigned: continue
            members = [other for other in rule_ids if other == r or (other in reachable[r] and r in reachable[other])]
            assigned.update(members)
            sid = 'scc:' + str(len(j['analysis']['sccs']))
            j['analysis']['sccs'].append(dict(id=sid, ruleIds=members, recursive=len(members) > 1 or r in edges[r], memoPolicy='allEligible'))
            for analysis in j['analysis']['rules']:
                if analysis['ruleId'] in members: analysis['sccId'] = sid

    def rule_id(self, r): return f'{self.name}::{r}'

    def symbol(self, name, kind):
        self.j['semantics']['symbols'].append(dict(id=self.rule_id(name), qualifiedSymbolId=self.rule_id(name), name=name, originalName=name, grammarId=f'grammar:{self.name}', kind=kind, sourceId=self.source, source=self.span))

    def expression(self, tree, rule, role):
        captures = []
        while tree[0] == 'capture':
            _, tree, name, card, conv = tree
            captures.append((name, card, conv))
        kind, arg = tree
        eid = f'expr:{self.source}:0:{len(self.text)}:{rule}/{role}/{kind}'
        transaction = {'seq': 'once', 'choice': 'perAlternative', 'repeat': 'occurs', 'optional': 'occurs', 'ruleRef': 'delegate', 'tokenRef': 'delegate', 'literal': 'none'}[kind]
        e = dict(id=eid, node={}, ruleId=self.rule_id(rule), source=self.span, captureSiteIds=[], diagnosticLabel=None, execution=dict(childMode='inherit', entryFlag='setFalse' if kind == 'seq' else 'preserve', transaction=transaction, primitiveProfile='legacyWord' if kind == 'literal' else None, failureCursor='primitiveMayResetMatched' if kind == 'literal' else 'delegate' if kind.endswith('Ref') else 'restoreTransaction'))
        self.j['semantics']['expressions'].append(e)
        a = copy.deepcopy(BASE['analysis']['expressions'][0]); a['exprId'] = eid; a['mayStart'] = dict(certainty='unknown', eof=True, includesTrivia=True, ranges=[], zeroConsumeCondition='unknown'); a['minConsume'] = 0; a['nullable'] = True
        self.j['analysis']['expressions'].append(a)
        self.j['plan']['memo']['entries'].append(dict(exprId=eid, cacheSuccess=True, cacheFailure=True, omissionProof=None, reason='defaultSafe'))
        if kind in ('seq', 'choice'):
            children = [self.expression(child, rule, f'{role}/{i}') for i, child in enumerate(arg)]
            if kind == 'seq':
                e['node'] = dict(kind=kind, items=children)
                for at, index in [('entry', None)] + [('afterChild', i) for i in range(len(children))]:
                    self.j['semantics']['trivia']['insertionPlan'].append(dict(boundary=dict(ownerExprId=eid, at=at, childIndex=index), policyId='trivia:javaStyle'))
            else:
                e['node'] = dict(kind=kind, alternatives=children, selection='ordered', tieBreak='declarationOrder')
                self.j['plan']['choices'].append(dict(exprId=eid, candidateOrder=children, strategy='ordered', fallback='none', proof=None, trieId=None))
        elif kind in ('repeat', 'optional'):
            e['node'] = dict(kind=kind, child=self.expression(arg, rule, role + '/0'))
            if kind == 'repeat': e['node'].update(min=0, max=None, terminator=None)
        elif kind == 'literal': e['node'] = dict(kind=kind, value=arg, caseSensitive=True, wordBoundary=False)
        else: e['node'] = dict(kind=kind, symbolId=self.rule_id(arg))
        for name, card, conv in captures:
            site = eid + '/capture/' + str(len(e['captureSiteIds']))
            e['captureSiteIds'].append(site)
            value_type = dict(kind='text') if conv == 'text' else dict(kind='node', typeId='Node')
            self.j['semantics']['captures'].append(dict(siteId=site, exprId=eid, ruleId=self.rule_id(rule), name=name, cardinality=card, conversion=conv, occurrenceOrder='completionInnerBeforeOuter', skip=False, spanRule='consumedExtent', valueType=value_type))
        return eid

    def write(self, path): Path(path).write_text(json.dumps(self.j, ensure_ascii=False, sort_keys=True, indent=2) + '\n')


binary = [field('left'), field('op', 'list', 'text'), field('right', 'list')]
number = [field('value', kind='text')]
operators = [
    ('Expr', seq(cap(ref('Term'), 'left'), repeat(seq(cap(ref('AddOp'), 'op', 'list', 'text'), cap(ref('Term'), 'right', 'list')))), 'Binary', ('left', 'Term', 'AddOp')),
    ('Term', seq(cap(ref('Factor'), 'left'), repeat(seq(cap(ref('MulOp'), 'op', 'list', 'text'), cap(ref('Factor'), 'right', 'list')))), 'Binary', ('left', 'Factor', 'MulOp')),
    ('AddOp', choice(lit('+'), lit('-')), None, None),
    ('MulOp', choice(lit('*'), lit('/')), None, None),
    ('Factor', choice(ref('Number'), seq(lit('('), ref('Expr'), lit(')'))), None, None),
    ('Number', seq(cap(seq(token()), 'value', conv='text')), 'Number', None),
]
Fixture('Operators', operators, [('Binary', binary), ('Number', number)]).write(Path(__file__).with_name('operators.ir.json'))
power = []
for mode in ('Text', 'Mapped', 'Unicode', 'Grouped'):
    expr, atom, base, op = [mode + s for s in ('Expr', 'Atom', 'Base', 'Op')]
    power.append((expr, choice(seq(cap(ref(base), 'left'), cap(ref(op), 'op', 'list', 'text'), cap(ref(expr), 'right', 'list')), cap(ref(base), 'left')), 'Power', ('right', base, op)))
    power.append((atom, seq(token()) if mode == 'Text' else seq(*([optional(lit('😀'))] if mode == 'Unicode' else []), cap(seq(token()), 'value', conv='text')), None if mode == 'Text' else 'Number', None))
    power.append((base, choice(ref(atom), seq(lit('('), ref(expr), lit(')'))) if mode == 'Grouped' else ref(atom), None, None))
    power.append((op, lit('^'), None, None))
power_fields = copy.deepcopy(binary)
power_fields[0]['valueType'] = dict(kind='mixed', members=[dict(kind='text'), dict(kind='node', typeId='Node')])
Fixture('Power', power, [('Power', power_fields), ('Number', number)]).write(Path(__file__).with_name('power.ir.json'))
