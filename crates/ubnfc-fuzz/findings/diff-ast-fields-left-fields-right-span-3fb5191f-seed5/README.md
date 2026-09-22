# diff-ast-fields-left-fields-right-span-3fb5191f-seed5

seed: 5 / 入力 4 code point / 縮小 8 回

## signature

`diff:/ast/fields/left/fields/right/*/span/*`

## 入力（JSON escape）

```json
"😀+😀\t"
```

## 文法

```ubnf
grammar Fuzz {
  @whitespace: javaStyle
  token T0 = EMPTY
  token T1 = ANY
  token T2 = NEGATION('ab\n')
  L0 ::= '😀';
  @whitespace M0 ::= [ { 'x'* % ')' T2? % 'x' }? @v2 ] T2 @v3 [ ( '😀' | '*'{2,4} % ':' ';' 'c'{1,} @v4 ) T0{0,} ( T1 @v5 L0{0,} % ':' ')' ) ] | ( 'あ' @v6 '-' @v7 )? T2 { ( T1 @v8 L0? ';' ) ERROR('fuzz error')+ % 'ab' [ 'ab'+ '(' ',' ]? }{1,2};
  @mapping(Bin0, params=[left, op, right]) @rightAssoc @precedence(level=90) Op0 ::= T2 @left { '+' @op Op0 @right };
  @mapping(Bin1, params=[left, op, right]) @leftAssoc @precedence(level=80) Op1 ::= Op0 @left { '-' @op Op0 @right };
  @root Root ::= Op1 @head ( ',' @tail '\n' )* @tail;
}
```

## 観測差（左 Java / 右 Rust）

- `/ast/fields/left/fields/right/0/span/1`: java=`3` rust=`4`
- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/diagnostics/0`: java=`{"expectedNonEmpty":true,"farthestExpected":["'\n'","' '","'*/'","'+'","','","'-'","'/*'","'//'"],"farthestOffset":4,"kind":"syntax","length":null,"message":null,"severity":null}` rust=`<欠落>`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/references`: java=`[]` rust=`<欠落>`

## 応答

