#!/usr/bin/env python3
"""採取入力だけを定義する。期待値は含めない。"""
import json
import sys
import re

specs = []

def add(slug, feature, body, inputs, **extra):
    imports = re.findall(r"@import \w+ from '[^']+'", body)
    body = re.sub(r"@import \w+ from '[^']+'", '', body)
    grammar = 'grammar Probe {\n ' + '\n '.join(imports) + '\n @package: org.example.oracle\n ' + body + '\n}\n'
    specs.append(dict(slug=slug, feature=feature, grammar=grammar,
                      cases=[dict(id=f'{slug}/{i + 1}', input=s) for i, s in enumerate(inputs)], **extra))

def token(slug, feature, expression, inputs, suffix='', prefix=''):
    add(slug, feature, f'token T={expression}\n @root @mapping(Value, params=[value]) Root ::= {prefix} T @value {suffix};', inputs)

# compat-matrix §4 の19項目。
token('digit', 'DigitParser', 'org.unlaxer.parser.posix.DigitParser', ['7', '73', '７', ''], suffix='E', prefix='')
specs[-1]['grammar'] = specs[-1]['grammar'].replace('token T=', 'token E=EOF\n token T=')
token('posix', 'posix', 'org.unlaxer.parser.posix.AlphabetParser', ['A', '1', 'あ', 'Az'])
token('ci', 'CI', "CI('let')", ['LeT', 'LeT!', 'LET', 'le'])
token('ci-unicode', 'CI', "CI('é')", ['É', 'é', 'e'])
token('regex-ascii', 'REGEX', "REGEX('[a-z]+')", ['ab!', 'ab'])
token('regex-zero', 'REGEX', "REGEX('a*')", ['', 'b'])
token('regex-backref', 'REGEX', r"REGEX('(a+)\\1')", ['aaaa'])
token('regex-emoji', 'REGEX', "REGEX('😀')", ['😀', '😀x'])
token('regex-prefix', 'REGEX', "REGEX('a')", ['😀a', '😀ab'], prefix="'😀'")
token('extern', 'Extern', 'oracle.ExternalCursorParser', ['😀x', '😀y', '😀', 'a'])
add('extern-full', 'Extern', "token T=oracle.ExternalCursorParser @root @mapping(Value, params=[value]) Root ::= T @value 'x';", ['😀x', '😀y', '😀xx'])
add('mapping-sum', 'transparent-mapping', "@root @mapping(Node) Root ::= A | B; @mapping(ItemA) A ::= 'a'; @mapping(ItemB) B ::= 'b';", ['a', 'b', 'c'])
add('mapping-alias', 'transparent-mapping', "@root @mapping(Node) Root ::= Alias; @mapping(Expr) Alias ::= Leaf; @mapping(Item) Leaf ::= 'a';", ['a', 'aa', ''])
add('mapping-dotted', 'dotted-mapping', "@root @mapping(Node) Root ::= A | B; @mapping(Node.A) A ::= 'a'; @mapping(Node.B) B ::= 'b';", ['a', 'b', 'c'], evaluator=True)
add('skip', '@skip', "@root @mapping(Items, params=[items]) Root ::= Item @items { Comma Item @items }; @mapping(Item, params=[value]) Item ::= 'a' @value; @skip @mapping(Comma) Comma ::= ',';", ['a,a', 'a', 'a,', ''])
add('enum', '@enum', "@enum Mode ::= 'on' | 'off'; @root @mapping(Config, params=[mode]) Root ::= Mode @mode;", ['on', 'off', 'other'])
add('enum-plain', '@enum', "@enum Mode ::= 'on' | 'off'; @root @mapping(Config) Root ::= Mode;", ['on', 'off', 'other'])
add('enum-collision', '@enum', "@enum Mode ::= 'on' | 'ON'; @root @mapping(Config, params=[mode]) Root ::= Mode @mode;", ['on'])
common = "token ID=IdentifierParser @root @mapping(Node) @commonField(name) Root ::= A | B; @mapping(A, params=[name]) A ::= 'a:' ID @name; "
add('common-field', '@commonField', common + "@mapping(B, params=[name]) B ::= 'b:' ID @name;", ['a:one', 'b:two', 'x:three'])
add('common-field-missing', '@commonField', common + "@mapping(B) B ::= 'b';", ['b'])
add('common-field-type', '@commonField', 'token NUM=NumberParser ' + common + "@mapping(B, params=[name]) B ::= 'b:' NUM @name;", ['b:1'])
add('typeof', '@typeof', "@root @mapping(Pair, params=[left,right]) Root ::= Item @left ':' @typeof(left) Item @right; @mapping(Node) Item ::= A | B; @mapping(A) A ::= 'a'; @mapping(B) B ::= 'b';", ['a:a', 'b:b', 'a:b', 'b:a', 'a:'])
add('longest', '@longestChoice', "@root @scopeTree(mode=lexical) @longestChoice @mapping(Node) Root ::= A | B | C; @declares(symbol=value) @mapping(A, params=[value]) A ::= 'a' @value; @declares(symbol=value) @mapping(B, params=[value]) B ::= 'ab' @value; @declares(symbol=value) @mapping(C, params=[value]) C ::= 'a' @value;", ['a', 'ab', 'ab!', 'x'])
add('predictive', '@predictiveChoice', "@root @predictiveChoice @mapping(Node) Root ::= A | B; @mapping(A) A ::= 'a' 'x'; @mapping(B) B ::= 'b';", ['ax', 'az', 'b', 'z', ''])
add('predictive-nullable', '@predictiveChoice', "token E=EMPTY @root @predictiveChoice @mapping(Node) Root ::= A | B; @mapping(A) A ::= [ 'a' ] 'x'; @mapping(B) B ::= E 'b';", ['x', 'b', 'z'])
add('predictive-extern', '@predictiveChoice', "token T=oracle.ExternalCursorParser @root @predictiveChoice @mapping(Node) Root ::= A | B; @mapping(A, params=[value]) A ::= T @value; @mapping(B) B ::= 'b';", ['😀x', 'b'])
add('comment-line', '@comment', "@comment: { line: '#' } @root @mapping(Value) Root ::= 'a' 'b';", ['ab', 'a//x\nb', 'a#x\nb'])
add('comment-block', '@comment', "@comment: { block: '(* *)' } @root @mapping(Value) Root ::= 'a' 'b';", ['a/*x*/b', 'a(*x*)b', 'ab'])
add('comment-whitespace', '@comment', "@whitespace: javaStyle @comment: { line: '#' } @root @mapping(Value) Root ::= 'a' 'b';", ['ab', 'a//x\nb', 'a#x\nb'])
add('backref', '@backref-no-scope', "token ID=IdentifierParser @root @backref(name=name) @mapping(Pair, params=[name]) Root ::= ID @name ':' ID @name;", ['x:x', 'x:y', 'x:', 'foo:foo'])
for mode, arg, inputs in [('sync', "sync=';'", ['bad;ok;', 'bad', '😀;x']), ('auto', 'auto', ['bad;ok;', '']), ('skip', 'skip', ['bad;ok;', 'bad'])]:
    add('recovery-' + mode, '@recovery', f"@root @mapping(Root, params=[items]) Root ::= Stmt @items {{ Stmt @items }}; @recovery({arg}) @mapping(Stmt, params=[value]) Stmt ::= 'ok' @value ';';", inputs)
