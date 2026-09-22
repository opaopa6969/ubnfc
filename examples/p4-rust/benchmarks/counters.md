
### counters: complex (332 bytes, 認識), 計測込み 27.760 ms

| runtime 呼出し | 回数/parse |
|---|---:|
| limit | 9660 |
| enter_rule | 2455 |
| mark | 10190 |
| restore | 10364 |
| save_effects | 908 |
| replay_effects | 816 |
| effect_at | 155 |
| lookup | 7108 |
| memo_hit | 816 |
| store | 6292 |
| memo_insert | 6292 |
| event | 1462 |
| join | 2895 |
| combine | 4532 |
| fail | 1482 |
| primitive | 1616 |
| literal | 1570 |
| any | 0 |
| eof | 1 |
| number | 24 |
| identifier | 22 |
| quoted | 0 |
| until | 0 |
| scan | 43 |
| mapped_text | 0 |
| node | 0 |
| display_enter | 11175 |
| display_failures | 7535 |
| display_reach | 11175 |
| display_leave | 11175 |
| diag_push | 1081968 |
| diag_join | 1499934 |
| diag_rule | 2455 |
| display_frame_iterations | 4327682 |
| display_max_frames | 133 |

rule 関数呼出し合計 2455、expression 関数呼出し合計 7205、rule 自己時間合計 24.584 ms

| rule | calls (consumed) | calls (matchOnly) | 自己時間 % | 包含時間 ms | 自己 ns/call |
|---|---:|---:|---:|---:|---:|
| r86 SliceBaseExpression | 189 | 0 | 10.3 | 7.295 | 13415 |
| r82 SliceBaseReceiver | 168 | 0 | 6.1 | 4.760 | 8947 |
| r87 SliceNestedExpression | 21 | 0 | 4.5 | 8.332 | 52641 |
| r121 VariableRef | 166 | 0 | 4.1 | 1.438 | 6047 |
| r106 IfExpression | 68 | 0 | 2.9 | 21.427 | 10471 |
| r98 BooleanComparable | 31 | 0 | 2.6 | 14.948 | 20663 |
| r41 NumberTerm | 30 | 0 | 2.3 | 52.128 | 18613 |
| r40 NumberExpression | 38 | 0 | 2.0 | 52.766 | 12944 |
| r44 MathFunction | 29 | 0 | 1.9 | 21.010 | 16443 |
| r122 TypeKeyword | 7 | 0 | 1.8 | 0.434 | 61971 |
| r45 SinFunction | 29 | 0 | 1.4 | 1.818 | 11905 |
| r63 TrimFunction | 58 | 0 | 1.4 | 0.343 | 5908 |
| r109 NumberMatchExpression | 31 | 0 | 1.4 | 6.280 | 10891 |
| r89 StringExpression | 37 | 0 | 1.4 | 11.052 | 9122 |
| r104 CompareOp | 15 | 0 | 1.4 | 0.333 | 22174 |
| r33 ExternalStringInvocation | 42 | 0 | 1.3 | 0.329 | 7841 |
| r61 ToUpperCaseFunction | 58 | 0 | 1.3 | 0.323 | 5569 |
| r46 CosFunction | 28 | 0 | 1.3 | 0.323 | 11532 |
| r108 TernaryExpression | 31 | 0 | 1.3 | 0.322 | 10372 |
| r66 ToUpperCaseDotMethod | 42 | 0 | 1.3 | 1.276 | 7472 |
| r62 ToLowerCaseFunction | 58 | 0 | 1.3 | 0.314 | 5408 |
| r60 NumberFactor | 31 | 0 | 1.3 | 51.403 | 10000 |
| r68 TrimDotMethod | 42 | 0 | 1.3 | 0.358 | 7365 |
| r75 EndsWithDotMethod | 16 | 0 | 1.2 | 0.386 | 18827 |
| r67 ToLowerCaseDotMethod | 42 | 0 | 1.2 | 0.431 | 7044 |

