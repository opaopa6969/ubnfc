// unlaxer-parser の `unlaxer-dsl/src/test/resources/right-associative/Power.ubnf` と
// 同じ言語を ANTLR 4 で書いたもの。
//
//   grammar Power { @whitespace: javaStyle  token NUMBER = NumberParser
//     Expr ::= Atom @left { '^' @op Expr @right };
//     Atom ::= NUMBER; }
//
// 差分（examples/arith/README.md）:
//   - UBNF の `{ '^' Expr }` は「0 回以上の繰り返し」だが、内側の Expr が右再帰で
//     残りを食い切るため、生成する言語は `Atom ('^' Expr)?` と同一である。ANTLR で
//     `*` のまま書くと本当に曖昧になり ALL(*) の full-context 予測を強いる（＝ ANTLR を
//     不利にする書き方）ので、言語が同じ `?` で書いた。
//   - NUMBER の先頭符号は Operators.g4 と同じ理由で落としてある。
grammar Power;

start : expr EOF ;
expr  : atom ('^' expr)? ;
atom  : NUMBER ;

NUMBER  : (D+ '.' D+ | D+ '.' | D+ | '.' D+) ([eE] [+\-]? D+)? ;
fragment D : [0-9] ;

WS            : [\u0009\u000A\u000B\u000C\u000D ]+ -> skip ;
LINE_COMMENT  : '//' ~[\r\n]* -> skip ;
BLOCK_COMMENT : '/*' .*? '*/' -> skip ;
ERRCHAR : . ;
