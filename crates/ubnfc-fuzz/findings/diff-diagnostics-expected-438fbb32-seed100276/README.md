# diff-diagnostics-expected-438fbb32-seed100276

seed: 100276 / 入力 1 code point / 縮小 6 回

## signature

`diff:/diagnostics/*/expected/*`

## 入力（JSON escape）

```json
"+"
```

## 文法

```ubnf
grammar Fuzz {
  @ubnf: 2
  token T0 = LOOKAHEAD('a')
  token T1 = NEGATION('\u0061b\u{A}')
  token T2 = UNTIL('#')
  L0 ::= ERROR('fuzz error');
  @mapping(C0) @recovery(SKIP) M0 ::= 'あ';
  @whitespace(none) @longestChoice M1 ::= '='* @v5 'int'{2,3} ERROR('fuzz error') | T1 { ERROR('fuzz error')+ T2 }? ( [ 'ab'? ] T2? @v6 [ 'let' @v7 'b' T2 ] | 'ab' { 'x'{1,} @v8 }+ L0{2,3} ) | { { ','+ % '\n' } @v9 } 'if';
  @mapping(TplExpr, params=[head, tail]) TplWrap ::= '(' M0 @head { ',' M0 @tail } ')' | M0 @head { ';' M0 @tail };
  @root @mapping(Top, params=[head, tail]) Root ::= TplWrap @head ( ',' @tail '😀' )* @tail;
}
```

## 観測差（左 Java / 右 Rust）

- `/diagnostics/0/expected/0`: java=`"')'"` rust=`"あ"`
- `/diagnostics/0/expected/1`: java=`"','"` rust=`<欠落>`
- `/diagnostics/0/expected/2`: java=`"';'"` rust=`<欠落>`
- `/diagnostics/0/expected/3`: java=`"'あ'"` rust=`<欠落>`
- `/diagnostics/0/farthestExpected/0`: java=`"')'"` rust=`"あ"`
- `/diagnostics/0/farthestExpected/1`: java=`"','"` rust=`<欠落>`
- `/diagnostics/0/farthestExpected/2`: java=`"';'"` rust=`<欠落>`
- `/diagnostics/0/farthestExpected/3`: java=`"'あ'"` rust=`<欠落>`
- `/recoveries/0/captureOccurrences/1`: java=`<欠落>` rust=`1`

## 応答

```json
java = {"accepted":true,"allConsumed":true,"ast":null,"captures":[{"name":"head","span":[0,1],"text":"+"},{"name":"head","span":[0,1],"text":"+"}],"catalogs":[],"consumed":1,"declarations":[],"diagnostics":[{"expected":["')'","','","';'","'あ'"],"farthestExpected":["')'","','","';'","'あ'"],"farthestOffset":0,"kind":"recovery","length":1,"offset":0,"origin":"recovery","ruleId":"Fuzz::M0","severity":"ERROR"}],"fullConsumed":1,"fullMatched":1,"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:501:503:body/1/0/ruleRef","occurrenceId":4,"parentOccurrenceId":3,"ruleId":"Fuzz::TplWrap","span":[0,1],"token":null},{"completionOrder":1,"exprId":"expr:grammar.ubnf:510:526:body/1/1/repeat","occurrenceId":5,"parentOccurrenceId":3,"ruleId":"Fuzz::TplWrap","span":[1,1],"token":null},{"completionOrder":2,"exprId":"expr:grammar.ubnf:501:526:body/1/seq","occurrenceId":3,"parentOccurrenceId":2,"ruleId":"Fuzz::TplWrap","span":[0,1],"token":null},{"completionOrder":3,"exprId":"expr:grammar.ubnf:465:526:body/choice","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::TplWrap","span":[0,1],"token":null},{"completionOrder":4,"exprId":"expr:grammar.ubnf:580:587:body/0/ruleRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,1],"token":null},{"completionOrder":5,"exprId":"expr:grammar.ubnf:580:612:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,1],"token":null}],"mappingError":null,"matched":1,"nativeFailure":{"consumed":1,"expected":["','","';'","'😀'"],"matched":1,"offset":1},"parsedStatus":"succeeded","prefixCaptures":[{"binding":"TplWrap:2","name":"head","span":[0,1],"text":"+"},{"binding":"Root:0","name":"head","span":[0,1],"text":"+"}],"recoveries":[{"captureOccurrences":[0],"mode":"SKIP","ruleId":"Fuzz::M0","skippedSpan":[0,1],"span":[0,1],"syncPattern":null,"syncSpan":null}],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0}
rust = {"accepted":true,"allConsumed":true,"ast":null,"captures":[{"name":"head","span":[0,1],"text":"+"},{"name":"head","span":[0,1],"text":"+"}],"catalogs":[],"consumed":1,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::M0","expected":["あ"],"farthestExpected":["あ"],"farthestOffset":0,"kind":"recovery","length":1,"offset":0,"origin":"recovery","recoveryId":0,"ruleId":"Fuzz::M0","rulePath":["Fuzz::M0"],"severity":"ERROR"}],"fullConsumed":1,"fullMatched":1,"hints":[{"deepestRule":"Fuzz::TplWrap","expected":[";",","],"offset":1,"rulePath":["Fuzz::Root","Fuzz::TplWrap"]}],"id":0,"lexical":[{"completionOrder":1,"exprId":"expr:grammar.ubnf:580:612:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,1]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:465:526:body/choice","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::TplWrap","span":[0,1]}],"mappingError":null,"matched":1,"parsedStatus":"succeeded","prefixCaptures":[{"binding":"TplWrap:2","name":"head","span":[0,1],"text":"+"},{"binding":"Root:0","name":"head","span":[0,1],"text":"+"}],"recoveries":[{"captureOccurrences":[0,1],"mode":"SKIP","ruleId":"Fuzz::M0","skippedSpan":[0,1],"span":[0,1],"syncPattern":null,"syncSpan":null}],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0}
```
