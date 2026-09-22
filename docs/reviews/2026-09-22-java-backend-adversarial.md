(A) 契約違反の一覧

結論: 基本ケースは再現できるが、実行契約を満たす状態ではない。認識結果が変わるカーソル不具合と、self-host の AST 欠落・例外を確認した。

許可された Maven package は成功。oracle 由来 42 文法を生成・javac --release 21 -Xlint:all -Werror でコンパイルし、129 ケースを実行した（既存の受理観測があるのは119ケース）。追加の adversarial driver と既存 Java の実走も行った。git 操作・ソース変更は行っていない。
調査中に対象ソースと target/classes の更新を検出したため、snapshot/ 以下に固定した。以下の行番号はそのコピーを基準にする。核心の Session.java は開始時の生成物と同一。
略記: J=ubnfc-java/src/main/java/org/ubnfc/java/emit/JavaEmitter.java、S=ubnfc-java/src/main/resources/templates/internal/Session.java、V=ubnfc-java/src/main/java/org/ubnfc/java/ir/Validation.java。
重大度: P1=認識・AST・生成の破綻、P2=診断・互換性の破綻。

1. P1: mixed の list mapping が空要素を null にし、複数ノードは先頭だけ採用する。
   self-host 生成パーサに grammar G { R ::= 'a'; } → NullPointerException。期待は空 annotations を持つ AST。
   grammar G { @root @mapping(V) R ::= 'a'; } → annotations=[RootAnnotation]。期待は RootAnnotation と MappingAnnotation の2件。
   原因: S:125 が conversion=node の list だけを展開し、J:333 の mixed は S:450 の getFirst() を利用。空の null は J:140 の List.copyOf に渡る。生成物 UBNFAST.java:56、UBNFParser.java:44 でも確認。
2. P1: repeat が毎周回で余分な transaction を開始し、R=true の M を C に戻す。
   IR lookahead(repeat(any,2,2))、入力 abc → 成功 C=0,M=1。既存 MatchOnly(Repeat(ANY,2,2)) は成功 C=0,M=2。
   {3,3} → 生成側失敗 C=0,M=0、既存側成功 C=0,M=3。S:242 の f.copy(true) が原因。既存 Occurs.java:53 は同一 frame で子を実行する。
3. P1: optional の失敗時に consume(0) の M リセットを捨てる。
   token L=LOOKAHEAD('a'); Root ::= L ['x']; 入力 ab → 生成側 [成功,0,1]、既存 Java [成功,0,0]。
   S:221 の optional は子失敗で親を変更せず返す。契約上は Occurs の成功結果として子が変更した M を引き継ぐ必要がある。
4. P1: 空先読みパターンの成否が逆。
   LOOKAHEAD('') は空入力・ab とも生成側成功、既存側失敗。NEGATIVE_LOOKAHEAD('') は生成側失敗、既存側成功。
   J:264 → S:285 が空文字列一致を成功扱いする。compat-matrix.md の C09 に明示された契約違反。
5. P2: EOF・反復の失敗で診断を登録しない。
   EMPTY{0,0}/ab と EMPTY{2,2}/ab は成否と C=M=0 は一致するが、生成側 offset=1, expected=[]、oracle は offset=0, expected=[Repeat,RootParser]。
   EOF/a も expected=[]、farthestCp=-1。期待は EndOfSourceParser 等の非空 hints。S:247、296、419。oracle/empty-zero.json:21、oracle/eof.json。
6. P2: Not 自身が失敗する場合まで子の投機的失敗を抑止する。
   Not(Choice(Chain('a','x'),'a'))/ab → 生成側 offset=0, expected=[Not]、既存 Java offset=1, expected=['x']。
   S:274 が常に suppressBegin。既存 Not.java:40 は「子失敗で Not 成功」の場合だけ診断を破棄する。
