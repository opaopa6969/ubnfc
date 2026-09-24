// TinyExpressionP4 — an ANTLR4 port of the UBNF v2 grammar
// examples/p4-v2/tinyexpression-p4-v2.ubnf (119 rules; 124 in the v1 hand-written form).
//
// SCOPE OF THE PORT: syntax only. P4's scope annotations (@scopeTree / @declares /
// @backref / @catalog) and its AST mapping (@mapping) have no ANTLR counterpart and are
// NOT reproduced: this grammar recognises the language, it does not resolve names.
//
// Rule names are the P4 rule names with a lower-case initial (ANTLR requires it);
// the correspondence table is in README.md. Parameterised P4 rules (VarDecl<>,
// MethodDecl<>, ExternalInvocation<>, Slice<>, MatchBody<>, CaseRule<>,
// DefaultCaseRule<>, ValuesList<>, NAryNumberFn<>, StringPredFn<>, StringPredDot<>)
// create no rule in P4 either -- they are expanded at every instantiation site here,
// exactly as the UBNF desugaring does.
//
// Deviations forced by ANTLR's separate lexer are listed in README.md section
// "scannerless vs lexer".
grammar TinyExpressionP4;

// ═══════════════════════════════════════════════════════════════════════════
// Parser
// ═══════════════════════════════════════════════════════════════════════════

// @root. P4: { CodeBlock } { ImportDeclaration } { VariableDeclaration } { Annotation }
//            Expression { MethodDeclaration } EOF
formula
    : codeBlock* importDeclaration* variableDeclaration* annotation* expression methodDeclaration* EOF
    ;

// ── Fenced Java code block ────────────────────────────────────────────────
// P4 splits this into CODE_START + CODE_BODY(UNTIL '```') + CODE_END; a combined
// ANTLR grammar cannot carry lexer modes, so the three are one token here (README).
codeBlock
    : CODE_BLOCK
    ;

// ── Import declaration ────────────────────────────────────────────────────
importDeclaration
    : 'import' className ('#' IDENTIFIER)? 'as' IDENTIFIER ';'
    ;

className
    : IDENTIFIER ('.' IDENTIFIER)*
    ;

// ── Variable declarations ─────────────────────────────────────────────────
variableDeclaration
    : numberVariableDeclaration
    | stringVariableDeclaration
    | booleanVariableDeclaration
    | objectVariableDeclaration
    ;

// VarDecl<NumberTypeHint, NumberExpression>
numberVariableDeclaration
    : ('variable' | 'var') '$' IDENTIFIER numberTypeHint?
      ('set' onlyIfAbsent? numberExpression)? description? ';'
    ;

// VarDecl<StringTypeHint, StringExpression>
stringVariableDeclaration
    : ('variable' | 'var') '$' IDENTIFIER stringTypeHint?
      ('set' onlyIfAbsent? stringExpression)? description? ';'
    ;

// VarDecl<BooleanTypeHint, BooleanExpression>
booleanVariableDeclaration
    : ('variable' | 'var') '$' IDENTIFIER booleanTypeHint?
      ('set' onlyIfAbsent? booleanExpression)? description? ';'
    ;

// VarDecl<ObjectTypeHint, ObjectExpression>
objectVariableDeclaration
    : ('variable' | 'var') '$' IDENTIFIER objectTypeHint?
      ('set' onlyIfAbsent? objectExpression)? description? ';'
    ;

typeHint
    : 'as'? ('number' | 'Number' | 'string' | 'String' | 'boolean' | 'Boolean' | 'object' | 'Object')
    ;

numberTypeHint  : 'as'? ('number' | 'Number' | 'float' | 'Float') ;
stringTypeHint  : 'as'? ('string' | 'String') ;
booleanTypeHint : 'as'? ('boolean' | 'Boolean') ;
objectTypeHint  : 'as'? ('object' | 'Object') ;

onlyIfAbsent : 'if' 'not' 'exists' ;
description  : 'description' '=' STRING ;

annotation           : '@' IDENTIFIER '(' annotationParameters? ')' ;
annotationParameters : annotationParameter (',' annotationParameter)* ;
annotationParameter  : IDENTIFIER '=' expression ;

// ── Method declarations ───────────────────────────────────────────────────
methodDeclaration
    : numberMethodDeclaration
    | stringMethodDeclaration
    | booleanMethodDeclaration
    | objectMethodDeclaration
    ;

// MethodDecl<NumberReturnType, NumberExpression>
numberMethodDeclaration
    : numberReturnType IDENTIFIER '(' methodParameters? ')' '{' numberExpression '}'
    ;
stringMethodDeclaration
    : stringReturnType IDENTIFIER '(' methodParameters? ')' '{' stringExpression '}'
    ;
