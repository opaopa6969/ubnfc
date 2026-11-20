# Rust の AST 後処理における event 走査（#22）

## 変更

`occurrences` は event から公開出現表と AST の直下 capture 表を組み立てる。左枝と単一の子には局所変数で降り、続きを処理する右枝と Capture / Rule の完了だけをスタックに積む。タグと必要な欄は packed event から直接読む。公開出現表を作るかどうかは同じ走査の const generic に渡し、AST のみの場合は公開出現用の分岐やスタック操作をコンパイル時に省く。lexical が無効なら Token の span・規則・式番号の読み出しは要らない。Rule の入口と出口、Join の左・右、Capture の完了順を保つため、直下 capture の区間・番号、公開出現の親と完了順、token extent と木の見積りは従来どおり。

JSON 100 KB 所有 AST では `occurrences` の 1 走査に入口 134,301、完了 37,900 の計 172,201 visit があり、A/B で一致した。変更はこの走査に限定した。値構築、木から所有 AST への射影、node span 集計、event の容量制限は従来どおりで、再帰フレームの追加もない。

## 帰属計測

A は計測開始時の `origin/master`（`ccee0ba`）、B は走査変更のコミット `20a3e74`。Rust stable 1.100.0（`0ef1bc386`）、release、debuginfo=2、AMD Ryzen 9 7950X、WSL2。固定入力は JSONTestSuite の `1ef36fa01286573e846ac449e8683f8833c5b26a` から `benchmarks/scripts/make-json-inputs.py` で生成した 100,039 byte（SHA-256 `d7a4d32f25212e517f1a4e48cf684326c3874fb49e566f2f541911e9798d2ee1`）。所有 AST は公開出現表・value spans を既定の off にした。

Callgrind 3.19.0、CPU 30、100 parse 暖機、10 parse 計装。A B を交互に 3 組実行し、下表の全体計数は側ごとに 3 回とも一致した。各実行直前に `uptime` を記録。01:18:57–01:20:07 JST、稼働 11 日 3:15–3:16、load average（1/5/15 分）9.00–19.62 / 8.89–12.05 / 7.49–8.69。

| 区間 | 回数 / parse | 自己 Ir A → B（Ir / 回） | 分岐ミス A → B | D1 miss A → B | LL miss A → B |
|---|---:|---:|---:|---:|---:|
| event 走査 `occurrences` | 172,201 visit（1 走査） | 11,023,465 → 6,259,473（64.02 → 36.35） | 172,887.3 → 146,902.4 | 86,179 → 84,697 | 238.3 → 236.4 |
| `build_values_into` | 20,073（根 1＋再帰 20,072） | 1,364,967 → 1,364,967（68.00） | 7.8 → 281.9 | 22,394 → 22,539 | 3.1 → 3.2 |
| `build_rule` と `map_rule_*` | 21,922 calls | 3,568,366 → 3,568,366（162.78） | 31,784.3 → 29,759.6 | 8,905 → 8,790 | 427.7 → 428.0 |
| 所有 AST 射影 | 23,243 calls | 1,507,740 → 1,507,740（64.87） | 12,608.5 → 18,525.2 | 13,540.7 → 13,540.5 | 302.9 → 302.9 |
| `push_node_spans` | 1 走査 | 427,918 → 427,918 | 8,851.4 → 8,849.6 | 17,096 → 17,095 | 1,041.2 → 1,041.2 |
| parse 全体（結果の破棄も含む） | 1 | **48,236,828 → 43,472,763（−9.88%）** | 354,747.0 → 342,406.3 | 319,815.7 → 318,231.9 | 12,742.8 → 12,741.0 |

表の区間は関数と DWARF の inline frame で分類した自己コスト。射影は `AstTree::project`、record ごとの射影と射影 closure を含み、別関数に出た allocator や文字列複製の共通処理は全体に含む。分岐ミスは Bcm+Bim。D1 は read/write、LL は instruction/read/write の和。キャッシュと分岐は模擬値。

