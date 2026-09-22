# diff-scope-events-b247d3d4-seed92

seed: 92 / 入力 0 code point / 縮小 5 回

## signature

`diff:/scope/events/*`

## 入力（JSON escape）

```json
""
```

## 文法

```ubnf
grammar Fuzz {
  token T0 = NumberParser
  token T1 = EOF
  L0 ::= ;
  L1 ::= '\n' @v3 L0 [ 'あ'{2} % 'a' ] @v4 | [ '('{1} T1* % ':' @v5 'あ' @v6 ]? '-' L0;
  @mapping(Node.A, params=[w]) VA ::= 'p' @w T1;
  @mapping(Node.B, params=[w]) VB ::= 'q' @w 'a';
  @mapping(Node) Node ::= VA | VB;
  @mapping(C0, params=[v7, v8, v9, v11, v10]) @whitespace @longestChoice @scopeTree(mode=lexical) M0 ::= ( [ 'x' @v7 ] Node ERROR('fuzz error') | Node ':' ) | ERROR('fuzz error') @v8 | ')'* '+'+ @v9 ( L1? @v10 ) @v11;
  @mapping(C1, params=[v12]) @recovery(SKIP) @scopeTree(mode=dynamic) @backref(name=v12) M1 ::= ( ERROR('fuzz error')? )* @v12 | ','?;
  @mapping(Bin0, params=[left, op, right]) @leftAssoc @precedence(level=90) Op0 ::= M1 @left { '-' @op M1 @right };
  @root @mapping(Top, params=[head]) Root ::= Op0 @head '😀'* @head;
}
```

## 観測差（左 Java / 右 Rust）

- `/catalogs`: java=`{"<unobserved>":"catalogs"}` rust=`[]`
- `/diagnostics/0/expected`: java=`<欠落>` rust=`[]`
- `/diagnostics/0/expectedNonEmpty`: java=`true` rust=`<欠落>`
- `/diagnostics/0/farthestExpected`: java=`["'-'","'😀'","fuzz error"]` rust=`null`
- `/diagnostics/0/farthestOffset`: java=`0` rust=`null`
- `/diagnostics/0/kind`: java=`"syntax"` rust=`"WARNING"`
- `/diagnostics/0/length`: java=`null` rust=`0`
- `/diagnostics/0/message`: java=`null` rust=`"未定義のシンボル: ''"`
- `/diagnostics/0/offset`: java=`<欠落>` rust=`0`
- `/diagnostics/0/severity`: java=`null` rust=`"WARNING"`
- `/recoveries`: java=`[]` rust=`{"<unobserved>":"recoveries"}`
- `/scope/declarations`: java=`[]` rust=`<欠落>`
- `/scope/depth`: java=`0` rust=`<欠落>`
- `/scope/diagnostics`: java=`[]` rust=`<欠落>`
- `/scope/events/0/name`: java=`"M1"` rust=`null`
- `/scope/events/1/name`: java=`"M1"` rust=`null`
- `/scope/events/2`: java=`<欠落>` rust=`{"action":"reference","length":0,"mode":"dynamic","name":"","offset":0,"order":2}`
- `/scope/mode/0`: java=`"dynamic"` rust=`"lexical"`
- `/scope/mode/1`: java=`<欠落>` rust=`"dynamic"`
- `/scope/references`: java=`[]` rust=`<欠落>`

## 応答

