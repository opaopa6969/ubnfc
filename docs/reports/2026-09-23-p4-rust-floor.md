# P4 Rust 生成 parser を床まで詰める（2026-09-23）

[`2026-09-23-p4-rust-profile.md`](2026-09-23-p4-rust-profile.md) が順位を付けた残りの候補を、安い順・安全な順に実装した報告。
基準は **master `65c4a3f`**（M5 round 5 の finish 走査統合と観測契約の統一を取り込んだ後）。結論を先に書く:

- **採用 2 件**（候補4 出現表と value span の opt-in = D-035、候補2 `const DIAG` 単相化 = D-036）で
  **complex 認識 −20.5%（0.083→0.066 ms）／complex AST −26.4%（0.121→0.089 ms）／
  x64 認識 −17.4%（5.48→4.52 ms）／x64 AST −20.0%（7.33→5.86 ms）**。
  確保は complex AST が **680 回 113 KB → 509 回 77 KB**、x64 AST が **40,759 回 5.69 MB → 30,130 回 3.41 MB**。
- **AST 経路は profile が出した現実的な床（complex 0.085〜0.100 / x64 5.3〜6.2 ms）の中に入った**。
  認識は床の上端をわずかに超える（complex 0.066 対 0.065、x64 4.52 対 4.3）。§5。
- **却下 1 件**（候補3 認識での event arena 削減）。**`crates/ubnfc-front` には一切触っていない。**
  到達解析を書く前に「解析なしで取れる分」（event の 46%）を実測したら、2 セッションとも
  complex 認識 −1.5% / −3.0%、AST 0% だったため。§3。
- 候補5（AST の arena / SoA 化）は指示どおり着手していない。最終値での価値は §6。
- **driver（診断モード）経路**: AST 解析は ±0.5% で不変、**認識の prefix 解析だけ +1.8〜3.2% 悪化**する。§4。
  corpus 観測は両 backend で期待どおり（Rust 1292 / Java 1310、Fail 0）。

## 0. 計測条件

| 項目 | 値 |
|---|---|
| 基準 | master `65c4a3f`。branch `perf/rust-floor` は仕上げに master `963b822` も取り込んでいるが、その差分は harness と Java 側だけで **Rust の生成物は byte 単位で変わらない**ので、計測はそのまま当てはまる |
| rustc / cargo | `rustc 1.98.1` / `cargo 1.98.1`、`--release --offline` |
| host | 32 core、WSL2。共有機で他エージェントが並走する |
| 固定 | `taskset -c 27`。全 variant を**同一セッションで交互に 9 round**（1 round = 各 variant 1 回、bench は内部で warmup 後 5 sample の中央値） |
| 集計 | round 中央値と**全 run の最小値**。Δ は最小値どうし |
| load average | **採否に使う最終セッションは 2.5〜4.5 の静かな窓**。探索セッション（8〜22）は符号の確認にだけ使い、数値は採っていない |
| 雑音帯 | profile §2.1 と同じ: complex ±1.5%、x64 認識 ±3.5%、x64 AST ±5.5%。対照（変わらないはずの組合せ）で毎回確認する |
| 回数 | `examples/p4-rust` の `counters` feature（計時は歪むので回数だけ使う） |
| 確保 | bench の GlobalAlloc 計数（計時とは別 parse） |

variant:

| 名前 | 中身 |
|---|---|
| `master` | master `65c4a3f` |
| `final` | 本 branch（候補4 + 候補2）。既定は出現表・value span とも要求しない |
| `c2only` | `final` に `occurrences` / `value_spans` を要求した状態 = **候補2 だけを入れた状態**（master と同じものを集める） |
| `c3` | `final` + 認識で `Event::Rule` / `Event::Token` を置かない（候補3 の「解析なしで取れる分」） |
| `noevent` | `final` + 認識で event を一切置かない。**受理は同じだが scope 結果が壊れる上限**（profile §2.2 と同じ性質） |
| `drvold` / `drvnew` | driver と同じ設定（`diagnostics: true`）。old = master、new = `final` + `value_spans` |

