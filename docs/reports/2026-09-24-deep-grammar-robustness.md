# 深い文法・巡回の多い文法での異常終了と計算量（issue #2）

> 2026-09-24。[`2026-09-23-crosscheck-fixes.md`](2026-09-23-crosscheck-fixes.md) §2「結果 3 — 直していない 2 件」で
> 再現手順だけ残していた 2 件を直す。変えたのは `crates/ubnfc-front/src/analysis/**` と
> `crates/ubnfc-front/src/validate/operators.rs`。IR・生成物・self-host 固定点・corpus は不変。
> 診断が変わるのは `W-LEFT-RECURSION` の**件数**だけで、文言・severity・category・出る順序は同じ。

| # | 欠陥 | 症状 | 直し方 |
|---|---|---|---|
| 1 | analysis の規則参照グラフ走査が再帰 | 16,000 段の規則連鎖で **exit 134**（stack overflow）。release でも落ちる | 4 か所すべて明示スタック化（上限は設けない） |
| 2 | `validate::operators::find_cycles` が階乗時間 | 互いを左端参照する 10 規則（609 byte）で **120 秒超** | Tarjan の SCC + 成分内 BFS。規則ごとに 1 件 |

---

## 1. 再帰だったのはどの走査か

`ubnfc validate --grammar chain.ubnf` を段ごとに切って確かめた（`parse` → `resolve` →
`validate` → `lower` → `analyze`）。**落ちるのは `analyze` の中だけ**で、`validate` の
`find_cycles` は同じ 16,000 段でも（フレームが小さいので）溢れなかった。

`analyze` の中で rule 本体をまたいで降りる再帰は 4 つあり、**規則の入れ子の深さではなく
「規則参照の連鎖長」がそのまま再帰の深さになる**。並び順に潰した。

| 走査 | 役割 | 連鎖 16,000 段での挙動 |
|---|---|---|
| `analysis/graph.rs` の `scc`（Tarjan） | 規則グラフ / 式依存グラフの SCC。`analyze` 冒頭・`checks::left_recursion`・`follow::compute` の 3 か所から呼ばれる | ここで最初に落ちる |
| `analysis/planning.rs` の `effects` | 式から到達する scope 効果・Extern・recovery の要約。`cx.deps` は RuleRef の本体を含む | `scc` を直した後、次に落ちる |
| `analysis/reachability.rs` の `lead` | 順序付き選択の「左端で必ず呼ぶ rule」。`A ::= B 'x'` のように**左端が rule 参照**の連鎖でだけ降りる | 連鎖の左端が rule のとき落ちる |
| `analysis/reachability.rs` の `literal_prefix` | 候補の必須リテラル接頭辞。Seq の先頭リテラルを繋ぎながら rule 本体へ降りる | `S ::= 'a' \| B;` + `R_i ::= 'a' R_{i+1}` で落ちる |

`analysis` の他の再帰（`checks::left_recursion` の `collect`、`checks::cardinality` の `count`、
`reachability::same`）は **rule 参照で止まる**（`children()` は RuleRef の本体を含まない）ので、
深さは構文層の `DEPTH_LIMIT = 128` に縛られる。`facts::solve` と `follow::compute` の本体は
もともと反復（SCC ごとの固定点）で、`planning::memo` の省略証明も明示スタックだった。

### 上限ではなく反復化を選んだ理由

`analysis` が計算するのは nullable / mayStart / minConsume の固定点と cursor 遷移で、
**どれも「全規則を見る」性質のもの**。深さで打ち切ると答えが変わる（あるいは打ち切り値を
memo して答えが走査順に依存する）。4 つとも DAG/グラフ上の memo 付き走査なので、
再帰を明示スタックに置き換えれば**結果は 1 bit も変わらない**。実際、子を辿る順序・
memo に書く時点・循環の打切り判定の時点をすべて再帰版と揃えてあるので、IR golden
（`ir/fixtures`）・`analysis_golden.rs`・`choice_reachability.rs` は無変更で通る。

加えて `E-*` 診断で拒否する案は、`pipeline::validate_file` が analysis の失敗を
`if let Ok(..)` で捨てる（診断を足せないだけ）ので、`ubnfc validate` からは
**何も言わずに終わる**ことになり、上限の存在が利用者に見えない。

### 実在文法の規則参照の深さ（上限を設けないことの裏付け）

24 fixture + `ubnf.ubnf` + P4 を解決して、規則参照グラフを SCC で縮約した最長経路と
最大 SCC を測った（`all-tokens.ubnf` と `namespace.ubnf` は既知の resolve エラーで対象外）。

| 文法 | 規則数 | 縮約後の最大深さ | 最大 SCC |
|---|---|---|---|
| P4 (`tinyexpression-p4.ubnf`) | 124 | **8** | **77** |
| `unlaxer-dsl/grammar/ubnf.ubnf` | 59 | 6 | 9 |
| `mixed-values/Shared.ubnf`、`evolution/*` 他 | 4–7 | 4 | 3 |
| その他 18 件 | 1–7 | 1–3 | 1–3 |

