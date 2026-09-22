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

## REGEX の DFA 計画

- `regex.rs` は Java Pattern のリテラル、`.`、文字クラス（否定・範囲）、
  `\d/\w/\s` と各補集合、制御文字/記号の escape、グループ（`(?:...)` を含む）、
  選択、`* + ? {n} {n,} {n,m}`、`^` を Thompson NFA → 優先順位付き部分集合 DFA にする。
  `^` は token 開始位置でのみ通る epsilon とし、`a^b` を `ab` に変えない。
  `.` は Java 既定の改行集合 LF/CR/NEL/LS/PS を除く。`\d/\w/\s` は ASCII 集合。
- 正則と証明できない構文（後方参照、先読み/後読み、`\b`、Unicode property、flags、
  lazy/possessive、class の交差/入れ子、nullable 本体の複数回反復など）は
  JavaPattern に委譲し Rust は生成時エラー。
  nullable 本体の反復は Java のゼロ幅進捗停止により探索順が変わるため委譲する。
  DFA/NFA 各10,000状態、構成処理量2,000,000、入れ子128段、pattern 65,536 byte、
  class 1,024要素を上限とし、超過も同じ fallback にする。DFA は最小化しない。
- `plan.lexical[].dfa` の既存状態別形式を保持し、pipeline は同じ表から token spec に
  optional な `dfa={states,start,accepting,transitions,anchoredAtStart:true,semantics:"leftmostFirst",
  zeroWidth:"fail"}` を追加する。`states` は状態数、ID は0始まりの連番、遷移は
  `{from,ranges:[[lo,hi]],to}`。未構成は `dfa:null,fallback:"javaPattern"`。
  新旧の表・fallback の不整合は IR の参照検証で拒否する。
- 入力の現在位置から code point 閉区間で遷移し、最後の正長受理位置を記録する。
  開始状態が受理でも最終候補がゼロ幅なら失敗。ε-閉包は左枝・貪欲の継続枝を優先し、
  受理より低優先の状態を切り捨てる。同じ集合でも順序が異なれば別の DFA 状態。
  Java Pattern の `lookingAt` と同じく `a|ab` は `ab` を1文字受理する。
  受理時には停止せず、入力終端または遷移なしまで走査し、最後の候補を採用する。
- `tests/regex_dfa.rs` は Java 21 の `Pattern.lookingAt` 実測値を固定し、
  選択順・貪欲・バックトラック・ゼロ幅・非 BMP を独立評価器で検証する。
  既存 oracle の ascii/zero の受理・消費・照合長と一致。
  prefix/emoji は C15 の非 BMP 既知差分を明示的に検査する。prefix の Java 実走は
  `😀` の後の `a` に失敗し、emoji は単独 `😀` に失敗、`😀x` 全体を消費する。
  DFA は `😀` を1 code point とし、これらを Java の挙動に合わせて歪めない。
- 左再帰の lowering 履歴（normalizations / directRewritten）は保持するが、
  実行本体は通常の Seq/Repeat なので `leftRecursionRewrite` を requiredFeatures に要求しない。