```json
java = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":[{"fields":{"left":{"fields":{"v12":[""]},"span":[0,0],"type":"C1"},"op":[],"right":[]},"span":[0,0],"type":"Bin0"}]},"span":[0,0],"type":"Top"},"captures":[{"name":"v12","span":[0,0],"text":""},{"name":"left","span":[0,0],"text":""},{"name":"head","span":[0,0],"text":""}],"consumed":0,"declarations":[],"diagnostics":[{"expected":["'-'","'😀'","fuzz error"],"farthestExpected":["'-'","'😀'","fuzz error"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose"}],"id":0,"lexical":[{"completionOrder":0,"exprId":"expr:grammar.ubnf:605:625:body/0/0/0/0/0/repeat","occurrenceId":8,"parentOccurrenceId":7,"ruleId":"Fuzz::M1","span":[0,0],"token":null},{"completionOrder":1,"exprId":"expr:grammar.ubnf:605:625:body/0/0/0/0/seq","occurrenceId":7,"parentOccurrenceId":6,"ruleId":"Fuzz::M1","span":[0,0],"token":null},{"completionOrder":2,"exprId":"expr:grammar.ubnf:603:627:body/0/0/0/group","occurrenceId":6,"parentOccurrenceId":5,"ruleId":"Fuzz::M1","span":[0,0],"token":null},{"completionOrder":3,"exprId":"expr:grammar.ubnf:603:627:body/0/0/seq","occurrenceId":5,"parentOccurrenceId":4,"ruleId":"Fuzz::M1","span":[0,0],"token":null},{"completionOrder":4,"exprId":"expr:grammar.ubnf:603:640:body/choice","occurrenceId":4,"parentOccurrenceId":3,"ruleId":"Fuzz::M1","span":[0,0],"token":null},{"completionOrder":5,"exprId":"expr:grammar.ubnf:726:728:body/0/ruleRef","occurrenceId":3,"parentOccurrenceId":2,"ruleId":"Fuzz::Op0","span":[0,0],"token":null},{"completionOrder":6,"exprId":"expr:grammar.ubnf:735:756:body/1/repeat","occurrenceId":9,"parentOccurrenceId":2,"ruleId":"Fuzz::Op0","span":[0,0],"token":null},{"completionOrder":7,"exprId":"expr:grammar.ubnf:726:756:body/seq","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::Op0","span":[0,0],"token":null},{"completionOrder":8,"exprId":"expr:grammar.ubnf:804:807:body/0/ruleRef","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Root","span":[0,0],"token":null},{"completionOrder":9,"exprId":"expr:grammar.ubnf:804:818:body/seq","occurrenceId":0,"parentOccurrenceId":-1,"ruleId":"Fuzz::Root","span":[0,0],"token":null}],"mappingError":null,"matched":0,"nativeFailure":{"consumed":0,"expected":["'-'","'😀'","fuzz error"],"matched":0,"offset":0},"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"M1:0","name":"v12","span":[0,0],"text":""},{"binding":"Op0:0","name":"left","span":[0,0],"text":""},{"binding":"Root:0","name":"head","span":[0,0],"text":""}],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[{"action":"enter","length":0,"mode":"dynamic","name":"M1","offset":0,"order":0},{"action":"leave","length":0,"mode":"dynamic","name":"M1","offset":0,"order":1}],"mode":["dynamic"],"references":[]},"scopeDepth":0,"texts":[[0,0,""]]}
rust = {"accepted":true,"allConsumed":true,"ast":{"fields":{"head":[{"fields":{"left":{"fields":{"v12":[""]},"span":[0,0],"type":"C1"},"op":[],"right":[]},"span":[0,0],"type":"Bin0"}]},"span":[0,0],"type":"Top"},"captures":[{"name":"v12","span":[0,0],"text":""},{"name":"left","span":[0,0],"text":""},{"name":"head","span":[0,0],"text":""}],"catalogs":[],"consumed":0,"declarations":[],"diagnostics":[{"expected":[],"kind":"WARNING","length":0,"message":"未定義のシンボル: ''","offset":0,"origin":"ScopeStore","severity":"WARNING"}],"hints":[{"deepestRule":"Fuzz::M1","expected":["fuzz error","-","😀"],"offset":0,"rulePath":["Fuzz::Root","Fuzz::Op0","Fuzz::M1"]}],"id":0,"lexical":[{"completionOrder":2,"exprId":"expr:grammar.ubnf:804:818:body/seq","occurrenceId":0,"parentOccurrenceId":null,"ruleId":"Fuzz::Root","span":[0,0]},{"completionOrder":1,"exprId":"expr:grammar.ubnf:726:756:body/seq","occurrenceId":1,"parentOccurrenceId":0,"ruleId":"Fuzz::Op0","span":[0,0]},{"completionOrder":0,"exprId":"expr:grammar.ubnf:603:640:body/choice","occurrenceId":2,"parentOccurrenceId":1,"ruleId":"Fuzz::M1","span":[0,0]}],"mappingError":null,"matched":0,"nodes":[],"parsedStatus":"succeeded","prefixCaptures":[{"binding":"M1:0","name":"v12","span":[0,0],"text":""},{"binding":"Op0:0","name":"left","span":[0,0],"text":""},{"binding":"Root:0","name":"head","span":[0,0],"text":""}],"recoveredRegions":[],"references":[],"scope":{"events":[{"action":"enter","length":0,"mode":"dynamic","name":null,"offset":0,"order":0},{"action":"leave","length":0,"mode":"dynamic","name":null,"offset":0,"order":1},{"action":"reference","length":0,"mode":"dynamic","name":"","offset":0,"order":2}],"mode":["lexical","dynamic"]},"scopeDepth":0,"texts":[]}
```
