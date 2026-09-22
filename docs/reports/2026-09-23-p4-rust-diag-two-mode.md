# 二段構えの解析: 成功経路の診断コストを 0 にする（2026-09-23）

[`2026-09-23-p4-rust-perf-m5-round4.md`](2026-09-23-p4-rust-perf-m5-round4.md)（round 4 末 fb0ed05:
complex 認識 0.157 / AST 0.212 ms、complex-x64 認識 11.1 / AST 15.5 ms）の続き。round 4 の上限見積りで、
complex.tiny（332 byte、完全成功）の **失敗記録が認識の 32%（約 51 µs）**、主診断 DAG が 14 µs、
event 記録が 28 µs だった。失敗記録も DAG も、成功して診断を返さない parse では捨てられる。
本 round はそれを **記録しない mode** に分け、必要なときだけ記録付きで解析し直す。

## 結論

- 成功する入力は **認識 −43〜−50%、AST −29〜−43%**（同一セッションの交互 A/B、中央値）。
  complex.tiny は **認識 0.202 → 0.113 ms、AST 0.253 → 0.180 ms**、
  complex-x64 は **認識 15.6 → 8.9 ms、AST 28.2 → 16.4 ms**。
- 代金は **拒否される入力が約 1.8 倍**（同じ入力を診断モードで解析し直す）。
- 実装は**実行時フラグ**。計測では `enabled` の分岐はほぼ無料で、残っていたのは**呼出し**のほうだった
  ので、guard だけを `#[inline(always)]` で展開して本体は `#[inline(never)]` に置いた。
  `const DIAG: bool` の単相化は、上限を別 binary で測って（complex 認識であと 8 µs / AST で 11 µs）
  生成 parser を 2 部にする価値が無いと判断し、**作っていない**。
- 観測は corpus・driver JSON・golden で不変。**fast mode が落とす唯一の観測は「成功した解析の `hints`」**で、
  これが要る利用者（生成 driver を含む）は `ParseOptions::diagnostics = true` を渡す。

## 設計

### mode と切り替え条件

`ParseOptions::diagnostics`（既定 `false`）を追加した。

- **fast mode（既定）**: 主診断 DAG（`rt::diag::Diagnostics`）も公開 farthest（D-014、`DisplayDiagnostics`）も
  記録しない。frame の出入り・label の記録・DAG の node 追加・snapshot の保存が全て消える。
  cursor・event arena（AST / 出現 / recovery）・scope 効果・memo は変わらない。
- **診断モード**: round 4 までと同じ振る舞い。
- 入口（`parse` / `parse_with_options` / `parse_with_scanner` / `parse_entry_with_options` / driver 用の
  `parse_entry_with_scanner`）は fast mode で 1 回解析し、結果が診断を要するなら
  **同じ入力を `diagnostics: true` で解析し直して、その結果をそのまま返す**（fast 側の観測を混ぜない）。

再解析の条件は `finish_checked` が返す（保守的に広く取る）:

| 条件 | 理由 |
|---|---|
| `!ok`（構文失敗・trailing・深さ超過・resource 失敗） | 診断そのものが結果 |
| `consumed != text.len()`（`require_eof` を問わず入力が残った） | prefix 解析でも farthest が要る |
| `diagnostics` が非空（recovery / mapping / resource） | 成功時も残る診断がある |
| `recoveries` が非空 | Recovery event の診断 handle が fast mode では空 |

条件に **入れなかった**もの: `scope_diagnostics`（未定義シンボル等の意味診断）と `declarations` /
`references` / `scope.events`。これらは診断機構ではなく scope store の効果で、fast mode でも
同じものが出る。入れると complex.tiny が毎回 2 回解析になり、目的が消える（後述の「落とし穴」）。

**両 mode の結果が等しいこと**は「決定的で副作用が無い」という主張に頼らず、差分テストで確認した（後述）。
`TokenScanner` は再解析で 2 回呼ばれるので、scanner が純粋（同じ入力・同じ state に同じ `ScanResult`）で
あることが前提になる。P4 の 3 Extern scanner と `RejectExtern` は状態を持たない。

### fast mode が落とす唯一の観測: 成功した解析の `hints`

`ParseResult::hints`（D-014 の「投機的失敗」）は **成功した解析でも非空**になる。これは fast mode では作れない
（主診断 DAG そのもの）。したがって:

