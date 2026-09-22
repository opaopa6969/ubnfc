# diff-captures-ccc0d8bb-seed1

seed: 1 / 入力 1 code point / 縮小 9 回

## signature

`diff:/captures/*`

## 入力（JSON escape）

```json
"#"
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = UNTIL('#')
  token T1 = NEGATIVE_LOOKAHEAD('#')
  token T2 = LOOKAHEAD('a')
  L0 ::= ;
  @enum L1 ::= 'alpha' | 'beta';
  L2 ::= '(';
  M0 ::= { '+' } '('* @v2 '\n' @v3 | [ '('+ @v4 ] | L0{2,3} @v5 { ( L1 '😀'* % ':' @v6 '😀'? @v7 ) ( '(' @v8 L1 | L1 ){2} @v9 L1 };
  @root Root ::= T0 @head L2? @head ( ',' @tail M0* )* @tail;
}
```

## 観測差（左 Java / 右 Rust）

- `/captures/0`: java=`<欠落>` rust=`{"binding":null,"name":"head","span":[0,0],"text":""}`
- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/references`: java=`[]` rust=`<欠落>`

## 応答

```json
java = {"accepted":true,"allConsumed":false,"ast":null,"captures":[],"consumed":0,"declarations":[],"diagnostics":[{"expected":["end of input"],"farthestExpected":["'('","','","RootRepeat0Parser"],"farthestOffset":1,"kind":"trailing_input","offset":0,"origin":"Mapper.diagnose"}],"id":0,"lexical":[],"mappingError":null,"matched":1,"nativeFailure":{"consumed":0,"expected":["'('","','","RootRepeat0Parser"],"matched":1,"offset":1},"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,0],"text":""}],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"value":null}
rust = {"accepted":true,"allConsumed":false,"ast":null,"captures":[{"name":"head","span":[0,0],"text":""}],"catalogs":[],"consumed":0,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::L2","expected":["end of input"],"farthestExpected":["'('","','","RootRepeat0Parser"],"farthestOffset":1,"kind":"trailing_input","offset":0,"origin":"Mapper.diagnose","rulePath":["Fuzz::Root","Fuzz::L2"]}],"hints":[{"deepestRule":"Fuzz::L2","expected":["(",","],"offset":0,"rulePath":["Fuzz::Root","Fuzz::L2"]}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:309:346:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,0]}],"mappingError":null,"matched":1,"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,0],"text":""}],"recoveredRegions":[],"references":[],"scope":{"events":[],"mode":[]},"scopeDepth":0,"value":null}
```
