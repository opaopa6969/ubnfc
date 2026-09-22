# 位置ごとの字句 cache（統合 DFA）の実装と計測（2026-09-23）

[`2026-09-23-p4-rust-perf-m5-round4.md`](2026-09-23-p4-rust-perf-m5-round4.md)（round 4 末 fb0ed05）の続き。
**tokenizer は作らない**（D-002 のとおり UBNF の literal に単語境界は無く、順序付き選択が「どの終端を使うか」を
決めるので、token 列に確定させると受理言語が変わる）。代わりに、**同じ述語の cache** を作る:
位置ごとに統合 DFA を 1 回だけ走らせ、その位置で一致する終端の集合を保存する。

本 round の結論を先に書く: **実装し、観測が同じことを確認し、そして採らなかった**。
理由は「終端照合はもう律速ではない」で、根拠は本文の計数と A/B にある。既定は off
（`ParseOptions::lexical_cache = false`）で、逃げ道（on/off）と差分テストの軸は残してある。

## 1. 設計

### 1.1 統合 DFA に入れたもの / 入れなかったもの

| 終端 | cache する | 理由 |
|---|---|---|
| 大文字小文字を区別する ASCII の非空 literal | **する** | 統合 trie（byte DFA）1 本にまとめ、位置ごとに「一致した literal の id 集合」を bitset にする。長さは id ごとに静的（word の byte 長）なので位置ごとの長さ配列は要らない |
| CI literal（`CI('w')` / `caseSensitive=false`） | しない | `equal_ignore_case` は Java `equalsIgnoreCase` の近似で、非 ASCII の入力文字が ASCII の文字と一致し得る（`'İ'` と `'i'`）。byte trie では再現できない |
| 非 ASCII を含む literal | しない | 先頭 byte が UTF-8 継続 byte と衝突し得るため、文字境界の検査を DFA で省けない |
| 空 literal | しない | EOF を含む任意の**文字境界**で一致する。境界かどうかは byte trie では判定しない |
| Number / Identifier / Quoted / SingleQuoted | しない | 一致長だけでは再現できない**途中位置の display 記録**を持つ（`number` は SignParser / DigitParser / OneOrMore / `'.'` / EParser / ExponentParser を最大 8 回、走査位置に応じて記録する）。長さを cache しても記録の位置を得るには再走査が要る。P4 実測で number 25 回・identifier 22 回 / parse なので、記録も併せて cache する価値が無い |
| REGEX（`plan.lexical[].dfa`） | しない（P4 には無い） | 既に token ごとの DFA として生成されている。統合 DFA へ畳み込むと leftmostFirst / zeroWidth=fail の受理規則を token ごとに保つ必要があり、統合 DFA の受理集合を「token ごとの最長有効前置」に拡張しないと表せない。P4 の文法には REGEX token が無く、効果を測れないので入れていない |
| Extern token | しない（できない） | ユーザーコード。round 3 の `firstChars` による呼出し回避は入ったまま。純粋性の宣言は IR に無い（`TokenSpec.extern` に `firstChars` はあるが purity フラグは無い）ので (scanner, 位置) の cache は入れていない |
| `UNTIL` | しない | 位置だけの述語ではあるが、一致長が終端文字列までの**任意長**で、統合 DFA に入れると入力全体を走ることになる。位置ごとの専用 cache に値するほど呼ばれない（P4 complex で 0 回） |
| LOOKAHEAD / NEGATIVE_LOOKAHEAD | 内側の literal だけ経由で cache する | ゼロ幅で、本体は `literal::<true>` を呼ぶ。その literal が trie に入っていれば cache が効く。ゼロ幅であること自体は cache しない（何も走査しない） |
| EOF / EMPTY | しない | ゼロ幅。`text.len()` との比較 1 回で、cache のほうが高い |
| trivia の読み飛ばし | **する**（診断を記録しない側だけ） | `skip_i`（候補除外 §3.4 の判定位置にだけ使う読み飛ばし）は display にも主診断にも触れないので、`next_non_trivia[policy][pos]` を位置で直接引ける表にした。診断を記録する `trivia_i` は §1.4 |

### 1.2 cache の key と健全性

**主張**: UBNF の終端はすべて `(入力, 位置)` の純粋な述語である（例外は状態依存を宣言し得る Extern token
だけで、そこは cache しない）。したがって結果を位置ごとに保存しても、どの parse の受理・診断も変わらない。