## 1. 候補4（採用）: 出現表と value span を opt-in にする — D-035

`ParseOptions` に `occurrences` と `value_spans`（どちらも既定 false）を足した。

**誰が要るか**（実装前に調べた）:

| 利用者 | `captures` | rule / token 出現 | `value_spans` | 対応 |
|---|---|---|---|---|
| harness driver（corpus 観測） | `lexical` のときだけ出力 | `lexical` のときだけ出力 | AST が出たら常に出力 | AST 解析で `value_spans: true`。出現は `lexical: true` が含意するので追加要求は不要 |
| self-host bootstrap（`crates/ubnfc-selfhost/src/lib.rs`） | 読む | rule 出現を読む | 読まない | `lexical: true` のまま**無変更** |
| LSP 的な投影 | 読む | 読む | 読む | `occurrences` / `value_spans` を立てる |
| 「解析して AST が欲しい」だけの利用者 | 読まない | 読まない | 読まない | **何も払わなくなる（本案の得）** |
| 差分 fuzzing（`crates/ubnfc-fuzz/src/observe.rs`） | `captures` を比較 | 比較しない | 比較しない | driver 経由なので不変 |

`lexical: true` は「出現表そのものが目的」なので要求を含意させた。これで self-host と driver は無変更で通り、
**既定で空になるのは「AST だけを要求した parse の `captures`」と「`value_spans`」の 2 つだけ**になる。

**M5 round 5 との関係（重要な前提の訂正）**: master の round 5 は「出現収集」と「AST が rule ごとに読む
直下 capture 表」を**1 回の走査に統合**した。したがって **AST を作る parse では走査そのものは省けない**。
本案が省くのは (a) 公開する出現表の行を作る分（`captures` / `lexical` / `tokens` の確保と充填）と
(b) `ast.collect` による value span の走査、の 2 つである。profile が見積もった「出現収集 7 µs」は
round 5 で一部が別の仕事に吸収されており、実測はそれを反映する。
`finish.rs` では走査の骨組み（rule ごとの開始位置）を `build_ast || occurrences || lexical` で、
直下 capture 表を `build_ast` で、公開出現表を `occurrences || lexical` で、それぞれ独立に制御するようにした。

**D-023 との違い**: D-023 は結果を見れば診断が要るかが分かるので決定的再解析で埋められる。
出現表と value span は**結果からは要否が判定できない**ので、自動の再解析は無い。要る側が要求する。

### A/B（最終セッション、9 round 交互、Δ は最小値どうし）

`c2only` → `final` が本案の差（どちらも `const DIAG` 入り）。

| 入力 / mode | `c2only` 中央値 / **最小** | `final` 中央値 / **最小** | Δ | 確保 `c2only` → `final` |
|---|---:|---:|---:|---|
| complex 認識 | 0.0740 / **0.0710** | 0.0670 / **0.0660** | （下の注） | 329 → 324 回、50,554 → 31,829 B |
| complex AST | 0.1030 / **0.1010** | 0.0920 / **0.0890** | **−12 µs（−11.9%）** | **681 → 509 回、112,861 → 77,181 B（−32%）** |
| x64 認識 | 5.272 / **4.952** | 4.621 / **4.522** | （下の注） | 18,843 → 18,836 回、1,757,398 → 732,673 B |
| x64 AST | 6.953 / **6.273** | 6.255 / **5.863** | **−410 µs（−6.5%）** | **40,760 → 30,130 回、5,693,519 → 3,414,725 B（−40%）** |

**認識の 2 行は本案の得ではない。** master は認識だけの parse では元から出現を集めない（`build_ast` も
`lexical` も false）ので、`master` と `final` の認識は同じものを集める（差は §2 の単相化だけ）。
`c2only` は `occurrences` を明示的に立てているため、master がしない収集をしている。
つまりこの 2 行は「**要求すればその分を払う**」ことの確認で、`final` の既定が正しく省いていることを示す。

AST の得は complex −12 µs / x64 −410 µs。**profile の見積り（18 µs）より小さい**のは上の前提の訂正どおりで、
`ast.collect` の 11 µs はそのまま取れているが、出現収集の 7 µs は round 5 が既に一部を回収していた。

