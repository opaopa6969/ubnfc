# P4 を ANTLR 4 と tree-sitter へ移植したもの

`examples/p4-v2/tinyexpression-p4-v2.ubnf`（UBNF v2、119 規則。v1 の手書き形では 124 規則）を
**構文だけ** 2 つの外部 parser generator へ移植したもの。文法本体は

- ANTLR 4: [`TinyExpressionP4.g4`](TinyExpressionP4.g4)（このディレクトリ）
- tree-sitter: [`../p4-treesitter/grammar.js`](../p4-treesitter/grammar.js)

の 2 本で、**対応表はこの README に 1 つだけ置く**（tree-sitter 側の README はここを参照する）。
計測結果は [`docs/reports/2026-09-24-p4-antlr-treesitter.md`](../../docs/reports/2026-09-24-p4-antlr-treesitter.md)。

## 移植していないもの

P4 の注釈のうち、**scope 解決**（`@scopeTree(mode=lexical)` / `@declares` / `@backref` /
`@catalog`）と **AST 射影**（`@mapping`）には ANTLR にも tree-sitter にも対応物が無い。
どちらのツールも「その入力がこの文法の文であるか」しか答えない。
したがってこの 2 本は **構文解析のみ**で、名前解決も型付き AST も作らない。
速度の表で ubnfc 側だけが余分な仕事をしている点は報告の注記を参照。

移植したのは 119 規則すべて（ANTLR は規則名の先頭を小文字にしただけ、tree-sitter は
機械的な snake_case）と、7 つの token 宣言、11 本のパラメータ化規則（雛形）である。
雛形は P4 でも規則を作らないので、両方とも実体化した位置へその場で展開した
（ANTLR はコメントで、tree-sitter は JS のヘルパ関数で）。

## 生成と実行

生成物（ANTLR の `*.java`、tree-sitter の `src/parser.c`）は **commit しない**。
`$WORK` 配下に作る。

```sh
WORK=$(mktemp -d) benchmarks/scripts/p4-external.sh prepare
WORK=$WORK benchmarks/scripts/p4-external.sh accept
WORK=$WORK CORE=26 RUNS=7 benchmarks/scripts/p4-external.sh speed
```

driver は [`P4AntlrDriver.java`](P4AntlrDriver.java) と
[`../p4-treesitter/driver.c`](../p4-treesitter/driver.c)。

## 優先順位塔

P4 v2 は候補順が優先順位（先頭が最高）で、塔は左再帰から脱糖で作られる（D-039）。
**P4 には `@rightAssoc` が 1 つも無い**（v1 / v2 とも grep で 0 件）ので、
ANTLR の `<assoc=right>` も tree-sitter の `prec.right` もこの移植では 1 度も使っていない。

| P4 v2 の塔 | ANTLR 4 | tree-sitter |
|---|---|---|
| `NumberExpression` 候補1 `MulOp` | 直接左再帰の第 1 候補 | `prec.left(2, …)` |
| `NumberExpression` 候補2 `AddOp` | 直接左再帰の第 2 候補 | `prec.left(1, …)` |
| `StringExpression` 候補1 `+`（連結） | 直接左再帰の第 1 候補 | `prec.left(1, …)` |
| `BooleanExpression` 候補1 `^` | 直接左再帰の第 1 候補 | `prec.left(3, …)` |
| `BooleanExpression` 候補2 `&` | 直接左再帰の第 2 候補 | `prec.left(2, …)` |
| `BooleanExpression` 候補3 `\|` | 直接左再帰の第 3 候補 | `prec.left(1, …)` |

塔**以外**の順序つき候補（`Expression` / `ObjectExpression` / `BranchExpression` /
`ArgumentExpression` / `BooleanFactor` / `VariableDeclaration` / `SliceBaseReceiver`）は、
ANTLR では候補の並び順そのものが ALL(*) の決定順になるので追加の指定が要らない。
tree-sitter は GLR で複数の解析が同時に成功しうるので、P4 の候補順を再現するために
`prec.dynamic` を 27 箇所（7 規則）置いた。刻み幅は規則ごとに変えてある
（dynamic precedence は木全体で加算されるため、塔への再入が直接候補を上回らないようにする）。