- 成功時の hints が要る利用者は `diagnostics: true` を渡す。
- **生成 driver は hints を無条件に JSON へ出すので `diagnostics: true` を渡している**。
  driver JSON は byte 一致のまま（corpus も同じ）で、driver 自身は高速化しない。
  「driver も fast mode を先に走らせる」は、成功時も必ず再解析になる（= 常に 2 回解析する）ので採らなかった。

### `limits.diagnostics`（予算）の意味

予算は「記録した診断の件数」の上限なので、**fast mode では消費されず、成功した解析を予算超過で失敗にしない**
（`Diagnostics` は limit 0 で `push` を即返し、`exhausted` は立てない。`DisplayDiagnostics` は `work` を数えない）。
再解析した診断モードで予算を超えれば、これまで通り Resource 診断 1 件で全体失敗になる。
予算の契約を検証する `default_diagnostic_budget_accepts_every_fixture_and_small_budget_fails_as_resource` は
`diagnostics: true` で回すよう直した。

### 実装: 実行時フラグか const generic か

`const DIAG: bool` で `Session` を単相化すると、生成 parser の `impl Session<'_>`（P4 で 759 rule 関数・
約 20,000 行）が 2 部できる。round 4 で **rule 入口検査 `limit` を `#[inline]` にしただけで認識 +5 µs**
（i-cache）だったので、2 倍の単相化は同じ方向の悪化が予想される。そこで

- **採用（実行時フラグ）**: `Diagnostics` / `DisplayDiagnostics` に `enabled` を持たせる。
  - `Diagnostics` は **fast mode で `limit = 0`** にするので、`push` の最初の検査（`nodes.len() >= limit`）が
    そのまま早期復帰になり、**分岐は 1 つも増えない**。`enabled` は「limit 0 を予算超過と誤判定しない」ためだけに使う。
    `join` / `rule` は handle が `Diag::NONE` なので既存の早期復帰で抜ける。
  - `DisplayDiagnostics` は `enabled` を `enter` / `leave` / `discard` / `reach` / `fail` / `failures` /
    `merge` / `snapshot` / `summary` の先頭で見る（全て `#[inline]`）。frame stack の push / pop も止まる。
- **上限の測定**: 同じ木から `DisplayDiagnostics` の全 guard を定数 `true`（＝常に無効）に、
  `Diagnostics::fail` を `#[inline]` で `Diag::NONE` を返す形に潰した binary（`static`）を作って計測した。
  これは **単相化が到達できる最良値（しかも複製 1 部だけなので i-cache penalty なし）** で、
  実行時フラグとの差がそのまま「分岐の代金」になる。

#### 測ってわかったこと: 代金は分岐ではなく **呼出し**

complex 認識で 4 変種を交互に 13 回計測（`taskset -c 28`、load average 11〜14、中央値 / 最小 ms）:

| 変種 | 認識 中央値 (最小) | AST 中央値 (最小) | 読み取り |
|---|---:|---:|---|
| base（round 4 末 fb0ed05） | 0.184 (0.151) | 0.247 (0.206) | — |
| flag: `enabled` を各メソッド先頭で見る（`#[inline]`） | 0.099 (0.092) | 0.154 (0.139) | ここでも −46% / −38% |
| flag2: 上 + `Diagnostics::fail` の guard を `#[inline]` 化 | 0.094 (0.090) | 0.154 (0.136) | 主診断 DAG の呼出しを消す |
| flag3/4: 上 + display の 5 メソッドの **guard だけ** `#[inline(always)]`、本体は `#[inline(never)]` | 0.097 (0.088) | 0.142 (0.133) | 採用 |
| static: guard を定数で潰した上限（複製 1 部、診断モードも壊れる） | 0.088 (0.081) | 0.139 (0.122) | 単相化が到達できる最良値 |

`enabled` の**分岐**はほぼ無料で、残っていたのは **呼出し** のほうだった（display の `failures` / `reach` /
`enter` / `leave` / `fail` が 1 parse で約 16,000 回）。guard だけを展開すると呼出しが消え、
**単相化の上限（static）まで AST で 2%、認識で 10% の距離**に入る。

