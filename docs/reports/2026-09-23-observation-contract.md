# 観測契約の統一（2026-09-23）

差分 fuzzing 第 1 回（`docs/reports/2026-09-23-differential-fuzzing.md`）が出した 12 の根本原因のうち、
**「どちらかの backend が間違っている」ではなく「契約が決まっていない」型**の 6 件を閉じる。
決定は `docs/design/03-design-v1.md` §8 の **D-024 〜 D-029**、wire の正本は
`docs/harness-driver-protocol.md` の「観測契約の統一（2026-09-23）」節。

対象: C-1 / B-1 / A-5 / A-6 / A-8 / A-9。A-1（recovery 受理差）と D（front の validate/ir 差）は
別ブランチ `fix/fuzz-front-and-recovery` の担当で、本稿では触れない。

## 1. 所見ごとの結論

### C-1 `allConsumed == (consumed == 入力長)` が成り立たない — 決定 **D-024**

規範の所在: `docs/harness-driver-protocol.md` の「`allConsumed`: 構文受理かつ入力全体を消費したか」。
これは **「消費し切ったが構文は失敗」を排除できていない**うえ、`consumed` は
**別の解析（prefix 解析）のカーソル**なので、そもそも導出関係が無い。両 backend が同じ振る舞いで
355 件ずつ違反していたのは実装差ではなく文書の不足である（fuzz レポート §4 C-1 と同じ結論）。

決定: `allConsumed` は**全入力解析が構文成功したか**。どの観測がどちらの解析に属するかを
protocol に表で明記し、応答に **`fullConsumed` / `fullMatched`**（全入力解析の二カーソル）を追加した。
不変条件は `allConsumed == (consumed == 入力長)` を捨て、
`allConsumed ⇒ fullConsumed == 入力長` と `fullConsumed <= fullMatched <= 入力長` に置き換えた。

### B-1 driver の wire 形式 — 決定 **D-025**

| 項目 | 前 | 後 |
|---|---|---|
| 回復領域 | Java `recoveries` / Rust `recoveredRegions`、要素も `captureOccurrences` と `captureOccurrenceIds` | 両者 `recoveries`、要素は `{ruleId, mode, span, syncPattern, skippedSpan, syncSpan, captureOccurrences}` |
| `scope` | Java は 6 キー / Rust は `{mode, events}` だけ | 両者 `{depth, mode, events, declarations, references, diagnostics}` |
| `catalogs` | Rust のみ | 両者（`{rule, context, captures}`） |
| `scope.mode` | Java=観測されたイベントの mode / Rust=宣言された mode | **宣言された mode**（Rust の解釈） |

`syncPattern` / `skippedSpan` は `span` / `syncSpan` / `mode` から導けるので Rust は driver 内で組み立て、
backend（`templates/finish.rs` の `Recovery`）には触っていない。定義は Java 側の実装
（`templates/internal/Session.java:630-632`）と同じ。

`scope.mode` を「宣言」に寄せたのは、観測 mode が `events[].mode` から復元できて情報量が無く、
入力ごとに変わるため静的 metadata として使えないから。`catalogs` は corpus の Catalog 系統
（`crates/ubnfc-harness/src/corpus/operations.rs` の `CatalogExpectation`）が要求する観測で、
Java が返していなかったために **Java backend では catalog 系統が「未観測」で通っていた**。
これを返すようにしたことで、その系統が Java でも実際に検証されるようになった。

### A-5 成功した解析に Java だけ投機的 `syntax` 診断を残す — 決定 **D-026**

規範の所在: D-023（`03-design-v1.md` §8 / §3.9）。二段構えの解析の定義は
「fast mode が落とす観測は**成功した解析の `hints`（投機的失敗）だけ**」であり、
成功経路内の失敗は `hints` の領分だと既に決まっている。Rust はそのとおり `hints` に入れており、
Java driver だけが合成 `syntax` 診断を `diagnostics` に 1 件足していた。

決定: 全入力解析が構文成功した応答は `syntax` / `trailing_input` 診断を持たない。
`Driver.java` の合成診断に `result.ok()` の条件を足した。recovery / scope / mapping / resource 由来の
診断は成功時も残る。harness の Oracle 経路が `allConsumed==true` のとき同じ診断を捨てていたので
corpus の期待値は変わらない。

### A-6 `farthestOffset` / `farthestExpected` — 決定 **D-027**

規範を当たった結果、**蓄積規則に非対称は無かった**:

| 出所 | 規則 |
|---|---|
| 旧実装 `unlaxer-common/.../context/ParseContext.java:1042-1064,1349-1351` | 候補位置は `max(maxConsumed, maxMatched)`。グローバルに単調更新。前進したら `expectedAtFarthestFailure` を捨て、同位置は union。`Not` が成功したときだけ子の失敗を巻き戻す |
| 新 Java `ubnfc-java/runtime-template/.../Diagnostics.java:50-70` | 同じ（`pos > farthest` で expected を捨てる。公開は global で rule ごとにリセットしない） |
| 新 Rust `crates/ubnfc-rust/templates/diag.rs:553-590,665-701` | 同じ（frame の `open()` が前進で log を truncate、`leave()` が farthest-wins で親へ畳み込み） |

したがって **D-014 の共有契約を維持し、言語別 oracle へ降格しない**。fuzz が見つけた
`farthestOffset` の食い違い（seed34、入力 `"*"`、Java=1 / Rust=0）は、Java 側が位置 1 まで到達して
位置 0 の候補を捨てたのに対し Rust が位置 0 から進んでいないという**到達位置そのものの差**で、
その原因は `@recovery(AUTO)` の適用差＝ **A-1**。farthest の記録側の欠陥ではないので、
この signature は A-1 の担当ブランチに帰属させ、既知として残す。

ただし `farthestExpected` のうち**文法に現れない合成 parser の表示名**
（`Repeat` / `OneOrMore` / `Bounded` / `Optional` / `__CaptureSite` / `<Rule>…Parser`）は、
旧実装では `ParseContext.java:1113-1143` の「子を 2 段 BFS して空のときだけクラス単純名」という
フォールバックに由来する。helper の作り方は言語ごとに違う（Java は `Session.repeat` の既定ラベル
`"Repeat"`、Rust は `src/diagnostics.rs` の Plan が旧クラス名を再現）ので一致し得ない。
これを D-014 の「言語別 hints」に含めると決めた。**corpus は Java 採取 oracle を両 backend で
厳密比較したまま**（現状 Fail 0）で、緩めたのは oracle の無いランダム文法を比べる fuzzer だけ。
副作用として、fuzzer は `...Parser` で終わる token 表示名（Extern scanner のクラス名由来など）も
比較から落とす。corpus 側は落とさないので、この範囲の退行は corpus が拾う。

### A-8 根が空一致したときの `mappingError` / `value` — 決定 **D-028**

- **`mappingError`**: Rust runtime は根が AST 値を作らないと
  `entry must produce one AST value` を mapping 診断にしていた。`@mapping` の無い entry には
  作るべき値が無いので、これは失敗ではない（Java は `ast=null` / `mappingError=null` を返す）。
  `templates/finish.rs` を `nodes.is_empty() ⇒ Ok(None)` に直した。値が 2 つ以上のときだけ失敗。
- **例外クラス名**: Rust driver が全 mapping 診断に `java.lang.IllegalArgumentException: ` を
  前置していたのをやめた。ただし `@typeof` 違反の文字列だけは
  `corpus/oracle/typeof.json` が固定した契約値なので、**生成器**（`crates/ubnfc-rust/src/mapping.rs`）が
  互換の文字列を組み立てる。oracle JSON は変更していない。
- **`value` の有無**: protocol の中核は「観測済みの `null` と未観測のキー欠落を区別する」で、
  harness の TokenValue / RuleValue は `value` の**有無**で全入力受理を決めている
  （`docs/harness-driver-protocol.md` の比較表）。したがって「評価していない」はキー欠落が正。
  Rust driver を Java（`Driver.java:115`）と同じ規則に揃えた。

### A-9 scope イベント — 決定 **D-029**

- `events[].name` は `declare` / `reference` では対象シンボル名、`enter` / `leave` / `clear` では
  **その scope を持つ規則名**。この名前は IR の `semantics.scope.effects[].events[].name` に既にあり
  （例: `ir/fixtures/…tinyexpression-p4…json` の `{"action":"enter","name":"Formula"}`）、
  Java は使っていて Rust が捨てていた。Rust の内部 `Effect` に規則名を持たせて埋めた。
- 未解決シンボル警告の差は、`@backref` の対象が空文字だったときに Rust だけが
  参照イベントと `未定義のシンボル: ''` 警告を出していたもの。Java は capture 由来の経路でだけ
  「trim 後に空なら記録しない」という規則を持っていた（`Session.java:223`）。
  これを**両言語・全経路**へ広げた（Rust は `effect_shared` と生成コード、Java は site 無しの経路）。
  seed92 の `action` / `offset` の差はこの余分なイベントで `order` がずれていただけで、同時に消える。