7. P2: tokenRef で expectedDisplay を無視する。
   CI('let')/le に IR expectedDisplay.java=CUSTOM_CI を設定しても expected=[let]。期待は [CUSTOM_CI]。
   J:235 で読んだ label を J:252 の token() に渡さない。通常の oracle でも ['let'] 対 [let]、REGEX('a*') 対 a* 等の差がある。
8. P2: trailing_input の primary expected が投機的失敗の expected になる。
   Root ::= ('a' 'b' 'x') | 'a'; 入力 abq、全入力要求 → offset=1,farthestCp=2 は正しいが expected=['x']。
   期待は primary expected=['end of input']。S:418 が EOF を最遠診断 accumulator に混ぜ、S:432 がその expected をそのまま返す。
9. P2: 行コメントの matched 到達が診断に反映されない。
   oracle comment-line の a//x\nb → 双方 syntax 失敗だが、生成側 offset=4、oracle offset=5。
   S:385 のスカラー走査と S:397 の advance は既存 CPPComment の終端照合を再現しない。oracle/comment-line.json:39,52。

一致を確認できた範囲:
- consume(0): (1,3)→(1,1)。begin の R=true リセット、commit が親 R を変更しないことを FrameDriver で確認。J:226 は caller frame に setFalse してから子を呼ぶ。
- EMPTY/ab=[成功,0,1]、EMPTY{1,2}/ab=[成功,0,1]、EMPTY{0,0}/ab=[失敗,0,0]、LOOKAHEAD('a')→UNTIL('#')/a#=[成功,0,2]。単純な EOF と Not 成功時の子診断抑止も一致。
- chain/group/反復 helper の空白挿入、choice への一律挿入なし、@whitespace(none) の内部 a b 拒否を既存 Java と比較。S:191、226、372。
- memo の C/M、mode/invert/R のビット、stateVersion を確認（S:99）。unsafe Extern の祖先も再実行し scanner=2回。safe では memo ON=1回/OFF=2回で、宣言1件・診断・カーソルは一致。
- scope は enter→leave→親への declare→use、U+0020以下の trim と cp offset=2、先読みからの完全復元を実走。pop済み scope、同名上書き、reference、diagnostic clear の undo も一致。
- 遅延 mapping: 1.5 の失敗枝では例外なし。採用した 1e2/2147483648 は MappingException、42 は int。内部 Recipe と公開 AST record を区別して確認した。(NUMBER) @value の 1.5/1e2 は String。

(B) 生成できなかった文法と不足機能

- P4: 元の IR は Recipe conversion mismatch: recipe:SliceNestedExpression.value で拒否。AST field が text を含む mixed なのに recipe が node。これは frontend/IR の整合性問題で、検証を削除して通すべきではない。
- P4 の次の不足機能は externTokens.inputState。対象は TinyExpressionP4::STRING、::CODE_START、::CODE_END。
  要求状態は consumed, matched, mode, invertMatch, resetMatchedWithConsumed。TokenScanner.java:7 は consumed/matched しか受け取らず、V:168 は inputState 非空を全拒否する。
  一時 IR コピーで recipe conversion を mixed にしたところ Extern 拒否を実測。さらに inputState を外した探索用コピーは生成・コンパイル成功したが、P4 の意味論検証には数えていない。
- ubnf.ubnf: 生成・コンパイルは成功し、元文法の全9646 code pointを認識した。ただし AST 有効時は上記 NPE。frontendTransform は未対応で、この IR 自体も construct recipe のみ。GrammarAst 変換・固定点成立まで達していない。
- 追加確認した22 fixtureでは、eval-default.ubnf が eval、positive/elements.ubnf が typeof、mixed-values/Shared.ubnf が recipe:Second.value の conversion mismatch で拒否。
- oracle 文法では ERROR() が execution.transaction:error:once、recovery-auto/root/skip/sync が recovery.legacyMarker で拒否。実装済み表記と front が実際に出す IR の組合せに隔たりがある。

