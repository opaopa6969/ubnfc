# `@recovery` を通った解析の観測差（差分 fuzzing、issue #41 / D-069）

差分 fuzzing（[`2026-09-23-differential-fuzzing.md`](2026-09-23-differential-fuzzing.md)）の
v2 seed 帯 **100276** が、CI の固定 campaign が踏んでいない差分クラスを 2 つ出した。

```
diff:/diagnostics/*/expected/*
diff:/recoveries/*/captureOccurrences/*
```

同じ seed で既知の `diff:/diagnostics/*/farthestExpected/*`（`tests/campaign.rs` の `KNOWN`）も出ていた。
issue #34 の作業で、この 3 つは v1 の綴り（`NEGATION('ab\n')`）でもそのまま出ることが確かめられており、
D-063（code point クラス）とは無関係の**既存の backend 間不一致**である。

## 1. 最小再現

fuzzer の縮小は `crates/ubnfc-fuzz/findings/diff-*-seed100276/` にある。そこから手で更に削ると、
根本原因ごとに次の 2 つになる（どちらも `@ubnf: 2` は不要で、v1 の文法で再現する）。

### 1.1 回復診断の候補 — `diff:/diagnostics/*/expected/*`

```ubnf
grammar Fuzz {
  @recovery(SKIP) M0 ::= 'a';
  @root Root ::= M0 'zz'?;
}
```

入力 `"+"`（両 backend とも受理・全消費。`M0` が `SKIP` で 1 code point 読み飛ばす）:

| 観測 | Java | Rust（修正前） |
|---|---|---|
| `diagnostics[0].kind` | `recovery` | `recovery` |
| `diagnostics[0].expected` | `["'a'", "'zz'"]` | `["a"]` |
| `diagnostics[0].farthestExpected` | `["'a'", "'zz'"]` | `["a"]` |

**綴りと集合の両方**が違う。

### 1.2 `captureOccurrences` — `diff:/recoveries/*/captureOccurrences/*`

```ubnf
grammar Fuzz {
  @recovery(SKIP) M0 ::= 'a';
  @mapping(W, params=[head]) W ::= M0 @head;
  @root @mapping(Top, params=[head]) Root ::= W @head;
}
```

入力 `"+"`（`captures` は両者とも `head`[0,1] が 2 件で一致）:

| 観測 | Java | Rust（修正前） |
|---|---|---|
| `recoveries[0].captureOccurrences` | `[0]` | `[0, 1]` |

`W` の `@mapping` を外す（透過にする）と Java も `[0, 1]` になる。

## 2. どちらが契約どおりか

**どちらも Java が正しく、Rust が契約から外れていた**（決定は D-069）。

### 2.1 候補の語彙と範囲（D-014 / D-027）

Rust の診断は 2 本立てで、`fail_at` の主診断 DAG（`templates/diag.rs` の `Diagnostics`）と、
旧実装の表示名を再現する公開 farthest（同 `DisplayDiagnostics`）を別々に持つ。
`syntax` 診断は `farthestExpected` を**公開側**から作り直していたが、`expected` と、
**`recovery` 診断の両方**は DAG 側の内部 label（`literal()` に渡す生の語 `a`）をそのまま出していた。

D-014 が言語別としたのは `syntax` 診断の `expected`（hints の表記）だけで、
`farthestOffset` / `farthestExpected` は D-027 で**共有契約のまま**である。
したがって `kind: "recovery"` の `farthestExpected` を内部語彙で出すのは契約違反で、
`expected` も同じ診断の同じ位置の候補なので同じ語彙で出す。

範囲は「回復した規則の解析」。Java の `Session.recover` は
`diagnostics.enterExpression()` → 本体 → `leaveRule()` で**規則の frame の**最遠候補集合を取る。
1.1 で Java に `'zz'` が入り Rust に入らないのは位置のずれではなく、次の節の静的ヒントの差である。

### 2.2 規則の同期文字列（旧実装の観測）

