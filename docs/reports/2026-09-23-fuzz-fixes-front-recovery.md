# 差分 fuzzing の所見 D（front の契約違反）と A-1（`@recovery(AUTO)` の受理差）の修正（2026-09-23）

対象は `docs/reports/2026-09-23-differential-fuzzing.md` の 12 根本原因のうち 2 件。
どちらも front / 意味論側で、**片方は「validate は通るのに ir が落ちる」、もう片方は
「同じ文法・同じ入力で受理そのものが backend ごとに違う」**という、この fuzzer が
探せる中で最も重い部類の 2 つ。

| 所見 | 症状 | 修正前 | 修正後 |
|---|---|---|---|
| D | `ubnfc validate` を通った文法が `ubnfc ir` で落ちる | seed 1..320 の生成文法で **28 件 discard**（`internal` 19 / `analysis` 9） | **0 件**（同じ 320 seed） |
| A-1 | `@recovery(AUTO)` の適用範囲が両 backend で違い、受理・カーソル・capture・AST が変わる | seed 269 / 272 で 13 差分クラス | 受理系クラスは **0**（残りは既知の wire 差のみ） |

corpus は両 backend とも Fail 0 のまま、family 表は基準報告と**完全一致**（§4）。

## 1. 所見 D — front が自分の契約に落ちる **(c)**

### 1.1 根本原因

cardinality を決める場所が 3 つあり、互いに別の規則を使っていた。

| 場所 | 役割 | 修正前の規則 |
|---|---|---|
| `lower::captures::occurrences` | `Capture.cardinality`（構文上の出現回数） | Bounded は常に `List` |
| `lower::captures::fields` → recipe / AST field | 意味値の個数 | Bounded は常に `List`、capture site より内側の量指定子も text を List 化 |
| `analysis::checks::cardinality` | 出現回数の推論（`Count{min,max}`） | `max <= 1` は `Scalar` / `Optional` |
| `pipeline::cardinality::Values::captured` | recipe と capture の突き合わせ | `Repeat` は min/max を見ず常に `List` |

そのため

- `'a'{1} @v` / `'a'{0} @v` / `'a'{0,1} @v` / `( 'a' @v ){1}` → `CaptureCardinality { declared: List, inferred: Scalar|Optional }`（9 件）
- `[ N? ] @v`（`N` は mapped）→ `X?` が `ExprNode::Repeat{min:0,max:1}` に落ち、突き合わせ側だけ `List` になる（19 件のうち大半）
- `{ 'a' 'b'{1} }? @v` → capture site の内側の反復が text 値まで List 化する
- `@rightAssoc` の operand が未 mapping helper（`M1 ::= { M0? }`）経由のとき、正規形が固定した
  `left: Scalar` と、意味値の数え上げ `List` が衝突する

### 1.2 正本

`unlaxer-parser/docs/nested-capture-migration.md`（#177）:

> これは単に文法に同名が何回書かれているかではなく、**同じ解析で何個の値が共存し得るか**の契約である。
> …量指定子へのcaptureは従来どおり各要素を対象とする。反復全体の文字列をさらに1件追加するものではない。

同じ表の `'a' @value | 'b' @value` → scalar、`'a' @value | 'b'` → optional も「共存し得る個数」で説明できる。
演算子の operand は `03-design-v1.md` §3.11 の正規形（左: `left + op[] + right[]`）が個数を決める。
旧 Java 実装（`MapperTypeResolver.inferLegacyType` / `SemanticCardinality`）は bounded を常に `List`
と型付けするが、`SemanticCardinality.fieldType` は assoc rule では legacy 型をそのまま返す
（operand の内部多重度を数えない）ので、operand 側は正規形が正本という読みと一致する。

### 1.3 修正

- `Cardinality::quantified(min, max)` を新設（`ir/types.rs`）: 上限 2 以上だけ `List`、`{1}` は `Scalar`、
  `{0}` / `{0,1}` は `Optional`。`lower::captures` の 4 か所と `pipeline::cardinality` がこれを使う。
- `lower::captures::captured` を `captured_at(site: bool)` に分け、**capture が直接貼られた原子の量指定子は
  出現回数としてそのまま数え、内側の量指定子は意味値だけを増やす**（text だけの領域は一致範囲全体で 1 値。
  既にあった `semantic_wrap` と同じ規則を capture 側にも適用した）。