| expression | 所属 rule / 種別 | calls (consumed) | calls (matchOnly) |
|---|---|---:|---:|
| e681 | SliceBaseExpression Choice | 189 | 0 |
| e659 | SliceBaseReceiver Choice | 168 | 0 |
| e1014 | VariableRef Seq | 166 | 0 |
| e922 | IfExpression Seq | 68 | 0 |
| e505 | ToUpperCaseFunction Seq | 58 | 0 |
| e510 | ToLowerCaseFunction Seq | 58 | 0 |
| e515 | TrimFunction Seq | 58 | 0 |
| e623 | StringPredicateReceiver Choice | 48 | 0 |
| e297 | ExternalStringInvocation Seq | 42 | 0 |
| e530 | ToUpperCaseDotMethod Seq | 42 | 0 |
| e535 | ToLowerCaseDotMethod Seq | 42 | 0 |
| e540 | TrimDotMethod Seq | 42 | 0 |
| e804 | ParenthesizedStringExpression Seq | 42 | 0 |
| e369 | NumberExpression Seq | 38 | 0 |
| e798 | StringExpression Seq | 37 | 0 |
| e345 | MethodInvocation Seq | 36 | 0 |
| e923 | IfExpression Literal | 34 | 0 |
| e488 | NumberFactor Choice | 31 | 0 |
| e489 | NumberFactor RuleRef | 31 | 0 |
| e490 | NumberFactor RuleRef | 31 | 0 |
| e865 | BooleanComparable Choice | 31 | 0 |
| e943 | TernaryExpression Seq | 31 | 0 |
| e944 | TernaryExpression Literal | 31 | 0 |
| e951 | NumberMatchExpression Seq | 31 | 0 |
| e952 | NumberMatchExpression Literal | 31 | 0 |

### counters: complex (332 bytes, AST), 計測込み 28.017 ms

| runtime 呼出し | 回数/parse |
|---|---:|
| limit | 9660 |
| enter_rule | 2455 |
| mark | 10190 |
| restore | 10364 |
| save_effects | 908 |
| replay_effects | 816 |
| effect_at | 155 |
| lookup | 7108 |
| memo_hit | 816 |
| store | 6292 |
| memo_insert | 6292 |
| event | 1462 |
| join | 2895 |
| combine | 4532 |
| fail | 1482 |
| primitive | 1616 |
| literal | 1570 |
| any | 0 |
| eof | 1 |
| number | 24 |
| identifier | 22 |
| quoted | 0 |
| until | 0 |
| scan | 43 |
| mapped_text | 0 |
| node | 131 |
| display_enter | 11175 |
| display_failures | 7535 |
| display_reach | 11175 |
| display_leave | 11175 |
| diag_push | 1081968 |
| diag_join | 1499934 |
| diag_rule | 2455 |
| display_frame_iterations | 4327682 |
| display_max_frames | 133 |

rule 関数呼出し合計 2455、expression 関数呼出し合計 7205、rule 自己時間合計 24.456 ms

| rule | calls (consumed) | calls (matchOnly) | 自己時間 % | 包含時間 ms | 自己 ns/call |
|---|---:|---:|---:|---:|---:|
| r86 SliceBaseExpression | 189 | 0 | 10.1 | 7.349 | 13108 |
| r82 SliceBaseReceiver | 168 | 0 | 6.3 | 4.872 | 9208 |
| r87 SliceNestedExpression | 21 | 0 | 4.3 | 8.340 | 50397 |
| r121 VariableRef | 166 | 0 | 4.1 | 1.423 | 5974 |
| r106 IfExpression | 68 | 0 | 2.8 | 21.284 | 10228 |
| r98 BooleanComparable | 31 | 0 | 2.4 | 14.738 | 18622 |
| r41 NumberTerm | 30 | 0 | 2.3 | 51.935 | 18767 |
| r40 NumberExpression | 38 | 0 | 2.0 | 52.579 | 13133 |
| r44 MathFunction | 29 | 0 | 2.0 | 21.083 | 17062 |
| r122 TypeKeyword | 7 | 0 | 1.8 | 0.431 | 61563 |
| r45 SinFunction | 29 | 0 | 1.6 | 1.915 | 13655 |
| r89 StringExpression | 37 | 0 | 1.5 | 11.069 | 9807 |
| r63 TrimFunction | 58 | 0 | 1.5 | 0.357 | 6154 |
| r33 ExternalStringInvocation | 42 | 0 | 1.4 | 0.342 | 8132 |
| r62 ToLowerCaseFunction | 58 | 0 | 1.4 | 0.332 | 5724 |
| r66 ToUpperCaseDotMethod | 42 | 0 | 1.3 | 1.297 | 7855 |
| r109 NumberMatchExpression | 31 | 0 | 1.3 | 6.215 | 10573 |
| r60 NumberFactor | 31 | 0 | 1.3 | 51.195 | 10368 |
| r104 CompareOp | 15 | 0 | 1.3 | 0.310 | 20685 |
| r68 TrimDotMethod | 42 | 0 | 1.3 | 0.354 | 7372 |
| r67 ToLowerCaseDotMethod | 42 | 0 | 1.2 | 0.438 | 7257 |
| r61 ToUpperCaseFunction | 58 | 0 | 1.2 | 0.301 | 5187 |
| r75 EndsWithDotMethod | 16 | 0 | 1.2 | 0.389 | 18754 |
| r113 StringMatchExpression | 21 | 0 | 1.2 | 0.692 | 14270 |
| r50 MaxFunction | 27 | 0 | 1.2 | 8.937 | 10967 |

