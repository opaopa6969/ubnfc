# semantics → analysis / plan

`analyze(&Semantics)` は入力を変更せず `Result<(Analysis, Plan), Vec<AnalysisError>>` を返す。
GrammarIr に戻す呼出側は `plan_required_features(&plan)` を既存 requiredFeatures に追加し、
feature 名順に並べる。`scope_schedule` は rule の enter → 本体評価 → leave → declare → use
を返す。本体内の分岐・反復・呼出しは Expr 木に従い、失敗時は実行側の undo 契約を使う。

- 式の子参照と RuleRef を依存グラフにし、Tarjan SCC の依存先から nullable / mayStart /
  minConsume を同時に固定点まで更新する。min の初期値は未到達を表す内部の無限大。
  全式を出力するが、左再帰・rule SCC は実行中の body から求め、正規化前の保存式を除く。
  左端参照グラフは nullable prefix をたどって別に構築する。未正規化の直接左再帰、
  間接左再帰はエラー。正規化済みは lowering の Rewrite 記録を検証して記録する。
- ASCII は u128、非 ASCII は昇順の閉区間で計算し、IR 出力時だけ ranges に統合する。
  Negation は文字集合の補集合で EOF 不可。負の先読みは全文字・EOF を残す条件付きゼロ幅。
  正の先読みも C と M の異なる状態を誤除外しない上界。Extern は unknown・全候補。
  Unicode の Java equalsIgnoreCase と正規表現の FIRST は保守的に全文字を残す。
  trivia は insertionPlan の実際の境界で開始集合へ加える。recovery は適用 site だけ未知扱い。
- 自動 Predict は非 nullable、未知/trivia なし、pairwise disjoint の単文字 literal 選択に限定。
  除外された不一致候補の失敗位置は入口と確定できる。backend は元の訪問順に診断と
  モードに応じた consume(0) の効果を復元する。長い literal は trie だけを構築し Ordered を維持。
  明示 predictive は predict + fullChoice。候補なし・全失敗の両方で完全選択を再試行する。
  trie は完了済みの短い候補と未知候補を後続 node に残し、元の候補順を保持する。
- memo は全 rule 本体を保持し、判定不能/再帰 SCC は全体を保持する。保証のない Extern の
  依存元閉包は成功・失敗とも対象外。保証は token alias に対応する userAliasAssertion を確認。
  省略は非再帰・trivia/効果なし・単一所有の式に限定し、rule 本体 memo を支配点とする。
  Seq/Group の単一パス、fallback なしの排他選択、minConsume > 0 の反復の単調前進だけを
  通り、巻戻して同じ式へ再訪する祖先を通らない。証明の kind とパスを omissionProof に残す。
- 演算子は左の Base { Op Base }、右の Base Op Self | Base を検算し、数値 precedence ではなく
  base 参照で塔を並べる。trivia/先読み/scope/Extern/recovery、演算子接頭辞の重複、前進不明、
  不足した lift recipe、operator metadata 外の capture があれば tierFunctions と理由を出す。
  単純な無注釈の同形も検出する。climbing の実行コードは backend の担当。
  テストの独立した段別/climbing 評価器は左右結合4通り・括弧・不完全入力78,144ケースで
  AST、span、消費位置を比較する。生成 Java/Rust backend の実測結果を意味するものではない。
- AUTO 要求時だけ終端文字列 FIRST/FOLLOW の有限集合を解く。nullable suffix、呼出し、
  反復、区切りを伝播し、opaque token の spelling は捏造しない。空なら `;`、他 rule は null。
  capture は同名の出現を Seq で加算、Choice で区間和、量指定で乗算して lowering 値を検算。

## tinycalc の記録済み差分

`tests/analysis_golden.rs` で以下の変更だけを期待値に適用し、analysis/plan 全体を比較する。
nullable、minConsume、文字範囲/EOF/ゼロ消費条件、rule 解析、trie は実例と一致する。

- Literal / builtin TokenRef の13式を conservative → exact に精密化。
- transaction の16式の失敗は restore、R を変更しない18式は preserve に精密化。
- AddOp / MulOp の2選択を Predict にし、proof を追加。3つの既存 trie を plan から参照。
- 上記2 rule の本体 memo は保持し、4つの literal 式だけ証明付きで memo を省略。
  2 SCC の memoPolicy と plan.profile を更新する。
- 演算子塔は trivia のため tierFunctions のまま。proof に非適用理由を追加する。
- GrammarIr に統合する際は requiredFeatures に predictiveChoice を追加する。

## IR v1 に専用欄がない情報

- ASCII bitset は ranges に直列化する。Predict の文字→候補表は candidateOrder と各候補の
  mayStart から作る。PredictiveWithFallback という列挙値はなく predict + fullChoice で表す。
- `ExprAnalysis.effects` は unique な文字列の要約なので、実行回数・再帰・分岐付きのイベント列は
  表現できない。`scope:<ruleId>:<index>` は semantics.scope.events の参照であり、実行列ではない。
  `scope_schedule` と Expr 木が実行順を与える。再生はこの文字列要約を走査して行わない。
- CursorTransition は選択カーソルの進行要約だけで、M のリセット/後退条件、EMPTY の M+1、
  UNTIL の終端 commit を数式として格納できない。semantics.execution / contracts が正本。
- climbing 非適用理由と診断再現手順の専用欄はなく、Proof の kind/exprIds/explanation に記録する。
  DFA 生成は本解析の範囲外。正規部分集合は scalar、非正規 regex は JavaPattern の計画とする。
  既存 ir API / schema / examples は変更していない。
