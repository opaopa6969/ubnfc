/**
 * Stage 1 — literal 1:1 port of the tinyexpression "P4" grammar (UBNF v2, 119 rules).
 * Rule names are the mechanical snake_case of the P4 rule names.
 */

// P4 parameterized rule Slice<Recv> (8 candidates, in P4 candidate order).
function slice(recv, $) {
  return choice(
    seq(recv, '[', $.slice_start_index, ':', $.slice_end_index, ':', $.slice_step_index, ']'),
    seq(recv, '[', $.slice_start_index, ':', $.slice_end_index, ']'),
    seq(recv, '[', $.slice_start_index, ':', ':', $.slice_step_index, ']'),
    seq(recv, '[', $.slice_start_index, ':', ']'),
    seq(recv, '[', ':', $.slice_end_index, ':', $.slice_step_index, ']'),
    seq(recv, '[', ':', $.slice_end_index, ']'),
    seq(recv, '[', ':', ':', $.slice_step_index, ']'),
    seq(recv, '[', ':', ']'),
  );
}

// P4 parameterized rule VarDecl<Hint, Value>
function varDecl(hint, value, $) {
  return seq(
    choice('variable', 'var'),
    '$', $.identifier,
    optional(hint),
    optional(seq('set', optional($.only_if_absent), value)),
    optional($.description),
    ';',
  );
}

// P4 parameterized rule MethodDecl<Ret, Body>
function methodDecl(ret, body, $) {
  return seq(ret, $.identifier, '(', optional($.method_parameters), ')', '{', body, '}');
}

// P4 parameterized rule ExternalInvocation<Ret>
function externalInvocation(ret, $) {
  return seq(
    'external', optional(seq('returning', optional('as'))), ret,
    optional(':'),
    choice(seq($.class_name, '#', $.identifier), $.identifier),
    '(', optional($.arguments), ')',
  );
}

// P4 parameterized rule MatchBody<Case, Default>
function matchBody(caseRule, defaultRule) {
  return seq('match', '{', caseRule, repeat(seq(',', caseRule)), ',', defaultRule, '}');
}

// P4 parameterized rule CaseRule<Value> / DefaultCaseRule<Value>
function caseRule(value, $) { return seq($.boolean_expression, '->', value); }
function defaultCaseRule(value) { return seq('default', '->', value); }

// P4 parameterized rule ValuesList<X>
function valuesList(x) { return seq(x, repeat(seq(',', x))); }

// P4 parameterized rule NAryNumberFn<Kw>
function nAryNumberFn(kw, $) {
  return seq(kw, '(', $.argument_expression, repeat(seq(',', $.argument_expression)), ')');
}

// P4 parameterized rule StringPredFn<Kw> / StringPredDot<Kw>
function stringPredFn(kw, $) {
  return seq(kw, '(', $.string_expression, ',', $.string_expression,
    repeat(seq(',', $.string_expression)), ')');
}
function stringPredDot(kw, $) {
  return seq($.string_predicate_receiver, kw, '(', $.string_expression,
    repeat(seq(',', $.string_expression)), ')');
}

// P4 ordered choice: candidate order is precedence (first = highest).
// `step` is chosen per rule so that a nested re-entry into the same tower
// (e.g. expression -> object_expression -> number_expression) can never
// out-score the direct candidate.
function ordered(step, ...alts) {
  return choice(...alts.map((a, i) => {
    const d = (alts.length - 1 - i) * step;
    return d === 0 ? a : prec.dynamic(d, a);
  }));
}

const IDENT = /[A-Za-z_][A-Za-z0-9_]*/;

// P4_TS_ENTRY=Formula (default) | BooleanExpression
const ENTRY = process.env.P4_TS_ENTRY || 'Formula';

