# クロスチェックで見つかった 3 つの欠陥の修正（2026-09-23）

対象は [`2026-09-23-parser-feature-correctness-comparison.md`](2026-09-23-parser-feature-correctness-comparison.md)
が **ubnfc 側の欠陥**として挙げた 3 類型。比較そのものはやり直していない。
コーパスと self-host 固定点は不変で、変えたのは「不正入力の扱い」「self-host の受理範囲」
「Java backend 例の scanner の value span」と、それらを固定する検証である。

| # | 欠陥 | 正本 | 変更 | 回帰 |
|---|---|---|---|---|
| 1 | 不正な token 宣言 7 件を受理、うち 1 件は panic | `specs/ubnf-syntax.md`、`02-facts` §A、参照実装 2 つ | 新 ERROR 6 種（D-050） | `tests/validate.rs` |
| 2 | 生成 frontend だけがブロックコメントを受理 | `specs/ubnf-syntax.md`「`/* */` は未対応」 | self-host に字句の門（D-051） | `tests/parity.rs` |
| 3 | Java backend だけ文字列値に引用符が残る（256 件） | `specs/generators.md` #132 | `P4Scanners.valueSpan` を修正 | `p4-rust/tests/fixtures.rs` |

---

## 1. 不正な token 宣言・注釈値（最優先。1 件は panic）

### 根本原因

`crates/ubnfc-front/src/lower/tokens.rs:81` の

```rust
T::CharRange { min, max } => TokenSpec::CharRange {
    min: min.chars().next().unwrap() as u64,
    max: max.chars().next().unwrap() as u64,
},
```

が、`CHAR_RANGE('','a')` の空境界で `None.unwrap()` になり **exit 101 で落ちていた**。
構文層は `CHAR_RANGE(STRING ',' STRING)` を通し、resolve は文字列をそのまま運び、
validate には境界の検査が 1 つも無かったため、不正値が lowering まで素通りしていた。
同じ「値を検査しない」形が 6 か所あり、いずれも unlaxer-ubnf と Java bootstrap は拒否する。

### 正本

- **CHAR_RANGE の境界**: `unlaxer-parser/unlaxer-dsl/specs/ubnf-syntax.md`
  「各境界はエスケープ展開後に1個の非 surrogate BMP 文字で、`min <= max` でなければならない。
  空文字・複数文字・surrogate・補助文字（例 `😀`）・逆順の範囲は `IllegalArgumentException` で
  **明示的に拒否**し、**先頭文字へ切り捨てない**」。
  [`../design/02-facts-tokens-bench-corpus.md`](../design/02-facts-tokens-bench-corpus.md) §A も
  「両端含む、`char` 境界（BMP 非 surrogate）」と同じことを言う。
  参照実装は `rust/unlaxer-ubnf/src/parser.rs` の `bmp()`
  （"CHAR_RANGE boundary requires exactly one non-surrogate BMP character"）と
  "CHAR_RANGE minimum exceeds maximum"。
- **32 bit の整数上限**: 仕様文には無い。正本は参照実装 2 つ — Java bootstrap の `Integer.parseInt`
  （`NumberFormatException`）と `rust/unlaxer-ubnf/src/parser.rs:102-108` の `parse::<i32>()`
  （"integer exceeds 2147483647"）。
- **recovery mode**: [`../design/03-design-v1.md`](../design/03-design-v1.md) §3.10 の表が
  `SYNC` / `AUTO` / `SKIP` の 3 つだけを定義する。`rust/unlaxer-ubnf/src/parser.rs:438-461` は
  "expected recovery mode auto, skip or sync" で拒否する。

### 修正

新しい ERROR を 6 つ足した（`crates/ubnfc-front/src/validate/`）。

| コード | カテゴリ | 対象 | 追加場所 |
|---|---|---|---|
| `E-TOKEN-CHAR-RANGE-BOUNDARY` | TOKEN（新設） | 空境界・2 文字以上の境界 | `validate/general.rs` |
| `E-TOKEN-CHAR-RANGE-SUPPLEMENTARY` | TOKEN | 非 BMP（補助文字）境界 | 同上 |
| `E-TOKEN-CHAR-RANGE-ORDER` | TOKEN | `min > max` | 同上 |
| `E-BOUNDED-OVERFLOW` | GENERAL（既存 `E-BOUNDED-INVALID` と同じ） | `{n,m}` の境界が `2147483647` 超 | 同上 |
| `E-PRECEDENCE-OVERFLOW` | PRECEDENCE | `@precedence(level=N)` が `2147483647` 超 | `validate/annotations.rs` |
| `E-ANNOTATION-RECOVERY-MODE` | ANNOTATION | `sync`/`auto`/`skip` 以外の mode（lower は黙って SYNC に落としていた） | 同上 |

