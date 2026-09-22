保存先: [review.md](/tmp/tmp.tW8X6MP8bN/review.md)

Rust バックエンドには、認識・scope・診断・実行資源保護に修正が必要です。Java の反復と self-host AST の不具合は再現しませんでした。
両リポジトリのファイル変更・git 操作・ネットワーク利用は行っていません。対象ソース22ファイルの開始／終了 SHA-256 は一致しています。

**(A) 契約違反一覧**

P1＝認識／意味情報の破綻、またはプロセス停止・著しい計算量増大。P2＝診断・位置・公開情報の不整合。C/M は code point。
まず Java レビュー(A)の全9項目を、同じ再現入力で検証した結果です。

| Java項目 | Rust の観測 vs 期待 | 判定・共通性 |
|---|---|---|
| A1 mixed list | `grammar G { R ::= 'a'; }` は annotations=[]。`grammar G { @root @mapping(V) R ::= 'a'; }` は注釈2件。双方 AST 成功 | Java の例外・要素欠落は再現せず |
| A2 matchOnly repeat | `lookahead(repeat(any,2,2))/abc` → 成功 C=0,M=2。`{3,3}` → 成功 C=0,M=3 | 期待一致。Java の transaction 不具合は再現せず |
| A3 optional | `L=LOOKAHEAD('a'); Root ::= L ['x']; / ab` → 成功 C=0,M=0 | 期待一致。Java の M 保持は再現せず |
| A4 空先読み | `LOOKAHEAD('')` は空入力・`ab`で成功、`NEGATIVE_LOOKAHEAD('')` は双方失敗。期待はそれぞれ失敗／成功 | **P1、Java共通**。compat-matrix C09 違反 |
| A5 失敗診断 | `EMPTY{0,0}/ab`、`EMPTY{2,2}/ab`、`EOF/a` は失敗 C=M=0、syntax@0、expected 非空 | Java の診断欠落は再現せず。hints 完全集合の一致とは区別 |
| A6 Not | `Not(Choice(Chain('a','x'),'a'))/ab` → syntax@0、expected=[expression]。期待は子の失敗@1、expected に x | **P2、Java共通**。Not 自身が失敗する場合にも子診断を破棄 |
| A7 表示ラベル | `CI('let')/le`、expectedDisplay.rust=`CUSTOM_CI` → expected=[CI]。期待は [CUSTOM_CI] | **P2、Java共通**。tokenRef からラベルを渡さない |
| A8 trailing | `Root ::= ('a' 'b' 'x') \| 'a'; / abq` → diagnostics は syntax@2[x]、trailing_input@1[end of input] の順 | **P2、主診断の問題は共通、形態はRust固有**。trailing の値は正しいが、protocol の先頭診断比較は syntax を主診断として拾う |
| A9 行コメント | `comment-line` の `a//x\nb` → 拒否、syntax@4。保存 Java oracle は @5 | 同じ差を観測。ただし Rust は fail_at、Java は frame の最大到達という別契約。D-017/D-018 のコメント扱いもあり、この差だけを Rust 違反に計上しない |

追加で確認した問題です。「Rustで追加検出」は、Java 側の同一ケースは未確認という意味です。

