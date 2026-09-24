// TinyExpressionP4Idiomatic — the same language as examples/p4-v2/tinyexpression-p4-v2.ubnf,
// written the way an ANTLR 4 user would write it instead of rule-for-rule after P4.
//
// WHAT CHANGES vs TinyExpressionP4.g4 (the 1:1 port):
//   P4 decides *types* in the syntax. It has four parallel towers -- NumberExpression,
//   StringExpression, BooleanExpression, ObjectExpression -- and an ordered choice
//     Expression ::= NumberExpression | BooleanExpression | StringExpression | ... ;
//   picks between them. A PEG commits to the first tower that succeeds; ALL(*) cannot,
//   so it resolves that choice with full-context LL prediction on every parse (the 1:1
//   port spends 99% of its time there -- see docs/reports/2026-09-24-p4-antlr-treesitter.md).
//
//   Here the four towers are ONE left-recursive `expression` rule whose alternative
//   order is the precedence ladder, which is the ANTLR idiom. Typing is not decided by
//   the parser at all; it belongs to a later pass (a listener / a type checker).
//
// CONSEQUENCE: the accepted language is a SUPERSET of P4's. `1 + true` is a syntax
// error in P4 and a *type* error here. The newly accepted inputs are listed in
// README.md (section "ANTLR / tree-sitter 流に書き直した版"). The 16 benchmark fixtures accept and
// reject exactly as the 1:1 port does.
//
// The lexer is byte-identical to TinyExpressionP4.g4 so that only the parser differs.
grammar TinyExpressionP4Idiomatic;

// ═══════════════════════════════════════════════════════════════════════════
// Parser
// ═══════════════════════════════════════════════════════════════════════════

// P4: { CodeBlock } { ImportDeclaration } { VariableDeclaration } { Annotation }
//     Expression { MethodDeclaration } EOF -- unchanged.
formula
    : codeBlock* importDeclaration* variableDeclaration* annotation* expression methodDeclaration* EOF
    ;

// ═══════════════════════════════════════════════════════════════════════════
// The one expression rule
//
// Direct left recursion; alternative order is precedence, highest first:
//
//   postfix  [slice] .method()        P4: Slice<>, StringPredDot<>, *DotMethod
//   * /                               P4: NumberExpression alt 1 (MulOp)
//   + -                               P4: NumberExpression alt 2 (AddOp) and
//                                          StringExpression alt 1 (concatenation)
//   == != <= >= < >                   P4: ComparisonExpression / StringComparisonExpression /
//                                          BooleanEqualityExpression (one level, all of them)
//   ^                                 P4: BooleanExpression alt 1
//   &                                 P4: BooleanExpression alt 2
//   |                                 P4: BooleanExpression alt 3
//   ?:                                P4: ArgumentTernary / TernaryExpression
//
// P4 declares no @rightAssoc anywhere, so no <assoc=right> appears except on `?:`
// (README: the one associativity marker with no P4 counterpart; it only changes how
// nested un-parenthesised ternaries group, and P4 rejects those outright).
// ═══════════════════════════════════════════════════════════════════════════
expression
    // ── postfix ──────────────────────────────────────────────────────────
    // P4 writes Slice<> out as 8 alternatives over 2 receiver rules; one generic
    // form covers all 8 (and accepts `[a:b:]` / `[::]`, which P4 does not).
    : expression '[' expression? ':' expression? (':' expression?)? ']'
    // P4: .toUpperCase/.toLowerCase/.trim/.length/.startsWith/.endsWith/.contains/.in,
    // each with its own receiver rule. Here any expression is a receiver.
    | expression dotMethod '(' arguments? ')'
    // ── binary operators, highest precedence first ───────────────────────
    | expression mulOp expression
    | expression addOp expression
    | expression compareOp expression
    | expression '^' expression
    | expression '&' expression
    | expression '|' expression
    | <assoc=right> expression '?' expression ':' expression
    // ── primaries ────────────────────────────────────────────────────────
    | numberLiteral
    | STRING
    | 'true'
    | 'false'
    | variableRef
    | functionCall
    | inDayTimeRangeFunction
    | notExpression
    | ifExpression
    | matchExpression
    | externalInvocation
    | methodInvocation
    | stringCastVariable
    | '(' expression ')'
    ;

// P4's NUMBER token carries an optional leading sign; a maximal-munch lexer would then
// read "+2" as one NUMBER in "1+2*3", so the sign is moved into the parser -- same
// deviation (#1) as the 1:1 port.
numberLiteral : ('+' | '-')? NUMBER ;

addOp     : '+' | '-' ;
mulOp     : '*' | '/' ;
compareOp : '==' | '!=' | '<=' | '>=' | '<' | '>' ;

dotMethod
    : '.toUpperCase' | '.toLowerCase' | '.trim' | '.length'
    | '.startsWith' | '.endsWith' | '.contains' | '.in'
    ;

// P4 gives every built-in its own rule with its own arity and its own argument type.
// One call rule replaces all of them; arity and argument types become semantic checks.
functionCall : builtinFunction '(' arguments? ')' ;

