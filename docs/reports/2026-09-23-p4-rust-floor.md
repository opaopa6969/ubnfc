# P4 Rust 生成 parser を床まで詰める（2026-09-23）

[`2026-09-23-p4-rust-profile.md`](2026-09-23-p4-rust-profile.md) が順位を付けた残りの候補を、安い順・安全な順に実装した報告。
結論を先に書く:

- **採用 2 件**（候補4 出現表と value span の opt-in、候補2 `const DIAG` 単相化）で、
  master `9e97776` から **complex 認識 −20.5%（0.083→0.066 ms）／complex AST −30.2%（0.126→0.088 ms）／
  x64 認識 −18.4%（5.58→4.55 ms）／x64 AST −23.3%（7.68→5.89 ms）**。
  確保は complex AST が **738 回 128 KB → 536 回 60 KB**、x64 AST が **44,786 回 6.63 MB → 32,234 回 2.46 MB**。
- **AST 経路は profile が出した現実的な床（complex 0.085〜0.100 / x64 5.3〜6.2 ms）の中に入った**。
  認識は床の上端をわずかに超える（complex 0.066 対 0.065、x64 4.55 対 4.3）。§5。
- **却下 1 件**（候補3 認識での event arena 削減）。**front の到達解析には手を付けていない。**
  先に「解析なしで取れる上限」を測ったら雑音以下だったため。§3。
- 候補5（AST の arena / SoA 化）は指示どおり着手していない。最終値での価値は §6。
- **driver 経路は退行していない**。AST 解析で **complex −6 µs（−2.8%）／x64 −407 µs（−2.9%）**、
  確保 byte は complex −25%・x64 −29%。corpus 観測は両 backend で不変（§4）。

## 0. 計測条件

| 項目 | 値 |
|---|---|
| 基準 | master `9e97776`（profile round 終了時点）。branch `perf/rust-floor` |
| rustc / cargo | `rustc 1.98.1` / `cargo 1.98.1`、`--release --offline` |
| host | 32 core、WSL2。共有機で他エージェントが並走する |
| 固定 | `taskset -c 27`。全 variant を**同一セッションで交互に 9 round**（1 round = 各 variant 1 回、bench は内部で warmup 後 5 sample の中央値） |
| 集計 | round 中央値と**全 run の最小値**。Δ は最小値どうし |
| load average | **最終セッション（§1〜§3 の表）は 3.5〜10.3 の静かな窓**。途中の探索セッションは 8〜22 で、そちらは符号の確認にだけ使う |
| 雑音帯 | profile §2.1 と同じ: complex ±1.5%、x64 認識 ±3.5%、x64 AST ±5.5%。対照（変わらないはずの組合せ）で毎回確認する |
| 回数 | `examples/p4-rust` の `counters` feature（計時は歪むので回数だけ使う） |
| 確保 | bench の GlobalAlloc 計数（計時とは別 parse） |

variant:

| 名前 | 中身 |
|---|---|
| `base` | master `9e97776` |
| `c4` | + 候補4（出現表と value span の opt-in、D-026） |
| `c4c2` | + 候補2（`const DIAG` 単相化、D-027）。**本 branch の最終形** |
| `c3` | `c4c2` + 認識で `Event::Rule` / `Event::Token` を置かない（候補3 の「解析なしで取れる分」） |
| `noevent` | `c4c2` + 認識で event を一切置かない。**受理は同じだが scope 結果が壊れる上限**（profile §2.2 と同じ性質） |
| `drvold` / `drvnew` | driver の AST 解析の設定。old = `diagnostics+出現+value span`、new = `diagnostics+value span` |

## 1. 候補4（採用）: 出現表と value span を opt-in にする — D-026

`ParseOptions` に `occurrences` と `value_spans`（どちらも既定 false）を足し、`finish` の
**arena 3 走査のうち 2 つ**（出現収集 `occurrences`、`ast.collect` による value span）を要求時だけにした。

**誰が要るか**（実装前に調べた。§0 の順序どおり「先に決める」）:

| 利用者 | `captures` | rule / token 出現 | `value_spans` | 対応 |
|---|---|---|---|---|
| harness driver（corpus 観測） | `lexical` のときだけ出力 | `lexical` のときだけ出力 | AST が出たら常に出力 | AST 解析で `value_spans: true`。出現は `lexical: true` が含意するので追加要求は不要 |
| self-host bootstrap（`crates/ubnfc-selfhost/src/lib.rs`） | 読む | rule 出現を読む | 読まない | `lexical: true` のまま**無変更** |
| LSP 的な投影 | 読む | 読む | 読む | `occurrences` / `value_spans` を立てる |
| 「解析して AST が欲しい」だけの利用者 | 読まない | 読まない | 読まない | **何も払わなくなる（本案の得）** |
| 差分 fuzzing（`crates/ubnfc-fuzz/src/observe.rs`） | `captures` を比較 | 比較しない（`lexical` は無視 metadata） | 比較しない | driver 経由なので不変 |

`lexical: true` は「出現表そのものが目的」なので要求を含意させた。これで self-host と driver は無変更で通り、
**既定で空になるのは「AST だけを要求した parse の `captures`」と「`value_spans`」の 2 つだけ**になる。

**D-023 との違い**: D-023 は結果を見れば診断が要るかが分かるので決定的再解析で埋められる。
出現表と value span は**結果からは要否が判定できない**ので、自動の再解析は無い。要る側が要求する。
これは決定行 D-026 に明記した。

### A/B（最終セッション、9 round 交互、Δ は最小値どうし）

| 入力 / mode | `base` 中央値 / **最小** | `c4` 中央値 / **最小** | Δ | 確保 base → c4 |
|---|---:|---:|---:|---|
| complex 認識 | 0.0880 / **0.0830** | 0.0850 / **0.0830** | **0 µs**（対照） | 320 / 31,769 B（不変） |
| complex AST | 0.1300 / **0.1260** | 0.1110 / **0.1090** | **−17 µs（−13.5%）** | **738 → 536 回、127,814 → 59,583 B（−53%）** |
| x64 認識 | 5.757 / **5.580** | 5.762 / **5.540** | −40 µs（−0.7%、対照） | 18,706 / 729,715 B（不変） |
| x64 AST | 9.927 / **7.681** | 8.184 / **7.071** | **−610 µs（−7.9%）** | **44,786 → 32,234 回、6,628,353 → 2,460,684 B（−63%）** |

認識は 2 つとも雑音帯の中（対照として働いている。認識では元から出現収集をしていない）。
AST の −17 µs は profile の見積り（出現収集 7 µs + value span 11 µs = 18 µs）と 1 µs 以内で一致する。

**規模**: `templates/api.rs` / `templates/finish.rs` / `templates/driver.rs` で 60 行ほど。
`occurrences()` の中の `build_ast || lexical` 条件は、関数自体が要求時しか呼ばれなくなったので無条件化した
（到達し得る設定での振る舞いは同じ）。

## 2. 候補2（採用）: `const DIAG` による診断配管の単相化 — D-027

`Session<'a, const DIAG: bool>` にし、入口が `ParseOptions::diagnostics` を見て `Session::<false>`（fast mode）か
`Session::<true>`（診断モード）を選ぶ。D-023 の再解析も `Session::<true>` で走る。
`Diagnostics` / `DisplayDiagnostics` への呼出しは Session 上の `diag_*` / `display_*` wrapper を通し、
`DIAG == false` では定数分岐として消える。候補除外の失敗再生 `guard_e*` の早期 return と
`finish_checked` の escalate 判定も `options.diagnostics` から `DIAG` に替えた。

**profile §4.2 が最大の失敗要因に挙げた i-cache の代償は出なかった。** binary は **4.64 → 7.24 MB（+56%）**に
なったが、4 つの組合せ全てで中央値も最小値も改善した。round 4 の `limit` inline（+5 µs 悪化）と違い、
単相化は**熱い関数を太らせず、fast 側では細くする**（記録の呼出しと引数準備が消える）ためと考えられる。

### 回数（`counters`、complex 認識、1 parse あたり）

| 計数 | `c4`（= master の配管） | `c4c2` | 意味 |
|---|---:|---:|---|
| 主診断 `join` | **3,429** | **0** | `rt/diag.rs` 内部の計数。実際の呼出しが消えた |
| 主診断 `rule` | **759** | **0** | 同上 |
| 主診断 `push` | 0 | 0 | fast mode は元から記録しない |
| display の frame 走査回数 | **3,226** | **0** | `rt/diag.rs` 内部の計数 |
| display 呼出し位置 `reach` / `failures` / `enter` / `leave` | 3,057 / 2,825 / 590 / 590 | 同数 | **計数フックは生成物の呼出し位置に挿す**ので数は変わらないが、単相化後は中身が空になる |
| `event` / `join` / `combine` / `mark` / `restore` | 1,308 / 1,408 / 1,712 / 1,975 / 2,750 | 同数 | 認識の作業量そのものは変えていない |