`ValidationIssue::category()` に `("E-TOKEN-", "TOKEN")` を足した。既存の `W-TOKEN-*` は
`GENERAL` のままなので、`tests/fixtures/validation-java.json` の 83 ケースの category は動かない。

**両方の経路で出ることの保証**: `pipeline` は validate の ERROR で lower より前に止まり
（`pipeline.rs:170-173`）、**`lower::lower` 自身も冒頭で `validate::validate` を呼んで
ERROR があれば即 `Err`**（`lower/mod.rs:85-93`）。したがって validate に足すだけで
lowering へ到達する全経路が閉じる。回帰テストはこの 2 つ目の門も明示的に叩く。

それとは独立に、`lower/tokens.rs` の `unwrap` 自体を除いた。空境界は `E-LOWER-CHAR-RANGE`
という lower エラーにする。**利用者入力に対して panic しない**ことを二重に保証するための
防御であり、通常は validate が先に止めるので到達しない。

契約の選択（受理範囲を狭める）なので [`../design/03-design-v1.md`](../design/03-design-v1.md)
§8 に **D-050** として記録した。

### 実測（修正前 → 修正後）

`ubnfc validate --grammar X.ubnf --strict`:

| 入力 | 修正前 | 修正後 |
|---|---|---|
| `token T=CHAR_RANGE('','a')` | **panic / exit 101** | `E-TOKEN-CHAR-RANGE-BOUNDARY` / exit 1 |
| `CHAR_RANGE('ab','z')` | exit 5（先頭 1 文字を黙って採用） | `E-TOKEN-CHAR-RANGE-BOUNDARY` / exit 1 |
| `CHAR_RANGE('😀','😇')` | exit 5（診断なし） | `E-TOKEN-CHAR-RANGE-SUPPLEMENTARY` ×2 / exit 1 |
| `CHAR_RANGE('z','a')` | exit 5（診断なし） | `E-TOKEN-CHAR-RANGE-ORDER` / exit 1 |
| `'x'{2147483648}` | exit 0 | `E-BOUNDED-OVERFLOW` / exit 1 |
| `@precedence(level=2147483648)` | 別警告のみ | `E-PRECEDENCE-OVERFLOW` / exit 1 |
| `@recovery(magic)` | exit 0 | `E-ANNOTATION-RECOVERY-MODE` / exit 1 |

### 回帰テスト

`crates/ubnfc-front/tests/validate.rs`

- `crosscheck_malformed_declarations_are_rejected` — 上の 7 件（＋ `CHAR_RANGE('a','')` と
  `'x'{2147483648,}` の 2 変種）を code 単位で固定し、**`lower::lower` も同じ code で
  `Err` を返すこと**まで確認する。
- `crosscheck_wellformed_declarations_stay_accepted` — `CHAR_RANGE('a','z')`、
  `CHAR_RANGE('a','a')`、エスケープ境界 `CHAR_RANGE('\t','\n')`、`'x'{2147483647}`、
  `@recovery(sync|AUTO|skip|sync='...')` が ERROR にならないこと。

---

## 2. panic 掃き出し（`crates/ubnfc-front` 全体）

「利用者が書いた文法テキストに由来する値」に対する `unwrap()` / `expect()` / 添字 /
整数演算を全数で見て、**保証が無いものを探した**。並行して、疑わしい箇所ごとに `.ubnf` を
作って `validate` / `ir` に流す実地確認（変異ファズ 1,900 件を含む）も行った。

### 結果 1 — `unwrap()/expect()/添字` 由来の panic は **`CHAR_RANGE` の 1 件だけ**だった

他は全て、同じ関数内の直前の分岐が構造を保証している。主なものの根拠:

| 箇所 | 根拠 |
|---|---|
| `desugar.rs:282,395,442-443` | 同じ式の先頭が `elements.len() == 3` / `shape` 違反で `ok=false` → 早期 return / `primaries.is_empty()` を先に弾く |
| `analysis/operators.rs:39,52-53,68-70,78` | すべて `seq.len()==2/3`・`pair.len()==2`・`alternatives.len()==2` の直後 |
| `analysis/mod.rs:183,202,241` | Tarjan の SCC 成分は非空、`rule_sccs` は全規則の分割 |
| `analysis/graph.rs:25,31` | `index[w].is_none()` の else 枝と Tarjan の不変条件 |
| `analysis/regex.rs:202-208,357,374-381` | 長さ 1 以外は手前で `None`、`windows(2)` は常に 2、境界値は `CharSet::ranges()` 由来で ≤ 0x10FFFF |
| `analysis/facts.rs` | 溢れる算術は `saturating_add/mul`（`'ab'{u64::MAX}` でも exit 1） |
| `lower/expr.rs:170,217,434,437,444` | `seq.len()==1` の枝 / 直前に push した式 / ExprId の role は固定語のみで利用者テキストは入らない |
| `lower/recursion.rs:243,250-251` | `supported()` の分解に成功した場合だけ到達 |
| `lower/schema.rs:109,158,327,336` | `cx.types` は `@enum`/`@mapping` のみ、`@enum` は手前で `continue`、`family` は自分自身を含む |
| `validate/operators.rs:288,294` | 巡回の要素は `grammar.rules` 由来、巡回は非空 |
| `resolve/mod.rs:414,436,657-658` | 同じ `finish()` 内で直前に全規則・全 token を `register()` 済み、`row` は `b.len()+1` 要素 |
| `templates.rs:505` / `pipeline.rs:482` / `ir/references.rs:930` | それぞれ `len()==1` / `len()!=2` で internal error / `len()!=1` で return の直後 |
| `syntax/lexer.rs:89` | `offset < bytes.len()` かつ全分岐が文字境界でしか進まない |
| `cli.rs:90,94` | `Options::parse` が `gen` の必須オプション欠落を先に弾く（実測） |
| `lower/normalize.rs:49,67` の `unreachable!()` | 量指定子・区切りの 20 通りの組合せを流しても到達しない |

### 結果 2 — panic ではないが同類の「不正入力でプロセスが落ちる」を 1 件見つけて直した

**`syntax/parser.rs` の `atom()` ⇄ `template_args()` に深さ上限が無かった。**
`body()` には上限 128 の guard があるのに、パラメータ化ルールの実引数（D-043）の経路が
それを共有していなかったので、`Q<Q<…2,000 段…>>` が **stack overflow（exit 134）** になった。
`(((…20,000 段…)))` は guard が効いて exit 1 なので、`<>` の経路だけが守られていなかった。

修正は `DEPTH_LIMIT` 定数を切り出し、`template_args()` でも同じ深さを数えるだけ。
`Q` は未定義でよく `@ubnf: 2` も要らない（構文解析だけで落ちる）ので、**発見時点で
`ubnfc validate` に .ubnf を 1 本渡すだけの再現**だった。
回帰は `crates/ubnfc-front/tests/syntax.rs` の
`excessive_template_argument_nesting_reports_an_error_instead_of_overflowing_the_stack`。

### 結果 3 — 直していない 2 件（別件として記録する）

どちらも `unwrap`/添字ではなく「深さ・計算量に上限が無い」類で、直すには再帰の反復化か
明示的な上限の導入が要る。**この報告の 3 欠陥とは独立**なので手を付けず、再現手順だけ残す。

1. **規則参照グラフを辿る再帰に深さ上限が無い**（`analysis/reachability.rs` の `Reach`、
   `analysis/graph.rs` の再帰 Tarjan）。規則数ではなく**参照の深さ**が効く。
   release ビルドでの実測: 8,000 段の規則連鎖は exit 0、**16,000 段で exit 134**。
   debug ビルドでは 4,000 段、あるいは 30 規則 × 括弧 120 段（7.8 KB）で落ちる。
   lower ではなく analysis 側であること、`@root` から到達不能な連鎖でも落ちることを切り分け済み。

   ```sh
   python3 -c "
   n=16000
   print('grammar G {'); print(\"  @root R0 ::= 'a' R1;\")
   [print(f\"  R{i} ::= 'a' R{i+1};\") for i in range(1, n)]
   print(f\"  R{n} ::= 'x';\"); print('}')" > chain.ubnf
   ubnfc validate --grammar chain.ubnf   # exit 134
   ```

2. **`validate/operators.rs` の `find_cycles` が階乗時間**。visited 集合を持たず全単純経路を
   列挙するので、互いを左端参照する 10 規則・1 KB 未満の文法で固まる
   （実測 7 規則 0.03 秒、8 で 0.30 秒、9 で 4.66 秒、10 で 120 秒超）。
   同種に、雛形 `T{i}<P> ::= T{i-1}<P> T{i-1}<P>;` の指数展開（24 段・34 行で 60 秒超）がある。