## trivia（`@whitespace: javaStyle`）

空白・`//` 行コメント・`/* */` ブロックコメントをどこでも読み飛ばす。

| | 実現方法 |
|---|---|
| P4 | 挿入位置を front が計画して IR に載せる（scannerless） |
| ANTLR 4 | lexer 規則 `WS` / `LINE_COMMENT` / `BLOCK_COMMENT` に `-> skip` |
| tree-sitter | `extras: [$.comment, /\s/]`（`comment` は名前つき node にした） |

## 語境界（D-040）

`@ubnf: 2` の既定は `literalBoundary: word`。末尾が `[A-Za-z0-9_]` のリテラルは、
直後も `[A-Za-z0-9_]` なら一致しない（`'min'` が `minimum` の先頭に食い付かない）。

- **ANTLR**: 無料。keyword 規則を `IDENTIFIER` より前に置けば、最長一致＋先着順で同じ振る舞いになる。
- **tree-sitter**: `word: $ => $.identifier` を宣言して keyword extraction を有効にする。
  これが無いと同じ食い付きが起きる。`variablex` / `notx($a)` / `$a as numberx` が
  いずれも ERROR になることを確認した。
- どちらも `.toUpperCase` のように英数字以外で始まるリテラルには境界が掛からない。
  P4 も同じ（境界は**末尾**だけを見る）。

## スキャナレス意味論からの逸脱

P4 は scannerless（字句器を持たず、規則が直接文字を読む）。ANTLR も tree-sitter も
字句器を持つので、次の点が変わる。**影響する入力**を併記する。

