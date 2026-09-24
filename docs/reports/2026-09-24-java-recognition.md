# Java の認識そのものを詰める（2026-09-24、issue #22）

[`2026-09-24-java-ast-path.md`](2026-09-24-java-ast-path.md) が「AST 層を 0 円にしても M5 の目標に届かない、
残りは**認識そのもの**」と計数で確定した。本ラウンドは認識専用経路（`ParseOptions` が出現表も診断も
要求しない = D-056 / D-057 の二段構え）だけを対象に、**Rust backend と同じ回数表を並べて取り**、
単価の差が大きい順に 1 施策ずつ A/B した。**`crates/` は 1 行も触っていない**（front / Rust backend は不変）。

## 結論（先に）

- **P4 complex の認識専用は 0.2465 → 0.1357 ms（中央値 0.55 倍）/ 0.2097 → 0.1025 ms（最小 0.49 倍）、
  complex-x64 は 15.73 → 7.57 ms（0.48 倍）/ 14.85 → 7.14 ms（0.48 倍）**（binary 交互 3 周、§5.4）。
  **issue の目標（complex ≤0.12 ms / x64 ≤9 ms）は x64 が達成、complex は最小値で達成・中央値で 13% 超**（§6）。
  同じ機械の同じ日の Rust は complex 0.088 / 0.076 ms、x64 6.76 / 6.41 ms なので、
  **認識の Java / Rust 比は 3.3 倍 → 1.35〜1.54 倍（complex）/ 1.11〜1.12 倍（x64）**になった。
- **3.3 倍の内訳は回数ではなく単価だった**。1 parse の回数は両 backend でほぼ同じ（rule 本体評価 Java 612 /
  Rust 586、memo 保存 608 / 586、literal 627 / 548）で、**gap 144 µs の 87% は上位 6 操作の単価差**で説明できる:
  memo 探索 +35 µs、scope 巻戻し +24 µs、**capture の包み +22 µs（Rust には存在しない層）**、連接 +11 µs、
  rule 入口 +9 µs、trivia +7 µs（§3）。
- **最大の 1 手は「認識経路で恒等になる capture の包みを、生成側で飛ばす」**（D-059）。
  1 parse 2,983 回の 14 引数呼出しと同数の interface 呼出し（`Call.parse`）が消え、
  **interface 経由の呼出しは 5,962 → 2,992 回**になる。これ単独で complex 0.61 倍・x64 0.64 倍（§5.2）。
- 次点は **memo 表の世代印**（表を消すのに容量ぶんの書込みをしない。x64 では 1 parse あたり 8 MB を
  触っていた）と**容量の適正化**（入力長 ×8 → ×4）、**scope 取消しの早抜け**（2,070 回中 1,930 回が空振り）、
  **trivia の先頭 1 文字判定**（区切り試行 4,005 → 102 回）。
- **回数の差は 1 か所だけ本物**: Java は rule 関数を 1,076 回呼ぶのに Rust は 759 回で、差はすべて
  memo hit（464 対 169）。原因は **Java backend に左因数分解が無い**こと（`crates/ubnfc-rust/src/factor.rs`
  に対応するものが `ubnfc-java` に無い。生成 Rust は 18 群を因数分解している）。**本 PR の範囲外**で、
  次の一手として §7 に残す。
- 観測は動いていない: `TwoModeEquivalenceTest` 2,652 件 / `PredictEquivalenceTest` 5,304 件、
  canonical JSON golden 13 件、corpus Java Pass 1,310 / Fail 0（系統別表は
  [`2026-09-24-java-corpus-ast19.md`](2026-09-24-java-corpus-ast19.md) と byte 一致）、IR golden 不変。

## 0. 計測条件