再帰の深さの上界は「縮約後の深さ + 最大 SCC の規則数」なので、実在文法では **100 段未満**。
落ちたのは 16,000 段で、その間には 2 桁以上の開きがある。上限を置くとしたら実在文法の
100 倍でも足りる値になり、置く意味が薄い。反復化してあるので上限そのものが要らない。

### 実測

`ubnfc validate --grammar chain.ubnf`（release、`R_i ::= 'a' R_{i+1}` の連鎖）:

| 連鎖長 | 修正前 | 修正後 |
|---|---|---|
| 8,000 | exit 0 | exit 0（1.6 秒） |
| 16,000 | **exit 134（stack overflow）** | exit 0（4–9 秒） |
| 64,000 | exit 134 | stack overflow しないが 5 分でも終わらない（§4） |

issue の再現（16,000 段）は数秒で診断 0 件・exit 0 になる。64,000 段は**異常終了はしない**が、
深さとは別の二乗コスト（§4）で現実的な時間に収まらない。ここは今回の対象外で、後退でもない。

---

## 2. `find_cycles` の階乗時間

### 何が起きていたか

`validate::operators::left_recursion` は左端呼出しグラフの巡回を報告する。旧 `find_cycles` は
visited 集合を持たず、**開始点から届く全ての単純経路**を数え上げていた。互いを左端参照する
N 規則（完全有向グラフ）ではその数が階乗的に増える。

| N | 文法サイズ | 修正前の時間 | 修正前の `W-LEFT-RECURSION` 件数 | 修正後の時間 | 修正後の件数 |
|---|---|---|---|---|---|
| 7 | 327 byte | 0.07 秒 | 13,692 | 0.004 秒 | 7 |
| 8 | 411 byte | 0.76 秒 | 109,592 | 0.007 秒 | 8 |
| 9 | 505 byte | 9.2 秒 | 986,400 | 0.02 秒 | 9 |
| 10 | 609 byte | **> 60 秒（打切り）** | — | 0.005 秒 | 10 |

時間だけでなく**出力も病的**だった。1 KB 未満の文法に 986,400 件の警告が出るのは
診断として使いものにならない（旧実装は 1 つの基本巡回をその回転の数だけ、さらに
到達経路の数だけ数え上げていた）。

### 置き換え

1. 左端呼出しグラフの **Tarjan の SCC**（明示スタック。O(V+E)）。
2. 巡回に載っている規則ごとに、**その規則を通る最短の巡回**を成分内の BFS で 1 本だけ求める。
   辺は文法に書かれた順に見るので決定的。

出力は「**左再帰の規則ごとに 1 件**」。`W-LEFT-RECURSION` の文言・hint・severity・category と
（`BTreeSet<Vec<SymbolId>>` 由来の）出力順は変えていない。

計算量は O(V+E) + Σ_SCC |S|·(|S|+E_S)。実在文法の最大 SCC は 77（P4）なので線形とみなせる。
単一の巨大 SCC では成分サイズの二乗になるが、**旧実装も同じ入力で同じ二乗の出力を出していた**
ので後退ではない（階乗になるのは成分内に基本巡回が多数ある場合で、そこが今回の改善点）。

### 診断の互換（goldens は変えていない）

Java bootstrap から採った `crates/ubnfc-front/tests/fixtures/validation-java.json` の
`W-LEFT-RECURSION` は 5 ケースあり、**全件そのまま通る**。

| ケース | 期待 | 新実装 |
|---|---|---|
| `Expr ::= Expr \| 'n'` | `Expr -> Expr` 1 件 | 同じ |
| `A ::= B; B ::= A` | `A -> B -> A` と `B -> A -> B` の 2 件 | 同じ |
| `[ 'a' ] Start` / `{ 'a' } Start` | `Start -> Start` 1 件 | 同じ |

`tests/resolve_validate_fixtures.rs` の `story/code/calc-leftrec.ubnf`（2 件）と
24 fixture 全体の合計（`W-LEFT-RECURSION` = 2）も不変。`ir/fixtures/errors/*.txt` の
3 件は左再帰と無関係で、IR golden も動いていない。

つまり **Java 参照実装と観測が一致する範囲は完全に保ったまま、参照実装も一緒に病的だった
領域（同じ成分に基本巡回が複数ある場合）でだけ出力を 1 規則 1 件に畳んだ**。
契約の選択なので [`../design/03-design-v1.md`](../design/03-design-v1.md) §8 に **D-052** として記録した。

---

## 3. 生成パーサ側の深い「入力」

契約は design v1 §6 の追記にある: 「**再帰深度は `ParseOptions.maxDepth`（既定 10,000）で
制限し、超過は `syntax` 失敗として報告する。JVM/スタック限界に達した場合も失敗へ変換する**」。
両 backend に実装がある（Rust: `crates/ubnfc-rust/templates/runtime.rs` の `enter_rule` +
スタックアドレス差 256 KiB の保険、Java: `templates/internal/Session.java` の `DepthLimit` +
`StackOverflowError` の捕捉）。P4 で確認した。

