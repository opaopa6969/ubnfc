# LSP capability の追加と UBNF 自身の VSIX — 段階 3

issue #28 の段階 3（capability 追加）と段階 4 の前半（UBNF 自身の VSIX）。
段階 1（調査）と段階 2（診断だけの縦切り）は
[`2026-09-24-lsp-vsix-stage1.md`](2026-09-24-lsp-vsix-stage1.md)（D-060）。
DAP（`--emit-dap`）と増分解析は入っていない。

**入ったもの**: definition / references / document symbols / semantic tokens / completion / hover /
TextMate 文法の生成 / 解析失敗時の scope 診断 / UBNF 自身の VSIX。
**opt-in は段階 2 のまま**（`-Plsp` と `--emit-lsp`。`ParserUnchangedTest` がパーサの byte 不変を押さえる）。

---

## 1. capability 表（段階 1 §2 の更新）

「旧」は parity の基準 `tinyexpression/tools/tinyexpression-p4-lsp-vscode`（手書き `*Ext` 3,000 行超）。

| capability | 旧 | 段階 2 | **段階 3** | 何から作っているか |
|---|---|---|---|---|
| diagnostics（構文） | ○ | ○ | ○ | `ParseResult.diagnostics()` の主位置・`expected`・`deepestRule`。`trailing_input` だけ**最遠位置**へ寄せた（§4.1） |
| diagnostics（未定義参照） | ○ | ○ | ○ | `scope().diagnostics()` |
| **diagnostics（解析失敗時の scope）** | ○ | **✕** | **○** | 主失敗位置までを別入力として再解析（D-061、§3）。`code` は `scope-prefix` |
| **definition** | ○ | ✕ | **○** | `scope().declarations()`。scope 効果が記号表を作らない文法（UBNF の `sameRuleText`）は `lexical()` × IR の capture site（§5.1） |
| **references** | ○ | ✕ | **○** | `scope().references()` / 同上 |
| **document symbols** | ○ | ✕ | **○** | `scope().events()` の enter / leave / declare を事象順に積む |
| **semantic tokens** | ○ | ✕ | **○** | `lexical()` の終端 × 生成時に畳み込んだ式 → 種別表（legend 5 種） |
| **completion** | ○ | ✕ | **○** | カーソル直前の識別子を落とした前半を解析し、`farthestExpected` を候補にする（§2）＋ scope 記号 |
| **hover** | ○ | ✕ | **○** | `@doc` →`@declares(description)` → 規則名・宣言位置（§6） |
| **TextMate 文法** | ○（`init` の雛形 + 手書き維持） | ✕ | **○（毎回生成）** | literal / trivia 区切り / token 種別（§7） |
| rename | ○ | ✕ | ✕ | definition / references と同じ索引で出せる。次段 |
| folding / inlay hints / signature help / formatting / code actions | ○ | ✕ | ✕ | 文法から機械的に出ないものを含む。段階 4 以降 |
| diagnostics（カタログ `.tecatalog`） | ○ | ✕ | ✕ | 外部ファイルを読む層が要る |
| 文法の意味検査（`GrammarValidator` 相当） | ○（ubnf-vscode） | ✕ | ✕ | ubnfc では front（`ubnfc validate`）が持つ。LSP には載せていない（§5.2） |
| DAP | ○ | ✕ | ✕ | 段階 4。`--emit-dap` は「未実装」で落ちる |
| incremental parsing | ✕ | ✕ | ✕ | 入れない（D-060。費用は §8） |

生成される LSP サーバは 5 → **11 ファイル**（`Positions` / `Meta` / `Analysis` / `Symbols` /
`Diagnostics` / `DocumentStore` / `Completion` / `Hovers` / `SemanticTokens` /
`LanguageServerImpl` / `ServerMain`）。

