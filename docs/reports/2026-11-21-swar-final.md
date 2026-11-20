# Rust SWAR の固定 JSON 1 MB の残差と P4・Operators の再検証（#64）

[PR #65](https://github.com/opaopa6969/ubnfc/pull/65) と [D-104](../design/03-design-v1.md) では、
ASCII 2 byte を消費してから 8 byte ずつ走査する方式を opt-in にし、scalar を既定とした。
前回の固定 JSON 1 MB は旧 #48 後の 1.035 倍だった。
今回は、短い区間で毎 byte 行うシフトと区間開始位置との差分計算を減らすため、
クラスごとの 256 要素の直接表引きと、最初の 2 byte の個別判定を組み合わせた。
非 ASCII は既存の `cp_at` に渡す。word マスクと ASCII 2 byte の閾値は引き継いだ。

## 実時間

- AMD Ryzen 9 7950X の論理 CPU 30（SMT の相方は 31）、Linux 6.18.33.2 WSL2、
  Rust stable 1.100.0、release。AST なし、バッファ再利用あり。
- #65 と同じ固定 JSON 1,000,112 byte（SHA-256 `ae482dce4b9b5f5ed2dad2f988e1d0f675663faf61f22c31d52dccbefaf9849c`）、
  P4 6 fixture、Operators の `+` で連結した 512 atom を使用した。
  各入力で同じ IR を全版に渡し、`plan.ll1` を外した。
- 旧 #48 後（`66a18ff`）、現行 scalar と SWAR（`bc49efc`）、試作（`41526b8`）、
  現行 SWAR の同一生成物を別位置にリンクした対照を、1 バイナリに入れた。
  対照と現行 SWAR の生成物は crate 名を除き一致し、JSON の入口アドレスは異なる。
- 各入力・セッションで scalar の 1 parse を使って約 0.14 秒分の反復数を決め、全版に同じ反復数と暖機を適用した。
  各セッションは順序を回転・反転して版ごとに 10 周。各版の合計は 20 試行。
  各試行の直後に `uptime` を採った。計測は共有機で行い、ほかの処理も動いていた。

各欄は中央値［最小–最大］、単位は ms / parse。除外した試行はない。

| 入力 | 旧 #48 後 | 現行 scalar | 現行 SWAR | 試作 | 配置対照 |
|---|---:|---:|---:|---:|---:|
| `complex.tiny` | 0.0621［0.0602–0.0723］ | 0.0573［0.0564–0.0660］ | 0.0583［0.0569–0.0697］ | 0.0580［0.0565–0.0922］ | 0.0577［0.0563–0.0669］ |
| `complex-x4.tiny` | 0.2594［0.2403–0.5804］ | 0.2509［0.2262–0.3539］ | 0.2624［0.2295–0.4821］ | 0.2459［0.2269–0.4781］ | 0.2481［0.2242–0.3763］ |
| `complex-x16.tiny` | 1.0867［1.0090–1.3302］ | 0.9893［0.9553–1.4868］ | 1.0325［0.9493–1.3714］ | 1.0515［0.9618–1.6719］ | 1.0070［0.9510–1.2587］ |
| `complex-x64.tiny` | 4.5990［4.1639–5.5895］ | 4.4206［4.0726–8.6566］ | 4.4448［4.0056–7.0287］ | 4.5740［4.0456–7.0021］ | 4.5610［3.9174–6.4009］ |
| `flat-arithmetic.tiny` | 0.0776［0.0752–0.0889］ | 0.0746［0.0716–0.0846］ | 0.0735［0.0706–0.0931］ | 0.0740［0.0712–0.0882］ | 0.0747［0.0713–0.0937］ |
| `large-match.tiny` | 0.3473［0.3314–0.4127］ | 0.3235［0.3140–0.3408］ | 0.3275［0.3151–0.4175］ | 0.3221［0.3128–0.4069］ | 0.3234［0.3134–0.4575］ |
| `json-1m.json` | 12.8138［12.3959–20.5508］ | 9.9015［9.6633–14.5786］ | 9.4723［9.0653–14.6409］ | 10.0804［9.4201–14.6718］ | 9.4528［9.2485–14.7673］ |
| `operators-atoms-512.txt` | 0.0990［0.0960–0.1347］ | 0.1003［0.0937–0.1215］ | 0.0986［0.0937–0.1205］ | 0.0998［0.0938–0.1205］ | 0.0989［0.0940–0.1193］ |

| セッション | JSON の中央値（旧 #48 後 / 現行 scalar / 現行 SWAR / 試作 / 配置対照） | 開始時 load average（1 / 5 / 15 分） | 終了時 |
|---|---:|---:|---:|
| 1 | 17.1245 / 13.1950 / 12.6448 / 12.8963 / 11.9176 | 17.32 / 11.28 / 7.92 | 16.75 / 11.90 / 8.32 |
| 2 | 12.6231 / 9.8188 / 9.2525 / 9.5421 / 9.3430 | 12.77 / 11.89 / 8.70 | 9.23 / 11.04 / 8.59 |

固定 JSON 1 MB の全試行の中央値は、現行 SWAR 9.4723 ms、試作 10.0804 ms。
同じ周の速度比（現行 SWAR / 試作）の中央値は 0.967 倍で、試作が速かったのは 2 / 20 周。
各セッションでも試作の中央値は現行 SWAR より遅い。配置対照の同じ周の比は中央値 1.004 倍
（範囲 0.880–1.259 倍）で、同じソースでも測定順・共有負荷・命令配置で時刻差が出る。
旧 #48 後の 12.8138 ms 比では試作は **1.271 倍**、現行 SWAR は **1.353 倍**。
1.5 倍に必要な 8.5425 ms まで、試作は 1.5379 ms、現行 SWAR は 0.9298 ms 残った。
同じ周で取った旧 #48 後 / 試作の速度比の中央値は 1.317 倍で、こちらも目標は未達。

P4 6 fixture と Operators は、現行 scalar・現行 SWAR・試作・配置対照の生成物が
crate 名を除きバイト一致した。試作と現行 SWAR の同じ周の速度比は P4 で中央値 0.995–1.006 倍、
Operators で 1.001 倍。SWAR 変更による生成コード上の退行はない。
旧 #48 後比の無退行は実時間で一律には確認できなかった。
たとえば P4 x64 の同じ周の中央値は試作が 0.966 倍、配置対照と現行 SWAR の同じ周の速度比は 1.035 倍だった。
Operators は 0.099040 → 0.099810 ms で中央値同士では 0.8% 増だが、
同じ周の旧 #48 後 / 試作の速度比の中央値は 1.019 倍で、集計の違いで向きが変わった。

## Callgrind の自己命令数

Callgrind 3.19.0 で固定 JSON を 10 parse 暖機し、続く 10 parse の Ir を集計した。
コンパイラが付与したソース位置（inline を含む）で自己命令を分類し、呼び出し辺の費用は加算していない。
構造規則は生成した規則・式・ガード、cursor は `rt/cursor.rs` の位置に帰属した命令。
UTF-8 復号は `cp_at_non_ascii`（旧版は ASCII の検証も行う `cp_at`）と
標準ライブラリの `str` 検証・変換に帰属した命令。入力索引、残る終端走査、位置の帰属が
曖昧な inline 命令、その他のランタイム処理は別に集計した。単位は百万命令 / parse。

| 帰属先 | 旧 #48 後 | 現行 scalar | 現行 SWAR | 試作 | 配置対照 |
|---|---:|---:|---:|---:|---:|
| 一括走査ループ | 9.929 | 9.590 | 8.043 | 4.707 | 8.043 |
| その他の終端走査 | 9.121 | 6.000 | 10.350 | 10.337 | 10.350 |
| 構造規則・式・ガード | 27.231 | 26.901 | 27.047 | 26.946 | 27.047 |
| cursor | 2.744 | 2.625 | 2.968 | 2.968 | 2.968 |
| UTF-8 復号・検証 | 47.041 | 3.405 | 3.405 | 3.405 | 3.405 |
| その他の入力処理 | 9.332 | 8.742 | 8.735 | 8.735 | 8.735 |
| その他 | 89.972 | 82.569 | 82.850 | 83.215 | 82.850 |
| 全体 | 195.369 | 139.831 | 143.398 | 140.312 | 143.398 |

一括走査の自己 Ir は現行 SWAR 比で 41.5% 減り、全体 Ir は 2.15% 減った。現行 scalar の全体 Ir に対しては 0.34% 多い。
一括走査ループが現行 SWAR の全体 Ir に占めるのは 5.6%。構造規則・cursor・復号だけでも
合計 33.42 百万命令あり、今回の変更の範囲外に費用が残る。旧版からの大きな減少は
復号・検証に帰属しており、旧版比の速度向上を今回の SWAR 変更の効果として数えない。

現行 scalar で記録した 364,624 個の走査区間を、両方の SWAR ループで個別に再実行し、
終了位置がすべて一致することを確認した。98.19% は 8 byte 未満だった。以下はループを
独立関数にしたリプレイの自己 Ir / 回。inline と配置が変わるので、上のソース位置別 Ir とは別の値になる。

| 消費 byte | escape 終端 | 回数 / parse | 現行 SWAR | 試作 |
|---|---|---:|---:|---:|
| 0 | なし | 203,439 | 37.825 | 33.065 |
| 0 | あり | 58,139 | 46.000 | 34.000 |
| 1–7 | なし | 91,156 | 70.507 | 55.521 |
| 1–7 | あり | 5,285 | 109.242 | 78.244 |
| 8 以上 | なし | 6,605 | 215.200 | 186.400 |
| 8 以上 | あり | 0 | — | — |

配置対照の全体 Ir は現行 SWAR と 2 命令 / parse しか違わず、同じ仕事量でも実時間は変動した。
Callgrind の I1 miss は模擬値であり、実機のキャッシュミスや CPU 時間への寄与は未測定。

## 採否と目標の見直し

試作は採用しない。狙った命令数の削減は得られたが、固定入力の実時間は両セッションで悪化した。
生成器の既定は D-104 の scalar、opt-in は現行 SWAR とする。今回の試作前後で
feature なしの JSON・P4・Operators の生成物も crate 名を除きバイト一致した。

固定 JSON 1 MB では一括走査が実行する仕事が短い区間に偏り、全体 Ir の 5.6% にとどまる。
現行 SWAR から 1.5 倍の目標に届くには約 9.8% の実時間短縮が要るため、この走査だけに
残差の解消を求める目標は現実的でない。SWAR の目標は、同じ生成物の配置対照の変動を
超える実時間短縮が複数セッションで再現し、P4 と Operators が退行しないことへ見直す。
1.5 倍を全体の指標として扱う場合は、#20 の復号や #2 の構造領域なども含む全体の費用で判断する。

## 検証と計測データ

- 固定 pin の外部 fixture を取得し、`UNLAXER_PARSER_DIR` / `UNLAXER_PARSER_ROOT` /
  `TINYEXPRESSION_DIR` を設定した。Java は `mvn -q -DskipTests package` と
  `mvn -q test`、CI と同じ SWAR・LL(1) の対象テストを通した。
- 試作では Rust の opt-in 生成テスト 6 件と、固定 seed の 1,363 入力で scan on/off を
  比較する UTF-8・word 境界・escape・反復下限・診断の差分テストを通した。
- [セッション 1](evidence/2026-11-21-swar-final/session-1.csv)・[uptime](evidence/2026-11-21-swar-final/session-1.uptime)、
  [セッション 2](evidence/2026-11-21-swar-final/session-2.csv)・[uptime](evidence/2026-11-21-swar-final/session-2.uptime)。
- [帰属別 Ir](evidence/2026-11-21-swar-final/attribution.csv)、
  [リプレイ Ir](evidence/2026-11-21-swar-final/buckets.csv)、
  [Callgrind 生データ](evidence/2026-11-21-swar-final/callgrind.tar.gz)・[uptime](evidence/2026-11-21-swar-final/profile.uptime)。
- [入力・生成物のハッシュ](evidence/2026-11-21-swar-final/parser-hashes.txt)、
  [配置対照のアドレス](evidence/2026-11-21-swar-final/layout.txt)。

## 再現手順

専用 worktree のルートで実行する。旧 #48 後は `66a18ff`、今回の基準は `bc49efc`、
試作は `41526b8`。IR の生成には基準版の `ubnfc` を使う。各生成器の target は独立させる。

```sh
export UBNFC_ROOT="$PWD" RUSTUP_TOOLCHAIN=stable
scratch=$(mktemp -d)
scripts/fetch-external-fixtures.sh "$scratch/fixtures" > "$scratch/fixtures.env"
set -a
. "$scratch/fixtures.env"
set +a
git clone --no-checkout https://github.com/nst/JSONTestSuite.git "$scratch/suite"
git -C "$scratch/suite" archive 1ef36fa01286573e846ac449e8683f8833c5b26a |
  tar -x -C "$scratch/suite"

for rev in bc49efc 41526b8 66a18ff; do
  mkdir "$scratch/$rev"
  git archive "$rev" | tar -x -C "$scratch/$rev"
done
CARGO_TARGET_DIR="$PWD/target/issue64-scalar" cargo build --release \
  --manifest-path "$scratch/bc49efc/Cargo.toml" --bin ubnfc-rust
CARGO_TARGET_DIR="$PWD/target/issue64-base" cargo build --release \
  --manifest-path "$scratch/bc49efc/Cargo.toml" --bin ubnfc --bin ubnfc-rust --features ubnfc-rust/swar
CARGO_TARGET_DIR="$PWD/target/issue64-candidate" cargo build --release \
  --manifest-path "$scratch/41526b8/Cargo.toml" --bin ubnfc-rust --features ubnfc-rust/swar
CARGO_TARGET_DIR="$PWD/target/issue64-old" cargo build --release \
  --manifest-path "$scratch/66a18ff/Cargo.toml" --bin ubnfc-rust

evidence=docs/reports/evidence/2026-11-21-swar-final
work="$PWD/target/issue64-ab"
mkdir -p "$work"
python3 "$evidence/measure.py" "$work" prepare "$scratch/suite" \
  "$PWD/target/issue64-scalar/release/ubnfc-rust" "$PWD/target/issue64-base/release/ubnfc-rust" \
  "$PWD/target/issue64-candidate/release/ubnfc-rust" "$PWD/target/issue64-old/release/ubnfc-rust"
python3 "$evidence/measure.py" "$work" time 1 30 2> "$scratch/session-1.uptime"
image="ubnfc-issue64-$(date +%s)-$$"
docker build -t "$image" benchmarks/attribution
python3 "$evidence/profile.py" prepare "$work" "$image" 30
python3 "$evidence/profile.py" measure "$work" "$image" 30
python3 "$evidence/attribute.py" "$work"
python3 "$evidence/measure.py" "$work" time 2 30 2> "$scratch/session-2.uptime"
```

実時間の中央値・範囲と同じ周の速度比は
`python3 "$evidence/stats.py" "$work/session-1.csv" "$work/session-2.csv"` で集計する。
