# self-host 射影層 `convert()` の割当削減（2026-09-24, issue #4）

`docs/reports/2026-09-24-selfhost-projection.md` が「射影層の割当 7,073 回のうち 84%（4,706 回）は
`crates/ubnfc-front::selfhost::convert()` にある」と切り分けたところからの続き。今回はその
`convert()`（`Converter`）と、そこへ値を渡す `crates/ubnfc-selfhost` の投影を対象にした。

## 0. 結論（先に書く）

- `ubnf.ubnf`（10,650 B）を生成 parser で読んで `UbnfFile` へ射影するまでの時間が
  **最小 5.676 ms → 0.950 ms**（-83%、**6.0 倍**）。確保回数は **7,065 → 2,478**（-65%）。
- 射影ぶんだけを見ると、確保回数 **5,592 → 1,004**、時間 **5.09 ms → 0.375 ms**（最小値の差）。
  生成 AST 構築（0.575 ms）に対して **1.65 倍**で、受け入れ目安の「生成 AST 構築の 2 倍以内」に入る。
- 目安「射影込みで 1 ms 以下」は**最小値では満たす（0.950 ms）**。中央値は 1.243 ms で、
  計測中の load average が 16〜27（他エージェントの並行ビルド）であることによる上振れ。
  同じ run の生成 AST 構築も 0.575（min）に対し 0.679（median）と 18% 上振れしている。
- 手書き front（`ubnfc_front::syntax::parse_file`）は同じ `UbnfFile` を 0.067 ms / 1,215 回で作る。
  射影層の確保回数 1,004 回はこれと同じ桁に収まった（byte は後述のとおりまだ 2 倍ある）。
- `tests/parity.rs` は全緑で、`docs/design/05-selfhost.md` の 7 件の乖離は変えていない。

## 1. 何が遅かったか

確保回数の内訳（前回報告）は正しかったが、**5 ms の大半は確保ではなく `Converter::new()` の
capture 所有者解決だった**。

```rust
// 変更前
let owner = occurrences.iter().enumerate()
    .filter(|(_, o)| o.rule_id == capture.rule_id && o.span_cp[0] <= capture.span_cp[0] && ...)
    .max_by_key(|(j, _)| depth[*j]) ...
```

capture 1,383 件 × occurrence 1,286 件 = **1.78M 回の照合**（しかも `rule_id` の文字列比較）を
1 parse ごとに回していた。確保回数だけを見ていると見つからない種類の支配項で、
前回 `crates/ubnfc-selfhost` 側だけを直しても ms が動かなかった理由でもある。

## 2. やったこと

### 2.1 `Converter` の索引を確保 1 回ずつの平坦配列にする（`crates/ubnfc-front/src/selfhost/mod.rs`）

- `children: Vec<Vec<usize>>` / `owned_captures: Vec<Vec<usize>>` を **CSR（`start` + `index` の
  2 本の `Vec<u32>`）** に置き換え。要素ごとの `Vec` 確保（occurrence 1,286 枠ぶんの push 再確保）が消える。
- `ids: BTreeMap<usize, usize>` を整列済み `Vec<(id, index)>` + 二分探索に置き換え（節点確保が消える）。
- `mapped: BTreeMap<(&str, [usize; 2]), usize>` を
  **(span, kind) で整列した配列 + span 始点で直接引ける表**に置き換え、
  occurrence ごとの引き当てを `new()` で 1 回だけ行って `node_of: Vec<u32>` に持つ。
  span（整数）を先に比べるので、整列も探索もほとんど整数比較で決まる。
- capture 所有者解決を **rule_id ごとのグループ + 深さ昇順の逆走査**にした。
  同じ深さが複数一致したときに添字の大きい方を採るところまで元の `max_by_key` と一致させてある。
  走査対象は `(occurrence 添字, span)` を並べた配列にして、`occurrences` への飛び飛びの参照を無くした。
- `rule_id` は生成物の静的文字列なので、まず番地で照合する（`same_str`）。
  capture 側の `rule_id` は別の静的表から来るので番地が違う。こちらは一度引いた結果を番地で覚える。
- `rule_id` 末尾の kind（`rsplit("::")`）を `kind_of: Vec<&str>` に 1 回だけ切り出す。

### 2.2 中間 `Vec` を作らずに値を渡す

