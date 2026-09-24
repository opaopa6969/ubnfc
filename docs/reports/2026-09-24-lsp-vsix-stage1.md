# LSP サーバ生成と VSIX 梱包 — 段階 1（調査）と段階 2（薄い縦切り）

issue #28。ubnfc は「LSP は非ゴール」としてきた（README「インクリメンタル解析・LSP は無い」）。
ユーザー要望でこれを撤回し、**opt-in の feature** として LSP サーバ生成を足す（D-060）。
本報告は段階 1（旧スタックの棚卸しと材料の突き合わせ）と段階 2（診断だけ動く縦切り）。
段階 3（completion / hover / definition / references）と段階 4（UBNF 自身の VSIX・DAP）は別 PR。

---

## 1. 旧スタックの構造（段階 1）

| 拡張 | 位置 | 生成の度合い | サーバ実装 |
|---|---|---|---|
| `ubnf-vscode` | `unlaxer-parser/unlaxer-dsl/ubnf-vscode` | 生成 base + 手書き Ext | `UBNFLanguageServerExt extends UBNFLanguageServer`(生成) |
| `tinycalc-vscode` | `unlaxer-parser/unlaxer-dsl/tinycalc-vscode` | 生成 base のみ | `TinyCalcLanguageServer`（生成） |
| `calculator-lsp-vscode` | `unlaxer-parser/unlaxer-common/examples/calculator-lsp-vscode` | **完全手書き**（生成器非依存の旧例） | `CalculatorLanguageServer` |
| `tinyexpression-p4-lsp-vscode` | `tinyexpression/tools/` | 生成 base + 3,000 行超の手書き Ext | `TinyExpressionP4LanguageServerExt`（**parity の基準**） |

生成器は `org.unlaxer.dsl.CodegenMain --generators Parser,AST,Mapper,Evaluator,LSP,Launcher[,DAP,DAPLauncher]`。
`LSPGenerator`（91 行）が抽象クラスの骨、`LSPServerEmitter`（828 行）が initialize / parseDocument /
publishDiagnostics / hover / completion / semanticTokens の中身、`LSPLauncherGenerator` が stdio 入口を出す。
生成器が読むのは **UBNF の AST**（`UBNFAST.GrammarDecl`）そのもので、キーワードは文法中の
`'literal'` 終端を全走査して集める。`@catalog` / `@declares` / `@backref` / `@scopeTree` の有無で
生成内容が分岐する。生成された抽象クラスは `additionalDiagnostics` / `additionalCompletionItems` /
`customDefinition` / `additionalSemanticTokenData` / `configureAdditionalCapabilities` という
protected フックを持ち、**IDE 機能の大半はそのフックを override した手書き**にある。

lsp4j は 4 拡張すべて **0.23.1**。`vscode-languageclient` は `^9.0.1`、
`@vscode/vsce` は p4 が `^3.9.2`、他は `^2.26.0`。

## 2. capability × 旧スタック × ubnfc の材料（段階 1 の本表）

「旧: どう作っているか」は parity 基準の `tinyexpression-p4-lsp-vscode` を主に、
`ubnf-vscode` が別方式のときだけ併記する。

