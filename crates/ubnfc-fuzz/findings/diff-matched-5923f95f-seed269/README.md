# diff-matched-5923f95f-seed269

seed: 269 / 入力 2 code point / 縮小 5 回

## signature

`diff:/matched`

## 入力（JSON escape）

```json
"c;"
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = EOF
  @recovery(AUTO) M0 ::= T0 T0? 'c';
  @root @mapping(Top, params=[head]) Root ::= M0 @head ';' @head;
}
```

## 観測差（左 Java / 右 Rust）

- `/accepted`: java=`false` rust=`true`
- `/allConsumed`: java=`false` rust=`true`
- `/ast`: java=`null` rust=`{"fields":{"head":[";"]},"span":[0,2],"type":"Top"}`
- `/captures/0`: java=`<欠落>` rust=`{"binding":null,"name":"head","span":[0,1],"text":"c"}`
- `/captures/1`: java=`<欠落>` rust=`{"binding":null,"name":"head","span":[1,2],"text":";"}`
- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/consumed`: java=`0` rust=`2`
- `/diagnostics/0/expected`: java=`<欠落>` rust=`["T0"]`
- `/diagnostics/0/expectedNonEmpty`: java=`true` rust=`<欠落>`
- `/diagnostics/0/farthestExpected/0`: java=`"';'"` rust=`"T0"`
- `/diagnostics/0/farthestOffset`: java=`2` rust=`0`
- `/diagnostics/0/kind`: java=`"syntax"` rust=`"recovery"`
- `/diagnostics/0/length`: java=`null` rust=`1`
- `/diagnostics/0/offset`: java=`<欠落>` rust=`0`
- `/diagnostics/0/severity`: java=`null` rust=`"ERROR"`
- `/matched`: java=`0` rust=`2`
- `/parsedStatus`: java=`"failed"` rust=`"succeeded"`
- `/prefixCaptures/0`: java=`<欠落>` rust=`{"binding":"Root:0","name":"head","span":[0,1],"text":"c"}`
- `/prefixCaptures/1`: java=`<欠落>` rust=`{"binding":"Root:1","name":"head","span":[1,2],"text":";"}`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/references`: java=`[]` rust=`<欠落>`
- `/value`: java=`null` rust=`{"<unobserved>":"value"}`

## 応答

```json
java = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"consumed":0,"declarations":[],"diagnostics":[{"expected":["';'"],"farthestExpected":["';'"],"farthestOffset":2,"kind":"syntax","offset":2,"origin":"Mapper.diagnose"}],"id":0,"lexical":[],"mappingError":null,"matched":0,"nativeFailure":{"consumed":2,"expected":["';'"],"matched":2,"offset":2},"parsedStatus":"failed","prefixCaptures":[],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"value":null}
rust = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":[";"]},"span":[0,2],"type":"Top"},"captures":[{"name":"head","span":[0,1],"text":"c"},{"name":"head","span":[1,2],"text":";"}],"catalogs":[],"consumed":2,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::M0","expected":["T0"],"farthestExpected":["T0"],"farthestOffset":0,"kind":"recovery","length":1,"offset":0,"origin":"recovery","recoveryId":0,"ruleId":"Fuzz::M0","rulePath":["Fuzz::M0"],"severity":"ERROR"}],"hints":[{"deepestRule":"Fuzz::M0","expected":["T0"],"offset":0,"rulePath":["Fuzz::Root","Fuzz::M0"]}],"id":0,"lexical":[{"completionOrder":1,"exprId":"expr:grammar.ubnf:115:127:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,2]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:124:127:body/1/literal","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[1,2]}],"mappingError":null,"matched":2,"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,1],"text":"c"},{"binding":"Root:1","name":"head","span":[1,2],"text":";"}],"recoveredRegions":[{"captureOccurrenceIds":[0],"mode":"AUTO","ruleId":"Fuzz::M0","span":[0,1],"syncSpan":[1,2]}],"references":[],"scope":{"events":[],"mode":[]},"scopeDepth":0,"texts":[]}
```
