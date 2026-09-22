# P4 Rust 生成 parser の性能改善 M5 round 3（2026-09-22〜23）

[`2026-09-22-p4-rust-perf-m5-round2.md`](2026-09-22-p4-rust-perf-m5-round2.md)（round 2: complex 0.312 ms、x64 35.2 ms）の続き。
目標は complex ≤ 0.2 ms（AST 付き）、complex-x64 ≤ 15 ms。意味論は変更していない（corpus は round7 / M5 と同じ verdict、
selfhost 固定点、P4 golden AST byte 一致、`cargo test --workspace` 全緑、clippy `-D warnings`、fmt）。

## 結論

- complex.tiny（332 byte）は **認識 0.180 ms / AST 0.228 ms**（round 2 の同セッション再計測 0.243 / 0.313 ms）、
  complex-x64 は **認識 14.7 ms / AST 15.7 ms**（同 34.7 / 36.3 ms）。
  目標に対して: complex AST は 0.2 ms に**届かず**（認識だけなら 0.18 ms で下回る）、x64 は認識で 15 ms を下回り、
  AST 付きは **15.5〜16 ms で境界上**（負荷の低い区間で 15.5〜15.8、最終 A/B の中央値 15.7。目標 ≤ 15 ms には僅かに届かない）。
- rule 評価 1,063 → 759 / parse（memo hit 463 → 169）、確保 1,822 → 858 回（AST。認識は 717 → 367）、
  1 parse の確保量 1.17 MB → 0.16 MB（AST）。x64 は 81 MB → 9 MB。
- 4 つの手順のうち効いたのは **buffer pool（x64 のページフォルト）**と**左因数分解**で、Extern 先頭文字宣言は
  単独では利得がなく（下記）、AST 経路の確保削減は AST 差を 80 → 50 µs に縮めた。

## 手順 1: Extern token の先頭文字宣言

### 仕組み

