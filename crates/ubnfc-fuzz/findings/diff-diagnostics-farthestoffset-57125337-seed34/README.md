# diff-diagnostics-farthestoffset-57125337-seed34

seed: 34 / 入力 1 code point / 縮小 11 回

## signature

`diff:/diagnostics/*/farthestOffset`

## 入力（JSON escape）

```json
"*"
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = LOOKAHEAD('a')
  token T1 = UNTIL('#')
  token T2 = LOOKAHEAD('a')
  L0 ::= ;
  @interleave(profile=javaStyle) @recovery(AUTO) @longestChoice M0 ::= 'b'+ | ERROR('fuzz error') @v1 ( [ ',' 'あ' ] | { 'ab' @v2 } L0 ) T2 | T2 @v3 ( '😀' | T1+ @v4 ) '(';
  @doc('fuzz 😀') M1 ::= ( L0 ( T2 ';' T2 ) | '-' @v5 ':' 'あ'{2,} @v6 ) @v7;
  @mapping(C2, params=[v8]) @interleave(profile=javaStyle) M2 ::= M1 @v8 M0?;
  @mapping(Bin0, params=[left, op, right]) @leftAssoc @precedence(level=90) Op0 ::= M0 @left { '*' @op M0 @right };
  @root @mapping(Top, params=[head]) Root ::= Op0 @head ERROR('fuzz error') @head;
}
```

## 観測差（左 Java / 右 Rust）

- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/diagnostics/0/farthestExpected/0`: java=`"'*'"` rust=`"' '"`
- `/diagnostics/0/farthestExpected/1`: java=`"fuzz error"` rust=`"'('"`
- `/diagnostics/0/farthestExpected/2`: java=`<欠落>` rust=`"'*/'"`
- `/diagnostics/0/farthestExpected/3`: java=`<欠落>` rust=`"'/*'"`
- `/diagnostics/0/farthestExpected/4`: java=`<欠落>` rust=`"'//'"`
- `/diagnostics/0/farthestExpected/5`: java=`<欠落>` rust=`"'a'"`
- `/diagnostics/0/farthestExpected/6`: java=`<欠落>` rust=`"'b'"`
- `/diagnostics/0/farthestExpected/7`: java=`<欠落>` rust=`"'😀'"`
- `/diagnostics/0/farthestExpected/8`: java=`<欠落>` rust=`"Op0Parser"`
- `/diagnostics/0/farthestExpected/9`: java=`<欠落>` rust=`"__CaptureSite"`
- `/diagnostics/0/farthestExpected/10`: java=`<欠落>` rust=`"fuzz error"`
- `/diagnostics/0/farthestOffset`: java=`1` rust=`0`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/references`: java=`[]` rust=`<欠落>`

## 応答

```json
java = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"consumed":0,"declarations":[],"diagnostics":[{"expected":["'*'","fuzz error"],"farthestExpected":["'*'","fuzz error"],"farthestOffset":1,"kind":"syntax","offset":1,"origin":"Mapper.diagnose"}],"id":0,"lexical":[],"mappingError":null,"matched":0,"nativeFailure":{"consumed":1,"expected":["'*'","fuzz error"],"matched":1,"offset":1},"parsedStatus":"failed","prefixCaptures":[],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"value":null}
rust = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"catalogs":[],"consumed":0,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::M0","expected":["b","expression","fuzz error","T2"],"farthestExpected":["' '","'('","'*/'","'/*'","'//'","'a'","'b'","'😀'","Op0Parser","__CaptureSite","fuzz error"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose","rulePath":["Fuzz::Root","Fuzz::Op0","Fuzz::M0"]}],"hints":[{"deepestRule":"Fuzz::M0","expected":["b","expression","fuzz error","T2"],"offset":0,"rulePath":["Fuzz::Root","Fuzz::Op0","Fuzz::M0"]}],"id":0,"lexical":[],"mappingError":null,"matched":0,"parsedStatus":"failed","prefixCaptures":[],"recoveredRegions":[],"references":[],"scope":{"events":[],"mode":[]},"scopeDepth":0,"value":null}
```