booleanMethodDeclaration
    : booleanReturnType IDENTIFIER '(' methodParameters? ')' '{' booleanExpression '}'
    ;
objectMethodDeclaration
    : objectReturnType IDENTIFIER '(' methodParameters? ')' '{' objectExpression '}'
    ;

// ValuesList<MethodParameter>
methodParameters : methodParameter (',' methodParameter)* ;
methodParameter  : '$' IDENTIFIER ('as' returnType)? ;

numberReturnType  : 'number' | 'float' ;
stringReturnType  : 'string' ;
booleanReturnType : 'boolean' ;
objectReturnType  : 'object' ;
returnType        : numberReturnType | stringReturnType | booleanReturnType | objectReturnType ;

// ── External invocations ──────────────────────────────────────────────────
// ExternalInvocation<BooleanReturnType>
externalBooleanInvocation
    : 'external' ('returning' 'as'?)? booleanReturnType ':'?
      (className '#' IDENTIFIER | IDENTIFIER) '(' arguments? ')'
    ;
// P4 writes this one out by hand: the return type itself is optional.
externalNumberInvocation
    : 'external' (('returning' 'as'?)? numberReturnType ':'? | ':'?)
      (className '#' IDENTIFIER | IDENTIFIER) '(' arguments? ')'
    ;
externalStringInvocation
    : 'external' ('returning' 'as'?)? stringReturnType ':'?
      (className '#' IDENTIFIER | IDENTIFIER) '(' arguments? ')'
    ;
externalObjectInvocation
    : 'external' ('returning' 'as'?)? objectReturnType ':'?
      (className '#' IDENTIFIER | IDENTIFIER) '(' arguments? ')'
    ;

// ── Built-in method invocation ────────────────────────────────────────────
methodInvocationHeader : 'call' 'internal'? | 'internal' ;
methodInvocation       : methodInvocationHeader IDENTIFIER '(' arguments? ')' ;

// ── Argument expression ───────────────────────────────────────────────────
argumentTernary
    : booleanExpression '?' branchExpression ':' branchExpression
    ;
argumentExpression
    : argumentTernary
    | comparisonExpression
    | stringComparisonExpression
    | expression
    ;
// ValuesList<ArgumentExpression>
arguments : argumentExpression (',' argumentExpression)* ;

// ── Numeric expressions ───────────────────────────────────────────────────
// Direct left recursion; alternative order is precedence, highest first.
// P4 declares no @rightAssoc anywhere, so no <assoc=right> is used in this file.
numberExpression
    : numberExpression mulOp numberExpression     // precedence 2 (left)
    | numberExpression addOp numberExpression     // precedence 1 (left)
    | ternaryExpression
    | numberMatchExpression
    | ifExpression
    | mathFunction
    | toNumFunction
    | lengthDotMethod
    | lenFunction
    | lengthFunction
    | externalNumberInvocation
    | numberLiteral
    | variableRef
    | methodInvocation
    | '(' numberExpression ')'
    ;

// P4's NUMBER token carries an optional leading sign. A maximal-munch lexer would
// then read "+2" as one NUMBER in "1+2*3", so the sign is moved into the parser here.
numberLiteral : ('+' | '-')? NUMBER ;

addOp : '+' | '-' ;
mulOp : '*' | '/' ;

// ── Math functions ────────────────────────────────────────────────────────
mathFunction
    : sinFunction
    | cosFunction
    | tanFunction
    | sqrtFunction
    | minFunction
    | maxFunction
    | randomFunction
    | absFunction
    | roundFunction
    | ceilFunction
    | floorFunction
    | powFunction
    | logFunction
    | expFunction
    ;

sinFunction    : 'sin'  '(' argumentExpression ')' ;
cosFunction    : 'cos'  '(' argumentExpression ')' ;
tanFunction    : 'tan'  '(' argumentExpression ')' ;
sqrtFunction   : 'sqrt' '(' argumentExpression ')' ;
// NAryNumberFn<'min'> / NAryNumberFn<'max'>
minFunction    : 'min' '(' argumentExpression (',' argumentExpression)* ')' ;
maxFunction    : 'max' '(' argumentExpression (',' argumentExpression)* ')' ;
randomFunction : 'random' '(' ')' ;
absFunction    : 'abs'   '(' argumentExpression ')' ;
roundFunction  : 'round' '(' argumentExpression ')' ;
ceilFunction   : 'ceil'  '(' argumentExpression ')' ;
floorFunction  : 'floor' '(' argumentExpression ')' ;
powFunction    : 'pow' '(' argumentExpression ',' argumentExpression ')' ;
logFunction    : 'log' '(' argumentExpression ')' ;
expFunction    : 'exp' '(' argumentExpression ')' ;

// ── Type conversion ───────────────────────────────────────────────────────
toNumFunction : 'toNum' '(' stringExpression ',' argumentExpression ')' ;