A の行別では `Session::ev` の arena 読み（生成 `runtime.rs:1033`）が 1,429,639 Ir と D1 read miss 82,818、`PackedEvent::unpack` のタグ分岐（同 241 行）が 1,025,283 Ir と間接分岐ミス 125,058.1。B の直接タグ分岐（同 2296 行）は 1,091,707 Ir、D1 read miss 37,347、間接分岐ミス 84,389.0。行への帰属は命令の配置で分かれるため、走査全体で比較すると D1 miss は 1,482 減、Ir は 4,763,992 減（43.22%）。全体でも Ir が 4,764,065 減った。読み出しとスタック書き込みが減り、値構築・射影・span の自己 Ir は一致した。JSON 認識の補助 Callgrind は 17,212,812.5 → 17,212,730.5 Ir（−0.0005%）、D1 miss は 3,934.9 → 3,938.5。

`perf record -e cpu-clock:u -F 499 --call-graph dwarf,8192` で JSON 所有 AST を別採取（CPU 30、暖機 100、反復 4,000）。A は 6,377 samples、lost 0。A の `finish_checked` の自己 sample は全体の 21.17%。`perf annotate --stdio -l` の命令を `addr2line -afiC` の inline frame に対応させると、丸めを含めその約 99.65% は event 走査に帰属した。`build_values_into` は全体の 6.98%、`AstTree::project` 本体は 2.21%、`push_node_spans` は 1.08%。射影の下位処理は Callgrind 表で個別関数を合算した。perf の前後採取は時刻が異なり、改善率には交互計測を使った。A/B の `uptime` は 01:09:28 / 01:19:40 JST、load average 7.48 / 6.55 / 5.94 と 16.35 / 10.76 / 8.18。B は 12,078 samples（lost 0）で、独立した `occurrences::<false>` の自己 sample は全体の 27.12%。

別バイナリの counting allocator で `alloc` / `alloc_zeroed` / `realloc` の要求サイズを合計。CPU 30、100 暖機後の 1 parse を A B 交互に 3 組（01:21:19–01:21:26 JST、load average 13.01–13.81 / 11.46–11.66 / 8.72–8.80）。同じ回数・byte だった。暖機で確保した pool の容量はこの表に含めない。

| 区間 | 確保・再確保回数 | byte |
|---|---:|---:|
| event 走査（直下 capture 表・木の reserve を含む） | 8 | 130,560 |
| 値構築（根の `Vec` と再帰処理） | 1 | 16 |
| 所有 AST 射影 | 13,075 | 734,698 |
| node spans の reserve と集計 | 1 | 665,616 |
| parse 全体 | 13,097 | 1,565,922 |

## Native 交互計測

Valgrind イメージと同じ Debian 12 の glibc 2.36 上で、計装なしの release バイナリを CPU 6 に固定。24 parse 暖機。各ケースで A B の順に繰り返し、7 組の比較可能な run を採取した。両実行とも elapsed に対するプロセス CPU 時間が 97% 以上、かつ SMT sibling（CPU 7）の使用率の差が 5 ポイント以内を満たす組を採用した。条件は計時前に固定。共有負荷は高く、両者で sibling が飽和した組も含む。各実行直前に `uptime` を収録した。01:19:15–01:22:14 JST、稼働 11 日 3:15–3:18、load average（1/5/15 分）11.56–20.11 / 9.55–12.83 / 7.73–9.27。

| 入力・経路 | 反復 / run | 採用 / 試行 | A 中央値 ms/parse | B 中央値 ms/parse | 時間差 |
|---|---:|---:|---:|---:|---:|
| JSON 100 KB 所有 AST | 180 | 7/14 | 5.118432 | 4.181104 | -18.31% |
| JSON 100 KB 認識 | 600 | 7/8 | 0.861051 | 0.880408 | +2.25% |
| P4 complex 所有 AST | 5,000 | 7/31 | 0.122205 | 0.121988 | -0.18% |
| P4 complex-x64 所有 AST | 64 | 7/17 | 9.694352 | 9.900596 | +2.13% |
| P4 complex 認識 | 6,000 | 7/10 | 0.092885 | 0.090410 | -2.66% |
| P4 complex-x64 認識 | 96 | 7/27 | 6.394319 | 6.194675 | -3.12% |

<details>
<summary>採用した 7 組の実行順（ns/parse）</summary>