key に入れてよいものは、述語が依存するものだけ:

- **位置**。呼出し側が「選ばれたカーソル位置」`state.position::<MATCH>()` を渡す。
  `04-cursor-and-diagnostics-contract.md` §1 のとおり、consumed mode は C、matchOnly mode は M を選ぶ。
  cache は選ばれた**結果の位置**で引くので、matchOnly の照合が consumed 用の entry を汚すことはない
  （同じ位置なら同じ答えになるのが、まさに述語が位置だけの関数だという主張である）。
  `consume(0)` が M を C に戻す・`MatchOnly` が M だけ進める、といった遷移は「どの位置を選ぶか」を変えるだけで、
  「その位置で終端が一致するか」を変えない。
- **trivia policy**（区切り集合で正規化した番号）と mode。§1.4。

key に入れてはいけない（＝入れていない）もの: parser の状態（consumed と matched の組、`invert`、`reset`、
transaction frame、memo の状態版、scope）。とくに `invert`（`primitive` の反転）は**照合後**に適用される
（`primitive` が `length` を受け取ってから反転する）ので、cache するのは反転前の生の一致だけでよい。

**文字境界**: `Input::starts_with` は `is_boundary(byte)` を要求する。trie を **ASCII の literal だけ**に
限ったので、trie の遷移が発火する位置の byte は必ず ASCII（最上位 bit 0）＝必ず文字境界であり、
継続 byte（`0x80..=0xBF`）や UTF-8 先頭 byte（`0xC2..`）では state 0 から遷移が無く「一致なし」になる。
`starts_with` の境界検査と一致する。

**最長一致に確定しない**: trie は受理 node を通過しても走り続け、`'var'` と `'variable'` の**両方**の bit を
立てる。どちらを使うかは文法（順序付き選択）が決める。tokenizer にしないという要求はここで満たしている。

### 1.3 表現と確保

- `bits: Vec<u64>`（1 位置あたり `LEX_WORDS` 語 = literal 数 / 64 切り上げ。P4 は literal 98 個で 2 語）と
  `epoch: Vec<u32>`（位置ごとの世代印）。遅延充填で、触れた位置だけ DFA を走らせる。
- **parse ごとの消去をしない**: 世代を 1 進めるだけ（`u32` が一周したときだけ全消去）。
  確保は既存の buffer pool（`ParseOptions::reuse_buffers`）に載せた。
- 一致長は保存しない（literal ごとに静的）。位置ごとの長さ配列を持たずに済むのは、統合 DFA に入れた終端を
  literal に限ったからで、Number のような「同じ終端でも位置ごとに長さが違う」ものを入れるなら
  `[u32; 対象終端数]` の並列配列が要る。
- 表の大きさは入力長に比例する。`LEX_CACHE_MAX_BYTES`（256 MiB）を超える入力では cache を使わない
  （`literal` は走査へ戻る）。

### 1.4 trivia

- `skip_i`（**診断を記録しない**読み飛ばし。候補除外の判定位置 `ct` の計算にだけ使う）は
  `next_non_trivia[policy][mode][pos]` の配列に入れた。key は (policy, 位置, mode) だけで、
  display にも主診断にも触れないので観測は変わらない。保存するのは開始位置からの差分
  `(consumed 側の進み, matched 側の終端)` で、「1 文字も進まない」は番兵 `u32::MAX` で表す
  （進まないときの出力 State は入力 State そのもの＝入力の `matched` を持ち越すので、差分では表せない）。
  policy は区切り集合で正規化してから引く（同じ区切りの policy は表を共有する）。
- **`trivia_i`（診断を記録する側）は cache していない**。理由は 2 つある。
  1. 記録の再生には frame（`enter` / `leave`）で囲って局所 snapshot を作る必要があり、それは既に
     `plan.trivia_cache.enabled` の経路（hash 表 + `DisplaySnapshot` の `merge`）として存在する。
     P4 では front の plan がこれを無効にしている。
  2. 計数すると、**P4 complex で `trivia_i` は 1 parse 1,740 回呼ばれるが、実際に位置が進むのは 70 回（4%）**。
     進まない 96% の経路は「先頭 byte が区切りの先頭 byte 集合に無い」→ `display.failures` →`break` の
     3 命令程度で、表を引くほうが高い。`next_non_trivia` を入れても縮むのは残り 4% だけになる。
  この判断は計測に基づくもので、契約上の制限ではない。`trivia_i` の記録も位置だけの関数なので、
  `plan.trivia_cache` の仕組み（frame + snapshot + `merge`）で cache すること自体は健全である。

