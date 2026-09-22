# ubnfc 設計 v1

> 状態: v0 に astra（codex gpt-6-astra）レビューを統合。決定は `docs/decisions/D-*.md`。
> 2026-09-22

## 0. v0 からの変更点（要約）

| # | v0 の主張 | v1（astra 指摘を採用） |
|---|---|---|
| 1 | 解析状態は `int pos` と AST だけ | **consumed / matched の二重カーソル + mode / invert / R フラグ**（`04-cursor-and-diagnostics-contract.md`）。`LOOKAHEAD` 等は consumed を進めず matched を進める（corpus `primitives` が `[成功, consumed=0, matched=1]` を要求） |
| 2 | `'let'` は `letter` を横取りしない | **リテラルは接頭辞一致、単語境界なし**（`WordParser` は code point 長だけ比較。`ParserRuleEmitter.java:1067,1182`）。境界を足すのは非互換 |
| 3 | 予測分岐で backtrack を減らし memo は限定的 | **予測分岐と memo は併用**。memo は「省略が安全と証明できた箇所だけ省略」（`S ::= 'a' S 'b' \| 'a' S 'c' \| EMPTY` で予測だけだと 31/511/8191 回、memo で 5/9/13 回） |
| 4 | CST を作らず typed AST を直接構築 | CST オブジェクト木は作らないが、**capture 出現・字句出現（未 capture のキーワード/演算子）・scope イベント・診断**を結果に含める。P4 LSP の semantic tokens と self-host（`AnnotatedElement` の量指定子は capture されていない）が要る |
| 5 | 失敗枝の AST は捨てるだけ | **失敗し得る意味変換（`Integer.parseInt`、`@typeof` 型検査）は採用された結果へ遅延**。旧実装で到達しなかった例外を出さない |
| 6 | scope rollback はログ長 truncate で O(1) | **undo ログ**。pop 済み scope・同名再宣言の lookup 上書き・診断 clear は append の逆ではない |
| 7 | 直接左再帰は既存もループ化 | 既存 Java は警告のみでループ化しない（`GrammarValidator.java:291`）。ubnfc は**直接左再帰を書き換える（拡張）**。決定 D-010 |
| 8 | precedence climbing は M7 で | 条件付きで両立可（223,948 ケースの試作で AST/span/位置一致）。ただし trivia 境界計画が要る。**保守的生成を先に、最適化生成は差分比較で採用**（M5） |
| 9 | 最遠失敗 + expected だけ | 診断は **corpus が観測する範囲**（`evolution/diagnostics.json` の offset、expected hints、deepest rule）を契約化。memo hit 時の局所診断再生を含む |
| 10 | M3/M4「corpus 緑」→ M5 で scope | 循環していた。**M1 を Java/Rust 両方の縦断**（接頭辞リテラル・二重カーソル・trivia・空 capture・数値失敗・scope rollback・longest）にし、self-host 経路を M2 で成立させる |
| 11 | 非ゴールに「Java int 契約」 | 誤り。Java 厳密変換 / Rust 字句保持は**明示された別契約**で、両方守る |

## 1. ゴール / 非ゴール

### ゴール（第一段の完了条件 = 全互換 + self-host。性能は別建て）

1. unlaxer-parser の全 `.ubnf`（24）+ tinyexpression P4（124 ルール）が生成・コンパイル・実行できる。
2. 既存 corpus（5 系統・約 700 ケース）を両バックエンドで通す。互換の対象は
   **認識結果・prefix/全入力判定・両カーソル・AST 型/値/全出現 span・mapping 失敗・scope 履歴・診断・metadata**。
   言語別 oracle（Java `int` / Rust 字句保持、javaOffset / rustOffset）は維持する。
3. self-host: 生成 Rust フロントエンドが `ubnf.ubnf` を読み、手書きと **同じ IR JSON（byte 一致）**。
   手書き解析へ戻らず固定点（生成→生成）も一致。
4. 生成コードは外部依存ゼロ。
5. **性能目標（互換と分離して報告）**: P4 の complex / complex-x64 / comparison-heavy-x64 で既存 Java facade 比・既存 Rust 比。
   数値目標は M1 の縦断ベンチで初期値を取ってから固定。未達は「未達」と報告する。

### 非ゴール

- LSP / DAP サーバ生成（LSP が要る情報＝字句出現・scope・診断は結果 API で提供）。
- 旧 `Token` / `ParseContext` API との互換（利用側アダプターの仕事）。
- `CodegenMain` CLI 互換。