| expression | 所属 rule / 種別 | calls (consumed) | calls (matchOnly) |
|---|---|---:|---:|
| e681 | SliceBaseExpression Choice | 189 | 0 |
| e659 | SliceBaseReceiver Choice | 168 | 0 |
| e1014 | VariableRef Seq | 166 | 0 |
| e922 | IfExpression Seq | 68 | 0 |
| e505 | ToUpperCaseFunction Seq | 58 | 0 |
| e510 | ToLowerCaseFunction Seq | 58 | 0 |
| e515 | TrimFunction Seq | 58 | 0 |
| e623 | StringPredicateReceiver Choice | 48 | 0 |
| e297 | ExternalStringInvocation Seq | 42 | 0 |
| e530 | ToUpperCaseDotMethod Seq | 42 | 0 |
| e535 | ToLowerCaseDotMethod Seq | 42 | 0 |
| e540 | TrimDotMethod Seq | 42 | 0 |
| e804 | ParenthesizedStringExpression Seq | 42 | 0 |
| e369 | NumberExpression Seq | 38 | 0 |
| e798 | StringExpression Seq | 37 | 0 |
| e345 | MethodInvocation Seq | 36 | 0 |
| e923 | IfExpression Literal | 34 | 0 |
| e488 | NumberFactor Choice | 31 | 0 |
| e489 | NumberFactor RuleRef | 31 | 0 |
| e490 | NumberFactor RuleRef | 31 | 0 |
| e865 | BooleanComparable Choice | 31 | 0 |
| e943 | TernaryExpression Seq | 31 | 0 |
| e944 | TernaryExpression Literal | 31 | 0 |
| e951 | NumberMatchExpression Seq | 31 | 0 |
| e952 | NumberMatchExpression Literal | 31 | 0 |

### counters: complex-x4 (1293 bytes, 認識), 計測込み 116.447 ms

| runtime 呼出し | 回数/parse |
|---|---:|
| limit | 37062 |
| enter_rule | 9418 |
| mark | 39083 |
| restore | 39725 |
| save_effects | 3503 |
| replay_effects | 3141 |
| effect_at | 602 |
| lookup | 27259 |
| memo_hit | 3141 |
| store | 24118 |
| memo_insert | 24118 |
| event | 5640 |
| join | 11109 |
| combine | 17381 |
| fail | 5697 |
| primitive | 6218 |
| literal | 6043 |
| any | 0 |
| eof | 1 |
| number | 93 |
| identifier | 82 |
| quoted | 0 |
| until | 0 |
| scan | 163 |
| mapped_text | 0 |
| node | 0 |
| display_enter | 42864 |
| display_failures | 28865 |
| display_reach | 42864 |
| display_leave | 42864 |
| diag_push | 4313145 |
| diag_join | 5981307 |
| diag_rule | 9418 |
| display_frame_iterations | 17272034 |
| display_max_frames | 135 |

rule 関数呼出し合計 9418、expression 関数呼出し合計 27644、rule 自己時間合計 101.558 ms