### 1.5 逃げ道

- `ParseOptions::lexical_cache`（既定 false）。true で literal の統合 DFA cache と `skip_i` の
  `next_non_trivia` を使う。false は従来どおり毎回走査する参照経路。
- 生成 driver は環境変数 `UBNFC_LEXICAL_CACHE=1` で corpus 全体を cache 有効で回せる。
- ベンチは `UBNFC_BENCH_OPTS=lexcache=1`。

## 2. 前提の検証: 終端照合はもう律速ではない

指示にあった見積り（complex.tiny 332 byte で literal 約 1,570 + primitive 約 1,616、byte あたり約 10 回）は
**round 3 / round 4 より前の値**である。fb0ed05 の生成物で計り直すと:

| 計数（complex.tiny 332 byte、1 parse、認識） | 回数 |
|---|---:|
| `literal` | 548 |
| `primitive`（literal 経由を含む） | 595 |
| `number` / `identifier` / `quoted` / `any` / `until` | 25 / 22 / 0 / 0 / 0 |
| `scan`（Extern） | 43 |
| 終端照合の合計 | **595**（byte あたり 1.8） |
| `display.failures` | 5,400 |
| `display.reach` | 5,528 |
| 主診断 `push` / `join` / `rule` | 5,947 / 4,147 / 2,334 |
| `trivia_i` 呼出し / うち位置が進んだもの | 1,740 / 70 |

§3.4 の候補除外と §3.4 の左因数分解（round 2 / 3）が、同じ位置での重複した終端試行を既に取り除いている。
残った 595 回のうち **541 が (終端, 位置, mode) として相異なる**（cache 命中率 9%）。
位置だけで見ると **相異なる位置は 112**（595 回 / 112 位置 = 5.3 回）なので、統合 DFA を位置ごとに 1 回
走らせる形にすれば命中率は 81% になる。実装後の実測も同じで、complex で probe 548 / DFA 実行 91（命中 83%)、
complex-x64 で probe 33,812 / DFA 実行 5,509（命中 84%）。**cache の設計としては意図どおり効いている。**

問題は 1 回あたりの単価のほうで、生成物を一時的に書き換えて `literal` の `starts_with` を 5 回に増やし、
増分から 1 回の走査の費用を見積もると **literal 548 回の走査は complex 認識（0.22 ms、当時の負荷での値）の
約 4%** にとどまる（`black_box` を挟んだ上限寄りの見積り）。P4 の literal は呼出し回数の上位が
`'('` 53・`')'` 53・`':'` 31・`'['` `']'` 16 と**1 文字**が大半で、`starts_with` が 1 byte の比較で終わる。
cache を引く側（範囲検査 + 世代照合 + bitset の 1 bit 取り出し）のほうが命令数が多い。

## 3. A/B

共用ホスト（32 core、他エージェントの corpus / ベンチが並走）。`taskset -c 28` で 1 コアに固定し、
**baseline binary（merge-base fb0ed05）/ 本 branch の cache off / 本 branch の cache on** を
同一セッションで交互に 9 回（round 開始時の load average 20〜48）。表は 5 サンプル中央値の 9 round 中央値、
括弧内は全 run の最小値（干渉に強い）。ms/parse。

| 入力 | mode | baseline (fb0ed05) | cache off | cache on | on/off（中央値） | on/off（最小値） |
|---|---|---:|---:|---:|---:|---:|
| complex (332 B) | 認識 | 0.163 (0.153) | 0.168 (0.155) | 0.175 (0.157) | +4.2% | **+1.3%** |
| complex | AST | 0.220 (0.207) | 0.218 (0.210) | 0.232 (0.213) | +6.4% | **+1.4%** |
| complex-x4 (1,293 B) | 認識 | 0.685 (0.597) | 0.639 (0.600) | 0.647 (0.612) | +1.3% | **+2.0%** |
| complex-x4 | AST | 0.811 (0.759) | 0.816 (0.742) | 0.838 (0.770) | +2.7% | **+3.8%** |
| complex-x16 (5,179 B) | 認識 | 2.785 (2.441) | 3.124 (2.486) | 2.898 (2.564) | −7.2% | **+3.1%** |
| complex-x16 | AST | 3.351 (2.916) | 3.558 (2.927) | 3.388 (3.089) | −4.8% | **+5.5%** |
| complex-x64 (20,923 B) | 認識 | 13.009 (11.118) | 13.750 (11.023) | 13.323 (11.458) | −3.1% | **+3.9%** |
| complex-x64 | AST | 18.292 (15.592) | 17.347 (14.451) | 19.503 (15.548) | +12.4% | **+7.6%** |

