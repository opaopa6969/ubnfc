# 関数別の帰属レポート（issue #6）

P4 complex（332 byte）と JSON 100 KB を、認識のみ／所有 typed AST に分けて測る。
parser・front のソースは変更しない。Rust は現行生成器＋固定 P4 IR、Java P4 は
commit 済みの生成物、Java JSON は現行生成器を使う。各 parse の結果とその破棄までを含む。

## 準備と実行

必要なものは Docker、stable Rust（依存は cargo cache に必要）、JDK 21 と Maven
（依存は Maven cache に必要）、Python 3、C コンパイラ（`cc`）と `ar`、`taskset`、`c++filt`。
WSL2 で host に perf / valgrind をインストールする必要はない。
**既存 container には接続せず、自分専用 image と `--rm` container だけを使う。**

```bash
work=$(mktemp -d)
image="ubnfc-attribution-$(id -u)-$(date +%s)"
export WORK="$work" IMAGE="$image" RUSTUP_TOOLCHAIN=stable CORE=30
# 外部 fixture を読むテストも行う場合。pin は変更しない。
d=$(mktemp -d)
eval "$(scripts/fetch-external-fixtures.sh "$d" | sed 's/^/export /')"
docker build -t "$image" benchmarks/attribution
benchmarks/attribution/prepare.sh
benchmarks/attribution/run.sh probe
benchmarks/attribution/run.sh rust
benchmarks/attribution/run.sh perf
benchmarks/attribution/run.sh java
```

`prepare.sh` は生成物・別 crate・Maven のビルドコピー・クラス・target・入力を WORK に置く。
JSONTestSuite は既存の比較と同じ commit `1ef36fa01286573e846ac449e8683f8833c5b26a` を
WORK に取得し、`make-json-inputs.py` で入力を作る（100,039 byte）。P4 は既存の `complex.tiny`。
WORK は引用符・backslash を含まない絶対パスを使う。`versions.txt` と `inputs/inputs.sha256` を残す。

`probe` は host のツール、Docker のバージョン、`perf_event_paranoid`、通常権限の perf、
`PERFMON` を追加した perf、cachegrind の動作を記録する。
2026-11-01 の WSL2 では通常 perf は `No permission to enable task-clock event`、
専用 container の `--cap-add PERFMON --security-opt seccomp=unconfined` で成功した。
この権限は perf の container だけに与える。host の sysctl は変更しない。
環境によって拒否される場合は結果を記録し、`rust` と `java` で採れる列を埋め、未取得を明示する。

## 何を測るか

| 列 | 取得方法 | 意味・制約 |
|---|---|---|
| 自己 CPU 時間 ns/parse | perf `cpu-clock:u`、499 Hz、DWARF call graph、自己 period 合計 / 反復数 | サンプリング推定。包含時間ではない。host の wall time と異なる。起動・暖機 100 parse も含むため十分に反復する |
| 関数の自己 CPU % | perf `report --no-children --call-graph none` | inline は最適化後の symbol に帰属。細かい bounds check をこれだけから特定できない |
| Ir / Dr / Dw | callgrind（cache / branch simulation 有効） | 実行命令・data read / write の模擬計数。実機の retired instructions と同一ではない |
| Bc / Bcm / Bi / Bim | 同上 | 条件分岐／間接分岐と予測失敗の模擬値。実 CPU の predictor ではない |
| I1mr / D1mr / D1mw | 同上 | L1 instruction miss／data read miss／write miss の模擬値 |
| ILmr / DLmr / DLmw | 同上 | last-level cache miss の模擬値。構成は `.log` / callgrind header を参照 |
| calls / parse | callgrind の入辺 `calls=` の和 / 反復数 | 最適化後の関数呼出し。inline された操作の回数は得られない |
| Java 自己 CPU sample % | JFR `jdk.ExecutionSample` の main thread の先頭 frame | 暖機後だけ収録。GC / compiler thread は除外。絶対時間・命令数・miss 数は JFR からは未取得 |
| Java inline 判断 | `-XX:+UnlockDiagnosticVMOptions -XX:+PrintInlining` | `.inlining.txt` の `too big` / `hot method too big` / `inline (hot)` 等。CPU サンプルと照合し、表示があるだけで原因と断定しない |

