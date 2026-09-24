// Operators.ubnf（unlaxer-parser、読み取り専用）と同じ言語の tree-sitter 版。
// tree-sitter の字句は「その構文状態で有効な語彙だけを試す」ので、UBNF のスキャナレス
// 字句と同じく NUMBER の**先頭符号を保てる**（ANTLR 版は保てない。examples/arith/README.md）。
module.exports = grammar({
  name: 'operators',

  // @whitespace: javaStyle の区切り集合。
  extras: $ => [
    /[\t\n\u000B\u000C\r ]/,
    $.line_comment,
    $.block_comment,
  ],

  rules: {
    start: $ => $.expression,

    expression: $ => seq($.term, repeat(seq($.add_op, $.term))),
    term: $ => seq($.factor, repeat(seq($.mul_op, $.factor))),
    add_op: $ => choice('+', '-'),
    mul_op: $ => choice('*', '/'),
    factor: $ => choice($.numeric_literal, seq('(', $.expression, ')')),
    numeric_literal: $ => $.number,

    // NumberParser = [+-]? ( D+ '.' D+ | D+ '.' | D+ | '.' D+ ) ( [eE] [+-]? D+ )?
    number: $ => token(seq(
      optional(/[+-]/),
      choice(
        seq(/[0-9]+/, '.', /[0-9]+/),
        seq(/[0-9]+/, '.'),
        /[0-9]+/,
        seq('.', /[0-9]+/),
      ),
      optional(seq(/[eE]/, optional(/[+-]/), /[0-9]+/)),
    )),

    line_comment: $ => token(seq('//', /[^\r\n]*/)),
    block_comment: $ => token(seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),
  },
});