**回数が減ったのは診断側だけで、認識の作業量（rule 評価 759、`mark`/`restore` 4,725、memo 1,349）は 1 つも動いていない。**
つまり得は「同じ作業を、記録の配管なしで行う」分である。

### A/B（最終セッション、9 round 交互）

| 入力 / mode | `c4` **最小** | `c4c2` **最小** | Δ（対 `c4`） | Δ（対 `base`） |
|---|---:|---:|---:|---:|
| complex 認識 | 0.0830 | **0.0660** | **−17 µs（−20.5%）** | −17 µs（−20.5%） |
| complex AST | 0.1090 | **0.0880** | **−21 µs（−19.3%）** | −38 µs（−30.2%） |
| x64 認識 | 5.540 | **4.554** | **−986 µs（−17.8%）** | −1,026 µs（−18.4%） |
| x64 AST | 7.071 | **5.894** | **−1,177 µs（−16.6%）** | −1,787 µs（−23.3%） |

profile §2.2 の上限（主診断だけを定数化して complex 認識 −13 µs、display を消して −5 µs、合計 18 µs）と
ほぼ一致する（実測 −17 µs）。**この案は「上限どおり取れた」数少ない例である。**

**規模**: emitter 側 90 箇所の呼出し名の置換（機械的）＋ `templates/runtime.rs` の wrapper 13 個、
入口 2 つと `entry_step` の単相化、`src/driver.rs` と `examples/p4-rust/build.rs` の切り出し目印の更新。
生成物は byte 一致で再生成できる（`tests/regenerate.rs` / `crates/ubnfc-selfhost/tests/regenerate.rs`）。

## 3. 候補3（却下）: 認識で event arena を削る

**front の到達解析（`crates/ubnfc-front/src/analysis/**`、200〜400 行）には着手していない。**
着手前に profile §8.2 が指示する「削減率の計数」と、**解析なしで取れる分の実測**を先に行い、
どちらも「取り分が無い」と出たためである。

### 3.1 event の種別内訳（新規計測、1 parse あたり）

| 種別 | complex | x64 | 認識で読む主体 |
|---|---:|---:|---|
| `Join` | 208 | 12,681 | scope（`scope_captures`）・recovery |
| `Values` | 218 | 13,070 | 同上 |
| `Capture` | 279 | 17,478 | 同上（**scope 効果が読む本体**） |
| `Rule` | **319** | **19,723** | **誰も読まない**（AST の `build_values` と出現収集だけ） |
| `Token` | **284** | **17,294** | **誰も読まない**（AST・lexical だけ） |
| 合計 | 1,308 | 80,246 | — |

つまり **認識だけの parse では event の 46%（603 / 1,308）が最初から不要**で、これは
front の解析を一切必要としない（`options.build_ast` / `options.lexical` を見るだけ）。
`Event::Rule` は `child` をそのまま返せば部分木の到達性は保たれる。

### 3.2 実測

| variant | complex 認識 | complex AST | x64 認識 | x64 AST | arena entry（complex 認識） |
|---|---:|---:|---:|---:|---:|
| `c4c2`（採用形） | 0.0660 | 0.0880 | 4.554 | 5.894 | 1,308 |
| `c3`（Rule / Token を置かない） | 0.0650（**−1 µs、−1.5%**） | 0.0900（+2 µs、対照） | 4.472（−82 µs、**−1.8%**） | 5.901（+7 µs、対照） | **616** |
| `noevent`（event を一切置かない。**scope 結果が壊れる上限**） | 0.0560（−10 µs、−15.2%） | 0.0900（+2 µs、対照） | 3.654（−900 µs、−19.8%） | 5.993（+99 µs、対照） | 1 |

**`c3` は 4 つとも雑音帯の中**（complex ±1.5%、x64 認識 ±3.5%）。
**event の 46% を消して測れる差が出ない**のだから、残り 54%（`Capture` / `Values` / `Join`）の
**一部**しか消せない front 到達解析が 12 µs を取ることはあり得ない。

