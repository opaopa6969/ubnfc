# Java SWAR の先頭文字判定と固定 JSON 1 MB の再計測（#68）

固定 JSON 1 MB の認識は、現行 SWAR の 10.540 ms から試作の 9.834 ms に短縮した（1.072 倍）。両セッションで改善を再現したため、先頭文字の判定を opt-in の `--swar` に採用する。D-080 後比は 0.926 倍で、1.5 倍には届かなかった。

## 試した変更

Java SWAR は、4 UTF-16 コード単位を `long` に詰めて、文字クラスとの一致をまとめて判定する。固定 JSON 1 MB では、word 判定の 63.6% が先頭で不一致になっていた。今回の試作 `81ecc08` は、入力が残り 4 コード単位以上の場合に先頭の 1 文字を調べる。ASCII なら既存のマスクと同じ区間を使う直接比較を生成し、不一致なら一括走査を終える。4 lane の読み込みと判定は、先頭の ASCII が通ったときに行う。先頭が非 ASCII なら従来の code point 判定に渡し、末尾や escape などの子の処理も従来の経路を使う。[PR #69](https://github.com/opaopa6969/ubnfc/pull/69)

## 固定 JSON の反復箇所別の回数

以下は計時とは別に、計装版で JSON 1 parse を数えた結果。lane 0〜4 は従来版で word を組んだ後に一致した先頭 lane の数。

| 反復 | lane 0 | lane 1 | lane 2 | lane 3 | lane 4 | 試作で省いた ASCII / 非 ASCII |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 空白（`scan6`） | 183,623 | 52,843 | 0 | 0 | 0 | 183,623 / 0 |
| 文字列の中身（`scan58`） | 81,918 | 17,175 | 2,642 | 15,852 | 34,347 | 77,955 / 3,963 |
| 整数部の後続桁（`scan97`） | 6,605 | 1,321 | 0 | 0 | 0 | 6,605 / 0 |
| 小数部（`scan101`） | 0 | 2,642 | 1,321 | 0 | 25,099 | 0 / 0 |
| 指数部（`scan112`） | 0 | 0 | 2,642 | 0 | 0 | 0 / 0 |
| 計 | 272,146 | 73,981 | 6,605 | 15,852 | 59,446 | 268,183 / 3,963 |

試作では各反復の lane 0 が 0 になり、lane 1〜4 の回数はすべて一致した。省いた 272,146 回の 67.5% は空白の反復に集中した。試作で実行した word 判定 155,884 回のうち、96,438 回は 1〜3 lane で止まり、4 lane すべてを消費したのは 59,446 回（38.1%）だった。[反復別の集計](evidence/2026-11-21-java-swar-short/counts.csv)

## 実時間と認識スレッドの CPU 時間

- 基準は作業開始時の `daa22a0` の scalar 生成と `--swar` 生成。試作は `81ecc08` の `--swar`。D-080 後の `a990dac` の scalar 生成も JSON に限り同じ各周で測った。
- 固定 JSON は JSONTestSuite `1ef36fa01286573e846ac449e8683f8833c5b26a` から復元した 1,000,112 byte。SHA-256 は #69 と同じ。P4 は固定 IR と 6 fixture を使用。通常の 4 fixture は Formula、comparison-heavy の 2 fixture は BooleanExpression で認識した。Operators は `+` でつないだ 512 atom。各 JVM で入力を読み込み、計時前にコードポイント単位で全入力の受理を照合した。
- AMD Ryzen 9 7950X、WSL2（Linux 6.18.33.2）、Oracle JDK 21.0.9。論理 CPU 28 に固定し、`-Xms512m -Xmx512m -Xss16m -XX:+UseSerialGC -XX:ActiveProcessorCount=1`。各入力を 200 parse 暖機し、1 JVM で固定反復数のサンプルを 3 個採った。同じ周に各版を別 JVM で 1 回ずつ実行し、実行順を周ごとに回転・反転させた。反転の開始順を変えて 7 周ずつ 2 セッション。実時間は `System.nanoTime`、CPU 時間は `ThreadMXBean.getCurrentThreadCpuTime` の差分。GC や JIT の別スレッドは CPU 時間の集計範囲外。
- 集計単位は各 JVM のサンプル 3 個の中央値。表は全 14 周の中央値［最小–最大］、単位は ms / parse。各周の速度比は対照版の時間を試作の時間で割った値。全サンプルを掲載し、除外は行っていない。[セッション 1](evidence/2026-11-21-java-swar-short/session-1.csv)・[uptime](evidence/2026-11-21-java-swar-short/session-1.uptime)、[セッション 2](evidence/2026-11-21-java-swar-short/session-2.csv)・[uptime](evidence/2026-11-21-java-swar-short/session-2.uptime)

| 入力 | D-080 後 scalar | 現行 scalar | 現行 SWAR | 試作 | 現行 SWAR / 試作 | 試作が速い周 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 固定 JSON 1 MB | 9.106［8.625–18.016］ | 9.254［8.592–11.108］ | 10.540［10.175–13.570］ | 9.834［9.206–13.257］ | 1.072 倍 | 12 / 14 |
| `operators-atoms-512` | — | 0.085［0.078–0.093］ | 0.086［0.079–0.096］ | 0.087［0.080–0.095］ | 0.988 倍 | 7 / 14 |
| `complex` | — | 0.233［0.211–0.276］ | 0.237［0.199–0.292］ | 0.228［0.189–0.369］ | 1.037 倍 | 8 / 14 |
| `complex-x4` | — | 0.629［0.464–0.798］ | 0.679［0.534–0.745］ | 0.659［0.577–0.763］ | 1.030 倍 | 8 / 14 |
| `complex-x16` | — | 1.164［1.020–3.942］ | 1.141［1.048–2.589］ | 1.123［1.040–1.797］ | 1.016 倍 | 7 / 14 |
| `complex-x64` | — | 5.185［4.509–8.116］ | 5.180［4.094–7.323］ | 5.188［3.941–6.900］ | 0.998 倍 | 7 / 14 |
| `comparison-heavy` | — | 0.055［0.046–0.060］ | 0.053［0.047–0.064］ | 0.057［0.045–0.069］ | 0.942 倍 | 8 / 14 |
| `comparison-heavy-x64` | — | 2.202［2.070–2.819］ | 2.222［2.080–2.523］ | 2.215［2.094–3.311］ | 1.003 倍 | 8 / 14 |

| セッション | JSON 実時間（D-080 後 / scalar / 現行 SWAR / 試作） | JSON CPU 時間（同順） | 開始時 load average | 終了時 load average |
| --- | ---: | ---: | ---: | ---: |
| 1 | 8.950 / 9.278 / 10.347 / 9.973 | 8.874 / 9.250 / 10.339 / 9.881 | 11.77 / 10.39 / 10.54 | 5.25 / 7.90 / 9.50 |
| 2 | 9.175 / 9.046 / 10.597 / 9.675 | 9.037 / 9.030 / 10.371 / 9.552 | 4.96 / 7.29 / 9.17 | 5.92 / 6.81 / 8.42 |

load average の各列は 1 / 5 / 15 分。全記録の範囲は 4.07–11.77 / 6.36–10.39 / 8.42–10.54。

固定 JSON で現行 SWAR と試作を同じ周で割った速度比の中央値は、セッション 1 が 1.090 倍、セッション 2 が 1.103 倍。いずれも試作が 7 周中 6 周で速かった。全 14 周の速度比の中央値は 1.094 倍［0.780–1.403 倍］、CPU 時間では 1.092 倍［0.796–1.418 倍］で 12 / 14 周で短縮した。1 周ごとのばらつきはあるが、実時間と CPU 時間の中央値は両セッションで下がった。

P4 6 fixture と Operators は、scalar・現行 SWAR・試作で生成ソースと対象 class が一致した。試作列に出た中央値の差は P4 で約 −3.6%〜+6.1%、Operators で約 +1.3%。同じコードでも観測値はこの範囲で動き、変更によるコード上の退行はない。実時間で一律の無退行を判定するには今回の対照のばらつきが大きい。

## JFR による認識経路の CPU 標本

計時と分け、計装を加えていない 3 版の JSON 認識を別 JVM で各 200 parse 暖機した後、約 20 秒ずつ記録した。JFR の `ExecutionSample` と `NativeMethodSample` を 2 ms 周期で有効化し、main スレッドで `JsonParser.parse` を含む標本を集計した。自己標本は先頭フレーム、包含標本は対象の反復関数をスタックに含むもの。コンパイラによるインライン化や標本点の偏りがあるため、メソッドの精密な CPU 時間や追加分岐の費用はこの集計からは求められない。[標本の集計](evidence/2026-11-21-java-swar-short/jfr.csv)・[uptime](evidence/2026-11-21-java-swar-short/jfr.uptime)

| 版 | 認識標本数 | 対象反復と SWAR 補助関数の自己標本 | 対象反復の包含標本 | 空白走査の自己標本 |
| --- | ---: | ---: | ---: | ---: |
| scalar | 5,373 | 1,004（18.7%） | 1,465（27.3%） | 295（5.5%） |
| 現行 SWAR | 5,872 | 1,459（24.8%） | 2,576（43.9%） | 1,112（18.9%） |
| 試作 | 5,580 | 1,234（22.1%） | 1,789（32.1%） | 462（8.3%） |

空白走査の自己標本の割合が 18.9% から 8.3% に下がり、word 判定を多く省いた箇所と一致した。文字列の中身の包含標本は現行 SWAR が 1,357 / 5,872（23.1%）、試作が 1,190 / 5,580（21.3%）だった。試作の認識標本の約 3 分の 2 は対象反復を含まないスタックにあり、規則・式の評価、位置変換などにも処理が残る。

## 採否と目標の見直し

先頭文字の判定を `--swar` に採用する。固定 JSON では現行 SWAR の 10.540 ms から 9.834 ms へ 6.7% 短縮し、同じ方向の差を 2 セッションの実時間・CPU 時間で観測した。既定の scalar はバイト一致を確認し、固定 JSON でも 9.254 ms と速いため引き続き既定とする。

D-080 後の 9.106 ms を基準に、試作は **0.926 倍**。1.5 倍に必要な 6.070 ms へはあと 3.763 ms、試作から約 38.3% の短縮が要る。先頭 lane で失敗する word 判定を省いても、1〜3 lane で止まる判定が 96,438 回残る。word を組む経路には先頭文字の比較も加わる。加えて試作で直接触れた反復を含む JFR 標本は認識の約 32% で、規則評価など走査の外にも費用が残る。

固定 JSON 1 MB の 1.5 倍はパーサ全体の指標に見直す。SWAR 単独では、長い一致区間のある入力で scalar に対する短縮が複数セッションで再現することと、既定の生成がバイト一致することを採否の目安とする。

## 検証

- 試作の既定生成ソースは基準の scalar と JSON・P4・Operators でバイト一致。P4 と Operators は opt-in の生成ソースと対象 class ファイルも両版でバイト一致した。[入力・ソース・クラスのハッシュ](evidence/2026-11-21-java-swar-short/prepare.txt)
- 試作で `SwarScanTest` と `Ll1EquivalenceTest` の計 8 テストが成功した。固定 seed と境界を含む JSON 994 入力で `options.scan` の両値を比較した。UTF-16 の word 境界、escape、非 BMP、孤立サロゲート、反復下限、不正入力、出現表、診断、rollback の観測を比較した。別の IR で capture、scope、AST span を照合した。
- 試作の JSON 生成物を外部ライブラリなしで OpenJDK 17.0.21.0.3 の `javac -Xlint:all -Werror` でビルドした。同じ JDK 上のドライバに非 BMP・escape・長い数字列を含む入力を渡し、認識と AST 生成の成功を確認した。
- 外部 fixture は `scripts/fetch-external-fixtures.sh` で固定 pin を取得し、`UNLAXER_PARSER_DIR`、`UNLAXER_PARSER_ROOT`、`TINYEXPRESSION_DIR` を設定した。全体の検証コマンドは `mvn -q -DskipTests package`、`mvn -q test`、`RUSTUP_TOOLCHAIN=stable cargo test --workspace --no-fail-fast`。

## 再現

JDK 21、Maven、Git、Python 3、`taskset` を使用。現在の checkout から一時ディレクトリに比較用の 3 commit と計測用ファイルを展開する。既定で論理 CPU 28 を使う。`CORE` で変更できる。

```bash
root=$PWD
work=$(mktemp -d)
evidence=docs/reports/evidence/2026-11-21-java-swar-short
cp "$evidence"/{trial.sh,ShortScanBench.java,WordCounts.java,JfrCounts.java,word-counts.py,stats.py} "$work/"
for row in 'old a990dac' 'before daa22a0' 'candidate 81ecc08'; do
  read -r name revision <<< "$row"
  mkdir "$work/$name"
  git archive "$revision" | tar -x -C "$work/$name"
  mvn -q -f "$work/$name/ubnfc-java/pom.xml" -DskipTests package
done
git init -q "$work/suite-repo"
git -C "$work/suite-repo" fetch -q --depth 1 https://github.com/nst/JSONTestSuite.git \
  1ef36fa01286573e846ac449e8683f8833c5b26a
mkdir "$work/suite"
git -C "$work/suite-repo" archive FETCH_HEAD | tar -x -C "$work/suite"
python3 "$root/benchmarks/scripts/make-json-inputs.py" "$work/suite/test_parsing" "$work/inputs"
python3 -c 'print("+".join(["1"] * 512), end="")' > "$work/inputs/operators-atoms-512.txt"
export REPO="$work/candidate"
bash "$work/trial.sh" prepare > "$work/prepare.txt"
bash "$work/trial.sh" time 1 > "$work/session-1.csv" 2> "$work/session-1.uptime"
bash "$work/trial.sh" jfr > "$work/jfr.uptime" 2>&1
bash "$work/trial.sh" time 2 > "$work/session-2.csv" 2> "$work/session-2.uptime"
python3 "$work/word-counts.py" "$work" > "$work/counts.csv"
java "$work/JfrCounts.java" "$work" > "$work/jfr.csv"
python3 "$work/stats.py" "$work"
```
