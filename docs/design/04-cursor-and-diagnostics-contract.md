# 二重カーソル・診断・self-host の契約（astra 第 2 ラウンド、2026-09-22）

根拠は `unlaxer-common/src/main/java/org/unlaxer/` 配下の ファイル:行。

## 1. カーソル状態（D-013）

状態は `(C=consumed, M=matched, mode ∈ {consumed, matchOnly}, invertMatch, R=resetMatchedWithConsumed)`。
`R` は transaction frame ごとのフラグで、**呼出し元 frame** が変更する（Chain は `begin` 前に `R=false`。`ChainInterface.java:18`）。
既存 enum 名は `TokenKind.consumed` / `TokenKind.matchOnly`（`TokenKind.java:7`）。`L`=照合長、`M₀ = R ? C : M`。

| 操作 | 遷移 |
|---|---|
| `getPosition(kind)` | consumed → C、それ以外 → M（反復の進捗判定・memo キーはこれ） |
| transaction begin | 子へ C を複製、子の M は M₀、R も継承 |
| `consume(L)` | `(C,M) → (C+L, C+L)`。**L=0 でも M を C に戻す**（失敗リテラルも `consume(0)`。`AbstractTokenParser.java:40`） |
| `matchOnly(L)` | `(C,M) → (C, M+L)` |
| commit | 子の C/M を親へコピー（R はコピーしない） |
| rollback | 子 frame を捨て親の C/M へ |
| `MatchOnly(X)` 成功 | 子を matchOnly で照合、`(C, M₀+L)`。意味的副作用は復元（`MatchOnly.java:28`） |
| `MatchOnly(X)` 失敗 | rollback、`(C,M)` |
| `Not(X)` | 子成功 → 自身失敗・rollback。子失敗 → 自身成功 `(C,M₀)`、子の失敗診断を破棄（`Not.java:22`） |
| `EMPTY` | `Optional(MatchOnly(ANY))`。非 EOF で `(C, M₀+1)` になり得る（`EmptyParser.java:24`） |
| `EOF` | `Occurs(ANY,0,0)`。選択カーソルから ANY を試し失敗なら成功 |
| `UNTIL` 終端 | `MatchOnly(Word)` が **M から**照合。外側が consumed mode なら終端の照合位置を commit（`WildCardStringTerminatorParser.java:63`、setter は 1 引数 `parse` のみ） |

**M は最遠到達位置ではない**（消費・transaction 開始で後退する）。

反復（`Occurs.java:36`）: 「位置」は `getPosition(tokenKind)`。consumed mode では M だけ進んでも停止し、その周回は数える。
上限は周回後に確認する。terminator は外側 mode を渡さず `parse(context)` で呼び、成功時は外側が consumed なら commit、他は rollback して停止。

実測（既存ビルド済みクラス）:

| 入力 / 構成 | 結果 |
|---|---|
| `ab` / `EMPTY` | 成功 C=0 M=1 |
| `ab` / `EMPTY{2,2}` | 失敗 C=0 M=0 |
| `ab` / `EMPTY{1,2}` | 成功 C=0 M=1 |
| `ab` / `EMPTY{0,0}` | **失敗** C=0 M=0 |
| `a#` / `LOOKAHEAD('a')` → `UNTIL('#')` | 成功 **C=0 M=2** |

→ 生成関数 `parseX(C, M)` は mode / invert / R を **生成時特殊化**（同じ Expr を consumed 版と matchOnly 版で別関数に）
するか状態引数で渡す。memo キーは `(ExprId, C, M, mode, invert, R, stateVersion)`。

## 2. 診断（D-011 / D-014）

`javaOffset` / `rustOffset` は診断の**主位置 `offset`**であり最遠失敗とは限らない。

| 種別 | Java | Rust |
|---|---|---|
| `syntax` | 失敗 frame 内の `max(maxConsumed, maxMatched)` を候補とする最遠位置（`ParseContext.java:1042,1350`、`MapperGenerator.java:196`） | `fail_at` の最遠失敗位置（`lib.rs:1014,1466`） |
| `trailing_input` | 採用 prefix の消費末尾 | 同左 |
| 単位 | code point | byte → code point 変換 |

`neg(1e+)` の Java=7 / Rust=5 は単位差ではなく、Java は不完全指数の試行失敗を残し Rust の数値スキャナは記録しない
（`NumberParser.java:55`、`lib.rs:2244`）。**言語別 oracle として両方守る**。

corpus の対応テストが検証するのは kind・offset・expected 非空・末尾入力時の expected 一致のみ
（`RustConformanceTest.java:758`）。expected 完全一致・deepest rule は未検証 → D-014: 主 offset / farthest / 言語別 hints / deepest を分け、
未検証部分は ubnfc が独立 oracle（既存実装の実測）を追加する。`Not` は「成功時の子失敗破棄」と「自身の失敗」を分ける。

## 3. self-host（D-012 / D-015）

既存 mapper（`UBNFMapper.java:740,789,992`）が読むもの: 基底 `AtomicElement` 部分木、**現在の要素階層だけ**の postfix / bounded / separated、
bounded のソース文字列、separated 内の別 `AtomicElement` 部分木。文法は 3 種 suffix の併存を許すが mapper は
`postfix → bounded → separated` の `if/else` で **1 つだけ採用**（`ubnf.ubnf:237`）。

決定: **フロント専用変換**。採用枝・親 occurrence・必要な非終端完了情報を構造付き occurrence として保持し、そこから
`GrammarAst` を構築する。手書き解析へのフォールバックはしない。suffix 併存の意味は互換段階では既存の優先採用を維持（D-015）。

## 4. 左再帰（D-010 / D-016）

既存: `W-LEFT-RECURSION` 警告のうえ本体をそのまま生成（`GrammarValidator.java:285`、`ParserRuleEmitter.java:659`）。
memo に seed-growing は無く（`PackratMemoTable.java:72`）、実行時は StackOverflowError に至る。
→ 直接左再帰のループ化は**明示的な言語拡張**。D-016: 対象形を限定（nullable suffix、代替順、capture・trivia・scope・recovery の投影を定義）、
未対応形は明示拒否。

## 5. IR スキーマ（`ir/schema/grammar-ir.v1.schema.json`、`ir/examples/tinycalc.ir.json`）

- `semantics` / `analysis` / `plan` を分離。trivia 挿入と左再帰 lowering の記録は意味論側。
- `analysisVersion = "ubnfc-analysis.v1-draft1"`。
- ExprId = 論理 sourceId + 元ソースの code point 区間 + 構文上の role（絶対パス・乱数・走査通番に依存しない）。
- 直列化: キー順・UTF-8・escape・改行・パス正規化・意味を持つ配列順を `description` に規定。
- TinyCalc 例: 7 rules、40 Expr、11 capture、27 trivia 境界、3 trie。recovery FOLLOW 不要は `null`（空集合と区別）。
- スキーマ適合は解析値の正しさを証明しない（検証責務は harness）。