// ── String functions (function form) ──────────────────────────────────────
toUpperCaseFunction : 'toUpperCase' '(' stringExpression ')' ;
toLowerCaseFunction : 'toLowerCase' '(' stringExpression ')' ;
trimFunction        : 'trim'        '(' stringExpression ')' ;
lengthFunction      : 'length'      '(' stringExpression ')' ;
lenFunction         : 'len'         '(' stringExpression ')' ;

// ── String dot methods ────────────────────────────────────────────────────
toUpperCaseDotMethod : variableRef '.toUpperCase' '(' ')' ;
toLowerCaseDotMethod : variableRef '.toLowerCase' '(' ')' ;
trimDotMethod        : variableRef '.trim'        '(' ')' ;
lengthDotMethod      : variableRef '.length'      '(' ')' ;

// ── String predicates (function form) ─────────────────────────────────────
// StringPredFn<'startsWith'> / <'endsWith'> / <'contains'>
startsWithFunction : 'startsWith' '(' stringExpression ',' stringExpression (',' stringExpression)* ')' ;
endsWithFunction   : 'endsWith'   '(' stringExpression ',' stringExpression (',' stringExpression)* ')' ;
containsFunction   : 'contains'   '(' stringExpression ',' stringExpression (',' stringExpression)* ')' ;

inMethod : stringExpression '.in' '(' stringExpression (',' stringExpression)* ')' ;

// ── String predicates (dot form) ──────────────────────────────────────────
// StringPredDot<'.startsWith'> / <'.endsWith'> / <'.contains'>
startsWithDotMethod : stringPredicateReceiver '.startsWith' '(' stringExpression (',' stringExpression)* ')' ;
endsWithDotMethod   : stringPredicateReceiver '.endsWith'   '(' stringExpression (',' stringExpression)* ')' ;
containsDotMethod   : stringPredicateReceiver '.contains'   '(' stringExpression (',' stringExpression)* ')' ;

stringPredicateReceiver
    : toLowerCaseFunction
    | toUpperCaseFunction
    | trimFunction
    | variableRef
    ;

// ── isPresent ─────────────────────────────────────────────────────────────
isPresentFunction : 'isPresent' '(' variableRef ')' ;

// ── Time range functions ──────────────────────────────────────────────────
inTimeRangeFunction    : 'inTimeRange' '(' numberExpression ',' numberExpression ')' ;
inDayTimeRangeFunction : 'inDayTimeRange' '(' dayOfWeek ',' numberExpression ',' dayOfWeek ',' numberExpression ')' ;

dayOfWeek : 'MONDAY' | 'TUESDAY' | 'WEDNESDAY' | 'THURSDAY' | 'FRIDAY' | 'SATURDAY' | 'SUNDAY' ;

// ── String slice (Python style) ───────────────────────────────────────────
sliceBaseReceiver
    : externalStringInvocation
    | '(' variableRef ')'
    | parenthesizedStringExpression
    | toUpperCaseFunction
    | toLowerCaseFunction
    | trimFunction
    | toUpperCaseDotMethod
    | toLowerCaseDotMethod
    | trimDotMethod
    | STRING
    | variableRef
    | methodInvocation
    ;

sliceStartIndex : numberExpression ;
sliceEndIndex   : numberExpression ;
sliceStepIndex  : numberExpression ;

// Slice<SliceBaseReceiver>
sliceBaseExpression
    : sliceBaseReceiver '[' sliceStartIndex ':' sliceEndIndex ':' sliceStepIndex ']'
    | sliceBaseReceiver '[' sliceStartIndex ':' sliceEndIndex ']'
    | sliceBaseReceiver '[' sliceStartIndex ':' ':' sliceStepIndex ']'
    | sliceBaseReceiver '[' sliceStartIndex ':' ']'
    | sliceBaseReceiver '[' ':' sliceEndIndex ':' sliceStepIndex ']'
    | sliceBaseReceiver '[' ':' sliceEndIndex ']'
    | sliceBaseReceiver '[' ':' ':' sliceStepIndex ']'
    | sliceBaseReceiver '[' ':' ']'
    ;

// Slice<SliceBaseExpression>
sliceNestedExpression
    : sliceBaseExpression '[' sliceStartIndex ':' sliceEndIndex ':' sliceStepIndex ']'
    | sliceBaseExpression '[' sliceStartIndex ':' sliceEndIndex ']'
    | sliceBaseExpression '[' sliceStartIndex ':' ':' sliceStepIndex ']'
    | sliceBaseExpression '[' sliceStartIndex ':' ']'
    | sliceBaseExpression '[' ':' sliceEndIndex ':' sliceStepIndex ']'
    | sliceBaseExpression '[' ':' sliceEndIndex ']'
    | sliceBaseExpression '[' ':' ':' sliceStepIndex ']'
    | sliceBaseExpression '[' ':' ']'
    ;

