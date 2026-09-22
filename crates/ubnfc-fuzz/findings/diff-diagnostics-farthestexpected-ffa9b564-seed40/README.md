# diff-diagnostics-farthestexpected-ffa9b564-seed40

seed: 40 / 入力 5 code point / 縮小 8 回

## signature

`diff:/diagnostics/*/farthestExpected`

## 入力（JSON escape）

```json
"7(ab😀"
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = org.unlaxer.parser.elementary.SingleQuotedParser
  token T1 = NumberParser
  L0 ::= ;
  @mapping(Node.A, params=[w]) VA ::= 'p' @w T1;
  @mapping(Node.B, params=[w]) VB ::= 'q' @w T0{0,2};
  @mapping(Node) Node ::= VA | VB;
  M0 ::= '+' ')' @v1 Node | ')'+ ';' @v2 '(';
  @mapping(C1, params=[v3, v4, v5]) @recovery(sync=';') @predictiveChoice @scopeTree(mode=dynamic) @backref(name=v5) M1 ::= 'あ' @v3 { '\n' } | { T1 @v4 '('* @v5 'ab' } L0*;
  @recovery(AUTO) M2 ::= 'c' { '😀'+ '😀'* @v6 };
  @root @mapping(Top, params=[head, tail]) Root ::= M1 @head '😀' @head ( ',' @tail [ '😀' T1* ] )* @tail;
}
```

## 観測差（左 Java / 右 Rust）

- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/diagnostics/0/expected`: java=`<欠落>` rust=`[]`
- `/diagnostics/0/expectedNonEmpty`: java=`true` rust=`<欠落>`
- `/diagnostics/0/farthestExpected`: java=`["','","RootRepeat0Parser"]` rust=`null`
- `/diagnostics/0/farthestOffset`: java=`5` rust=`null`
- `/diagnostics/0/kind`: java=`"syntax"` rust=`"WARNING"`
- `/diagnostics/0/length`: java=`null` rust=`1`
- `/diagnostics/0/message`: java=`null` rust=`"未定義のシンボル: '('"`
- `/diagnostics/0/offset`: java=`<欠落>` rust=`1`
- `/diagnostics/0/severity`: java=`null` rust=`"WARNING"`
- `/diagnostics/1`: java=`{"expected":[],"farthestExpected":null,"farthestOffset":null,"kind":"WARNING","length":1,"message":"未定義のシンボル: '('","offset":1,"severity":"WARNING"}` rust=`<欠落>`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[{"expected":null,"farthestExpected":null,"farthestOffset":null,"kind":null,"length":1,"message":"未定義のシンボル: '('","offset":1,"severity":"WARNING"}]` rust=`<欠落>`
- `/scope/events/0/name`: java=`"M1"` rust=`null`
- `/scope/events/1/name`: java=`"M1"` rust=`null`
- `/scope/references`: java=`[{"length":1,"name":"(","offset":1}]` rust=`<欠落>`

## 応答