## 2. 変更したもの

| ファイル | 変更 |
|---|---|
| `docs/design/03-design-v1.md` §8 | D-024 〜 D-029 を追加 |
| `docs/harness-driver-protocol.md` | 「観測契約の統一（2026-09-23、D-024 〜 D-029）」節を追加（二つの解析と観測の所属、`recoveries` / `scope` / `catalogs` / `diagnostics` / `mappingError` / `value` の正本） |
| `ubnfc-java/src/main/resources/templates/driver/Driver.java` | `fullConsumed` / `fullMatched` / `catalogs` を追加、成功時の合成 `syntax` 診断を抑止、`scope.mode` を宣言値に |
| `ubnfc-java/src/main/java/org/ubnfc/java/emit/JavaEmitter.java` | driver に `__CATALOGS__` / `__SCOPE_MODES__` を埋める |
| `ubnfc-java/src/main/resources/templates/internal/Session.java` | site を持たない `declare` / `use` の空名を記録しない |
| `crates/ubnfc-rust/templates/driver.rs` | `recoveries` へ改名・要素を統一、`scope` に `depth`/`declarations`/`references`/`diagnostics`、`fullConsumed`/`fullMatched`、`mappingError` の前置をやめ、`value` の有無を Java と同規則に |
| `crates/ubnfc-rust/templates/finish.rs` | 空の AST 値を mapping 失敗にしない、scope イベントの `name` に規則名 |
| `crates/ubnfc-rust/templates/runtime.rs` | `Effect` に規則名、空の記号名を記録しない |
| `crates/ubnfc-rust/src/expressions.rs` | enter/leave/clear を `effect_scope_at` で出す、capture 由来の空名を飛ばす |
| `crates/ubnfc-rust/src/mapping.rs` | `@typeof` 違反の互換文字列を生成側で組み立てる |
| `crates/ubnfc-fuzz/src/observe.rs` | 不変条件を D-024 の形へ、`farthestExpected` の合成名を D-027 に従って除く |
| `crates/ubnfc-fuzz/tests/campaign.rs` | 既知 signature の一覧を更新 |
| `crates/ubnfc-selfhost/src/generated/**`、`examples/p4-rust/src/generated/**` | テンプレート変更に伴う再生成（手編集なし） |

corpus の期待値 JSON・oracle JSON・`corpus/oracle/divergences.json` は変更していない。
harness の比較規則（`crates/ubnfc-harness/**`）も変更していない。

## 3. fuzzer の前後

所見の元になった seed を `--inputs 24 --shrink-builds 0` で回し直した（最小化はしない）。
「前」は `docs/reports/2026-09-23-differential-fuzzing.md` §4 と
`crates/ubnfc-fuzz/findings/<slug>/README.md` に記録された signature。

| seed | 前（差分クラスの signature） | 後 |
|---|---|---|
| 1 | `diff:/catalogs` `diff:/recoveries` `diff:/scope/declarations` `diff:/scope/depth` `diff:/scope/diagnostics` `diff:/scope/references` `diff:/diagnostics/*` `diff:/mappingError/present` `diff:/value` `diff:/captures/*` | **`diff:/captures/*` のみ**（A-7） |
| 2 | `diff:/diagnostics/*/farthestExpected/*` `diff:/catalogs` ほか | **`diff:/captures/*` のみ** |
| 3 | `java:allconsumed-consistency` `rust:allconsumed-consistency` `diff:/scope/mode/*` `diff:/catalogs` `diff:/recoveries` `diff:/scope/*` | **0 件** |
| 9 | `diff:/scope/events/*/name` `diff:/catalogs` `diff:/recoveries` `diff:/scope/*` `diff:/ast/fields/head/*` `diff:/captures/*` | `diff:/ast/fields/head/*`（A-4）、`diff:/captures/*`（A-7） |
| 34 | `diff:/diagnostics/*/farthestOffset` `diff:/diagnostics/*/farthestExpected/*` `diff:/catalogs` | `diff:/diagnostics/*/farthestOffset` `diff:/diagnostics/*/farthestExpected/*` |
| 92 | `diff:/scope/events/*` `diff:/scope/events/*/action` `diff:/scope/events/*/offset` `diff:/catalogs` `diff:/recoveries` `diff:/scope/*` | **`diff:/captures/*` のみ** |

固定 seed campaign（`cargo test -p ubnfc-fuzz`、seed 4001..4004、30 比較）は
`diff:/captures/*` と `diff:/diagnostics/*/farthestExpected/*` の 2 つだけになり、
既知一覧（`crates/ubnfc-fuzz/tests/campaign.rs`）を 11 件から 2 件へ減らして緑。