module.exports = grammar({
  // One file, two entry rules. P4's harness parses the comparison-heavy fixtures at
  // the BooleanExpression entry; tree-sitter has a single start rule, so the entry is
  // selected at `tree-sitter generate` time instead of being merged into one start
  // symbol. Merging them would make the start symbol itself ambiguous (formula derives
  // boolean_expression almost unconditionally), add conflicts, grow the state table and
  // -- worst for this comparison -- make the tool accept the union, which is exactly the
  // ubnfc behaviour under test.
  name: ENTRY === 'BooleanExpression' ? 'tinyexpression_p4_boolean' : 'tinyexpression_p4',

  // D-040 word boundary: keyword extraction against the identifier token.
  word: $ => $.identifier,

  // @whitespace: javaStyle
  extras: $ => [/[ \t\r\n]/, $.comment],

  // 23 GLR conflicts, each reported by `tree-sitter generate` and added verbatim.
  // They are all the places where P4's type-indexed towers overlap.
  conflicts: $ => [
    [$.number_expression, $.string_expression, $.boolean_comparable, $.object_expression, $.expression],
    [$.number_expression, $.string_expression],
    [$.number_expression, $.string_expression, $.boolean_comparable],
    [$.object_expression, $.expression],
    [$.number_expression, $.string_expression, $.boolean_comparable, $.object_expression],
    // Only the Formula entry needs this one.
    ...(ENTRY === 'BooleanExpression' ? [] : [[$.variable_ref]]),
    [$.number_expression, $.object_expression, $.expression],
    [$.parenthesized_string_expression, $.object_expression, $.expression],
    [$.boolean_comparable, $.object_expression, $.expression],
    [$.number_expression, $.slice_base_receiver, $.string_expression, $.boolean_comparable, $.object_expression],
    [$.argument_expression, $.boolean_factor],
    [$.number_variable_declaration, $.string_variable_declaration, $.boolean_variable_declaration, $.object_variable_declaration],
    [$.string_typed_variable, $.type_keyword],
    [$.number_expression, $.slice_base_receiver, $.string_expression, $.boolean_comparable],
    [$.number_expression, $.string_expression, $.boolean_comparable, $.object_expression, $.branch_expression],
    [$.object_expression, $.branch_expression],
    [$.boolean_factor, $.branch_expression],
    [$.slice_base_receiver, $.string_expression],
    [$.string_expression, $.boolean_comparable],
    [$.slice_base_receiver, $.string_expression, $.boolean_comparable],
    [$.number_variable_declaration, $.object_expression],
    [$.string_variable_declaration, $.object_expression],
    [$.boolean_variable_declaration, $.object_expression],
  ],

  rules: {
    source_file: $ => ENTRY === 'BooleanExpression' ? $.boolean_expression : $.formula,

    // ── Formula (P4 @root). EOF is implicit in tree-sitter. ──
    formula: $ => seq(
      repeat($.code_block),
      repeat($.import_declaration),
      repeat($.variable_declaration),
      repeat($.annotation),
      $.expression,
      repeat($.method_declaration),
    ),

    code_block: $ => seq($.code_start, optional($.code_body), $.code_end),

    import_declaration: $ => seq('import', $.class_name,
      optional(seq('#', $.identifier)), 'as', $.identifier, ';'),

    class_name: $ => seq($.identifier, repeat(seq('.', $.identifier))),

    // ── Variable declarations ──
    variable_declaration: $ => ordered(10,
      $.number_variable_declaration,
      $.string_variable_declaration,
      $.boolean_variable_declaration,
      $.object_variable_declaration,
    ),
    number_variable_declaration: $ => varDecl($.number_type_hint, $.number_expression, $),
    string_variable_declaration: $ => varDecl($.string_type_hint, $.string_expression, $),
    boolean_variable_declaration: $ => varDecl($.boolean_type_hint, $.boolean_expression, $),
    object_variable_declaration: $ => varDecl($.object_type_hint, $.object_expression, $),

    type_hint: $ => seq(optional('as'), choice('number', 'Number', 'string', 'String',
      'boolean', 'Boolean', 'object', 'Object')),
    number_type_hint: $ => seq(optional('as'), choice('number', 'Number', 'float', 'Float')),
    string_type_hint: $ => seq(optional('as'), choice('string', 'String')),
    boolean_type_hint: $ => seq(optional('as'), choice('boolean', 'Boolean')),
    object_type_hint: $ => seq(optional('as'), choice('object', 'Object')),

    only_if_absent: $ => seq('if', 'not', 'exists'),
    description: $ => seq('description', '=', $.string),

    annotation: $ => seq('@', $.identifier, '(', optional($.annotation_parameters), ')'),
    annotation_parameters: $ => seq($.annotation_parameter, repeat(seq(',', $.annotation_parameter))),
    annotation_parameter: $ => seq($.identifier, '=', $.expression),

    // ── Method declarations ──
    method_declaration: $ => choice(
      $.number_method_declaration,
      $.string_method_declaration,
      $.boolean_method_declaration,
      $.object_method_declaration,
    ),
    number_method_declaration: $ => methodDecl($.number_return_type, $.number_expression, $),
    string_method_declaration: $ => methodDecl($.string_return_type, $.string_expression, $),
    boolean_method_declaration: $ => methodDecl($.boolean_return_type, $.boolean_expression, $),
    object_method_declaration: $ => methodDecl($.object_return_type, $.object_expression, $),

    method_parameters: $ => valuesList($.method_parameter),
    method_parameter: $ => seq('$', $.identifier, optional(seq('as', $.return_type))),

    number_return_type: $ => choice('number', 'float'),
    string_return_type: $ => 'string',
    boolean_return_type: $ => 'boolean',
    object_return_type: $ => 'object',
    return_type: $ => choice($.number_return_type, $.string_return_type,
      $.boolean_return_type, $.object_return_type),

    // ── External invocations ──
    external_boolean_invocation: $ => externalInvocation($.boolean_return_type, $),
    external_number_invocation: $ => seq(
      'external',
      choice(seq(optional(seq('returning', optional('as'))), $.number_return_type, optional(':')),
        optional(':')),
      choice(seq($.class_name, '#', $.identifier), $.identifier),
      '(', optional($.arguments), ')',
    ),
    external_string_invocation: $ => externalInvocation($.string_return_type, $),
    external_object_invocation: $ => externalInvocation($.object_return_type, $),

    // ── Built-in method invocation ──
    method_invocation_header: $ => choice(seq('call', optional('internal')), 'internal'),
    method_invocation: $ => seq($.method_invocation_header, $.identifier,
      '(', optional($.arguments), ')'),

    argument_ternary: $ => seq($.boolean_expression, '?', $.branch_expression, ':',
      $.branch_expression),
    argument_expression: $ => ordered(1000,
      $.argument_ternary,
      $.comparison_expression,
      $.string_comparison_expression,
      $.expression,
    ),
    arguments: $ => valuesList($.argument_expression),

    // ── Numeric expressions (candidate order = precedence, first = highest) ──
    number_expression: $ => choice(
      prec.left(2, seq($.number_expression, $.mul_op, $.number_expression)),
      prec.left(1, seq($.number_expression, $.add_op, $.number_expression)),
      $.ternary_expression,
      $.number_match_expression,
      $.if_expression,
      $.math_function,
      $.to_num_function,
      $.length_dot_method,
      $.len_function,
      $.length_function,
      $.external_number_invocation,
      // DEVIATION: P4's NUMBER token carries its own sign; here the sign is at parser level.
      seq(optional(choice('+', '-')), $.number),
      $.variable_ref,
      $.method_invocation,
      seq('(', $.number_expression, ')'),
    ),
    add_op: $ => choice('+', '-'),
    mul_op: $ => choice('*', '/'),

    // ── Math functions ──
    math_function: $ => choice(
      $.sin_function, $.cos_function, $.tan_function, $.sqrt_function,
      $.min_function, $.max_function, $.random_function, $.abs_function,
      $.round_function, $.ceil_function, $.floor_function, $.pow_function,
      $.log_function, $.exp_function,
    ),
    sin_function: $ => seq('sin', '(', $.argument_expression, ')'),
    cos_function: $ => seq('cos', '(', $.argument_expression, ')'),
    tan_function: $ => seq('tan', '(', $.argument_expression, ')'),
    sqrt_function: $ => seq('sqrt', '(', $.argument_expression, ')'),
    min_function: $ => nAryNumberFn('min', $),
    max_function: $ => nAryNumberFn('max', $),
    random_function: $ => seq('random', '(', ')'),
    abs_function: $ => seq('abs', '(', $.argument_expression, ')'),
    round_function: $ => seq('round', '(', $.argument_expression, ')'),
    ceil_function: $ => seq('ceil', '(', $.argument_expression, ')'),
    floor_function: $ => seq('floor', '(', $.argument_expression, ')'),
    pow_function: $ => seq('pow', '(', $.argument_expression, ',', $.argument_expression, ')'),
    log_function: $ => seq('log', '(', $.argument_expression, ')'),
    exp_function: $ => seq('exp', '(', $.argument_expression, ')'),

    to_num_function: $ => seq('toNum', '(', $.string_expression, ',', $.argument_expression, ')'),

    // ── String functions (function form) ──
    to_upper_case_function: $ => seq('toUpperCase', '(', $.string_expression, ')'),
    to_lower_case_function: $ => seq('toLowerCase', '(', $.string_expression, ')'),
    trim_function: $ => seq('trim', '(', $.string_expression, ')'),
    length_function: $ => seq('length', '(', $.string_expression, ')'),
    len_function: $ => seq('len', '(', $.string_expression, ')'),

    // ── String dot methods ──
    to_upper_case_dot_method: $ => seq($.variable_ref, '.toUpperCase', '(', ')'),
    to_lower_case_dot_method: $ => seq($.variable_ref, '.toLowerCase', '(', ')'),
    trim_dot_method: $ => seq($.variable_ref, '.trim', '(', ')'),
    length_dot_method: $ => seq($.variable_ref, '.length', '(', ')'),

    // ── String predicates (function form) ──
    starts_with_function: $ => stringPredFn('startsWith', $),
    ends_with_function: $ => stringPredFn('endsWith', $),
    contains_function: $ => stringPredFn('contains', $),
    in_method: $ => seq($.string_expression, '.in', '(', $.string_expression,
      repeat(seq(',', $.string_expression)), ')'),

    // ── String predicates (dot form) ──
    starts_with_dot_method: $ => stringPredDot('.startsWith', $),
    ends_with_dot_method: $ => stringPredDot('.endsWith', $),
    contains_dot_method: $ => stringPredDot('.contains', $),
    string_predicate_receiver: $ => choice($.to_lower_case_function, $.to_upper_case_function,
      $.trim_function, $.variable_ref),

    is_present_function: $ => seq('isPresent', '(', $.variable_ref, ')'),

    in_time_range_function: $ => seq('inTimeRange', '(', $.number_expression, ',',
      $.number_expression, ')'),
    in_day_time_range_function: $ => seq('inDayTimeRange', '(', $.day_of_week, ',',
      $.number_expression, ',', $.day_of_week, ',', $.number_expression, ')'),
    day_of_week: $ => choice('MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY',
      'SATURDAY', 'SUNDAY'),

    // ── String slice ──
    slice_base_receiver: $ => choice(
      $.external_string_invocation,
      prec.dynamic(1, seq('(', $.variable_ref, ')')),
      $.parenthesized_string_expression,
      $.to_upper_case_function, $.to_lower_case_function, $.trim_function,
      $.to_upper_case_dot_method, $.to_lower_case_dot_method, $.trim_dot_method,
      $.string,
      $.variable_ref,
      $.method_invocation,
    ),
    slice_start_index: $ => $.number_expression,
    slice_end_index: $ => $.number_expression,
    slice_step_index: $ => $.number_expression,
    slice_base_expression: $ => slice($.slice_base_receiver, $),
    slice_nested_expression: $ => slice($.slice_base_expression, $),
    slice_expression: $ => choice($.slice_nested_expression, $.slice_base_expression),

    // ── String expressions ──
    string_expression: $ => choice(
      prec.left(1, seq($.string_expression, '+', $.string_expression)),
      $.string_match_expression,
      $.if_expression,
      $.string_cast_variable,
      $.string_typed_variable,
      $.slice_expression,
      $.parenthesized_string_expression,
      $.external_string_invocation,
      $.to_upper_case_function, $.to_lower_case_function, $.trim_function,
      $.to_upper_case_dot_method, $.to_lower_case_dot_method, $.trim_dot_method,
      $.string,
      $.variable_ref,
      $.method_invocation,
    ),
    parenthesized_string_expression: $ => seq('(', $.string_expression, ')'),
    string_cast_variable: $ => seq('(', choice('string', 'String'), ')', '$', $.identifier),
    string_typed_variable: $ => seq('$', $.identifier, 'as', choice('string', 'String')),

    // ── Boolean expressions (OR < AND < XOR) ──
    boolean_expression: $ => choice(
      prec.left(3, seq($.boolean_expression, '^', $.boolean_expression)),
      prec.left(2, seq($.boolean_expression, '&', $.boolean_expression)),
      prec.left(1, seq($.boolean_expression, '|', $.boolean_expression)),
      $.boolean_factor,
    ),

    not_expression: $ => seq('not', '(', $.boolean_expression, ')'),

    boolean_comparable: $ => choice(
      $.not_expression,
      $.if_expression,
      $.boolean_match_expression,
      $.external_boolean_invocation,
      $.in_method,
      $.starts_with_dot_method,
      $.ends_with_dot_method,
      $.contains_dot_method,
      $.starts_with_function,
      $.ends_with_function,
      $.contains_function,
      $.is_present_function,
      $.in_time_range_function,
      $.in_day_time_range_function,
      'true',
      'false',
      $.variable_ref,
      $.method_invocation,
      seq('(', $.boolean_expression, ')'),
    ),
    boolean_equality_expression: $ => seq($.boolean_comparable, $.equality_op, $.boolean_comparable),
    boolean_factor: $ => ordered(1,
      $.boolean_equality_expression,
      $.comparison_expression,
      $.string_comparison_expression,
      $.boolean_comparable,
    ),

    string_comparison_expression: $ => seq($.string_expression, $.equality_op, $.string_expression),
    equality_op: $ => choice('==', '!='),
    comparison_expression: $ => seq($.number_expression, $.compare_op, $.number_expression),
    compare_op: $ => choice('==', '!=', '<=', '>=', '<', '>'),

    // ── Object expressions ──
    object_expression: $ => ordered(1,
      $.number_expression,
      $.string_expression,
      $.boolean_expression,
      $.external_object_invocation,
      $.variable_ref,
      $.method_invocation,
    ),

    // ── Control flow ──
    if_expression: $ => seq('if', '(', $.boolean_expression, ')',
      '{', $.branch_expression, '}', 'else', '{', $.branch_expression, '}'),
    branch_expression: $ => ordered(10,
      $.comparison_expression,
      $.string_comparison_expression,
      $.boolean_equality_expression,
      $.number_expression,
      $.boolean_expression,
      $.string_expression,
      $.object_expression,
      $.method_invocation,
    ),
    ternary_expression: $ => seq('(', $.boolean_expression, '?', $.branch_expression, ':',
      $.branch_expression, ')'),

    // ── Match expressions ──
    number_match_expression: $ => matchBody($.number_case, $.number_default_case),
    number_case: $ => caseRule($.number_case_value, $),
    number_default_case: $ => defaultCaseRule($.number_case_value),
    number_case_value: $ => $.number_expression,

    string_match_expression: $ => matchBody($.string_case, $.string_default_case),
    string_case: $ => caseRule($.string_case_value, $),
    string_default_case: $ => defaultCaseRule($.string_case_value),
    string_case_value: $ => $.string_expression,

    boolean_match_expression: $ => matchBody($.boolean_case, $.boolean_default_case),
    boolean_case: $ => caseRule($.boolean_case_value, $),
    boolean_default_case: $ => defaultCaseRule($.boolean_case_value),
    boolean_case_value: $ => $.boolean_expression,

    // ── Variable references ──
    variable_ref: $ => seq('$', $.identifier, optional(seq(optional('as'), $.type_keyword))),
    type_keyword: $ => choice('number', 'Number', 'float', 'Float', 'string', 'String',
      'boolean', 'Boolean', 'object', 'Object'),

    // ── Top-level expression ──
    expression: $ => ordered(20,
      $.number_expression,
      $.boolean_expression,
      $.string_expression,
      $.object_expression,
      $.method_invocation,
      seq('(', $.expression, ')'),
    ),

    // ── Tokens ──
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
