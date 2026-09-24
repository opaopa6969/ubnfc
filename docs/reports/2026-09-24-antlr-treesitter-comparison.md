# ANTLR4 / tree-sitter との比較（JSON と算術式）

2026-09-24。issue [#26](https://github.com/opaopa6969/ubnfc/issues/26)。ubnfc は `d257a9b`。

これまでの速度比較はワークスペース内の実装同士だけで、**外部のパーサ生成系との比較は無かった**
（README §3.2 がそう書いていた）。ネットワークが使える環境になったので、同じ言語を 3 記法で書き、
**正しさ（JSONTestSuite 全 318 件）** と **速度（同一入力・同一ホスト・交互実行）** を測った。

> **この報告が測っていないものを先に書く。** ANTLR4 は ALL(\*) の適応的予測と**エラー回復**を、
> tree-sitter は GLR の**増分解析**とエラー回復を持ち、どちらもエディタでの再解析を主目的に
> 設計されている。ubnfc はそのどちらも目的にしていない（増分解析は無く、回復は文法が
> `@recovery` を書いたときだけ）。**ここで比べているのは「1 回の完全な解析」だけ**であり、
> 各ツールが最も価値を出す用途は測っていない。§5 の注意も併せて読むこと。

---

## 1. 表

### 1.1 JSONTestSuite（正しさ）

318 件（`y_` 95 / `n_` 188 / `i_` 35）。`y_` は受理必須、`n_` は拒否必須、`i_` は実装依存。
**ファイル × 4 実装の全 318 行**（期待値と理由つき）は
[`examples/json/evidence/jsontestsuite-verdicts.tsv`](../../examples/json/evidence/jsontestsuite-verdicts.tsv) にある。

> **2026-09-24 追記（issue #34 / D-063）**: 下記 A の 3 件は **UBNF v2 に code point クラスを足して解消した**。
> 現在は ubnfc の 2 backend も `n_` 188/188・`y_` 95/95 で、suite との不一致は 0 件、
> 実装間で割れるのは `i_` の 2 件（BOM と 500 重ネスト）だけである。
> この節の表と `examples/json/` の生データは**改訂後の値**になっている。改訂の記録は
> [`2026-09-24-v2-codepoint-classes.md`](2026-09-24-v2-codepoint-classes.md)。以下の A は
> **なぜ穴だったか**の記録として残す。

| 実装 | `y_` 受理 / 95 | `n_` 拒否 / 188 | `i_` 受理 | `i_` 拒否 | suite との不一致 |
|---|---:|---:|---:|---:|---|
| ubnfc Rust（生成） | **95** | **188**（当初 185） | 20 | 15 | なし（当初 3 件。下記 A） |
| ubnfc Java（生成） | **95** | **188**（当初 185） | 20 | 15 | なし（Rust と 318 件すべて一致） |
| ANTLR4 4.13.2 | **95** | **188** | 21 | 14 | なし |
| tree-sitter 0.25.10 | **95** | **188** | 22 | 13 | なし |

実装間で判定が割れるのは（issue #35 解消後の再検証時点で）**1 件だけ**（C。当初は B を含む 2 件、
さらにその前は A の 3 件を含む 5 件）。上の表の `i_` 受理/拒否の内訳（20/15）は B 解消前の初回走査の
値のままで、B の 1 件が reject→accept に変わった分だけ両 backend とも `i_` 受理 21・拒否 14 になる
（ANTLR4 と同じ内訳。318 件全体を再走査した表の更新は次回の速度比較のやり直し時にまとめて行う）。

> **2026-09-24 追記（issue #35 / D-070・D-071）**: 下記 B は **native stack 番兵を `max_depth` の
> 範囲内では大きい専用 thread へエスカレーションするように直して解消した**。現在は ubnfc の
> 2 backend も `i_structure_500_nested_arrays` を accept し、suite との不一致・実装間の割れは
> **C（BOM）の 1 件だけ**になった。記録は
> [`2026-09-24-runtime-limits.md`](2026-09-24-runtime-limits.md)。以下の B は**なぜ穴だったか**の
> 記録として残す。

| file | suite の分類 | ubnfc Rust（当初） | ubnfc Java（当初） | ANTLR4 | tree-sitter | 理由 |
|---|---|---|---|---|---|---|
| `i_structure_500_nested_arrays` | 実装依存 | reject → **accept** | reject → **accept** | accept | accept | **B**: ubnfc の生成ランタイムはネイティブスタック 256 KiB で再帰を止めていた（issue #35。D-070 で解消） |
| `i_structure_UTF-8_BOM_empty_object` | 実装依存 | reject | reject | reject | accept | **C**: tree-sitter だけ先頭 BOM を読み飛ばす |

当初（この報告の初版、ubnfc `d257a9b`）は次の 3 件でも割れていた。**D-063 で解消済み**。

| file | suite の分類 | 当初の ubnfc | 現在の ubnfc | 理由 |
|---|---|---|---|---|
| `n_string_unescaped_ctrl_char` (`["a<0x00>a"]`) | 拒否必須 | accept | **reject** | **A**: UBNF に「制御文字を除く code point 集合」を書く手立てが無かった |
| `n_string_unescaped_newline` (`["new<0x0A>line"]`) | 拒否必須 | accept | **reject** | 同上 |
| `n_string_unescaped_tab` (`["<0x09>"]`) | 拒否必須 | accept | **reject** | 同上 |

**A — UBNF v2 の表現力の穴（3 記法で唯一揃えられなかった点）**
RFC 8259 §7 は文字列の中に生で置ける文字を「`"` と `\` と U+0000〜U+001F を除く全 Unicode 文字」と
定める。ANTLR は `~["\\\u0000-\u001F]`、tree-sitter は `/[^"\\\x00-\x1f]/` でそのまま書けるが、
UBNF では書けない。

- `NEGATION('…')` の除外集合はリテラルで、UBNF リテラルのエスケープは `\n \t \r \\ \'` の 5 つだけ
  （`\uXXXX` は無い）。綴れる制御文字は 3 つしかない。
- 正の範囲で書くと `CHAR_RANGE` の境界が **Java char = BMP 限定**
  （`E-TOKEN-CHAR-RANGE-SUPPLEMENTARY`）なので、補助面の生文字を落として
  **受理必須の `y_string_utf8` 等 3 件を拒否**してしまう。
- `REGEX('[^"\\\x00-\x1F]')` は front の正則部分集合パーサが `\x` を拒否し、Rust backend が生成時エラー。

そこで当初は `token SAFE = NEGATION('"\\')` を採り、**制御文字が通ることを受け入れた**。
文法の書き方の問題ではなく、**UBNF v2 に足りない機能**だった。
→ **2026-09-24 に D-063 で解消**（`CHAR_RANGE` の境界に `\uXXXX` / `\u{X…}` と補助面を許す。
issue #34、[`2026-09-24-v2-codepoint-classes.md`](2026-09-24-v2-codepoint-classes.md)）。

**B — 入れ子の深さ**。生成ランタイムはスタック番兵（`anchor.abs_diff(address) > 256 * 1024`）で
`maximum parse depth exceeded` を出す。`ParseOptions.max_depth`（既定 10,000）とは別物で、
上げても変わらない。この文法では約 200 重が境目。`i_` なので規格違反ではない。

**C — BOM**。RFC 8259 §8.1 は「JSON text を解析する実装は BOM を誤りとせず無視して**よい**」と
述べており、受理も拒否も規格内。3 つの文法はいずれも BOM を書いていない。

**`i_` 35 件のうち 30 件は 4 実装が一致**する。内訳は「巨大な数値・指数（10 件）は全実装が受理」
「孤立サロゲートの `\uXXXX`（10 件）は全実装が受理 = 3 記法とも対の検査をしない」
「UTF-8 として不正な byte 列・UTF-16・overlong（13 件）は全実装が拒否」。

#### byte と code point の扱い（`n_`/`i_` の符号化系はここで決まる）

| 実装 | 入力の見え方 | 不正 UTF-8 の落ち方 |
|---|---|---|
| ubnfc Rust | 生成物の入口が `&str` | ドライバが厳密復号で弾く（`not-utf8`）。そもそも渡せない |
| ubnfc Java | `CharSequence` | ドライバが `CodingErrorAction.REPORT` で厳密復号し失敗を拒否とする |
| ANTLR4 | 同上 | **既定の `CharStreams` は不正 byte を U+FFFD へ置換する**ので、置換に任せると不正 UTF-8 を受理してしまう。ドライバ側で厳密復号している |
| tree-sitter | `ts_parser_parse_string` に **byte 列と長さ** | tree-sitter 自身が字句で落とす（NUL も長さ指定で通るので `n_structure_null-byte-outside-string` も拒否できる） |

UTF-16 ファイル・overlong 符号化・裸の継続 byte はいずれも「UTF-8 として不正」なので
4 実装すべて拒否する（理由の出所が違うだけ）。**Java 側 2 実装と ANTLR は同じ復号関数を使う**ので、
比較が「復号の差」にならないようにしてある。

### 1.2 JSON の速度

> **2026-09-24 追記**: この節の数字は**初版の文法**（`token SAFE = NEGATION('"\\')`）で測ったもので、
> D-063 で `SAFE` を 4 本の `CHAR_RANGE` へ置き換えた後には測り直していない。
> 置き換えで `Char` の選択が 2 候補から 5 候補に増えるので、ubnfc の JSON の速度は動き得る。
> 速度の比較をやり直すときは §4.4 の手順をそのまま走らせること（他の 3 実装の文法は不変）。

入力は JSONTestSuite の大きめの `y_*` を要素に並べた JSON 配列（§4.2 に sha256）。
**「木」の意味は実装ごとに違う**ので、モード名にそのまま書いてある。

#### 中央値 ms（括弧内は全 run の最小値）

| 実装 / モード | 1 KB | 10 KB | 100 KB | 1 MB |
|---|---:|---:|---:|---:|
| ANTLR4 / 認識のみ（`setBuildParseTree(false)`） | **0.012 (0.011)** | **0.160 (0.121)** | **1.095 (1.001)** | **11.166 (10.310)** |
| ANTLR4 / **parse tree**+no-op listener walk | 0.024 (0.024) | 0.266 (0.248) | 2.665 (2.505) | 27.173 (25.284) |
| tree-sitter / **CST** 構築（必ず作られる） | 0.031 (0.029) | 0.339 (0.335) | 5.507 (3.436) | 47.316 (42.808) |
| tree-sitter / CST + `TSTreeCursor` 全走査 | 0.039 (0.035) | 0.417 (0.404) | 4.861 (4.312) | 61.194 (54.698) |
| ubnfc Rust / 認識のみ | 0.063 (0.062) | 0.670 (0.659) | 11.864 (11.357) | 234.478 (223.716) |
| ubnfc Rust / **typed AST**（`@mapping` の record 木） | 0.097 (0.095) | 1.174 (1.101) | 48.286 (46.262) | 635.357 (561.686) |
| ubnfc Java / 認識のみ | 0.127 (0.111) | 1.300 (1.273) | 24.316 (23.270) | 376.981 (276.620) |
| ubnfc Java / typed AST | 1.141 (0.668)※ | 3.229 (2.779) | 43.722 (39.309) | 664.713 (572.497) |

※ この 1 セルだけ run 間のばらつきが大きい。同条件で測り直すと中央値 0.696 ms / 最小 0.311 ms
（`warmup` 200・`runs` 7 は同じ）。GC と JIT の揺れで、**信用しない**こと。

#### µs/byte（中央値から）／ MB/s

| 実装 / モード | 1 KB | 10 KB | 100 KB | 1 MB |
|---|---:|---:|---:|---:|
| ANTLR4 / 認識のみ | 0.0114 / 87.7 | 0.0159 / 62.8 | 0.0109 / 91.4 | **0.0112 / 89.6** |
| ANTLR4 / parse tree+walk | 0.0229 / 43.6 | 0.0264 / 37.9 | 0.0266 / 37.5 | 0.0272 / 36.8 |
| tree-sitter / CST 構築 | 0.0289 / 34.6 | 0.0337 / 29.7 | 0.0550 / 18.2 | 0.0473 / 21.1 |
| tree-sitter / CST+走査 | 0.0364 / 27.4 | 0.0414 / 24.1 | 0.0486 / 20.6 | 0.0612 / 16.3 |
| ubnfc Rust / 認識のみ | 0.0590 / 16.9 | 0.0666 / 15.0 | 0.1186 / 8.4 | **0.2345 / 4.3** |
| ubnfc Rust / typed AST | 0.0911 / 11.0 | 0.1167 / 8.6 | 0.4827 / 2.1 | 0.6353 / 1.6 |
| ubnfc Java / 認識のみ | 0.1192 / 8.4 | 0.1292 / 7.7 | 0.2431 / 4.1 | 0.3769 / 2.6 |
| ubnfc Java / typed AST | 1.0697 / 0.9※ | 0.3208 / 3.1 | 0.4371 / 2.3 | 0.6646 / 1.5 |

**読み方**: ANTLR4 の µs/byte は 1 KB→1 MB でほぼ一定（0.011 前後）、tree-sitter は 1.6 倍に増える程度、
**ubnfc は 4 倍に悪化する**。1 MB の認識で ANTLR4 の **21 倍遅い**。原因は §3 で特定してある。

**注**: tree-sitter の 100 KB は「CST+走査」（4.861 ms）が「CST 構築」（5.507 ms）より速く出ていて
順序が逆転している。反復数が 99 / 53 と少なく、最小値では 3.436 / 4.312 と正しい順序なので、
ホスト負荷の揺れである。この 2 セルは差を読まないこと。

#### 1 parse あたりの割当 byte

| 実装 / モード | 1 KB | 10 KB | 100 KB | 1 MB |
|---|---:|---:|---:|---:|
| ANTLR4 / 認識のみ | 25,592 | 273,152 | 2,774,944 | 27,067,376 |
| ANTLR4 / parse tree+walk | 40,768 | 442,616 | 4,487,136 | 43,763,576 |
| tree-sitter | 測っていない（C。GlobalAlloc 相当の仕掛けを入れていない） | | | |
| ubnfc Rust / 認識のみ | 18,785 | 20,705 | 35,041 | 76,557,129 |
| ubnfc Rust / typed AST | 119,965 | 1,003,987 | **118,627,057** | **1,543,850,645** |
| ubnfc Java / 認識のみ | 180,160 | 1,780,016 | 17,688,512 | 277,523,240 |
| ubnfc Java / typed AST | 346,344 | 3,568,144 | 35,601,960 | 460,486,600 |

**注**: ubnfc Rust の割当は計時とは別に 1 回だけ走らせた parse で数えている（§4.1）。
`reuse_buffers` が効くので**サイズ間で素直には比べられない**（認識のみの 100 KB が 35 KB、
1 MB が 76.5 MB と桁で飛ぶのはこのため。どちらが「1 parse の実費」かはこの測り方では分けられない）。
tree-sitter は C なので同じ仕掛けを入れておらず、測っていない。

ubnfc Rust の typed AST は **1 MB の入力に 1.5 GB を確保する**（割当**回数**は 23 回なので、
巨大なバッファを数本取っている）。既定の `ResourceLimits.arena_entries = 2,000,000` では
1 MB の JSON が `arena budget exceeded` で**解析に失敗する**ため、計測では全サイズ共通で
200,000,000 へ上げている（§4.3）。

### 1.3 算術式（Operators / Power）の速度

入力は既存の速度比較（family B）と同じ **512 atom**（`1+1+…+1`、1,023 byte）と
**power-64**（`2^2^…^2`、127 byte）。

#### 中央値 ms（括弧内は最小値）

| 実装 / モード | 512 atom (1,023 B) | power-64 (127 B) |
|---|---:|---:|
| ANTLR4 / 認識のみ | **0.045 (0.045)** | **0.007 (0.005)** |
| ANTLR4 / parse tree+walk | 0.209 (0.186) | 0.023 (0.019) |
| tree-sitter / CST 構築 | 0.403 (0.397) | 0.024 (0.023) |
| tree-sitter / CST+走査 | 0.466 (0.461) | 0.032 (0.028) |
| ubnfc Rust / 認識のみ | 0.119 (0.118) | 0.009 (0.008) |
| ubnfc Rust / typed AST | 0.260 (0.259) | 0.025 (0.023) |
| ubnfc Java / 認識のみ | 0.199 (0.159) | 0.016 (0.014) |
| ubnfc Java / typed AST | 0.662 (0.653) | 0.389 (0.338) |

#### 1 parse あたりの割当 byte

| 実装 / モード | 512 atom | power-64 |
|---|---:|---:|
| ANTLR4 / 認識のみ | 148,104 | 13,672 |
| ANTLR4 / parse tree+walk | 351,544 | 27,064 |
| ubnfc Rust / 認識のみ | **18,721** | **18,721** |
| ubnfc Rust / typed AST | 511,871 | 80,119 |
| ubnfc Java / 認識のみ | 344,432 | 31,760 |
| ubnfc Java / typed AST | 1,464,696 | 159,192 |

**読み方**: 512 atom の認識では ANTLR4 が最速（ubnfc Rust の 0.38 倍）。
木込みでは ubnfc Rust の typed AST（0.260 ms）が ANTLR4 の parse tree（0.209 ms）に近く、
tree-sitter（0.403 ms）より速い。ただし**作っているものが違う**——ANTLR4 の parse tree は
文法規則そのままの木、tree-sitter の CST は全トークンを持つ具象木、ubnfc の typed AST は
`@mapping` で指定した record（`Binary(left, op, right)`）の木である。
power-64 のような 127 byte の入力では、どの実装も 10〜30 µs の桁で、
ubnfc Java の typed AST（0.389 ms）だけが外れている（右再帰 64 段の mapping）。

---

## 1.4 2026-09-24 追記 — issue #33 後の JSON（**PR #32 の数字は上の表のまま**）

issue [#33](https://github.com/opaopa6969/ubnfc/issues/33)（[`2026-09-24-json-perf.md`](2026-09-24-json-perf.md)）で
§3 の原因（memo の `defaultSafe` 15 式）を取り除き、併せて排他 predict の空振りを消した。
**上の §1.2 の表は PR #32 の測定のまま残す。**以下は別セッションの測り直しで、
before（`origin/master` 75dec71）と after を同じ走行で交互に測ったもの。認識のみ、`taskset -c 30`、
7 run × 2 周、load average 10〜26 の共有機なので**最小値**を主に読む。

### µs/byte（最小値）

| 実装 / モード | 1 KB | 10 KB | 100 KB | 1 MB | 1 MB / 1 KB |
|---|---:|---:|---:|---:|---:|
| ANTLR4 / 認識のみ（同セッション再測） | 0.0133 | 0.0119 | 0.0119 | **0.0114** | 0.86 |
| ubnfc Rust / 認識のみ（**before** = PR #32 と同じ生成物） | 0.0608 | 0.0719 | 0.1537 | **0.2148** | **3.53** |
| ubnfc Rust / 認識のみ（**after** = issue #33） | **0.0291** | **0.0309** | **0.0316** | **0.0305** | **1.05** |

| | 1 MB ms（中央値 / 最小） | ANTLR4 比（最小値） |
|---|---:|---:|
| ubnfc Rust 認識 before | 239.48 / 214.82 | 18.8 倍 |
| **ubnfc Rust 認識 after** | **35.59 / 30.53** | **2.7 倍** |

**µs/byte のサイズ悪化（3.53 倍）は消えた**（1.05 倍）。§3 の「memo が原因」という診断はそのとおりで、
`defaultSafe` 15 式のうち 14 式は再入判定の精密化（D-066）で、残る `Ws` は「終端走査だけの規則に
memo は割に合わない」という計画上の判断（D-065）で落ちた。さらに、排他 predict が外れたときに
候補を全部試していた空振り（1 byte あたり literal 照合 1.0 回）を消した（D-066）。
**issue #33 の目標「ANTLR の 2 倍以内（≤0.022 µs/byte）」には届いていない**（2.7 倍）。
残差は単価ではなく**回数**で、scannerless PEG が 1 byte あたりに評価する式が 4.52 個、cursor 更新が
6.58 回あるのに対し、ANTLR は字句 DFA 遷移 1.00 + token 0.204 + 規則呼出し 0.137 しか払わない
（[`2026-09-24-json-perf.md`](2026-09-24-json-perf.md) §3 の回数 × 単価の表）。

tree-sitter と typed AST の列、算術式（§1.3）、正しさ（§1.1）はこの変更で動かない
（memo と predict の空振りは観測を変えない）。

---

## 2. ツールと版

| もの | 版 | 取得元 |
|---|---|---|
| ANTLR4 | 4.13.2（complete jar、sha256 `eae2dfa1…`） | Maven Central `org/antlr/antlr4/4.13.2/antlr4-4.13.2-complete.jar` |
| tree-sitter CLI | 0.25.10 (`da6fe9be`) | npm `tree-sitter-cli@0.25.10` |
| tree-sitter C ランタイム | 0.25.10（tag commit `208c6cac`） | GitHub `tree-sitter/tree-sitter` を shallow clone、`gcc -O2 -std=gnu11` で生成 `parser.c` と一緒にリンク |
| JSONTestSuite | commit `1ef36fa0` | GitHub `nst/JSONTestSuite` |
| RFC 8259 本文 | sha256 `61a5378f…` | rfc-editor.org（corpus の引用照合用） |
| ubnfc | `d257a9b`（この PR の commit） | 本 repo |
| JDK | 21.0.9 (Temurin 21.0.9+7-LTS) | |
| rustc / cargo | 1.98.1 / 1.98.1、`--release` | |
| gcc | 11.4.0 | |
| node | v20.20.0（`tree-sitter generate` の実行に必要） | |

取得は `scripts/fetch-external-tools.sh`（版と sha256 を固定、`benchmarks/external/` に置く。
このディレクトリは `.gitignore` 済み）。ネットワークが無い環境では**何が足りないかを述べて非 0 で終わる**
（`--check` は取得済みかどうかだけを見る）。

---

## 3. ubnfc が JSON で伸びない原因（特定済み）

µs/byte が入力サイズで 4 倍悪化するのは **memo** である。同じ生成物・同じ入力で
`ParseOptions.memo` だけを切ると、**µs/byte がサイズによらず平らになる**。

| 入力 | memo あり（既定） | memo なし | 比 |
|---|---:|---:|---:|
| 1 KB | 0.0880 µs/B | 0.0501 µs/B | 1.76 |
| 10 KB | 0.0736 | 0.0498 | 1.48 |
| 100 KB | 0.1418 | 0.0585 | 2.42 |
| 1 MB | 0.1947 | **0.0509** | **3.82** |

（この表だけ別ドライバの実測。`taskset -c 30`、認識のみ、7 run の中央値。
§1.2 の本表とは反復数の決め方が違うので絶対値は少しずれる。）

- **形の問題ではない。** 同じ 1 MB を「1 配列に 31,705 要素」と「32 要素の小配列を 991 個」で
  作り分けても、認識の中央値は 231.1 ms と 229.2 ms でほとんど同じだった。繰り返しの幅ではなく
  **入力の総量**に効いている。
- **割当回数は入力サイズによらず 23 回**で、byte だけが増える（1 MB で 76.5 MB）。
  巨大なハッシュ表を数本持つ形なので、効いているのは局所性だと考えられる。
- JSON は本質的に LL(1) 相当で、同じ位置を 2 度試す規則がほとんど無い。
  IR の memo plan（`plan.memo.entries`）は 117 式のうち **102 式で memo を省略している**（証明付き）が、
  残る **15 式は `reason: "defaultSafe"`**——「危なくないから既定で入れる」であって
  「当たるから入れる」ではない。この 15 式ぶんの保存コストとハッシュ表がそのまま損になっている。
- D-058（PR #24、issue #17）が「同じ位置へ再入し得る規則だけに memo を絞る」を入れたが、
  **`defaultSafe` に落ちた式は絞り込みの対象外**である。ここを詰めれば JSON 1 MB の認識は
  234 ms → 51 ms（**4.6 倍**）になり、ANTLR4 比 21 倍が 4.5 倍まで縮む。→ §6 の申し送り。

**この報告の数字は既定設定のまま**で、`memo: false` の値は原因特定のための診断であって
比較表には入れていない（issue の「JSON 向けに ubnfc を調整しない」に従う）。

---

## 4. 条件

### 4.1 ホストと実行の作法

- AMD Ryzen 9 7950X（16 コア / 32 スレッド）、WSL2（Linux 6.18.33.2）。**共有機で並走負荷がある。**
- `taskset -c 30` で 1 コアに固定、単一 thread。
- load average（`/proc/loadavg` の 1/5/15 分）:
  JSON 開始 7.35 / 9.05 / 10.31 → 終了 8.30 / 8.87 / 10.02、
  算術 開始 7.41 / 8.66 / 9.94 → 終了 6.63 / 8.31 / 9.77。
- **入力ごとに実装を交互に回す**（実装ごとに全入力ではない）。ホスト負荷の傾きを 1 つの実装へ
  押しつけないため。
- 1 run は 0.5 秒ぶんの反復、**7 run**。表には中央値と最小値を両方載せた。
- Java（ANTLR4 / ubnfc Java）: `-Xms2g -Xmx2g`、`System.nanoTime()`。
  **暖機は反復数で切る（既定 200 反復、最長 20 秒）**。冷えた 1 反復から反復数を決めると
  解釈実行の所要で反復数が 3 になり「暖機なしの値」を測ってしまう（最初に測ったとき 10 倍ずれた）。
  各セルの実際の反復数・暖機反復数は CSV の `iterations` / `warmup` 列にある。
  割当は `com.sun.management.ThreadMXBean.getThreadAllocatedBytes`。
- Rust（ubnfc Rust）: `--release`、`std::time::Instant`、割当は `GlobalAlloc` を数える別 parse（計時に含めない）。
- C（tree-sitter）: `gcc -O2 -std=gnu11`、`clock_gettime(CLOCK_MONOTONIC)`、
  1 反復 = `ts_parser_parse_string` + `ts_tree_delete`（増分解析は使わない。入力ごとに `ts_parser_reset`）。

### 4.2 入力（byte 同一の確認用に sha256）

```
d447f5b338fad2f4418b6b762c6cb6bd451ef33bd36551010252af6c300ddc90  json-1k.json    1,067 byte   (27 要素)
7ab96e563a716f06876abdead3c7125bc945e730831dc5790f99a60ef01f02c1  json-10k.json   10,066 byte  (314 要素)
d7a4d32f25212e517f1a4e48cf684326c3874fb49e566f2f541911e9798d2ee1  json-100k.json  100,039 byte (3,169 要素)
ae482dce4b9b5f5ed2dad2f988e1d0f675663faf61f22c31d52dccbefaf9849c  json-1m.json    1,000,112 byte (31,705 要素)
c6973089da125bc7162b53db8e8e6fa05cd62ce89ff873f197625ab4c08bf194  operators-atoms-512.txt  1,023 byte
c5342c44c6b7ea82cfc884a1d0d745848fb91824b2bff0a7a740b6961fed00fd  power-64.txt             127 byte
01ceafe99577c1362d900b69c6994fe6d4090febe8aabeb024ea0a1095a87650  json-1m-chunked.json  971,380 byte（§3 の診断用）
```

JSON 入力は `benchmarks/scripts/make-json-inputs.py` が JSONTestSuite の `y_*` から決定的に組む
（サイズ降順・名前昇順で上位 24 件を要素に、`,\n` 区切りで目標サイズまで）。同じ commit の
JSONTestSuite なら byte 同一になる。

### 4.3 各実装で使った設定（「速くする細工」「遅くする細工」をしていないことの申告）

| 実装 | 設定 | 備考 |
|---|---|---|
| ANTLR4 | 生成は既定（`-no-visitor -no-listener`）。実行時は `BailErrorStrategy` + 例外を投げる `ErrorListener` | 既定の `DefaultErrorStrategy` は挿入・削除で回復して**拒否できない**ので、正しさの測定には使えない。速度側では回復が走らない（全入力が受理される）ので影響しない。SLL/LL のモード切り替えなどの**高速化は一切していない** |
| tree-sitter | CLI 既定の生成（ABI 15）。C ランタイムは `-O2` | 増分解析（`ts_parser_parse` の old_tree）は使わない。**CST は常に作られる**ので「認識のみ」は存在しない |
| ubnfc Rust | `ParseOptions::default()`（memo / predict / factor / buffer 再利用がすべて有効）から `build_ast` だけを切り替え。**`ResourceLimits.arena_entries` を 2,000,000 → 200,000,000 へ上げた** | 上げた理由は §1.2 の注記（既定では 1 MB の typed AST が失敗する）。入力サイズで設定を変えないよう全サイズ共通。認識のみの経路はこの上限に触れない |
| ubnfc Java | 認識 = `ParseOptions.RECOGNIZE`、AST = `DEFAULT.withObservations(false, false)`（出現表と診断を作らない最速の AST 経路、D-057） | |

文法側も、3 記法のどれにも「遅くするための書き方」を入れていない。むしろ逆に、
`Power` の `{ '^' Expr }` は `.g4` / `grammar.js` では **`?` / `optional(...)` で書いた**——
`*` / `repeat` のままだと本当に曖昧になり、ANTLR には ALL(\*) の full-context 予測を、
tree-sitter には GLR の分岐を強いてしまうからである（言語は同じ。`examples/arith/README.md`）。

### 4.4 再現手順

```sh
scripts/fetch-external-tools.sh                     # 外部ツールの取得（版固定）
export WORK=$(mktemp -d)
benchmarks/scripts/prepare-external.sh              # ANTLR / tree-sitter / ubnfc を 3 文法ぶん建てる
benchmarks/scripts/check-jsontestsuite.sh           # 正しさ（§1.1 の表）
benchmarks/scripts/run-external.sh json             # 速度（§1.2）
benchmarks/scripts/run-external.sh arith            # 速度（§1.3）
```

---

## 5. 注意（数字の横に置くこと）

1. **同条件ではない部分が本質的に残る。** ANTLR4 は「字句解析器 + ALL(\*) パーサ」、
   tree-sitter は「構文状態依存の字句 + GLR」、ubnfc は「スキャナレス PEG」である。
   同じ言語を受理することは JSONTestSuite 318 件で確かめたが、**内部でやっている仕事は違う**。
2. **ANTLR4 の強みは測っていない。** ALL(\*) の適応的予測は、LL(k) では書けない文法を
   実用速度で解くためのもので、JSON のような LL(1) 相当の文法では持ち味が出ない
   （逆に言えば、ここでの ANTLR4 の速さは「難しい文法でも同じ」ではない）。
   **エラー回復**（`DefaultErrorStrategy`）は測定では無効化している。
3. **tree-sitter の強みは測っていない。** tree-sitter の設計目標は**増分解析**——
   1 文字編集したときに木の差分だけを作り直すことで、その経路はここでは 1 度も通していない。
   全文解析だけを見て「tree-sitter は遅い」と読むのは誤り。
   また CST を作らない選択肢が無いので、「認識のみ」の列は原理的に存在しない。
4. **ubnfc の強みも測っていない。** `@scopeTree` / `@declares` / `@backref` のような
   宣言的なスコープ・記号表、診断の位置と `expected` の契約、Java と Rust の観測一致は
   JSON にも算術式にも現れない。ubnfc が最適化してきたのは P4（124 規則）のような
   実用文法での**認識と typed AST**である。
5. **木の意味が違う**（列見出しに書いたとおり）。ANTLR4 の parse tree は規則そのままの木、
   tree-sitter の CST は全トークンを含む具象木、ubnfc の typed AST は `@mapping` の record 木。
   ノード数も表現力も違うので、木込みの列を横に並べて「n 倍」と言うのは慎重に。
6. **共有機の測定である。** load average 7〜10 の環境で `taskset -c 30` に固定して測った。
   最小値と中央値の差（特に Java の 1 MB で 1.2〜1.4 倍）はホスト負荷と GC の揺れを含む。
7. **1 MB の Java セルは反復数が 3**（1 反復が 0.3〜0.7 秒のため）。中央値は 7 run のもので
   反復平均ではないので、他のセルより粗い。

---

## 6. 申し送り

1. ~~**UBNF v2 に「制御文字を除く code point 集合」を書く手立てを足す**（§1.1 A）。~~
   **完了（2026-09-24、issue #34 / D-063）**: 候補 (a)+(b) を採り、`CHAR_RANGE` の境界と
   `NEGATION` の除外集合に `\uXXXX` / `\u{X…}` を、`CHAR_RANGE` に補助面を許した（v2 限定）。
   `unlaxer-parser` 側の字句と corpus の `expected` 表記には触れていない（ubnfc が先行実装）。
   以下は当初の検討メモ。
   候補は (a) `CHAR_RANGE` の補助面対応、(b) `NEGATION` の除外集合に `\uXXXX` を許す、
   (c) リテラルのエスケープに `\uXXXX` を足す。(c) が最も影響範囲が小さそうだが、
   `unlaxer-parser` 側の字句と corpus の `expected` 表記に触れるので別 issue で検討する。
   これが入るまで、`examples/json/json.ubnf` は `n_string_unescaped_*` 3 件で
   ANTLR / tree-sitter と判定が割れ続ける。
2. **memo の `defaultSafe` を絞る**（§3）。JSON 1 MB の認識が 234 ms → 51 ms（4.6 倍）になる見込み。
   D-058 の省略証明が届かなかった 15 式が対象。
3. **typed AST の割当**（1 MB で 1.5 GB、既定 `arena_entries` を超えて失敗する）。
   既定の `ResourceLimits` のままでは 1 MB の JSON が解析できない、というのは
   利用者から見ると「壊れている」に近いので、arena の使い方か既定値のどちらかを直す。

---

## 7. 成果物

| もの | 場所 |
|---|---|
| 3 記法の JSON 文法と対応表 | [`examples/json/`](../../examples/json/)（`json.ubnf` / `antlr/JSON.g4` / `treesitter/grammar.js` / README） |
| 3 記法の算術式文法と対応表 | [`examples/arith/`](../../examples/arith/) |
| 外部ツールの取得（版・sha256 固定） | [`scripts/fetch-external-tools.sh`](../../scripts/fetch-external-tools.sh) |
| 判定・計測のドライバ | `benchmarks/{c/ts_runner.c, java/AntlrRunner.java, java/UbnfcRunner.java, src/family_ext.rs}` |
| 実行スクリプト | `benchmarks/scripts/{prepare-external,run-external,check-jsontestsuite}.sh`、`merge-verdicts.py`、`make-json-inputs.py`、`make-json-spec-corpus.py` |
| JSONTestSuite 由来の仕様ケース 283 件 | [`corpus/spec/json/`](../../corpus/spec/json/)（引用は RFC 8259。`crates/ubnfc-harness/tests/spec_corpus.rs` が照合） |
| **この報告の生データ** | [`examples/json/evidence/`](../../examples/json/evidence/)（318 件 × 4 実装の判定 `jsontestsuite-verdicts.tsv`、要約、速度 CSV、入力の sha256）、[`examples/arith/evidence/`](../../examples/arith/evidence/) |