| rule | calls (consumed) | calls (matchOnly) | 自己時間 % | 包含時間 ms | 自己 ns/call |
|---|---:|---:|---:|---:|---:|
| r86 SliceBaseExpression | 729 | 0 | 9.9 | 29.600 | 13826 |
| r82 SliceBaseReceiver | 648 | 0 | 6.2 | 19.521 | 9749 |
| r87 SliceNestedExpression | 81 | 0 | 4.4 | 33.788 | 55330 |
| r121 VariableRef | 643 | 0 | 3.9 | 5.621 | 6082 |
| r106 IfExpression | 263 | 0 | 2.9 | 90.334 | 11381 |
| r40 NumberExpression | 143 | 0 | 2.5 | 222.164 | 17722 |
| r41 NumberTerm | 117 | 0 | 2.3 | 219.009 | 20202 |
| r44 MathFunction | 113 | 0 | 2.2 | 89.068 | 20217 |
| r98 BooleanComparable | 118 | 0 | 2.1 | 60.980 | 18461 |
| r122 TypeKeyword | 28 | 0 | 1.7 | 1.710 | 61074 |
| r61 ToUpperCaseFunction | 223 | 0 | 1.6 | 1.607 | 7204 |
| r113 StringMatchExpression | 81 | 0 | 1.5 | 3.171 | 19253 |
| r109 NumberMatchExpression | 121 | 0 | 1.4 | 25.623 | 12088 |
| r93 StringTypedVariable | 81 | 0 | 1.4 | 1.425 | 17596 |
| r89 StringExpression | 139 | 0 | 1.4 | 46.170 | 10111 |
| r60 NumberFactor | 121 | 0 | 1.4 | 215.896 | 11457 |
| r33 ExternalStringInvocation | 162 | 0 | 1.4 | 1.377 | 8501 |
| r62 ToLowerCaseFunction | 223 | 0 | 1.3 | 1.358 | 6089 |
| r66 ToUpperCaseDotMethod | 162 | 0 | 1.3 | 5.335 | 8226 |
| r104 CompareOp | 60 | 0 | 1.3 | 1.318 | 21964 |
| r91 StringTerm | 81 | 0 | 1.3 | 44.764 | 16240 |
| r63 TrimFunction | 223 | 0 | 1.3 | 1.309 | 5870 |
| r68 TrimDotMethod | 162 | 0 | 1.3 | 1.486 | 7923 |
| r67 ToLowerCaseDotMethod | 162 | 0 | 1.2 | 1.447 | 7641 |
| r56 PowFunction | 97 | 0 | 1.2 | 16.442 | 12740 |

| expression | 所属 rule / 種別 | calls (consumed) | calls (matchOnly) |
|---|---|---:|---:|
| e681 | SliceBaseExpression Choice | 729 | 0 |
| e659 | SliceBaseReceiver Choice | 648 | 0 |
| e1014 | VariableRef Seq | 643 | 0 |
| e922 | IfExpression Seq | 263 | 0 |
| e505 | ToUpperCaseFunction Seq | 223 | 0 |
| e510 | ToLowerCaseFunction Seq | 223 | 0 |
| e515 | TrimFunction Seq | 223 | 0 |
| e623 | StringPredicateReceiver Choice | 183 | 0 |
| e297 | ExternalStringInvocation Seq | 162 | 0 |
| e530 | ToUpperCaseDotMethod Seq | 162 | 0 |
| e535 | ToLowerCaseDotMethod Seq | 162 | 0 |
| e540 | TrimDotMethod Seq | 162 | 0 |
| e804 | ParenthesizedStringExpression Seq | 162 | 0 |
| e369 | NumberExpression Seq | 143 | 0 |
| e798 | StringExpression Seq | 139 | 0 |
| e345 | MethodInvocation Seq | 138 | 0 |
| e923 | IfExpression Literal | 133 | 0 |
| e488 | NumberFactor Choice | 121 | 0 |
| e489 | NumberFactor RuleRef | 121 | 0 |
| e490 | NumberFactor RuleRef | 121 | 0 |
| e943 | TernaryExpression Seq | 121 | 0 |
| e944 | TernaryExpression Literal | 121 | 0 |
| e951 | NumberMatchExpression Seq | 121 | 0 |
| e952 | NumberMatchExpression Literal | 121 | 0 |
| e1015 | VariableRef Literal | 121 | 0 |

