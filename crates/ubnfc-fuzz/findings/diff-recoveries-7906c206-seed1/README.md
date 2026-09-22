# diff-recoveries-7906c206-seed1

seed: 1 / 入力 0 code point / 縮小 5 回

## signature

`diff:/recoveries`

## 入力（JSON escape）

```json
""
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

- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/diagnostics/0`: java=`{"expectedNonEmpty":true,"farthestExpected":["'#'","'('","','","ANY","RootRepeat0Parser"],"farthestOffset":0,"kind":"syntax","length":null,"message":null,"severity":null}` rust=`<欠落>`
- `/mappingError/present`: java=`false` rust=`true`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/references`: java=`[]` rust=`<欠落>`
- `/value`: java=`{"<unobserved>":"value"}` rust=`null`

## 応答

```json
java = {"accepted":true,"allConsumed":true,"ast":null,"captures":[{"name":"head","span":[0,0],"text":""}],"consumed":0,"declarations":[],"diagnostics":[{"expected":["'#'","'('","','","ANY","RootRepeat0Parser"],"farthestExpected":["'#'","'('","','","ANY","RootRepeat0Parser"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose"}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:309:311:body/0/tokenRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,0],"token":""},{"completionOrder":1,"exprId":"expr:grammar.ubnf:309:346:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,0],"token":null}],"mappingError":null,"matched":0,"nativeFailure":{"consumed":0,"expected":["'#'","'('","','","ANY","RootRepeat0Parser"],"matched":0,"offset":0},"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,0],"text":""}],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0}
rust = {"accepted":true,"allConsumed":true,"ast":null,"captures":[{"name":"head","span":[0,0],"text":""}],"catalogs":[],"consumed":0,"declarations":[],"diagnostics":[],"hints":[{"deepestRule":"Fuzz::L2","expected":["(",","],"offset":0,"rulePath":["Fuzz::Root","Fuzz::L2"]}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:309:346:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,0]}],"mappingError":"java.lang.IllegalArgumentException: entry must produce one AST value","matched":0,"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,0],"text":""}],"recoveredRegions":[],"references":[],"scope":{"events":[],"mode":[]},"scopeDepth":0,"value":null}
```