| 項目 | 値 |
|---|---|
| 基準 | `origin/master` `790622a`（PR #24 = 規則ごとの memo plan D-058 を取り込んだ後） |
| host | AMD Ryzen 9 7950X（32 論理 CPU）、86 GB、WSL2 Linux 6.18.33.2 |
| Java | Oracle JDK 21.0.9 / HotSpot、`-Xms512m -Xmx2g -Xss16m`、`taskset -c 28`、単一 thread |
| Rust | `rustc 1.98.1`、`--release --offline`、`taskset -c 28`（`examples/p4-rust` の `bench` example） |
| 負荷 | **共有機で別エージェントが並走**し、作業を通じて load average は 6.5〜13 で動いた。各表に採取時の値を書く |
| A/B | **binary 交互**（`origin/master` の生成物と本 PR の生成物を 1 round ごとに交互に走らせる。1 JVM の中では全 cell を warmup してから 1 run ごとに全 cell を巡回する `Benchmark` をそのまま使う） |
| 集計 | round 中央値の中央値と、**全 run の最小値の中央値**。共有機なので採否は最小値で見る |
| mode | `recognize-fast`（`P4Scanners.recognize()` = 出現表も診断も要求しない） |
| 回数 | 生成物の複製に計数を挿した一時 binary（`mktemp -d` 配下。commit には含めない）。計時は歪むので回数だけ使う |
| 単価 | JFR `jdk.ExecutionSample`（1 ms 周期）の**葉フレーム**の割合 × 1 parse の時間 ÷ 回数 |

## 1. 回数 — Java と Rust を並べる（1 parse、complex 332 byte、認識専用）

Rust は `examples/p4-rust` の `counters` feature、Java は同等のフックを生成物の複製に挿して数えた。

| 計数 | Java `origin/master` | Java 本 PR | Rust | 備考 |
|---|---:|---:|---:|---|
| rule 関数呼出し（memo hit 込み） | 1,076 | 1,076 | 759 | **差は左因数分解の有無**（§7） |
| うち本体を評価 | 612 | 612 | 586 | **ほぼ同じ** |
| memo 参照（probe） | 1,680 | 1,680 | 1,341 | Rust は lookup 755 + insert 586 |
| memo hit | 464 | 464 | 169 | 上と同じ理由 |
| memo 保存 | 608 | 608 | 586 | **ほぼ同じ** |
| memo の線形探索の追加歩数 | 668 | 1,401 | （max probe 5） | 表を半分の大きさにした代償（§4.1） |
| 印（mark） | 2,693 | 2,693 | 1,971 | |
| 巻戻し（restore） | 2,070 | 2,070 | 2,748 | |
| scope 取消し記録の追加 | 157 | 157 | 66 | Rust は `effect_at` |
| scope 取消しの巻戻し | 140 | 140 | — | 巻戻し 2,070 回中 140 回しか記録が無い |
| trivia 呼出し | 1,301 | 1,301 | （式に inline） | |
| trivia の区切り試行 | 4,005 | **102** | — | 先頭 1 文字判定（§4.3） |
| choice 呼出し / 候補評価 | 301 / 1,058 | 301 / 1,058 | — | |
| 候補除外（predict） | 951 | 951 | 819 | Rust は失敗再生関数 `guard_e*` の呼出し |
| literal | 627 | 627 | 548 | |
| literal の文字比較 | 735 | 735 | — | |
| capture の包み | 2,983 | 2,983（うち 2,970 は迂回） | **0** | **Rust にこの層は無い** |
| **interface 経由の呼出し `Call.parse`** | **5,962** | **2,992** | **0** | Rust は直接呼出し（単相化） |
| `Match` 確保 | 658 | 658 | — | Rust は event 1,308 |
| 失敗記録 `fail()` | 1,982 | 1,982 | 457 | Java は `diag=false` でも `reached` 更新のため呼ぶ |
| `cp()` | 1,854 | 553 | — | trivia の早抜けで減った |
| 1 parse の割当 | 74,168 B | 72,768 B | 31,829 B | complex 単独 JVM の値 |

x64（20,923 byte）も同じ形（Java 本 PR: rule 65,210 / 本体 36,711 / probe 101,535 / hit 28,499 / 保存 36,518 /
巻戻し 125,172 / trivia 78,728（区切り試行 242,712 → 6,528）/ capture 181,021 / `Call.parse` 361,660 → 181,471。
Rust: rule 46,119 / lookup 45,926 / hit 10,564 / insert 35,362 / 巻戻し 166,800 / literal 33,812）。

**回数が違う唯一の本物**は rule 関数呼出し（1,076 対 759）で、差 317 はすべて memo hit（464 − 169 = 295）に
出ている。**本体の評価回数は 612 対 586 でほぼ同じ**なので、算法段階（predict・restricted memo）は揃っていて、
Java だけが「同じ位置の同じ規則をもう一度呼んで memo で返す」経路を通っている。原因は §7。

