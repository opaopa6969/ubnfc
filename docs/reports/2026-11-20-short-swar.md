# Rust の短区間 SWAR を再評価（#56）

`swar` feature で生成する bulk scan では、同じ ASCII 区間の先頭 2 byte を表引きし、
両方を消費したときだけ 8 byte ずつの走査を開始する。word 経路が止めた位置と
残りの端数は表引きで処理し、許容された UTF-8 の code point の直後から 2 byte を
数え直す。escape の処理で外の反復へ戻った場合も同じ手順を使う。
この変更を選んだ理由は、マスクを構築する必要のない空区間と 1 byte の区間を
既存の表引きで処理できる最小の prefix 長だから。8 byte の読込には従来どおりスライスの境界検査を使う。
生成器の既定は scalar。比較対象にした 3 文法で、feature なしの変更前後の生成ファイルは
crate 名を除いてバイト単位で一致する。

## 固定入力の交互測定

- AMD Ryzen 9 7950X、論理 CPU 30 に固定、Linux 6.18.33.2 WSL2、`rustc 1.100.0 (0ef1bc386 2026-11-09)` の release。
- 4 版を 1 個の計測バイナリにリンク。scalar は作業開始時の `origin/master`、短区間は `0d44aaf` の `swar`、旧 SWAR は `3fa69b5`、旧 scalar は #48 後の `66a18ff`。
- 全版へ同じ IR を渡し、`plan.ll1` は全版で外した。JSONTestSuite は `1ef36fa01286573e846ac449e8683f8833c5b26a`。JSON 1 MB は 1,000,112 byte、SHA-256 は `ae482dce4b9b5f5ed2dad2f988e1d0f675663faf61f22c31d52dccbefaf9849c`。
- AST なし、バッファ再利用あり。入力ごとに scalar の 1 parse から約 0.5 秒分の反復数を決め、全版で同じ反復数と事前暖機を使う。各版 7 round × 2 セッション、順序を round ごとに入れ替え、全 14 値を集計した。生データには実行順と反復数、全試行後の `uptime` を残した。

## 全 14 値の中央値と最小値

単位は ms/parse。各欄は中央値 / 最小値。

| 入力 | 旧 scalar | 旧 SWAR | 現行 scalar | 短区間 SWAR |
|---|---:|---:|---:|---:|
| complex.tiny | 0.631268 / 0.341452 | 0.633888 / 0.385636 | 0.632407 / 0.422844 | 0.492362 / 0.345132 |
| complex-x4.tiny | 2.509756 / 1.656871 | 2.820711 / 1.353313 | 2.502579 / 1.614075 | 2.705721 / 1.359204 |
| complex-x16.tiny | 18.103423 / 12.279293 | 17.661750 / 9.566272 | 15.161300 / 8.936308 | 19.341566 / 9.248323 |
| complex-x64.tiny | 111.129513 / 71.330713 | 94.327346 / 67.237978 | 102.698359 / 72.686447 | 109.367197 / 73.240124 |
| flat-arithmetic.tiny | 0.563303 / 0.420517 | 0.536598 / 0.423404 | 0.571838 / 0.389163 | 0.650181 / 0.383168 |
| large-match.tiny | 3.387868 / 2.131444 | 3.841918 / 2.114230 | 3.729125 / 2.261898 | 3.512094 / 2.093999 |
| json-1m.json | 98.517129 / 44.617069 | 96.422308 / 63.291677 | 90.803658 / 65.411175 | 95.171606 / 40.636723 |
| operators-atoms-512.txt | 0.713874 / 0.193416 | 0.707153 / 0.168349 | 0.865093 / 0.331518 | 0.803559 / 0.229236 |

| セッション | JSON 1 MB の中央値（旧 scalar / 旧 SWAR / 現行 scalar / 短区間 SWAR） | 開始 load average | 終了 load average |
|---|---:|---:|---:|
| 1 | 104.545494 / 99.092955 / 97.686936 / 97.762918 | 43.31, 60.09, 50.75 | 52.71, 62.22, 55.54 |
| 2 | 76.187995 / 95.328699 / 74.754795 / 68.508049 | 52.71, 62.22, 55.54 | 54.79, 57.60, 56.54 |