**規模**: `templates/api.rs` / `templates/finish.rs` / `templates/driver.rs` で 70 行ほど。

## 2. 候補2（採用）: `const DIAG` による診断配管の単相化 — D-036

`Session<'a, const DIAG: bool>` にし、入口が `ParseOptions::diagnostics` を見て `Session::<false>`（fast mode）か
`Session::<true>`（診断モード）を選ぶ。D-023 の再解析も `Session::<true>` で走る。
`Diagnostics` / `DisplayDiagnostics` への呼出しは Session 上の `diag_*` / `display_*` wrapper（13 個）を通し、
`DIAG == false` では定数分岐として消える。候補除外の失敗再生 `guard_e*` の早期 return と
`finish_checked` の escalate 判定も `options.diagnostics` から `DIAG` に替えた。

**profile §4.2 が最大の失敗要因に挙げた i-cache の代償は fast path には出なかった。** binary は
**4.69 → 7.34 MB（+56%）**になったが、fast path の 4 つの組合せ全てで中央値も最小値も改善した。
round 4 の `limit` inline（+5 µs 悪化）と違い、単相化は**熱い関数を太らせず、fast 側では細くする**
（記録の呼出しと引数準備が消える）ためと考えられる。ただし**診断モードには 0〜3% の悪化がある**（§4）。

### 回数（`counters`、complex 認識、1 parse あたり）

| 計数 | `master` | `final` | 意味 |
|---|---:|---:|---|
| 主診断 `join` | **3,429** | **0** | `rt/diag.rs` 内部の計数。実際の呼出しが消えた |
| 主診断 `rule` | **759** | **0** | 同上 |
| 主診断 `push` | 0 | 0 | fast mode は元から記録しない |
| display の frame 走査回数 | **3,226** | **0** | `rt/diag.rs` 内部の計数 |
| display 呼出し位置 `reach` / `failures` / `enter` / `leave` | 3,057 / 2,825 / 590 / 590 | 同数 | **計数フックは生成物の呼出し位置に挿す**ので数は変わらないが、単相化後は中身が空になる |
| `event` / `join` / `combine` / `mark` / `restore` / memo `lookup` | 1,308 / 1,408 / 1,712 / 1,975 / 2,750 / 759 | 同数 | 認識の作業量そのものは変えていない |

**回数が減ったのは診断側だけで、認識の作業量は 1 つも動いていない。**
得は「同じ作業を、記録の配管なしで行う」分である。

### A/B（最終セッション、9 round 交互）

`master` → `c2only` が本案の差（どちらも master と同じものを集める）。

| 入力 / mode | `master` **最小** | `c2only` **最小** | Δ（本案） | `final` **最小** | Δ（両案の合計） |
|---|---:|---:|---:|---:|---:|
| complex 認識 | 0.0830 | 0.0710 | −12 µs（−14.5%） | **0.0660** | **−17 µs（−20.5%）** |
| complex AST | 0.1210 | 0.1010 | **−20 µs（−16.5%）** | **0.0890** | **−32 µs（−26.4%）** |
| x64 認識 | 5.476 | 4.952 | −524 µs（−9.6%） | **4.522** | **−954 µs（−17.4%）** |
| x64 AST | 7.333 | 6.273 | **−1,060 µs（−14.5%）** | **5.863** | **−1,470 µs（−20.0%）** |

認識の 2 行は `c2only` が余分な収集をしている分だけ本案の効果を過小に見せる（§1 の注）。
**認識での本案の効果は `master` → `final` の −17 µs / −954 µs そのもの**で、候補4 は認識では何もしない。
profile §2.2 の上限（主診断の定数化 −13 µs + display の除去 −5 µs = 18 µs）と一致する。
**この案は「上限どおり取れた」数少ない例である。**

**規模**: emitter 側 91 箇所の呼出し名の置換（機械的）＋ `templates/runtime.rs` の wrapper 13 個、
入口 2 つと `entry_step` の単相化、`src/driver.rs` と `examples/p4-rust/build.rs` の切り出し目印の更新。
生成物は byte 一致で再生成できる。