| 重大度 | 再現入力・設定 | 観測 vs 期待 | 共通性 |
|---|---|---|---|
| **P1** | scope corpus `quantified-capture-occurrences` の `aa\|bb\|c,c` | 宣言は a@1,b@4,c@8 の3件。期待は a@0,a@1,b@3,b@4,c@6,c@8 の6件。AST の値は全件存在 | Rustで追加検出 |
| **P1** | `repeated-capture-occurrences` の `a,b;a,b,missing` | 参照が a@4,missing@8 のみ。期待にある b@6 が欠落。同じ capture site の最後だけを採用 | 上と同根、Java未確認 |
| **P2** | `trimmed-group-non-bmp` の `>  😀x  <? 😀x ` | 名前は正しく `😀x`。宣言 offset=1、参照 offset=9,length=4。期待は宣言3、参照10,length=2 | Rustで追加検出 |
| **P2** | `RightScoped`、` x /*r*/ ^ x ` | 右子 Power.span=[11,13]。corpus の期待は [11,12]。末尾 trivia が子 node span に入る | 生成経路の不一致。front/IR と emitter の責任分界は未確認 |
| **P2** | `Root ::= ('a' 'b' 'x') \| 'a'; / abx`、lexical=true | lexical は rule occurrence 1件だけ。期待される未captureリテラル3件がない | Rustで追加検出。設計§3.6違反 |
| **P1** | `D0='x'; Dn=D(n-1) \| D(n-1)`、空入力 | 深さ24/26/28のdriver実行が0.445/1.828/7.497秒。共有診断DAGを指数回展開。深さ20でも評価41回・memo61件に対し診断走査は大量反復 | RustのDAG実装固有 |
| **P1** | `Root ::= '(' Root ')' \| 'a'`、括弧20,000段 | release driver が SIGABRT（stack overflow）。深度制限や構造化エラーへの変換なし | 再帰保護不足はJava共通。Rustはプロセスabort |

確認範囲・正常だった点:

- 保存 oracle 34文法を試し、生成できた23文法・74ケースの accepted/C/M は一致。Java専用の数値型・診断offsetをRustへ一律適用していません。
- rule-trivia 33件の受理・C/M は一致。AST は上記の右結合span 1件が不一致。scope-annotations 14件の受理・C/M・AST は一致し、副作用には上記の欠落・位置差がありました。
- カーソル／memo 全7キー成分の128組を区別。begin の R、consume(0)、matchOnly、commit の親flag保持を確認。`EMPTY/ab=[true,0,1]`、`LOOKAHEAD('a')→UNTIL('#')/a#=[true,0,2]` も一致。
- 安全宣言した Extern は巻戻し後 scanner=1回・memo hit=1、unsafe版は2回・hit=0。両者とも採用宣言1件、診断・C/M一致。既存のmemo occurrence／診断path再生テストも成功。
- scope store の undo・上書き・pop・clear・epoch を含む共通46テスト成功。ただし store の正しさは、emitter の全capture出現処理を保証しません。
- 遅延mappingの生成検証は成功し、採用結果のmapping失敗は構文成功と区別。NUMBER の `1.5`、`1e2`、`2147483648`、`42` は例外なく字句を保持し、Rustの契約に一致。
- self-host は元ソースの改行を保持した **9,646 code point 全体を認識し、AST構築も成功**。GrammarAst変換・IR固定点の達成を意味しません。

**(B) 生成できなかった IR と不足機能**

`ir/fixtures/*.ir.json` 全22件中 **18件生成・release/offline build成功、4件拒否**。別途 `ir/examples/tinycalc.ir.json` も成功しました。
以下のパスは `ir/fixtures/` 相対です。

| IR | 実測された拒否機能 |
|---|---|
| `tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json` | `catalog` |
| `rust__unlaxer-ubnf__tests__fixtures__known__eval-default.ubnf.ir.json` | `eval` |
| `rust__unlaxer-ubnf__tests__fixtures__positive__elements.ubnf.ir.json` | `typeof` |
| `story__code__calc-leftrec.ubnf.ir.json` | `leftRecursionRewrite` |

- P4のcatalog宣言とrequiredFeaturesだけを除いた探索用コピーは生成・build成功。今回の元IRでは、Javaレビュー時のrecipe conversion mismatchは確認されませんでした。RustのExtern APIは要求されたC/M/mode/invert/Rを渡せます。実STRING/CODE_START/CODE_END scanner付き実行は未確認です。
- 追加oracleでは recovery-auto/root/skip/sync が `recovery`、ERROR() が `execution.transaction: Once for Error`、ci-unicode が `caseInsensitive.unicode` で拒否。
- regex-ascii/prefix/zero/emoji/backref はすべて `regex: requires an analyzed DFA`。正則な `[a-z]+` も現在のfrontend出力では生成できません。手製DFAを供給した `😀+` は生成・実行成功、ゼロ幅は拒否しました。
- `structuredOccurrences.frontendTransform` は `validate.rs::validate()` が明示拒否。self-hostの認識成功と、設計のfrontend固定点成立には隔たりがあります。