固定 JSON 1 MB の中央値は旧 #48 後の 98.517129 ms から 95.171606 ms。**1.035 倍で、1.5 倍の目標は未達**。
今回の対照で目標に必要な 65.678086 ms までは 29.493520 ms 残る。
旧 SWAR 比では時間が 1.30% 減り、現行 scalar 比では 4.81% 増えた。

P4 の 6 fixture と Operators は生データの全値を併記した。現行 scalar と短区間 SWAR は
P4 と Operators の生成ソースが一致する。P4 x64 の中央値は 102.698359 ms から 109.367197 ms に
増え、共有負荷とコード配置の影響を含めて無退行を実時間で確認できなかった。
測定中はこの worktree の検証や共有マシンの他の処理も動いていた。
既定は scalar とし、残差の再現と分析は [#64](https://github.com/opaopa6969/ubnfc/issues/64) に引き継ぐ。

[セッション 1 の生データ](evidence/2026-11-20-short-swar/session-1.csv)・
[uptime](evidence/2026-11-20-short-swar/session-1.uptime)、
[セッション 2 の生データ](evidence/2026-11-20-short-swar/session-2.csv)・
[uptime](evidence/2026-11-20-short-swar/session-2.uptime)、
[生成器・生成物・入力のハッシュ](evidence/2026-11-20-short-swar/build-options.txt)。

## 区間の回数と自己命令数

回数は現行 scalar のコピーで bulk loop の式 ID、開始位置、終了位置を記録して 1 parse 分を集計した。
終了位置にある byte が `\` なら escape 終端とした。空の区間も同じ基準で分けた。
全版で記録した各開始位置から同じ bulk loop を個別に再実行し、終了位置が一致することを検査した。

自己 Ir は上記の全区間を長さと終端ごとに分け、生成した bulk loop を独立の関数へ置いた
リプレイで採った。Callgrind 3.19.0 が loop 関数そのものに帰属した命令数を合算した。
タイマーや集計を loop に差し込んでおらず、ドライバの反復や照合も自己 Ir に含まれない。
関数の切り出しで inline と配置が変わるため、リプレイの自己 Ir から本番パーサー全体への寄与を直接換算できない。
全体の Ir は計数・リプレイを追加する前の生成 parser を 10 parse 暖機し、次の 10 parse を別に収集した。
命令数と cache simulation は Callgrind の値。実機の実時間は独立した交互測定の表を参照。

#31 のタイマー付き自己壁時間には timer と TLS 集計の費用が入る。今回は計数、自己 Ir、
実時間を独立に採り、共有負荷を uptime で併記した。

| 消費 byte | escape 終端 | 回数 / parse | 旧 scalar 自己 Ir / 回 | 旧 SWAR 自己 Ir / 回 | 現行 scalar 自己 Ir / 回 | 短区間 SWAR 自己 Ir / 回 |
|---|---|---:|---:|---:|---:|---:|
| 0 | なし | 203,439 | 35.727 | 36.760 | 35.727 | 37.825 |
| 0 | あり | 58,139 | 43.000 | 44.000 | 43.000 | 46.000 |
| 1..7 | なし | 91,156 | 61.232 | 87.390 | 61.232 | 70.507 |
| 1..7 | あり | 5,285 | 90.495 | 95.000 | 90.495 | 109.242 |
| 8 以上 | なし | 6,605 | 608.800 | 210.200 | 608.800 | 215.200 |
| 8 以上 | あり | 0 | — | — | — | — |

空は 261,578 区間、1..7 byte は 96,441 区間で、合計 364,624 区間のうち **98.19% が 8 byte 未満**。
escape 終端は 63,424 区間。空の区間と短い escape 終端では新しい SWAR の自己 Ir が増えた。
短い非 escape 終端では旧 SWAR の 87.390 から 70.507 Ir/回へ約 19.3% 減った。

| 全体 / parse | 旧 scalar | 旧 SWAR | 現行 scalar | 短区間 SWAR |
|---|---:|---:|---:|---:|
| Ir | 195,369,173.0 | 201,497,701.0 | 195,369,167.0 | 201,081,515.0 |
| 模擬 I1 miss | 24,510.7 | 5,977.5 | 60,661.8 | 711.1 |
| 模擬 LL instruction miss | 80.2 | 79.4 | 79.9 | 78.2 |

短区間 SWAR は旧 SWAR 比で全体 Ir を 416,186 / parse 減らした（0.21% 減）。
現行 scalar 比では全体 Ir が 2.92% 増えた。模擬 I1 miss は binary 配置や inline にも依存するため、
この結果から実機の命令キャッシュや壁時間への寄与は確定できない。
構造規則、cursor、UTF-8 復号など、今回の bulk 走査の外側にも全体の費用が残る。
個別の残差はリンク先の後続 issue で追う。

[区間の回数と自己 Ir CSV](evidence/2026-11-20-short-swar/bulk-self-ir.csv)、
[全体・再実行の Callgrind 生データ](evidence/2026-11-20-short-swar/callgrind.tar.gz)、
[プロファイルの uptime](evidence/2026-11-20-short-swar/profile.uptime)。

## 検証

- 既定と `swar` feature の生成テストは各 6 件成功。固定 seed の 1,363 入力を両方の生成器で
  scan on/off 比較し、word 境界、UTF-8、escape、反復下限、失敗時の診断差分を確認した。
- 短区間 SWAR を指定して生成した JSON を `wasm32-unknown-unknown` の release で build した。
  生成した JSON・P4・Operators の既定 feature で有効なライブラリ依存は 0。
  3 文法とも生成コードの `#![forbid(unsafe_code)]` を確認した。
- 外部 fixture は `scripts/fetch-external-fixtures.sh` で固定 pin を取得し、
  `UNLAXER_PARSER_DIR`、`UNLAXER_PARSER_ROOT`、`TINYEXPRESSION_DIR` を設定した。
  差分テスト用の Java jar は専用 worktree で build した。

## 再現手順

この報告を追加した PR の head を展開した専用 worktree のルートで実行する。記録した生成器はそれぞれ独立した Cargo target で
ビルドし、併記した生成器と生成物の SHA-256 で取り違えを検出できる。
パスと基準を検証してから測定する。

```sh
export RUSTUP_TOOLCHAIN=stable
export UBNFC_ROOT="$PWD"
export TMPDIR="$(mktemp -d)"
scripts/fetch-external-fixtures.sh "$TMPDIR/fixtures" > "$TMPDIR/fixtures.env"
set -a
. "$TMPDIR/fixtures.env"
set +a

git clone --no-checkout https://github.com/nst/JSONTestSuite.git "$TMPDIR/suite"
git -C "$TMPDIR/suite" archive 1ef36fa01286573e846ac449e8683f8833c5b26a |
  tar -x -C "$TMPDIR/suite"
mkdir -p target
work=$(mktemp -d "$PWD/target/issue56-XXXXXX")
for rev in 568dca6 3fa69b5 66a18ff; do
  source_dir="$TMPDIR/source-$rev"
  mkdir -p "$source_dir"
  git archive "$rev" | tar -x -C "$source_dir"
  CARGO_TARGET_DIR="$work/gen-$rev" cargo build --offline --release -j 4 \
    --manifest-path "$source_dir/Cargo.toml" -p ubnfc-rust --bin ubnfc-rust
done
mkdir -p "$TMPDIR/source-0d44aaf"
git archive 0d44aaf | tar -x -C "$TMPDIR/source-0d44aaf"
CARGO_TARGET_DIR="$work/gen-short" cargo build --offline --release -j 5 \
  --manifest-path "$TMPDIR/source-0d44aaf/Cargo.toml" \
  --bin ubnfc --bin ubnfc-rust --features ubnfc-rust/swar
evidence=docs/reports/evidence/2026-11-20-short-swar
python3 "$evidence/ab.py" "$work" prepare "$TMPDIR/suite" \
  "$work/gen-568dca6/release/ubnfc-rust" "$work/gen-short/release/ubnfc-rust" \
  "$work/gen-3fa69b5/release/ubnfc-rust" "$work/gen-66a18ff/release/ubnfc-rust" \
  > "$TMPDIR/build-options.txt"
for session in 1 2; do
  python3 "$evidence/ab.py" "$work" time "$session" 30 2> "$work/session-$session.uptime"
done

# 自分専用の image のタグを使う。
image="ubnfc-swar56-$(date +%s)-$$"
docker build -t "$image" benchmarks/attribution
python3 "$evidence/profile.py" prepare "$work" "$image" 30
python3 "$evidence/profile.py" measure "$work" "$image" 30
```