`Meta.java` は**文法ごとの表を生成時に畳み込んだもの**（規則 → `@doc` / `@declares` の説明、
式 → semantic token 種別、宣言・参照を生む capture site、keyword、trivia 区切り）。
実行時に IR を読まないので VSIX に IR を同梱しなくてよい。

---

## 2. completion — 「この位置で何が来られるか」をパーサの契約を変えずに得る

生成パーサに「位置 → 期待集合」API は無い。足したのは**呼び方**だけで、パーサは 1 行も変えていない。

1. カーソルから左へ識別子を遡り、その開始位置 `s` を求める（`va|` なら `va` の頭）
2. `text[0, s)` を**1 つの文書として**解析する（全入力要求つき、診断観測を明示要求）
3. 主 `expected` ではなく **`farthestExpected`**（最遠到達位置で期待していた集合）を候補にする
4. `'…'` の literal ラベルだけを候補にし、trivia の区切り（`//` `/*` `*/`）を外す
5. カーソルより前で宣言済みの記号（`declarations()`、UBNF は capture site 由来）を足す
6. カーソル下の綴りで前方一致（大小無視）で絞る

**`farthestExpected` を採る理由**: 入口が `UBNFFile ::= { GrammarDecl }` のような 0 回以上の
繰返しだと、前半は「0 件で成功 → 末尾入力」になり、主 `expected` が `end of input` だけになる。
最遠到達位置の期待集合はそのとき正しく「次に書けるもの」を指す。

**診断観測を明示要求する理由**: D-023 のとおり、認識専用で解析すると**成功した解析の hints は空**になる。
補完が欲しいのはまさにその hints なので、この 1 呼出しだけ `diagnostics=true` を渡す
（打鍵ごとの診断経路は段階 2 のまま認識モード）。

実測（golden より）。`partial.tiny` の `$pri` の右で 1 件（`price`）。
`declarations.tiny` の本体行頭で 40 件（keyword 33 + 宣言済み記号 3 + 演算子）。
UBNF の `refs.ubnf` で規則名 `Start` / `Item` が候補に出る。

---

## 3. 解析が失敗した文書の scope 診断（段階 2 の唯一のパリティ欠落）

### 3.1 契約の検討

生成パーサの `ScopeStore` は失敗した枝を巻き戻す（`restore(checkpoint)`）。
入口規則が失敗した文書では `declarations()` / `references()` / `diagnostics()` が**全部空**になる
（段階 1 §5.1-4、`complex-tail.tiny` で実測 `refs=0 decls=0`）。

「巻き戻さずに観測を残す」案は採らなかった。`docs/design/04-cursor-and-diagnostics-contract.md` §1 の
rollback は cursor と意味的副作用の両方を戻すことになっており、scope 観測は
**corpus が突き合わせる観測値**（`declarations` / `references` / `scope` / `scopeDepth` / `rollback`、
harness の driver protocol）そのものである。残す側に倒すと corpus の期待値が動く。
D-025 の「scope mode は宣言どおり」も崩れる。**観測契約を変えずに済む案**を採った。

### 3.2 採った形（D-061）

全文解析が失敗したら、**主失敗位置で入力を切り、その前半を別の文書として解析し直す**
（`requireFullInput=false`）。成功したらその観測を scope 診断・definition / references /
document symbols / semantic tokens の出所にする。解析は決定的なので、この観測は
「その前半だけを書いた文書」の観測と完全に一致する。診断には `code: scope-prefix` を付け、
文面に「構文エラーより前の範囲だけを見た結果」と書く。

- 余計な解析は**失敗したときだけ 1 回**。通る文書の打鍵ごとの費用は段階 2 と同じ
- 前半が解析できないとき（`complex-half.tiny` / `invalid-syntax.tiny`）は何も足さない
- パーサの API も観測契約も変えていない

### 3.3 パリティ（`complex-tail.tiny`）

| | 旧（重複除去後） | 段階 2 | **段階 3** |
|---|---|---|---|
| 構文エラー | E `[14:0–14:2]` | E `[14:1–14:1]` | E `[14:1–14:1]` |
| scope 警告 | W `[4:37–4:42]` | **なし** | **W `[4:38–4:43]`** |