builtinFunction
    : 'sin' | 'cos' | 'tan' | 'sqrt' | 'min' | 'max' | 'random'
    | 'abs' | 'round' | 'ceil' | 'floor' | 'pow' | 'log' | 'exp'
    | 'toNum'
    | 'toUpperCase' | 'toLowerCase' | 'trim' | 'length' | 'len'
    | 'startsWith' | 'endsWith' | 'contains'
    | 'isPresent'
    | 'inTimeRange'
    ;

// Kept as its own rule: its arguments are day names, which are not expressions.
inDayTimeRangeFunction
    : 'inDayTimeRange' '(' dayOfWeek ',' expression ',' dayOfWeek ',' expression ')'
    ;

dayOfWeek : 'MONDAY' | 'TUESDAY' | 'WEDNESDAY' | 'THURSDAY' | 'FRIDAY' | 'SATURDAY' | 'SUNDAY' ;

notExpression : 'not' '(' expression ')' ;

arguments : expression (',' expression)* ;

variableRef         : '$' IDENTIFIER ('as'? typeKeyword)? ;
stringCastVariable  : '(' ('string' | 'String') ')' '$' IDENTIFIER ;

typeKeyword
    : 'number' | 'Number' | 'float' | 'Float'
    | 'string' | 'String' | 'boolean' | 'Boolean' | 'object' | 'Object'
    ;

// ── Control flow ──────────────────────────────────────────────────────────
ifExpression
    : 'if' '(' expression ')' '{' expression '}' 'else' '{' expression '}'
    ;

// P4 has three MatchBody<> instantiations (number / string / boolean); one is enough
// once the case values are untyped. Mixing case value types is accepted here.
matchExpression : 'match' '{' matchCase (',' matchCase)* ',' defaultCase '}' ;
matchCase       : expression '->' expression ;
defaultCase     : 'default' '->' expression ;

// ── Declarations (unchanged from the 1:1 port except for the collapsed towers) ──
codeBlock : CODE_BLOCK ;

importDeclaration : 'import' className ('#' IDENTIFIER)? 'as' IDENTIFIER ';' ;
className         : IDENTIFIER ('.' IDENTIFIER)* ;

// P4: VarDecl<NumberTypeHint, NumberExpression> and its three siblings. One rule here;
// whether the hint agrees with the initialiser is a type question.
variableDeclaration
    : ('variable' | 'var') '$' IDENTIFIER typeHint?
      ('set' onlyIfAbsent? expression)? description? ';'
    ;

typeHint     : 'as'? typeKeyword ;
onlyIfAbsent : 'if' 'not' 'exists' ;
description  : 'description' '=' STRING ;

annotation           : '@' IDENTIFIER '(' annotationParameters? ')' ;
annotationParameters : annotationParameter (',' annotationParameter)* ;
annotationParameter  : IDENTIFIER '=' expression ;

// P4: MethodDecl<NumberReturnType, NumberExpression> and its three siblings.
methodDeclaration
    : returnType IDENTIFIER '(' methodParameters? ')' '{' expression '}'
    ;

methodParameters : methodParameter (',' methodParameter)* ;
methodParameter  : '$' IDENTIFIER ('as' returnType)? ;

returnType : 'number' | 'float' | 'string' | 'boolean' | 'object' ;

// P4: ExternalInvocation<Ret> for boolean/string/object plus a hand-written number form
// whose return type may be omitted. Here the return type is optional for every type.
externalInvocation
    : 'external' (('returning' 'as'?)? returnType ':'? | ':'?)
      (className '#' IDENTIFIER | IDENTIFIER) '(' arguments? ')'
    ;

methodInvocationHeader : 'call' 'internal'? | 'internal' ;
methodInvocation       : methodInvocationHeader IDENTIFIER '(' arguments? ')' ;


// ═══════════════════════════════════════════════════════════════════════════
// Lexer
//
// Keyword rules precede IDENTIFIER, so maximal munch plus first-rule-wins gives
// P4's D-040 word boundary for free: 'min' does not match the prefix of "minimum".
// The price is that a keyword can no longer be used as an IDENTIFIER (README).
// ═══════════════════════════════════════════════════════════════════════════

// ── Fenced Java code block ────────────────────────────────────────────────
// P4's CodeStartParser / CodeEndParser require the fence to start a line
// (StartOfLineParser); ANTLR has no such assertion, so this is the one predicate
// in the file. The '.*?' is P4's CODE_BODY = UNTIL('```').
CODE_BLOCK
    : {getCharPositionInLine() == 0}?
      '```' IdentPart ':' IdentPart ('.' IdentPart)* LineEnd
      .*?
      '```' LineEnd
    ;

