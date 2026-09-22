# diff-prefixcaptures-span-039546d5-seed269

seed: 269 / 入力 3 code point / 縮小 9 回

## signature

`diff:/prefixCaptures/*/span/*`

## 入力（JSON escape）

```json
" ;;"
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = EOF
  @recovery(AUTO) M0 ::= T0? 'c';
  @root @mapping(Top, params=[head]) Root ::= M0 @head ';' @head;
}
```

## 観測差（左 Java / 右 Rust）

- `/allConsumed`: java=`true` rust=`false`
- `/ast`: java=`{"fields":{"head":[";",";"]},"span":[0,3],"type":"Top"}` rust=`null`
- `/captures/0/span/1`: java=`2` rust=`1`
- `/captures/0/text`: java=`" ;"` rust=`" "`
- `/captures/1/span/0`: java=`2` rust=`1`
- `/captures/1/span/1`: java=`3` rust=`2`
- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/consumed`: java=`3` rust=`2`
- `/diagnostics/0/expected/0`: java=`"';'"` rust=`"T0"`
- `/diagnostics/0/expected/1`: java=`"'c'"` rust=`"c"`
- `/diagnostics/0/expected/2`: java=`"EndOfSourceParser"` rust=`<欠落>`
- `/diagnostics/0/farthestExpected/0`: java=`"';'"` rust=`"T0"`
- `/diagnostics/0/farthestExpected/1`: java=`"'c'"` rust=`"c"`
- `/diagnostics/0/farthestExpected/2`: java=`"EndOfSourceParser"` rust=`<欠落>`
- `/diagnostics/0/length`: java=`2` rust=`1`
- `/diagnostics/1`: java=`<欠落>` rust=`{"expected":["end of input"],"farthestExpected":["'c'","EndOfSourceParser"],"farthestOffset":0,"kind":"trailing_input","length":null,"message":null,"offset":2,"severity":null}`
- `/matched`: java=`3` rust=`2`
- `/prefixCaptures/0/span/1`: java=`2` rust=`1`
- `/prefixCaptures/0/text`: java=`" ;"` rust=`" "`
- `/prefixCaptures/1/span/0`: java=`2` rust=`1`
- `/prefixCaptures/1/span/1`: java=`3` rust=`2`
- `/recoveries`: java=`[{"captureOccurrences":[0],"mode":"AUTO","ruleId":"Fuzz::M0","skippedSpan":[0,1],"span":[0,2],"syncPattern":";","syncSpan":[1,2]}]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/references`: java=`[]` rust=`<欠落>`
- `/value`: java=`{"<unobserved>":"value"}` rust=`null`

## 応答

```json
java = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":[";",";"]},"span":[0,3],"type":"Top"},"captures":[{"name":"head","span":[0,2],"text":" ;"},{"name":"head","span":[2,3],"text":";"}],"consumed":3,"declarations":[],"diagnostics":[{"expected":["';'","'c'","EndOfSourceParser"],"farthestExpected":["';'","'c'","EndOfSourceParser"],"farthestOffset":0,"kind":"recovery","length":2,"offset":0,"origin":"recovery","ruleId":"Fuzz::M0","severity":"ERROR"}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:112:114:body/0/ruleRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,2],"token":null},{"completionOrder":1,"exprId":"expr:grammar.ubnf:121:124:body/1/literal","occurrenceId":2,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[2,3],"token":";"},{"completionOrder":2,"exprId":"expr:grammar.ubnf:112:124:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,3],"token":null}],"mappingError":null,"matched":3,"nativeFailure":{"consumed":1,"expected":["';'","'c'","EndOfSourceParser"],"matched":1,"offset":0},"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,2],"text":" ;"},{"binding":"Root:1","name":"head","span":[2,3],"text":";"}],"recoveries":[{"captureOccurrences":[0],"mode":"AUTO","ruleId":"Fuzz::M0","skippedSpan":[0,1],"span":[0,2],"syncPattern":";","syncSpan":[1,2]}],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"texts":[[0,2,";"],[2,3,";"]]}
rust = {"accepted":true,"allConsumed":false,"ast":null,"captures":[{"name":"head","span":[0,1],"text":" "},{"name":"head","span":[1,2],"text":";"}],"catalogs":[],"consumed":2,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::M0","expected":["T0","c"],"farthestExpected":["T0","c"],"farthestOffset":0,"kind":"recovery","length":1,"offset":0,"origin":"recovery","recoveryId":0,"ruleId":"Fuzz::M0","rulePath":["Fuzz::M0"],"severity":"ERROR"},{"deepestRule":"Fuzz::M0","expected":["end of input"],"farthestExpected":["'c'","EndOfSourceParser"],"farthestOffset":0,"kind":"trailing_input","offset":2,"origin":"Mapper.diagnose","rulePath":["Fuzz::Root","Fuzz::M0"]}],"hints":[{"deepestRule":"Fuzz::M0","expected":["T0","c"],"offset":0,"rulePath":["Fuzz::Root","Fuzz::M0"]}],"id":0,"lexical":[{"completionOrder":1,"exprId":"expr:grammar.ubnf:112:124:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,2]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:121:124:body/1/literal","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[1,2]}],"mappingError":null,"matched":2,"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,1],"text":" "},{"binding":"Root:1","name":"head","span":[1,2],"text":";"}],"recoveredRegions":[{"captureOccurrenceIds":[0],"mode":"AUTO","ruleId":"Fuzz::M0","span":[0,1],"syncSpan":[1,2]}],"references":[],"scope":{"events":[],"mode":[]},"scopeDepth":0,"value":null}
```