## 2. 全体構成（不変）

```
ubnfc-front (Rust) : .ubnf → GrammarAst → resolve → validate → 意味論 IR → 解析 → 最適化計画 → IR JSON
ubnfc-rust  (Rust) : IR JSON → *.rs
ubnfc-java  (Java) : IR JSON → *.java
ubnfc-harness      : corpus → 生成 → compile → run → 比較
```

IR は **「意味論」と「最適化計画」を分けて**持つ（D-009）。バックエンドは文字表・分岐形式・SIMD の
選択をしてよいが、capture・trivia・結合性・memo 安全性・カーソル遷移を再解釈しない。
IR に `analysisVersion` と対象機能集合を持たせ、未対応版は明示拒否。

## 3. 実行意味論（互換の核）

### 3.1 二重カーソル

- `consumed`: 実際に消費した位置。`matched`: 照合が到達した位置（≥ consumed）。
- `LOOKAHEAD('p')` / `NEGATIVE_LOOKAHEAD` / `EOF` / `EMPTY` / `MatchOnly` 相当はゼロ消費。
  corpus の `prefix: [accepted, consumed, matched]`（要 harness 確認）を満たす。
- `UNTIL` は `setResetMatchedWithConsumed(false)` で終端の照合位置を残す。
- 生成関数のシグネチャは `parseX(consumed, matched) -> (ok, consumed', matched')`。
  memo キーも両方を含む。

### 3.2 リテラル・token

- リテラル `'x'` = code point 列の接頭辞一致（境界なし、`CI` は ignoreCase）。
- 組込み token の受理範囲は `02-facts` §A の表。**`Extern` token** は D-008 の API。
- `REGEX`: Java 側は正則部分集合を DFA、それ以外は `java.util.regex.Pattern` の `lookingAt` に委譲
  （依存ゼロを保てる）。Rust 側は正則部分集合のみ、非正則は生成時エラー（明示）。ゼロ幅一致は失敗。

### 3.3 反復の進捗規則

`Occurs.java:31-72` の再現: 各周回で (a) terminator があれば照合、成功なら停止 (b) 子を照合、失敗/停止なら停止
(c) count++、**位置が進んでいなければ停止（その周回は数える）** (d) count ≥ max なら停止。
終了後 `min ≤ count ≤ max` なら成功。→ `EMPTY{2,2}` は失敗、`EMPTY{1,2}` は成功、空 span の mapped node が生じる。

### 3.4 選択

- 通常 `|`: 宣言順・最初の成功（PEG）。
- `@predictiveChoice`: FIRST で不可能と**証明**した候補だけ除外。候補なし／候補全失敗なら**完全な選択を再試行**
  （`PredictiveChoiceInterface.java:34`）。診断は元の Choice と同値。
- `@longestChoice`: 全候補を共通開始状態から試行、最大消費、同長は宣言順。
- 生成器は通常の `|` にも予測を適用してよいが、**候補除外は保守的証明付き**で、除外後の失敗診断が元と同値であること。

### 3.5 trivia（空白/コメント）

rule-trivia 契約（`docs/rule-trivia.md`）。有効設定は rule `@whitespace` > rule `@interleave` > grammar。
挿入位置は既存生成 chain と同じ（先頭・各要素後・helper 境界。`ParserGenerator.java:495`）。
IR に **挿入計画**（どの ExprId 境界で、どの policy で）を持たせる。global `@comment` は
`line` キーの存在で `CPPComment` を有効化（値は使われない）+ block open/close。`rawSettings + effectiveDelimiterSet + insertionPlan` を IR に載せる。
skip cache は policy とカーソルをキーにした局所最適化であり保証ではない。

### 3.6 capture と AST 構築

- 認識中は **capture の出現（siteId, span, 完了順）と構築レシピ**を保持し、CST オブジェクト木は作らない。
- 採用された結果に対してだけ typed AST を構築し、失敗し得る変換（数値、`@typeof`）を適用。
  変換失敗は「構文成功・mapping 失敗」として区別して報告（既存契約）。
- cardinality（scalar/optional/list、同名 nested は内側→外側）は front で解析済み。
- 結果 API: `AST + nodeSpans + captureOccurrences + lexicalOccurrences + scopeEvents + diagnostics`。
  `lexicalOccurrences`（未 capture のリテラル・token 出現、RuleId/ExprId 付き）は **オプション出力**
  （LSP・self-host 用）。self-host は `AnnotatedElement` の量指定子を復元するのにこれを使う
  か、フロントエンド専用の意味変換を持つ（M2 で決める）。