`rust` は 100 parse 暖機後、`attribution::measured` 内の COUNT（既定 10）parse だけを
callgrind に収集させる。公式 client request header を Docker から WORK に取り出して
C wrapper をリンクし、`--instr-atstart=no` から暖機後だけ計装を開始・停止する。
最初の収集 parse の cache は cold、残り 9 parse は warm。`--collect-atstart=no` と
`--toggle-collect` だけでは `calls=` に暖機分が残るため採用しない。
`*.simulated.csv` の時間は valgrind によって遅くなった値なので性能の時間比較には使わない。
`*.native.csv` は別実行の実時間。CPU を CORE に固定し、`*.load.txt` で競合を記録する。

`perf` の既定反復は P4 100,000 / JSON 5,000。`P4_PERF_COUNT` / `JSON_PERF_COUNT` で変更する。
stat（実機の総カウンタ）と record（自己時間）は別実行。どちらも起動・暖機を含むため
callgrind の区間とは厳密には一致しない。`*.record.log` / report で標本数と lost samples を確認する。
Java は 2,000 parse 暖機後 `SECONDS_PER_CASE`（既定 15）秒を JFR の API で収録する。
`StartFlightRecording` と同じ JFR の `profile` 設定を使い、起動・暖機サンプルを混ぜない。
JFR を併用した `ns/parse` は JFR の overhead を含み、Rust との直接の速度比較には使わない。

## 定型と生データ

`run.sh` は `WORK/results/` に次を保存する。OUT で出力先を変更できる（WORK の配下にする）。

- `rust-*.callgrind` / `.annotated.txt` / `.tsv`: 完全な模擬計数、標準注釈、自己コストの表。
- `perf-*.stat.csv` / `.data` / `.self.txt` / `.tsv`: 実機総カウンタ、サンプル、自己 CPU 時間の表。
- `java-*.jfr` / `.samples.json` / `.tsv` / `.inlining.txt`: 暖機後の録画、全 sample、自己 sample 表、inline 判断。
- `*.native.csv` / `*.load.txt` / `versions.txt`: 時間・反復数・競合・生成元。

```bash
python3 benchmarks/attribution/summarize.py callgrind "$WORK/results/rust-p4-recognize.callgrind" 10
python3 benchmarks/attribution/summarize.py perf "$WORK/results/perf-p4-recognize.self.txt" 100000
python3 benchmarks/attribution/summarize.py jfr "$WORK/results/java-p4-recognize.samples.json"
```

レポートには上位関数（自己 CPU%・自己 Ir%）と、次の表を入れる。異なる回数を混ぜない。
inline されていない関数なら calls を使い、inline で回数が失われるときは入力 byte を共通単位にする。
CPU 時間・実機カウンタ・模擬値はそれぞれの実行から割って列を並べる。因果関係は A/B で別途検証する。

| 入力 / mode / 対象 | 回数/parse（単位を明記） | 時間/parse | 単価 ns/回 | 単価内訳 Ir/回 | Bcm+Bim/回（模擬） | D1mr+D1mw/回（模擬） | ILmr+DLmr+DLmw/回（模擬） |
|---|---:|---:|---:|---:|---:|---:|---:|
| P4 / 認識 / 全体 | 入力 332 byte | 実測 | 時間 / 332 | Ir / 332 | … | … | … |
| 関数名 | calls | 自己時間推定 | 自己時間 / calls | 自己 Ir / calls | … | … | … |

取得できない値は `未取得`。0 と書かない。実機 miss、模擬 miss、自己時間、包含時間を足さない。
生データは大きいので通常は要約 TSV・stat・ログと入力 hash を repository に残す。
今回の例は [`2026-11-01-attribution.md`](../../docs/reports/2026-11-01-attribution.md)。

## 検証と片付け

```bash
bash -n benchmarks/attribution/{prepare,run}.sh
python3 -m unittest discover -s benchmarks/attribution -p 'test_*.py'
# 結果を必要な場所に保存した後（自分が作ったものだけ）
docker image rm "$image"
rm -rf -- "$work" "$d"
```