Java の `Session.rule` は、規則の本体が**自分の開始位置で**失敗したとき、
`@recovery` の同期文字列を候補として記録する（`recoveryHints`）。
これは旧実装の観測で、`corpus/oracle/recovery-auto.json` の `recovery-auto/2`（空入力）が
`expected = ["';'", "'ok'", "RootParser", "__CaptureSite"]` として固定している
（`';'` は入力位置 0 では一度も試されない。`Stmt` の FOLLOW から来る）。

Rust の静的失敗ヒント計画（`src/diagnostics.rs`）は規則の候補だけを持ち、同期文字列を持たない。
corpus の 3 つの `recovery-*` fixture は同期文字列が規則本体のリテラルと一致するため通っていた。
これが `KNOWN` の `diff:/diagnostics/*/farthestExpected/*` の一部でもある。

### 2.3 `captureOccurrences`（D-025）

protocol は「回復領域に属する capture 出現の添字列」としか書いていなかった **(d)**。
Java の実装は「値の直下に回復 marker `Recipe` を持つ出現」で、
`Session.rule` が `@mapping` のある規則（および `@skip`）で `nodes` を自分の `Recipe` 1 個に
置き換えるため、**AST ノードを作る規則をまたぐと外側の capture は数えない**。
Rust は event 木の祖先 capture をすべて数えていた。

Java の規則を正本にした。「その capture の値が回復領域そのものか」という判定で、
`@mapping` の有無で結果が変わるのも一貫している（値がノードなら回復領域ではない）。

## 3. 直したもの（Rust backend だけ）

| ファイル | 変更 |
|---|---|
| `crates/ubnfc-rust/templates/diag.rs` | `DisplayDiagnostics::local_summary()` / `local_far()`（最も内側 frame だけを畳み込む） |
| `crates/ubnfc-rust/templates/runtime.rs` | `Event::Recovery` に候補表の添字を持たせ、`recover()` が回復時に frame 局所の公開候補を控える |
| `crates/ubnfc-rust/templates/finish.rs` | 回復診断の `expected` / `farthestExpected` / `farthestOffset` を公開候補から作る。`captureOccurrences` は node 境界の深さが一致する回復だけに付ける |
| `crates/ubnfc-rust/src/emit.rs` | 回復つき規則の本体を `display_enter` / `display_leave` で囲む。開始位置で失敗したときの同期文字列を候補として出す |
| `crates/ubnfc-rust/src/mapping.rs` | `RULE_NODE`（自分の AST ノードを作る／捨てる規則） |

`ubnfc-java` と front は 1 行も変えていない。IR schema・IR golden も変わらない。
`crates/ubnfc-selfhost/src/generated` と `examples/p4-rust/src/generated` は
同じ IR から再生成した（`rt/diag.rs` と `runtime.rs` だけが動く。`parser.rs` は
どちらの文法にも `@recovery` が無いので 1 byte も変わらない）。

## 4. 消えた差分クラス

seed 100276 は **`KNOWN` の signature も含めて 0 件**になった（`--inputs 12`）。

| signature | この seed での扱い |
|---|---|
| `diff:/diagnostics/*/expected/*` | 消えた（§2.1） |
| `diff:/recoveries/*/captureOccurrences/*` | 消えた（§2.3） |
| `diff:/diagnostics/*/farthestExpected/*` | 消えた（§2.2。**クラスとしては `KNOWN` に残す** — `@leftAssoc` の静的失敗ヒント計画と Extern token の表示名という別の原因が seed 34 / 4001 に残っているため） |

## 5. 回帰試験

- `crates/ubnfc-fuzz/tests/recovery_diagnostics.rs` — 文法と入力を固定し、`recoveries` と
  `kind: "recovery"` の診断だけを突き合わせる（回復領域を値に持つ field の AST 投影は
  `recovery-projection` の未決事項なので巻き込まない）。3 つの文法で、同期文字列の候補・
  ノード境界あり・ノード境界なしを 1 つずつ踏む。
- `crates/ubnfc-fuzz/tests/campaign.rs` の `f_v2_recovery_diagnostics_agree_across_backends` —
  seed 100276 を固定 campaign に入れ、**`KNOWN` を使わずに** 0 件を要求する。
- `crates/ubnfc-fuzz/findings/diff-*-seed100276/` — 縮小した 3 件を経緯として残した。