## 3. 候補3（却下）: 認識で event arena を削る

**`crates/ubnfc-front/src/analysis/**` には着手していない。**
着手前に profile §8.2 が指示する「削減率の計数」と、**解析を一切必要としない分の実測**を先に行い、
どちらも「取り分が無い」と出たためである。

### 3.1 event の種別内訳（新規計測、認識だけの parse、1 parse あたり）

| 種別 | complex | x64 | 認識で読む主体 |
|---|---:|---:|---|
| `Join` | 208 | 12,681 | scope（`scope_captures`）・recovery |
| `Values` | 218 | 13,070 | 同上 |
| `Capture` | 279 | 17,478 | 同上（**scope 効果が読む本体**） |
| `Rule` | **319** | **19,723** | **誰も読まない**（AST の `build_values` と出現収集だけ） |
| `Token` | **284** | **17,294** | **誰も読まない**（AST・lexical だけ） |
| 合計 | 1,308 | 80,246 | — |

つまり **認識だけの parse では event の 46%（603 / 1,308）が最初から不要**で、これを省くのに
front の解析は要らない（`options.build_ast` / `options.lexical` を見るだけ）。
`Event::Rule` は `child` をそのまま返せば部分木の到達性は保たれる。

### 3.2 実測

| variant | complex 認識 | complex AST | x64 認識 | x64 AST | arena entry（complex 認識） |
|---|---:|---:|---:|---:|---:|
| `final`（採用形） | 0.0660 | 0.0890 | 4.522 | 5.863 | 1,308 |
| `c3`（Rule / Token を置かない） | 0.0640（**−2 µs、−3.0%**） | 0.0900（+1 µs、対照） | 4.442（−80 µs、**−1.8%**） | 5.785（−78 µs、−1.3%、対照） | **616** |
| `noevent`（event を一切置かない。**scope 結果が壊れる上限**） | 0.0550（−11 µs、−16.7%） | 0.0900（+1 µs、対照） | 3.602（−920 µs、−20.3%） | 5.782（−81 µs、対照） | 1 |

`c3` を独立の 2 セッションで測って complex 認識 **−1.5% と −3.0%**、x64 認識 **−1.8% と −1.8%**、
AST は両方 0%。**event の 46% を消しても、雑音帯の縁（complex ±1.5%）にかかる 1〜2 µs しか出ない。**

`noevent` の −11 µs / −920 µs は**到達解析では取れない**。内訳が確保の計数で分かる:
`noevent` の complex 認識は確保が **324 → 47 回 / 31,829 → 20,764 byte**、x64 認識は **18,836 → 1,816 回 /
732,673 → 102,222 byte** に落ちている。arena の push が消えたからではなく、
**`scope_captures` が capture を 1 つも見つけられず、scope の宣言・参照の `String` と `ScopeStore` の
更新が丸ごと起きなくなった**ためである（＝受理は同じでも scope 結果が壊れているという上限の中身）。
保守的な到達解析は、まさにその scope 経路を残さねばならないので、この分は取れない。

**判定: 却下。**
- 到達解析が消せるのは残り 54%（`Capture` / `Values` / `Join`）の**一部**だけで、
  46% を消して 1〜2 µs しか出ない以上、上限は 1〜2 µs を超えない。200〜400 行の front 解析、
  全 IR golden・selfhost 生成物・`examples/p4-*` 生成物の作り直し、Java backend との schema 整合に見合わない。
- `c3`（解析不要の 15 行版）も採っていない。**AST 経路では 0%**（AST は arena を読む側なので当然）で、
  得るのは認識だけの 1〜2 µs。その代わりに「認識モードでは arena の読み手が `Rule` / `Token` を読まない」
  という不変条件を新たに背負うことになり、現状それを守るテストが無い。釣り合わない。

### 3.3 profile の 12 µs との照合

profile §4.3 は「event arena で 12 µs」と書いたが、その根拠は `noevent`（上限）だった。
本 round はその 12 µs を **arena の push 1〜2 µs と scope の仕事 9〜10 µs** に分解した。
前者は取れるが小さく、後者は取ってはいけない。**profile の順位表のこの行は上限としても過大である。**