| 入力・経路 | A | B |
|---|---|---|
| JSON 100 KB 所有 AST | 7054603, 5564435, 3705772, 4247161, 5118432, 4588587, 5264608 | 6758624, 4181104, 3435039, 3960834, 4142807, 5245637, 4561989 |
| JSON 100 KB 認識 | 861051, 830622, 847150, 829514, 897207, 1133456, 894637 | 858218, 880408, 796571, 857752, 1044953, 952497, 897543 |
| P4 complex 所有 AST | 129148, 122205, 127084, 130616, 118705, 110310, 116421 | 122452, 124882, 121988, 117607, 122469, 113716, 109686 |
| P4 complex-x64 所有 AST | 11928750, 9610996, 8707262, 9694352, 11382864, 8549641, 12694095 | 11192944, 10001906, 9666331, 9900596, 9335795, 10044937, 9782494 |
| P4 complex 認識 | 92885, 88597, 85227, 86182, 94429, 144002, 115608 | 90410, 85216, 89794, 85464, 114179, 97570, 113329 |
| P4 complex-x64 認識 | 6672505, 6394319, 7096382, 5782331, 6798051, 5334256, 6115685 | 8891131, 6923715, 6063566, 6092122, 6297092, 5260869, 6194675 |

</details>

直接降下と packed 欄の読み分けまでの試作では、全体 Ir は 45,799,132（−5.05%）。CPU 12 の 7 組では JSON 所有 AST 3.647948 → 3.247752 ms、JSON 認識 0.730188 → 0.775671 ms（+6.23%）で、対照が許容幅を超えた。CPU 30 の補助計測は JSON 所有 AST・認識で負荷条件を満たした組が 1・4 組に留まった。最終版は公開出現 off の走査をコンパイル時に特化し、全体 Ir をさらに削減した。上表では JSON 所有 AST の全体 Ir と時間中央値の両方で 5% 目標を満たし、すべての対照が +3% 以内だったため、既定の経路へ採用する。

## 再現と契約の検査

1. A/B とも [帰属計測の Rust 手順](../../benchmarks/attribution/README.md) の `driver.rs` と `region.c` を使って独立した release バイナリを作る。`[profile.release]` の `debug` は 2。P4 IR は `ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json`、入力は `examples/p4-rust/tests/fixtures/complex{,-x64}.tiny`。JSON は上記の固定入力。AST と認識のどちらも観測は既定の off。
2. Callgrind は `--dump-instr=yes --instr-atstart=no --cache-sim=yes --branch-sim=yes --I1=32768,8,64 --D1=32768,8,64 --LL=67108864,16,64`。CPU 30 固定で `uptime` の後に `valgrind --tool=callgrind … <AまたはB> json ast <入力> 100 10` を A B の順に 3 組。全体は `benchmarks/attribution/summarize.py callgrind <出力> 10`。自己の行別計数では call edge を除き、命令アドレスを `addr2line -afiC -e <binary>` で inline frame まで展開する。確保の計測では走査、値構築、射影、node spans の直前と直後で allocator の集計先を切り替える。走査ループで入口・完了を別カウンタに加算する。
3. Native は AMD Ryzen の同一論理 CPU 上で `uptime`、`taskset -c 6 <AまたはB> <p4|json> <recognize|ast> <入力> 24 <表の反復数>` を A B の順に繰り返す。コンテナなら `--cpuset-cpus=6` で固定する。結果の破棄を含み、入力 I/O と暖機は計測区間外。負荷条件は `wait4` の user+sys と `/proc/stat` の SMT sibling の idle+iowait から確認する。
4. `scripts/fetch-external-fixtures.sh` の pin で取得した外部 fixture を使う。P4 fixture の既存検査は認識・木・所有 AST の golden、node/value span、公開出現と完了順、診断と資源上限を照合する。生成 Rust の runtime・semantics・review・driver 検査は scope、mapping 失敗、rollback を含む。P4 と自己ホストの regenerate は生成物の byte 一致も検査する。全 Rust corpus は `ubnfc-harness run-all --backend rust --jobs 6`。実行中の再ビルドを避けるため、固定コピーの front と Rust 生成器を `UBNFC_BIN` / `UBNFC_RUST_GENERATOR` に指定する。