```json
java = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":[{"fields":{"v3":null,"v4":[7],"v5":["("]},"span":[0,4],"type":"C1"},"😀"],"tail":[]},"span":[0,5],"type":"Top"},"captures":[{"name":"v4","span":[0,1],"text":"7"},{"name":"v5","span":[1,2],"text":"("},{"name":"head","span":[0,4],"text":"7(ab"},{"name":"head","span":[4,5],"text":"😀"}],"consumed":5,"declarations":[],"diagnostics":[{"expected":["','","RootRepeat0Parser"],"farthestExpected":["','","RootRepeat0Parser"],"farthestOffset":5,"kind":"syntax","offset":5,"origin":"Mapper.diagnose"},{"expected":[],"kind":"WARNING","length":1,"message":"未定義のシンボル: '('","offset":1,"origin":"ScopeStore","severity":"WARNING"}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:443:445:body/1/0/0/0/tokenRef","occurrenceId":6,"parentOccurrenceId":5,"ruleId":"Fuzz::M1","span":[0,1],"token":"7"},{"completionOrder":1,"exprId":"expr:grammar.ubnf:450:453:body/1/0/0/1/0/0/literal","occurrenceId":8,"parentOccurrenceId":7,"ruleId":"Fuzz::M1","span":[1,2],"token":"("},{"completionOrder":2,"exprId":"expr:grammar.ubnf:450:453:body/1/0/0/1/0/seq","occurrenceId":7,"parentOccurrenceId":5,"ruleId":"Fuzz::M1","span":[1,2],"token":null},{"completionOrder":3,"exprId":"expr:grammar.ubnf:459:463:body/1/0/0/2/literal","occurrenceId":9,"parentOccurrenceId":5,"ruleId":"Fuzz::M1","span":[2,4],"token":"ab"},{"completionOrder":4,"exprId":"expr:grammar.ubnf:443:463:body/1/0/0/seq","occurrenceId":5,"parentOccurrenceId":4,"ruleId":"Fuzz::M1","span":[0,4],"token":null},{"completionOrder":5,"exprId":"expr:grammar.ubnf:441:465:body/1/0/repeat","occurrenceId":4,"parentOccurrenceId":3,"ruleId":"Fuzz::M1","span":[0,4],"token":null},{"completionOrder":6,"exprId":"expr:grammar.ubnf:112:112:body/seq","occurrenceId":12,"parentOccurrenceId":11,"ruleId":"Fuzz::L0","span":[4,4],"token":null},{"completionOrder":7,"exprId":"expr:grammar.ubnf:466:468:body/1/1/0/ruleRef","occurrenceId":11,"parentOccurrenceId":10,"ruleId":"Fuzz::M1","span":[4,4],"token":null},{"completionOrder":8,"exprId":"expr:grammar.ubnf:466:469:body/1/1/repeat","occurrenceId":10,"parentOccurrenceId":3,"ruleId":"Fuzz::M1","span":[4,4],"token":null},{"completionOrder":9,"exprId":"expr:grammar.ubnf:441:469:body/1/seq","occurrenceId":3,"parentOccurrenceId":2,"ruleId":"Fuzz::M1","span":[0,4],"token":null},{"completionOrder":10,"exprId":"expr:grammar.ubnf:422:469:body/choice","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::M1","span":[0,4],"token":null},{"completionOrder":11,"exprId":"expr:grammar.ubnf:571:573:body/0/ruleRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,4],"token":null},{"completionOrder":12,"exprId":"expr:grammar.ubnf:580:583:body/1/literal","occurrenceId":13,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[4,5],"token":"😀"},{"completionOrder":13,"exprId":"expr:grammar.ubnf:571:616:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,5],"token":null}],"mappingError":null,"matched":5,"nativeFailure":{"consumed":5,"expected":["','","RootRepeat0Parser"],"matched":5,"offset":5},"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"M1:1","name":"v4","span":[0,1],"text":"7"},{"binding":"M1:2","name":"v5","span":[1,2],"text":"("},{"binding":"Root:0","name":"head","span":[0,4],"text":"7(ab"},{"binding":"Root:1","name":"head","span":[4,5],"text":"😀"}],"recoveries":[],"references":[{"length":1,"name":"(","offset":1}],"scope":{"declarations":[],"depth":0,"diagnostics":[{"length":1,"message":"未定義のシンボル: '('","offset":1,"severity":"WARNING"}],"events":[{"action":"enter","length":0,"mode":"dynamic","name":"M1","offset":0,"order":0},{"action":"leave","length":0,"mode":"dynamic","name":"M1","offset":4,"order":1},{"action":"reference","length":1,"mode":"dynamic","name":"(","offset":1,"order":2}],"mode":["dynamic"],"references":[{"length":1,"name":"(","offset":1}]},"scopeDepth":0,"texts":[[1,2,"("],[4,5,"😀"]]}
rust = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":[{"fields":{"v3":null,"v4":["7"],"v5":["("]},"span":[0,4],"type":"C1"},"😀"],"tail":[]},"span":[0,5],"type":"Top"},"captures":[{"name":"v4","span":[0,1],"text":"7"},{"name":"v5","span":[1,2],"text":"("},{"name":"head","span":[0,4],"text":"7(ab"},{"name":"head","span":[4,5],"text":"😀"}],"catalogs":[],"consumed":5,"declarations":[],"diagnostics":[{"expected":[],"kind":"WARNING","length":1,"message":"未定義のシンボル: '('","offset":1,"origin":"ScopeStore","severity":"WARNING"}],"hints":[{"deepestRule":"Fuzz::Root","expected":[","],"offset":5,"rulePath":["Fuzz::Root"]}],"id":0,"lexical":[{"completionOrder":6,"exprId":"expr:grammar.ubnf:571:616:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,5]},{"completionOrder":4,"exprId":"expr:grammar.ubnf:422:469:body/choice","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::M1","span":[0,4]},{"completionOrder":3,"exprId":"expr:grammar.ubnf:112:112:body/seq","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::L0","span":[4,4]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:443:445:body/1/0/0/0/tokenRef","occurrenceId":3,"parentOccurrenceId":1,"ruleId":"Fuzz::M1","span":[0,1]},{"completionOrder":1,"exprId":"expr:grammar.ubnf:450:453:body/1/0/0/1/0/0/literal","occurrenceId":4,"parentOccurrenceId":1,"ruleId":"Fuzz::M1","span":[1,2]},{"completionOrder":2,"exprId":"expr:grammar.ubnf:459:463:body/1/0/0/2/literal","occurrenceId":5,"parentOccurrenceId":1,"ruleId":"Fuzz::M1","span":[2,4]},{"completionOrder":5,"exprId":"expr:grammar.ubnf:580:583:body/1/literal","occurrenceId":6,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[4,5]}],"mappingError":null,"matched":5,"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"M1:1","name":"v4","span":[0,1],"text":"7"},{"binding":"M1:2","name":"v5","span":[1,2],"text":"("},{"binding":"Root:0","name":"head","span":[0,4],"text":"7(ab"},{"binding":"Root:1","name":"head","span":[4,5],"text":"😀"}],"recoveredRegions":[],"references":[{"length":1,"name":"(","offset":1}],"scope":{"events":[{"action":"enter","length":0,"mode":"dynamic","name":null,"offset":0,"order":0},{"action":"leave","length":0,"mode":"dynamic","name":null,"offset":4,"order":1},{"action":"reference","length":1,"mode":"dynamic","name":"(","offset":1,"order":2}],"mode":["dynamic"]},"scopeDepth":0,"texts":[[4,5,"😀"]]}
```
