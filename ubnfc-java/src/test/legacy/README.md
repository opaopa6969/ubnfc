# 旧 Java 診断の独立実測

`../resources/legacy-diagnostics/*.json` は **旧 Java 実測値**。`LegacyDiagnosticsDump.java` が
旧 `unlaxer-parser` の生成器と runtime だけで root parser を実行し、入力ごとに新しい
`ParseContext` から `ParseFailureDiagnostics.getFarthestOffset()` と `getExpectedTokens()` を採取した。
候補は表示文字列を保持してソートするだけで、EOF・空白・コメント・内部 parser 名も除去しない。
ubnfc の生成器や実行結果から期待値を作成していない。通常の Maven テストは保存済みの実測値との
完全一致を検証するため、旧 Java の checkout / build は不要。

実測日: 2026-09-22。旧 Java checkout: `ed4178e673f93e719b62a7bd181f3a219abf8ed1`
（作業ツリー clean）、JDK 21.0.9。既存の `unlaxer-common/target/classes` と
`unlaxer-dsl/target/classes` を読み取り専用で使用した。依存は Vavr 0.10.4 / Gson 2.11.0。

| fixture | 旧 Java 文法（unlaxer-parser からの相対パス） | 入力（ubnfc からの相対パス、input のみ使用） | 件数 |
|---|---|---|---:|
| `operators.json` | `unlaxer-dsl/src/test/resources/associative/Operators.ubnf` | `examples/handwritten-target/java/src/test/resources/corpus/operators.json` | 24 |
| `power.json` | `unlaxer-dsl/src/test/resources/right-associative/Power.ubnf` | `examples/handwritten-target/java/src/test/resources/corpus/power.json` | 28 |
| `tinycalc.json` | `unlaxer-dsl/examples/tinycalc.ubnf` | `ubnfc-java/src/test/resources/golden/tinycalc.json` | 8 |

`BackendTest` の 60 件に加え、`DriverTest.unicodeCapturesPrefixAndFullInputObservations` は
Operators の入力 `"1+"` の同じ実測値を使用する（更新対象は計 61 テスト、独立実測は 60 入力）。
各 fixture 行の `_comment` にも実測ドライバ・文法・入力の参照方法を記録した。
手書き目標の golden およびテストは歴史的資料として保持し、AST・受理・消費範囲・captures の
既存検証も保持する。主診断の `expected` と最遠失敗の `farthestExpected` は別々に検証する。

TinyCalc は旧 bootstrap の制限を避けるため、ドライバ内で先頭の説明コメントを除き、
CRLF を LF に変換し、冗長な `@comment: { line: "//" }` 行を取り除く。
`@whitespace: javaStyle` 自体が同じ `CPPComment` と `BlockComment` を登録するため、
実効文法は変わらない（旧 `ParserGenerator.java` の `createContext`、275–309 行）。
文法ファイルそのものは変更せず、正規化した実測用ソースも一時ディレクトリに保存する。

採取に渡した文法ソースの SHA-256:

| 文法 | SHA-256 |
|---|---|
| Operators | `1ab3ecfbeac4ca959047fdf1f694fe60978cd60de3470434a2d72338633cc5d9` |
| Power | `c3945ec12f4f0f9ab20195d581fc2a3ae5e67165df855734874a9c5563d28298` |
| TinyCalc（正規化後） | `709e181c2b6e671bbeb023b01b5078d5271f0ec3de6d6814cbf2b85a28968d4b` |

再実測は ubnfc worktree のルートで次を実行する。出力は自分専用の一時ディレクトリに限定する。
最後の比較はコメントを含む保存済み JSON 全体の一致を検証し、fixture を上書きしない。

```bash
java_r3_legacy=/home/opa/work/unlaxer-workspace/unlaxer-parser
java_r3_scratch=$(mktemp -d)
java_r3_cp="$java_r3_legacy/unlaxer-common/target/classes:$java_r3_legacy/unlaxer-dsl/target/classes:$HOME/.m2/repository/io/vavr/vavr/0.10.4/vavr-0.10.4.jar:$HOME/.m2/repository/com/google/code/gson/gson/2.11.0/gson-2.11.0.jar"
javac --enable-preview --release 21 -cp "$java_r3_cp" -d "$java_r3_scratch" \
  ubnfc-java/src/test/legacy/LegacyDiagnosticsDump.java
java --enable-preview -cp "$java_r3_scratch:$java_r3_cp" LegacyDiagnosticsDump \
  "$java_r3_legacy" "$PWD" "$java_r3_scratch"
python3 - "$java_r3_scratch" <<'PY'
import json, pathlib, sys
scratch = pathlib.Path(sys.argv[1])
fixtures = pathlib.Path('ubnfc-java/src/test/resources/legacy-diagnostics')
for grammar in ('operators', 'power', 'tinycalc'):
    measured = json.loads((scratch / f'legacy-{grammar}.json').read_text())
    expected = json.loads((fixtures / f'{grammar}.json').read_text())
    assert measured == expected, grammar
    print(grammar, len(measured), 'exact match')
PY
```