// ── Keywords ──────────────────────────────────────────────────────────────
K_VARIABLE    : 'variable' ;
K_VAR         : 'var' ;
K_SET         : 'set' ;
K_IF          : 'if' ;
K_NOT         : 'not' ;
K_EXISTS      : 'exists' ;
K_DESCRIPTION : 'description' ;
K_AS          : 'as' ;
K_IMPORT      : 'import' ;
K_EXTERNAL    : 'external' ;
K_RETURNING   : 'returning' ;
K_CALL        : 'call' ;
K_INTERNAL    : 'internal' ;
K_ELSE        : 'else' ;
K_MATCH       : 'match' ;
K_DEFAULT     : 'default' ;
K_TRUE        : 'true' ;
K_FALSE       : 'false' ;

K_NUMBER_LC  : 'number' ;
K_NUMBER_UC  : 'Number' ;
K_STRING_LC  : 'string' ;
K_STRING_UC  : 'String' ;
K_BOOLEAN_LC : 'boolean' ;
K_BOOLEAN_UC : 'Boolean' ;
K_OBJECT_LC  : 'object' ;
K_OBJECT_UC  : 'Object' ;
K_FLOAT_LC   : 'float' ;
K_FLOAT_UC   : 'Float' ;

K_SIN    : 'sin' ;
K_COS    : 'cos' ;
K_TAN    : 'tan' ;
K_SQRT   : 'sqrt' ;
K_MIN    : 'min' ;
K_MAX    : 'max' ;
K_RANDOM : 'random' ;
K_ABS    : 'abs' ;
K_ROUND  : 'round' ;
K_CEIL   : 'ceil' ;
K_FLOOR  : 'floor' ;
K_POW    : 'pow' ;
K_LOG    : 'log' ;
K_EXP    : 'exp' ;
K_TONUM  : 'toNum' ;

K_TOUPPERCASE : 'toUpperCase' ;
K_TOLOWERCASE : 'toLowerCase' ;
K_TRIM        : 'trim' ;
K_LENGTH      : 'length' ;
K_LEN         : 'len' ;
K_STARTSWITH  : 'startsWith' ;
K_ENDSWITH    : 'endsWith' ;
K_CONTAINS    : 'contains' ;
K_ISPRESENT   : 'isPresent' ;

K_INTIMERANGE    : 'inTimeRange' ;
K_INDAYTIMERANGE : 'inDayTimeRange' ;

K_MONDAY    : 'MONDAY' ;
K_TUESDAY   : 'TUESDAY' ;
K_WEDNESDAY : 'WEDNESDAY' ;
K_THURSDAY  : 'THURSDAY' ;
K_FRIDAY    : 'FRIDAY' ;
K_SATURDAY  : 'SATURDAY' ;
K_SUNDAY    : 'SUNDAY' ;

// ── Dot methods (single tokens: P4 writes them as single literals) ────────
DOT_TOUPPERCASE : '.toUpperCase' ;
DOT_TOLOWERCASE : '.toLowerCase' ;
DOT_TRIM        : '.trim' ;
DOT_LENGTH      : '.length' ;
DOT_STARTSWITH  : '.startsWith' ;
DOT_ENDSWITH    : '.endsWith' ;
DOT_CONTAINS    : '.contains' ;
DOT_IN          : '.in' ;

// ── Punctuation and operators ─────────────────────────────────────────────
ARROW  : '->' ;
EQ     : '==' ;
NEQ    : '!=' ;
LE     : '<=' ;
GE     : '>=' ;
LT     : '<' ;
GT     : '>' ;
ASSIGN : '=' ;
PLUS   : '+' ;
MINUS  : '-' ;
STAR   : '*' ;
SLASH  : '/' ;
CARET  : '^' ;
AMP    : '&' ;
PIPE   : '|' ;
QUESTION : '?' ;
DOLLAR : '$' ;
HASH   : '#' ;
AT     : '@' ;
DOT    : '.' ;
COMMA  : ',' ;
SEMI   : ';' ;
COLON  : ':' ;
LPAREN : '(' ;
RPAREN : ')' ;
LBRACE : '{' ;
RBRACE : '}' ;
LBRACK : '[' ;
RBRACK : ']' ;

// ── Literals ──────────────────────────────────────────────────────────────
// P4's builtin NUMBER also accepts a leading '+'/'-'; see numberLiteral above.
NUMBER
    : (Digit+ ('.' Digit*)? | '.' Digit+) ([eE] [+-]? Digit+)?
    ;

// P4 extern StringLiteralParser: single- or double-quoted, '\' escapes one code
// point, newlines allowed inside, must be terminated.
STRING
    : '\'' ('\\' . | ~['\\])* '\''
    | '"'  ('\\' . | ~["\\])* '"'
    ;

IDENTIFIER : [a-zA-Z_] [a-zA-Z0-9_]* ;

// ── Trivia: @whitespace: javaStyle ────────────────────────────────────────
WS            : [ \t\r\n]+       -> skip ;
LINE_COMMENT  : '//' ~[\r\n]*    -> skip ;
BLOCK_COMMENT : '/*' .*? '*/'    -> skip ;

fragment Digit     : [0-9] ;
fragment IdentPart : [a-zA-Z_] [a-zA-Z0-9_]* ;
fragment LineEnd   : '\r\n' | [\r\n] | EOF ;