sliceExpression : sliceNestedExpression | sliceBaseExpression ;

// ── String expressions ────────────────────────────────────────────────────
stringExpression
    : stringExpression '+' stringExpression       // precedence 1 (left)
    | stringMatchExpression
    | ifExpression
    | stringCastVariable
    | stringTypedVariable
    | sliceExpression
    | parenthesizedStringExpression
    | externalStringInvocation
    | toUpperCaseFunction
    | toLowerCaseFunction
    | trimFunction
    | toUpperCaseDotMethod
    | toLowerCaseDotMethod
    | trimDotMethod
    | STRING
    | variableRef
    | methodInvocation
    ;

parenthesizedStringExpression : '(' stringExpression ')' ;
stringCastVariable            : '(' ('string' | 'String') ')' '$' IDENTIFIER ;
stringTypedVariable           : '$' IDENTIFIER 'as' ('string' | 'String') ;

// ── Boolean expressions (XOR > AND > OR) ──────────────────────────────────
booleanExpression
    : booleanExpression '^' booleanExpression     // precedence 3 (left)
    | booleanExpression '&' booleanExpression     // precedence 2 (left)
    | booleanExpression '|' booleanExpression     // precedence 1 (left)
    | booleanFactor
    ;

notExpression : 'not' '(' booleanExpression ')' ;

booleanComparable
    : notExpression
    | ifExpression
    | booleanMatchExpression
    | externalBooleanInvocation
    | inMethod
    | startsWithDotMethod
    | endsWithDotMethod
    | containsDotMethod
    | startsWithFunction
    | endsWithFunction
    | containsFunction
    | isPresentFunction
    | inTimeRangeFunction
    | inDayTimeRangeFunction
    | 'true'
    | 'false'
    | variableRef
    | methodInvocation
    | '(' booleanExpression ')'
    ;

booleanEqualityExpression : booleanComparable equalityOp booleanComparable ;

booleanFactor
    : booleanEqualityExpression
    | comparisonExpression
    | stringComparisonExpression
    | booleanComparable
    ;

stringComparisonExpression : stringExpression equalityOp stringExpression ;
equalityOp                 : '==' | '!=' ;

comparisonExpression : numberExpression compareOp numberExpression ;
compareOp            : '==' | '!=' | '<=' | '>=' | '<' | '>' ;

// ── Object expressions ────────────────────────────────────────────────────
objectExpression
    : numberExpression
    | stringExpression
    | booleanExpression
    | externalObjectInvocation
    | variableRef
    | methodInvocation
    ;

// ── Control flow ──────────────────────────────────────────────────────────
ifExpression
    : 'if' '(' booleanExpression ')' '{' branchExpression '}' 'else' '{' branchExpression '}'
    ;

branchExpression
    : comparisonExpression
    | stringComparisonExpression
    | booleanEqualityExpression
    | numberExpression
    | booleanExpression
    | stringExpression
    | objectExpression
    | methodInvocation
    ;

ternaryExpression
    : '(' booleanExpression '?' branchExpression ':' branchExpression ')'
    ;

// ── Match expressions ─────────────────────────────────────────────────────
// MatchBody<NumberCase, NumberDefaultCase>
numberMatchExpression : 'match' '{' numberCase (',' numberCase)* ',' numberDefaultCase '}' ;
numberCase            : booleanExpression '->' numberCaseValue ;
numberDefaultCase     : 'default' '->' numberCaseValue ;
numberCaseValue       : numberExpression ;

stringMatchExpression : 'match' '{' stringCase (',' stringCase)* ',' stringDefaultCase '}' ;
stringCase            : booleanExpression '->' stringCaseValue ;
stringDefaultCase     : 'default' '->' stringCaseValue ;
stringCaseValue       : stringExpression ;

booleanMatchExpression : 'match' '{' booleanCase (',' booleanCase)* ',' booleanDefaultCase '}' ;
booleanCase            : booleanExpression '->' booleanCaseValue ;
booleanDefaultCase     : 'default' '->' booleanCaseValue ;
booleanCaseValue       : booleanExpression ;

// ── Variable references ───────────────────────────────────────────────────
variableRef : '$' IDENTIFIER ('as'? typeKeyword)? ;

typeKeyword
    : 'number' | 'Number' | 'float' | 'Float'
    | 'string' | 'String' | 'boolean' | 'Boolean' | 'object' | 'Object'
    ;

// ── Top-level expression ──────────────────────────────────────────────────
expression
    : numberExpression
    | booleanExpression
    | stringExpression
    | objectExpression
    | methodInvocation
    | '(' expression ')'
    ;

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