`[4:37]` と `[4:38]` の差は旧の UTF-16 取り違え（段階 1 §5.1-1 の対照実験と同じ原因。
この fixture だけが BMP 外文字 U+1F600 を 1 個含む）。**新が正しい**。
旧は同じ警告を 2 回出すので上表は重複除去後。

### 3.4 16 fixture の回帰（段階 2 → 段階 3）

`scripts/lsp-probe.py` で同じ 16 fixture を通し、段階 2 の
`examples/p4-vscode/evidence/parity-new.json` と 1 件ずつ比べた。

**差は `complex-tail.tiny` に scope 警告が 1 件増えただけ**。
他の 15 fixture は診断の件数・位置・severity・code・文面まで完全に一致する。

---

## 4. 診断の位置の細かい変更

### 4.1 `trailing_input` は最遠位置で報せる

`ParseResult` の `trailing_input` の主位置は「採用した prefix の消費末尾」（D-024）。
編集者に見せたいのはそこではなく**どこで話が合わなくなったか**なので、
**LSP の診断の位置だけ** `farthestCp` へ寄せ、文面にも `farthestExpected` を並べる。
`ParseResult` は変えていない（`Diagnostics.java` の中だけの写像）。

効果は UBNF で大きい（入口が 0 回以上の繰返しなので、寄せないと全部 `0:0` になる）。
P4 の 16 fixture に `trailing_input` は 1 件も出ないので、§3.4 の比較には影響しない。

---

## 5. UBNF 自身の VSIX（`examples/ubnf-vscode`）

`ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json` から P4 と**同じ経路**で組む。
extern token を使わないので scanner の複製は要らない。言語 id `ubnf`、拡張子 `.ubnf`。

```sh
scripts/build-vsix.sh ubnf     # p4 / ubnf / all（既定 all）
```

### 5.1 UBNF の宣言・参照は `ScopeStore` に載らない

`ubnf.ubnf` は `@declares(symbol=name)`（`RuleDecl`）と `@backref(name)`（`QuantifiedRef`）を
**`sameRuleText` モード**で使う。このモードは「同じ規則の中で綴りが一致するか」しか見ないので、
`ScopeStore` には宣言が積まれない（実測 `decls=0 refs=0`）。

そこで、**IR の scope 効果が読む capture site** と `lexical()` の `exprId` を突き合わせる経路を足した。
`captureSiteId` の末尾 `/capture/N` を落とすと `lexical()` の `exprId` と一致するので、
「宣言を生む終端」「参照を生む終端」がそのまま取れる。
`ubnf.ubnf` 自身で **59 宣言 / 110 参照**。`scope().declarations()` が空のときだけこちらに落ちる。

### 5.2 旧 `unlaxer-dsl/ubnf-vscode` との突き合わせ（24 文法）

資材は `crates/ubnfc-front/tests/pipeline_golden.rs` の `FILES`（24 個の `.ubnf`）。
旧サーバは `unlaxer-dsl/ubnf-vscode/server-dist/ubnf-lsp-server.jar`（既存のビルド成果物）を
scratch へ複製して動かした（兄弟リポジトリには書いていない）。
再現は `scripts/ubnf-parity.sh`。生ログは `examples/ubnf-vscode/evidence/parity-{old,new}.json`。