**(C) emitter／テンプレートの設計上の問題**

基点は `crates/ubnfc-rust/`。実測資料とソースsnapshotは保存先と同じディレクトリにあります。

- `src/expressions.rs::scope_events()` と `templates/runtime.rs::scope_capture()`（522行）は `.last()` で出現を捨てています。名前だけを `text()` でtrimし、位置には元spanを使うこともscope不一致の直接原因です。
- `templates/finish.rs::occurrences()` は Event::Rule だけをlexicalへ出力し、Event::Tokenを無視します。Token自体にもExprIdがありません。`api.rs::ParseResult` には順序付きscopeEvents・lexical/dynamic modeがなく、この公開契約欠落はJavaレビューとも共通です。
- `finish()` は局所syntax診断をtrailing診断より先に追加します。`templates/driver.rs::execute()` はその順序を維持し、farthest/deepest/rule_pathも出力しません。API内部の診断情報とprotocolで観測できる情報が一致していません。
- `templates/diag.rs::Diagnostics::summary()`（52行）は共有節点を再訪し続けます。DAG保存量が小さくても展開時間を抑えられず、expectedの重複除去も線形検索です。破棄された診断節点はsession終了まで残ります。
- `templates/memo.rs::PackedKey::hash()/Memo::insert()` は固定hash・線形probe・負荷率約75%で拡張。同一bucketへの集中は二次時間です。単体計測で512/1,024/2,048衝突キーのinsert+getが173/695/2,792µs、通常キーは3.6/7.0/14.5µs。実parser入力からの同等攻撃は未確認です。
- `runtime.rs::Session::new()` のmemo予約上限32,768、arena予約上限32,768は**初期容量**です。総量制限ではなく、その後増えます。PackedKey/Savedのu32変換やscope epoch枯渇はpanic経路を持ち、4GiB超入力等の事前拒否はありません（巨大入力実走は未確認）。
- `emit.rs::generate()` の生成関数呼出しと `mapping.rs` のbuild_values/build_ruleは再帰に依存し、深度予算を持ちません。
- 生成19 crateと追加probeは `cargo clippy --offline -- -D warnings -F unsafe-code` 成功。標準featureの通常依存はゼロ。ただし **生成rootに `#![forbid(unsafe_code)]` はありません**。json featureはoptionalなserde/serde_json依存です。
- AST・ParseResult・カーソル・memo・diag・scope・Inputの代表型は `Send + Sync` をコンパイル確認。ローカルStringを借用した並行parseも成功。`&'static` はID/label文字列に使い、文法オブジェクトグラフはありません。TokenScannerにSend/Sync boundはなく、任意scannerを含むSession全体の保証ではありません。
- UTF-8検証は全12 TokenSpec種、全11 Builtin種を対象に実施。494ケースにCI・DFA・Externの追加ケースを実行し、panicなし。`😀`のbyte 1/2/3はInputでNone、Externの同境界は構文失敗として拒否しました。
- driverは生の`😀`と`\uD83D\uDE00`を同じ1cpとして扱い、巨大整数idと制御文字escapeを保持。孤立surrogate、不正escape、JSON深度超過をエラー1行にし、後続リクエストを処理しました。
- 同一TinyCalc IRからの生成14ファイルはbyte一致。識別子は `ast.rs::ident()` の決定的エンコードを使用します。READMEの「全Builtin」「scope」「JSON全標準観測」と、実用frontend経路・出現／metadata公開の達成状況は分けて記載すべきです。

性能は実fixtureのOperators生成crateで計測しました。Ryzen 9 7950X、WSL2 Linux、rustc 1.98.1、release、CPU固定なし。
毎回新規parse、warmup100回、5測定の中央値。512 atom／深さ64は各200回、5,000 atomは各50回。JSON・評価器は計測外です。