## 2. 単価（JFR、`origin/master`、complex 認識 0.220 ms / parse、10,158 sample）

葉フレームの割合をそのまま自己時間とみなす。

| 操作（葉フレーム） | 自己 % | 自己 µs | 回数 | Java 単価 |
|---|---:|---:|---:|---:|
| `MemoTable.locate`（memo 探索） | 17.8 | 39.1 | 1,680 | **23.2 ns** |
| `ScopeStore.restore`（scope 巻戻し） | 13.2 | 29.0 | 2,070 | **14.0 ns** |
| `Session.capture`（capture の包み） | 10.2 | 22.5 | 2,983 | **7.6 ns** |
| `Session.sequence` | 6.0 | 13.3 | 759 | 17.5 ns |
| `Session.rule` | 5.5 | 12.2 | 1,076 | 11.3 ns |
| `Session.trivia` | 4.4 | 9.7 | 1,301 | 7.5 ns |
| `Session.choice` | 2.6 | 5.8 | 301 | 19.3 ns |
| `Session.replay`（memo payload の効果再生） | 2.6 | 5.7 | 約 700 | 8.1 ns |
| `Input.codePointAt` | 2.3 | 5.1 | 1,854 | 2.8 ns |
| `String.hashCode`（scope 表の鍵） | 1.7 | 3.8 | — | |
| `Arrays.binarySearch`（`Input.cpIndex`） | 1.5 | 3.3 | — | |
| `Session.literal` | 1.4 | 3.1 | 627 | 5.0 ns |

## 3. 3.3 倍の内訳 — Σ(回数 × Δ単価)

Rust 側の単価は [`2026-09-23-p4-rust-profile.md`](2026-09-23-p4-rust-profile.md) §1 の実測
「**この parser は入力 1 byte あたり 111 回の runtime 操作を行い、1 操作あたり 2.4〜2.6 ns**（complex と
complex-x64 でこの単価は 7% しか違わない）」を使う。つまり Rust 側は「どの操作もおよそ 2.5 ns」である。

複素 complex の gap は **220 µs（Java）− 76 µs（Rust、同日同機の最小値）= 144 µs**。

| 操作 | 回数 | Java 単価 | Rust 単価 | Δ 単価 | **Δ × 回数** |
|---|---:|---:|---:|---:|---:|
| memo 探索 | 1,680 | 23.2 ns | 2.5 ns | 20.7 ns | **+34.8 µs** |
| scope 巻戻し | 2,070 | 14.0 ns | 2.5 ns | 11.5 ns | **+23.8 µs** |
| capture の包み | 2,983 | 7.6 ns | **0（層が無い）** | 7.6 ns | **+22.5 µs** |
| 連接 `sequence` | 759 | 17.5 ns | 2.5 ns | 15.0 ns | **+11.4 µs** |
| rule 入口 | 1,076 | 11.3 ns | 2.5 ns | 8.8 ns | **+9.5 µs** |
| trivia | 1,301 | 7.5 ns | 2.5 ns | 5.0 ns | **+6.5 µs** |
| 小計 | | | | | **+108.5 µs（gap の 75%）** |
| 効果再生・choice・`codePointAt`・`cpIndex`・`String.hashCode` | | | | | **+18 µs（12%）** |
| **合計** | | | | | **+126 µs（87%）** |

**結論は「同じ回数を 3〜9 倍高い単価で払っていた」**であって、余計な段階を踏んでいたわけではない
（唯一の例外が capture の包みで、これは Rust に対応物が無い**層そのもの**）。したがって施策は
「回数を減らす」ではなく「単価を下げる」「層を消す」になる。

## 4. 入れたもの

### 4.1 memo 表: 世代印と容量（`MemoTable`）

- **世代印**。slot の 4 語目を `世代(30) | outcome(2) | payload(32)` にして、占有判定を
  「印が今の世代か」にした。`clear()` と解析ごとの初期化が**世代を 1 進めるだけ**になり、
  容量に比例した書込みが消える。従来は解析のたびに全 slot の outcome 語を 0 にしており、
  **complex で 4,096 slot（128 KB を跨ぐ striding write）、x64 で 262,144 slot（8 MB）**を触っていた。
  thread local の世代は単調増加なので、pool から回ってきた配列に残る印は必ず古い世代になる
  （2^30 回で折り返すときだけ pool を捨てて 1 から振り直す）。