add('recovery-root', '@recovery', "@root @recovery(sync=';') @mapping(Stmt, params=[value]) Root ::= 'ok' @value ';';", ['bad;', 'ok;', ''])
for kind in ['literal', 'passthrough', 'variable_ref', 'binary_arithmetic', 'conditional', 'manual', 'unknown']:
    annot = f"@eval(kind='{kind if kind != 'manual' else 'literal'}', strategy='{'manual' if kind == 'manual' else 'default'}')"
    if kind == 'variable_ref':
        body = f"token ID=IdentifierParser @root {annot} @mapping(Variable, params=[name]) Root ::= ID @name;"
        inp = 'x'
    elif kind == 'binary_arithmetic':
        body = f"token NUM=NumberParser @root {annot} @leftAssoc @precedence(level=10) @mapping(Binary, params=[left,op,right]) Root ::= Leaf @left {{ '+' @op Leaf @right }}; @eval(kind='literal', strategy='default') @mapping(Literal, params=[value]) Leaf ::= NUM @value;"
        inp = '1+2'
    elif kind == 'conditional':
        body = f"token NUM=NumberParser @root {annot} @mapping(Conditional, params=[condition,thenExpr,elseExpr]) Root ::= Leaf @condition '?' Leaf @thenExpr ':' Leaf @elseExpr; @eval(kind='literal', strategy='default') @mapping(Literal, params=[value]) Leaf ::= NUM @value;"
        inp = '1?2:3'
    else:
        body = f"token NUM=NumberParser @root {annot} @mapping(Literal, params=[value]) Root ::= NUM @value;"
        inp = '7'
    add('eval-' + kind, '@eval', body, [inp], evaluator=True)