| 入力 | 認識のみ µs/parse | AST付き µs/parse | peak RSS KiB 認識／AST | peak要求heap byte 認識／AST |
|---|---:|---:|---:|---:|
| 512 atom | 416.0 | 714.2 | 2,768 / 4,064 | 724,136 / 1,393,207 |
| 括弧深さ64 | 110.6 | 134.3 | 2,484 / 2,828 | 183,624 / 269,277 |
| 5,000 atom | 13,258.6 | 10,485.1 | 12,184 / 21,728 | 8,127,112 / 15,222,775 |

全モード受理・AST有無を検査。heapは別プロセスの計測用allocatorによる最大live要求量、RSSはtimingプロセス全体のVmHWMです。両者は異なる量です。
5,000 atomでmemo=80,002件、AST arena=59,996件まで拡張。AST側が速い測定値から一般的優位は主張せず、allocator・配置等の原因は未確認です。
ログ: [時間](/tmp/tmp.tW8X6MP8bN/bench-final-time.txt)、[確保量](/tmp/tmp.tW8X6MP8bN/bench-alloc.txt)、[DAG](/tmp/tmp.tW8X6MP8bN/dag-results.json)、[scope等の生成検証](/tmp/tmp.tW8X6MP8bN/families.log)。

**(D) 修正の優先順位付き提案**

1. **P1・scope**: `runtime.rs::scope_capture()` を全出現列の取得へ変更し、`expressions.rs::scope_events()` で各siteの出現を完了順に統合してdeclare/useを発行。trim後のbyte spanからcp offset/lengthを計算する。上記3つのscope corpusを回帰条件にする。
2. **P1・空先読み**: `src/tokens.rs::generate()` のLookahead/NegativeLookaheadにWordの空パターン失敗を実装する。通常の空Literalまで一律拒否しない。
3. **P1・資源保護**: `diag.rs::summary()` を節点単位の局所要約＋呼出しpath再接続へ変更。単純なvisited集合でdeepestを失わない設計にする。`emit.rs::generate()` とmappingの再帰に明示stackまたは深度予算を導入し、abort前に構造化診断を返す。
4. **P2・診断**: `expressions.rs::generate()` のNegativeLookaheadで自身失敗時のchild.diagを保持。TokenRef→token関数へrust表示ラベルを渡す。`finish.rs::finish()`／`driver.rs::execute()` で主診断・farthest・局所hints・deepestを区別して出力する。
5. **P2・出現／span**: `runtime.rs::Event::Token` にExprId/RuleIdを保持し、`finish.rs::occurrences()` にtoken出力を追加。`emit.rs` のRule span生成と `mapping.rs` のnode構築で末尾trivia境界を検証。`api.rs::ParseResult` に順序付きscopeEventsとmodeを公開する。
6. **生成機能**: `validate.rs::validate()`、`src/expressions.rs`、`src/mapping.rs` でERRORのOnce正規形、catalog、eval、typeof、左再帰loweringを段階的に実装。regexはfrontのDFA計画供給と接続し、未対応を検証削除で通さない。self-host用frontendTransformを実装する。
7. **品質／性能**: `emit.rs::generate()` のcrate rootへforbid属性を追加。`memo.rs` にprobe長の観測・集中対策、`runtime.rs::Session::new()/store()` に入力／arena／効果保存の資源予算を設ける。READMEを生成・build・認識・mapping・公開metadata・固定点の別々の達成状態へ更新する。

**(E) 未確認事項**

- 約700件の全corpus、P4の実scanner付き実行、GrammarAst変換とIR byte固定点、suffix併存の完全互換は未確認です。
- 新規scope／lexical／右結合span問題のJava側再現は未確認です。scopeのtrailing入力ケース `[kept]kept!` ではdriverが宣言・参照を空にしますが、prefix由来scopeと全入力由来scopeのprotocol上の位置付けは要整理です。
- 右結合span差のfront／emitter責任分界、全execution組合せ、極大入力のu32上限、scope効果保存量の最悪計算量は未確認です。
- `comment-line` のJava oracle値をRust診断へ直接適用していません。新コメント契約、言語別hints/deepestの完全な独立oracle整備は残ります。
- 性能はこの環境・文法での実測です。Java／旧Rustとの同時比較、P4性能、深度の保証値は未確認です。