- **容量**。初期容量を `入力長 × 8` → `× 4` にした。P4 の実測は **1 文字あたり 1.8 entry**
  （complex 608 / 326、x64 36,518 / 20,923）で、load factor 0.5 の表は長さの 4 倍で足りる。
  表は complex 128 KB → 64 KB、x64 8 MB → 4 MB。**代償**は線形探索の追加歩数 668 → 1,613 で、
  これを **hash の混合を「依存する 2 連乗算」から「独立な 2 乗算の和」に**変えて 1,401 に戻した
  （遅延の連鎖も消える）。1 probe あたり 0.83 歩なので探索そのものは依然ほぼ 1 回で当たる。

### 4.2 scope 取消しの早抜け（`ScopeStore` / `Session`）

巻戻しは 1 parse 2,070 回あるが、**取消し記録が 1 件でも増えているのは 140 回だけ**。
`ScopeStore.restore` の先頭に「checkpoint == 現在の記録数なら何もしない」を置き、
`Session.restore` は**印と現在値が等しければ呼出しごと省く**形に分けて、判定だけが inline されるようにした。
併せて `journal` / `effects` の `ArrayList` を 64 で確保する（1 parse で 157 / 139 件まで伸びるので、
既定容量 10 から 8 回育て直していた）。

### 4.3 trivia の先頭 1 文字判定（`Session.Delimiters`）

`Delimiters` に「どれかの区切りが始まり得る先頭文字」の ASCII bitmap を 1 度だけ畳み込み、
`trivia()` の各周回の先頭で 1 回の bit 試験で弾く。**一般経路が全区切りを失敗したときに登録する
hint は `fullGroup()` 1 件だけ**なので、早抜けは同じ 1 件を登録して返る（診断モードでも同値）。
区切り試行は 4,005 → 102 回、`cp()` は 1,854 → 553 回になった。

### 4.4 認識経路で恒等になる capture の包みを生成側で飛ばす（**D-059**）

`JavaEmitter` は**すべての式**に `s.capture(f, …, 14 引数)` を出していた。認識専用経路
（`tree=false` かつ `diag=false`）で、**scope 効果が読む capture site を 1 つも持たない式**では
`capture()` は「中身を呼ぶ → `progress(f)` → そのまま返す」だけの恒等写像になる。
生成側でその条件を静的に判定し（1,043 式のうち **1,032 式**が該当）、

```java
if (s.bypassCapture) { Match r = f.matched ? eNM(s, f) : eNC(s, f); s.progress(f); return r; }
```

を前置した。`Session` は `bypassCapture = !tree && !diag` を構築時に決める。
**消えるのは 14 引数の呼出しと、同数の interface 呼出し（`Call.parse`）**で、
1 parse の interface 経由の呼出しは **5,962 → 2,992 回**（x64 は 361,660 → 181,471）。
残る 11 式（scope 効果が読む site を持つもの）は従来どおり包みを通る。
**観測は 1 bit も変わらない**: 到達位置の更新（`progress`）は同じ位置で行い、
失敗 hint は `diag` のときだけ記録されるので `!diag` の経路には無く、`tree=true`（AST・出現表・lexical を
要求する経路）はこの分岐に入らない。

### 4.5 memo payload の効果列に null 番兵（`Session`）

memo hit のたびに `replay(payloads.effects[payload])` を呼んでいたが、**効果を持つ payload は数件**で
大半は空 list だった。`effectsSince` が空なら `null` を返し、`replay` は null 検査だけの小さな本体にして
inline させ、実体（`replayAll`）を分けた。

## 5. A/B（binary 交互、`bench.runs=7`、`bench.warmup=1000`）

「中央値」= round 中央値の中央値、「最小」= 各 round の最小値の中央値。

### 5.1 群 1（memo 世代印 + 容量 ×4 + scope 早抜け + trivia 先頭文字 + 予備確保）— 3 周、load 6.5〜12.5