```json
java = {"accepted":true,"allConsumed":true,"ast":{"fields":{"left":{"fields":{"left":"😀","op":["+"],"right":[{"fields":{"left":"😀","op":[],"right":[]},"span":[2,3],"type":"Bin0"}]},"span":[0,4],"type":"Bin0"},"op":[],"right":[]},"span":[0,4],"type":"Bin1"},"captures":[{"name":"left","span":[0,1],"text":"😀"},{"name":"op","span":[1,2],"text":"+"},{"name":"left","span":[2,3],"text":"😀"},{"name":"right","span":[2,4],"text":"😀\t"},{"name":"left","span":[0,4],"text":"😀+😀\t"},{"name":"head","span":[0,4],"text":"😀+😀\t"}],"consumed":4,"declarations":[],"diagnostics":[{"expected":["'\n'","' '","'*/'","'+'","','","'-'","'/*'","'//'"],"farthestExpected":["'\n'","' '","'*/'","'+'","','","'-'","'/*'","'//'"],"farthestOffset":4,"kind":"syntax","offset":4,"origin":"Mapper.diagnose"}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:449:451:body/0/tokenRef","occurrenceId":5,"parentOccurrenceId":4,"ruleId":"Fuzz::Op0","span":[0,1],"token":"😀"},{"completionOrder":1,"exprId":"expr:grammar.ubnf:460:463:body/1/0/0/literal","occurrenceId":8,"parentOccurrenceId":7,"ruleId":"Fuzz::Op0","span":[1,2],"token":"+"},{"completionOrder":2,"exprId":"expr:grammar.ubnf:449:451:body/0/tokenRef","occurrenceId":11,"parentOccurrenceId":10,"ruleId":"Fuzz::Op0","span":[2,3],"token":"😀"},{"completionOrder":3,"exprId":"expr:grammar.ubnf:458:480:body/1/repeat","occurrenceId":12,"parentOccurrenceId":10,"ruleId":"Fuzz::Op0","span":[4,4],"token":null},{"completionOrder":4,"exprId":"expr:grammar.ubnf:449:480:body/seq","occurrenceId":10,"parentOccurrenceId":9,"ruleId":"Fuzz::Op0","span":[2,4],"token":null},{"completionOrder":5,"exprId":"expr:grammar.ubnf:468:471:body/1/0/1/ruleRef","occurrenceId":9,"parentOccurrenceId":7,"ruleId":"Fuzz::Op0","span":[2,4],"token":null},{"completionOrder":6,"exprId":"expr:grammar.ubnf:460:471:body/1/0/seq","occurrenceId":7,"parentOccurrenceId":6,"ruleId":"Fuzz::Op0","span":[1,4],"token":null},{"completionOrder":7,"exprId":"expr:grammar.ubnf:458:480:body/1/repeat","occurrenceId":6,"parentOccurrenceId":4,"ruleId":"Fuzz::Op0","span":[1,4],"token":null},{"completionOrder":8,"exprId":"expr:grammar.ubnf:449:480:body/seq","occurrenceId":4,"parentOccurrenceId":3,"ruleId":"Fuzz::Op0","span":[0,4],"token":null},{"completionOrder":9,"exprId":"expr:grammar.ubnf:566:569:body/0/ruleRef","occurrenceId":3,"parentOccurrenceId":2,"ruleId":"Fuzz::Op1","span":[0,4],"token":null},{"completionOrder":10,"exprId":"expr:grammar.ubnf:576:598:body/1/repeat","occurrenceId":13,"parentOccurrenceId":2,"ruleId":"Fuzz::Op1","span":[4,4],"token":null},{"completionOrder":11,"exprId":"expr:grammar.ubnf:566:598:body/seq","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::Op1","span":[0,4],"token":null},{"completionOrder":12,"exprId":"expr:grammar.ubnf:617:620:body/0/ruleRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,4],"token":null},{"completionOrder":13,"exprId":"expr:grammar.ubnf:617:646:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,4],"token":null}],"mappingError":null,"matched":4,"nativeFailure":{"consumed":4,"expected":["'\n'","' '","'*/'","'+'","','","'-'","'/*'","'//'"],"matched":4,"offset":4},"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Op0:0","name":"left","span":[0,1],"text":"😀"},{"binding":"Op0:1","name":"op","span":[1,2],"text":"+"},{"binding":"Op0:0","name":"left","span":[2,3],"text":"😀"},{"binding":"Op0:2","name":"right","span":[2,4],"text":"😀\t"},{"binding":"Op1:0","name":"left","span":[0,4],"text":"😀+😀\t"},{"binding":"Root:0","name":"head","span":[0,4],"text":"😀+😀\t"}],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"texts":[[0,1,"😀"],[1,2,"+"],[2,3,"😀"]]}
rust = {"accepted":true,"allConsumed":true,"ast":{"fields":{"left":{"fields":{"left":"😀","op":["+"],"right":[{"fields":{"left":"😀","op":[],"right":[]},"span":[2,4],"type":"Bin0"}]},"span":[0,4],"type":"Bin0"},"op":[],"right":[]},"span":[0,4],"type":"Bin1"},"captures":[{"name":"left","span":[0,1],"text":"😀"},{"name":"op","span":[1,2],"text":"+"},{"name":"left","span":[2,3],"text":"😀"},{"name":"right","span":[2,4],"text":"😀\t"},{"name":"left","span":[0,4],"text":"😀+😀\t"},{"name":"head","span":[0,4],"text":"😀+😀\t"}],"catalogs":[],"consumed":4,"declarations":[],"diagnostics":[],"hints":[{"deepestRule":"Fuzz::Op0","expected":["+","-",","],"offset":4,"rulePath":["Fuzz::Root","Fuzz::Op1","Fuzz::Op0","Fuzz::Op0"]}],"id":0,"lexical":[{"completionOrder":6,"exprId":"expr:grammar.ubnf:617:646:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,4]},{"completionOrder":5,"exprId":"expr:grammar.ubnf:566:598:body/seq","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Op1","span":[0,4]},{"completionOrder":4,"exprId":"expr:grammar.ubnf:449:480:body/seq","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::Op0","span":[0,4]},{"completionOrder":3,"exprId":"expr:grammar.ubnf:449:480:body/seq","occurrenceId":3,"parentOccurrenceId":2,"ruleId":"Fuzz::Op0","span":[2,4]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:449:451:body/0/tokenRef","occurrenceId":4,"parentOccurrenceId":2,"ruleId":"Fuzz::Op0","span":[0,1]},{"completionOrder":1,"exprId":"expr:grammar.ubnf:460:463:body/1/0/0/literal","occurrenceId":5,"parentOccurrenceId":2,"ruleId":"Fuzz::Op0","span":[1,2]},{"completionOrder":2,"exprId":"expr:grammar.ubnf:449:451:body/0/tokenRef","occurrenceId":6,"parentOccurrenceId":3,"ruleId":"Fuzz::Op0","span":[2,3]}],"mappingError":null,"matched":4,"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Op0:0","name":"left","span":[0,1],"text":"😀"},{"binding":"Op0:1","name":"op","span":[1,2],"text":"+"},{"binding":"Op0:0","name":"left","span":[2,3],"text":"😀"},{"binding":"Op0:2","name":"right","span":[2,4],"text":"😀\t"},{"binding":"Op1:0","name":"left","span":[0,4],"text":"😀+😀\t"},{"binding":"Root:0","name":"head","span":[0,4],"text":"😀+😀\t"}],"recoveredRegions":[],"references":[],"scope":{"events":[],"mode":[]},"scopeDepth":0,"texts":[]}
```