中央値は負荷の揺れで符号が入れ替わる（x16 / x64 の認識は cache on のほうが速く見える round がある）。
**最小値は全ての入力・両モードで cache on が遅い**方向に揃い、しかも入力が大きいほど悪化する
（+1.3% → +7.6%）。表が入力長に比例して大きくなり、parser の作業集合を追い出すためと読める。

`baseline` と `cache off` の差（complex 認識の最小値 0.153 → 0.155）は、`literal` に足した
`self.lex_enabled && lit != u32::MAX` と `skip_i` に足した表引きの分岐（cache off のときも通る）である。
1% 程度で雑音に埋もれるが、符号は一貫して遅い側。

### 計数（cache on。1 parse あたり）

| 入力 | 終端照合（`primitive`） | cache 参照 | 統合 DFA 実行（miss） | 命中 | 命中率 | `skip` 命中 / miss |
|---|---:|---:|---:|---:|---:|---:|
| complex | 595 | 548 | 91 | 457 | 83.4% | 114 / 38 |
| complex-x4 | — | 2,132 | 349 | 1,783 | 83.6% | 438 / 146 |
| complex-x16 | — | 8,468 | 1,381 | 7,087 | 83.7% | 1,734 / 578 |
| complex-x64 | — | 33,812 | 5,509 | 28,303 | 83.7% | 6,918 / 2,306 |

`literal` 548 回のうち 548 回すべてが cache 対象（P4 の literal は 98 個すべて ASCII・大文字小文字を区別）。
統合 DFA は 91 位置でしか走らず、残り 457 回は bitset の 1 bit を読むだけ。**cache としては設計どおり効いている**。
それでも遅いのは、置き換えた走査（1〜2 byte の `starts_with`）が cache の参照より安いからである。

## 4. crossover（どこまで小さい入力なら割に合うか / 表の費用）

buffer pool（`ParseOptions::reuse_buffers`、既定 true）の有無で分けて 7 回ずつ交互に測った
（中央値（最小値）、ms/parse）。

| 入力 | mode | pool + off | pool + on | on/off | pool なし + off | pool なし + on | on/off |
|---|---|---:|---:|---:|---:|---:|---:|
| complex | 認識 | 0.162 (0.154) | 0.163 (0.156) | +0.6% | 0.177 (0.167) | 0.185 (0.174) | +4.5% |
| complex | AST | 0.216 (0.206) | 0.216 (0.212) | +0.0% | 0.237 (0.226) | 0.243 (0.229) | +2.5% |
| complex-x4 | 認識 | 0.688 (0.596) | 0.628 (0.605) | −8.7% | 1.105 (1.056) | 1.149 (1.082) | +4.0% |
| complex-x4 | AST | 0.818 (0.744) | 0.781 (0.745) | −4.5% | 0.788 (0.763) | 0.851 (0.768) | +8.0% |
| complex-x16 | 認識 | 2.541 (2.383) | 2.570 (2.400) | +1.1% | 4.773 (4.332) | 4.851 (4.475) | +1.6% |
| complex-x16 | AST | 3.388 (2.854) | 3.139 (2.895) | −7.3% | 3.035 (2.919) | 3.334 (2.900) | +9.9% |
| complex-x64 | 認識 | 11.749 (10.670) | 12.368 (10.968) | +5.3% | 28.462 (25.869) | 27.618 (25.904) | −3.0% |
| complex-x64 | AST | 16.683 (13.938) | 17.281 (14.915) | +3.6% | 31.478 (15.772) | 33.841 (27.496) | +7.5% |

確保（1 parse、GlobalAlloc の計数）:

| 入力 | mode | pool あり allocs / bytes（off → on） | pool なし allocs / bytes（off → on） |
|---|---|---|---|
| complex | 認識 | 354 → 354 / 36,243 → 36,243 | 408 → 412 / 641,079 → 655,731 |
| complex | AST | 772 → 772 / 132,288 → 132,288 | 913 → 917 / 757,264 → 771,916 |
| complex-x4 | 認識 | 1,265 → 1,265 / 68,466 → 68,466 | 1,325 → 1,329 / 2,282,162 → 2,339,098 |
| complex-x16 | 認識 | 4,891 → 4,891 / 203,549 → 203,549 | 4,957 → 4,961 / 8,935,013 → 9,162,933 |
| complex-x64 | 認識 | 19,370 → 19,370 / 749,309 → 749,309 | 19,444 → 19,448 / 45,768,085 → 46,688,741 |

- 表の費用は **入力 1 byte あたり 44 byte**（P4: bitset 2 語 = 16 B + epoch 4 B + `next_non_trivia`
  1 policy × 2 mode × 12 B = 24 B）。complex で +14,652 byte、x64 で +920,656 byte、確保回数は +4。
  pool に載せれば定常状態では確保 0 回（表の中身は世代印を進めるだけで消去しない）ので、
  **pool 有効なら「小さい入力で確保が効いて損」という形の crossover は起きない**。
  pool を切ると小さい入力でも +2.5〜4.5% になる。
- **時間の crossover は（この文法・この入力では）存在しない**。cache on は最小値で常に遅く、
  しかも入力が大きいほど差が開く。損益分岐に必要なのは「1 回の走査が cache 参照より高い」ことで、
  それには (a) 試行される literal が長い（4 byte 以上）か、(b) 同じ位置で同じ終端が何度も試される、
  のどちらかが要る。(b) は §3.4 の候補除外と左因数分解が既に取り除いている。
- (a) を確かめるため、統合 DFA に入れる literal を **3 byte 以上に限った版**（`MIN_LITERAL_BYTES = 3`）も
  測った。cache 参照は 548 → 203、DFA 実行は 91 → 42 に減り、complex の最小値は 0.154 → 0.155（認識）、
  0.206 → 0.206（AST）で**損は消えるが得も無い**（中央値では +3.1% / +3.8%）。
  P4 の literal 試行は 63%（548 中 345）が 1〜2 byte で、`starts_with` が 1 命令の比較で終わる。
  採用版は「全 literal を入れる」設計どおり `MIN_LITERAL_BYTES = 1` のままにしてある。

## 5. corpus / 契約の検証

- `cargo test --workspace --offline` 全緑（`crates/ubnfc-rt-template/tests/lexcache.rs` に
  `LexCache` / `SkipCache` の単体テストを追加。世代印で parse をまたいで無効化すること、
  表の外は `None` を返して呼出し側が走査へ戻ること、policy / mode / 位置が混ざらないこと）。
- `cargo clippy --workspace --all-targets --offline -- -D warnings` / `cargo fmt --all -- --check` 緑。
  生成 crate 側の clippy（`crates/ubnfc-rust/tests/review.rs` が `-D warnings` で回す）も緑
  （literal も trivia policy も無い文法で `LEX_WORDS > 0` が `absurd_extreme_comparisons` に当たったので
  `!= 0` にした）。
- 差分テスト `predict_matches_reference_path_on_truncated_and_corrupted_inputs` を
  **(predict, factor, lexical_cache) の 7 組合せ**に拡張（cache on/off の軸を追加）。
  16 fixture × 切り詰め・1 文字破壊 × memo on/off で、全て参照経路（3 つとも off）と結果・診断が一致。
- selfhost 固定点（`crates/ubnfc-selfhost/tests`）緑、`src/generated` を再生成して commit。
  `examples/p4-rust/scripts/generate.sh --check` で P4 生成物 byte 一致。
  handwritten-target の golden（Operators / Power、`crates/ubnfc-rust/tests/generated.rs`）byte 一致。
- corpus（既定 = cache off）: `docs/reports/2026-09-23-rust-corpus-lexcache.{md,json}`。
  **Pass 1292 / 期待拒否 Pass 30 / Known 28 / Fail 0 / 未観測 75**。
  `2026-09-23-rust-corpus-m5r4.md` との差は build 時間と generator / front の SHA-256 行だけ（系統別の表は完全一致）。