---

## 3. 7 件目の self-host 乖離（ブロックコメント）

### 根本原因（既に特定済みだったものの確認）

`unlaxer-dsl/grammar/ubnf.ubnf:18` の `@whitespace: javaStyle` →
`crates/ubnfc-front/src/lower/trivia.rs:90-99` が javaStyle の区切り集合へ
`BlockComment { open: "/*", close: "*/" }` を無条件に入れる。
一方 `crates/ubnfc-front/src/syntax/lexer.rs:89` の手書き lexer は `/*` を専用メッセージで拒否する。
**自己記述文法が、それを記述している実装より広い言語を記述している。**

### どちらが正しいか

**手書き側が正しい。** `unlaxer-parser/unlaxer-dsl/specs/ubnf-syntax.md` が
「UBNF ファイル自体のコメント構文は `//` のみで、`/* ... */` は未対応です」と書き、
unlaxer-ubnf（"UBNF block comments are unsupported; use // comments"）と Java bootstrap も拒否する。
4 実装中 3 つと仕様が一致する。

### 採らなかった案（と、その理由）

| 案 | 却下理由 |
|---|---|
| (a) `ubnf.ubnf` の `@whitespace` を `none` + `@comment{line:'//'}` に直す | `unlaxer-parser` は読み取り専用。本来はこれが正しい直し方 |
| (b) javaStyle の既定からブロックコメントを外す | corpus `comment-whitespace/1` の `expected` が `'/*'` / `'*/'` を含み、旧実装の javaStyle がブロックコメントを持つことを固定している |
| (c) 明示 `@comment` を「その文法が持つコメントの**種類**の宣言」にし、未宣言の種類を style の既定から落とす | 実装して測ったところ corpus は通った（`comment-whitespace/2` の `//` は javaStyle 由来で残るため）が、**`unlaxer-dsl/examples/tinycalc.ubnf` が壊れる**。TinyCalc は `@whitespace: javaStyle` + `@comment: { line: "//" }` と書きつつ**ブロックコメントを意図**しており（`tinycalc-vscode/language-configuration.json` の `"blockComment": ["/*", "*/"]`、`crates/ubnfc-harness/tests/runner_java_e2e.rs` が入力 `/*😀*/ 1+2` を受理と固定）、この案では拒否になる。`ir/fixtures` 2 本と `crates/ubnfc-selfhost/src/generated/parser.rs` の再生成まで進めた上で revert した |
| (d) 手書き lexer をブロックコメント受理に合わせる | 仕様と他 2 実装に反する。UBNF の受理範囲を広げてしまう |

### 修正（D-051）

`crates/ubnfc-selfhost/src/lib.rs` の `parse_file` が、生成パーサの結果を見る前に
`block_comment_start(source)` で最初のブロックコメントを探し、あれば手書き lexer と同じ
位置・同じ `expected` で `SelfhostError::Syntax` を返す。

- 走査規則は手書き lexer と同じ: 文字列リテラル（`'…'` / `"…"`、`\` エスケープ込み）と
  `//` 行コメントの中は見ない。
- **`/*` より手前に別の字句エラー**（未閉の文字列、使用できない文字）があるときは `None` を返し、
  生成パーサ自身の診断に任せる。手書き lexer は解析前に入力全体を走査するので、
  この順序でしか位置が一致しない。
- **再解析ではなく字句の門**なので、`docs/design/05-selfhost.md` の
  「この経路は `syntax::parse_file` へフォールバックしない」という契約は保たれる。
- IR・生成物・固定点は 1 byte も変わらない。
- **速度**: `ubnfc_selfhost::parse_file` に入力全体の byte 走査（分岐のみ・確保なし）が 1 回増える。
  [速度比較](2026-09-23-parser-speed-comparison.md) の family C「AST(UbnfFile 射影)」行だけが
  この分を含む（同報告の他の行は `generated::parse_with_options` を直接呼ぶので無関係）。
  走査は解析より 2 桁以上安いので計測値の解釈は変わらないが、再計測時はこの差を織り込むこと。

`docs/design/05-selfhost.md` の既知乖離節に「7 件目 — 唯一の逆向き乖離」として、
表・根本原因・採らなかった案・上流での本来の直し方を追記した。

### 回帰テスト

`crates/ubnfc-selfhost/tests/parity.rs` の `block_comments_are_rejected_by_both_frontends`:
4 位置（先頭・中間・末尾・要素間）で両 frontend の拒否位置が一致すること、
文字列リテラル中／行コメント中の `/*` は従来どおり受理されること、
`/*` より手前に構文エラーがある 2 入力で位置が一致することを固定する。