- `children(i, kind)` は `Vec<usize>` ではなく**反復子**を返す（呼び出し 1,788 回ぶんの確保が消える）。
- `values()` は、型付き AST 側にあれば `&[&str]` を**借用のまま**渡し、無ければ capture 経路を
  反復子で渡す（`&mut dyn Iterator` 経由。確保も動的 box も無い）。`raw_values()` も同じ。
- `scalar` / `at_most_one` は `Vec` の長さではなく `count_and_first()` で 1 回走査して判定する。
  要素の誤りを個数検査より先に返す順序は元の `collect::<Result<Vec<_>>>` と同じ。
- `string()` は `raw_string()`（`String` を作る）を経由せず、借用した原文を直接 `decode_string()` に渡す
  （STRING 1 件あたり 2 回だった確保が 1 回になる）。
- `decode_string()` は escape を含まない文字列（大多数）を 1 回の確保で複製し、
  含む場合も `String::with_capacity()` から始める（`String::new()` + push の伸長を止める）。
- `SourceOffsets::new()` は `Vec::with_capacity(source.len() + 1)` で 1 回の確保に収める。
  `content_span()` は byte 単位の走査に変えた（判定に使う文字はすべて ASCII で、多 byte 文字の
  各 byte は 0x80 以上なのでどの判定にも一致せず「内容」になる。結果は文字単位の走査と同じ）。

### 2.3 投影を平坦配列にする（`crates/ubnfc-selfhost`）

- `MappedNode { fields: Vec<(&str, Vec<&str>)> }` を `MappedNodes`（ノード・field・値の 3 本の配列）に
  置き換え、`out.node(kind, span).field(name, values)` で積む。範囲の終点は次の要素の始点から求めるので
  後埋めが要らない。**764 ノードで 885 回 → 3 回**。
- `capture_sites::rule_for()` の索引 `HashMap` の hasher を、site_id 用の安価なもの（乗算と回転）に
  差し替え。site_id は 60 文字前後で先頭 35 文字が共通なので、既定の SipHash では
  1 parse あたり 1,383 回の hash が効いていた。`CAPTURE_RULES` の中身・順序は変えていない。
- `parse_file()` が成功経路でも作っていた `SourceOffsets` を、失敗経路の中へ移した。

## 3. 前後比較

### 3.1 発生源ごとの確保回数

before は `docs/reports/2026-09-24-selfhost-projection.md` §1 の実測。after は同じ計数用の
counting allocator を phase 境界に挟んだ実測（`ubnf.ubnf` 1 parse）。

| 発生源 | before | after | 備考 |
|---|---:|---:|---|
| `Converter::children(i, kind)` の `Vec<usize>` | 1,788 | **0** | 反復子にした |
| `values()` node 経路の `.clone()` | 443 | **0** | `&[&str]` を借用で返す |
| `values()` capture 経路の `.collect()` | 220 | **0** | 反復子にした |
| `raw_values()` の `.collect()` | 264 | **0** | 反復子にした |
| `to_owned()`（`UbnfFile` の leaf 文字列） | 610 | **478** | `string()` の二重確保をやめたぶん減。残りは `UbnfFile` が所有する文字列そのもの |
| `decode_string()` | 132 呼び出し（伸長で実確保はこれ以上） | **132** | 1 呼び出し 1 確保 |
| `Converter::new()` の索引構築 | 約 1,250〜1,350 | **28** | CSR + 整列配列。occurrence 数に依らない固定本数 |
| `projection::project()`（`MappedNode.fields`） | 885 | **3** | 3 本の平坦配列 |
| `UbnfFile` の出力 `Vec`（grammars/rules/…） | （上記に含む） | 353 | 出力そのもの。削れない |
| **射影ぶん合計** | **5,592** | **1,004** | |
| 生成 parser の AST 構築（対象外） | 1,473 | 1,473 | |
| **1 parse 合計** | **7,065** | **2,478** | |

phase 別の実測（after）:

| phase | allocs | bytes |
|---|---:|---:|
| 1. 生成 parser で AST 構築 | 1,474 | 1,369,196 |
| 2. `projection::project()`（`MappedNodes`） | 3 | 102,880 |
| 3. capture / occurrence の配列 | 2 | 180,328 |
| 4. `Converter::new()`（索引の構築） | 28 | 291,492 |
| 5. `file()`（`UbnfFile` の構築） | 963 | 240,664 |