| capability | 旧: 提供 | 旧: どの API / 注釈で作っているか | ubnfc: 今ある材料 | ubnfc: 足りないもの |
|---|---|---|---|---|
| **diagnostics（構文）** | ○ | `org.unlaxer.context.ParseFailureDiagnostics.getFarthestOffset()` + `expectedTokens` / `expectedHintCandidates` / `deepestMatchedRule`。メッセージは Java 内の固定表 `ERROR_CATALOG`（`TE001`–`TE025` + `ULX-PARSE-001`）から選ぶ | `ParseResult.diagnostics()`（`kind` / 主 `offset` / `farthestCp` / `expected` / `farthestExpected` / `deepestRule` / `ruleStack`、すべて code point）。IR の `semantics.diagnostics.expectedDisplay` が Java 用の表示ラベル 390 件 | **実装済み（段階 2）**。エラー分類表（TE0xx 相当）は無い ＝ 文言は expected ラベルの列挙になる |
| **diagnostics（未定義参照）** | ○ | `ScopeStore.getDiagnostics(ctx)`（`@declares` / `@backref` 由来） | `ParseResult.scope().diagnostics()`（生成 parser が `use` 効果で `未定義のシンボル: 'x'` を WARNING で積む）+ `declarations()` / `references()` / `events()` | **実装済み（段階 2）**。ただし**解析が失敗した文書では scope 観測が空**（失敗時に ScopeStore が巻き戻る）。旧は前半の宣言・参照を保って報告する → 段階 3 で `@recover` か「最良前方一致の観測」が要る |
| **diagnostics（カタログ）** | ○ (`TE022`) | `@catalog` → 生成される `CatalogResolver` / `FileCatalogResolver` が `.tecatalog` を読む | IR `semantics.catalogs`（P4 は `variable` 文脈 2 site）+ `ParseResult.catalogs()`（capture を文脈別に束ねた表） | **外部カタログファイルを読む層が無い**。`.tecatalog` 相当の設定と照合を段階 3 で足す |
| **diagnostics（型検査）** | ○ (`TE025`) | 手書き `P4StrictMatchTypingValidator`（AST 走査） | IR `semantics.typeofConstraints`（`@typeof`）。P4 では未使用 | 文法外の意味検査。生成の対象外 — 拡張側フックで足す設計が要る |
| **completion** | ○ | P4 キーワード定数 + `ScopeStore.SymbolInfo` + `CatalogResolver` | カーソル位置の `expected` 集合（診断モードで前方解析すれば取れる）、`scope().declarations()`、IR の literal 一覧 | **カーソル位置での前方解析 API が無い**（`ParseOptions.PREFIX` はあるが「この位置で何が来られるか」を返す形ではない）。段階 3 |
| **hover** | ○ | AST ノード型の `instanceof` 判定 + `ScopeStore` 宣言 + カタログ説明文 | `ParseResult.nodeSpans()`（AST ノード → span）、`astTypes`、`scope().resolved()` | **`@doc` が codegen に届かない**（`07-annotation-inventory.md`: front は IR に載せるが backend が読まない）。説明文の出所が無い。段階 3 |
| **definition** | ○ | `ScopeStore` 宣言位置 | `scope().declarations()`（名前 + `offsetCp`）と `references()`（名前 + offset + len） | 位置 → 記号の逆引き索引（`Positions` はできたので索引だけ）。段階 3 |
| **references** | ○ | `ScopeStore.getAllReferences` | 同上 `references()` | 同上。段階 3 |
| **rename** | ○ | 同上 + テキスト置換 | 同上 | 同上。段階 3 |
| **semantic tokens** | ○（P4 は `instanceof` 型安全分類、ubnf-vscode は行単位の正規表現） | AST ノード型 / 正規表現 | `ParseResult.lexical()`（token ごとの span + rule + 文字列）を `ParseOptions.lexical=true` で取れる | 種別 → LSP token type の対応表。**材料は揃っている**。段階 3 |
| **document symbols** | ○ | `ScopeStore.SymbolInfo` | `scope().declarations()` + `scope().events()`（enter/leave で入れ子が作れる） | 階層の組み立て。段階 3 |
| **folding** | ○ | テキスト行走査（`{`/`}`） | `nodeSpans()` / `lexical()` | 段階 3（AST ベースなら旧より良くなる） |
| **formatting** | ○（P4 のみ） | 手書き | — | 文法から整形規則は出ない。対象外 |
| **code actions** | ○ | `ERROR_CATALOG` の `fix` 文字列 + コード別分岐 | — | エラー分類表が前提。段階 3 以降 |
| **signature help** | ○ | 固定シグネチャ文字列 | IR `semantics.rules` の `mappingParams` | 段階 3 以降 |
| **inlay hints** | ○（P4 のみ） | AST 型 | `astTypes` の `valueType` | 段階 3 以降 |
| **TextMate 文法** | ○（両方に `syntaxes/*.tmLanguage.json`） | `org.unlaxer.dsl.init.TmLanguageEmitter` が `.ubnf` の literal から**雛形を 1 度だけ**出す（`unlaxer-dsl init`）。通常ビルド（`--generators Parser,AST,...`）では再生成されず、以後は**手書きで維持** | IR の literal 一覧（`expectedDisplay` 390 件が実質そのもの）、token の `TokenSpec` | 生成器が無い。**段階 3 の項目**（雛形だけなら IR から機械的に出せる） |
| **incremental parsing** | ✕（旧も全文再解析） | — | — | 無い。D-060 で「持たない」と決めた（§6 に実測費用） |

