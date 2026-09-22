# rust-allconsumed-consistency-0a922f49-seed3

seed: 3 / 入力 0 code point / 縮小 6 回

## signature

`rust:allconsumed-consistency`

## 入力（JSON escape）

```json
""
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = NEGATIVE_LOOKAHEAD('#')
  token T1 = org.unlaxer.parser.elementary.SingleQuotedParser
  L1 ::= ERROR('fuzz error');
  @mapping(C0, params=[v3, v5, v4]) @scopeTree(mode=dynamic) @declares(symbol=v3) @backref(name=v4) M0 ::= ( 'b' L1+ ) T0 T0 | T0 @v3 ( '😀'? 'ab'{0,} % ',' { '*' @v4 }+ @v5 | 'c'{1,2} )? % ')' ERROR('fuzz error');
  @whitespace(none) @scopeTree(mode=dynamic) @declares(symbol=v6) @catalog(context='fuzz') M1 ::= ERROR('fuzz error') @v6 ';'? % ';';
  @mapping(Bin0, params=[left, op, right]) @rightAssoc @precedence(level=90) Op0 ::= M1 @left { '-' @op Op0 @right };
  @root @mapping(Top, params=[head, v7, v8, tail]) @eval(kind='binary', strategy='manual') Root ::= Op0 @head ( ':'+ T1 @v7 | 'ab'? M1 @v8 ){1,3} @head ( ',' @tail 'b' )* @tail;
}
```

## 観測差（左 Java / 右 Rust）

- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[{"captures":["v6"],"context":"fuzz","rule":"M1"}]`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/mode/0`: java=`<欠落>` rust=`"dynamic"`
- `/scope/references`: java=`[]` rust=`<欠落>`

## Java の不変条件違反

- `allconsumed-consistency`: allConsumed=false だが consumed=0 / 入力長=0

## Rust の不変条件違反

- `allconsumed-consistency`: allConsumed=false だが consumed=0 / 入力長=0

## 応答

```json
java = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"consumed":0,"declarations":[],"diagnostics":[{"expected":["';'","Op0Parser","RootParser","__CaptureSite","fuzz error"],"farthestExpected":["';'","Op0Parser","RootParser","__CaptureSite","fuzz error"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose"}],"id":0,"lexical":[],"mappingError":null,"matched":0,"nativeFailure":{"consumed":0,"expected":["';'","Op0Parser","RootParser","__CaptureSite","fuzz error"],"matched":0,"offset":0},"parsedStatus":"failed","prefixCaptures":[],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"value":null}
rust = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"catalogs":[{"captures":["v6"],"context":"fuzz","rule":"M1"}],"consumed":0,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::M1","expected":["fuzz error"],"farthestExpected":["';'","Op0Parser","RootParser","__CaptureSite","fuzz error"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose","rulePath":["Fuzz::Root","Fuzz::Op0","Fuzz::M1"]}],"hints":[{"deepestRule":"Fuzz::M1","expected":["fuzz error"],"offset":0,"rulePath":["Fuzz::Root","Fuzz::Op0","Fuzz::M1"]}],"id":0,"lexical":[],"mappingError":null,"matched":0,"parsedStatus":"failed","prefixCaptures":[],"recoveredRegions":[],"references":[],"scope":{"events":[],"mode":["dynamic"]},"scopeDepth":0,"value":null}
```