- corpus（**cache on**、`UBNFC_LEXICAL_CACHE=1` で driver の `ParseOptions::lexical_cache` を立てる）:
  同じ corpus を cache 有効で回した結果は
  **cache off の報告と完全一致**（build 時間の行だけが違う。系統別・loader 系統別・文法別の表、
  Fail 詳細、未観測項目まで同一）。この corpus は組込み token 全種・CI・REGEX・NEGATION・
  CHAR_RANGE・UNTIL・先読み・trivia policy を通るので、§1.2 の健全性主張の実測の裏付けになる。
- 契約: `04-cursor-and-diagnostics-contract.md` §1（cursor）— cursor の遷移コードは無変更。
  cache は「選ばれたカーソル位置」で引くので `consume(0)` / `MatchOnly` の遷移とは独立（§1.2）。
  §2（診断）— display / 主診断の記録経路は 1 行も変えていない（cache が置き換えたのは `starts_with` の
  真偽値と、診断を記録しない `skip_i` の結果だけ）。記録回数の予算も変わらない。
  `03-design-v1.md` §3.2（terminals）/ §3.5（trivia）/ §5（許容手法）— 受理範囲は不変、
  最長一致に確定しない（§1.2）。
- `ubnfc-java`、`crates/ubnfc-fuzz`、corpus / oracle 期待値、unlaxer-parser、tinyexpression は触っていない。

## 6. 未達・残り

1. **既定 off のまま採用しなかった**。P4 の文法と入力では、終端照合が認識時間の 1 桁 % しかなく
   （§2）、cache を引く費用（範囲検査・世代照合・bitset）が 1〜2 byte の `starts_with` より高い。
   統合 DFA の命中率（83〜84%）は設計どおりで、命中率の不足が原因ではない。
2. **REGEX token を統合 DFA に畳み込んでいない**。P4 の文法に REGEX token が無く、効果を測れない。
   畳み込むには「位置ごとの一致集合」に加えて token ごとの一致長（leftmostFirst の最長有効前置）が要るので、
   位置ごとに `[u32; 対象 token 数]` の並列配列が要る。同じ理由で Number / Identifier / Quoted も入れていない。
3. **Extern token の (scanner, 位置) cache は入れていない**。IR の `TokenSpec.extern` に純粋性の宣言が無い
   （`firstChars` はある）。宣言を足すのは front の変更になるので、本 round では触っていない。
4. **診断を記録する `trivia_i` は cache していない**（§1.4 の 2 つの理由。とくに「進むのは 4% だけ」）。
   一般の cache 機構としては `plan.trivia_cache` が既にあり、P4 では front の plan が無効にしている。
   front の判断を変えるかどうかは別 issue。
5. **CI literal と非 ASCII literal を統合 DFA に入れていない**（§1.1）。CI を入れるには code point 単位の
   DFA と `equal_ignore_case` の折り畳みが要る。
6. **計測は共用ホスト**（32 core、load average 13〜85。他エージェントの corpus / ベンチが並走）。
   `taskset -c 28` で 1 コアに固定し、同一セッション内で交互に走らせて中央値と最小値を採ったが、
   専有ホストでの再現は未実施。round 4 と同じ条件の制約。
7. 差分テストは (predict, factor, lexical_cache) の 7 組合せ × 16 fixture × 切り詰め・破壊 × memo on/off。
   debug build で約 260 秒（round 4 の約 2 倍）。組合せを減らす余地はある。

## 7. 再現

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
bash examples/p4-rust/scripts/generate.sh --check
# A/B（cache off / on を 1 つの binary で切り替える）
cargo build --release --offline -p ubnfc-p4-rust --example bench
UBNFC_BENCH_OPTS=lexcache=0 taskset -c 28 ./target/release/examples/bench complex complex-x4 complex-x16 complex-x64
UBNFC_BENCH_OPTS=lexcache=1 taskset -c 28 ./target/release/examples/bench complex complex-x4 complex-x16 complex-x64
# corpus（既定 = cache off）
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust \
  --report docs/reports/2026-09-23-rust-corpus-lexcache.md --json docs/reports/2026-09-23-rust-corpus-lexcache.json
# corpus（cache on。報告は cache off と完全一致する）
UBNFC_LEXICAL_CACHE=1 cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust \
  --report /tmp/corpus-on.md --json /tmp/corpus-on.json
```