### 2.1 位置変換 — 旧実装の UTF-16 / code point 取り違え

unlaxer のランタイムは `String.codePoints().toArray()` で走査するので、
`ParseFailureDiagnostics.getFarthestOffset()` も `ScopeStore` の位置も **code point 単位**。
LSP の `Position.character` は **UTF-16 code unit**。

- 生成側は直している: `LSPServerEmitter.java:349-357` の `codePointOffsetToStringOffset()`
  （「The parser runtime counts Unicode code points, while LSP positions use UTF-16 code units.」）
- **手書き Ext 側で再発している**: `TinyExpressionP4LanguageServerExt.java:631` が
  `ctxDiag.getFarthestOffset()` を無変換で包み、`757` / `762` / `765-766` で
  `content.charAt` / `content.substring` / `offsetToPositionWithOffset`（UTF-16 前提）に渡す。
  同ファイル `851-859` には正しい `codePointOffsetToRangeWithOffset()` があるのに、構文エラー経路だけ通っていない。

**実測で確認した**（§5 の対照実験）。BMP 外文字を 1 個含む複製と含まない複製を並べると、
旧サーバの column が**ちょうど 1 ずれる**。新実装は `Positions`（§4）1 箇所だけで変換し、
BMP 外を含む fixture でも位置が動かない。

## 3. VSIX に何が入るか

| 中身 | 旧 (`tinyexpression-p4-lsp-vscode`) | 新 (`examples/p4-vscode`) |
|---|---|---|
| クライアント | `src/extension.ts` → esbuild で `out/extension.js` | `src/extension.ts` → `tsc` で `out/extension.js` |
| サーバ起動 | `java --enable-preview -jar server-dist/tinyexpression-p4-lsp-server.jar`（+ `-Dtinyexpressionp4.catalog.path`） | `java -jar server-dist/ubnfc-p4-lsp-server.jar` |
| `contributes.languages` | id `tinyexpressionP4`、`.tinyexp` / `.formulainfo` ほか | id `tinyexpression-p4`、`.tiny` |
| `contributes.grammars` | `syntaxes/tinyexpression.tmLanguage.json`（手書き、Java ブロック埋め込み付き） | **無し（段階 3）** |
| `contributes.debuggers` | あり（DAP を同梱） | 無し（段階 4） |
| その他 | `configuration`、`walkthroughs`、`commands`、`semanticTokenScopes`、`config/*.tecatalog` | `configuration`（java パス / jar パス / JVM 引数）、`language-configuration.json` |
| 大きさ | — | 208 ファイル / 1.5 MB（jar 1.36 MB） |

---

## 4. 段階 2 で入れたもの

### 4.1 opt-in の形（既定 off）

- 集約 `pom.xml` の **Maven profile `lsp`** にだけ `ubnfc-lsp` module が入る。
  プロファイル無しのビルドは lsp4j を 1 つも引かない。
- 生成器は `ubnfc-java`（パーサ）と `ubnfc-lsp`（LSP 包み）に分かれ、
  **`ubnfc-java` のソースには一切触れていない**。`ubnfc-lsp` の CLI は
  `--emit-lsp` を明示しない限り何も出さない。`--emit-dap` は「未実装（段階 4）」で落ちる。
- `ParserUnchangedTest` が機械で押さえる: (a) 同じ出力先に LSP を重ねてもパーサ側のファイルは
  1 byte も変わらない、(b) 追加されるのは `org/ubnfc/p4/lsp/` 配下だけ、
  (c) 生成パーサのソースに `lsp4j` / `org.eclipse.` が 1 度も出ない、
  (d) `--emit-lsp` なしは失敗する。