| 入れ子 | Rust（`examples/p4-rust`） | Java（`examples/p4-java`、`-Xss1m` / `-Xss16m`） |
|---|---|---|
| 1 / 20 | 受理 | 受理（`-Xss16m`。`-Xss1m` は 16 段まで） |
| 50 | `syntax` / `maximum parse depth exceeded` | 同じ |
| 1,000 | `syntax` / `maximum parse depth exceeded`、exit 0 | `maximum parse depth exceeded (JVM stack limit; maxDepth=10000)`、exit 0 |
| 10,000 | 同上 | 同上 |

**どちらも異常終了しない。**P4 は入れ子 1 段あたり 200 段超の rule 呼出しを積むので、
`maxDepth = 10,000` には入れ子 50 段ほどで当たる（文法の段数に比例する既定なので、
P4 で「20 段までしか受理しない」のは契約どおりの帰結）。

回帰テストは既にある（Rust: `crates/ubnfc-rust/tests/support/review_assertions.rs` の
`recursion_aborts_optional_and_recovers_next_call` が 2,000 段 + `max_depth = 8`、
Java: `ReviewRegressionTest` が 2,000 段を `-Xss256k` / `-Xss1m` の別 JVM で確認）。
P4 そのものの固定は Rust 側に無かったので
`examples/p4-rust/tests/fixtures.rs` に
`deeply_nested_input_reports_a_depth_diagnostic_instead_of_crashing` を足した
（1 / 20 段は受理、50 / 1,000 / 10,000 段は `maximum parse depth exceeded`）。
Java 側の同じ観測は `examples/p4-java/scripts/depth.sh` と `evidence/depth.log` にある
（`ubnfc-java/**` は別 issue が編集中なので触っていない）。**D-\* 行は足していない**:
契約は既にあり、今回はそれに合致することを確かめただけ。

---

## 4. 直していない計算量（後退ではないが残る）

異常終了はしないが、深い文法で二乗以上になる箇所が残る。いずれも**今回の変更で
悪くなったものではなく**、実在文法（最大 124 規則）では問題にならない。

| 箇所 | 計算量 | 実測 |
|---|---|---|
| `reachability::lead` の `LeadMap` | 連鎖長 n の左端連鎖で 1 つの map が O(n) 要素になり、合計 O(n²) 以上 | 左端連鎖 + 選択で 1,000 段 0.96 秒 / 2,000 段 5.8 秒 / 4,000 段 38 秒 |
| `planning::effects` | 式ごとに到達部分グラフを辿るので O(式数 × 到達式数) | scope 効果・recovery・Extern token が 1 つも無い文法では走査自体を飛ばす近道を入れた（結果は同じ）ので、連鎖 16,000 段は 4 秒で通る。Extern token が 1 つでもある深い文法では二乗のまま |
| その他（`resolve` / `lower` / `planning::memo` を含む全体） | 規則数に対して二乗以上 | 連鎖 8,000 段 1.6 秒 → 16,000 段 9 秒 → 64,000 段は 5 分でも未完。**異常終了はしない** |

`lead` は「順序付き選択の**先行候補が rule 参照**」のときだけ降りるので、深い左端連鎖と
順序付き選択が同時に要る。実在文法には無い形で、上限を置くと警告が減る方向にしか
効かないが、打切り値を memo する現在の設計では走査順に結果が依存するため、
上限の導入は別途設計が要る。issue に残す候補として記録だけしておく。

---

## 5. 検証

- `cargo test --workspace --offline` 緑（`ubnfc-java` を先に `mvn -o -q -DskipTests package`）。
- `cargo clippy --workspace --offline --all-targets -- -D warnings` 緑、`cargo fmt --all --check` 緑。
- corpus 無退行: **Java Pass 1310 / Fail 0**、**Rust Pass 1292 / Fail 0**。
- self-host 固定点（`crates/ubnfc-selfhost`）不変、`cargo test -p ubnfc-fuzz` 緑。
- IR golden（`ir/fixtures/**`）差分なし。

### 追加した回帰（`crates/ubnfc-front/tests/deep_grammar.rs`）

`compile()` を **512 KiB の thread stack** で走らせる。明示スタック化してあれば深さは
heap にしか乗らないので通り、再帰のままだとプロセスごと SIGABRT になる
（実際に旧実装へ戻して確認した: 深さ 3 件は stack overflow、巡回の 1 件は期待値不一致）。

| テスト | 何を再現するか |
|---|---|
| `deep_rule_reference_chain_does_not_overflow_the_stack` | `graph::scc` と `planning::effects`（4,000 段） |
| `deep_leftmost_chain_does_not_overflow_the_stack` | `reachability::lead`（600 段） |
| `deep_literal_prefix_chain_does_not_overflow_the_stack` | `reachability::literal_prefix`（4,000 段） |
| `mutually_left_recursive_rules_are_reported_in_linear_time` | 互いを左端参照する 12 規則（1 KB 未満）が 500 ms 未満、警告は 12 件 |
| `left_recursive_cycles_are_reported_once_per_rule` | 巡回の報告内容（自己ループ / 相互 / 3 規則） |