| 文法 | 旧 構文 | 旧 意味検査 | 新 | 旧 構文の位置 | 新の位置 |
|---|---|---|---|---|---|
| `rust/known/common-field-space.ubnf` | 0 | 1 | 0 | | |
| `rust/known/eval-default.ubnf` | 0 | 1 | 1 | | 0:32 |
| `rust/known/keyword-boundary.ubnf` | 1 | 0 | 1 | 0:24 | **0:24** |
| `rust/known/namespace.ubnf` | 0 | 2 | 0 | | |
| `rust/positive/all-tokens.ubnf` | 0 | 6 | 0 | | |
| `rust/positive/annotations.ubnf` | 0 | 11 | 1 | | 1:21 |
| `rust/positive/boundaries.ubnf` | 0 | 2 | 0 | | |
| `rust/positive/elements.ubnf` | 1 | 0 | 0 | 3:8 | |
| `story/calc-leftrec.ubnf` | 0 | 2 | 0 | | |
| `story/calc-var.ubnf` | 0 | 0 | 0 | | |
| `story/calc.ubnf` | 0 | 0 | 0 | | |
| `dsl/examples/tinycalc.ubnf` | 1 | 0 | 1 | 8:20 | **8:20** |
| `dsl/grammar/ubnf.ubnf` | 0 | 0 | 0 | | |
| `dsl/associative/Operators.ubnf` | 0 | 1 | 0 | | |
| `dsl/cardinality/Cardinality.ubnf` | 0 | 1 | 0 | | |
| `dsl/evolution/{0,1,2,3}/Evolution.ubnf` | 0 | 1 ×4 | 0 | | |
| `dsl/mixed-values/Delimited.ubnf` | 0 | 1 | 0 | | |
| `dsl/mixed-values/Mixed.ubnf` | 0 | 1 | 0 | | |
| `dsl/mixed-values/Shared.ubnf` | 0 | 0 | 0 | | |
| `dsl/right-associative/Power.ubnf` | 0 | 1 | 0 | | |
| `dsl/tinycalc-vscode/grammar/tinycalc.ubnf` | 0 | 0 | 0 | | |
| **合計** | **3** | **34** | **4** | | |

**差の理由（全件）**

1. **旧の 34 件は文法の意味検査**（`W-TOKEN-UNRESOLVED` 9 / `E-RULE-UNDEFINED` 8 /
   `W-GENERAL-NO-ROOT` 4 / `E-ASSOC-*` 8 / `W-LEFT-RECURSION` 2 / ほか 3）で、
   旧 `UBNFLanguageServerExt` が `GrammarValidator` を呼んで出しているもの。**構文解析の結果ではない**。
   ubnfc では同じ検査を front が持つ（`ubnfc validate`。`crates/ubnfc-front/src/validate/`）が、
   **LSP サーバには載せていない**。載せるには「生成パーサを包む」以上のもの（front を JVM から呼ぶか、
   検査結果を IR から引く層）が要るので、段階 4 以降の課題として切り出す。
2. **両方が構文エラーを出す 2 件は開始位置が一致する**（`keyword-boundary` `0:24`、`tinycalc` `8:20`）。
   期待ラベルも一致し、新はそれに block コメントの区切り（`/*` `*/`）が加わるだけ。
   範囲の終端は旧が 1 文字幅、新は幅 0（段階 1 §5.1-2 と同じ意図的な差）。
3. **`elements.ubnf`（旧だけ）と `annotations.ubnf` / `eval-default.ubnf`（新だけ）は
   「`ubnf.ubnf` が書いている文法」と「旧の手書き UBNF パーサが受理する範囲」の差**。
   新は `ubnf.ubnf` から**機械的に**生成した解析器なので、文法どおりに振る舞う。
   - `elements.ubnf` `Item+` / `Item{2,}` / `Item % ','` — `ubnf.ubnf` は受理し、旧は落ちる
   - `annotations.ubnf` `@mapping(Node.Item, …)` の**ドット付きクラス名** — 旧は受理し、`ubnf.ubnf` は書いていない
   - `eval-default.ubnf` `@eval(kind='…')` の `=` 区切り — 旧は受理し、`ubnf.ubnf` は `key: value` しか書いていない

   これは self-host の既知の溝（`docs/design/05-selfhost.md`、D-012 / D-015）で、
   **LSP の実装差ではなく文法と旧実装の差**である。`ubnf.ubnf` 自身は両方とも診断 0 件。

