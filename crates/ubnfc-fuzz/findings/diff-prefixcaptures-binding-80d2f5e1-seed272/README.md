# diff-prefixcaptures-binding-80d2f5e1-seed272

seed: 272 / 入力 7 code point / 縮小 7 回

## signature

`diff:/prefixCaptures/*/binding`

## 入力（JSON escape）

```json
"\n,ああ==あ"
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = ANY
  token T1 = LOOKAHEAD('a')
  L0 ::= '\n' ',' 'あ';
  @mapping(C0) @whitespace @recovery(AUTO) M0 ::= ERROR('fuzz error') T1{0,};
  @root Root ::= L0 @head [ 'あ' '='{2,4} @v5 M0 ]+ @head;
}
```

## 観測差（左 Java / 右 Rust）

- `/allConsumed`: java=`true` rust=`false`
- `/captures/1/name`: java=`"v5"` rust=`"head"`
- `/captures/1/span/0`: java=`4` rust=`3`
- `/captures/1/span/1`: java=`5` rust=`3`
- `/captures/1/text`: java=`"="` rust=`""`
- `/captures/2`: java=`{"binding":null,"name":"v5","span":[5,6],"text":"="}` rust=`<欠落>`
- `/captures/3`: java=`{"binding":null,"name":"head","span":[3,7],"text":"あ==あ"}` rust=`<欠落>`
- `/captures/4`: java=`{"binding":null,"name":"head","span":[7,7],"text":""}` rust=`<欠落>`
- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/consumed`: java=`7` rust=`3`
- `/diagnostics/0/expected/0`: java=`"' '"` rust=`"end of input"`
- `/diagnostics/0/expected/1`: java=`"'*/'"` rust=`<欠落>`
- `/diagnostics/0/expected/2`: java=`"'/*'"` rust=`<欠落>`
- `/diagnostics/0/expected/3`: java=`"'//'"` rust=`<欠落>`
- `/diagnostics/0/expected/4`: java=`"'あ'"` rust=`<欠落>`
- `/diagnostics/0/expected/5`: java=`"fuzz error"` rust=`<欠落>`
- `/diagnostics/0/farthestExpected/4`: java=`"'あ'"` rust=`"'='"`
- `/diagnostics/0/kind`: java=`"recovery"` rust=`"trailing_input"`
- `/diagnostics/0/length`: java=`1` rust=`null`
- `/diagnostics/0/offset`: java=`6` rust=`3`
- `/diagnostics/0/severity`: java=`"ERROR"` rust=`null`
- `/matched`: java=`7` rust=`3`
- `/prefixCaptures/1/binding`: java=`"Root:2"` rust=`"Root:1"`
- `/prefixCaptures/1/name`: java=`"v5"` rust=`"head"`
- `/prefixCaptures/1/span/0`: java=`4` rust=`3`
- `/prefixCaptures/1/span/1`: java=`5` rust=`3`
- `/prefixCaptures/1/text`: java=`"="` rust=`""`
- `/prefixCaptures/2`: java=`{"binding":"Root:2","name":"v5","span":[5,6],"text":"="}` rust=`<欠落>`
- `/prefixCaptures/3`: java=`{"binding":"Root:1","name":"head","span":[3,7],"text":"あ==あ"}` rust=`<欠落>`
- `/prefixCaptures/4`: java=`{"binding":"Root:1","name":"head","span":[7,7],"text":""}` rust=`<欠落>`
- `/recoveries`: java=`[{"captureOccurrences":[3],"mode":"AUTO","ruleId":"Fuzz::M0","skippedSpan":[6,6],"span":[6,7],"syncPattern":"あ","syncSpan":[6,7]}]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/references`: java=`[]` rust=`<欠落>`
- `/value`: java=`{"<unobserved>":"value"}` rust=`null`

## 応答

```json
java = {"accepted":true,"allConsumed":true,"ast":null,"captures":[{"name":"head","span":[0,3],"text":"\n,あ"},{"name":"v5","span":[4,5],"text":"="},{"name":"v5","span":[5,6],"text":"="},{"name":"head","span":[3,7],"text":"あ==あ"},{"name":"head","span":[7,7],"text":""}],"consumed":7,"declarations":[],"diagnostics":[{"expected":["' '","'*/'","'/*'","'//'","'あ'","fuzz error"],"farthestExpected":["' '","'*/'","'/*'","'//'","'あ'","fuzz error"],"farthestOffset":6,"kind":"recovery","length":1,"offset":6,"origin":"recovery","ruleId":"Fuzz::M0","severity":"ERROR"}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:69:73:body/0/literal","occurrenceId":3,"parentOccurrenceId":2,"ruleId":"Fuzz::L0","span":[0,1],"token":"\n"},{"completionOrder":1,"exprId":"expr:grammar.ubnf:74:77:body/1/literal","occurrenceId":4,"parentOccurrenceId":2,"ruleId":"Fuzz::L0","span":[1,2],"token":","},{"completionOrder":2,"exprId":"expr:grammar.ubnf:78:81:body/2/literal","occurrenceId":5,"parentOccurrenceId":2,"ruleId":"Fuzz::L0","span":[2,3],"token":"あ"},{"completionOrder":3,"exprId":"expr:grammar.ubnf:69:81:body/seq","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::L0","span":[0,3],"token":null},{"completionOrder":4,"exprId":"expr:grammar.ubnf:178:180:body/0/ruleRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,3],"token":null},{"completionOrder":5,"exprId":"expr:grammar.ubnf:189:192:body/1/0/0/0/literal","occurrenceId":9,"parentOccurrenceId":8,"ruleId":"Fuzz::Root","span":[3,4],"token":"あ"},{"completionOrder":6,"exprId":"expr:grammar.ubnf:193:196:body/1/0/0/1/0/literal","occurrenceId":11,"parentOccurrenceId":10,"ruleId":"Fuzz::Root","span":[4,5],"token":"="},{"completionOrder":7,"exprId":"expr:grammar.ubnf:193:196:body/1/0/0/1/repeat/lower/helper/0","occurrenceId":10,"parentOccurrenceId":8,"ruleId":"Fuzz::Root","span":[4,5],"token":null},{"completionOrder":8,"exprId":"expr:grammar.ubnf:193:196:body/1/0/0/1/0/literal","occurrenceId":13,"parentOccurrenceId":12,"ruleId":"Fuzz::Root","span":[5,6],"token":"="},{"completionOrder":9,"exprId":"expr:grammar.ubnf:193:196:body/1/0/0/1/repeat/lower/helper/0","occurrenceId":12,"parentOccurrenceId":8,"ruleId":"Fuzz::Root","span":[5,6],"token":null},{"completionOrder":10,"exprId":"expr:grammar.ubnf:206:208:body/1/0/0/2/ruleRef","occurrenceId":14,"parentOccurrenceId":8,"ruleId":"Fuzz::Root","span":[6,7],"token":null},{"completionOrder":11,"exprId":"expr:grammar.ubnf:189:208:body/1/0/0/seq","occurrenceId":8,"parentOccurrenceId":7,"ruleId":"Fuzz::Root","span":[3,7],"token":null},{"completionOrder":12,"exprId":"expr:grammar.ubnf:187:210:body/1/0/optional","occurrenceId":7,"parentOccurrenceId":6,"ruleId":"Fuzz::Root","span":[3,7],"token":null},{"completionOrder":13,"exprId":"expr:grammar.ubnf:187:210:body/1/repeat/lower/helper/0","occurrenceId":6,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[3,7],"token":null},{"completionOrder":14,"exprId":"expr:grammar.ubnf:187:210:body/1/0/optional","occurrenceId":16,"parentOccurrenceId":15,"ruleId":"Fuzz::Root","span":[7,7],"token":null},{"completionOrder":15,"exprId":"expr:grammar.ubnf:187:210:body/1/repeat/lower/helper/0","occurrenceId":15,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[7,7],"token":null},{"completionOrder":16,"exprId":"expr:grammar.ubnf:178:211:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,7],"token":null}],"mappingError":null,"matched":7,"nativeFailure":{"consumed":7,"expected":["'あ'"],"matched":7,"offset":7},"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,3],"text":"\n,あ"},{"binding":"Root:2","name":"v5","span":[4,5],"text":"="},{"binding":"Root:2","name":"v5","span":[5,6],"text":"="},{"binding":"Root:1","name":"head","span":[3,7],"text":"あ==あ"},{"binding":"Root:1","name":"head","span":[7,7],"text":""}],"recoveries":[{"captureOccurrences":[3],"mode":"AUTO","ruleId":"Fuzz::M0","skippedSpan":[6,6],"span":[6,7],"syncPattern":"あ","syncSpan":[6,7]}],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0}
rust = {"accepted":true,"allConsumed":false,"ast":null,"captures":[{"name":"head","span":[0,3],"text":"\n,あ"},{"name":"head","span":[3,3],"text":""}],"catalogs":[],"consumed":3,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::M0","expected":["end of input"],"farthestExpected":["' '","'*/'","'/*'","'//'","'='","fuzz error"],"farthestOffset":6,"kind":"trailing_input","offset":3,"origin":"Mapper.diagnose","rulePath":["Fuzz::Root","Fuzz::M0"]}],"hints":[{"deepestRule":"Fuzz::M0","expected":["=","fuzz error"],"offset":6,"rulePath":["Fuzz::Root","Fuzz::M0"]}],"id":0,"lexical":[{"completionOrder":4,"exprId":"expr:grammar.ubnf:178:211:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,3]},{"completionOrder":3,"exprId":"expr:grammar.ubnf:69:81:body/seq","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::L0","span":[0,3]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:69:73:body/0/literal","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::L0","span":[0,1]},{"completionOrder":1,"exprId":"expr:grammar.ubnf:74:77:body/1/literal","occurrenceId":3,"parentOccurrenceId":1,"ruleId":"Fuzz::L0","span":[1,2]},{"completionOrder":2,"exprId":"expr:grammar.ubnf:78:81:body/2/literal","occurrenceId":4,"parentOccurrenceId":1,"ruleId":"Fuzz::L0","span":[2,3]}],"mappingError":null,"matched":3,"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,3],"text":"\n,あ"},{"binding":"Root:1","name":"head","span":[3,3],"text":""}],"recoveredRegions":[],"references":[],"scope":{"events":[],"mode":[]},"scopeDepth":0,"value":null}
```