`const DIAG: bool` による単相化は**作らなかった**。上限（static）との差は complex 認識で 9 µs / AST で 3 µs
しかないのに対し、代償は生成 parser の `impl Session<'_>`（759 rule 関数・約 20,000 行）が 2 部になることで、
round 4 では rule 入口検査 1 つを `#[inline]` にしただけで認識 +5 µs の i-cache 悪化が出ている。
**上限を測ったうえで採らなかった**、というのが正確な状態（作って測ったわけではない）。

### memo・効果再生の同値

- memo キーは `(ExprId, C, M, mode, invert, R, stateVersion)` のままで、mode は入らない。
  保存値 `Saved` は診断 handle（`Diag`）と `DisplaySnapshot` を持つが、fast mode ではどちらも空の値が
  入るだけで、同じ session 内では一貫している。memo / trivia cache の buffer は `Session::new` が
  必ず `reset` するので、pool 経由で前の mode の内容が残ることはない。
- 効果（scope の enter/leave/declare/use）の保存と再生は診断に触れないので両 mode で同じ。
  `declarations` / `references` / `scope_diagnostics` / `scope.events` は fast mode でも同じものが出る。
- 唯一 mode が解析結果を変え得るのは **診断予算** で、これは上記のとおり fast mode では消費されない。

### 落とし穴（実測で見つけた）

最初は「scope の意味診断が出たら再解析する」も条件に入れていた。ところが complex.tiny は
`未定義のシンボル` 警告を 1 件（complex-x64 は 64 件）出すので、**成功しているのに毎回 2 回解析**になり、
fast mode が baseline より遅かった（complex 認識 0.160 → 0.295 ms、確保 353 → 682 回）。
scope の診断は診断機構ではなく scope store の効果で、fast mode でも同じものが得られるため条件から外した。
差分テストはこの条件を外した状態で **fast mode 1 回の結果と診断モードの結果を直接比較**するので、
弱めた分はテストが強くなっている。

## 正しさの根拠

- **差分テストを拡張した**（`examples/p4-rust/tests/fixtures.rs::predict_matches_reference_path_on_truncated_and_corrupted_inputs`）。
  16 fixture × 切り詰め・破壊 × memo on/off の各入力で、**fast mode（既定）と `diagnostics: true` の
  結果を直接比較**する:
  - 再解析が起きる結果（`!ok` / `diagnostics` 非空 / `recoveries` 非空）は **`hints` まで含めて完全一致**。
  - 完全成功の結果は **`hints` が空であること**と、**`hints` 以外の全フィールドが診断モードと一致すること**。
  既存の predict / factor の参照経路比較（約 11,000 件）はそのまま緑。
- `cargo test --workspace --offline` 全緑、`cargo clippy --workspace --all-targets --offline -- -D warnings` 無警告、
  `cargo fmt --all -- --check` 緑。
- selfhost 固定点（`crates/ubnfc-selfhost/tests/regenerate.rs`、`generated_frontend_reaches_the_ir_and_rust_fixed_point`）緑。
  `crates/ubnfc-selfhost/src/generated` と `examples/p4-rust/src/generated` は再生成して commit（byte 一致テスト緑）。
  selfhost 自身は fast mode で動く（成功する文法では再解析しない。失敗時は hints を使うので再解析される）。
- P4 golden（canonical AST）と handwritten-target の golden は byte 一致。
- corpus: `docs/reports/2026-09-23-rust-corpus-diag2.{md,json}`。
  **Pass 1292 / 期待拒否 Pass 30 / Known 28 / Fail 0 / 未観測 75**、系統別・文法別の表は
  `2026-09-23-rust-corpus-m5r4.md` と完全一致（差分は generator / front の SHA-256 と build 時間の 3 行だけ）。
  driver は `diagnostics: true` を渡すので JSON は byte 一致。
- `ubnfc-java`、corpus / oracle の期待値、`crates/ubnfc-front`、unlaxer-parser、tinyexpression は触っていない。

## A/B（`examples/p4-rust` bench、release、`taskset -c 28`、baseline と交互に 9 run）

中央値 / 括弧内は全 run の最小（ms/parse）。**共有ホストで load average 17〜24**（他エージェントの
Rust テストと Java ベンチが並走）。絶対値は round 4 の表より 2〜3 割悪く出るので、
**同一セッションの列どうし**で読むこと。`static` は単相化の上限（診断モードも壊れているので
complex-tail / invalid-syntax の行は参考外）。確保は GlobalAlloc の計数。