---

## 4. Java backend だけ文字列値に引用符が残る（256 件）

### 根本原因

`examples/p4-java/src/main/java/org/ubnfc/p4/P4Scanners.java` が、3 つの Extern scanner
すべてに対して `new TokenScanner.ValueSpan(start, end)`、つまり **token 全体**を value span に
していた。

対して、
- `scanners/rust/scanners.rs:143-144`: `value_start = start + (quote == '\'')`,
  `value_end = p - (quote == '\'')`
- `scanners/java/Registry.java:70-71`: `valueStart = quote == '\'' ? start + 1 : start`

はどちらも**単引用符のときだけ** value span を 1 単位内側へ寄せる。
`examples/p4-rust` は共有 registry をそのまま `#[path]` で取り込んでいるのに対し、
**`examples/p4-java` だけが同じ 3 scanner を独自に書き直していた**のが差の出どころ。

### 正本

`unlaxer-parser/unlaxer-dsl/specs/generators.md` #132:
「既存の文字列変換（**外側空白の strip と単引用符だけの除去**）を適用し、
内部の区切り文字やコメントは消さない」。
実装上の出所は `unlaxer-common` の `elementary/QuotedParser.java:55-71`。
旧 Mapper もこの規則で、`examples/p4-java/evidence/ast-differences.json` の
`legacy: "jp"` / `generated: "'jp'"` が旧実装が除く側であることを示している。

### 修正

`P4Scanners.scanner(...)` に `unquote` を足し、STRING だけ

```java
return unquote && end - start >= 2 && in.startsWith(start, "'")
    ? new TokenScanner.ValueSpan(start + 1, end - 1)
    : new TokenScanner.ValueSpan(start, end);
```

とした。二重引用符は従来どおり value に残す（共有 registry と同じ）。
scanner は入力文字列を改変せず、span を寄せるだけ。
**生成器・生成コード・IR は変更していない**（差は例の手書き scanner だけにあった）。

### 影響（すべて「旧実装へ近づく」向き）

| 対象 | 変化 |
|---|---|
| `examples/p4-java/src/test/resources/golden/` | 5 本を再生成（`complex`, `complex-x4/x16/x64`, `valid-unicode`）。`boolean-entry/` は文字列を含まず不変 |
| `examples/p4-java/evidence/ast-differences.json` | 旧実装との差が **324 行減**。`complex-x4` 12→0、`complex-x16` 48→0、`complex` 4→1、`valid-unicode` 4→3。**残るのは UTF-16 span の差だけ** |
| `examples/p4-java/src/test/java/.../ScannerTest.java` | `valueSpan` の期待値を更新（`''`→`[1,1]`、`'a'tail`→`[1,2]`、`"'a'"`→`[0,5]`、`'😀'` の matchOnly →`[2,4]`） |
| corpus | **不変**。理由は次節 |

### なぜ 1,425 ケースのコーパスも差分ファジングも捕まえられなかったか

**理由は 2 つあり、2 つ目のほうが重い。**

1. **corpus と fuzzer は、この scanner を一度も実行していない。**
   `crates/ubnfc-harness/src/runner/scanners.rs:9-22` が、生成した成果物へ
   **共有 registry**（`scanners/java/Registry.java` と `scanners/rust/scanners.rs`）を
   注入する。共有 registry は両言語とも単引用符を除くので、**両 backend が一致する**。
   差が出ていたのは `examples/p4-java/P4Scanners.java` という**第 2 の Java 実装**だけで、
   corpus も差分 fuzzer もそこを通らない。
   差分 fuzzer が `token:singleQuoted`（サンプル `'q'` / `'😀'` / `''`）と
   `extern 'SingleQuotedParser'` を実際に生成している（`crates/ubnfc-fuzz/src/gen.rs:79-89,128-131`）
   にもかかわらず見つからなかったのは、この「両側とも共有実装」という構造のため。
   すなわち **fuzzer の生成側の穴ではなく、比較対象の範囲の穴**だった。