### 4.2 生成される LSP サーバ（5 ファイル）

| ファイル | 役目 |
|---|---|
| `Positions.java` | code point ↔ (行, UTF-16 column)。行頭を **char index と code point index の両方**で持ち、行内は `Character.charCount` で数える。改行は `\n` / `\r\n` / `\r` |
| `Diagnostics.java` | `ParseResult` → LSP `Diagnostic`。構文エラー（主位置・expected ラベル・最深規則を `code` と `relatedInformation` に）と scope 診断（`未定義のシンボル: 'x'`、WARNING） |
| `DocumentStore.java` | full sync の文書保持 |
| `LanguageServerImpl.java` | `initialize`（`TextDocumentSyncKind.Full` のみ）/ `didOpen` / `didChange` / `didClose`。解析は **D-023 の認識モード**（`occurrences=false, diagnostics=false`）で、失敗したときだけ生成パーサ自身が診断モードで決定的に再解析する |
| `ServerMain.java` | stdio の `LSPLauncher`。`System.out` を stderr へ差し替えて JSON-RPC を汚さない |

extern token scanner（P4 の `STRING` / `CODE_START` / `CODE_END`）は
`--scanner-class` で与えたクラスを反射で読む。生成パーサの型でしか表現できないので名前で結ぶ。

### 4.3 ビルド

`scripts/build-vsix.sh` が 6 段階を通す:
生成器を組む（`-Plsp`）→ パーサ生成 → `--emit-lsp` で LSP 生成 → extern scanner を
`examples/p4-java` から複製（読むだけ）→ `mvn package`（golden テスト込み、shade で fat jar）→
`npm ci` + `vsce package`。出力は `examples/p4-vscode/target/ubnfc-tinyexpression-p4-0.1.0.vsix`。
`code` がこのマシンの PATH に無いので `code --install-extension` は未確認。代わりに
`unzip -l` で構造を確かめ（`extension/out/extension.js`・`extension/server-dist/ubnfc-p4-lsp-server.jar`・
`extension/package.json` が入っている）、台本 LSP クライアント（§5）でサーバを実際に動かした。

---

## 5. パリティ（旧 `tinyexpression-p4-lsp-vscode` との突き合わせ）

同じ台本クライアント（`examples/p4-vscode/scripts/lsp-probe.py`、JSON-RPC over stdio:
initialize → didOpen → publishDiagnostics）で両サーバを駆動した。旧サーバは
`tinyexpression/tools/tinyexpression-p4-lsp-vscode/server-dist/tinyexpression-p4-lsp-server.jar`
（既存のビルド成果物）をコピーして使い、兄弟リポジトリには一切書いていない。
入力は `examples/p4-java/src/test/resources/fixtures` の **16 fixture**（うち負例 3 件:
`invalid-syntax` / `complex-half` / `complex-tail`、`comparison-heavy*` 4 件も `Formula` 入口では失敗する）。
生ログは `examples/p4-vscode/evidence/parity-{old,new}.json`。

