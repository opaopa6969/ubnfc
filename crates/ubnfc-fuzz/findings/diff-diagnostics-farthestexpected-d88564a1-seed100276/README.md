# diff-diagnostics-farthestexpected-d88564a1-seed100276

seed: 100276 / 入力 0 code point / 縮小 4 回

## signature

`diff:/diagnostics/*/farthestExpected/*`

## 入力（JSON escape）

```json
""
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

- `/diagnostics/0/farthestExpected/2`: java=`"','"` rust=`"'あ'"`
- `/diagnostics/0/farthestExpected/3`: java=`"';'"` rust=`<欠落>`
- `/diagnostics/0/farthestExpected/4`: java=`"'あ'"` rust=`<欠落>`

## 応答

```json
java = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"catalogs":[],"consumed":0,"declarations":[],"diagnostics":[{"expected":["'('","')'","','","';'","'あ'","RootParser","__CaptureSite"],"farthestExpected":["'('","')'","','","';'","'あ'","RootParser","__CaptureSite"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose"}],"fullConsumed":0,"fullMatched":0,"id":0,"lexical":[],"mappingError":null,"matched":0,"nativeFailure":{"consumed":0,"expected":["'('","')'","','","';'","'あ'","RootParser","__CaptureSite"],"matched":0,"offset":0},"parsedStatus":"failed","prefixCaptures":[],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"value":null}
rust = {"accepted":false,"allConsumed":false,"ast":null,"captures":[],"catalogs":[],"consumed":0,"declarations":[],"diagnostics":[{"deepestRule":"Fuzz::M0","expected":["(","あ"],"farthestExpected":["'('","')'","'あ'","RootParser","__CaptureSite"],"farthestOffset":0,"kind":"syntax","offset":0,"origin":"Mapper.diagnose","rulePath":["Fuzz::Root","Fuzz::TplWrap","Fuzz::M0"]}],"fullConsumed":0,"fullMatched":0,"hints":[{"deepestRule":"Fuzz::M0","expected":["(","あ"],"offset":0,"rulePath":["Fuzz::Root","Fuzz::TplWrap","Fuzz::M0"]}],"id":0,"lexical":[],"mappingError":null,"matched":0,"parsedStatus":"failed","prefixCaptures":[],"recoveries":[],"references":[],"scope":{"declarations":[],"depth":0,"diagnostics":[],"events":[],"mode":[],"references":[]},"scopeDepth":0,"value":null}
```