- `pipeline::checks`: `AssocLists` recipe の `left` / `right` は正規形が cardinality を決めるので、
  意味値の数え上げでは検査しない（型と site 対応の検査はそのまま）。

「拒否すべき文法だった」ではない。上のどれも `docs/nested-capture-migration.md` の表で
cardinality が一意に決まるので、**`validate` を通った文法は IR まで通る**方に寄せた（D-025）。

### 1.4 証拠

```sh
cargo run -q -p ubnfc-fuzz --offline -- --start-seed 1 --dump 320 --front-only 1 | grep -c '^-> discard'
# 修正前: 28   修正後: 0
```

最小例（`crates/ubnfc-fuzz/findings/front-*/`）は `ubnfc ir` が exit 0 になる。
`ubnfc-fuzz` の固定 seed campaign（4001..4004）は build 3/4 → **4/4**、`discards` は空になり、
テストに `assert_eq!(summary.discarded, 0)` を追加した。

## 2. 所見 A-1 — `@recovery(AUTO)` で受理が変わる **(a)**

### 2.1 根本原因

3 段の食い違いが重なっていた。

1. **front**: `@recovery(auto)` を小文字のまま比較していたので、`AUTO` と書いた文法だけが
   `mode=AUTO` になり、`auto` と書いた文法は `mode=SYNC`（同期文字列 `;`）に落ちていた。
2. **front**: `consumeSync` を `mode != SKIP`、つまり AUTO でも `true` にしていた。
3. **Java backend**: 1 の legacy encoding を補正するため、`JavaEmitter.recoveryCall` が注釈から
   mode を推定し直し、FOLLOW を**自前で再計算**して `consume=false` にしていた。Rust backend は
   IR の FOLLOW を使い、`mode == "SYNC"` のときだけ消費する実装だった。

結果、`@recovery(AUTO)`（大文字）では Java が同期文字列まで消費し、Rust は手前で止まる。
`M0 ::= T0? 'c';` / 入力 `" ;;"` で Java は全消費して AST を作り、Rust は `;` を残して拒否。
逆向き（`M0 ::= T0 T0? 'c';` / 入力 `"c;"`）では Java だけが `;` を食い潰して解析全体が失敗した。

### 2.2 正本

- `03-design-v1.md` §8 の **D-017**: `corpus/oracle/README.md` の疑義のうち「**AUTO の過剰消費**」は
  **再現しない**（既存バグ扱い）。`corpus/oracle/divergences.json` の `recovery-auto/1` も
  「`bad;ok` を消費して後続 `ok;` を壊す AUTO を再現しない」で `follow-spec` 登録済み。
  AUTO の同期文字列は FOLLOW、つまり**外側の rule が次に読む字句**なので、消費すれば外側が壊れる。
- §3.10 の「SYNC は同期文字列自体も消費・未検出なら失敗」「SKIP は別の前進規則」はそのまま。
- 旧実装 `SyncPointRecoveryParser.java:102`（`skipLength = 位置 + syncToken.length()`）と
  `ParserRuleEmitter.java:500`（AUTO も同じ parser に FOLLOW を渡す）は**消費する**側だが、
  それこそが D-017 で再現しないと決めた挙動。SKIP だけは旧実装も同期文字列の手前で止まる。
- D-021 / D-018 `recovery-projection` の「`@root`・直接 entry にも適用する」は維持。

### 2.3 修正

front（`lower/annotations.rs`、`analysis/follow.rs`、`pipeline.rs`）:

- mode の綴りは大小無視。`@recovery(auto)` と `@recovery(AUTO)` は同じ IR になる。
- `consumeSync = (mode == SYNC)`。SKIP の `noSyncAction` は実挙動どおり `consumeToEof`。
- FOLLOW は AUTO だけでなく **SKIP でも解決**して IR に載せる（backend が自前で FOLLOW を
  計算する必要がなくなる）。`analysis` は FOLLOW そのものを返し、AUTO で FOLLOW が空のときの
  既定 `;` は pipeline（回復方針）が入れる。既定 `;` は FOLLOW ではないので `consumeSync = true`。

backend:

- Java `JavaEmitter.recoveryCall` から legacy mode 補正と自前 FOLLOW 計算
  （`followPatterns` / `collectFollow` / `firstPatterns`）を削除。IR の値をそのまま渡す。
- Rust `templates/runtime.rs#Session::recover` は `consume_sync` / `no_sync` を引数で受け取り、
  mode 文字列で消費を決めない。走査・SKIP の前進規則・未検出時の動作を Java `Session.recover` と
  同じ式に揃えた。
