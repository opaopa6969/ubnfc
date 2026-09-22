# diff-ast-fields-head-a25d01b1-seed9

seed: 9 / 入力 17 code point / 縮小 3 回

## signature

`diff:/ast/fields/head/*`

## 入力（JSON escape）

```json
"gammaSELECTSELECT"
```

## 文法

```ubnf
grammar Fuzz {
  @whitespace: javaStyle
  token T0 = LOOKAHEAD('a')
  token T1 = CI('select')
  token T2 = org.unlaxer.parser.clang.IdentifierParser
  L0 ::= T1 @v1 | T1 @v2 T2;
  @enum L1 ::= 'alpha' | 'beta' | 'gamma';
  @mapping(Node.A, params=[w]) VA ::= 'p' @w 'b';
  @mapping(Node.B, params=[w]) VB ::= 'q' @w L0{0,1};
  @mapping(Node) Node ::= VA | VB;
  @whitespace @recovery(SKIP) @scopeTree(mode=dynamic) @declares(symbol=v3) M0 ::= ( 'ab'* % '*' @v3 ){2,} 'x'? 'a' @v4 | T2* @v5 L1 @v6 [ [ ';'* ';' ]? % 'b' @v7 '*' @v8 [ T2{2,} @v9 ] ]+;
  @predictiveChoice @scopeTree(mode=lexical) @declares(symbol=v10) @backref(name=v12) M1 ::= [ '-' ] [ 'b'+ ')' @v10 ] T1* @v11 | '😀' @v12 { { 'ab'+ 'a'{2,} L1 } } '+';
  @root @mapping(Top, params=[head, tail]) Root ::= L1 @head M1 @head ( ',' @tail L1 )* @tail;
}
```

## 観測差（左 Java / 右 Rust）

- `/ast/fields/head/1`: java=`<欠落>` rust=`"SELECTSELECT"`
- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/diagnostics/0`: java=`{"expectedNonEmpty":true,"farthestExpected":["' '","'*/'","','","'/*'","'//'","'alpha'","'beta'","'gamma'","'select'","RootRepeat0Parser"],"farthestOffset":17,"kind":"syntax","length":null,"message":null,"severity":null}` rust=`<欠落>`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/events/0/name`: java=`"M1"` rust=`null`
- `/scope/events/1/name`: java=`"M1"` rust=`null`
- `/scope/mode/0`: java=`"lexical"` rust=`"dynamic"`
- `/scope/mode/1`: java=`<欠落>` rust=`"lexical"`
- `/scope/references`: java=`[]` rust=`<欠落>`

## 応答