### counters: complex-x4 (1293 bytes, AST), 計測込み 148.422 ms

| runtime 呼出し | 回数/parse |
|---|---:|
| limit | 37062 |
| enter_rule | 9418 |
| mark | 39083 |
| restore | 39725 |
| save_effects | 3503 |
| replay_effects | 3141 |
| effect_at | 602 |
| lookup | 27259 |
| memo_hit | 3141 |
| store | 24118 |
| memo_insert | 24118 |
| event | 5640 |
| join | 11109 |
| combine | 17381 |
| fail | 5697 |
| primitive | 6218 |
| literal | 6043 |
| any | 0 |
| eof | 1 |
| number | 93 |
| identifier | 82 |
| quoted | 0 |
| until | 0 |
| scan | 163 |
| mapped_text | 0 |
| node | 515 |
| display_enter | 42864 |
| display_failures | 28865 |
| display_reach | 42864 |
| display_leave | 42864 |
| diag_push | 4313145 |
| diag_join | 5981307 |
| diag_rule | 9418 |
| display_frame_iterations | 17272034 |
| display_max_frames | 135 |

rule 関数呼出し合計 9418、expression 関数呼出し合計 27644、rule 自己時間合計 130.340 ms

| rule | calls (consumed) | calls (matchOnly) | 自己時間 % | 包含時間 ms | 自己 ns/call |
|---|---:|---:|---:|---:|---:|
| r86 SliceBaseExpression | 729 | 0 | 9.5 | 36.614 | 16940 |
| r82 SliceBaseReceiver | 648 | 0 | 6.0 | 24.264 | 11974 |
| r87 SliceNestedExpression | 81 | 0 | 4.6 | 42.277 | 74283 |
| r121 VariableRef | 643 | 0 | 3.9 | 7.118 | 7876 |
| r106 IfExpression | 263 | 0 | 2.7 | 116.484 | 13450 |
| r44 MathFunction | 113 | 0 | 2.6 | 110.328 | 30108 |
| r40 NumberExpression | 143 | 0 | 2.5 | 283.931 | 23047 |
| r41 NumberTerm | 117 | 0 | 2.5 | 279.833 | 27502 |
| r98 BooleanComparable | 118 | 0 | 2.2 | 76.716 | 24263 |
| r60 NumberFactor | 121 | 0 | 1.6 | 275.604 | 17205 |
| r122 TypeKeyword | 28 | 0 | 1.6 | 2.053 | 73324 |
| r61 ToUpperCaseFunction | 223 | 0 | 1.5 | 2.018 | 9048 |
| r93 StringTypedVariable | 81 | 0 | 1.5 | 2.000 | 24696 |
| r33 ExternalStringInvocation | 162 | 0 | 1.4 | 1.833 | 11313 |
| r91 StringTerm | 81 | 0 | 1.4 | 56.331 | 22465 |
| r89 StringExpression | 139 | 0 | 1.4 | 58.150 | 13083 |
| r113 StringMatchExpression | 81 | 0 | 1.4 | 3.857 | 22037 |
| r109 NumberMatchExpression | 121 | 0 | 1.4 | 33.828 | 14715 |
| r66 ToUpperCaseDotMethod | 162 | 0 | 1.3 | 6.623 | 10449 |
| r56 PowFunction | 97 | 0 | 1.3 | 20.200 | 17390 |
| r62 ToLowerCaseFunction | 223 | 0 | 1.3 | 1.635 | 7332 |
| r104 CompareOp | 60 | 0 | 1.2 | 1.624 | 27073 |
| r108 TernaryExpression | 121 | 0 | 1.2 | 1.596 | 13189 |
| r68 TrimDotMethod | 162 | 0 | 1.2 | 1.865 | 9837 |
| r63 TrimFunction | 223 | 0 | 1.2 | 1.589 | 7125 |