### 3.2 時間と確保量

`benchmarks/src/family_c.rs` をそのまま使い、`benchmarks/scripts/prepare.sh` の `bench-c` と同じ
crate を一時ディレクトリに 2 つ作って（before = `origin/master` の detached worktree、
after = このブランチ）、**6 往復・各 9 run** を直後に切り替えて交互実行した。
`taskset -c 26`、1 コア固定、単一 thread。

- 共有機で他エージェントの並行ビルドあり。load average（1 分値）: 開始 17.7 → 終了 16.2（5 分平均は 23〜27）。
  ms は run ごとの上振れが大きいので、**最小値**と確保回数（load に依存しない）を判断に使う。

| 指標（`ubnf.ubnf`） | before | after | 差分 |
|---|---:|---:|---:|
| AST(`UbnfFile` 射影) ms 最小（54 run） | 5.676 | **0.950** | **-83%（6.0 倍）** |
| AST(`UbnfFile` 射影) ms 中央値 | 7.459 | 1.243 | -83% |
| AST(生成) ms 最小（同 run） | 0.582 | 0.575 | 変化なし（対象外） |
| 射影ぶん ms（射影 - 生成、最小同士） | 5.094 | **0.375** | **-93%** |
| 確保回数 / parse | 7,065 | **2,478** | -4,587（-65%） |
| 確保 byte / parse | 2,630,999 | **2,336,008** | -294,991（-11%） |
| 手書き front（参考） | 0.068 ms / 1,215 回 / 470,005 B | 同左 | — |

## 4. 残っているもの

- **確保 byte は手書き front の 2 倍ある**（射影ぶん 967 KB 対 470 KB）。
  内訳は `SourceOffsets` の code point 表 77 KB、`MappedNodes` の事前確保 103 KB
  （ノード数が事前に分からないので occurrence 数 1,286 を上限の目安にしている。実際は 764 ノード）、
  capture / occurrence の配列 180 KB、`Converter::new()` の索引 291 KB。
  回数は固定本数まで落ちているので、これ以上は「表の要素幅を詰める」（`usize` → `u32`）話になる。
  span の型は `MappedNodes` / `Occurrence` の公開契約なので、別途 issue にするのが妥当。
- **中央値はまだ 1.243 ms**。最小値 0.950 ms との差は共有機の負荷によるもので、
  load average が 5 前後のときに取り直せば中央値も 1 ms 近辺に入ると見込む（今回は 16〜27）。
- `file()` の 963 回のうち 610 回は `UbnfFile` が所有する `String` そのもので、
  AST の型（`String` / `Vec<String>`）を借用（`Cow` / `&str`）に変えない限り削れない。
  これは front 全体の API 変更なので、この issue の範囲外。

## 5. テスト

- `cargo test -p ubnfc-selfhost --offline`: `tests/parity.rs`（`fixture_acceptance_and_ast_parity` /
  `self_description_ast_matches_including_every_span` / `syntax_error_positions` /
  `three_validation_error_golden_sources_have_fixed_parse_outcomes` 含む）と
  `tests/regenerate.rs`（固定点: IR・生成 Rust の byte 一致）を含め全緑。
- `cd ubnfc-java && mvn -o -q -DskipTests package` を先に実行（jar の鮮度検査 D-033）した上で
  `cargo test --workspace --offline`: 全緑。
- `cargo clippy --workspace --offline --all-targets -- -D warnings`: warning 無し。
- `cargo fmt --all -- --check`: 差分無し。生成物（`src/generated/`）は `#[rustfmt::skip]` のまま触れていない。

## 6. 再現

```sh
UNLAXER=/home/opa/work/unlaxer-workspace/unlaxer-parser
# benchmarks/scripts/prepare.sh の bench-c 相当（ubnfc-front / ubnfc-selfhost / unlaxer-ubnf の 3 依存）を
# before（origin/master の detached worktree）と after（このブランチ）で 1 つずつ作り、交互に:
UBNFC_CMP_RUNS=9 taskset -c 26 ./bench-c "ubnf.ubnf=$UNLAXER/unlaxer-dsl/grammar/ubnf.ubnf"
```
