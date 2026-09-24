// RFC 8259 の JSON を ANTLR 4 で書いたもの。`examples/json/json.ubnf` と
// `examples/json/treesitter/grammar.js` と同じ言語を受理することが目的で、
// 速度のための細工（意図的に遅く／速くする書き方）はしていない。
//
// 対応表と、3 記法で揃えられなかった点は examples/json/README.md を見ること。
//
// 判定の作法（benchmarks/java/JsonAntlrRunner.java）:
//   - parser は BailErrorStrategy（回復せずに例外で落ちる＝拒否）
//   - lexer / parser の両方に例外を投げる ErrorListener を付ける
//     （そうしないと lexer が認識できない文字を読み飛ばして「受理」になる）
//   - 入口は json（末尾に EOF を要求する）
grammar JSON;

json
  : value EOF
  ;

value
  : object
  | array
  | STRING
  | NUMBER
  | 'true'
  | 'false'
  | 'null'
  ;

object
  : '{' (member (',' member)*)? '}'
  ;

member
  : STRING ':' value
  ;

array
  : '[' (value (',' value)*)? ']'
  ;

// RFC 8259 §7 string。制御文字（U+0000..U+001F）と裸の " \ は入れない。
// \uXXXX はサロゲートの対を検査しない（3 記法とも同じ。README §「サロゲート」）。
STRING
  : '"' (ESC | SAFECODEPOINT)* '"'
  ;

fragment ESC        : '\\' (["\\/bfnrt] | UNICODE) ;
fragment UNICODE    : 'u' HEX HEX HEX HEX ;
fragment HEX        : [0-9a-fA-F] ;
fragment SAFECODEPOINT : ~ ["\\\u0000-\u001F] ;

// RFC 8259 §6 number。先頭 + 、先頭 0 の連なり、先頭・末尾のドット、Infinity/NaN は無い。
NUMBER
  : '-'? INT ('.' [0-9]+)? EXP?
  ;

fragment INT : '0' | [1-9] [0-9]* ;
fragment EXP : [Ee] [+\-]? [0-9]+ ;

// RFC 8259 §2 ws。空白は空白・水平タブ・LF・CR の 4 つだけ（\f \v は含まない）。
WS : [ \t\n\r]+ -> skip ;

// 上のどれにも当たらない 1 文字。これが無いと lexer が黙って読み飛ばし、
// `truex` のような入力が「受理」されてしまう。ErrorListener と二重の備え。
ERRCHAR : . ;
