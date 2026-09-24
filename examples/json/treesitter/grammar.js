// RFC 8259 の JSON を tree-sitter で書いたもの。`examples/json/json.ubnf` と
// `examples/json/antlr/JSON.g4` と同じ言語を受理することが目的。
// 速度のための細工はしていない（tree-sitter は必ず CST を作る）。
//
// 判定の作法（benchmarks/c/json_ts_runner.c）:
//   tree-sitter はエラー回復を持つので「parse が返った＝受理」ではない。
//   root.hasError() が偽で、かつ root が入力全体を覆っていることを受理とする。
module.exports = grammar({
  name: 'json',

  // RFC 8259 §2 ws。既定の extras は /\s/ で、\f \v や U+00A0 等まで拾ってしまうので置き換える。
  extras: $ => [/[ \t\n\r]/],

  rules: {
    // 最上位はあらゆる値（RFC 8259 §2: JSON-text = ws value ws）。
    document: $ => $._value,

    _value: $ => choice(
      $.object,
      $.array,
      $.string,
      $.number,
      $.true,
      $.false,
      $.null,
    ),

    object: $ => seq('{', optional(seq($.pair, repeat(seq(',', $.pair)))), '}'),

    pair: $ => seq(field('key', $.string), ':', field('value', $._value)),

    array: $ => seq('[', optional(seq($._value, repeat(seq(',', $._value)))), ']'),

    // RFC 8259 §7。token(...) で 1 語彙にまとめる（外側に extras が入らないようにするため）。
    // \uXXXX はサロゲートの対を検査しない（3 記法とも同じ）。
    string: $ => token(seq(
      '"',
      repeat(choice(
        /[^"\\\x00-\x1f]/,
        seq('\\', choice(/["\\\/bfnrt]/, /u[0-9a-fA-F]{4}/)),
      )),
      '"',
    )),

    // RFC 8259 §6。
    number: $ => token(seq(
      optional('-'),
      choice('0', /[1-9][0-9]*/),
      optional(seq('.', /[0-9]+/)),
      optional(seq(/[eE]/, optional(/[+-]/), /[0-9]+/)),
    )),

    // 語境界: tree-sitter の字句は最長一致なので `truex` は `true` + 未知の `x` になり、
    // 後者が ERROR ノードになる（hasError で拒否できる）。
    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null',
  },
});