| fixture | 旧（重複除去後） | 新 | 開始位置一致 | 件数（旧生 / 旧重複除去 / 新） |
|---|---|---|---|---|
| comparison-heavy.tiny | E `[0:5–0:178]` | E `[0:5–0:5]` | ○ | 1 / 1 / 1 |
| comparison-heavy-x4.tiny | E `[0:5–0:715]` | E `[0:5–0:5]` | ○ | 1 / 1 / 1 |
| comparison-heavy-x16.tiny | E `[0:5–0:2863]` | E `[0:5–0:5]` | ○ | 1 / 1 / 1 |
| comparison-heavy-x64.tiny | E `[0:5–0:11455]` | E `[0:5–0:5]` | ○ | 1 / 1 / 1 |
| complex.tiny | W `[4:37–4:42]` | W `[4:38–4:43]` | **×（旧が 1 ずれ）** | 2 / 1 / 1 |
| complex-x4.tiny | W ×4 | W ×4 | ○（全 4 件一致） | 8 / 4 / 4 |
| complex-x16.tiny | W ×16 | W ×16 | ○（全 16 件一致） | 32 / 16 / 16 |
| complex-x64.tiny | W ×64 | W ×64 | ○（全 64 件一致） | 128 / 64 / 64 |
| complex-half.tiny | E `[4:26–4:27]` | E `[4:27–4:27]` | **×（旧が 1 ずれ）** | 1 / 1 / 1 |
| complex-tail.tiny | W `[4:37–4:42]`, E `[14:0–14:2]` | E `[14:1–14:1]` | **×（旧が 1 ずれ＋ scope 差）** | 3 / 2 / 1 |
| invalid-syntax.tiny | E `[1:0–1:0]` | E `[1:0–1:0]` | ○ | 1 / 1 / 1 |
| flat-arithmetic.tiny | （なし） | （なし） | ○ | 0 / 0 / 0 |
| large-match.tiny | （なし） | （なし） | ○ | 0 / 0 / 0 |
| valid-basic.tiny | （なし） | （なし） | ○ | 0 / 0 / 0 |
| valid-multiline.tiny | （なし） | （なし） | ○ | 0 / 0 / 0 |
| valid-unicode.tiny | （なし） | （なし） | ○ | 0 / 0 / 0 |

### 5.1 差の内訳（全件に理由がある）

1. **開始 column が 1 ずれる 3 件（complex / complex-half / complex-tail）＝ 旧の UTF-16 取り違え。**
   この 3 件だけが BMP 外文字（`@bench(name='複雑😀')` の U+1F600）を 1 個含む。
   対照実験（`examples/p4-vscode/evidence` の元データと同じ台本、scratch 上で実施）:

   | 入力 | 旧 | 新 |
   |---|---|---|
   | `complex.tiny` の 😀 を `X` に置換 | `[4:38–4:43]` | `[4:38–4:43]` |
   | `complex.tiny` のまま（😀 あり） | `[4:37–4:42]` | `[4:38–4:43]` |

   本文上 `bonus` は行頭から UTF-16 で 38–43。**新が正しく、旧は BMP 外 1 文字につき 1 ずれる**。
   BMP 外を含まない `complex-x4/x16/x64`（84 件の scope 警告）は**全件で位置が一致**する。
   これが §2.1 の `TinyExpressionP4LanguageServerExt.java:631` の再発を実測で捉えたもの。
2. **範囲の終端。** 旧は構文エラーを**行末まで伸ばす**（`while (content.charAt(lineEnd) != '\n') lineEnd++`。
   x64 では 11,455 文字の波線になる）。新は主位置の**幅 0**（`length` は診断が持つ値、P4 では常に 0）。
   VS Code は幅 0 の range を語単位に広げて描くので、実用上は新のほうが狭くて読める。意図的な差。
3. **旧は scope 診断を 2 回出す**（全 fixture で正確に 2 倍）。新は 1 回。旧の重複は取り除いて比較した。
4. **`complex-tail.tiny` で新に scope 警告が出ない。** 解析が失敗した文書では ubnfc の
   `ScopeStore` が巻き戻り、`declarations` / `references` / `diagnostics` が**すべて空**になる
   （実測: `refs=0 decls=0`）。旧は前半の観測を保つ。**段階 3 の課題**（`@recover` を使うか、
   「最良前方一致時点の観測」を結果に載せるか）。この 1 件が唯一の「新が出せていない診断」。
5. **文言・`source` / `code`。** 旧は `TE001`–`TE025` の固定表（`[TE020] 構文エラーです。 修正例: …`）、
   `source` は `tinyexpression-p4` / `tinyexpression-p4-scope`。新は expected ラベルの列挙
   （`構文エラー: ' ' | '$' | '(' | … ほか 36 件 を期待しました`）、`source` は `ubnfc`、
   `code` は最深規則名（`SinFunction` 等）か `scope`。**文言の差は想定どおり**で、
   エラー分類表を持つかどうかは段階 3 の設計判断。

---

