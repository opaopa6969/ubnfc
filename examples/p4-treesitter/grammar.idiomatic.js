/**
 * The same language as examples/p4-v2/tinyexpression-p4-v2.ubnf, written the way a
 * tree-sitter user would write it -- the counterpart of
 * examples/p4-antlr/TinyExpressionP4Idiomatic.g4 (issue #39).
 *
 * grammar.js is the literal 1:1 port: P4's four type towers (NumberExpression /
 * StringExpression / BooleanExpression / ObjectExpression) overlap everywhere, which
 * costs 23 `conflicts:` entries and 27 `prec.dynamic` annotations -- every one of them
 * a place where the GLR parser has to keep several stacks alive and then pick.
 *
 * Here the towers are ONE `expression` rule with a `prec.left` ladder. Typing is not a
 * syntactic question any more, so 22 of the 23 conflicts and all 27 `prec.dynamic`
 * annotations disappear; the one conflict that survives has nothing to do with typing
 * (see `conflicts:` below). The accepted language is the same superset as the idiomatic
 * ANTLR grammar's (examples/p4-antlr/README.md lists the newly accepted inputs).
 *
 * The token rules are byte-identical to grammar.js.
 */

const IDENT = /[A-Za-z_][A-Za-z0-9_]*/;

// P4_TS_ENTRY=Formula (default) | Expression
const ENTRY = process.env.P4_TS_ENTRY || 'Formula';

// Precedence ladder; higher binds tighter. Same order as the .g4's alternative order.
const P = {
  postfix: 8,   // [slice]  .method()
  mul: 7,       // * /                    P4 NumberExpression alt 1
  add: 6,       // + -                    P4 NumberExpression alt 2 / StringExpression alt 1
  compare: 5,   // == != <= >= < >        P4 Comparison / StringComparison / BooleanEquality
  xor: 4,       // ^                      P4 BooleanExpression alt 1
  and: 3,       // &                      P4 BooleanExpression alt 2
  or: 2,        // |                      P4 BooleanExpression alt 3
  ternary: 1,   // ?:                     P4 ArgumentTernary / TernaryExpression
};

