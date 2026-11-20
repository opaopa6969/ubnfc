# Java P4 の認識費用と memo 保存の再探索（#67）

[PR #66](https://github.com/opaopa6969/ubnfc/pull/66) では反復と capture 集約の中間割当を減らした。complex AST の `Match` は 997 → 871 個、x64 は 60,343 → 52,909 個になった。変更後の代表値は complex で認識 0.376 ms / AST 0.679 ms、x64 で 35.690 ms / 58.023 ms だった。認識にも `MemoTable.locate` と `Match` の割当が残っていた。

今回は master の `c64a19d` と試作 `1953602` を比較した。基準版のコミット済み P4 生成物は #66 と同じで、後続の生成器更新による差がある。比較には両版ともその版の生成器で固定 IR から再生成した P4 を使い、LL(1)・SWAR は既定設定のまま測った。Operators は既存の生成手順を使った。

## 認識と AST のプロファイル

Oracle JDK 21.0.9、AMD Ryzen 9 7950X の論理 CPU 28、`-Xms512m -Xmx2g -Xss16m`。基準版の各入力・モードを別 JVM で 1,000 parse 暖機後、約 8 秒間 JFR に記録した。CPU は JFR の `ExecutionSample` の先頭フレームに帰属する自己標本。P4 パーサを含む main thread のスタックを分母にした。括弧内はその入力・モードの分母に対する割合。

| 帰属先 | complex 認識 | complex AST | x64 認識 | x64 AST |
|---|---:|---:|---:|---:|
| 自己標本の合計 | 1,304 | 845 | 1,573 | 889 |
| `MemoTable.locate` | 323（24.8%） | 94（11.1%） | 899（57.2%） | 357（40.2%） |
| `Session.restore` | 207（15.9%） | 73（8.6%） | 124（7.9%） | 33（3.7%） |
| `Match.empty` | 12（0.9%） | 6（0.7%） | 10（0.6%） | 4（0.4%） |
| `Session.wrap` | 6（0.5%） | 34（4.0%） | 5（0.3%） | 47（5.3%） |
| `Session.combine` | 0（0.0%） | 31（3.7%） | 0（0.0%） | 15（1.7%） |

`Match.empty` の自己標本は 1% 未満だが、認識の JFR 割当標本を weight で集計すると、`Match` 型は complex 53.0% / x64 53.2%、`Match.empty` を含むスタックは 49.0% / 49.4% を占める。P4 は scope 効果があるため、認識でも区間を持つ `Match` を使う。AST では `Match` 型が 30.7% / 32.7%、`wrap` を含むスタックが 24.5% / 25.9%、`combine` を含むスタックが 10.2% / 11.2%。型別とスタック別の集計には重複がある。最初の割当標本は記録前の暖機を含む値と判断して比率から外した。たとえば x64 AST では定常区間全体の割当カウンタ差が 2.13 GB に対し、最初の標本だけで 9.85 GB の weight が付いた。元の標本は証拠データに残した。

| 入力・モード | main CPU（ms / parse） | 実時間（ms / parse） | 割当（B / parse） | GC 停止の合計 / 実時間 |
|---|---:|---:|---:|---:|
| complex 認識 | 0.0945 | 0.1286 | 69,973 | 107.1 / 8,000.0 ms（1.34%） |
| complex AST | 0.2509 | 0.4116 | 163,282 | 109.2 / 8,020.6 ms（1.36%） |
| x64 認識 | 7.1401 | 9.6266 | 3,963,245 | 45.9 / 8,009.4 ms（0.57%） |
| x64 AST | 21.1547 | 36.5687 | 9,721,872 | 130.0 / 8,008.5 ms（1.62%） |

実時間と CPU の差は GC 停止の合計より大きい。GC 停止だけでこの差を説明できず、残りを OS のスケジューリング・JIT・JFR 自体の費用に個別帰属する計測はしていない。プロファイルはモードごとの JVM なので、時間・割当の採否には同一 JVM の交互計測を使う。

## 試した変更

認識時の `locate` の自己標本のうち、`get` からの呼び出しは complex で 74 / 323 件、x64 で 666 / 899 件だった。`put` からの呼び出しはそれぞれ 249 件、233 件だった。x64 の認識全体に占める保存側の探査は 14.8% だった。

自己 CPU 標本が最大だった memo 探索を対象にした。検索が未登録だった位置を解析本体の評価中に保持し、保存時の探査をそこから再開する。入れ子の保存でその位置が埋まった場合は線形探査を続け、表が拡張された場合はハッシュからやり直す。検索で先に通過した slot は同じ世代・容量では占有されたままなので、空き位置より前を再確認する必要がない。ハッシュの定数、キーの同値条件、memo 対象の選び方は同じ。

計時外にカウンタを付け、各入力・モードを 1 parse ずつ実行した。以下の件数は認識と AST で一致した。

| 入力 | ハッシュ計算（基準 → 試作） | 探査の開始回数 | 衝突後の前進回数（基準 → 試作） | 表の拡張 |
|---|---:|---:|---:|---:|
| complex | 1,185 → 676（−43.0%） | 1,185 | 1,042 → 710（−31.9%） | 0 |
| x64 | 71,367 → 40,870（−42.7%） | 71,367 | 12,186 → 6,901（−43.4%） | 0 |

## 同一 JVM の交互計測

各セッション 7 run、3 セッション。先行順を run とセッションごとに反転した。P4 は各セル 1,000 回暖機、complex 2,000 回 / x64 50 回を 1 run とした。Operators 512 atom は 20,000 回暖機、1 run 1,000 回。Rust stable 1.100.0 の参照も同じ論理 CPU で測った。Java main thread と Rust の解析 thread の CPU 時間をセルごとに記録した。実時間には CPU 時間外の待ちも含まれる。

各欄はセッション中央値の中央値［全 run の最小–最大］。単位は ms / parse。全 run を集計した。採否では、同じ run の基準 / 試作の比も併記して、実行順や負荷の変動の影響を確認する。

| 入力 | モード・版 | 実時間 | 解析 thread CPU | 割当 B / parse |
|---|---|---:|---:|---:|
| complex | AST・基準 | 0.24306 ［0.11257–0.70592］ | 0.15880 ［0.11186–0.28547］ | 230224 |
| complex | AST・試作 | 0.23908 ［0.11416–0.60597］ | 0.15539 ［0.11356–0.25433］ | 230248 |
| complex | 認識・基準 | 0.14969 ［0.07168–0.34076］ | 0.09691 ［0.07132–0.13965］ | 136952 |
| complex | 認識・試作 | 0.14100 ［0.07429–0.30714］ | 0.09574 ［0.07398–0.13344］ | 136976 |
| complex | AST・Rust | 0.17635 ［0.10551–0.34761］ | 0.10525 ［0.08666–0.14599］ | — |
| complex | 認識・Rust | 0.14514 ［0.07253–0.29878］ | 0.08094 ［0.06130–0.09926］ | — |
| complex-x64 | AST・基準 | 19.11729 ［7.68038–52.63272］ | 14.50742 ［7.46963–21.15391］ | 9783920 |
| complex-x64 | AST・試作 | 20.94502 ［7.85007–55.65851］ | 13.87293 ［7.60383–24.44557］ | 9783944 |
| complex-x64 | 認識・基準 | 13.24251 ［3.81321–35.82940］ | 8.57812 ［3.77722–14.44551］ | 4026296 |
| complex-x64 | 認識・試作 | 11.68419 ［3.80170–36.40643］ | 7.66615 ［3.76635–14.51595］ | 4026320 |
| complex-x64 | AST・Rust | 29.73477 ［9.17336–80.14468］ | 13.17785 ［7.41824–27.46801］ | — |
| complex-x64 | 認識・Rust | 11.82582 ［5.77549–34.30470］ | 6.95055 ［4.45105–15.34442］ | — |
| atoms-512 | AST・基準 | 0.68480 ［0.32054–1.32223］ | 0.40373 ［0.30074–0.51180］ | 1096472 |
| atoms-512 | AST・試作 | 0.66961 ［0.31294–1.39180］ | 0.37982 ［0.28202–0.57969］ | 1096472 |
| atoms-512 | 認識・基準 | 0.12798 ［0.06393–0.27633］ | 0.08134 ［0.06315–0.13552］ | 16696 |
| atoms-512 | 認識・試作 | 0.13642 ［0.07550–0.22344］ | 0.08401 ［0.06280–0.10818］ | 16696 |
| atoms-512 | AST・手書き | 1.17465 ［0.61966–2.28245］ | 0.71533 ［0.52167–0.92771］ | 2768784 |

| 受け入れ条件 | 基準 | 試作 | 判定 |
|---|---:|---:|---|
| complex AST ≤ 0.15 ms | 0.24306 | 0.23908 | 両版未達 |
| x64 AST ≤ 15 ms | 19.11729 | 20.94502 | 両版未達 |
| complex AST 層 ≤ Rust の 2 倍 | 2.40 倍 | 2.52 倍 | 両版未達 |
| x64 AST 層 ≤ Rust の 2 倍 | 0.39 倍 | 0.52 倍 | 両版達成 |
| Operators AST ≤ 同一 JVM の手書き | 0.68480 ms | 0.66961 ms | 両版達成（手書き 1.17465 ms） |

AST 層はセッションごとに AST と認識の実時間中央値の差を取り、3 セッションの代表値で Rust と比較した。CPU 時間で同じ計算をした場合は、complex が 2.45 → 2.45 倍、x64 が 1.06 → 1.00 倍だった。Rust は同じ core でセッション内の別区間に測ったため、Java と Rust の実時間比にも計測区間ごとの負荷差が入る。

| 入力・モード | 代表値の速度比 | 同じ run の実時間比の中央値［範囲］ | 試作が短縮した組 | 同じ run の CPU 時間比の中央値 |
|---|---:|---:|---:|---:|
| complex 認識 | 1.062 | 0.979［0.685–1.478］ | 9 / 21 | 0.981 |
| complex AST | 1.017 | 1.020［0.685–1.714］ | 12 / 21 | 0.989 |
| x64 認識 | 1.133 | 1.021［0.517–1.869］ | 11 / 21 | 1.056 |
| x64 AST | 0.913 | 1.083［0.735–1.432］ | 14 / 21 | 1.046 |

速度比は基準 / 試作。両 JVM 版は独立した class loader に載せた。x64 認識の同じ run の実時間比は、セッション別の中央値で 0.995 / 0.984 / 1.119。x64 AST は同じ run の比較で短縮が多かった一方、セッション代表値は悪化し、3 セッション目の CPU 時間比の中央値も 0.949 倍だった。Java 側の割当代表値は 24 B / parse 増で、試作に持続する割当削減は見られない。

計測中の `uptime` は [全記録](evidence/2026-11-21-java-p4-recognition/uptime.txt) に掲載した。P4 の開始時点の load average（1 / 5 / 15 分）は順に 13.70 / 6.76 / 6.46、18.95 / 13.43 / 9.16、19.43 / 16.62 / 11.72。区間の境界では 1 分値が 9.62–36.29 の範囲で変動し、別 core のローカル検証と共有機の処理が並行した。

## 採否と目標の見直し

試作は採用しない。ハッシュ計算と衝突時の前進は減ったが、表を読む探査の開始回数は同じで、x64 の `locate` の自己標本の約 4 分の 3 を占める検索側も残る。complex の認識は同じ run で見た実時間・CPU 時間のどちらにも短縮を確認できず、x64 の短縮も実時間のセッション間で再現しなかった。complex は現行の認識代表値だけで 0.15 ms の予算のほぼ全てを使い、試作でも AST 層に 0.098 ms 残る。x64 は試作の認識代表値 11.684 ms に対して 15 ms まで 3.316 ms だが、AST 層は 9.261 ms だった。AST の `wrap` / `combine` に残る割当費用もこの試作の範囲外にある。

共有機での採否は、固定入力・同一 run の認識 CPU 時間と AST 実時間を、同一版同士の対照で得た変動幅と比べ、複数セッションで再現する短縮を条件とする。0.15 / 15 ms は、同時処理を抑えた CPU で分けて評価する実時間目標とする。AST 層の Rust 比も実時間と CPU 時間を併記する。この試行で memo 保存が削れた分だけでは、認識・AST 両層の時間差を埋められない。

## 同値性と検証

- Java の package、生成器テスト 11,898 件、LSP の対象 19 件、固定 IR から再生成した試作 P4 34 件が成功した。直接経路と Recipe 経路の同値、投機的な枝の例外と公開 span を既存テストで確認した。
- `Recipe` のコンストラクタに計時外のカウンタを入れ、直接 AST では基準も試作も complex / x64 ともに 0 個だった。カウンタの対照として旧 Recipe 経路では 153 / 9,477 個を数えた。
- 同じ容量での memo 保存と、入れ子の保存中に容量が変わる場合を、固定 seed で通常の保存と比較した。12,800 回（容量維持 11,501、拡張 1,299）で登録数・成否・消費位置・一致位置・payload が一致した。
- 基準版でコミット済みの P4 に対して `mvn test` を実行すると、既存の生成物差により `generatedSourcesAreByteIdentical` が失敗する。固定 IR から再生成した基準版と試作版では同テストを含む P4 各 34 件がすべて成功した。
- `scripts/fetch-external-fixtures.sh` で取得した pin（unlaxer-parser `d3ea73e8`、tinyexpression `c736c2c6`）を設定し、同じ front・4 workers で Java corpus を比較した。展開後 1,492 件の `actual`、`outcome`、`differences`、`missing` と全区分の集計が一致した。両版とも Pass 1,377、期待拒否 Pass 27、Known 29、未観測 59、Fail 0、IR 失敗 0。[比較結果とハッシュ](evidence/2026-11-21-java-p4-recognition/corpus-diff.json)

## 計測データと再現手順

[全 run の実時間・CPU 時間・割当、IR と入力の SHA-256、uptime](evidence/2026-11-21-java-p4-recognition/) を保存した。JFR の [実行別集計](evidence/2026-11-21-java-p4-recognition/profile-runs.csv) と [抽出標本](evidence/2026-11-21-java-p4-recognition/profile-samples.csv.gz) には、CPU の先頭フレーム、割当型と weight、対象メソッドのスタック上の有無、GC 停止を記録した。CPU 標本の weight は 1、割当は B、GC は ns。

以下を専用 worktree のルートから実行する。計測データにあるパッチはベンチのセル単位の CPU 時間採取と Java の割当全サンプル出力に使う。

```bash
evidence="$PWD/docs/reports/evidence/2026-11-21-java-p4-recognition"
scratch=$(mktemp -d)
mkdir -p "$scratch"/{base,trial,tmp,timer}
export TMPDIR="$scratch/tmp" CARGO_TARGET_DIR="$PWD/target" RUSTUP_TOOLCHAIN=stable

git archive c64a19d | tar -x -C "$scratch/base"
git archive 1953602 | tar -x -C "$scratch/trial"
patch -d "$scratch/base" -p0 < "$evidence/thread-cpu.patch"
for rev in base trial; do
  repo="$scratch/$rev"
  mvn -q -f "$repo/ubnfc-java/pom.xml" -DskipTests package
  UBNFC_GENERATOR_JAR="$repo/ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar" \
    "$repo/examples/p4-java/scripts/generate.sh" --write
  base_work=
  if [[ "$rev" == trial ]]; then base_work="$scratch/base-build"; fi
  BASE_WORK="$base_work" WORK="$scratch/$rev-build" \
    "$repo/benchmarks/scripts/run-java-direct-ast.sh" prepare
done
javac --release 17 -d "$scratch/timer" "$scratch/base/benchmarks/java/JavaDirectAstBench.java"
cp "$evidence/measure.sh" "$scratch/"
bash "$scratch/measure.sh"
```

JFR は同じ `scratch` の基準版クラスに対して別に採取できる。

```bash
base="$scratch/base/examples/p4-java/target/classes"
fixtures="$scratch/base/examples/p4-java/src/test/resources/fixtures"
javac -cp "$base" -d "$scratch" "$evidence/"{ProfileP4,ReadProfile}.java
for fixture in complex complex-x64; do
  for ast in false true; do
    file="$scratch/$fixture-$ast"
    taskset -c 28 java -Xms512m -Xmx2g -Xss16m -XX:FlightRecorderOptions=stackdepth=128 \
      -cp "$scratch:$base" ProfileP4 "$fixtures/$fixture.tiny" "$ast" "$file.jfr"
    java -cp "$scratch" ReadProfile "$file.jfr" > "$file.tsv"
  done
done
python3 "$evidence/profile-summary.py" "$scratch" > "$scratch/profile-samples.csv"
```
