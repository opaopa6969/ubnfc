# Rust SWAR の再測定と scalar 既定への復帰（#31）

2026-11-01。**固定 JSON 1 MB の 1.5 倍目標は未達**。
SWAR の全体命令数は +3.11%、交互 A/B の JSON 1 MB は両セッションで悪化した。
過去の +15.30% 全体を SWAR 由来とは断定できないが、安定した利益を確認できないため、
**生成器の既定を scalar の bulk scan へ戻す**。SWAR は `ubnfc-rust` の `swar` feature に残す。
未達目標と SWAR の再評価条件は [#56](https://github.com/opaopa6969/ubnfc/issues/56)。

## 判断と変更範囲

`plan.scans` の範囲、bulk scan 自体の既定、`options.scan`、UTF-8 の参照復号、
診断・lexical・occurrences・invert の脱出口は維持する。
変えるのは生成器が 8 byte の word 判定を出力する既定だけ。
生成ライブラリの API・依存・`#![forbid(unsafe_code)]` と非 x86 の scalar 実装を保つ。
`cargo build --bin ubnfc-rust --features ubnfc-rust/swar` で従来の portable SWAR を生成できる。
feature を外して再生成すれば scalar に戻り、PR 全体の revert でも戻せる。

小さな閾値変更から 1.5 倍を期待するより、利益が未確認の変更を既定から外す方が安全で可逆。
cp_at の単価削減（#20）や構造領域の別方式（#2）はこの PR に混ぜない。

## 旧版と SWAR の交互再測定

before=`66a18ff8e71839b893d2edabdad14825f8436dae`、
after=`3fa69b5c8e4e438c76a4679c634ef382ef2b1cb6`。両 commit は別の detached worktree。
同一 IR・入力・binary・認識オプション、stable `rustc 1.99.1 (58cf8f9c3 2026-10-13)`、
release、Ryzen 9 7950X / Linux 6.18.33.2-microsoft-standard-WSL2、`taskset -c 30`。
`plan.ll1` を両側から外し、固定 P4 IR と JSONTestSuite
`1ef36fa01286573e846ac449e8683f8833c5b26a` を使う。入力の SHA-256 は既存証拠と一致。
JSON 1 MB は 1,000,112 byte、SHA-256 は
`ae482dce4b9b5f5ed2dad2f988e1d0f675663faf61f22c31d52dccbefaf9849c`。

各入力で before から反復数を決め、同じ反復数で先後を交替する。
0.5 秒目安 / run、7 round × 2 セッション、半数を暖機。外れ値を除外しない。
各 run の load average を CSV に、セッションの uptime をテキストに残した。
最終測定の道具では各 run の `uptime` コマンド出力も保存する。
共有マシン上の他作業を止めたり負荷条件を操作したりしていない。

| セッション | JSON 1 MB before / SWAR ms | 時間変化 | P4 x64 before / SWAR ms | 時間変化 | Operators before / SWAR ms | 時間変化 |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 13.987063 / 14.218482 | +1.65% | 8.610229 / 9.658607 | +12.18% | 0.112997 / 0.114413 | +1.25% |
| 2 | 17.675671 / 18.768278 | +6.18% | 9.311412 / 8.854180 | −4.91% | 0.119999 / 0.117858 | −1.78% |

load average（1 / 5 / 15 分、uptime）:

| セッション | 開始 | 終了 |
|---|---|---|
| 1 | 22.18 / 22.60 / 20.87 | 24.86 / 23.30 / 21.25 |
| 2 | 24.86 / 23.30 / 21.25 | 23.12 / 23.13 / 21.35 |

全 14 値の中央値と最小値（ms）:

| 入力 | before 中央値 | SWAR 中央値 | before 最小 | SWAR 最小 | 時間変化 |
|---|---:|---:|---:|---:|---:|
| complex.tiny | 0.090598 | 0.090568 | 0.076278 | 0.076479 | -0.03% |
| complex-x4.tiny | 0.364814 | 0.377909 | 0.299073 | 0.307128 | +3.59% |
| complex-x16.tiny | 1.622051 | 1.531372 | 1.329092 | 1.302840 | -5.59% |
| complex-x64.tiny | 8.678073 | 9.130074 | 6.619420 | 7.103540 | +5.21% |
| flat-arithmetic.tiny | 0.105220 | 0.103389 | 0.087854 | 0.088358 | -1.74% |
| large-match.tiny | 0.502609 | 0.502312 | 0.405994 | 0.405915 | -0.06% |
| json-1k.json | 0.015385 | 0.016445 | 0.013939 | 0.013986 | +6.89% |
| json-10k.json | 0.186389 | 0.185787 | 0.173036 | 0.172159 | -0.32% |
| json-100k.json | 1.871897 | 1.881384 | 1.476377 | 1.731220 | +0.51% |
| json-1m.json | 16.767257 | 17.007727 | 13.547344 | 13.001857 | +1.43% |
| operators-atoms-512.txt | 0.114445 | 0.117404 | 0.108568 | 0.104305 | +2.59% |

[生データ 1](evidence/2026-11-01-swar-default/original-1.csv)・
[2](evidence/2026-11-01-swar-default/original-2.csv)、
[uptime 1](evidence/2026-11-01-swar-default/original-1.txt)・
[2](evidence/2026-11-01-swar-default/original-2.txt)。

JSON は両セッションで悪化し、命令数も増えたので固定費増を無視しない。
ただし両者の **P4 / Operators は Cargo.toml の crate 名以外の生成ファイルが全て一致**するのに、
P4 x64 の差は +12.18% / −4.91% と逆転した。共有負荷・配置・実行順等の影響がある。
この測定は「過去の 15.30% 全部が真の退行」とも「同一ソースだから無退行を実測した」とも扱わない。

## 実行回数と短区間の費用

`profile-scans.py` は生成物の**専用の複製だけ**に計装を入れる。
全走査関数と bulk の各連続区間（ID 10000 以上）を別々に数え、
関数全体の長さ（escape をまたぐ）を連続区間長へ読み替えない。
10 parse 暖機後 10 parse を計数。子の計装時間を引いた自己壁時間を測る。

以下は 1 parse の bulk 区間の回数と、区間あたりの計装自己壁時間。
escape は走査の終端に `\` があることを表す。空の区間でも escape 終端なら数える。

| 区間長 byte | escape 終端 | 回数（両版一致） | scalar ns/回 | SWAR ns/回 |
|---|---|---:|---:|---:|
| 0 | なし | 203,439 | 19.72 | 21.22 |
| 0 | あり | 58,139 | 19.74 | 21.57 |
| 1..7 | なし | 91,156 | 21.52 | 24.79 |
| 1..7 | あり | 5,285 | 24.36 | 29.40 |
| 8 以上 | なし | 6,605 | 70.40 | 76.21 |

空 261,578 区間、短 96,441 区間、長 6,605 区間で、**98.19% が 8 byte 未満**。
escape 終端は **63,424 回**。長い ASCII の補助入力に差し替えていない。
これは実際の bulk の計数で、入力文字列の静的分割だけからの推測ではない。

**この時間には timer・TLS 集計・inline 判断の変化・preemption が含まれる。**
非計装の自己 CPU 時間や全体実時間の寄与率としては使えず、上表から全体の速度差は算出しない。
短い区間ほど計装の比重が大きく、長区間も単位が区間あたりなので byte 単価とは異なる。
空/短区間での固定費という原因候補は、回数と次の非計装 callgrind の命令増を合わせた判断である。
タイマーを関数にだけ入れた予備試行も `profile-*-preliminary-counts.csv` に残す。
それには区間計数がなく、他の計測との CPU 競合で壁時間も偏るため、速度判断には採らない。

[scalar 区間・関数生計数](evidence/2026-11-01-swar-default/profile-before-counts.csv)・
[SWAR 生計数](evidence/2026-11-01-swar-default/profile-after-counts.csv)、
[式との対応](evidence/2026-11-01-swar-default/profile-after-labels.json)。
負荷は scalar 開始 28.14/26.19/22.97 → 終了 26.93/25.97/22.91、
SWAR 開始 25.73/25.74/22.85 → 終了 25.03/25.59/22.82。

## 非計装の命令数

同じ JSON 1 MB、同じ binary に両 crate を入れた callgrind 3.19.0。
100 parse 暖機後 10 parse だけを公式 client request で計装し、cache / branch simulation を有効にした。
parser のソースに timer や計数は追加していない。値は 1 parse あたり。
実機の命令数・branch predictor・cache の直接測定とは異なる。

| 指標 | scalar `66a18ff` | SWAR `3fa69b5` |
|---|---:|---:|
| 全体 Ir | 195,415,378.6 | 201,489,757.6 |
| 全体 Dr | 51,614,493.4 | 53,099,359.4 |
| 全体 Dw | 36,844,915.6 | 38,917,634.6 |
| D1 read / write miss | 31,427.0 / 1,512.9 | 31,423.9 / 1,511.0 |
| LL read / write miss | 1,569.5 / 147.4 | 1,569.3 / 147.3 |
| 条件分岐 / miss | 30,239,100.1 / 714,775.1 | 29,649,958.1 / 618,983.6 |
| 間接分岐 / miss | 2,306,909.1 / 2,655.7 | 2,306,909.1 / 34,361.3 |

全体 Ir は **+3.11%**。分岐回数が減っても命令数が増える。
`Ws` の bulk を呼ぶ `s5` は SWAR 版で非 inline 化され、236,469 calls / parse、
自己 11,589,707 Ir が付く。scalar 版は inline され別 symbol がないため、
その自己 Ir だけを引き算して改善率としない。
UTF-8 / escape / 構造規則に残る仕事と inline / 配置の影響があり、word 判定だけで全体 1.5 倍は確認できない。

[scalar 全関数](evidence/2026-11-01-swar-default/callgrind-before.tsv)・
[SWAR 全関数](evidence/2026-11-01-swar-default/callgrind-after.tsv)、
[scalar 元データ](evidence/2026-11-01-swar-default/before.callgrind.gz)・
[SWAR 元データ](evidence/2026-11-01-swar-default/after.callgrind.gz)。
各負荷と simulator の構成は `callgrind-*.load.txt` / `callgrind-*.txt` に残す。

## scalar 既定復帰版と両基準の比較

現行の生成器（親 `12e5e80` と evidence の `final-generator.diff`）で scalar を生成し、
旧 scalar `66a18ff` / 旧 SWAR `3fa69b5` の双方と同じ IR・入力・compiler・CPU で交互測定した。
現行生成器には既に統合済みの LL(1) の API / 未使用 runtime も含まれるが、共通 IR の `plan.ll1` は空。
歴史的 commit から現行までの差が全て SWAR だけだとは解釈しない。
各 run の完全な uptime は `final-*.txt` にある。
測定中には共有 CI に加えて本作業の workspace 検査も動いており、静穏条件とは扱わない。

旧 scalar 対照は 7 round × 2 セッション。
**旧 scalar 比では JSON 1 MB +5.94%、P4 x64 +12.41% が残り、1.5 倍と無退行の実測条件は未達**。
既定復帰は短区間の命令増を除く保守的判断であり、wall time の全退行を修正したとの主張ではない。
残差と再測定は #56 の受け入れ条件に含める。

### 旧 scalar 対照（14 値）

単位 ms。before は対照、after は今回の scalar 既定版。

| 入力 | before 中央値 | after 中央値 | before 最小 | after 最小 | 時間変化 |
|---|---:|---:|---:|---:|---:|
| complex.tiny | 0.090621 | 0.083845 | 0.065357 | 0.064959 | -7.48% |
| complex-x4.tiny | 0.392387 | 0.397178 | 0.345049 | 0.335016 | +1.22% |
| complex-x16.tiny | 1.539715 | 1.613627 | 1.416931 | 1.220098 | +4.80% |
| complex-x64.tiny | 10.587228 | 11.901149 | 7.115148 | 6.309451 | +12.41% |
| flat-arithmetic.tiny | 0.108725 | 0.106993 | 0.090687 | 0.088440 | -1.59% |
| large-match.tiny | 0.580532 | 0.562574 | 0.495266 | 0.477342 | -3.09% |
| json-1k.json | 0.017102 | 0.016761 | 0.015332 | 0.015246 | -2.00% |
| json-10k.json | 0.180020 | 0.180063 | 0.162864 | 0.153601 | +0.02% |
| json-100k.json | 1.646860 | 1.685584 | 1.460451 | 1.426217 | +2.35% |
| json-1m.json | 15.678408 | 16.609569 | 14.233061 | 14.493107 | +5.94% |
| operators-atoms-512.txt | 0.112323 | 0.112790 | 0.103452 | 0.104318 | +0.41% |

### 無効と判定した試行も保存する

旧 SWAR へ基準を切り替えた最初の 3 セッションは、metadata の commit 表示と
実際の基準 binary が一致していなかった。別 worktree の同名・同 version の package を
同じ target へ build した際に出力が再利用され、基準 binary の mtime が変わらず、
生成物にも SWAR の word 読み込みがなかった。したがって SWAR 対照としては**無効**。
Cargo の内部の freshness 判断の詳細までは切り分けていない。

この試行は `invalid-baseline-swar-{1,2,3}.csv` / `.txt` に全値と uptime を保存し、
速度判断・最終集計には使わない。終了後の callgrind も同じ取り違えを含むため
`invalid-baseline-callgrind-*` として保存し、有効な A/B と扱わない。
セッション 1 には、計測後に実行中の script の編集による `csv: unbound variable` も出た。
エラー後に追加したセッション 3 も基準が誤っていたので同じく無効とした。

修正後は **commit ごとに target を分離**し、BASE / BASE_WORKTREE の commit 一致と
基準 worktree の未変更を検査する。生成器 binary と parser の hash も採る。
再測定前に基準 JSON の `parser.rs` が最初の有効な `3fa69b5` 生成物と `cmp` で一致し、
今回の scalar 版には word 読み込みがないことを確認した。
最初の `66a18ff` 対 `3fa69b5` と最終の旧 scalar 対照は、異なる target から始めており有効。

### 正しい旧 SWAR 対照（14 値）

基準は commit 別の新しい target で再ビルドし、word 読み込み 5 箇所と生成ソースの byte 一致を確認した。
7 round × 2 セッション、全 14 値。単位 ms。before は旧 SWAR、after は今回の scalar 既定版。

| 入力 | before 中央値 | after 中央値 | before 最小 | after 最小 | 時間変化 |
|---|---:|---:|---:|---:|---:|
| complex.tiny | 0.098599 | 0.091815 | 0.071319 | 0.070144 | -6.88% |
| complex-x4.tiny | 0.390261 | 0.390074 | 0.346914 | 0.333014 | -0.05% |
| complex-x16.tiny | 1.861457 | 1.503655 | 1.213046 | 1.203109 | -19.22% |
| complex-x64.tiny | 10.885495 | 10.221191 | 6.969078 | 6.498215 | -6.10% |
| flat-arithmetic.tiny | 0.109966 | 0.108895 | 0.088812 | 0.086990 | -0.97% |
| large-match.tiny | 0.536309 | 0.540941 | 0.373643 | 0.398146 | +0.86% |
| json-1k.json | 0.017219 | 0.015656 | 0.012708 | 0.013245 | -9.08% |
| json-10k.json | 0.158386 | 0.149077 | 0.139321 | 0.135543 | -5.88% |
| json-100k.json | 1.492295 | 1.530454 | 1.295542 | 1.313566 | +2.56% |
| json-1m.json | 15.287894 | 15.897490 | 13.308652 | 13.317132 | +3.99% |
| operators-atoms-512.txt | 0.116613 | 0.114045 | 0.100996 | 0.103403 | -2.20% |

### 最終比較の uptime と生データ

| 対照・セッション | 開始 1/5/15 分 | 終了 1/5/15 分 | 生データ / 全 uptime |
|---|---|---|---|
| old-1 | 29.81, 27.28, 23.97 | 25.47, 26.93, 24.10 | [CSV](evidence/2026-11-01-swar-default/final-old-1.csv) / [uptime](evidence/2026-11-01-swar-default/final-old-1.txt) |
| old-2 | 25.47, 26.93, 24.10 | 25.27, 26.67, 24.22 | [CSV](evidence/2026-11-01-swar-default/final-old-2.csv) / [uptime](evidence/2026-11-01-swar-default/final-old-2.txt) |
| verified-swar-1 | 26.58, 29.75, 27.15 | 30.14, 30.59, 27.68 | [CSV](evidence/2026-11-01-swar-default/final-verified-swar-1.csv) / [uptime](evidence/2026-11-01-swar-default/final-verified-swar-1.txt) |
| verified-swar-2 | 30.14, 30.59, 27.68 | 19.64, 27.48, 26.84 | [CSV](evidence/2026-11-01-swar-default/final-verified-swar-2.csv) / [uptime](evidence/2026-11-01-swar-default/final-verified-swar-2.txt) |

## 再現手順

本報告の PR の checkout を使う（後続 #20 等の実装後の master で代用しない）。
`66a18ff` / `3fa69b5` の detached worktree を別々に用意し、下記のパスを指定する。
外部 fixture は `scripts/fetch-external-fixtures.sh <mktemp の取得先>` で固定 pin を取得し、
`UNLAXER_PARSER_DIR` / `UNLAXER_PARSER_ROOT` / `TINYEXPRESSION_DIR` をその出力へ設定する。
JSONTestSuite は本文の固定 commit とする。

```sh
export RUSTUP_TOOLCHAIN=stable CARGO_BUILD_JOBS=8
export JSON_TEST_SUITE=/path/to/pinned/JSONTestSuite
base66=/path/to/detached-66a18ff
swar=/path/to/detached-3fa69b5
current=$PWD
mkdir -p target

# 旧版から SWAR への変更だけを測る。生成器・生成物・binary の target を分ける。
export WORK=$(mktemp -d "$PWD/target/original-swar-XXXXXX")
BASE=66a18ff BASE_WORKTREE="$base66" AFTER_WORKTREE="$swar" \
  benchmarks/scripts/run-swar-ab.sh prepare
CORE=30 RUNS=7 benchmarks/scripts/run-swar-ab.sh time "$WORK/session1" 2> "$WORK/session1.txt"
CORE=30 RUNS=7 benchmarks/scripts/run-swar-ab.sh time "$WORK/session2" 2> "$WORK/session2.txt"
PROFILE_SIDE=before benchmarks/scripts/profile-scans.sh
PROFILE_SIDE=after benchmarks/scripts/profile-scans.sh
# 自分専用の image。既存 container には接続しない。
export IMAGE=ubnfc-swar-$(date +%s)-$$
docker build -t "$IMAGE" benchmarks/attribution
benchmarks/scripts/profile-scan-cost.sh

# 復帰版を両基準と比べる。各基準は独立の WORK に置いてもよい。
for base in 66a18ff 3fa69b5; do
  baseline=$base66
  [ "$base" = 66a18ff ] || baseline=$swar
  export WORK=$(mktemp -d "$PWD/target/scalar-default-XXXXXX")
  BASE="$base" BASE_WORKTREE="$baseline" AFTER_WORKTREE="$current" \
    benchmarks/scripts/run-swar-ab.sh prepare
  CORE=30 RUNS=7 benchmarks/scripts/run-swar-ab.sh time "$WORK/session1" 2> "$WORK/session1.txt"
  CORE=30 RUNS=7 benchmarks/scripts/run-swar-ab.sh time "$WORK/session2" 2> "$WORK/session2.txt"
done
```

計測中のスクリプト・生成物は編集せず、変更するときは計測を終了してから別の出力へ採り直す。
CSV の全値を入力 / variant ごとに集めて中央値と最小を計算する。
7 round × 2 セッションなら 14 値の中央 2 値の平均が中央値。実行順序、反復数、uptime を保持する。
この手順の未達目標・自己 CPU の追加検証・SWAR 既定復帰は #56 で判断する。

## 復帰版の命令数の確認

正しい旧 SWAR 生成器を commit 別 target で再 build した後、同じ共通 IR / JSON 1 MB /
100 parse 暖機・10 parse 収集の callgrind を採り直した。値は 1 parse あたり。

| 指標 | 旧 SWAR | 今回の scalar 既定版 |
|---|---:|---:|
| Ir/parse | 201,509,573.6 | 195,435,194.6 |
| Dr/parse | 53,104,313.4 | 51,619,447.4 |
| Dw/parse | 38,917,634.6 | 36,844,915.6 |
| I1mr/parse | 4,672.0 | 48,752.8 |
| ILmr/parse | 78.8 | 79.7 |
| D1mr/parse | 31,422.8 | 31,423.9 |
| D1mw/parse | 1,513.9 | 1,510.0 |
| DLmr/parse | 1,569.3 | 1,569.4 |
| DLmw/parse | 147.3 | 147.4 |
| Bc/parse | 29,657,389.1 | 30,246,531.1 |
| Bcm/parse | 620,740.8 | 715,212.5 |
| Bi/parse | 2,306,909.1 | 2,306,909.1 |
| Bim/parse | 34,366.7 | 2,657.4 |

全体 Ir は **-3.01%**。SWAR の追加した仕事を既定から外せたことは命令数で確認した。
ただし native の最終 JSON 1 MB は旧 SWAR 比 **+3.99%** であり、
命令数の改善を実時間の改善へ読み替えない。simulator の I1 miss は増えており、
binary 配置・instruction cache も追加の原因候補だが、実機の寄与は未測定。
共有負荷とこの配置の影響を分離したとは主張しない。
旧 scalar 比の native 無退行も未確認のまま #56 へ引き継ぐ。

[旧 SWAR 全関数](evidence/2026-11-01-swar-default/final-callgrind-before.tsv)・[復帰版全関数](evidence/2026-11-01-swar-default/final-callgrind-after.tsv)、
[旧 SWAR 元データ](evidence/2026-11-01-swar-default/final-before.callgrind.gz)・[復帰版元データ](evidence/2026-11-01-swar-default/final-after.callgrind.gz)。

## 検証結果

- 固定外部 fixture と Java jar を用意して `RUSTUP_TOOLCHAIN=stable cargo test --workspace -j 8 --no-fail-fast -- --test-threads=2` を実施した。
  初回は環境に起因する **4 target が失敗**。`CARGO_TARGET_DIR` を明示したため、子 Cargo の生成物が
  一時 crate の `target` ではなく親 target へ出て、`classic_parameterized` / `driver` / `generated` が実行ファイルを見つけられなかった。
  継承を外して同じ test binary を再実行し、それぞれ **2 / 6 / 3 tests 成功**。
  もう 1 件の harness `cli` は、環境変数を外す検査で既定の sibling fixture を見つけられなかった。
  同じ binary と固定 fixture を、自分専用 image の `--rm` container 内に read-only mount して CI と同じ sibling 配置を再現し、**2 tests 成功**。
  共有の `.wt` に symlink を作って他の作業へ影響させていない。他の workspace target は初回で成功した。
  **初回の workspace コマンド自体が成功したとは記載しない**。全体の CI も緑を merge 条件とする。
- 既定 scalar / `swar` feature の生成検査が各 6 tests 成功。word mask の 3,145,728 比較も成功。
- SWAR 指定の生成器を明示した固定 seed・word / UTF-8 / escape 境界の scan on/off 差分が成功。
  workspace の同じ検査では既定 scalar も成功した。native CI に opt-in 生成と境界差分を追加した。
- SWAR 指定で生成した JSON の `wasm32-unknown-unknown` release build が成功。非 x86 実機の速度は未測定。
- BASE と BASE_WORKTREE が違う commit の場合に prepare が build 前に exit 2 で拒否することを確認した。
  各計測 script の `bash -n`、Python の構文、報告書のリンク、`git diff --check` が成功。

[workspace 初回ログ](evidence/2026-11-01-swar-default/tests31.txt.gz)、
[CLI 再検証](evidence/2026-11-01-swar-default/cli31-container.txt.gz)、
[Classic 再検証](evidence/2026-11-01-swar-default/retest31-classic_parameterized-45ec10e99ea626ae.txt.gz)、
[driver 再検証](evidence/2026-11-01-swar-default/retest31-driver-31f6e5b26c6ac783.txt.gz)、
[生成 crate 再検証](evidence/2026-11-01-swar-default/retest31-generated-c27c1dc9c72d3a7c.txt.gz)。
再実行時は親の `CARGO_TARGET_DIR` を unset し、親 workspace の既定 target は専用 worktree 内に置く。
子の fixture crate は各検査が作った専用 mktemp ディレクトリを使う。