2. **両 backend を実際に突き合わせる比較は存在したが、「差があること」を期待値にしていた。**
   `examples/p4-rust/tests/fixtures.rs` の
   `java_goldens_agree_except_single_quoted_string_values` が Java golden と Rust golden を
   `serde_json::Value` として比べていたが、その中身は

   - `strip_single_quotes()` で **Java 側の `'jp'` を `jp` に書き換えてから**比較する、
   - 書き換えた個数を fixture ごとに固定する（complex 3 / x4 12 / x16 48 / x64 192 /
     valid-unicode 1 = **合計 256**）、
   - 書き換え前は `assert_ne!(rust, java, "生の golden は一致しないはず")` で
     **一致しないことを要求する**

   というもので、報告書が数えた 256 件と**同じ 256 件**がテストの期待値になっていた。
   両 README もこれを「既知の差」として文章化していた
   （`p4-java/README.md`「`valueSpan` は引用符も含む token 全体。引用符の除去は旧 Mapper の
   意味変換であり…」、`p4-rust/README.md`「Java golden とは文字列値の単引用符だけが異なる」）。
   つまり**検証は差を見ていたのに、欠陥ではなく仕様として登録されていた**。

### 何が塞いだか

- テストを `java_and_rust_goldens_are_identical_without_normalization` に置き換えた。
  **正規化を一切せず**、`ACCEPTED` 9 本 + `BOOLEAN_ENTRY` 4 本の計 13 golden について
  Java backend と Rust backend の canonical AST が `serde_json::Value` として等しいことを要求する
  （key 順の違いだけは `Value` 比較が吸収する）。比較件数も `assert_eq!` で固定し、
  fixture が増えたのに比較が漏れる形にならないようにした。
  `strip_single_quotes` は削除した。
- 両 README の「既知の差」の記述を、実際の状態（**正規化なしで完全一致**）に直し、
  この経緯へのリンクを残した。
- これにより、**backend 間の観測差はもう「期待値に変換」できない**。
  同種の差が再び入れば、`cargo test --workspace` がこのテストで落ちる。

**残った限界**（正直に書く）: この閉じ方は「P4 例の 2 実装が一致すること」を保証するが、
「共有 registry と `P4Scanners` が一致すること」を構造的に保証するものではない。
`scanners/java/Registry.java` は `package ubnfc.scanners;` で `generated.api.*` を import する形なので、
p4-java（package `org.ubnfc.p4.generated`）からそのままは使えない。
根治は例側でも共有 registry を使えるようにすること（package を生成時に差し替える仕組みは
harness が既に持っている）で、これは別件。

---

## 検証

`cd ubnfc-java && mvn -o -q -DskipTests package` を先に実行（D-033 の jar 鮮度）。

| 項目 | 結果 |
|---|---|
| `cargo test --workspace --offline` | 全 pass |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | 警告なし |
| `cargo fmt --all -- --check` | 差分なし |
| `cd ubnfc-java && mvn -o -q test` | pass |
| `cd examples/p4-java && mvn -o -q test` | 28 tests pass |
| self-host 固定点（`ubnfc-selfhost` の `parity` / `regenerate`） | pass（IR・生成物とも byte 不変） |
| `cargo test -p ubnfc-fuzz` | pass |
| corpus Java | **Pass 1310 / Fail 0**（期待拒否 28、Known 28、未観測 59） |
| corpus Rust | **Pass 1292 / Fail 0**（期待拒否 30、Known 28、未観測 75） |

コーパスの系統別表は
[`2026-09-23-java-corpus-contract.md`](2026-09-23-java-corpus-contract.md) /
[`2026-09-23-rust-corpus-contract.md`](2026-09-23-rust-corpus-contract.md) と
**22 系統すべて完全一致**（JSON のスカラー集計も全一致）。実行結果は
`2026-09-23-{java,rust}-corpus-crosscheckfix.{md,json}`。

再現:

```sh
cd ubnfc-java && mvn -o -q -DskipTests package && cd ..
cargo run -q --offline --bin ubnfc-harness -- run-all --backend java \
  --report docs/reports/2026-09-23-java-corpus-crosscheckfix.md \
  --json   docs/reports/2026-09-23-java-corpus-crosscheckfix.json
cargo run -q --offline --bin ubnfc-harness -- run-all --backend rust \
  --report docs/reports/2026-09-23-rust-corpus-crosscheckfix.md \
  --json   docs/reports/2026-09-23-rust-corpus-crosscheckfix.json
```

## 決定行

[`../design/03-design-v1.md`](../design/03-design-v1.md) §8 に 2 行追加した。

- **D-050** 不正な token 宣言・注釈値は診断する（受理範囲を狭める）
- **D-051** self-host frontend は UBNF 自身のコメント規則を守る

#4 は契約の選択ではなく、共有 registry と旧実装に合わせる不具合修正なので決定行にしていない。