| fixture | base 中央値 | 本 PR 中央値 | 比 | base 最小 | 本 PR 最小 | 比 | base 割当 | 本 PR 割当 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| complex | 0.2695 | 0.2435 | **0.90** | 0.2240 | 0.2072 | **0.92** | 205,616 | 138,776 |
| complex-x64 | 16.440 | 13.222 | **0.80** | 14.233 | 12.327 | **0.87** | 4,196,920 | 4,110,400 |

（割当の差は「complex と x64 を同じ JVM に載せると pool が x64 の表に占有され、complex が毎回
表を確保し直す」ぶん。容量を半分にしたのでその確保も半分になる。単独 JVM では両者とも pool に当たる。）

### 5.2 群 2（capture の包みの迂回を追加）— 3 周、load 7.8〜9.3

| fixture | base 中央値 | 本 PR 中央値 | 比 | base 最小 | 本 PR 最小 | 比 |
|---|---:|---:|---:|---:|---:|---:|
| complex | 0.2398 | 0.1316 | **0.55** | 0.2080 | 0.1068 | **0.51** |
| complex-x64 | 14.489 | 7.437 | **0.51** | 12.924 | 6.806 | **0.53** |

群 1 との比で **capture 迂回の単独効果は complex 0.61 倍・x64 0.64 倍**。

### 5.3 群 3（巻戻しの inline 判定 + 効果列 null + hash の独立乗算）— 群 1+2 対 最終、4 周、load 8.4〜13

| fixture | 群1+2 中央値 | 最終 中央値 | 比 | 群1+2 最小 | 最終 最小 | 比 |
|---|---:|---:|---:|---:|---:|---:|
| complex | 0.1475 | 0.1353 | **0.92** | 0.1115 | 0.1020 | **0.91** |
| complex-x64 | 7.936 | 7.762 | 0.98 | 7.210 | 7.118 | 0.99 |

complex で 9% 効き、x64 は雑音の中（±3.5%）。**採用**（complex に効き、x64 で悪化しない）。

### 5.4 通し（`origin/master` 対 最終）— 3 周、load 7.5〜9.4

| fixture | base 中央値 | 最終 中央値 | 比 | base 最小 | 最終 最小 | 比 |
|---|---:|---:|---:|---:|---:|---:|
| complex | 0.2465 | 0.1357 | **0.55** | 0.2097 | 0.1025 | **0.49** |
| complex-x64 | 15.733 | 7.573 | **0.48** | 14.851 | 7.140 | **0.48** |

### 5.5 単独 JVM（complex のみ、`bench.iterations=8000`、`bench.runs=12`、load 7.2）

| 版 | 中央値 ms | 最小 ms | 割当 B |
|---|---:|---:|---:|
| `origin/master` | 0.2345 | 0.2196 | 74,136 |
| 本 PR | **0.1178** | **0.1105** | 72,768 |

## 6. 目標に対する到達

同じ機械・同じ日の Rust（`examples/p4-rust` の bench、`taskset -c 28`、load 12〜13）は
**complex 認識 0.088 ms（中央値）/ 0.076（最小）、complex-x64 認識 6.759 / 6.408 ms**。

| 目標（issue #22） | 本 PR | 判定 |
|---|---|---|
| P4 complex 認識 ≤ 0.12 ms | **0.1105〜0.1357**（単独 JVM 中央値 0.1178 / 最小 0.1105、通し中央値 0.1357 / 最小 0.1025） | **最小値と単独 JVM の中央値で達成**、複数 fixture を同居させた通しの中央値は 13% 超 |
| P4 complex-x64 認識 ≤ 9 ms | **7.57 ms（中央値）/ 7.14（最小）** | **達成** |
| 認識の Rust 比 | complex 1.54 倍（中央値）/ 1.35（最小）、x64 1.12 / 1.11 | **3.3 倍 → 1.1〜1.5 倍** |

**M5 の AST 目標（complex ≤0.2 ms / x64 ≤15 ms）への含意**: 認識が complex 0.118 ms / x64 7.6 ms、
AST 層の費用が [`2026-09-24-java-ast-path.md`](2026-09-24-java-ast-path.md) の計数で complex 0.089 ms /
x64 10.5 ms なので、**complex は AST でも目標圏内に入る見込みだが x64 は AST 層の側がなお 3 ms 超過**する。
AST 層（`Match` → `Recipe` → typed AST の 3 段）は本 PR の範囲外で、D-057 §6 の残りのまま。