- 両 backend とも「AUTO で同期集合が空なら `;` を同期集合として消費」だけは残した（手書き IR 用。
  front は必ず埋めるので通常経路では効かない）。

契約は `docs/design/03-design-v1.md` §3.10 に表と 3 段の手順として書き、§8 に **D-024** を追加した。
`ubnfc-java/src/test/resources/ir/m3/recovery-{auto,skip}.ir.json` は front から採り直した
（mode が宣言どおりになり、`resolvedSyncPatterns` が FOLLOW の `ok` になる）。

### 2.4 証拠

```sh
cargo run -q -p ubnfc-fuzz --offline -- --start-seed 269 --seeds 1 --inputs 24 --jobs 1 --shrink-builds 0
cargo run -q -p ubnfc-fuzz --offline -- --start-seed 272 --seeds 1 --inputs 24 --jobs 1 --shrink-builds 0
```

| seed | 修正前に出ていた A-1 の signature | 修正後 |
|---|---|---|
| 269 | `/accepted` `/allConsumed` `/consumed` `/matched` `/parsedStatus` `/captures/*/span/*` `/captures/*/text` `/prefixCaptures/*` `/prefixCaptures/*/span/*` `/prefixCaptures/*/text` | **すべて消滅**（24 入力・受理 15 / 部分 1 / 不受理 8） |
| 272 | `/captures/*/name` `/prefixCaptures/*/binding` `/prefixCaptures/*/name` | **すべて消滅** |

残る signature は `diff:/catalogs` `diff:/recoveries` `diff:/scope/*`（B-1 の wire 形式）、
`diff:/diagnostics/*` 系（A-5 / A-6）、`diff:/ast` `diff:/captures/*`（seed 272 は A-3/A-4/A-7）、
`java|rust:allconsumed-consistency`（C-1）で、いずれも別の根本原因として報告済み。

回帰試験 `crates/ubnfc-fuzz/tests/campaign.rs::b_recovery_auto_keeps_acceptance_identical_across_backends`
を追加した（seed 269 を 24 入力で回し、受理系 signature が 1 件も出ないことを確認。9 秒）。

## 3. 生成物の再生成

front を変えたので次を再生成し、byte 一致を確認した。

- IR golden（`crates/ubnfc-front/tests/pipeline_golden.rs` の 24 ファイル）: **差分なし**
  （既存 fixture に capture つき `{n<=1}` も `@recovery` も無い）。
- `crates/ubnfc-selfhost/src/generated`・`examples/p4-rust/src/generated`: `runtime.rs` のみ更新
  （`Session::recover` の署名と本体）。self-host の固定点テストと p4 の `generate.sh --check` は緑。
- `ubnfc-java/src/test/resources/ir/m3/recovery-{auto,skip}.ir.json` を front から採り直し、
  同 `README.md` の「backend が mode を補正する」記述を消した。

## 4. corpus

```sh
cargo run -q --offline --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-23-java-corpus-fuzzfix1.md --json ...
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust \
  --report docs/reports/2026-09-23-rust-corpus-fuzzfix1.md --json ...
```

| backend | Pass | Fail | 既知 | 期待拒否 | 未観測 | 基準との family 表 |
|---|---:|---:|---:|---:|---:|---|
| Java | 1306 | **0** | 28 | 28 | 63 | `2026-09-22-java-corpus-round7.md` と完全一致 |
| Rust | 1292 | **0** | 30 | 28 | 75 | `2026-09-23-rust-corpus-diag2.md` と完全一致 |

`corpus/oracle/*.json`・`divergences.json` は変更していない。recovery 系 fixture
（`recovery-auto` は `@recovery(auto)` 小文字）は、front が宣言どおり AUTO を載せるようになっても
FOLLOW（`ok`）と非消費が Java の旧補正と同じなので、観測は 1 件も動かない。

## 5. やっていないこと

- A-2（入れ子 span の末尾 trivia、44 クラス）・A-3〜A-9・B-1・C-1 はそのまま。
- `@recovery` の**回復領域を typed AST へどう投影するか**は D-018 `recovery-projection` のままで、
  ここでは触っていない（§3.10 にもそう書いた）。
- `@recovery(banana)` のような未知 mode は今も黙って SYNC になる。綴りの大小は揃えたが、
  未知語の拒否（`E-*`）は別の変更として残す。
