# diff-diagnostics-farthestexpected-d88564a1-seed2

seed: 2 / 入力 0 code point / 縮小 4 回

## signature

`diff:/diagnostics/*/farthestExpected/*`

## 入力（JSON escape）

```json
""
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = ANY
  L0 ::= '('{2};
  L1 ::= '=' @v6 L0* % '\n' @v7 | ')'+ % ')';
  @mapping(Node.A, params=[w]) VA ::= 'p' @w 'x';
  @mapping(Node.B, params=[w]) VB ::= 'q' @w T0;
  @mapping(Node) @commonField(w) Node ::= VA | VB;
  @whitespace M0 ::= ( '*'{1,} @v8 { L1 @v9 } @v10 ) | { '+' ',' @v11 } @v12 [ [ L0 @v13 ] ERROR('fuzz error') ];
  @mapping(C1) M1 ::= M0+ | L0 [ 'あ' '\n' ] '😀'+;
  @mapping(C2, params=[v14, v15, v16, v17, v18, v19, v20]) @interleave(profile=javaStyle) M2 ::= 'あ' @v14 'あ'* % '\n' @v15 'ab'? | ( T0 )? % 'b' @v16 | L1 T0 @v17 [ ERROR('fuzz error')+ @v18 T0 @v19 [ 'b'{2} @v20 ] ];
  @mapping(Bin0, params=[left, op, right]) @leftAssoc @precedence(level=90) Op0 ::= L0 @left { '-' @op L0 @right };
  @root Root ::= Op0 @head;
}
```

## 観測差（左 Java / 右 Rust）

- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/diagnostics/0/farthestExpected/3`: java=`"Repeat"` rust=`"RootParser"`
- `/diagnostics/0/farthestExpected/4`: java=`"RootParser"` rust=`"__CaptureSite"`
- `/diagnostics/0/farthestExpected/5`: java=`"__CaptureSite"` rust=`<欠落>`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/references`: java=`[]` rust=`<欠落>`

## Java の不変条件違反

- `allconsumed-consistency`: allConsumed=false だが consumed=0 / 入力長=0

## Rust の不変条件違反

- `allconsumed-consistency`: allConsumed=false だが consumed=0 / 入力長=0

## 応答

```json
java = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"consumed":0,"declarations":[],"diagnostics":[{"expected":["'('","L0Parser","Op0Parser","Repeat","RootParser","__CaptureSite"],"farthestExpected":["'('","L0Parser","Op0Parser","Repeat","RootParser","__CaptureSite"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose"}],"id":0,"lexical":[],"mappingError":null,"matched":0,"nativeFailure":{"consumed":0,"expected":["'('","L0Parser","Op0Parser","Repeat","RootParser","__CaptureSite"],"matched":0,"offset":0},"parsedStatus":"failed","prefixCaptures":[],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"value":null}
rust = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"catalogs":[],"consumed":0,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::L0","expected":["(","expression"],"farthestExpected":["'('","L0Parser","Op0Parser","RootParser","__CaptureSite"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose","rulePath":["Fuzz::Root","Fuzz::Op0","Fuzz::L0"]}],"hints":[{"deepestRule":"Fuzz::L0","expected":["(","expression"],"offset":0,"rulePath":["Fuzz::Root","Fuzz::Op0","Fuzz::L0"]}],"id":0,"lexical":[],"mappingError":null,"matched":0,"parsedStatus":"failed","prefixCaptures":[],"recoveredRegions":[],"references":[],"scope":{"events":[],"mode":[]},"scopeDepth":0,"value":null}
```