## 7. 残ギャップ — Java backend に左因数分解が無い

§1 の唯一の回数差（rule 関数呼出し Java 1,076 対 Rust 759、memo hit 464 対 169）は、
**Rust backend が共通接頭辞を持つ連続候補を 1 回だけ評価する左因数分解**
（`crates/ubnfc-rust/src/factor.rs`、`ParseOptions::factor`。生成 P4 parser に 18 群）を持ち、
**`ubnfc-java` には対応するものが無い**ことによる（`JavaEmitter` / `Prediction` に `factor` の文字列が 1 つも無い）。
Java は候補ごとに共有接頭辞の規則を呼び直し、memo で返している。代金は complex 1 parse あたり
**約 295 回の memo 参照 + payload 再生**で、§2 の単価（23.2 ns）で見ると **7 µs（現在の 0.11 ms の 6%）**。
これは「単価を下げる」本ラウンドの作法に収まらない**生成側の段階の追加**なので、別 issue に回す。

残りの Java 固有の下駄は **interface 経由の呼出し 2,992 回**（Rust は単相化した直接呼出し）で、
これを減らすには連接・選択の骨組みを生成コード側へ展開する（Rust backend の形）ことになる。

## 8. 同値性の検証

- `examples/p4-java` の **`TwoModeEquivalenceTest` 2,652 件**（884 入力 × 入口 3 種）と
  **`PredictEquivalenceTest` 5,304 件**が緑。前者は二段構えの観測要求あり / なしを突き合わせる。
- canonical JSON golden **13 件**と `java_and_rust_goldens_are_identical_without_normalization` が無変更で緑。
- corpus: `cargo run -q --offline --bin ubnfc-harness -- run-all --backend java`
  → **Java Pass 1,310 / 期待拒否 Pass 28 / Known 28 / Fail 0 / 未観測 59**
  （`docs/reports/2026-09-24-java-corpus-rec22.{md,json}`）。**系統別の表は
  [`2026-09-24-java-corpus-ast19.md`](2026-09-24-java-corpus-ast19.md) と 1 行も違わない**
  （報告全体でも差は生成器の SHA-256 の行だけ）。
- **IR golden 不変**: `crates` / `ir` / `corpus` は `origin/master` と byte 一致（front も Rust backend も触っていない）。
- 差分ファジング: `cargo test -p ubnfc-fuzz --offline` の固定 seed キャンペーンが緑。
- `cargo fmt --all --check` / `cargo clippy --workspace --all-targets -- -D warnings` /
  `cd ubnfc-java && mvn -o test` / `cd examples/p4-java && mvn -o test` /
  `examples/p4-java/scripts/generate.sh --check` はいずれも緑。
- `cargo test --workspace --offline --no-fail-fast` は **508 件すべて緑**（`origin/master` で赤だった
  `spec_corpus::ubnf_citations_resolve_to_real_spec_headings` は issue #11 の PR で解消済み）。
  なお `ubnfc-fuzz` の campaign テストは `target/debug/ubnfc-rust` が無いと `build:rust` で discard するので、
  新しい checkout では `cargo build --offline --bin ubnfc-rust` を先に走らせる（本 PR とは無関係の環境条件）。

## 9. 再現

```bash
cd ubnfc-java && mvn -o -q -DskipTests package
bash examples/p4-java/scripts/generate.sh --check
cd examples/p4-java && mvn -o -q -DskipTests package && mvn -o test
# 認識専用の A/B（binary 交互。base は origin/master の生成物を javac したもの）
taskset -c 28 java -Xms512m -Xmx2g -Xss16m \
  -Dbench.fixtures=complex,complex-x64 -Dbench.runs=7 -Dbench.warmup=1000 \
  -Dbench.modes=recognize-fast -cp target/classes org.ubnfc.p4.Benchmark src/test/resources/fixtures
# Rust 側（同じ core）
taskset -c 28 cargo run -q --offline --release -p ubnfc-p4-rust --example bench -- complex complex-x64
taskset -c 28 cargo run -q --offline --release -p ubnfc-p4-rust --features counters --example bench -- --counters complex
# corpus
cargo run -q --offline --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-24-java-corpus-rec22.md --json docs/reports/2026-09-24-java-corpus-rec22.json
```