`noevent` の −10 µs / −900 µs は**到達解析では取れない**。内訳が確保の計数で分かる:
`noevent` の complex 認識は確保が **320 → 43 回 / 31,769 → 20,704 byte**、x64 認識は **18,706 → 1,686 回 /
729,715 → 99,264 byte** に落ちている。これは arena の push が消えたからではなく、
**`scope_captures` が capture を 1 つも見つけられず、scope の宣言・参照の `String` と `ScopeStore` の
更新が丸ごと起きなくなった**ためである（＝受理は同じでも scope 結果が壊れているという上限の中身）。
保守的な到達解析は、まさにその scope 経路を残さねばならないので、この分は取れない。

**判定: 却下。** profile が上限として挙げた 12 µs は、その大半が「scope の仕事をしない」分であり、
event arena の push 自体には（字句 cache・`notokenevent` と同じく）取り分が無い。
**`crates/ubnfc-front` は一切変更していないので、IR golden・selfhost 生成物・Java backend への影響は無い。**

## 4. driver 経路（退行していないことの確認）

driver は corpus 観測のために出現表と value span を出す。D-026 で要求側になったので、
**AST 解析だけが `value_spans: true` を出し、出現は `lexical: true` が含意する分だけ**にした
（prefix の認識解析は `lexical` のときしか出現を出さないので、要求も出さない）。

| 入力 / mode | `drvold`（出現+value span+診断） **最小** | `drvnew`（value span+診断） **最小** | Δ | 確保 byte |
|---|---:|---:|---:|---|
| complex AST | 0.2120 | **0.2060** | **−6 µs（−2.8%）** | 132,290 → 98,844（−25%） |
| x64 AST | 14.166 | **13.759** | **−407 µs（−2.9%）** | 6,647,949 → 4,707,751（−29%） |

**退行なし**（わずかに速い）。診断モードは元から fast mode の 2 倍以上かかるので絶対値は大きい。
prefix（認識）解析は設定が変わらないので、変化は候補2 の診断側インスタンスの分だけである。

### corpus（指示の期待値と完全一致）

| backend | Pass | 想定拒否 | known | Fail | 未観測 | 期待 |
|---|---:|---:|---:|---:|---:|---|
| Rust | **1292** | **30** | **28** | **0** | **75** | 一致（`2026-09-23-rust-corpus-floor.md`） |
| Java | **1306** | **28** | **28** | **0** | 63 | 一致（`2026-09-23-java-corpus-floor.md`） |

## 5. 到達点と床

| 入力 / mode | master `9e97776`（最小） | 本 branch（最小） | 改善 | profile §5 の床の見込み | 判定 |
|---|---:|---:|---:|---:|---|
| complex 認識 | 0.083 ms | **0.066 ms** | −20.5% | 0.055〜0.065 | 上端+1 µs |
| complex AST | 0.126 ms | **0.088 ms** | −30.2% | 0.085〜0.100 | **床の中** |
| complex-x64 認識 | 5.58 ms | **4.55 ms** | −18.4% | 3.6〜4.3 | 上端+6% |
| complex-x64 AST | 7.68 ms | **5.89 ms** | −23.3% | 5.3〜6.2 | **床の中** |

**床の見積りは候補 4.1〜4.4 を「全部」入れた値だったが、実際には 2 件（候補4・候補2）だけで
AST の床に入り、認識も床の上端に並んだ。** 差額は候補3 が見積り（−12 µs / −904 µs）どおりには
出ないことで説明できる（§3）。したがって**この設計での認識の現実的な床は
complex 0.065 ms / x64 4.5 ms 前後**に更新される（profile の 0.055 / 3.6 は候補3 の 12 µs を当てにしていた）。

長期目標（complex AST 0.02 ms、x64 AST 1.5 ms）には**依然として届かない**。profile §5 の 3 つの根拠は
どれも変わっていない: 1 byte あたり 111 回の runtime 操作、生き残る node 1 つあたり rule 評価 5.8 回の投機、
候補除外は既に 209 選択肢中 105 個を除外済み。本 round が消したのは**作業ではなく配管**である。

## 6. 残りの差は何でできているか

profile §2.3 の帰属を本 round の結果で更新すると、complex 認識 66 µs の内訳はこうなる:

| bucket | µs | 根拠 |
|---|---:|---|
| **rule / expression 関数呼出し 3,158、`mark`/`restore` 4,725、memo `lookup`/`store` 1,349、`join`/`combine` 3,120、終端照合 685、`fail` 457、効果 472** | **約 56** | 差分。**切ると受理が変わる機構の束で、この host に `perf` が無いのでまだ割れていない** |
| event arena の push（1,308 回） | 約 1 | §3.2（`c3` の実測 −1 µs） |
| scope 効果（`scope_captures` の走査 + 宣言・参照の `String` + `ScopeStore`） | 約 9 | §3.2（`noevent` −10 µs から arena push 分を引いた残り） |

AST の増分 22 µs（88 − 66）は、profile §3.2 の「AST 構築 22 µs」とそのまま一致する
（出現収集 7 µs と value span 11 µs は D-026 で既定から外れた）。

**候補5（AST の arena / SoA 化）の価値は、この 22 µs のうち確保に由来する分だけ**である。
本 round 後の complex AST は 536 回 / 59,583 byte 確保していて、認識分（320 回 / 31,769 byte）を引くと
**AST 増分は 216 回 / 27,814 byte（node 1 個あたり 1.6 回 / 212 byte）**。D-026 の前は 3.2 回 / 733 byte
だったので、**候補5 が置き換える相手は既に半分以下になっている**。公開型 `Ast` の破壊的変更
（`03-design-v1.md` §6 の結果 API と canonical JSON golden に直撃）と引き換えに取れるのは
22 µs の一部でしかなく、**本 round の数字は「やらない」と言っている**。

次に効くのは、profile §5 が挙げた設計側の 2 つだけである:
(a) 「決定的に降りられる」ことを front が証明して rule 入口検査・memo・`mark`/`restore` を外す、
(b) 優先順位の連鎖を precedence climbing へ lowering して降下段数を減らす。
どちらも上の 56 µs の束に直接効く。**その前に `perf` が使える host を 1 台確保するのが依然として最優先**である。

## 7. 検証

- `cargo test --workspace --offline` 緑。`cargo clippy --workspace --all-targets --offline -- -D warnings` 緑。`cargo fmt --all -- --check` 緑。
- `examples/p4-rust/tests/fixtures.rs`:
  - `predict_matches_reference_path_on_truncated_and_corrupted_inputs`（predict / factor / lexical_cache の
    参照経路比較と fast / 診断モードの同値）は**出現表と value span も要求した状態で**回している
    （`observed()` を追加）。単相化と opt-in の両方をこの軸が守る。
  - **新規** `occurrences_and_value_spans_are_opt_in_without_changing_anything_else`:
    既定で空になるのが出現表と value span だけで、AST・`node_spans`・診断・scope・recovery・統計が
    要求時と完全一致することを、全 fixture × `build_ast` × `lexical` で固定する。
- selfhost の固定点（`generated_frontend_reaches_the_ir_and_rust_fixed_point`）緑。
  生成物は `examples/p4-rust/scripts/generate.sh --check` と両 `tests/regenerate.rs` で byte 一致。
- `examples/p4-*` の golden は byte 一致（`java_goldens_agree_except_single_quoted_string_values` 含む）。
- corpus は §4 の表のとおり両 backend で期待値と一致。
- `cargo test -p ubnfc-fuzz` 緑（固定 seed の差分 campaign）。**注意**: 新しい worktree では
  `ubnfc-java/target` が無く Java driver の build が失敗して `build:java` で discard される。
  先に `cd ubnfc-java && mvn -o -DskipTests package` が要る（本 round で 1 度踏んだ）。
- `ubnfc-java` は IR の意味を変えていないので無変更（front にも触れていない）。

## 8. 次にやるなら

1. **`perf` が使える host**。残り 56 µs（認識の 85%）を割る手段が無いままでは、次の案は当て推量になる。
2. §6 の (a)（決定的降下の証明）と (b)（precedence climbing への lowering）。どちらも front の解析追加。
3. **やらないと決めたもの**: 候補3（event arena の到達解析。§3）、候補5（AST の arena / SoA 化。§6）、
   位置ごとの字句 cache、`Event::Token` の省略、`limit` の軽量化、mapping の型適合検査の bitmask 化、
   `node_spans` の省略（いずれも実測で 0）。