add('eval-sum', '@eval', "@root @eval(kind='literal', strategy='default') @mapping(Node) Root ::= Leaf; @mapping(Item) Leaf ::= 'a';", ['a'], evaluator=True)
add('import-alias', '@import', "@import b from 'B.ubnf' @root @mapping(Value, params=[value]) Root ::= b.Leaf @value;", ['c', 'x', ''], imports={'B.ubnf': "grammar B { @mapping(Leaf) Leaf ::= 'c'; }"})
add('import-two-level', '@import', "@import b from 'B.ubnf' @root @mapping(Value, params=[value]) Root ::= b.Leaf @value;", ['c', 'b', ''], imports={'B.ubnf': "grammar B { @import c from 'C.ubnf' @mapping(Leaf) Leaf ::= c.Atom; }", 'C.ubnf': "grammar C { @mapping(Atom) Atom ::= 'c'; }"})
add('import-cycle', '@import', "@import b from 'B.ubnf' @root @mapping(Value) Root ::= 'a';", ['a'], imports={'B.ubnf': "grammar B { @import c from 'C.ubnf' Leaf ::= 'b'; }", 'C.ubnf': "grammar C { @import b from 'B.ubnf' Atom ::= 'c'; }"})
add('error', 'ERROR', "@root @mapping(Value) Root ::= 'a' | ERROR('need a');", ['a', 'x', ''])

# 指定の追加観測。
add('catalog', '@catalog', "@root @catalog(context=' variable ') @mapping(Value, params=[name]) Root ::= 'z' @name 'Z' @name;", ['zZ', 'z', 'x'])
token('until', 'UNTIL', "UNTIL('#')", ['abc', '😀x', '😀#tail', '#tail', ''])
token('empty', 'EMPTY', 'EMPTY', ['ab', '', '😀'])
token('eof', 'EOF', 'EOF', ['', 'a', '😀'])
token('lookahead', 'LOOKAHEAD', "LOOKAHEAD('a')", ['a', 'ab', 'b', ''])
add('lookahead-until', 'LOOKAHEAD', "token L=LOOKAHEAD('a') token U=UNTIL('#') @root @mapping(Value, params=[value]) Root ::= L U @value;", ['a#'])
add('lookahead-failure', 'LOOKAHEAD', "token L=LOOKAHEAD('a') @root @mapping(Value) Root ::= L 'x';", ['a'])
token('negative-lookahead', 'NEGATIVE_LOOKAHEAD', "NEGATIVE_LOOKAHEAD('a')", ['', 'b', 'a'])
for slug, expr, inputs in [('empty-zero', 'T{0,0}', ['ab']), ('empty-two', 'T{2,2}', ['ab']), ('empty-one-two', 'T{1,2}', ['ab'])]:
    add(slug, 'bounded-repeat', f'token T=EMPTY @root @mapping(Value) Root ::= {expr};', inputs)
add('repeat-zero', 'bounded-repeat', "@root @mapping(Value) Root ::= 'a'{0,0};", ['', 'a'])
add('repeat-unbounded', 'bounded-repeat', "@root @mapping(Value, params=[values]) Root ::= ('a' @values){2,};", ['', 'a', 'aa', 'aaa!'])
add('separated', 'separated', "@root @mapping(Value, params=[values]) Root ::= ('a' @values) % ',';", ['', 'a', 'a,a', 'a,', ','])
token('char-range', 'CHAR_RANGE', "CHAR_RANGE('a', 'c')", ['`', 'a', 'c', 'd', '😀'])
token('negation-empty', 'NEGATION', "NEGATION('')", ['', 'a', '😀'])
add('numeric', 'numeric-capture', "token NUM=NumberParser @root @mapping(Value, params=[value]) Root ::= 'n' NUM @value ';';", ['n2147483647;', 'n2147483648;', 'n-2147483649;', 'n1.5;', 'n1e2;', 'n+0007;', 'n1e+;'])
json.dump(specs, sys.stdout, ensure_ascii=False, indent=2)
print()