- 宣言ファイル `scanners/first-chars.json`（scanner registry が所有）: 元クラス名 → 成功し得る先頭 code point の集合
  （1 code point の文字列 / 整数 / `{start,end}` の閉区間、`null` は unknown）。同梱 6 クラスを宣言した
  （DoubleQuoted `"`、SingleQuoted `'`、StringLiteral `'` `"`、CodeStart / CodeEnd `` ` ``、ExternalCursor `😀`）。
- IR: `TokenSpec.extern.firstChars`（省略可。`ir/schema` に optional 追加、`CharRange` の配列、昇順・互いに素を
  `validate_references` で検査、`to_canonical_json` で整列）。
- front: `ubnfc ir|gen --extern-first-chars FILE` で読み込み `originalClassName` で引いて埋める
  （`crates/ubnfc-front/src/ir/first_chars.rs`、`apply_extern_first_chars`）。宣言の無いクラスは警告して unknown のまま。
  `ir/fixtures` の golden は `pipeline_golden.rs` がこのファイルを渡して生成する（P4 の golden だけが変わる: 3 token に
  `firstChars` が付く。他の fixture に Extern は無い）。harness の run-all は渡さない（corpus の verdict は宣言に依存しない）。
- emitter（`predict.rs::token_start`）: `firstChars` のある Extern だけ `Known`。除外した候補の失敗再生では Extern の葉を
  本物の scanner 呼出しで再生するので、失敗診断（scanner の表示文字列）は宣言の有無で変わらない。
- 健全性は scanner 側の責任（必要条件: 集合外の先頭文字では消費なしで失敗）。
  `examples/p4-rust/tests/fixtures.rs::extern_first_chars_declaration_matches_bundled_scanners` が同梱 6 scanner について、
  宣言外の ASCII 文字と数種の非 ASCII を先頭位置・行中位置 × consumed / matchOnly × reset の組で確認する。
  `crates/ubnfc-rust/tests/round3.rs` は宣言の有無で再生関数の生成が変わることと、不正な区間の拒否を固定する。
- front の `analysis.mayStart`（両 backend の predict 表）は変えていない。P4 の plan と Java 生成物が同時に変わるため別 issue。
- 文書: `docs/harness-driver-protocol.md`「Extern scanner の先頭文字宣言（M5 round 3）」。

### 計測と、再生方針の見直し

P4 で新たに除外対象になったのは SliceBaseExpression の 8 候補（全て `SliceBaseReceiver '[' …`。STRING を含む receiver が
Known になった）だけで、`StringTerm` 系（16 候補、`SliceExpression` を含む）は展開の大きさ上限（192 node）を超えるので
対象外のまま。この 8 候補の除外は **単独では遅くなった**（complex 認識 0.244 → 0.263 ms、rule 評価 1,063 → 983 なのに）。
除外した候補の再生関数は候補ごとに `SliceBaseReceiver` の 12 候補（STRING の scanner 呼出し・失敗時の `Vec` 確保を含む）を
展開するので、本物の経路（1 回評価 + 7 回 memo hit）より高くついた。

再生に本物の rule 関数（memo 付き）を混ぜる方針も試した（展開の大きさ上限を 24 に下げ、超える部分木と、複数箇所から
参照される大きな rule は本物を呼ぶ）。除外対象は 136 候補に増えたが rule 評価が 1,063 → 1,106 に増え（memo hit 1 回で
済んでいた `Expression` の 5 候補を再生が別々に本物で呼ぶ等）、0.253〜0.263 ms と遅いままだった。結論として **再生は
失敗経路を全て展開できる候補にだけ置く**（round 2 と同じ）方針に戻し、共有先頭要素の候補列は手順 3 の左因数分解で
「群として 1 回だけ除外判定・1 回だけ再生」する。この組合せで SliceBase の除外は利得になる（手順 3 の表）。

## 手順 2: memo と x64 の cache 圧（buffer pool）

x64 の µs/byte が complex の 1.7 倍だった原因は memo 表の cache 効果ではなく **ページフォルト**だった。
`/usr/bin/time -v` で x64 の bench は 1 parse あたり約 6,400 minor fault（1 parse で 59 MB を確保: 診断 arena 12 MB、
event arena 8 MB、memo 4.8 MB、display log、…）。glibc は同じ大きさの mmap chunk を free した直後の同じ大きさの確保を
再び mmap するので（`mmap_threshold` は free した chunk の大きさに更新され、`nb >= threshold` で mmap）、parse ごとに
全 page を触り直していた。

対策は `ParseOptions::reuse_buffers`（既定 true）: Session の大きな buffer（event arena、主診断 arena、display の log /
saved / frames、memo 表、trivia cache、効果列、AST 構築の作業 Vec）を thread local の pool に 1 組保持し、次の parse で
内容を捨てて再利用する（`Memo::reset` / `Diagnostics::reset` / `DisplayDiagnostics::reset`、確保は保持）。
前回の memo slot 表が今回の見積りの 8 倍より大きければ縮める（小さい入力で大きな表をゼロ埋めしない）。
保持する組が 64 MB（`POOL_RETAIN_BYTES`）を超えたら捨てる。結果には影響せず、確保の振る舞い（常駐量）だけが変わる。
x64 認識 41 → 17 ms（同一 binary で on/off 比較）、complex 認識 0.255 → 0.236 ms。

memo entry の縮小（68 byte）と失敗 entry の省略は行っていない。pool 後の x64 は認識 0.62 µs/byte で complex（0.54）との差が
小さく、残る差は memo 表（entries 4.3 MB + slots 0.5 MB）と event arena の L2 超えで、entry を 8〜16 byte 縮めても
数 % の見込み。失敗 entry は候補除外・因数分解後は memo hit で実際に再訪されるもの（SliceBase 系の残り）が中心で、
plan の `provedOmission`（単一実行経路）で省ける失敗 entry は無い。

## 手順 3: 共通接頭辞を持つ候補列の左因数分解（`crates/ubnfc-rust/src/factor.rs`）

### 適用した群

| 選択 | 群 | 共有接頭辞（trie） |
|---|---|---|
| SliceBaseExpression（8 候補） | 8 候補全体 | `SliceBaseReceiver @value '['` → {`SliceStartIndex ':'` → {`SliceEndIndex` → {`':' SliceStepIndex ']'`, `']'`}, `':' SliceStepIndex ']'`, `']'`}, `':'` → {`SliceEndIndex` → {…}, `':' …`, `']'`}} |
| SliceNestedExpression（8 候補） | 8 候補全体 | `SliceBaseExpression @value '['` → 同上 |

（`ubnf.ubnf`（selfhost）にも該当する選択は無い。）

### 見送った群と理由

- 型付き Expression 塔（`Expression ::= NumberExpression | BooleanExpression | StringExpression | ObjectExpression | …`、
  BranchExpression、ArgumentExpression、ObjectExpression）: 候補の先頭要素は互いに異なる rule で、共通なのは 3 段下の
  primary（`$var`、`if`、`(`…）だけ。`NumberTerm` と `StringTerm` の接頭辞は等しくない（`$x as string` は StringTerm だけ）
  ので `X A | X B` の形ではなく、「primary を 1 回解析して型で分岐」は文法の書換え（Rule イベント・Rule node・capture の
  投影が変わる）になる。対象外。
- BooleanFactor（`BooleanEqualityExpression | ComparisonExpression | StringComparisonExpression | BooleanComparable`）:
  `BooleanEqualityExpression ::= BooleanComparable EqualityOp BooleanComparable` と 4 番目の候補 `BooleanComparable` は
  rule 境界を越えた接頭辞。展開すると `BooleanEqualityExpression` の Rule イベント / node の生成条件が変わる。対象外。
- StringTerm / SliceBaseReceiver / NumberFactor の候補列: 先頭要素が全て異なる（構造的に等しい連続候補が無い）。

### 同値の論証（実装が守る不変条件）

対象は「連続する候補 Seq が構造的に等しい先頭要素列を持つ」群だけ（`factor::share`: 同じ RuleRef / literal / token、
同じ execution flag、同じ trivia policy（入口・各要素後）、capture 規則（span 規則・skip、1 site 以下）と label が一致、
要素 wrapper が memo されない、群内で候補除外の条件が一致）。最長一致は対象外。

- PEG 意味論: 共有要素 X は同じ状態で同じ結果（memo が保証する決定性）なので `X A | X B ≡ X (A | B)`。
  共有要素の後の続きは宣言順に試し、最初の成功を採る。接頭辞で完結する候補があれば、そこで成功する（参照経路でも
  それ以降の候補は試されない）。
- 主診断 DAG（offset / expected / rulePath）: 参照経路は候補ごとに X の診断を `Join` する（memo hit でも `Rule` node は
  候補ごとに新しく、子は同じ）。`Diagnostics::summary` が見るのは位置の最大、その位置の label の左から DFS の初出順、
  最深 rule 経路（同深さは左優先）だけで、同じ位置・同じ label 列・同じ深さの複製 1 個と n 個を区別できない。
  `join` は結合的（位置の最大、同位置は左から DFS）。全候補が X で失敗するときは 1 回だけ join する。
- 公開 farthest（display frame、D-014）: 観測は「最遠位置とその位置の label 集合」で、同じ位置への同じ label の記録は
  冪等。呼ばれない wrapper（群の各候補 Seq、共有要素の候補側 expression）が失敗時に記録する label は同じ位置に再生する
  （同じ位置の記録はまとめる。予算の計数は減る）。
- capture / Values / Token イベント: 共有要素は先頭候補の expression 関数で評価し、採用候補が別なら Capture の site と
  Token の expr をその候補のものに付け替える（`Session::retag`。要素 wrapper は memo されないのでこの event は今回の
  結果からしか参照されない）。候補 Seq の Values（`wrap` は候補ごと）・capture・出口 trivia は採用候補のものを生成する。
- scope 効果: 共有接頭辞の効果は 1 回だけ記録され、各候補は接頭辞後の mark へ restore して続きを試す。
- 候補除外との合成: 群の先頭要素が除外対象（全候補で同じ条件）なら、判定 1 回・再生 1 回（先頭要素の `guard_e{i}`）。
- 変わるのは統計（rule 評価・memo hit）と診断予算の計数だけ。`ParseOptions::factor`（既定 true）で参照経路へ戻せる。
- 検証: `predict_matches_reference_path_on_truncated_and_corrupted_inputs` を拡張し、全 16 fixture の完全入力・切り詰め・
  1 文字破壊で `(predict, factor) ∈ {(on,on), (on,off), (off,on)}` × memo on/off の ParseResult（診断・farthest・rulePath・
  AST・capture・scope イベント）を参照経路 `(off, off)` と比較する（比較 約 11,000 件）。`tests/round3.rs` が因数分解経路の
  生成（最長一致には生成しない）を固定する。`crates/ubnfc-rust/tests` の `scope_rollback_and_replay`（`Decl '!' | Decl Use`）は
  既定で因数分解され memo hit が消えるので、memo hit での効果の巻戻し・再生は `factor: false` の参照経路で検証し、
  両経路の宣言・参照・scope イベントが等しいことを併せて確認する形にした。

## 手順 4: 結果 API と AST 経路の確保

- rule ごとに局所 event を 1 回だけ走査して全 capture site の出現を集め（`collect_captures`）、field ごとに site で選ぶ
  （従来は field ごとに再走査）。
- `build_values` / `build_rule` / `map_rule_{i}` は出力 `Vec<Ast>` を受け取って push する（node ごとの `vec![node]` を廃止）。
  scalar / optional の field は pool の作業 Vec（`take_values` / `take_texts`）で集めて 1 要素を取り出す。List は node へ
  移すので新しい Vec。
- 認識だけの経路: scope 効果の capture 走査 Vec を pool 化し、Declare / Use が 1 件の site では `name` を複製せず移動。
  `occurrences()` は `lexical` でないとき rule 出現（`LexicalOccurrence`）を作らない（完了順だけ数える）。
- `value_spans` は事前確保。
- 残り（API 契約のため未着手）: `ValueSpan.path` / `text` の String（node + text 値ごとに 2〜3 確保）、`Ast::Text(String)`、
  node ごとの `Box<Ast>`、`Reference.name` / `ScanEffect::Use.name` の String。AST 経路の残り約 50 µs（131 node、380 ns/node）の
  大半はこれらと `Rc<ScanEffect>`。

## 段階別 A/B（`examples/p4-rust` bench、release、warmup 後 5 run の中央値、ms/parse）

共用ホストで Java M5 の計測が並走することがあり（load average 2〜5、段階 1〜2 の一部は cargo test と並走）、各段階の値は
その段階の直後に baseline binary（round 2 末 b4778ab）と交互に計った代表値。最終 2 行は「最終 A/B」（baseline と最終 binary を
同じセッションで交互に 3 回ずつ実行し、各列の中央値。load average 3〜6）。allocs / bytes は GlobalAlloc の計数。

| 段階 | complex 認識 | complex AST | x64 認識 | x64 AST | rule 評価 | memo hit | allocs (AST) | bytes (AST) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| round 2 末（b4778ab、作業開始時） | 0.239 | 0.314 | 34.5 | 36.4 | 1,063 | 463 | 1,822 | 1.17 MB |
| 1 Extern 先頭文字宣言（SliceBase 8 候補が除外対象に） | 0.263 | 0.329 | 40.4 | 46.0 | 983 | 393 | 1,894 | 1.58 MB |
| 1' 再生に本物の rule を混ぜる方針（不採用） | 0.253 | 0.339 | 34.4 | 36.5 | 1,106 | 506 | 1,822 | 1.17 MB |
| 2 buffer pool（1 の上に。同 binary で reuse off: 0.255 / 0.333 / 41.2 / 41.7） | 0.236 | 0.315 | 16.7 | 21.3 | 983 | 393 | 1,817 | 0.39 MB |
| 3 左因数分解（同 binary で factor off: 0.240 / 0.311 / 20.6 / 23.2） | 0.189 | 0.288 | 15.8 | 21.2 | 759 | 169 | 1,698 | 0.39 MB |
| 4 AST 経路の確保（1 回走査・出力 Vec・pool） | 0.194 | 0.241 | 13.0 | 16.5 | 759 | 169 | 1,133 | 0.22 MB |
| 4' 認識経路の確保（scope 走査 pool・name 移動・lexical 省略） | 0.187 | 0.230 | 13.0 | 15.8 | 759 | 169 | 858 | 0.16 MB |
| 最終 A/B: baseline | 0.243 | 0.313 | 34.7 | 36.3 | 1,063 | 463 | 1,822 | 1.17 MB |
| 最終 A/B: 本 round | 0.180 | 0.228 | 14.7 | 15.7 | 759 | 169 | 858 | 0.16 MB |

## 最終値と目標

| 入力 | bytes | 認識 ms | AST ms | µs/byte (AST) | 目標 | 手書き目安 0.4 µs/byte |
|---|---:|---:|---:|---:|---:|---:|
| complex | 332 | 0.180 | 0.228 | 0.69 | ≤ 0.2 ms | 0.13 ms |
| complex-x4 | 1,293 | 0.686 | 0.833 | 0.64 | — | 0.52 ms |
| complex-x16 | 5,179 | 2.75 | 3.15 | 0.61 | — | 2.1 ms |
| complex-x64 | 20,923 | 14.7 | 15.7 | 0.75 | ≤ 15 ms | 8.4 ms |

complex の AST 付き 0.2 ms には届かない。認識は 0.19 ms で、AST 構築（131 node）の約 50 µs が API 契約の確保
（`ValueSpan` の String、`Ast::Text(String)`、`Box<Ast>`）に縛られる。認識側の残りは display の failures 記録
（約 6,000 回 / parse）と rule 入口・memo lookup で、次の候補は末尾。

## 契約の検証

- `docs/design/04-cursor-and-diagnostics-contract.md` §1（cursor）: 生成 cursor コードは無変更。因数分解経路は候補 Seq と同じ
  `reset=false` / `begin` / `commit` を共有接頭辞に 1 回だけ適用する。§2（診断）: 上記の同値論証。
- `03-design-v1.md` §3.4: 候補除外（保守的証明付き、除外後の失敗診断が同値）に、Extern の先頭文字を利用者（scanner registry）
  宣言で加えた。左因数分解は `ParseOptions::factor` で参照経路へ戻せ、差分テストで同値を確認する。§3.7: memo key / 再生は
  不変。buffer pool は memo の内容を parse をまたいで保持しない（`reset` で空にする）。
- corpus: `docs/reports/2026-09-22-rust-corpus-m5r3.{md,json}`。系統別の表が round7 / M5 / round 2 と完全に同じ（Pass 1292 / 期待拒否 Pass 30 / Known 28 / Fail 0 / 未観測 75）。harness には先頭文字宣言を渡していない。
- selfhost 固定点（`crates/ubnfc-selfhost/tests`）緑、`src/generated` を再生成して commit。P4 golden AST は byte 一致、
  handwritten-target の Operators / Power golden も byte 一致（`crates/ubnfc-rust/tests/generated.rs`）。
- Java backend: 新しい optional field は schema に追加し、Java の `IrReader`（schema 検証 + record）は未知でないので受理する
  （worktree 内で `ReviewRegressionTest` を実行して確認: `mvn -o -q test -Dtest=ReviewRegressionTest` 成功）。`ubnfc-java` のソースは変更していない。

## 残り

1. complex の AST 経路: `ValueSpan` / `Ast::Text` / `Box` の確保は API 契約。`value_spans` を遅延（要求時）に構築する
   オプション、`Ast::Text` の `Rc<str>` 化は API 変更なので別 issue。
2. 認識側: display の `failures` 記録（6,000 回 / parse）の削減（guard 内の label を静的 1 配列にまとめる、rule label の
   重複除去）、rule 単位 memo（wrapper 後の Step を保存して hit の後半を省く）。
3. `analysis.mayStart` への `firstChars` の反映（Java backend の predict 表にも効く）。P4 の plan と Java 生成物が変わるので
   Java の M5 と調整して行う。
4. memo entry の縮小（68 → 56 byte）と `Event`（88 byte）の u32 化。x64 の残る cache 効果（認識 0.62 vs 0.54 µs/byte）向け。

## 再現

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cargo run -q --offline --release -p ubnfc-p4-rust --example bench -- complex complex-x4 complex-x16 complex-x64
UBNFC_BENCH_OPTS=factor=0,reuse=0 cargo run -q --offline --release -p ubnfc-p4-rust --example bench -- complex complex-x64
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust --report docs/reports/2026-09-22-rust-corpus-m5r3.md --json docs/reports/2026-09-22-rust-corpus-m5r3.json
```
