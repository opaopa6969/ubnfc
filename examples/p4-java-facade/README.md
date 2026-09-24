# tinyexpression P4PreferredAstMapper の互換 facade

tinyexpression の `org.unlaxer.tinyexpression.p4.P4PreferredAstMapper` と**同じ公開面**を持ち、
中身を ubnfc 生成パーサ（`examples/p4-java` の `org.ubnfc.p4.generated`）に差し替えたもの。
tinyexpression と `examples/p4-java` のソースは変更していない。

- 本体: `src/main/java/org/unlaxer/tinyexpression/p4/ubnfc/UbnfcP4PreferredAstMapper.java`
- 解析と選択: 同 `UbnfcP4Parse.java`
- AST 変換（生成物・手で編集しない）: 同 `UbnfcAstConverter.java`
- 差し込み用 shadow（生成物）: `src/shadow/java/org/unlaxer/tinyexpression/p4/P4PreferredAstMapper.java`

戻り値は tinyexpression 側の `TinyExpressionP4AST` / `P4SourceText` そのもので、
`ParsedAst(ast, selectionMode, sourceText)` の 3 つ組・`selectionMode` の文字列
（`preferred:X` / `default` / `explicit:object` / `document:...`）・失敗時の例外型とメッセージまで揃えてある。

## 組み立てと実行

tinyexpression の**作業ツリーのビルド出力**を読み取り専用で使う。公開済みの
`tinyExpression-1.4.15.jar`（`~/.m2` と `target/` の両方）は古く、facade が使う
mapper API（`mapParsedTree` 等）を持たないため、`target/classes` から jar を作り直す。
共有 `~/.m2` には何も書かない。

```bash
cd examples/p4-java-facade
bash scripts/prepare-te.sh          # ../../../tinyexpression/target/classes -> target/te/*.jar
mvn -o -q test                      # パリティ 340 件と単位テスト
bash scripts/path-proof.sh legacy   # tinyexpression 自身のテストを本来の facade で
bash scripts/path-proof.sh shadow   # 同じテストを ubnfc 版 facade に差し替えて
bash scripts/stage-timing.sh legacy # parse / 生成 / javac / 実行 の段別時間
bash scripts/stage-timing.sh shadow
```

`TINYEXPRESSION_DIR` で隣接 checkout の位置を、`MAVEN_REPO` / `UNLAXER_VERSION` で
unlaxer jar の探索先を変えられる。`scripts/prepare-te.sh` を先に走らせないと
`mvn -o -q test` は依存解決で止まる（system scope の jar が無いため）。

## 生成物の再生成

```bash
python3 scripts/generate-converter.py --check   # AST 変換器
python3 scripts/generate-shadow.py --check      # shadow クラス
```

`UbnfcAstConverter` は両側の record 定義（ubnfc 生成ソースと tinyexpression の jar）から
機械生成する。86 record すべてで名前・component 名・順序が一致しており、変換は 1:1。
唯一の宣言型の違いは `SliceExpr` の `start`/`end`/`step`（tinyexpression 側が
`Optional<Object>` に広がっている）だけで、これも生成器が吸収する。

## 差し込み方（tinyexpression は無変更）

tinyexpression には parser facade の SPI が無い（`ServiceLoader` も `META-INF/services` も無い）。
`P4PreferredAstMapper` は `public final class` + private コンストラクタ + 全 static、
呼び出し元の `DslGeneratedAstJavaEmitter` は package private final。したがって差し込み口は
**同 FQN クラスによる classpath shadow** だけになる。これは tinyexpression 自身が
unlaxer のコアクラスを shadow しているのと同じ手口で、`scripts/path-proof.sh shadow` が
`target/shadow-classes` を classpath の先頭に置いて実現する。

`org/unlaxer/compiler/CompileContext.java` は `org.unlaxer.tinyexpression.p4.P4PreferredAstMapper`
を **文字列で** `Class.forName` し、その CodeSource を javac の `-classpath` に足す。
shadow クラスは同じ FQN で解決できるので、この経路もそのまま働く。

## 旧実装との意図的な差

1. **解析期限**: ubnfc 生成パーサは packrat + 深さ上限なので、旧実装のような指数バックトラック
   (issue #19, #20) を起こさない。期限は解析の前後でだけ見る（解析途中で割り込む
   `TransactionListener` は持たない）。`ParseDeadlineExceededException` は公開面として残してある。
2. **span の単位**: ubnfc 側は最初から code point の半開区間しか持たない。旧実装は
   公開 3.0.15 の generator 向けに「code point の開始 + UTF-16 の長さ」という混在表現を
   `P4SourceMapping#normalizeLegacySpan` で後から正規化している。作業ツリーの現行ビルドでは
   この補正後の値が code point で一致する（`LegacySourceSpanTest` が固定）。