| # | 逸脱 | ANTLR | tree-sitter | 影響する入力 |
|---|---|---|---|---|
| **1** | **`NUMBER` の符号**。P4 の組込み `NUMBER` は `[+-]?` を含む 1 token。最長一致の字句器だと `1+2*3` の `+2` を 1 つの NUMBER に取ってしまう。そこで両方とも token を**符号なし**にし、符号を構文側へ出した | `numberLiteral : ('+'\|'-')? NUMBER ;`（P4 に無い規則を 1 つだけ足した） | `seq(optional(choice('+','-')), $.number)` | `abs(-3)` `[::-1]` `1+2*3` は正しく通る。代わりに**符号と数字の間に空白やコメントを挟んだ `abs(- 3)` を受理してしまう**（P4 は拒否） |
| **2** | **code fence の行頭条件**。P4 の `CodeStartParser` / `CodeEndParser` は `StartOfLineParser` で行頭を要求する | lexer 規則に `{getCharPositionInLine() == 0}?` を 1 つだけ置いて再現した（この文法で唯一の意味述語。Java target 専用） | 表現できない。行頭でないフェンスも受理する | tree-sitter だけ、行頭でない ` ```java:X ` を受理する。16 fixture は code fence を 1 つも使わないので受理結果には出ない |
| **3** | **code fence の token 分割**。P4 は `CODE_START` + `CODE_BODY`(`UNTIL('```')`) + `CODE_END` の 3 token | 結合文法は lexer mode を持てないので 3 つを 1 token `CODE_BLOCK` に畳んだ | 3 token を保つが `CODE_BODY` は正規表現近似で空マッチ不可（呼び側で `optional()`） | 本体が空のフェンス、本体末尾がバッククォート 1〜2 個で終わる異常系 |
| **4** | **keyword を識別子に使えない**。P4 では `IDENTIFIER` が要求される位置なら `min` も識別子になる | keyword 規則が `IDENTIFIER` より先に一致するので不可 | `word:` による keyword extraction で同じく不可 | `float min(...)` / `$min` / `import a.trim.B as f;` のような、予約語と同じ綴りの識別子。**ANTLR は `import a.trim.B` を lex できない**（`.trim` が 1 token になる）。tree-sitter は状態ごとに有効 token だけを試すので `class_name` の文脈では `.` + `trim` に分かれ、これは通る。16 fixture には現れない |
| **5** | **`EOF`** | `formula` の末尾に ANTLR の `EOF` を置いた。`booleanExpression` 入口は driver が残り token を見る | token を持たない。tree-sitter は常に入力全体を覆い、余りは `ERROR` になるので観測上は等価 | `complex-tail`（末尾に `@` を足したもの）は両方とも拒否できている |
| **6** | **順序つき選択のコミット**。P4 は PEG で、候補が 1 つ成功したら後続が失敗しても他の候補へ戻らない | ALL(*) は後続文脈込みで実行可能な最小番号の候補を選ぶ（＝無制限先読みの PEG 相当） | GLR は複数解析を並行に保ち、生き残ったもののうち `prec.dynamic` 最大を採る | **`comparison-heavy` 系 4 件**。`Expression ::= NumberExpression \| BooleanExpression \| …` で ubnfc は `NumberExpression` にコミットして `1+2*3` までで止まり EOF に届かず拒否するが、ANTLR と tree-sitter は `BooleanExpression` 経由の解析を見つけて受理する |
| 7 | **入口** | 規則を直接呼べるので 1 文法で両入口を使える | 開始規則は 1 本だけ。`grammar.js` は `P4_TS_ENTRY` で `formula` / `boolean_expression` を選んで **2 回 generate する**（`source_file: choice(...)` に合併すると「どちらの入口で受理されたか」が観測できなくなり、まさに測りたい差が消える） | — |
| 8 | **`prec.dynamic` が要る箇所** | 不要 | 27 箇所。無しだと `if($c){$a}else{$b}` の branch が `object_expression` に、`($name)[0:6]` の受け手が `parenthesized_string_expression` になるなど、P4 の候補順と違う木になる | 木の形だけ（受理には影響しない） |

ANTLR 側は `-w` 相当の警告抑止をしていない。`antlr4` は警告 0 で生成できる。
tree-sitter 側は `tree-sitter generate` が報告した **23 件の conflict をそのまま
`conflicts:` に書いた**（`BooleanExpression` 入口では `[$.variable_ref]` が不要になり 22 件）。
これらはすべて P4 の型別の塔（`NumberExpression` / `StringExpression` /
`BooleanExpression` / `ObjectExpression`）が重なる箇所である。外部 C scanner は使っていない
（`EXTERNAL_TOKEN_COUNT 0`）。

## ANTLR は SLL では足りない

ANTLR の推奨形は 2 段（SLL で試し、bail したら LL でやり直す）だが、この文法では
**`complex` 系・`valid-unicode`・`comparison-heavy` 系のすべてで SLL が bail する**。
P4 の型別の塔は SLL の文脈非依存な近似では区別できない。したがって速度の主表は
`-Dp4antlr.mode=ll`（LL 1 段）で採り、2 段の値も併記した（SLL の分だけ余分に払う）。
`-Dp4antlr.mode=sll` で確かめられる。

## 規則対応表（119 規則）

| # | P4 v2 rule | `TinyExpressionP4.g4` | `grammar.js` | 優先順位 / 備考 |
|---:|---|---|---|---|
| 1 | `Formula` | `formula` | `formula` | `@root`。ANTLR/tree-sitter の既定入口 |
| 2 | `CodeBlock` | `codeBlock` | `code_block` | ANTLR は `CODE_START`/`CODE_BODY`/`CODE_END` を 1 token `CODE_BLOCK` に畳む（逸脱 A2） |
| 3 | `ImportDeclaration` | `importDeclaration` | `import_declaration` |  |
| 4 | `ClassName` | `className` | `class_name` |  |
| 5 | `VariableDeclaration` | `variableDeclaration` | `variable_declaration` | `.g4` は候補順（ALL(*)）／`grammar.js` は `prec.dynamic` 30/20/10/0 |
| 6 | `NumberVariableDeclaration` | `numberVariableDeclaration` | `number_variable_declaration` | `VarDecl<NumberTypeHint, NumberExpression>` 展開 |
| 7 | `StringVariableDeclaration` | `stringVariableDeclaration` | `string_variable_declaration` | `VarDecl<StringTypeHint, StringExpression>` 展開 |
| 8 | `BooleanVariableDeclaration` | `booleanVariableDeclaration` | `boolean_variable_declaration` | `VarDecl<BooleanTypeHint, BooleanExpression>` 展開 |
| 9 | `ObjectVariableDeclaration` | `objectVariableDeclaration` | `object_variable_declaration` | `VarDecl<ObjectTypeHint, ObjectExpression>` 展開 |
| 10 | `TypeHint` | `typeHint` | `type_hint` |  |
| 11 | `NumberTypeHint` | `numberTypeHint` | `number_type_hint` |  |
| 12 | `StringTypeHint` | `stringTypeHint` | `string_type_hint` |  |
| 13 | `BooleanTypeHint` | `booleanTypeHint` | `boolean_type_hint` |  |
| 14 | `ObjectTypeHint` | `objectTypeHint` | `object_type_hint` |  |
| 15 | `OnlyIfAbsent` | `onlyIfAbsent` | `only_if_absent` |  |
| 16 | `Description` | `description` | `description` |  |
| 17 | `Annotation` | `annotation` | `annotation` |  |
| 18 | `AnnotationParameters` | `annotationParameters` | `annotation_parameters` |  |
| 19 | `AnnotationParameter` | `annotationParameter` | `annotation_parameter` |  |
| 20 | `MethodDeclaration` | `methodDeclaration` | `method_declaration` |  |
| 21 | `NumberMethodDeclaration` | `numberMethodDeclaration` | `number_method_declaration` | `MethodDecl<NumberReturnType, NumberExpression>` 展開 |
| 22 | `StringMethodDeclaration` | `stringMethodDeclaration` | `string_method_declaration` | `MethodDecl<StringReturnType, StringExpression>` 展開 |
| 23 | `BooleanMethodDeclaration` | `booleanMethodDeclaration` | `boolean_method_declaration` | `MethodDecl<BooleanReturnType, BooleanExpression>` 展開 |
| 24 | `ObjectMethodDeclaration` | `objectMethodDeclaration` | `object_method_declaration` | `MethodDecl<ObjectReturnType, ObjectExpression>` 展開 |
| 25 | `MethodParameters` | `methodParameters` | `method_parameters` | `ValuesList<MethodParameter>` 展開 |
| 26 | `MethodParameter` | `methodParameter` | `method_parameter` |  |
| 27 | `NumberReturnType` | `numberReturnType` | `number_return_type` |  |
| 28 | `StringReturnType` | `stringReturnType` | `string_return_type` |  |
| 29 | `BooleanReturnType` | `booleanReturnType` | `boolean_return_type` |  |
| 30 | `ObjectReturnType` | `objectReturnType` | `object_return_type` |  |
| 31 | `ReturnType` | `returnType` | `return_type` |  |
| 32 | `ExternalBooleanInvocation` | `externalBooleanInvocation` | `external_boolean_invocation` | `ExternalInvocation<BooleanReturnType>` 展開 |
| 33 | `ExternalNumberInvocation` | `externalNumberInvocation` | `external_number_invocation` | P4 でも手書き（返り値型ごと省略可） |
| 34 | `ExternalStringInvocation` | `externalStringInvocation` | `external_string_invocation` | `ExternalInvocation<StringReturnType>` 展開 |
| 35 | `ExternalObjectInvocation` | `externalObjectInvocation` | `external_object_invocation` | `ExternalInvocation<ObjectReturnType>` 展開 |
| 36 | `MethodInvocationHeader` | `methodInvocationHeader` | `method_invocation_header` |  |
| 37 | `MethodInvocation` | `methodInvocation` | `method_invocation` |  |
| 38 | `ArgumentTernary` | `argumentTernary` | `argument_ternary` |  |
| 39 | `ArgumentExpression` | `argumentExpression` | `argument_expression` | 候補順／`prec.dynamic` 3000/2000/1000/0 |
| 40 | `Arguments` | `arguments` | `arguments` | `ValuesList<ArgumentExpression>` 展開 |
| 41 | `NumberExpression` | `numberExpression` | `number_expression` | **塔**: `.g4` 直接左再帰、候補1 `MulOp` / 候補2 `AddOp`（左結合） ⇔ `grammar.js` `prec.left(2,…)` / `prec.left(1,…)` |
| 42 | `AddOp` | `addOp` | `add_op` |  |
| 43 | `MulOp` | `mulOp` | `mul_op` |  |
| 44 | `MathFunction` | `mathFunction` | `math_function` |  |
| 45 | `SinFunction` | `sinFunction` | `sin_function` |  |
| 46 | `CosFunction` | `cosFunction` | `cos_function` |  |
| 47 | `TanFunction` | `tanFunction` | `tan_function` |  |
| 48 | `SqrtFunction` | `sqrtFunction` | `sqrt_function` |  |
| 49 | `MinFunction` | `minFunction` | `min_function` | `NAryNumberFn<'min'>` 展開 |
| 50 | `MaxFunction` | `maxFunction` | `max_function` | `NAryNumberFn<'max'>` 展開 |
| 51 | `RandomFunction` | `randomFunction` | `random_function` |  |
| 52 | `AbsFunction` | `absFunction` | `abs_function` |  |
| 53 | `RoundFunction` | `roundFunction` | `round_function` |  |
| 54 | `CeilFunction` | `ceilFunction` | `ceil_function` |  |
| 55 | `FloorFunction` | `floorFunction` | `floor_function` |  |
| 56 | `PowFunction` | `powFunction` | `pow_function` |  |
| 57 | `LogFunction` | `logFunction` | `log_function` |  |
| 58 | `ExpFunction` | `expFunction` | `exp_function` |  |
| 59 | `ToNumFunction` | `toNumFunction` | `to_num_function` |  |
| 60 | `ToUpperCaseFunction` | `toUpperCaseFunction` | `to_upper_case_function` |  |
| 61 | `ToLowerCaseFunction` | `toLowerCaseFunction` | `to_lower_case_function` |  |
| 62 | `TrimFunction` | `trimFunction` | `trim_function` |  |
| 63 | `LengthFunction` | `lengthFunction` | `length_function` |  |
| 64 | `LenFunction` | `lenFunction` | `len_function` |  |
| 65 | `ToUpperCaseDotMethod` | `toUpperCaseDotMethod` | `to_upper_case_dot_method` |  |
| 66 | `ToLowerCaseDotMethod` | `toLowerCaseDotMethod` | `to_lower_case_dot_method` |  |
| 67 | `TrimDotMethod` | `trimDotMethod` | `trim_dot_method` |  |
| 68 | `LengthDotMethod` | `lengthDotMethod` | `length_dot_method` |  |
| 69 | `StartsWithFunction` | `startsWithFunction` | `starts_with_function` | `StringPredFn<'startsWith'>` 展開 |
| 70 | `EndsWithFunction` | `endsWithFunction` | `ends_with_function` | `StringPredFn<'endsWith'>` 展開 |
| 71 | `ContainsFunction` | `containsFunction` | `contains_function` | `StringPredFn<'contains'>` 展開 |
| 72 | `InMethod` | `inMethod` | `in_method` |  |
| 73 | `StartsWithDotMethod` | `startsWithDotMethod` | `starts_with_dot_method` | `StringPredDot<'.startsWith'>` 展開 |
| 74 | `EndsWithDotMethod` | `endsWithDotMethod` | `ends_with_dot_method` | `StringPredDot<'.endsWith'>` 展開 |
| 75 | `ContainsDotMethod` | `containsDotMethod` | `contains_dot_method` | `StringPredDot<'.contains'>` 展開 |
| 76 | `StringPredicateReceiver` | `stringPredicateReceiver` | `string_predicate_receiver` |  |
| 77 | `IsPresentFunction` | `isPresentFunction` | `is_present_function` |  |
| 78 | `InTimeRangeFunction` | `inTimeRangeFunction` | `in_time_range_function` |  |
| 79 | `InDayTimeRangeFunction` | `inDayTimeRangeFunction` | `in_day_time_range_function` |  |
| 80 | `DayOfWeek` | `dayOfWeek` | `day_of_week` |  |
| 81 | `SliceBaseReceiver` | `sliceBaseReceiver` | `slice_base_receiver` | 候補順／`'(' VariableRef ')'` にだけ `prec.dynamic(1,…)` |
| 82 | `SliceStartIndex` | `sliceStartIndex` | `slice_start_index` |  |
| 83 | `SliceEndIndex` | `sliceEndIndex` | `slice_end_index` |  |
| 84 | `SliceStepIndex` | `sliceStepIndex` | `slice_step_index` |  |
| 85 | `SliceBaseExpression` | `sliceBaseExpression` | `slice_base_expression` | `Slice<SliceBaseReceiver>` 展開（8 候補） |
| 86 | `SliceNestedExpression` | `sliceNestedExpression` | `slice_nested_expression` | `Slice<SliceBaseExpression>` 展開（8 候補） |
| 87 | `SliceExpression` | `sliceExpression` | `slice_expression` |  |
| 88 | `StringExpression` | `stringExpression` | `string_expression` | **塔**: `.g4` 直接左再帰、候補1 `+` ⇔ `prec.left(1,…)` |
| 89 | `ParenthesizedStringExpression` | `parenthesizedStringExpression` | `parenthesized_string_expression` |  |
| 90 | `StringCastVariable` | `stringCastVariable` | `string_cast_variable` |  |
| 91 | `StringTypedVariable` | `stringTypedVariable` | `string_typed_variable` |  |
| 92 | `BooleanExpression` | `booleanExpression` | `boolean_expression` | **塔**: `.g4` 直接左再帰 `^`>`&`>`|` ⇔ `prec.left(3/2/1,…)` |
| 93 | `NotExpression` | `notExpression` | `not_expression` |  |
| 94 | `BooleanComparable` | `booleanComparable` | `boolean_comparable` |  |
| 95 | `BooleanEqualityExpression` | `booleanEqualityExpression` | `boolean_equality_expression` |  |
| 96 | `BooleanFactor` | `booleanFactor` | `boolean_factor` | 候補順／`prec.dynamic` 3/2/1/0 |
| 97 | `StringComparisonExpression` | `stringComparisonExpression` | `string_comparison_expression` |  |
| 98 | `EqualityOp` | `equalityOp` | `equality_op` |  |
| 99 | `ComparisonExpression` | `comparisonExpression` | `comparison_expression` |  |
| 100 | `CompareOp` | `compareOp` | `compare_op` |  |
| 101 | `ObjectExpression` | `objectExpression` | `object_expression` | 候補順／`prec.dynamic` 5/4/3/2/1/0 |
| 102 | `IfExpression` | `ifExpression` | `if_expression` |  |
| 103 | `BranchExpression` | `branchExpression` | `branch_expression` | 候補順／`prec.dynamic` 70/60/50/40/30/20/10/0 |
| 104 | `TernaryExpression` | `ternaryExpression` | `ternary_expression` |  |
| 105 | `NumberMatchExpression` | `numberMatchExpression` | `number_match_expression` | `MatchBody<NumberCase, NumberDefaultCase>` 展開 |
| 106 | `NumberCase` | `numberCase` | `number_case` | `CaseRule<NumberCaseValue>` 展開 |
| 107 | `NumberDefaultCase` | `numberDefaultCase` | `number_default_case` | `DefaultCaseRule<NumberCaseValue>` 展開 |
| 108 | `NumberCaseValue` | `numberCaseValue` | `number_case_value` |  |
| 109 | `StringMatchExpression` | `stringMatchExpression` | `string_match_expression` | `MatchBody<StringCase, StringDefaultCase>` 展開 |
| 110 | `StringCase` | `stringCase` | `string_case` | `CaseRule<StringCaseValue>` 展開 |
| 111 | `StringDefaultCase` | `stringDefaultCase` | `string_default_case` | `DefaultCaseRule<StringCaseValue>` 展開 |
| 112 | `StringCaseValue` | `stringCaseValue` | `string_case_value` |  |
| 113 | `BooleanMatchExpression` | `booleanMatchExpression` | `boolean_match_expression` | `MatchBody<BooleanCase, BooleanDefaultCase>` 展開 |
| 114 | `BooleanCase` | `booleanCase` | `boolean_case` | `CaseRule<BooleanCaseValue>` 展開 |
| 115 | `BooleanDefaultCase` | `booleanDefaultCase` | `boolean_default_case` | `DefaultCaseRule<BooleanCaseValue>` 展開 |
| 116 | `BooleanCaseValue` | `booleanCaseValue` | `boolean_case_value` |  |
| 117 | `VariableRef` | `variableRef` | `variable_ref` |  |
| 118 | `TypeKeyword` | `typeKeyword` | `type_keyword` |  |
| 119 | `Expression` | `expression` | `expression` | 候補順／`prec.dynamic` 100/80/60/40/20/0 |

ANTLR 側にだけ `numberLiteral`（逸脱 1）がある。それ以外に P4 に無い規則は両方とも作っていない。

## token 対応表

| P4 v2 token | 定義 | `TinyExpressionP4.g4` | `grammar.js` |
|---|---|---|---|
| `NUMBER` | 組込み。`[+-]? ( [0-9]+ ('.' [0-9]*)? \| '.' [0-9]+ ) ([eE][+-]?[0-9]+)?` | `NUMBER`（**符号なし**、逸脱 1） | `number`（**符号なし**、逸脱 1） |
| `IDENTIFIER` | 組込み。`[A-Za-z_][A-Za-z0-9_]*` | `IDENTIFIER` | `identifier`（`word:` に指定） |
| `STRING` | `extern 'StringLiteralParser'`。`'…'` / `"…"`、`\` で 1 code point escape、改行可 | `STRING` | `string` |
| `CODE_START` | `extern 'CodeStartParser'`。行頭の ` ``` ` + `ident ':' ident ('.' ident)*` + 行末 | `CODE_BLOCK` に統合（逸脱 3） | `code_start`（行頭条件なし、逸脱 2） |
| `CODE_BODY` | `UNTIL('```')` | 同上 | `code_body`（正規表現近似、逸脱 3） |
| `CODE_END` | `extern 'CodeEndParser'`。行頭の ` ``` ` + 行末 | 同上 | `code_end`（行頭条件なし、逸脱 2） |
| `EOF` | `END_OF_INPUT` | ANTLR の `EOF` | 無し（逸脱 5） |

## 雛形（パラメータ化規則、D-043）

P4 の雛形は規則を作らず、実体化した位置へ展開される。移植でも同じにした。

| P4 雛形 | 実体化 | `TinyExpressionP4.g4` | `grammar.js` |
|---|---:|---|---|
| `VarDecl<Hint, Value>` | 4 | 各 `*VariableDeclaration` に直書き（コメントで出典） | JS 関数 `varDecl(hint, value, $)` |
| `MethodDecl<Ret, Body>` | 4 | 各 `*MethodDeclaration` に直書き | `methodDecl(ret, body, $)` |
| `ExternalInvocation<Ret>` | 3 | 各 `External*Invocation` に直書き | `externalInvocation(ret, $)` |
| `Slice<Recv>` | 2 | `sliceBaseExpression` / `sliceNestedExpression` に 8 候補ずつ直書き | `slice(recv, $)` |
| `MatchBody<Case, Default>` | 3 | 各 `*MatchExpression` に直書き | `matchBody(caseRule, defaultRule)` |
| `CaseRule<Value>` | 3 | 各 `*Case` に直書き | `caseRule(value, $)` |
| `DefaultCaseRule<Value>` | 3 | 各 `*DefaultCase` に直書き | `defaultCaseRule(value)` |
| `ValuesList<X>` | 2 | `methodParameters` / `arguments` に直書き | `valuesList(x)` |
| `NAryNumberFn<Kw>` | 2 | `minFunction` / `maxFunction` に直書き | `nAryNumberFn(kw, $)` |
| `StringPredFn<Kw>` | 3 | `startsWith` / `endsWith` / `contains` の関数形に直書き | `stringPredFn(kw, $)` |
| `StringPredDot<Kw>` | 3 | 同じ 3 つのドット形に直書き | `stringPredDot(kw, $)` |