- span は code point。内部は Java UTF-16 / Rust byte で、入力ごとに変換表を一度作る。

### 3.7 memoization

- キー `(ExprId, consumed, matched, mode, stateVersion)`。値は終了カーソル・構築レシピ・局所診断。
- 既定は **memo あり**（純粋・決定的・進捗保証済みの領域）。省略は「再訪不能を証明できた単位」だけ
  （単一実行経路、互いに排他の候補、巻戻し不能な単調前進区間）。判定不能な SCC は全体 memo。
- `Extern` token（保証なし）とそこへ到達可能な祖先は memo 対象外（既存 fail-closed と同じ）。
  `@memoSafeToken` は alias 単位の利用者保証（成功時も再現可能）として IR に保持。
- scope 効果は memo hit 時に **効果の再生**（enter/leave/declare/reference を同順で）。
- 保守的な「全 memo」経路を参照実装として残し、省略版と意味論・実行回数を比較（M5）。

### 3.8 scope（@scopeTree / @declares / @backref）

- 状態: scope スタック、宣言（全履歴 + 現在 lookup）、参照、診断。
- checkpoint は **undo ログ**（enter/leave/declare/reference/diagnostic/clear の逆操作）。
  append-only 部分は長さ、lookup 上書きは旧値を記録。
- 既存契約: 開始で enter、成功で leave、同 rule が宣言も行うなら leave 後に親へ登録、
  trim（U+0000〜U+0020）、offset は code point、宣言後に参照、未定義は WARNING で構文成功、
  scope なし `@backref` は同一 rule 内テキスト比較 + ERROR 診断（Java）。
- stateVersion: parse 結果に影響する変更（enter/leave/declare/clear）だけ epoch を進める（#269 の教訓）。

### 3.9 診断

- 契約対象: 最遠失敗位置（consumed 基準か matched 基準かは corpus `diagnostics.json` で確認）、
  expected hints（リテラル/token の表示文字列、`ERROR('msg')`）、deepest rule。
- 成功した rule 内の失敗も寄与、負の先読み内は抑止、memo hit 時は局所診断を再生し呼出し stack を付け替え。
- 生成コードは成功経路の診断コストを抑える（局所 accumulator を rule 開始時にだけ作る）。

### 3.10 @recovery

`SYNC` / `AUTO` / `SKIP`（`ParserRuleEmitter.java:500`、`SyncPointRecoveryParser.java:102`）:
AUTO は FOLLOW（空なら `;`）、SYNC は同期文字列自体も消費・未検出なら失敗、SKIP は別の前進規則。
IR: `mode, resolvedSyncPatterns, consumeSync, noSyncAction, progressRule, applicationSites`。
wrapper は rule 参照解決で選ばれる（`@root` へ一律適用しない）。回復領域の typed AST への投影は
独立契約（既存はエラーマーカー Token）。第一段は corpus に有る範囲。

### 3.11 演算子（@leftAssoc / @rightAssoc / @precedence）と左再帰

- 正規形は既存契約どおり（左: `left + op[] + right[]` リスト、右: `Base Op Self | Base`）。
  優先順位は参照階層で決まり、数値は検証と metadata。
- **保守的生成**: 段ごとの関数（既存と同じ構造）。
- **最適化生成**（M5）: precedence climbing + `lift` で段のノードを復元（astra 疑似コード、
  `docs/decisions/D-004.md`）。trivia・副作用・recovery のない塔だけ。差分比較で採用。
- 直接左再帰 `E ::= E op T | T`: ubnfc はループへ書き換える（D-010、既存は警告のみ）。
  間接左再帰は生成時エラー（既存 Rust と同じ）。

## 4. IR 契約（v1 で追加した項目）

`ir/schema/grammar-ir.v1.json`（JSON Schema）で固定。v0 §3 に加えて:

- **ExprId**: 全 Expr ノードに安定 ID（文法ソース位置由来）。capture site / trivia 挿入 / memo / 診断 が参照。
- **Expr ごとの解析**: `mayStart`（文字クラス + EOF + ゼロ消費成功条件）、`minConsume`、接頭辞 trie、
  `cursorTransition`（consumed/matched のどちらを動かすか）、`nullable`。負の先読みの FIRST は補集合にしない。
  `Extern` は「不明＝全候補」。
