# self-host 射影層の割当内訳と対処（2026-09-24, issue #4）

`docs/reports/2026-09-23-parser-speed-comparison.md` Family C で報告した、
`ubnf.ubnf` を生成 parser で読んでから `UbnfFile` へ射影するまでの
5.238 ms（9.3 倍、確保回数 1,473 → 7,073）を掘り下げた。

## 0. 結論（先に書く）

- 射影層の割当 7,073 回のうち、**このIssueのスコープ内（`crates/ubnfc-selfhost`）が生む分は 885 回（16%）**。
  残り **4,706 回（84%）は `crates/ubnfc-front::selfhost::convert()`（`Converter`）が生む**。
- `crates/ubnfc-selfhost` 側で安全に削れる分（`Vec` の再確保・capture site 照合の線形探索）を実装し、
  割当を 7,073 → 7,065（-8、-0.1%）、射影フェーズの確保 byte を 143,648 → 101,200（-30%）に減らした。
  ms は共有機のノイズに埋もれる程度（後述 §3）。
- **≤1 ms の受け入れ目安には届いていない。** 支配的な確保源 `Converter::children()` / `values()` /
  `to_owned()` 等は `crates/ubnfc-front` にあり、このブランチのスコープ（このIssueのセットアップで
  「`crates/ubnfc-selfhost` 配下に留める、`crates/ubnfc-front` は他エージェントが同時に触っている」と
  明示されている）の外にある。生成 parser の AST 形状の問題ではなく、**射影ロジックの置き場所（crate 境界）**
  が理由なので、正直に手前で止め、フォローアップを起票した（→ issue リンクは §4）。

## 1. 割当 7,073 回の内訳

計測は `benchmarks/` と同じ counting allocator（`GlobalAlloc` で確保回数・byte を数える別 parse、
計時には含めない）を使い、`ubnfc_selfhost::parse_file` を 3 フェーズに分けて差分を採った
（一時的な計測用フックで、コミットには含めていない）。

| フェーズ | 累積 allocs | 差分 allocs | 差分 bytes | 実装場所 |
|---|---:|---:|---:|---|
| 生成 parser で AST 構築 | 1,474 | 1,474 | 1,369,196 | `ubnfc-selfhost::generated`（生成物） |
| + `projection::project()`（型付き AST → `MappedNode`） | 2,359 | **885**（修正前 893） | **101,200**（修正前 143,648） | `crates/ubnfc-selfhost` — **このIssueのスコープ** |
| + `ubnfc_front::selfhost::convert()`（`MappedNode` + occurrence → `UbnfFile`） | 7,065 | **4,706** | 1,160,603 | `crates/ubnfc-front::selfhost` — **スコープ外** |

`convert()` 側 4,706 回の内訳（同じく `ubnf.ubnf` 1 parse、一時カウンタで実測）:

| 発生源 | 回数 | 型/理由 |
|---|---:|---|
| `Converter::children(i, kind)` | 1,788 | 呼び出しごとに親 occurrence の子を kind でフィルタし直し、`Vec<usize>` を新規確保。結果をキャッシュしない。木を辿るたび（`file`/`grammar`/`rule`/`body`/`element`/`atom` の各所）に呼ばれる |
| `values()` の node 経路 `.clone()` | 443 | `MappedNode.fields` から見つけた `Vec<&str>` をそのまま `clone()` して返す |
| `values()` の capture フォールバック `.collect()` | 220 | node 側に無い field を `owned_captures` から `Vec<&str>` に集約 |
| `raw_values()` の `.collect()` | 264 | STRING の raw（引用符付き）を同様に集約 |
| `to_owned()`（`scalar`/`at_most_one`/`strings` 経由） | 610 | `&str` → `String` の個別確保（`RuleDecl.name` 等、`UbnfFile` の全 leaf 文字列） |
| `decode_string()`（STRING の unescape） | 132 呼び出し（`String::new()` から push で伸長するため実確保回数はこれより多い） | escape 復号のため 1 文字ずつ `push` する専用バッファ |
| 残差（`Converter::new()` の木構築） | 約1,250〜1,350 | `children: Vec<Vec<usize>>` / `owned_captures: Vec<Vec<usize>>`（occurrence 数 1,286 分の outer Vec 自体は空 Vec の複製で確保 0 だが、実際に子や capture を持つ枠ごとに push で再確保）、`mapped`/`ids` の `BTreeMap` 挿入によるノード確保 |

いずれも「文字列の複製・中間 Vec・再帰的 clone」という issue の予想通りだが、
実装は `MappedNode`/`Converter` の定義とセットで `crates/ubnfc-front::selfhost` にあり、
この Issue のセットアップ指示（`crates/ubnfc-selfhost` 配下に留める）の外側にある。

## 2. 実施した変更（`crates/ubnfc-selfhost` 内）

- `crates/ubnfc-selfhost/src/lib.rs`: `projection::project()` に渡す `nodes: Vec<MappedNode>` を
  `Vec::with_capacity(result.lexical.len())` で事前確保。実測 node 数 764 に対し occurrence 数 1,286 を
  上限目安に使い、push による再確保（doubling で約 9 回）をほぼ 1 回に減らす。
- `crates/ubnfc-selfhost/src/capture_sites.rs`: capture site → 宣言 rule の解決を
  `CAPTURE_RULES`（62 件）の毎回線形探索から、プロセス寿命で 1 回だけ構築する `HashMap`
  （`OnceLock` でキャッシュ）に変更。`CAPTURE_RULES` 自体（内容・順序）は変えていないので、
  golden IR との一致を見る既存テスト（`capture_site_metadata_matches_unmodified_golden`）はそのまま。