消えたのは C-1（両 backend の `allconsumed-consistency` 710 件）、B-1 の 7 クラス全部、
A-5 の 9 クラス、A-8 の 2 クラス、A-9 の 4 クラス、A-6 のうち候補集合の差 1 クラス。

### 残った `farthestExpected` の差（新しい切り分け）

seed 34 / seed 4001 に残るのは、D-027 で言語別とした**合成 parser の表示名ではない**候補の差で、
fuzzing 第 1 回の時点では「farthestExpected の中身が別物」と一括りだったものが、
合成名を外したことで 2 つの原因に切り分けられた:

1. **到達位置そのものの差**（`farthestOffset` と連動）= A-1 の症状。
2. **静的な失敗ヒント計画の差**: `@leftAssoc` 規則の失敗で Java は到達していない演算子リテラル
   （`'*'`）を候補に載せ、Rust は載せない。Extern token の表示名（`SingleQuoted`）でも同じ向きの差が出る。
   旧実装の `expectedHintCandidatesFor`（`ParseContext.java:1113-1143`）は静的な 2 段 BFS なので
   「到達していない候補が載る」こと自体は正しく、**何段まで載せるか**が両者で違う。
   corpus の Java 採取 oracle には該当形が無く、どちらが旧実装と一致するかを固定できないので、
   これは新しい所見として残し、signature を既知一覧に置いた。

なお A-6 の調査中に、Java が `token X = ANY` の失敗候補へ UBNF の綴り `ANY` を出していたのを
直した（`JavaEmitter.token` が診断 plan の label を無視して固定文字列を使っていた）。
旧実装の採取値は `WildCardCharacterParser`（`corpus/oracle/{digit,empty,evolution-v3}.json`）で、
`ANY` は corpus のどこにも現れない。seed 2 の差はこれで消えた。

## 4. 検証

| 項目 | 結果 |
|---|---|
| `cargo test --workspace --offline` | 緑（68 target、Fail 0。`ubnfc-rust/tests/driver.rs` の 2 件は新契約に合わせて期待値を更新） |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | 警告なし |
| `cargo fmt --all -- --check` | 差分なし |
| `cd ubnfc-java && mvn -o -q test` | 緑 |
| selfhost 固定点 | 緑（`crates/ubnfc-selfhost/tests/regenerate.rs` の 2 件） |
| `examples/p4-rust` golden | 緑（`tests/fixtures.rs` / `tests/regenerate.rs`） |
| `examples/p4-java` golden | **base commit 09a81da の時点で既に stale**（下記） |
| corpus Java `run-all` | **Fail 0**。Pass 1310 / 期待拒否 28 / Known 28 / 未観測 59（round7 は 1306 / 28 / 28 / 63） |
| corpus Rust `run-all` | **Fail 0**。Pass 1292 / 期待拒否 30 / Known 28 / 未観測 75（diag2 と完全一致） |
| 固定 seed campaign | 緑 |

report は `docs/reports/2026-09-23-{java,rust}-corpus-contract.{md,json}`。

系統別の比較（`docs/reports/2026-09-22-java-corpus-round7.md` /
`docs/reports/2026-09-23-rust-corpus-diag2.md` との差）:

- Java `catalog`: Pass 0 → **2**、未観測 2 → **0**。`catalogs` を返すようにしたので、
  この系統が Java でも実際に検証されるようになった（D-025）。
- Java `oracle`: Pass 330 → **332**、未観測 36 → **34**。同じ理由（`oracle/catalog.json`）。
- 他の系統・Rust 側は 1 件も変わらない。Known 28 件の内訳も同じ。

### `examples/p4-java` の golden について

`P4Test.generatedSourcesAreByteIdentical` は**本ブランチの分岐元（09a81da）で既に赤**である。
その commit を detached worktree に取り出して `scripts/generate.sh --check` を回すと、
本稿が触っていない `TinyExpressionP4Parser.java`（2,457 行）・`api/Capture.java`・
`api/ParseResult.java`・`internal/Recipe.java`・`rt/Diagnostics.java` を含む 7 ファイルが差分になる。
`ubnfc-java/**` は並行して `perf/java-m5` が触っている領域なので、ここで `--write` すると
その作業を巻き込む。**本ブランチでは再生成しない**。commit 済み生成物の追従は
`ubnfc-java` を所有するブランチで行う（`examples/p4-rust` 側は本ブランチで再生成済み・緑）。