- **trivia**: `rawSettings`、`effectiveDelimiterSet`、`insertionPlan[ExprId 境界 → policy]`。
- **capture**: `siteId`、完了順、cardinality、変換規則（text/number/node/mixed）、span 規則、
  同名 nested の順序、`@skip`。
- **和型**: 透明な和型（sealed interface）と具象型の区別、`@mapping(Node.Item)` の入れ子。
- **`@enum`**: rule 名、literal→variant、宣言順、生成名（`GrammarValidator.java:982`、`ASTGenerator.java:307`）。
- **`@commonField`**: 所属 interface、対象 variants、field 型・cardinality（生成は抽象メソッド。`ASTGenerator.java:270`）。
- **`@eval`**: `kind, strategy, params`。自動種別 `binary_arithmetic / variable_ref / conditional / passthrough / literal`、
  manual・未知は抽象 callback、和型への自動指定は拒否（`EvaluatorGenerator.java:31,101`）。
- **`@catalog`**: `context` と対象 rule / capture site。capture 不在は検証エラー。
- **`@import`**: `sourceId, importEdges, alias, resolvedGrammarId, qualifiedSymbolId`。既存はルール/token を
  prefix して統合（`UBNFMapper.java:127`）→ ubnfc も統合後の単一シンボル表を IR に持ち、`GrammarUnit` は出所 metadata。
- **`@typeof`**: `siteId, ownCapture, referencedCapture, phase=mapping, comparisonKind`（実行時 class 一致、
  不一致は mapping 例外、構文失敗にしない。`MapperRuleEmitter.java:824`）。
- **`@memoSafeToken`**: 原宣言形、保証の出所、成功/失敗の安全性。
- **`@recovery`**: §3.10。
- **scope**: enter/leave/declare/use の順序、mode metadata（lexical/dynamic）。
- **診断**: 統合・抑止規則、表示文字列。
- **memo 計画**: 成功/失敗の可否、必要キー、再生情報、省略の証明。
- **直列化契約**（self-host byte 一致のため）: map はキー順、ID 採番規則、文字 escape、パス正規化、
  `analysisVersion`。選択肢・capture・同長 longest の**順序は意味を持つので並べ替えない**。

## 5. 生成コードの性能技法（適用条件付き）

| 手法 | 適用 |
|---|---|
| 文字クラス表 | ASCII `classBits[128]`、識別子/数字/空白の専用ループ。非 ASCII は token ごとの集合へ |
| リテラル trie / 複数文字予測 | 共通接頭辞を一度だけ比較し候補ビット集合。試行順は文法順 |
| LL(k) 局所予測 | 固定小 k の接頭辞から判別コード。不明候補は残す。文法全体を LL(k) に制限しない |
| ワード単位/SIMD 走査 | 長い空白・行コメント・引用文字列・`UNTIL` の候補探索。**標準経路はスカラー**、SIMD は環境別追加経路（JDK Vector API は incubator、Rust `std::simd` は nightly） |
| テーブル駆動 | 正則 token は DFA。大きな選択は分類→候補表、小さい選択は直接分岐 |
| 要求時スキャン共有 | `(tokenId, triviaPolicyId, position)` の局所 cache。全入力の事前 token 化はしない |
| ゼロコピー span / 遅延変換 | capture は範囲で保持、採用結果だけ文字列化/数値化 |
| 構築レシピ / arena | 失敗枝の record/Box 生成を避ける。memo が参照する領域を rollback で再利用しない |
| 呼出し削減 | 透明 alias のインライン化、末尾再帰のループ化、右結合の明示スタック化。rule 境界（capture・診断）は残す |
| 診断の遅延 | 成功経路のコストを減らし失敗時だけ詳細化（成功時の診断契約と Extern 再実行可否を先に固定） |

固定しない事実: `Result` vs `Option`、`char[]` vs `charAt`、record vs SoA → M5 で計測。
直接 threaded VM は採らない（間接ディスパッチより直接分岐 + 局所 DFA + インライン化を先に比較）。

## 6. 結果 API

Java:
```java
ParseResult r = XParser.parse(CharSequence src, ParseOptions opts);
r.ok(); r.consumed(); r.matched();           // code point
r.ast();                                    // sealed interface + record（採用結果にのみ構築）
r.nodeSpans(); r.captures(); r.lexical();   // 出現表（lexical は opts で有効化）
r.scope(); r.diagnostics();
```
Rust: 同等の `ParseResult { ok, consumed, matched, ast: Option<Ast>, spans, captures, lexical, scope, diagnostics }`。
Extern token は Java `interface TokenScanner { ScanResult scan(Input in, int consumed, int matched); }` /
Rust `trait TokenScanner`。`ScanResult { consumedEnd, matchedEnd, valueSpan, diagnostics }`。
純粋 / 状態参照 / 効果あり を宣言（memo 可否に使う）。

