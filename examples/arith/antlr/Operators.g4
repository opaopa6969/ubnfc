// unlaxer-parser の `unlaxer-dsl/src/test/resources/associative/Operators.ubnf`
// （読み取り専用の兄弟 repo）と同じ言語を ANTLR 4 で書いたもの。
//
//   grammar Operators { @whitespace: javaStyle  token NUMBER = NumberParser
//     Expression ::= Term { AddOp Term };
//     Term       ::= Factor { MulOp Factor };
//     AddOp ::= '+' | '-';  MulOp ::= '*' | '/';
//     Factor ::= NumericLiteral | '(' Expression ')';
//     NumericLiteral ::= (NUMBER) @value; }
//
// 揃えられなかった点（examples/arith/README.md §差分）:
//   NUMBER の**先頭符号**。UBNF の NumberParser は `[+-]? …` を受理するが、UBNF は
//   スキャナレス（NUMBER は「そこで NUMBER が期待されるとき」だけ試される）なのに対し
//   ANTLR の lexer は parser の状態を見ない。先頭符号を NUMBER に入れると `1+2` が
//   NUMBER(1) NUMBER(+2) と最長一致で字句化されて言語が壊れる。ここでは符号を落とした
//   （＝ ANTLR 版は `-5` を受理しない。tree-sitter 版は字句が構文状態依存なので符号を保てる）。
grammar Operators;

start          : expression EOF ;
expression     : term (addOp term)* ;
term           : factor (mulOp factor)* ;
addOp          : '+' | '-' ;
mulOp          : '*' | '/' ;
factor         : numericLiteral | '(' expression ')' ;
numericLiteral : NUMBER ;

// NumberParser = [+-]? ( D+ '.' D+ | D+ '.' | D+ | '.' D+ ) ( [eE] [+-]? D+ )?
// （docs/design/02-facts-tokens-bench-corpus.md §A）。先頭符号だけ上記の理由で外してある。
NUMBER  : (D+ '.' D+ | D+ '.' | D+ | '.' D+) ([eE] [+\-]? D+)? ;
fragment D : [0-9] ;

// @whitespace: javaStyle の区切り集合（crates/ubnfc-front/src/lower/trivia.rs:85-99）:
// code point 9,10,11,12,13,32 と `//` 行コメント、`/* */` ブロックコメント。
WS            : [\u0009\u000A\u000B\u000C\u000D ]+ -> skip ;
LINE_COMMENT  : '//' ~[\r\n]* -> skip ;
BLOCK_COMMENT : '/*' .*? '*/' -> skip ;

// どの語彙にも当たらない 1 文字。無いと lexer が黙って読み飛ばして「受理」になる。
ERRCHAR : . ;