| expression | 所属 rule / 種別 | calls (consumed) | calls (matchOnly) |
|---|---|---:|---:|
| e681 | SliceBaseExpression Choice | 729 | 0 |
| e659 | SliceBaseReceiver Choice | 648 | 0 |
| e1014 | VariableRef Seq | 643 | 0 |
| e922 | IfExpression Seq | 263 | 0 |
| e505 | ToUpperCaseFunction Seq | 223 | 0 |
| e510 | ToLowerCaseFunction Seq | 223 | 0 |
| e515 | TrimFunction Seq | 223 | 0 |
| e623 | StringPredicateReceiver Choice | 183 | 0 |
| e297 | ExternalStringInvocation Seq | 162 | 0 |
| e530 | ToUpperCaseDotMethod Seq | 162 | 0 |
| e535 | ToLowerCaseDotMethod Seq | 162 | 0 |
| e540 | TrimDotMethod Seq | 162 | 0 |
| e804 | ParenthesizedStringExpression Seq | 162 | 0 |
| e369 | NumberExpression Seq | 143 | 0 |
| e798 | StringExpression Seq | 139 | 0 |
| e345 | MethodInvocation Seq | 138 | 0 |
| e923 | IfExpression Literal | 133 | 0 |
| e488 | NumberFactor Choice | 121 | 0 |
| e489 | NumberFactor RuleRef | 121 | 0 |
| e490 | NumberFactor RuleRef | 121 | 0 |
| e943 | TernaryExpression Seq | 121 | 0 |
| e944 | TernaryExpression Literal | 121 | 0 |
| e951 | NumberMatchExpression Seq | 121 | 0 |
| e952 | NumberMatchExpression Literal | 121 | 0 |
| e1015 | VariableRef Literal | 121 | 0 |

### counters: comparison-heavy (179 bytes, 認識), 計測込み 26.493 ms

| runtime 呼出し | 回数/parse |
|---|---:|
| limit | 11715 |
| enter_rule | 3015 |
| mark | 12287 |
| restore | 13473 |
| save_effects | 717 |
| replay_effects | 624 |
| effect_at | 0 |
| lookup | 8511 |
| memo_hit | 624 |
| store | 7887 |
| memo_insert | 7887 |
| event | 1190 |
| join | 3161 |
| combine | 5398 |
| fail | 1968 |
| primitive | 2093 |
| literal | 2030 |
| any | 0 |
| eof | 0 |
| number | 63 |
| identifier | 0 |
| quoted | 0 |
| until | 0 |
| scan | 32 |
| mapped_text | 0 |
| node | 0 |
| display_enter | 13870 |
| display_failures | 10103 |
| display_reach | 13870 |
| display_leave | 13870 |
| diag_push | 948651 |
| diag_join | 938349 |
| diag_rule | 3015 |
| display_frame_iterations | 2652068 |
| display_max_frames | 52 |

rule 関数呼出し合計 3015、expression 関数呼出し合計 8700、rule 自己時間合計 23.617 ms

| rule | calls (consumed) | calls (matchOnly) | 自己時間 % | 包含時間 ms | 自己 ns/call |
|---|---:|---:|---:|---:|---:|
| r82 SliceBaseReceiver | 128 | 0 | 6.9 | 3.334 | 12640 |
| r86 SliceBaseExpression | 144 | 0 | 6.1 | 4.767 | 9951 |
| r51 RandomFunction | 63 | 0 | 4.6 | 1.087 | 17257 |
| r87 SliceNestedExpression | 16 | 0 | 4.2 | 5.703 | 62721 |
| r44 MathFunction | 63 | 0 | 4.0 | 8.091 | 14826 |
| r41 NumberTerm | 47 | 0 | 3.1 | 13.202 | 15423 |
| r106 IfExpression | 95 | 0 | 2.8 | 0.650 | 6842 |
| r121 VariableRef | 175 | 0 | 2.5 | 0.596 | 3406 |
| r55 FloorFunction | 63 | 0 | 2.2 | 0.527 | 8362 |
| r50 MaxFunction | 63 | 0 | 2.1 | 0.485 | 7700 |
| r49 MinFunction | 63 | 0 | 2.0 | 0.483 | 7664 |
| r56 PowFunction | 63 | 0 | 2.0 | 0.482 | 7649 |
| r40 NumberExpression | 32 | 0 | 2.0 | 13.807 | 14836 |
| r109 NumberMatchExpression | 63 | 0 | 2.0 | 0.471 | 7473 |
| r54 CeilFunction | 63 | 0 | 2.0 | 0.467 | 7412 |
| r52 AbsFunction | 63 | 0 | 2.0 | 0.465 | 7380 |
| r53 RoundFunction | 63 | 0 | 1.9 | 0.459 | 7283 |
| r58 ExpFunction | 63 | 0 | 1.9 | 0.457 | 7256 |
| r45 SinFunction | 63 | 0 | 1.9 | 0.454 | 7212 |
| r48 SqrtFunction | 63 | 0 | 1.9 | 0.452 | 7179 |
| r57 LogFunction | 63 | 0 | 1.9 | 0.452 | 7169 |
| r108 TernaryExpression | 63 | 0 | 1.9 | 0.448 | 7111 |
| r46 CosFunction | 63 | 0 | 1.9 | 0.448 | 7104 |
| r47 TanFunction | 63 | 0 | 1.9 | 0.439 | 6974 |
| r59 ToNumFunction | 63 | 0 | 1.8 | 0.433 | 6871 |