(C) emitter の設計上の問題

- P1: 合法な Java 識別子 java が名前衝突を起こす。list field java は java = java.util.List.copyOf(java) を生成して compile error。record 名 java も java.lang.String を隠して compile error。J:135,140、V:19 の単語単位検証だけでは不足。
- P2: 再帰は JVM スタックに依存。Root ::= '(' Root ')' | 'a' に括弧2000段で StackOverflowError（memo ON/OFF とも）。S:112 と生成関数の直接再帰。受理可能な深さの制限も診断もない。
- memo の先頭キーは ExprId そのものではなく ruleId（S:103）。rule 単位の識別はできるが、非 rule-body の ExprId memo 計画は実行されない。J:205、README:133,152。契約上の式単位 memo と区別すべき。
- scope の公開結果は宣言・参照・診断だけ（api/ParseResult.java:13）。enter/leave の順序付きイベントと lexical/dynamic mode を外部に渡さず、README:109 の metadata 対応表記を裏付けない。
- 対応判定が frontend の出す IR を網羅していない。独立 IrFixture の成功だけでは ERROR、P4、self-host の実用経路を保証できない。
- 決定性は固定した同一 IR・同一 generator から20 Javaファイルの byte 一致を確認。調査途中の版更新による差は決定性の欠陥に数えていない。
- Java 予約語 class は生成前拒否。日本語 rule 名2件を持つ手製 IR は衝突せずコンパイル・実行成功。front の日本語識別子拒否は emitter の欠陥ではない。
- ExprNode 15種類、TokenSpec 12種類の switch 分岐は schema と照合済み。問題は分岐漏れより、execution 組合せ・渡されない label・状態遷移にある。

(D) 修正の優先順位付き提案

1. P1: Session.repeat()/optional() の transaction 境界を Occurs と一致させる。R=true の matchOnly 反復と optional 失敗後の M を独立回帰テストにする。Session.literal()/JavaEmitter.token() に空 Word パターンの失敗契約を適用する。
2. P1: Session.rule()/list()/node() と JavaEmitter.convert()/astType() で mixed list の全 occurrence・空 capture を正しく扱う。null を安易に許可して隠さず、self-host の無注釈・複数注釈を golden にする。
3. P2: Session.repeat()/eof()/look()/trivia()/result() に失敗候補と frame 到達履歴を実装し、Not 成功時だけ投機診断を破棄。primary expected と farthest expected を分け、JavaEmitter.token() へ表示ラベルを渡す。
4. P1: Validation.identifier() と JavaEmitter.astType()/javaType() に生成スコープを考慮した名前検証または名前変換を追加。java の field/type の javac 検証を固定する。
5. P1: P4 は crates/ubnfc-front/src/lower/schema.rs の recipe() と captures.rs の conversion()、Java の TokenScanner.scan()/Session.external()/Validation.validate() を整合させる。self-host は合意した frontendTransform と occurrence→GrammarAst の経路を実装する。
6. P2: JavaEmitter.emitRule()/Validation.validate() の式単位 memo の扱い、ParseResult.Scope のイベント・mode 公開、再帰深度の扱いを明文化。README の対応表は生成・コンパイル・認識・mapping・self-host を分けて記載する。

(E) 未確認事項

- 全 corpus 約700ケース、P4の実scanner付き実行、self-host の GrammarAst/IR byte固定点、suffix併存優先順位の端から端までの一致は未確認。
- deepest rule の既存 Java との完全一致、全ての mode/invert/R 組合せ、全 Extern・recovery 経路は未確認。代表例の成功を全契約の証明とはしていない。
- 非BMP REGEX は既存 oracle と4ケースで差分を確認。compat-matrix C15 が旧 UTF-16 バグの互換採用を未確定としているため、契約違反には計上していない。
- 固定 snapshot 以降の並行更新はこのレビューの対象外。
