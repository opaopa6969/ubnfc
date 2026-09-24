# Rust typed AST の arena 化（issue #59）

2026-09-24〜25。issue [#59](https://github.com/opaopa6969/ubnfc/issues/59)（D-006 で保留した「AST を arena + index にする」案）。
決定は **D-077**。比較の基準（before）は `origin/master` `618225a`、after はこの PR。

## 0. 結論

- **JSON 1 MB の typed AST の確保（定常状態）は 523.4 MB → 所有 `Ast` 15.2 MB / 木 8.4 MB（30 回）。
  issue の目標 ≤ 50 MB（入力の 50 倍以内）を満たす**（入力の 15.2 倍 / 8.4 倍）。
  **cold（pool なし = プロセスで最初の parse）は 523 → 97 MB / 84 MB で目標に届かない**。残りは event arena
  （156 万 event × 32 byte = 50 MB が下限）で、AST の表現ではない（§3.2）。
- **確保の主因は AST の表現ではなかった**（§1）。523 MB のうち `Box<Ast>` / `String` / `Vec<Ast>` は 22 MB で、
  残りは 88 byte の event の倍々伸長（pool の上限超えで毎 parse 作り直し）と、使わない表の事前確保だった。
  この PR は AST の表現（arena の木）と、その 3 つの確保の両方を直した（決定 **D-077**、§2）。
- **表現**: 木（節点 24 byte + 子は `u32` の slot、文字列は入力の byte 範囲）。**所有 `Ast` は木の写像**として
  残したので、所有 API は byte 単位で不変、generator flag は要らない（§7）。木そのものは
  `ParseOptions::ast_tree` で受け取れる。型付き節点 1 個あたりの AST 層の確保は
  **JSON 536 → 129 byte（所有）/ 72 byte（木、入力の複製 8.4 byte 込み）**、木の実使用は 44 byte、
  理論下限（全節点 × 18 byte）は 27 byte（§4）。木の確保回数は認識の分を除いて JSON 1 MB で型付き節点 1 万個あたり約 1 回、P4 x64 で約 10 回。
- **P4 x64 AST**: 確保 3.41 MB → 2.10 MB（所有）/ 1.17 MB（木）、AST 層の確保回数 11,294 → 11,278（所有）/ **8**（木）。
  時間は 5.71〜6.05 → 5.50〜5.90 ms（所有、−3%）/ 5.01〜5.15 ms（木、−12〜−15%）で **≤ 6 ms**。
  complex の所有 AST は ±3% の雑音帯で変わらず、木は −9〜−11%（§5）。
- **時間（最小値）**: 所有 AST は JSON 1 MB −65%、100 KB −18〜−20%、10 KB −5〜−7%、1 KB +1.5%。
  木は before の所有 AST の 0.32（JSON 1 MB）〜 0.92 倍。認識は −8〜+2%（JSON 100 KB の −7% は、使わない memo 表を
  毎 parse ゼロ埋めしなくなった分）。
- **観測は変わらない**: canonical JSON golden 13 件（P4 の Formula 入口 9 + BooleanExpression 入口 4。Java golden との完全一致も）、JSONTestSuite の 293 件と
  JSON 4 入力で before / after / 木の canonical JSON・node span・value span が byte 一致、corpus
  Rust Pass 1,292 / Fail 0・Java Pass 1,310 / Fail 0（master と同じ表）、差分 fuzz 固定 seed、self-host 固定点、
  stax の 2 backend 突き合わせ（§6）。

## 1. 何が確保していたか（before の内訳）

#68 の報告は「JSON 1 MB の typed AST で Rust 523 MB（issue 本文の 350 MB は Java との取り違え）、
差のほとんどは `Box<Ast>` と `Ast::Text(String)`」と見立てていた。**計測するとそうではなかった。**
生成 runtime の `finish` の段ごとに GlobalAlloc の計数を挟んで、json-1m.json の定常状態（同じ入力の 2 回目）を分けた:

| 段 | 確保回数 | byte | 中身 |
|---|---:|---:|---|
| `Session::new` | 19 | 107,425,048 | memo 表の事前確保（76.4 MB、JSON は memo を **1 度も使わない**）+ `node_spans` の見積り確保（入力 byte / 2 × 56 byte = 28 MB、実際の節点は 118,896） |
| 解析（event の記録） | 6 | 363,331,584 | event arena の倍々伸長。`Event` が **88 byte** × 156 万 event、容量 210 万。pool の上限 64 MB を超えるので毎 parse 捨てて作り直す |
| 出現収集（`occurrences`） | 27 | 30,234,592 | 公開出現表 `captures` の見積り確保（260,000 × 72 byte、**要求していないのに確保**）+ 直下 capture 表 |
| AST 構築（`build_values`） | 132,154 | 22,348,196 | `Box<Ast>` / `String` / `Vec<Ast>` と、value span 用の `String` 先頭 pointer → span の hash 表 |
| 合計 | 132,210 | 523,358,132 | |

**AST そのもの（`Box` / `String` / `Vec`）は 523 MB のうち 22 MB（4%）** で、残りは AST を作るための
event arena（69%）と、使わない表の事前確保（memo 15%・`node_spans` 5%・`captures` 4%）だった。
100 KB 入力では pool が効くので 6.4 MB（AST 構築 1.57 MB + `node_spans` 2.8 MB + `captures` 1.9 MB）。
1 MB を境に桁が変わるのは、pool の保持上限（`POOL_RETAIN_BYTES` = 64 MB）を event arena が超えて、
毎 parse arena を倍々で作り直していたからである。

したがって issue の目標（JSON 1 MB の typed AST 確保 ≤ 50 MB）には、AST の表現に加えて
**event arena と 3 つの事前確保**も直す必要があった。この PR はその全部を扱う（§2）。

## 2. 変えたこと（D-077）

| # | 変えたこと | 主に効く所 |
|---|---|---|
| 1 | **AST を arena で作る**（生成 `ast.rs` の `pub mod tree`、`AstTree`）。節点は 24 byte（kind `u16`・規則 `u16`・`node_id` `u32`・code point span `2×u32`・`a`/`b` `2×u32`）で `Vec<TreeNode>` に作った順に積む。子は `Vec<u32>` の slot で指す（record の field ごとに scalar / optional は 1 slot、空は `NONE`。list は 2 slot = 開始と個数で、要素は同じ `slots` に先に積む）。**Text は入力の byte 範囲で持ち `String` を作らない**（semantic text は trim 済み範囲か token の content 範囲で、常に入力の部分文字列）。**所有 `Ast` は木からの写像**（`project`）で作るので、mapping の生成コードは 1 本（木を作るだけ）。value span も木から集めるので、`String` の先頭 pointer → span の hash 表（`TextSpans`）と `remember_text` の識別用 1 byte を捨てた。`node_spans` は木から作成順に導出する（容量ちょうど） | AST 構築の確保回数・byte |
| 2 | **`ParseOptions::ast_tree`（既定 false）と `ParseResult::tree`**。`build_ast: false, ast_tree: true` なら所有 `Ast` と `node_spans` を作らず木そのものを返す（入力を 1 回だけ複製して `&str` を貸す）。両方立てれば所有 `Ast` は返した木の写像。型付きの参照（`tree::Node` / 型ごとの `tree::g_…<'t>`）は所有形と同名の field メソッドを持つ（`node.g_value` → `node.g_value()`）。`canonical_json()` / `node_spans()` / `value_spans()` / `to_ast()` も木から直接作る | 利用者が選ぶ木の経路 |
| 3 | **event arena の 1 要素を 88 → 32 byte**（`PackedEvent`）。位置・event id・site・expr は `u32`、Token の規則は `u16`、Recovery の本体は別表に置いて添字だけ持つ。`Event` 列挙は読むときの形として残し（`Session::ev` / `set_ev`）、読む側の生成コードは `self.arena[x.0]` → `self.ev(x)` の置換だけ。literal / token 参照の Token 付け替え（P4 で 425 箇所）は `relabel_token` が詰めたまま複製し、記録するかの判定は呼出し側へ inline する（§5 の最後） | AST を作る parse の event 記録（JSON 1 MB で 156 万 event） |
| 4 | **使わない表を事前確保しない**。memo 表は最初の insert で見積りどおりに確保する（JSON は memo を 1 度も使わない）。公開出現表 `captures` の見積り確保は `occurrences` / `lexical` を要求したときだけ。`node_spans` の「入力 byte / 2」見積りは廃止（木から容量ちょうど） | 事前確保 |
| 5 | **pool と arena の容量**。保持上限（64 MB）を超えたら event arena と直下 capture 表を使用量まで縮めてから判定する（倍々の余りだけで上限を超えていた）。pool の無い最初の parse の arena 初期容量は「2 event / byte」を下限に見積もる（以前は 32,768 で打ち切って倍々） | 定常状態の 1 MB 入力、cold の伸長 |

1〜2 が issue の本題（D-006 の arena 案）、3〜5 は §1 で見つけた「AST の表現の外にあった確保」である。
いずれも **観測（受理・AST・canonical JSON・node / value span・診断・統計）を変えない**（§6）。

## 3. 確保

### 3.1 定常状態（同じ入力を 3 回解析した後の 1 回。`reuse_buffers` は既定の true）

確保回数 / byte（GlobalAlloc の alloc + realloc を数える。realloc は新しい大きさを足す）。

| 入力 | byte | before 認識 | before 所有 AST | **after 所有 AST** | **after 木** | 木 ÷ before AST |
|---|---:|---:|---:|---:|---:|---:|
| complex.tiny | 332 | 324 / 31,829 | 509 / 77,181 | 508 / 55,003 | 330 / 40,217 | 0.521 |
| complex-x4.tiny | 1,293 | 1,211 / 63,470 | 1,928 / 234,276 | 1,926 / 152,545 | 1,218 / 93,827 | 0.400 |
| complex-x16.tiny | 5,179 | 4,741 / 196,225 | 7,578 / 867,301 | 7,570 / 540,838 | 4,748 / 306,308 | 0.353 |
| complex-x64.tiny | 20,923 | 18,836 / 732,673 | 30,130 / 3,414,725 | 30,114 / 2,103,638 | 18,844 / 1,165,956 | 0.341 |
| flat-arithmetic.tiny | 1,426 | 18 / 19,790 | 1,066 / 225,360 | 1,053 / 131,153 | 29 / 124,200 | 0.551 |
| large-match.tiny | 1,380 | 2,152 / 92,063 | 3,527 / 429,928 | 3,521 / 265,389 | 2,159 / 147,211 | 0.342 |
| json-1k.json | 1,067 | 5 / 18,776 | 136 / 81,476 | 131 / 33,271 | 11 / 27,031 | 0.332 |
| json-10k.json | 10,066 | 9 / 20,696 | 1,345 / 674,146 | 1,324 / 176,458 | 18 / 108,494 | 0.161 |
| json-100k.json | 100,039 | 12 / 35,032 | 13,240 / 6,411,000 | 13,097 / 1,565,922 | 24 / 885,391 | 0.138 |
| json-1m.json | 1,000,112 | 15 / 149,720 | 132,210 / 523,358,132 | 130,814 / 15,205,081 | 30 / 8,397,624 | 0.016 |
| operators-atoms-512.txt | 1,023 | 4 / 18,712 | 1,567 / 511,862 | 1,550 / 203,591 | 14 / 127,775 | 0.250 |

定常状態の認識は 11 / 11 入力で before と確保回数・byte とも同じ。

### 3.2 cold（`reuse_buffers: false`。プロセスで最初の parse と同じ）

| 入力 | before 認識 | after 認識 | before 所有 AST | after 所有 AST | after 木 |
|---|---:|---:|---:|---:|---:|
| complex.tiny | 297,573 | 204,165 | 366,713 | 247,091 | 225,801 |
| complex-x4.tiny | 1,055,874 | 693,386 | 1,282,212 | 843,333 | 759,151 |
| complex-x16.tiny | 4,155,085 | 2,704,517 | 5,008,349 | 3,274,450 | 2,938,712 |
| complex-x64.tiny | 27,664,421 | 11,497,317 | 31,035,317 | 13,751,618 | 12,409,656 |
| flat-arithmetic.tiny | 1,008,950 | 609,222 | 1,383,588 | 928,161 | 834,112 |
| large-match.tiny | 2,510,927 | 1,350,383 | 2,937,364 | 1,631,941 | 1,463,595 |
| json-1k.json | 273,652 | 18,808 | 822,556 | 222,523 | 210,647 |
| json-10k.json | 2,340,744 | 20,728 | 6,057,350 | 1,419,102 | 1,289,294 |
| json-100k.json | 24,641,732 | 35,064 | 75,880,328 | 9,819,906 | 8,519,631 |
| json-1m.json | 76,538,416 | 149,752 | 523,339,420 | 97,123,033 | 84,115,864 |
| operators-atoms-512.txt | 264,612 | 18,744 | 2,460,158 | 1,026,079 | 874,495 |

## 4. 1 節点あたりの byte と理論下限

AST 層の確保（定常状態の「AST あり − 同じ側の認識」）を型付き節点数で割った値。「全節点」は型付き節点 + Text 節点
（Text は所有形の `Ast::Text` / `String` field に当たる）。理論下限は issue の定義どおり
**全節点 × (span 8 + kind 2 + 子の範囲 8) = 18 byte** を型付き節点あたりに直したもの。

| 入力 | 型付き節点 | 全節点 | before 所有 | after 所有 | after 木（確保） | 木の実使用（節点 24 B + slot 4 B） | 入力の複製 | 理論下限 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| complex.tiny | 131 | 165 | 346 | 177 | 64 | 45.4 | 2.5 | 22.7 |
| complex-x4.tiny | 515 | 654 | 332 | 173 | 59 | 45.7 | 2.5 | 22.9 |
| complex-x16.tiny | 2,051 | 2,610 | 327 | 168 | 54 | 45.8 | 2.5 | 22.9 |
| complex-x64.tiny | 8,195 | 10,434 | 327 | 167 | 53 | 45.8 | 2.6 | 22.9 |
| flat-arithmetic.tiny | 515 | 1,026 | 399 | 216 | 203 | 73.7 | 2.8 | 35.9 |
| large-match.tiny | 1,038 | 1,297 | 325 | 167 | 53 | 46.7 | 1.3 | 22.5 |
| json-1k.json | 109 | 162 | 575 | 133 | 76 | 44.0 | 9.8 | 26.8 |
| json-10k.json | 1,187 | 1,779 | 551 | 131 | 74 | 44.4 | 8.5 | 27.0 |
| json-100k.json | 11,886 | 17,828 | 536 | 129 | 72 | 44.4 | 8.4 | 27.0 |
| json-1m.json | 118,896 | 178,343 | 4401 | 127 | 69 | 44.4 | 8.4 | 27.0 |
| operators-atoms-512.txt | 1,025 | 2,048 | 481 | 180 | 106 | 63.9 | 1.0 | 36.0 |

読み方:

- before の JSON 1 MB（4,401 byte / 節点）は event arena の作り直しを含む（§1）。100 KB 以下の before は pool が効いた値で、
  そのうち `node_spans` の見積り確保（入力 byte / 2 × 56 byte）が JSON で 1 節点あたり約 240 byte を占めていた。
- after 所有の 127〜180 byte は、所有 `Ast` そのもの（`Box<Ast>` 56 byte・`String`・`Vec` の要素 56 byte）と
  `node_spans`（56 byte）。所有形を返す以上ここは下がらない。
- 木の実使用（44〜47 byte、Operators / flat-arithmetic は 64〜74 byte）と理論下限（23〜36 byte）の差は、
  節点が 18 byte ではなく 24 byte であること（規則番号と `node_id` を持つので `node_spans` を後から導出できる）と、
  optional field の空 slot・list の見出し 2 slot。
- flat-arithmetic の木（確保 203 byte / 節点）は見積りを外した例。数値が leaf 型へ昇格して節点が capture 数より
  33% 多く、節点と slot の `Vec` が 1 回ずつ倍になった（41,568 + 16,496 byte の realloc）。入力 1.4 KB の話で、
  JSON / complex 系では見積り（capture 数 × 1.125 と、規則ごとの record の slot 数の和）が 7〜12% の余裕で当たる。

確保回数（型付き節点あたり）:

| 入力 | before 所有 | after 所有 | after 木 |
|---|---:|---:|---:|
| complex.tiny | 1.41 | 1.40 | 0.0458 |
| complex-x4.tiny | 1.39 | 1.39 | 0.0136 |
| complex-x16.tiny | 1.38 | 1.38 | 0.0034 |
| complex-x64.tiny | 1.38 | 1.38 | 0.0010 |
| flat-arithmetic.tiny | 2.03 | 2.01 | 0.0214 |
| large-match.tiny | 1.32 | 1.32 | 0.0067 |
| json-1k.json | 1.20 | 1.16 | 0.0550 |
| json-10k.json | 1.13 | 1.11 | 0.0076 |
| json-100k.json | 1.11 | 1.10 | 0.0010 |
| json-1m.json | 1.11 | 1.10 | 0.0001 |
| operators-atoms-512.txt | 1.52 | 1.51 | 0.0098 |

## 5. 時間

`benchmarks/scripts/run-arena-ab.sh time`（`taskset -c 30`、1 run 0.5 秒、7 周。周ごとに 5 変種を順番を回して交互に
走らせる）。2 セッション続けて測った。host は他のエージェントの build が並行していたので、計測直前に
`/proc/stat` で core の使用率を見て静かなときに走らせた（S1 開始時の load 8.3、S2 終了時 3.8）。
ms/parse。各セルは「中央値 / 最小値」、2 セッション（S1 / S2）。変化率は最小値どうし（S1, S2）。

| 入力 | before 認識 | after 認識 | before 所有 AST | after 所有 AST | after 木 | 所有 AST 変化 | 木 ÷ before AST |
|---|---:|---:|---:|---:|---:|---:|---:|
| complex.tiny | 0.0668 / 0.0635<br>0.0638 / 0.0634 | 0.0655 / 0.064<br>0.0657 / 0.064 | 0.0894 / 0.0869<br>0.0914 / 0.0865 | 0.0941 / 0.0869<br>0.0918 / 0.0888 | 0.0827 / 0.0777<br>0.0822 / 0.0791 | +0.0%, +2.7% | 0.89, 0.91 |
| complex-x4.tiny | 0.2668 / 0.2524<br>0.258 / 0.2508 | 0.264 / 0.2571<br>0.2594 / 0.2545 | 0.3545 / 0.3391<br>0.3325 / 0.3279 | 0.3563 / 0.3449<br>0.3516 / 0.3365 | 0.3092 / 0.3054<br>0.3084 / 0.3003 | +1.7%, +2.6% | 0.90, 0.92 |
| complex-x16.tiny | 1.175 / 1.091<br>1.045 / 1.022 | 1.205 / 1.08<br>1.055 / 1.038 | 1.495 / 1.408<br>1.333 / 1.319 | 1.453 / 1.395<br>1.343 / 1.331 | 1.345 / 1.251<br>1.233 / 1.219 | -0.9%, +0.9% | 0.89, 0.92 |
| complex-x64.tiny | 4.948 / 4.378<br>4.444 / 4.369 | 4.896 / 4.432<br>4.417 / 4.281 | 7.717 / 6.053<br>5.856 / 5.705 | 6.521 / 5.904<br>5.622 / 5.497 | 6.122 / 5.152<br>5.236 / 5.008 | -2.5%, -3.6% | 0.85, 0.88 |
| flat-arithmetic.tiny | 0.0798 / 0.0775<br>0.0806 / 0.0789 | 0.0808 / 0.0763<br>0.0771 / 0.0762 | 0.1574 / 0.1556<br>0.1565 / 0.156 | 0.1579 / 0.1515<br>0.1494 / 0.1486 | 0.116 / 0.1133<br>0.1128 / 0.1118 | -2.6%, -4.7% | 0.73, 0.72 |
| large-match.tiny | 0.3513 / 0.3456<br>0.3599 / 0.3401 | 0.3518 / 0.3482<br>0.3473 / 0.3414 | 0.4805 / 0.4739<br>0.4784 / 0.4648 | 0.4891 / 0.4824<br>0.4783 / 0.4669 | 0.4251 / 0.4192<br>0.4311 / 0.4078 | +1.8%, +0.5% | 0.88, 0.88 |
| json-1k.json | 0.0129 / 0.0127<br>0.0127 / 0.0126 | 0.0127 / 0.0125<br>0.0125 / 0.0125 | 0.028 / 0.0275<br>0.0276 / 0.0272 | 0.0282 / 0.0279<br>0.0281 / 0.0276 | 0.0244 / 0.024<br>0.0244 / 0.0242 | +1.5%, +1.5% | 0.87, 0.89 |
| json-10k.json | 0.1322 / 0.1303<br>0.1326 / 0.1304 | 0.1318 / 0.1303<br>0.1313 / 0.1299 | 0.3399 / 0.3284<br>0.3401 / 0.3311 | 0.3157 / 0.3113<br>0.3094 / 0.3082 | 0.2663 / 0.2633<br>0.268 / 0.2613 | -5.2%, -6.9% | 0.80, 0.79 |
| json-100k.json | 1.425 / 1.399<br>1.401 / 1.384 | 1.314 / 1.293<br>1.298 / 1.285 | 4.802 / 4.009<br>4.435 / 4.016 | 3.293 / 3.273<br>3.294 / 3.2 | 2.843 / 2.821<br>2.845 / 2.784 | -18.4%, -20.3% | 0.70, 0.69 |
| json-1m.json | 13.29 / 13.2<br>13.17 / 13.09 | 12.91 / 12.84<br>12.85 / 12.82 | 162.7 / 159.6<br>162.4 / 158.4 | 57.5 / 55.6<br>56.27 / 55.26 | 52.22 / 50.62<br>51.28 / 49.92 | -65.2%, -65.1% | 0.32, 0.32 |
| operators-atoms-512.txt | 0.1021 / 0.1018<br>0.0994 / 0.0989 | 0.0986 / 0.0978<br>0.1002 / 0.0998 | 0.3146 / 0.2697<br>0.262 / 0.2493 | 0.2397 / 0.2391<br>0.243 / 0.2403 | 0.1955 / 0.1942<br>0.1968 / 0.1946 | -11.3%, -3.6% | 0.72, 0.78 |

load average（1 分値、計測開始 → 終了）: S1 8.28 → 5.26、S2 5.26 → 3.75。

読み方:

- **P4 complex の所有 AST は変わらない**（この表で +0.0% / +2.7%。この PR の途中の build を含めて別に 4 セッション
  測った値は −3.4%〜+0.9% で、符号が揃わない。profile 報告の雑音帯 ±1.5% よりやや広い ±3%）。**木は −9〜−11%**。
  x64 の所有 AST は −2.5% / −3.6%、木は −15% / −12%。issue の目標「x64 AST ≤ 6 ms」は所有 5.50〜5.90 ms・
  木 5.01〜5.15 ms で満たす（issue 本文の 7.39 ms は 2026-09-23 以前の値で、before 自体が 5.71〜6.05 ms）。
- JSON は入力が大きいほど効く。所有 AST は 10 KB 以上で速くなり、1 KB は +1.5%（写像で `Box` / `String` を
  作る分）。木は全サイズで before の所有 AST より速い。
- 認識は −8〜+2%。JSON 100 KB の −7% / 1 MB の −2〜−3% は memo 表の遅延確保の分（before は JSON が 1 度も使わない
  memo 表を pool から借りて毎 parse ゼロ埋めしていた。100 KB で 4 MB）。P4 は scope を持つので認識でも event を
  記録し、詰め替えの費用が載るが、x64 で −2〜+1%。
- **途中の版で JSON 1 KB の所有 AST が +10% だった**。event の記録判定（`event()` の先頭）と Token の付け替えの
  判定を呼出し側へ inline し、記録本体を別関数にしたら消えた（D-077 の (3)、`record_event`）。
  詰めた event を読む `unpack` が Recovery の本体（大きな値）を返す形だったときは、JSON 1 KB の木の経路が
  詰めない版より約 3.5 µs（parse の 13%）遅かった。Recovery の本体を別表へ出して `unpack` が scalar だけを
  返すようにした。

## 6. 検証

| 項目 | 結果 |
|---|---|
| canonical JSON golden | `examples/p4-rust/tests/golden` の 9 件（Formula 入口）と BooleanExpression 入口 4 件は byte 不変（`formula_accepts_and_matches_golden` ほか）。**新規** `arena_tree_matches_owned_ast_and_golden`: 木だけの parse の `canonical_json()` が golden と byte 一致し、`to_ast()`・`node_spans()`・`value_spans()`・型付き参照の field が所有形と同じ。Java golden との完全一致（`java_and_rust_goldens_are_identical_without_normalization`）も不変 |
| JSON | JSONTestSuite `test_parsing` の UTF-8 として読める 293 件（受理 116）と json-1k〜1m で、before（`618225a` の生成器）の所有 AST と after の所有 AST・木の canonical JSON / `node_spans` / `value_spans` / 診断 / `consumed_cp` が byte 一致（scratch の比較 bin。§8 の A/B driver と同じ生成物） |
| 生成 crate の契約 | `crates/ubnfc-rust/tests/generated.rs` の生成 main が、入力ごとに木だけの parse の canonical 値と所有 AST の canonical 値の一致を assert する（Operators / Power / tiny の全 golden 入力） |
| corpus | `ubnfc-harness run-all --backend rust`: **Pass 1,292** / 期待拒否 Pass 29 / Known 29 / **Fail 0** / 未観測 75。`--backend java`: **Pass 1,310** / 27 / 29 / **Fail 0** / 59。どちらも master（#79 の報告）と同じ表 |
| 差分 fuzz | `crates/ubnfc-fuzz/tests/campaign.rs` の固定 seed 7 件を含む workspace test が green |
| self-host | `crates/ubnfc-selfhost` の parity と regenerate（生成物の byte 一致・IR と Rust の固定点）が green。生成物は再生成して commit |
| stax | `examples/stax/scripts/crosscheck.sh`: 20 本で両 backend の受理・canonical AST・スコープ診断が一致 |
| workspace | `cargo test --workspace --offline --no-fail-fast` 537 passed / 0 failed、`cargo clippy --workspace --all-targets -D warnings`、`cargo fmt --check` |

途中で 2 つ落ちたものを直した: (1) 生成 driver は runtime を `#[path]` で別 crate に取り込むので、runtime が呼ぶ
木の構築用メソッドを `pub(crate)` にすると harness の build が落ちる（`#[doc(hidden)] pub` にした）。
(2) list field の無い文法で value span 走査の `use std::fmt::Write` が未使用になり、生成 crate の
clippy（`-D warnings`）が落ちる（完全修飾の `write_fmt` にした）。

## 7. 互換と移行

**表現の選び方。** issue は (a) 節点の arena + 子の `u32` 添字 + 文字列は入力の span、(b) 既存の enum の
`Box` を添字に・`String` を span に置き換える、(c) cargo feature で表現を切り替える、を挙げていた。
(a) を採り、**所有 `Ast` は残してその写像にした**。(b) は `Ast` の型を変えるので既存の利用者
（tinyexpression-rs の vendored P4、self-host の projection、生成 evaluator、driver の `ToJson`）が全部壊れる。
(c) は mapping の生成コードを 2 系統持つことになる。写像にすれば mapping は 1 本のまま、所有形は
一字一句そのまま残せる。**generator flag は要らなかった**（既定の生成物で所有 API は不変）。

**所有 API（不変）。** 生成 `ast.rs` の所有部分（`Ast` 列挙・型ごとの struct と `pub` field・`span` /
`type_name` / `type_id` / `common_*` / `canonical_value` / `canonical_json` / `Serialize`）と `evaluator.rs` は
P4 で `618225a` と byte 一致する。`ParseResult::ast` / `node_spans` / `value_spans` の値も不変（§6）。

**追加した API。**

| 追加 | 内容 |
|---|---|
| `ParseOptions::ast_tree: bool`（既定 false） | 木を `ParseResult::tree` で返す。`build_ast` とは独立（両方 false なら認識だけ） |
| `ParseResult::tree: Option<AstTree>` | AST を作れたときだけ `Some` |
| `ast::tree` module / `AstTree`（crate root にも再公開） | `root()` / `node(id)` / `to_ast()` / `node_spans()` / `value_spans()` / `canonical_json()`（`json` feature）/ `source()` / 大きさ（`node_count` / `typed_node_count` / `slot_count` / `heap_bytes`） |
| `tree::NodeRef<'t>` / `tree::Node<'t>` / 型ごとの `tree::g_…<'t>` | `get()` で所有形と同じ形の列挙へ分け、型ごとの参照は field と同名のメソッド（scalar は `NodeRef`、optional は `Option`、list は `NodeList` / `TextList`、文字列は `&'t str`）と `span()` / `node_id()` / `to_ast()`。所有形の `common_*` に当たる accessor は木の側には無い（`to_ast()` で写す） |
| `ParseOptions::wants_ast()` | `build_ast || ast_tree`（runtime が使う） |

`ParseOptions` / `ParseResult` を**全 field を列挙した struct literal や分解**で書いているコードは、
field が増えたので compile error になる（D-035 で `occurrences` / `value_spans` を足したときと同じ種類の変更）。
`..Default::default()` で作るコードは影響を受けない。木の構築用メソッド（`text` / `record` / `list` /
`project` など）は、driver が runtime を `#[path]` で別 crate に取り込むため crate 内可視にできず
`#[doc(hidden)] pub` にしてある（利用者は呼ばない）。

**生成物の大きさ。** 木の参照型・写像・canonical・value span の走査が文法ごとに増える。P4 の生成 `ast.rs` は
1,040 → 3,637 行。

**tinyexpression-rs（tinyexpression PR #184 で P4 parser を vendoring）。** `ParseOptions::default()` と
`result.ast` しか使っていない（`src/frontend.rs`）ので、**再 vendor するだけで動く**（コードの変更は不要）。
所有 `Ast` から自前の AST へ写している（`frontend.rs::project`）ので、木を使うと所有 `Ast` の確保
（型付き節点 1 個あたり 1.38 回・167 byte、§4）を丸ごと省ける: `ParseOptions { build_ast: false, ast_tree: true, .. }`
で受け取り、`tree.root().get()` から型ごとの参照で自前の AST を組む。これは任意の後続作業で、この PR の
完了条件ではない。

## 8. 再現

```sh
WORK=$(mktemp -d) BASE=618225a benchmarks/scripts/run-arena-ab.sh prepare
WORK=$WORK benchmarks/scripts/run-arena-ab.sh alloc
WORK=$WORK benchmarks/scripts/run-arena-ab.sh tree
WORK=$WORK CORE=30 RUNS=7 benchmarks/scripts/run-arena-ab.sh time
```

生の値は `examples/json/evidence/arena-59-*.csv`。