---

## 6. hover と `@doc`

### 6.1 `@doc` は IR にもう載っている

段階 1 は「`@doc` が codegen に届かない」と書いた。`docs/design/07-annotation-inventory.md` の
「無視」は**両 backend の codegen** のことで、**front は最初から IR に載せている**
（`semantics.rules[].annotations` の `{name: "doc", arguments: {text: …}}`。
`crates/ubnfc-front/src/lower/annotations.rs:53`）。

したがって段階 3 では **IR スキーマにも front にも一切手を入れていない**。
`ir/schema/grammar-ir.v1.schema.json` は無変更、**`ir/fixtures/` の IR golden は 1 件も動いていない**。
`semantics.rules[].doc` のような欄を足すと golden 全部に `"doc": null` が増えるだけで、
得るものが無い（読める情報は同じ）。これが D-061 の 2 つ目の決定。

`ubnfc-lsp` が生成時に `annotations` を読み、規則 → 説明文の表を `Meta.RULE_DOC` として畳み込む。

### 6.2 何を出すか

| 段 | 出所 | P4 | UBNF |
|---|---|---|---|
| 1 | 規則の `@doc('…')` | 使っていない | 使っていない |
| 2 | `@declares(description='…')` | `desc`（4 規則） | 無し |
| 3 | 記号の種類（宣言した規則名）と宣言位置 | ○ | ○ |
| 4 | 終端の文字列と規則名 | ○ | ○ |

VSIX を出す 2 文法はどちらも**読み取り専用の sibling repo**にあり（`tinyexpression/tools/…/tinyexpression-p4.ubnf`、
`unlaxer-parser/unlaxer-dsl/grammar/ubnf.ubnf`）、どちらも `@doc` を使っていない。
そこで `@doc` の経路は `ubnfc-lsp/src/test/resources/doc-demo.ubnf`（+ `ubnfc ir` の実出力
`doc-demo.ir.json`）で押さえる。`MetadataTest` が
「`@doc` → IR → `LspEmitter.ruleDocs()` → 生成 `Meta.java` の中身」を検査する。
表示の経路（`Meta.documentation()` → `Hovers`）は `@declares` の説明と共通で、
P4 の hover golden がそれを通す（`**price** — 宣言 / 種類: NumberVariableDeclaration / desc`）。

---

## 7. TextMate 文法の生成

`ubnfc-lsp --emit-tmlanguage <path>` が IR から `*.tmLanguage.json` を書く。
旧スタックは `unlaxer-dsl init` の雛形として 1 度だけ出して以後は手書き維持だったが、
ここでは**毎ビルド再生成**し、`syntaxes/` を `.gitignore` に入れて
先頭に `information_for_contributors` で「生成物・手で編集しない」と書く。

| セクション | 何から | P4 | UBNF |
|---|---|---|---|
| `comments` | trivia 方針の `lineComment` / `blockComment` 区切り | `//`、`/* */` | `//`、`/* */` |
| `strings` | token の `SingleQuoted` / `DoubleQuoted` と、引用符始まりの extern | `'…'` と `"…"` | `'…'` |
| `numbers` | builtin `Number` / `Digit` / `UnsignedInteger` の有無 | ○ | ○ |
| `keywords` | 語として読める literal を `\b(?:…)\b` で 1 本に | 61 語 | 23 語 |
| `operators` | それ以外の literal（長いものから） | ○ | ○ |

意味づけ（型名・関数名・埋め込み Java）は文法から出ないので入れない。
そこは LSP の semantic tokens が担う（TextMate 文法はサーバが起動するまでの下地）。

---

## 8. 打鍵ごとの費用（失敗経路が増えたぶん）

`LanguageServerImpl.analyze()`（`Analysis.of` + 診断写像）。Ryzen 9 7950X / WSL2、
JDK 21.0.9、`taskset -c 28`、`-Xms512m -Xmx2g -Xss16m`、500 warmup → 401 計測。
**他エージェントのベンチが同時に走っており load average は高い**ので min を下限として読む。

