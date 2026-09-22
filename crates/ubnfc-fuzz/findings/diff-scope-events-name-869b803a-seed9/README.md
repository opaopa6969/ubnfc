# diff-scope-events-name-869b803a-seed9

seed: 9 / 入力 4 code point / 縮小 9 回

## signature

`diff:/scope/events/*/name`

## 入力（JSON escape）

```json
"beta"
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = LOOKAHEAD('a')
  token T1 = CI('select')
  token T2 = org.unlaxer.parser.clang.IdentifierParser
  L0 ::= ;
  L1 ::= 'alpha' | 'beta' | 'gamma';
  @mapping(Node.A, params=[w]) VA ::= 'p' @w 'b';
  @mapping(Node.B, params=[w]) VB ::= 'q' @w L0{0,1};
  @mapping(Node) Node ::= VA | VB;
  @whitespace @recovery(SKIP) @scopeTree(mode=dynamic) @declares(symbol=v3) M0 ::= ( 'ab'* % '*' @v3 ){2,} 'x'? 'a' @v4 | T2* @v5 L1 @v6 [ [ ';'* ';' ]? % 'b' @v7 '*' @v8 [ T2{2,} @v9 ] ]+;
  @predictiveChoice @scopeTree(mode=lexical) @declares(symbol=v10) @backref(name=v12) M1 ::= [ '-' ] [ 'b'+ ')' @v10 ] T1* @v11 | '😀' @v12 { { 'ab'+ 'a'{2,} L1 } } '+';
  @root @mapping(Top, params=[head, tail]) Root ::= L1 @head M1 @head ( ',' @tail L1 )* @tail;
}
```

## 観測差（左 Java / 右 Rust）

- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/diagnostics/0`: java=`{"expectedNonEmpty":true,"farthestExpected":["')'","','","'-'","'alpha'","'b'","'beta'","'gamma'","'select'","Repeat","RootRepeat0Parser"],"farthestOffset":4,"kind":"syntax","length":null,"message":null,"severity":null}` rust=`<欠落>`
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
java = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":["beta",""],"tail":[]},"span":[0,4],"type":"Top"},"captures":[{"name":"head","span":[0,4],"text":"beta"},{"name":"head","span":[4,4],"text":""}],"consumed":4,"declarations":[],"diagnostics":[{"expected":["')'","','","'-'","'alpha'","'b'","'beta'","'gamma'","'select'","Repeat","RootRepeat0Parser"],"farthestExpected":["')'","','","'-'","'alpha'","'b'","'beta'","'gamma'","'select'","Repeat","RootRepeat0Parser"],"farthestOffset":4,"kind":"syntax","offset":4,"origin":"Mapper.diagnose"}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:154:160:body/1/literal","occurrenceId":3,"parentOccurrenceId":2,"ruleId":"Fuzz::L1","span":[0,4],"token":"beta"},{"completionOrder":1,"exprId":"expr:grammar.ubnf:144:170:body/choice","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::L1","span":[0,4],"token":null},{"completionOrder":2,"exprId":"expr:grammar.ubnf:722:724:body/0/ruleRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,4],"token":null},{"completionOrder":3,"exprId":"expr:grammar.ubnf:594:601:body/0/0/optional","occurrenceId":7,"parentOccurrenceId":6,"ruleId":"Fuzz::M1","span":[4,4],"token":null},{"completionOrder":4,"exprId":"expr:grammar.ubnf:602:619:body/0/1/optional","occurrenceId":8,"parentOccurrenceId":6,"ruleId":"Fuzz::M1","span":[4,4],"token":null},{"completionOrder":5,"exprId":"expr:grammar.ubnf:594:623:body/0/seq","occurrenceId":6,"parentOccurrenceId":5,"ruleId":"Fuzz::M1","span":[4,4],"token":null},{"completionOrder":6,"exprId":"expr:grammar.ubnf:594:668:body/choice","occurrenceId":5,"parentOccurrenceId":4,"ruleId":"Fuzz::M1","span":[4,4],"token":null},{"completionOrder":7,"exprId":"expr:grammar.ubnf:731:733:body/1/ruleRef","occurrenceId":4,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[4,4],"token":null},{"completionOrder":8,"exprId":"expr:grammar.ubnf:722:757:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,4],"token":null}],"mappingError":null,"matched":4,"nativeFailure":{"consumed":4,"expected":["')'","','","'-'","'alpha'","'b'","'beta'","'gamma'","'select'","Repeat","RootRepeat0Parser"],"matched":4,"offset":4},"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,4],"text":"beta"},{"binding":"Root:1","name":"head","span":[4,4],"text":""}],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[{"action":"enter","length":0,"mode":"lexical","name":"M1","offset":4,"order":0},{"action":"leave","length":0,"mode":"lexical","name":"M1","offset":4,"order":1}],"mode":["lexical"],"references":[]},"scopeDepth":0,"texts":[[0,4,"beta"],[4,4,""]]}
rust = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":["beta",""],"tail":[]},"span":[0,4],"type":"Top"},"captures":[{"name":"head","span":[0,4],"text":"beta"},{"name":"head","span":[4,4],"text":""}],"catalogs":[],"consumed":4,"declarations":[],"diagnostics":[],"hints":[{"deepestRule":"Fuzz::M1","expected":["-","b","expression","T1",","],"offset":4,"rulePath":["Fuzz::Root","Fuzz::M1"]}],"id":0,"lexical":[{"completionOrder":3,"exprId":"expr:grammar.ubnf:722:757:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,4]},{"completionOrder":1,"exprId":"expr:grammar.ubnf:144:170:body/choice","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::L1","span":[0,4]},{"completionOrder":2,"exprId":"expr:grammar.ubnf:594:668:body/choice","occurrenceId":2,"parentOccurrenceId":0,"ruleId":"Fuzz::M1","span":[4,4]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:154:160:body/1/literal","occurrenceId":3,"parentOccurrenceId":1,"ruleId":"Fuzz::L1","span":[0,4]}],"mappingError":null,"matched":4,"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"Root:0","name":"head","span":[0,4],"text":"beta"},{"binding":"Root:1","name":"head","span":[4,4],"text":""}],"recoveredRegions":[],"references":[],"scope":{"events":[{"action":"enter","length":0,"mode":"lexical","name":null,"offset":4,"order":0},{"action":"leave","length":0,"mode":"lexical","name":null,"offset":4,"order":1}],"mode":["dynamic","lexical"]},"scopeDepth":0,"texts":[]}
```