| expression | 所属 rule / 種別 | calls (consumed) | calls (matchOnly) |
|---|---|---:|---:|
| e1014 | VariableRef Seq | 175 | 0 |
| e681 | SliceBaseExpression Choice | 144 | 0 |
| e659 | SliceBaseReceiver Choice | 128 | 0 |
| e922 | IfExpression Seq | 95 | 0 |
| e271 | ExternalNumberInvocation Seq | 63 | 0 |
| e272 | ExternalNumberInvocation Literal | 63 | 0 |
| e378 | NumberTerm Seq | 63 | 0 |
| e379 | NumberTerm RuleRef | 63 | 0 |
| e384 | MulOp Choice | 63 | 0 |
| e387 | MathFunction Choice | 63 | 0 |
| e388 | MathFunction RuleRef | 63 | 0 |
| e389 | MathFunction RuleRef | 63 | 0 |
| e390 | MathFunction RuleRef | 63 | 0 |
| e391 | MathFunction RuleRef | 63 | 0 |
| e392 | MathFunction RuleRef | 63 | 0 |
| e393 | MathFunction RuleRef | 63 | 0 |
| e394 | MathFunction RuleRef | 63 | 0 |
| e395 | MathFunction RuleRef | 63 | 0 |
| e396 | MathFunction RuleRef | 63 | 0 |
| e397 | MathFunction RuleRef | 63 | 0 |
| e398 | MathFunction RuleRef | 63 | 0 |
| e399 | MathFunction RuleRef | 63 | 0 |
| e400 | MathFunction RuleRef | 63 | 0 |
| e401 | MathFunction RuleRef | 63 | 0 |
| e402 | SinFunction Seq | 63 | 0 |

### counters: comparison-heavy (179 bytes, AST), 計測込み 31.014 ms

| runtime 呼出し | 回数/parse |
|---|---:|
| limit | 11715 |
| enter_rule | 3015 |
| mark | 12287 |
| restore | 13473 |
| save_effects | 717 |
| replay_effects | 624 |
| effect_at | 0 |
| lookup | 8511 |
| memo_hit | 624 |
| store | 7887 |
| memo_insert | 7887 |
| event | 1190 |
| join | 3161 |
| combine | 5398 |
| fail | 1968 |
| primitive | 2093 |
| literal | 2030 |
| any | 0 |
| eof | 0 |
| number | 63 |
| identifier | 0 |
| quoted | 0 |
| until | 0 |
| scan | 32 |
| mapped_text | 0 |
| node | 193 |
| display_enter | 13870 |
| display_failures | 10103 |
| display_reach | 13870 |
| display_leave | 13870 |
| diag_push | 948651 |
| diag_join | 938349 |
| diag_rule | 3015 |
| display_frame_iterations | 2652068 |
| display_max_frames | 52 |

rule 関数呼出し合計 3015、expression 関数呼出し合計 8700、rule 自己時間合計 26.488 ms