## 4. driver 経路（診断モード）

driver は corpus 観測のために `diagnostics: true` で 2 回解析する（prefix は `build_ast: false`、
本体は `build_ast: true`）。D-035 で要求側になったので、**AST 解析だけが `value_spans: true` を出し、
出現は `lexical: true` が含意する分だけ**にした（prefix 解析は `lexical` のときしか出現を出さないので要求しない）。

| 入力 / mode | `drvold`（master、`diag=1`） 中央値 / **最小** | `drvnew`（`final`、`diag=1,vspan=1`） 中央値 / **最小** | Δ（最小） |
|---|---:|---:|---:|
| complex 認識（prefix 解析） | 0.1640 / **0.1560** | 0.1650 / **0.1610** | **+5 µs（+3.2%）** |
| complex AST（本体） | 0.2120 / **0.2040** | 0.2110 / **0.2050** | +1 µs（+0.5%、雑音帯内） |
| x64 認識 | 10.931 / **10.143** | 11.364 / **10.321** | +178 µs（+1.8%、雑音帯内） |
| x64 AST | 14.304 / **13.110** | 15.039 / **13.098** | −12 µs（−0.1%） |

**正直に書く: 診断モードの認識は 0〜3% 遅くなった。** 中央値では complex +0.6% / x64 +4.0%、
最小値では complex +3.2% / x64 +1.8% と符号が安定しないが、改善はしていない。
原因は単相化で機械語が 2 部になったこと（診断側は冷たい経路なので、fast 側が得た分の裏返しを受ける）と
考えられる。**fast path の −17〜−20% と引き換えの取引**として記録する。
AST 解析（driver の主経路）は ±0.5% で不変、確保 byte は complex −0.5% / x64 −0.9%
（driver は value span を要求するので、候補4 の確保削減はここには出ない）。

### corpus（両 backend、master の期待値と完全一致）

| backend | Pass | 想定拒否 | known | Fail | 未観測 | master `65c4a3f` の期待 |
|---|---:|---:|---:|---:|---:|---|
| Rust | **1292** | **30** | **28** | **0** | **75** | 一致（`2026-09-23-rust-corpus-floor.md`） |
| Java | **1310** | **28** | **28** | **0** | **59** | 一致（`2026-09-23-java-corpus-floor.md`） |

## 5. 到達点と床

| 入力 / mode | master `65c4a3f`（最小） | 本 branch（最小） | 改善 | profile §5 の床の見込み | 判定 |
|---|---:|---:|---:|---:|---|
| complex 認識 | 0.083 ms | **0.066 ms** | −20.5% | 0.055〜0.065 | 上端+1 µs |
| complex AST | 0.121 ms | **0.089 ms** | −26.4% | 0.085〜0.100 | **床の中** |
| complex-x64 認識 | 5.48 ms | **4.52 ms** | −17.4% | 3.6〜4.3 | 上端+5% |
| complex-x64 AST | 7.33 ms | **5.86 ms** | −20.0% | 5.3〜6.2 | **床の中** |

**床の見積りは候補 4.1〜4.4 を「全部」入れた値だったが、実際には 2 件だけで AST の床に入り、
認識も床の上端に並んだ。** 差額は候補3 が見積り（−12 µs / −904 µs）どおりには出ないことで説明できる（§3）。
したがって**この設計での認識の現実的な床は complex 0.065 ms / x64 4.5 ms 前後**に更新される
（profile の 0.055 / 3.6 は候補3 の 12 µs を当てにしていた）。

長期目標（complex AST 0.02 ms、x64 AST 1.5 ms）には**依然として届かない**。profile §5 の 3 つの根拠は
どれも変わっていない: 1 byte あたり 111 回の runtime 操作、生き残る node 1 つあたり rule 評価 5.8 回の投機、
候補除外は既に 209 選択肢中 105 個を除外済み。本 round が消したのは**作業ではなく配管**である。

## 6. 残りの差は何でできているか