| fixture | code point | 解析 | min | p50 | p90 |
|---|---|---|---|---|---|
| valid-basic.tiny | 6 | 成功 1 回 | 0.025 ms | 0.043 ms | 0.176 ms |
| complex.tiny | 325 | 成功 1 回 | 0.288 ms | 0.964 ms | 6.414 ms |
| **complex-tail.tiny** | 326 | **失敗 →診断再解析 → 前半再解析** | **0.818 ms** | 1.269 ms | 2.915 ms |
| complex-x16.tiny | 5,179 | 成功 1 回 | 1.477 ms | 1.950 ms | 3.136 ms |
| complex-x64.tiny | 20,923 | 成功 1 回 | 6.624 ms | 12.045 ms | 16.602 ms |
| comparison-heavy-x64.tiny | 11,456 | 失敗（切る位置が 5） | 0.061 ms | 0.067 ms | 0.123 ms |

失敗した文書は「認識 → 診断モード再解析（D-023、生成パーサ内）→ 前半の再解析」で最大 3 回通る。
`complex-tail` で成功時の約 2.8 倍。**切る位置までしか読まない**ので、
壊れている場所が前のほうにある文書（`comparison-heavy-x64`）はむしろ安い。
semantic tokens 用の `lexical()` は**要求されたときだけ**解析し直す（打鍵経路には載せない）。
増分解析を入れる根拠は依然として無い（D-060）。

---

## 9. テスト

| テスト | 位置 | 何を押さえるか |
|---|---|---|
| `ParserUnchangedTest` | `ubnfc-lsp` | feature off で何も変わらない（段階 2 の 4 点）、11 テンプレートに未置換 placeholder が無い |
| `MetadataTest` | `ubnfc-lsp` | `@doc` → IR → `Meta.RULE_DOC`、semantic token 種別、UBNF の capture site、trivia 除外、TextMate 文法の各セクションと正規表現 escape |
| `PositionsTest` | 両 server | code point ↔ UTF-16。BMP 外・`\r\n`・単独 `\r`・全位置の往復・範囲外の丸め |
| `GoldenDiagnosticsTest` | 両 server | 台本クライアント（実際の JSON-RPC）で診断 JSON を golden と突合 |
| **`GoldenCapabilitiesTest`** | 両 server | document symbols・semantic tokens を丸ごと、`probes/<fixture>.txt` の位置ごとに definition / references / hover / completion を golden と突合。initialize が返す capability と legend も検査 |

golden の更新は `mvn test -Dubnfc.golden.update=true`。

fixture（P4 8 件 / UBNF 4 件）のうち UTF-16 写像を押さえるもの:

- `non-bmp.tiny` — BMP 外 2 文字を前の行に置く（段階 2 から）
- **`non-bmp-nav.tiny`** — BMP 外 2 文字の**後ろ**で記号を宣言する。
  code point を UTF-16 index として渡す実装だと definition の桁が 2 ずれる（golden は `0:34`）
- **`non-bmp.ubnf`** — 同じことを UBNF で（`Word` の宣言がブロックコメント中の BMP 外 2 文字の後ろ。golden は `2:29`）

corpus・IR golden・self-host 固定点は**触っていない**（`ubnfc-java` / `crates/**` / `runtime-template` は無変更）。

---

## 10. 残り（段階 4 以降）

- **DAP**（`--emit-dap`）
- **増分解析**（入れない判断のまま。§8）
- rename（definition / references と同じ索引で出せる）
- 文法の意味検査（`GrammarValidator` 相当）を LSP に載せる（§5.2-1）
- 外部カタログ（`.tecatalog`）の照合
- folding / inlay hints / signature help / formatting / code actions
- `code --install-extension` の実機確認（このマシンに `code` が無い）