## 7. マイルストーン（astra 案を採用。完了条件は全互換で不変）

| M | 内容 | 完了条件 |
|---|---|---|
| M0 | 互換対象の固定: corpus 5 系統 loader、仕様/oracle/実装の矛盾一覧（正常受理 / 意図した拒否 / mapping 失敗 の分類）、UBNF 構文モジュール | loader 全件読める、矛盾一覧が `docs/compat-matrix.md` に有る、24+P4 が GrammarAst に落ちる |
| M1 | **縦断**を Java/Rust 両方で: 接頭辞リテラル・二重カーソル・trivia・空 capture・数値 mapping 失敗・scope rollback・longest・memo（全 memo 版）。ベンチ併設（認識のみ / AST 付き / 診断+scope 付き） | primitives / lexical / nested-captures / rule-trivia / scope-annotations の該当ケース緑 |
| M2 | front を全構文へ拡大 + 意味論 IR + 解析。**生成 Rust frontend → GrammarAst の self-host 経路**を成立 | 24+P4 の IR、self-host 経路が動く |
| M3 | 両バックエンドを機能単位で拡張: enum / typeof / eval / import / catalog / recovery / rightAssoc / longest / predictive | 全 corpus 緑、24+P4 compile |
| M4 | self-host 検証: 手書き / 生成 frontend を全対象文法 + 不正文法で比較、IR byte 一致、固定点 | 一致 |
| M5 | 最適化生成（予測・memo 省略・climbing・trie・遅延診断）を保守的生成と差分比較。P4 でベンチ（長い正常入力・不完全入力・深い共通接頭辞） | 性能報告（達成/未達を明記） |

## 8. 争点の状態

| ID | 状態 | 決定 |
|---|---|---|
| D-001 FIRST 表現 | 決定 | 文字クラス + EOF + ゼロ消費条件 + 接頭辞 trie。Extern は不明。recovery の FOLLOW は文字列集合で別持ち |
| D-002 キーワード境界 | 決定 | 境界なし（接頭辞一致）。仕様ガイドの記述は実装と矛盾しており実装を正とする |
| D-003 CST | 決定 | 標準出力から外す。capture/lexical 出現表 + scope + 診断を結果に含める |
| D-004 climbing | 決定 | M5 で条件付き採用（trivia 境界計画、差分比較） |
| D-005 memo 判定 | 決定 | 既定 memo あり、省略は証明ベース、SCC 単位 |
| D-006 Java AST | 決定 | 公開は record 維持、位置は別表、内部 arena は計測後 |
| D-007 span | 決定 | 内部 UTF-16/byte、公開 code point、入力ごとの変換表 |
| D-008 Extern | 決定 | interface/trait + ScanResult、純粋性宣言 |
| D-009 IR の太さ | 決定 | 意味論は front、最適化計画は分離、バックエンドは再解釈しない |
| D-010 左再帰 | 決定 | 直接左再帰は書き換え（**明示的拡張**。既存は StackOverflow）、間接はエラー。対象形は D-016 |
| D-011 診断契約の範囲 | 決定 | offset は診断の主位置（syntax: Java は frame 内 max(C,M) の最遠、Rust は fail_at 最遠 / trailing_input: 採用 prefix 末尾）。言語別 oracle。詳細 `04-cursor-and-diagnostics-contract.md` §2 |
| D-012 self-host の量指定子復元 | 決定 | フロント専用変換（構造付き occurrence → GrammarAst）。`04` §3 |
| D-013 カーソル状態の契約 | 決定 | `(C, M, mode, invert, R)`、遷移表は `04` §1。生成時特殊化 or 状態引数。memo キーに含める |
| D-014 診断の互換範囲 | 決定 | 主 offset / farthest / 言語別 hints / deepest を分離。未検証部分は独立 oracle を追加 |
| D-015 suffix 併存の意味 | 決定 | 互換段階は既存の優先採用（postfix → bounded → separated の 1 つ） |
| D-016 左再帰書換えの対象形 | 未着手 | M2 で定義（nullable suffix、代替順、capture/trivia/scope/recovery の投影）。未対応形は明示拒否 |