module.exports = grammar({
  name: ENTRY === 'Expression' ? 'tinyexpression_p4_idiomatic_expr' : 'tinyexpression_p4_idiomatic',

  // D-040 word boundary: keyword extraction against the identifier token.
  word: $ => $.identifier,

  // @whitespace: javaStyle
  extras: $ => [/[ \t\r\n]/, $.comment],

  // The 1:1 port needs 23, all of them places where the type towers overlap.
  // Unifying the towers removes 22 of them. The one that survives has nothing to do
  // with typing: at the Formula entry `$a string` is either `variable_ref` with a
  // trailing type keyword or a `variable_ref` followed by a `method_declaration` whose
  // return type is `string`. grammar.js needs exactly the same entry ([$.variable_ref],
  // and likewise only at the Formula entry).
  conflicts: $ => [
    ...(ENTRY === 'Expression' ? [] : [[$.variable_ref]]),
  ],

  rules: {
    source_file: $ => ENTRY === 'Expression' ? $.expression : $.formula,

    formula: $ => seq(
      repeat($.code_block),
      repeat($.import_declaration),
      repeat($.variable_declaration),
      repeat($.annotation),
      $.expression,
      repeat($.method_declaration),
    ),

    // ── The one expression rule ──
    expression: $ => choice(
      prec.left(P.postfix, seq($.expression, '[',
        optional($.expression), ':', optional($.expression),
        optional(seq(':', optional($.expression))), ']')),
      prec.left(P.postfix, seq($.expression, $.dot_method, '(', optional($.arguments), ')')),
      prec.left(P.mul, seq($.expression, $.mul_op, $.expression)),
      prec.left(P.add, seq($.expression, $.add_op, $.expression)),
      prec.left(P.compare, seq($.expression, $.compare_op, $.expression)),
      prec.left(P.xor, seq($.expression, '^', $.expression)),
      prec.left(P.and, seq($.expression, '&', $.expression)),
      prec.left(P.or, seq($.expression, '|', $.expression)),
      prec.right(P.ternary, seq($.expression, '?', $.expression, ':', $.expression)),
      $.number_literal,
      $.string,
      'true',
      'false',
      $.variable_ref,
      $.function_call,
      $.in_day_time_range_function,
      $.not_expression,
      $.if_expression,
      $.match_expression,
      $.external_invocation,
      $.method_invocation,
      $.string_cast_variable,
      seq('(', $.expression, ')'),
    ),

    // Deviation 1 (same as grammar.js): the sign is in the parser, not in the token.
    number_literal: $ => seq(optional(choice('+', '-')), $.number),

    add_op: $ => choice('+', '-'),
    mul_op: $ => choice('*', '/'),
    compare_op: $ => choice('==', '!=', '<=', '>=', '<', '>'),

    dot_method: $ => choice('.toUpperCase', '.toLowerCase', '.trim', '.length',
      '.startsWith', '.endsWith', '.contains', '.in'),

    function_call: $ => seq($.builtin_function, '(', optional($.arguments), ')'),
    builtin_function: $ => choice(
      'sin', 'cos', 'tan', 'sqrt', 'min', 'max', 'random',
      'abs', 'round', 'ceil', 'floor', 'pow', 'log', 'exp',
      'toNum',
      'toUpperCase', 'toLowerCase', 'trim', 'length', 'len',
      'startsWith', 'endsWith', 'contains',
      'isPresent',
      'inTimeRange',
    ),

    in_day_time_range_function: $ => seq('inDayTimeRange', '(',
      $.day_of_week, ',', $.expression, ',', $.day_of_week, ',', $.expression, ')'),
    day_of_week: $ => choice('MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY',
      'FRIDAY', 'SATURDAY', 'SUNDAY'),

    not_expression: $ => seq('not', '(', $.expression, ')'),

    arguments: $ => seq($.expression, repeat(seq(',', $.expression))),

    variable_ref: $ => seq('$', $.identifier, optional(seq(optional('as'), $.type_keyword))),
    string_cast_variable: $ => seq('(', choice('string', 'String'), ')', '$', $.identifier),

    type_keyword: $ => choice('number', 'Number', 'float', 'Float', 'string', 'String',
      'boolean', 'Boolean', 'object', 'Object'),

    // ── Control flow ──
    if_expression: $ => seq('if', '(', $.expression, ')',
      '{', $.expression, '}', 'else', '{', $.expression, '}'),

    match_expression: $ => seq('match', '{', $.match_case,
      repeat(seq(',', $.match_case)), ',', $.default_case, '}'),
    match_case: $ => seq($.expression, '->', $.expression),
    default_case: $ => seq('default', '->', $.expression),

    // ── Declarations ──
    code_block: $ => seq($.code_start, optional($.code_body), $.code_end),

    import_declaration: $ => seq('import', $.class_name,
      optional(seq('#', $.identifier)), 'as', $.identifier, ';'),
    class_name: $ => seq($.identifier, repeat(seq('.', $.identifier))),

    variable_declaration: $ => seq(
      choice('variable', 'var'), '$', $.identifier, optional($.type_hint),
      optional(seq('set', optional($.only_if_absent), $.expression)),
      optional($.description), ';'),

    type_hint: $ => seq(optional('as'), $.type_keyword),
    only_if_absent: $ => seq('if', 'not', 'exists'),
    description: $ => seq('description', '=', $.string),

    annotation: $ => seq('@', $.identifier, '(', optional($.annotation_parameters), ')'),
    annotation_parameters: $ => seq($.annotation_parameter, repeat(seq(',', $.annotation_parameter))),
    annotation_parameter: $ => seq($.identifier, '=', $.expression),

    method_declaration: $ => seq($.return_type, $.identifier,
      '(', optional($.method_parameters), ')', '{', $.expression, '}'),
    method_parameters: $ => seq($.method_parameter, repeat(seq(',', $.method_parameter))),
    method_parameter: $ => seq('$', $.identifier, optional(seq('as', $.return_type))),
    return_type: $ => choice('number', 'float', 'string', 'boolean', 'object'),

    external_invocation: $ => seq('external',
      choice(seq(optional(seq('returning', optional('as'))), $.return_type, optional(':')),
             optional(':')),
      choice(seq($.class_name, '#', $.identifier), $.identifier),
      '(', optional($.arguments), ')'),

    method_invocation_header: $ => choice(seq('call', optional('internal')), 'internal'),
    method_invocation: $ => seq($.method_invocation_header, $.identifier,
      '(', optional($.arguments), ')'),

    // ── Tokens (identical to grammar.js) ──
    identifier: $ => IDENT,
    number: $ => token(/([0-9]+(\.[0-9]*)?|\.[0-9]+)([eE][+-]?[0-9]+)?/),
    string: $ => token(choice(/'([^'\\]|\\.)*'/, /"([^"\\]|\\.)*"/)),
    code_start: $ => token(seq('```', IDENT, ':', IDENT, repeat(seq('.', IDENT)),
      choice('\r\n', '\r', '\n'))),
    code_body: $ => token(/([^`]|`[^`]|``[^`])+/),
    code_end: $ => token(seq('```', optional(choice('\r\n', '\r', '\n')))),
    comment: $ => token(choice(
      seq('//', /[^\n]*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'),
    )),
  },
});