## 6. 打鍵ごとの全文再解析の費用（増分解析を持たない根拠）

`LanguageServerImpl.analyze()`（解析 + 診断写像 + `Positions` 構築）を同一プロセスで測った。
AMD Ryzen 9 7950X / WSL2、Oracle JDK 21.0.9、`taskset -c 28`、`-Xms512m -Xmx2g -Xss16m`、
500 回 warmup → 401 回計測。**load average 8〜10**（他エージェントのベンチが走っている）ため、
min が信頼できる下限、p90 は騒音込みの上限として読む。

| fixture | code point | 診断件数 | min | p50 | p90 |
|---|---|---|---|---|---|
| valid-basic.tiny | 6 | 0 | 0.007 ms | 0.009 ms | 0.013 ms |
| valid-multiline.tiny | 40 | 0 | 0.010 ms | 0.011 ms | 0.016 ms |
| invalid-syntax.tiny | 4 | 1 | 0.031 ms | 0.037 ms | 0.055 ms |
| complex-half.tiny | 162 | 1 | 0.342 ms | 0.419 ms | 3.022 ms |
| complex.tiny | 325 | 1 | 0.344 ms | 0.452 ms | 1.657 ms |
| large-match.tiny | 1,380 | 0 | 0.762 ms | 0.964 ms | 1.454 ms |
| complex-x4.tiny | 1,293 | 4 | 0.677 ms | 0.789 ms | 0.948 ms |
| complex-x16.tiny | 5,179 | 16 | 2.514 ms | 2.856 ms | 4.034 ms |
| complex-x64.tiny | 20,923 | 64 | 11.249 ms | 15.583 ms | 23.084 ms |

`docs/reports/2026-09-24-java-two-mode.md` の認識専用の実測（complex 0.213 ms、x64 13.71 ms）と
同じ桁で、差は `Positions` 構築と LSP オブジェクト生成ぶん。

**判断**: 実用的な P4 の式（数百 code point）では打鍵ごとに **0.5 ms 未満**、5,000 code point でも
**3 ms 程度**で、60 fps の 16 ms 予算に十分収まる。20,000 code point（複製 fixture であり実在の式ではない）で
ようやく 12〜16 ms に届く。**増分解析は入れない**（D-060）。入れるとしても、その前に
「全文再解析で困る大きさの実文書がある」という証拠が要る。

---

## 7. 自動テスト

| テスト | 位置 | 何を押さえるか |
|---|---|---|
| `ParserUnchangedTest` | `ubnfc-lsp/src/test/java/org/ubnfc/lsp/` | feature off で何も変わらない（§4.1 の 4 点）、template に未置換の placeholder が無い |
| `PositionsTest` | `examples/p4-vscode/server/src/test/java/org/ubnfc/p4/lsp/` | code point ↔ UTF-16 変換。BMP 外 2 文字・`\r\n`・単独 `\r`・全位置の往復・範囲外の丸め |
| `GoldenDiagnosticsTest` | 同上 | 台本クライアント（lsp4j launcher 両端、実際の JSON-RPC）で 4 fixture の診断 JSON を golden と突合。`non-bmp.tiny` は BMP 外 2 文字を前行に置き、旧の取り違えを再発させない |

`GoldenDiagnosticsTest` の fixture: `valid-basic`（0 件）/ `invalid-syntax`（構文 1 件）/
`undefined-ref`（scope 1 件）/ `non-bmp`（BMP 外を挟んだ scope 1 件、位置は `undefined-ref` と同一）。

## 8. 残り（段階 3 / 4）

- 失敗した解析でも scope 観測を残す（§5.1-4）。これが無いと definition / references が
  「書きかけの文書」で効かない
- `@doc` を codegen に通す（hover の説明文）
- カーソル位置の expected 集合を返す API（completion）
- `semantics.catalogs` と外部カタログファイルの照合（`TE022` 相当）
- TextMate 文法の生成（`expectedDisplay` の literal から雛形は機械的に出せる）
- UBNF 自身（`ubnf.ubnf`）の VSIX、DAP（`--emit-dap`）
