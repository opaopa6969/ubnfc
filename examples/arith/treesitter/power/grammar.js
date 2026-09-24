// Power.ubnf（unlaxer-parser、読み取り専用）と同じ言語の tree-sitter 版。
// `{ '^' Expr }` は内側の右再帰が残りを食い切るので `optional(...)` と同じ言語になる。
// `repeat` のまま書くと本物の衝突になり GLR に分岐を強いる（＝不利にする書き方）ので使わない。
module.exports = grammar({
  name: 'power',

  extras: $ => [
    /[\t\n\u000B\u000C\r ]/,
    $.line_comment,
    $.block_comment,
  ],

  rules: {
    start: $ => $.expr,
    expr: $ => seq($.atom, optional(seq('^', $.expr))),
    atom: $ => $.number,

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