```json
java = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":["GAMMA"],"tail":[]},"span":[0,17],"type":"Top"},"captures":[{"name":"head","span":[0,5],"text":"gamma"},{"name":"v11","span":[5,11],"text":"SELECT"},{"name":"v11","span":[11,17],"text":"SELECT"},{"name":"head","span":[5,17],"text":"SELECTSELECT"}],"consumed":17,"declarations":[],"diagnostics":[{"expected":["' '","'*/'","','","'/*'","'//'","'alpha'","'beta'","'gamma'","'select'","RootRepeat0Parser"],"farthestExpected":["' '","'*/'","','","'/*'","'//'","'alpha'","'beta'","'gamma'","'select'","RootRepeat0Parser"],"farthestOffset":17,"kind":"syntax","offset":17,"origin":"Mapper.diagnose"}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:212:219:body/2/literal","occurrenceId":3,"parentOccurrenceId":2,"ruleId":"Fuzz::L1","span":[0,5],"token":"gamma"},{"completionOrder":1,"exprId":"expr:grammar.ubnf:193:219:body/choice","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::L1","span":[0,5],"token":null},{"completionOrder":2,"exprId":"expr:grammar.ubnf:771:773:body/0/ruleRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,5],"token":null},{"completionOrder":3,"exprId":"expr:grammar.ubnf:643:650:body/0/0/optional","occurrenceId":7,"parentOccurrenceId":6,"ruleId":"Fuzz::M1","span":[5,5],"token":null},{"completionOrder":4,"exprId":"expr:grammar.ubnf:651:668:body/0/1/optional","occurrenceId":8,"parentOccurrenceId":6,"ruleId":"Fuzz::M1","span":[5,5],"token":null},{"completionOrder":5,"exprId":"expr:grammar.ubnf:669:671:body/0/2/0/tokenRef","occurrenceId":9,"parentOccurrenceId":6,"ruleId":"Fuzz::M1","span":[5,11],"token":"SELECT"},{"completionOrder":6,"exprId":"expr:grammar.ubnf:669:671:body/0/2/0/tokenRef","occurrenceId":10,"parentOccurrenceId":6,"ruleId":"Fuzz::M1","span":[11,17],"token":"SELECT"},{"completionOrder":7,"exprId":"expr:grammar.ubnf:643:672:body/0/seq","occurrenceId":6,"parentOccurrenceId":5,"ruleId":"Fuzz::M1","span":[5,17],"token":null},{"completionOrder":8,"exprId":"expr:grammar.ubnf:643:717:body/choice","occurrenceId":5,"parentOccurrenceId":4,"ruleId":"Fuzz::M1","span":[5,17],"token":null},{"completionOrder":9,"exprId":"expr:grammar.ubnf:780:782:body/1/ruleRef","occurrenceId":4,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[5,17],"token":null},{"completionOrder":10,"exprId":"expr:grammar.ubnf:771:806:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,17],"token":null}],"mappingError":null,"matched":17,"nativeFailure":{"consumed":17,"expected":["' '","'*/'","','","'/*'","'//'","'alpha'","'beta'","'gamma'","'select'","RootRepeat0Parser"],"matched":17,"offset":17},"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,5],"text":"gamma"},{"binding":"M1:1","name":"v11","span":[5,11],"text":"SELECT"},{"binding":"M1:1","name":"v11","span":[11,17],"text":"SELECT"},{"binding":"Root:1","name":"head","span":[5,17],"text":"SELECTSELECT"}],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[{"action":"enter","length":0,"mode":"lexical","name":"M1","offset":5,"order":0},{"action":"leave","length":0,"mode":"lexical","name":"M1","offset":17,"order":1}],"mode":["lexical"],"references":[]},"scopeDepth":0,"texts":[]}
rust = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":["GAMMA","SELECTSELECT"],"tail":[]},"span":[0,17],"type":"Top"},"captures":[{"name":"head","span":[0,5],"text":"gamma"},{"name":"v11","span":[5,11],"text":"SELECT"},{"name":"v11","span":[11,17],"text":"SELECT"},{"name":"head","span":[5,17],"text":"SELECTSELECT"}],"catalogs":[],"consumed":17,"declarations":[],"diagnostics":[],"hints":[{"deepestRule":"Fuzz::M1","expected":["T1",","],"offset":17,"rulePath":["Fuzz::Root","Fuzz::M1"]}],"id":0,"lexical":[{"completionOrder":5,"exprId":"expr:grammar.ubnf:771:806:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,17]},{"completionOrder":1,"exprId":"expr:grammar.ubnf:193:219:body/choice","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::L1","span":[0,5]},{"completionOrder":4,"exprId":"expr:grammar.ubnf:643:717:body/choice","occurrenceId":2,"parentOccurrenceId":0,"ruleId":"Fuzz::M1","span":[5,17]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:212:219:body/2/literal","occurrenceId":3,"parentOccurrenceId":1,"ruleId":"Fuzz::L1","span":[0,5]},{"completionOrder":2,"exprId":"expr:grammar.ubnf:669:671:body/0/2/0/tokenRef","occurrenceId":4,"parentOccurrenceId":2,"ruleId":"Fuzz::M1","span":[5,11]},{"completionOrder":3,"exprId":"expr:grammar.ubnf:669:671:body/0/2/0/tokenRef","occurrenceId":5,"parentOccurrenceId":2,"ruleId":"Fuzz::M1","span":[11,17]}],"mappingError":null,"matched":17,"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,5],"text":"gamma"},{"binding":"M1:1","name":"v11","span":[5,11],"text":"SELECT"},{"binding":"M1:1","name":"v11","span":[11,17],"text":"SELECT"},{"binding":"Root:1","name":"head","span":[5,17],"text":"SELECTSELECT"}],"recoveredRegions":[],"references":[],"scope":{"events":[{"action":"enter","length":0,"mode":"lexical","name":null,"offset":5,"order":0},{"action":"leave","length":0,"mode":"lexical","name":null,"offset":17,"order":1}],"mode":["dynamic","lexical"]},"scopeDepth":0,"texts":[[5,17,"SELECTSELECT"]]}
```