complex 認識 66 µs の内訳（本 round の実測で更新）:

| bucket | µs | 根拠 |
|---|---:|---|
| **rule / expression 関数呼出し 3,158、`mark`/`restore` 4,725、memo `lookup`/`store` 1,349、`join`/`combine` 3,120、終端照合 685、`fail` 457、効果 472** | 約 55 | 差分。**切ると受理が変わる機構の束で、この host に `perf` が無いのでまだ割れていない** |
| event arena の push（1,308 回） | 1〜2 | §3.2（`c3` の実測） |
| scope 効果（`scope_captures` の走査 + 宣言・参照の `String` + `ScopeStore`） | 約 9 | §3.2（`noevent` から push 分を引いた残り） |

AST の増分は 23 µs（89 − 66）。profile §3.2 の「AST 構築 22 µs」とそのまま一致する
（出現収集と value span は D-035 で既定から外れた）。

**候補5（AST の arena / SoA 化）の価値は、この 23 µs のうち確保に由来する分だけ**である。
本 round 後の complex AST は 509 回 / 77,181 byte 確保していて、認識分（324 回 / 31,829 byte）を引くと
**AST 増分は 185 回 / 45,352 byte（node 1 個あたり 1.4 回 / 346 byte）**。profile 時点は 3.2 回 / 733 byte
だったので、**候補5 が置き換える相手は既に半分以下になっている**。公開型 `Ast` の破壊的変更
（`03-design-v1.md` §6 の結果 API と canonical JSON golden に直撃）と引き換えに取れるのは
23 µs の一部でしかなく、**本 round の数字は「やらない」と言っている**。

次に効くのは、profile §5 が挙げた設計側の 2 つだけである:
(a) 「決定的に降りられる」ことを front が証明して rule 入口検査・memo・`mark`/`restore` を外す、
(b) 優先順位の連鎖を precedence climbing へ lowering して降下段数を減らす。
どちらも上の 55 µs の束に直接効く。**その前に `perf` が使える host を 1 台確保するのが依然として最優先**である。

## 7. 検証

- `cargo test --workspace --offline` 緑、`cargo test -p ubnfc-p4-rust`（`--workspace` は example crate を
  含むが、`--release` でないと 1 fixture に数十秒かかるので別に回した）緑、
  `cargo clippy --workspace --all-targets --offline -- -D warnings` 緑、`cargo fmt --all -- --check` 緑。
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
- corpus は §4 の表のとおり両 backend で master の期待値と一致。
- `cargo test -p ubnfc-fuzz` 緑（固定 seed の差分 campaign）。**注意**: 新しい worktree では
  `ubnfc-java/target` が無いと Java driver の build が失敗して `build:java` で discard され、
  campaign も Java corpus も静かに値が変わる。`cd ubnfc-java && mvn -o -DskipTests package` が先に要る
  （本 round で 2 度踏んだ。1 度目は未 build、2 度目は master merge 後の再 build 忘れ）。
- `crates/ubnfc-rust/tests/semantics.rs` の `cached_trivia`（P4 には無い `plan.trivia_cache` 経路）は、
  単相化で `self.trivia_cache.insert(key,(out,self.display_snapshot()))` が二重借用になる。
  snapshot を先に局所変数へ受けるよう `src/tokens.rs` を直した（P4 の生成物は byte 不変）。
- `ubnfc-java` と `crates/ubnfc-front` は無変更（IR の意味も schema も変えていない）。

## 8. 次にやるなら

1. **`perf` が使える host**。残り 55 µs（認識の 83%）を割る手段が無いままでは、次の案は当て推量になる。
2. §6 の (a)（決定的降下の証明）と (b)（precedence climbing への lowering）。どちらも front の解析追加。
3. **やらないと決めたもの**: 候補3（event arena の到達解析、および解析不要の Rule/Token 省略。§3）、
   候補5（AST の arena / SoA 化。§6）、位置ごとの字句 cache、`Event::Token` の省略、`limit` の軽量化、
   mapping の型適合検査の bitmask 化、`node_spans` の省略（いずれも実測で 0〜2 µs）。