| rule | calls (consumed) | calls (matchOnly) | 自己時間 % | 包含時間 ms | 自己 ns/call |
|---|---:|---:|---:|---:|---:|
| r82 SliceBaseReceiver | 128 | 0 | 6.6 | 3.748 | 13619 |
| r86 SliceBaseExpression | 144 | 0 | 5.8 | 5.283 | 10659 |
| r51 RandomFunction | 63 | 0 | 4.5 | 1.196 | 18989 |
| r44 MathFunction | 63 | 0 | 4.2 | 9.128 | 17516 |
| r87 SliceNestedExpression | 16 | 0 | 4.0 | 6.301 | 66959 |
| r41 NumberTerm | 47 | 0 | 3.1 | 15.091 | 17510 |
| r121 VariableRef | 175 | 0 | 2.8 | 0.736 | 4205 |
| r106 IfExpression | 95 | 0 | 2.7 | 0.723 | 7612 |
| r55 FloorFunction | 63 | 0 | 2.3 | 0.615 | 9759 |
| r40 NumberExpression | 32 | 0 | 2.1 | 15.804 | 17447 |
| r49 MinFunction | 63 | 0 | 2.1 | 0.558 | 8858 |
| r48 SqrtFunction | 63 | 0 | 2.1 | 0.550 | 8728 |
| r108 TernaryExpression | 63 | 0 | 2.1 | 0.546 | 8674 |
| r50 MaxFunction | 63 | 0 | 2.0 | 0.539 | 8563 |
| r56 PowFunction | 63 | 0 | 2.0 | 0.536 | 8512 |
| r60 NumberFactor | 63 | 0 | 2.0 | 14.000 | 8510 |
| r47 TanFunction | 63 | 0 | 2.0 | 0.532 | 8438 |
| r59 ToNumFunction | 63 | 0 | 2.0 | 0.518 | 8229 |
| r54 CeilFunction | 63 | 0 | 1.9 | 0.509 | 8087 |
| r58 ExpFunction | 63 | 0 | 1.9 | 0.509 | 8082 |
| r109 NumberMatchExpression | 63 | 0 | 1.9 | 0.507 | 8045 |
| r45 SinFunction | 63 | 0 | 1.9 | 0.503 | 7989 |
| r57 LogFunction | 63 | 0 | 1.9 | 0.496 | 7869 |
| r46 CosFunction | 63 | 0 | 1.9 | 0.494 | 7842 |
| r52 AbsFunction | 63 | 0 | 1.9 | 0.494 | 7838 |

| expression | 所属 rule / 種別 | calls (consumed) | calls (matchOnly) |
|---|---|---:|---:|
| e1014 | VariableRef Seq | 175 | 0 |
| e681 | SliceBaseExpression Choice | 144 | 0 |
| e659 | SliceBaseReceiver Choice | 128 | 0 |
| e922 | IfExpression Seq | 95 | 0 |
| e271 | ExternalNumberInvocation Seq | 63 | 0 |
| e272 | ExternalNumberInvocation Literal | 63 | 0 |
| e378 | NumberTerm Seq | 63 | 0 |
| e379 | NumberTerm RuleRef | 63 | 0 |
| e384 | MulOp Choice | 63 | 0 |
| e387 | MathFunction Choice | 63 | 0 |
| e388 | MathFunction RuleRef | 63 | 0 |
| e389 | MathFunction RuleRef | 63 | 0 |
| e390 | MathFunction RuleRef | 63 | 0 |
| e391 | MathFunction RuleRef | 63 | 0 |
| e392 | MathFunction RuleRef | 63 | 0 |
| e393 | MathFunction RuleRef | 63 | 0 |
| e394 | MathFunction RuleRef | 63 | 0 |
| e395 | MathFunction RuleRef | 63 | 0 |
| e396 | MathFunction RuleRef | 63 | 0 |
| e397 | MathFunction RuleRef | 63 | 0 |
| e398 | MathFunction RuleRef | 63 | 0 |
| e399 | MathFunction RuleRef | 63 | 0 |
| e400 | MathFunction RuleRef | 63 | 0 |
| e401 | MathFunction RuleRef | 63 | 0 |
| e402 | SinFunction Seq | 63 | 0 |