- `projection.rs` 自体（各 AST ノードの `fields: Vec<(&str, Vec<&str>)>` 構築）は変更していない。
  `MappedNode` の型（`Vec<(&'a str, Vec<&'a str>)>`）は `crates/ubnfc-front` 側の定義で、
  1 個の値でも `Vec` を経由する契約になっているため、この crate 側からは
  ノードあたりの確保回数（885 回）をこれ以上減らせない。

## 3. 前後比較

`benchmarks/src/family_c.rs` をそのまま使い、`benchmarks/scripts/prepare.sh` が作る構成
（`ubnfc-front` / `ubnfc-selfhost` / sibling repo `unlaxer-ubnf` の 3 実装を 1 プロセスで交互測定）と
同じ crate を一時ディレクトリに作って、before（このブランチの変更を `git stash` で外した
origin/master 相当）と after（このブランチ）を直後に交互実行した。
`taskset -c 26`、1 コア固定、単一 thread、`UBNFC_CMP_RUNS=9`。

- host は共有機で他エージェントの並行ビルドあり。load average（1分値）: 開始 12.8 → 中間 18.1 → 終了 15.2
  （`docs/reports/2026-09-23-parser-speed-comparison.md` 計測時の load average 4.6〜5.2 よりかなり高い）。
  ms は測定間でノイズが大きく、alloc 数（load に依存しない決定的な指標）の方が信頼できる。

| 指標（`ubnf.ubnf`, AST(`UbnfFile` 射影)） | before（origin/master） | after（このブランチ） | 差分 |
|---|---:|---:|---:|
| 確保回数/parse | 7,073 | 7,065 | -8（-0.1%） |
| 確保 byte/parse | 2,673,447 | 2,630,999 | -42,448（-1.6%） |
| ms 中央値（9 run） | 7.432 | 5.898 | -1.53（ノイズ内。3 往復の min は前後とも 5.3〜5.7 ms で重なる） |
| ms 最小（9 run） | 5.866 | 5.686 | ノイズ内 |

3 往復（それぞれ 7 run）で before/after を直後に切り替えて取った参考値（load 15.8〜16.5）:

| round | before min ms | after min ms |
|---:|---:|---:|
| 1 | 5.380 | 5.344 |
| 2 | 5.671 | 5.361 |
| 3 | 5.355 | 5.425 |

確保回数・byte は決定的に減っているが、ms への寄与は測定誤差に埋もれる程度で、
**5.238 ms → ≤1 ms という目安には遠い**。

## 4. 遠因と評価

- 射影層の確保 5,600 回（7,073 - 1,473）のうち 84%（4,706 回）は `crates/ubnfc-front::selfhost::convert()`
  にある。支配的な確保源は `Converter::children(i, kind)` の毎回再フィルタ（1,788 回、木を辿るたび
  `Vec<usize>` を新規確保してキャッシュしない）と、`values()`/`raw_values()` の `Vec<&str>` 複製・収集
  （927 回）、`&str → String` の個別確保（610 回 + `decode_string` 132 回）。
- これらはこの Issue のセットアップで明示されたスコープ（`crates/ubnfc-selfhost` 配下、
  `crates/ubnfc-front` は他エージェントが同時に変更中につき対象外）の外側にある。
  生成 parser の AST 形状が複製を強制しているのではなく、**射影ロジックの実装場所**が原因なので、
  受け入れ条件の最後の項目（「生成器の AST 形状が原因なら、そこで止めて報告する」）に準じて、
  このブランチでは `crates/ubnfc-selfhost` 側で安全にできる分だけを直し、正直な到達点として報告する。
- フォローアップ: `crates/ubnfc-front::selfhost::convert()` の `children()` を
  occurrence ごとに 1 回だけ kind 別にグルーピングしてキャッシュする（現在は木を辿るたび毎回
  同じ `self.children[i]` を再フィルタしている）、`values()` の node-clone を借用に変える
  （`MappedNode` 側で `Vec<&str>` を返す代わりに slice を渡せる形にする等、型を含めた再設計が要る）
  ことで、5,600 回の大半を削れる見込み。issue を別途起票する（§5）。

## 5. テスト

- `cargo test -p ubnfc-selfhost --offline`: `tests/parity.rs`（`fixture_acceptance_and_ast_parity` /
  `self_description_ast_matches_including_every_span` 含む）と `tests/regenerate.rs`
  （固定点: IR・生成 Rust の byte 一致）を含め全緑。
- `cargo test --workspace --offline --exclude ubnfc-fuzz`: 全緑。
  `ubnfc-fuzz` の `tests/campaign.rs` はこのブランチの変更前（`origin/master` 素の状態、
  `git stash` で確認済み）から `front の discard: {"build:java": N}` / `seed 269 が build できていない`
  で失敗しており（Java ツールチェイン起因、この worktree の環境依存）、この変更とは無関係。
- `cargo clippy --workspace --offline --all-targets -- -D warnings`: warning 無し。
- `cargo fmt -p ubnfc-selfhost -- --check`: 差分無し。

## 6. 再現

```sh
UNLAXER=/home/opa/work/unlaxer-workspace/unlaxer-parser
inputs=("ubnf.ubnf=$UNLAXER/unlaxer-dsl/grammar/ubnf.ubnf" \
        "elements=$UNLAXER/rust/unlaxer-ubnf/tests/fixtures/positive/elements.ubnf" \
        "boundaries=$UNLAXER/rust/unlaxer-ubnf/tests/fixtures/positive/boundaries.ubnf" \
        "all-tokens=$UNLAXER/rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf")
# benchmarks/scripts/prepare.sh の bench-c 相当を作って
taskset -c 26 ./bench-c "${inputs[@]}"
```