| 入力 | bytes | mode | base (fb0ed05) | 本 round (flag4) | static（上限） | 比 | allocs base → 本 round | bytes base → 本 round |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| complex | 332 | 認識 | 0.202 (0.164) | **0.113** (0.090) | 0.097 (0.082) | −44% | 353 → 330 | 36.2 KB → 32.0 KB |
| complex | 332 | AST | 0.253 (0.226) | **0.180** (0.142) | 0.152 (0.131) | −29% | 771 → 748 | 132 KB → 128 KB |
| complex-x4 | 1,293 | 認識 | 0.759 (0.638) | **0.432** (0.379) | 0.431 (0.339) | −43% | 1,264 → 1,241 | 68.4 KB → 64.2 KB |
| complex-x4 | 1,293 | AST | 1.045 (0.805) | **0.597** (0.518) | 0.718 (0.492) | −43% | 2,907 → 2,884 | 442 KB → 438 KB |
| complex-x16 | 5,179 | 認識 | 3.635 (2.654) | **1.832** (1.583) | 2.675 (1.644) | −50% | 4,890 → 4,867 | 204 KB → 199 KB |
| complex-x16 | 5,179 | AST | 4.438 (3.203) | **2.929** (2.139) | 4.559 (2.337) | −34% | 11,425 → 11,402 | 1.68 MB → 1.67 MB |
| complex-x64 | 20,923 | 認識 | 15.607 (12.116) | **8.854** (7.203) | 10.750 (6.155) | −43% | 19,369 → 19,346 | 749 KB → 745 KB |
| complex-x64 | 20,923 | AST | 28.165 (19.354) | **16.384** (12.263) | 16.005 (11.413) | −42% | 45,449 → 45,426 | 6.65 MB → 6.64 MB |
| **complex-tail（拒否）** | 333 | 認識 | 0.185 (0.162) | **0.344** (0.288) | 0.204 (0.164) | **+86%** | 323 → 622 | 34.3 KB → 64.2 KB |
| **complex-tail（拒否）** | 333 | AST | 0.204 (0.168) | **0.365** (0.305) | 0.201 (0.174) | **+79%** | 324 → 624 | 43.6 KB → 82.8 KB |
| invalid-syntax（拒否、4 byte） | 4 | 認識 | 0.012 (0.011) | 0.016 (0.014) | 0.006 (0.005) | +33% | 46 → 59 | — |

- 成功する入力は **認識 −43〜−50%、AST −29〜−43%**。最小値で読んでも同じ（complex 認識 0.164 → 0.090、
  AST 0.226 → 0.142）。確保回数は微減（診断 arena の伸長が消えた分）で、確保 byte は complex 認識で
  36.2 → 32.0 KB（display の `Registry` cache と log が要らなくなる）。
- **拒否される入力は約 1.8 倍**。これは「失敗を 2 回解析する」代金そのもので、
  complex-tail の 0.344 ms ≒ fast 1 回（0.113 相当）+ 診断モード 1 回（0.185 相当）+ 差分。
- `static`（単相化の上限）は最小値では常に本 round 以下だが、中央値は雑音で逆転する
  （x16 / x64）。差は complex 認識で 8 µs / AST で 11 µs（最小値で 8 / 11 µs）。

## 最終値と長期目標

| 入力 | bytes | 認識 ms | AST ms | µs/byte (AST) | 長期目標 | 判定 |
|---|---:|---:|---:|---:|---:|---|
| complex | 332 | 0.113 (最小 0.090) | 0.180 (最小 0.142) | 0.54 | **0.02 ms** | **未達**（7〜9 倍） |
| complex-x4 | 1,293 | 0.432 | 0.597 | 0.46 | — | — |
| complex-x16 | 5,179 | 1.832 | 2.929 | 0.57 | — | — |
| complex-x64 | 20,923 | 8.854 | 16.384 | 0.78 | ≤ 15 ms | 中央値は未達、最小 12.3 は下回る |

round 4 の目標（complex AST ≤ 0.2 ms、x64 AST ≤ 15 ms）は、この計測条件（load average 17〜24）でも
complex AST 0.180 ms で**達成**、x64 AST は中央値 16.4 / 最小 12.3 で**境界**。
手書き目安 0.4 µs/byte に対して complex は 1.35 倍（round 4 は 1.6 倍）。
長期目標 0.02 ms にはまだ 7〜9 倍あり、残りは AST の確保（API 契約）と finish の走査。

## 満たせていないこと（正直な記録）

1. **生成 driver は fast mode を使わない**。driver は成功時も `hints`（投機的失敗）を JSON に出すので、
   fast mode で走らせると必ず再解析になり、常に 2 回解析するだけになる。そこで driver は
   `diagnostics: true` を渡す。結果として **driver JSON と corpus は byte 一致のまま、driver 自身は速くならない**。
   依頼の「driver も fast mode を先に走らせる」は満たしていない。満たすには `hints` を driver の出力から
   外す（= 公開契約の変更）か、hints を診断 DAG 無しで作れる形にする必要がある。
2. **成功した解析の `hints` は既定で空になる**。これは仕様変更で、`ParseOptions::diagnostics = true` で戻る。
   影響したテストは 2 つ（`crates/ubnfc-rust/tests/support/semantic_assertions.rs` の hints assertion、
   `examples/p4-rust/tests/fixtures.rs` の診断予算テスト）で、どちらも `diagnostics: true` を渡す形に直した。
   corpus / oracle の期待値は変更していない。
3. **`limits.diagnostics` の意味が変わる**（fast mode では消費されない）。設計節のとおり。
4. **`const DIAG: bool` の単相化は作っていない**。上限だけ測って採らなかった（上記）。
5. **拒否される入力は約 2 倍遅くなる**（再解析）。A/B の complex-tail 行を参照。
   これは設計上の交換で、「成功が既定、失敗は稀」を前提にしている。失敗が多い用途
   （エディタの逐次解析など）は `diagnostics: true` を渡せば 1 回解析に戻せる。
6. **`TokenScanner` が 2 回呼ばれる**（再解析する結果のとき）。純粋な scanner を前提にしている
   （P4 の Extern 3 種と `RejectExtern` は状態を持たない）。契約として明文化はしていない。
   これは実測で見つかった: `crates/ubnfc-rust/tests/support/review_assertions.rs` の
   `scanner_receives_five_state_fields` と `expression_memo_success_failure_effects_and_diagnostics` は
   scanner の呼出し回数を数えており、再解析で 1 → 2 / 2 → 4 になった。どちらも
   「1 回の解析の中で scanner が何回呼ばれるか」を見るテストなので、`diagnostics: true` を渡して
   解析 1 回に固定し、assertion は元のまま残した（回数の契約を緩めていない）。
   効果を持つ scanner を使う利用者は `diagnostics: true` を渡す必要がある。
7. 計測は共有ホストで行った（load average 11〜33、他エージェントの Rust テストと Java ベンチが並走）。
   絶対値は round 4 の表より悪く出る。**同一セッションで交互に実行した相対値**で読むこと。

## 残り

1. **AST 経路の確保**（round 3・4 と同じ）: `Box<Ast>` / `Ast::Text(String)` / `ValueSpan` の String /
   `Capture` 72 byte は公開 API の契約。`Ast::Text` の `Rc<str>` 化などは別 issue。
2. **finish の 2 回の全木走査**（出現収集と AST 構築）は x64 で 3 ms 級。round 4 の残り 2 のまま。
3. **driver の高速化**: `hints` を診断 DAG 無しで作れる形（例: 最遠位置と label 集合だけの軽い記録を
   fast mode でも残す）にすれば、driver も fast mode を使える。ただしそれは「成功経路は何も払わない」を
   薄める方向なので、hints の契約を先に決める必要がある。
4. **単相化（`const DIAG: bool`）** は上限（`static` 行）まで complex 認識で 9 µs / AST で 3 µs。
   生成 parser を 2 部にする代償に見合うかは、i-cache の実測が要る（本 round では作っていない）。
5. round 3 の残り 3（`analysis.mayStart` への `firstChars` 反映）と 4（memo entry の縮小）は未着手。

## 再現

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cargo run -q --offline --release -p ubnfc-p4-rust --example bench -- complex complex-x4 complex-x16 complex-x64 complex-tail invalid-syntax
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust \
  --report docs/reports/2026-09-23-rust-corpus-diag2.md --json docs/reports/2026-09-23-rust-corpus-diag2.json
```
