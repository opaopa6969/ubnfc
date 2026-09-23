# UBNF v2 — 言語中立な token 語彙と明示 extern（D-041 / D-042）

> 2026-09-23。`docs/design/06-ubnf-v2.md` §5 の実装。設計決定は `docs/design/03-design-v1.md` §8 の D-041 / D-042。

## 0. 直した弱点

v1 の token 宣言は **host 言語のクラス名**を書く。

```ubnf
token NUMBER     = org.unlaxer.parser.elementary.NumberParser
token IDENTIFIER = org.unlaxer.parser.clang.IdentifierParser
token STRING     = org.unlaxer.tinyexpression.parser.StringLiteralParser
```

Java と Rust の両方を出す「言語中立な文法形式」を名乗る以上、これは層の破れである。
IR の側は既に組込み種別（`TokenSpec::Builtin`）と `Extern` へ正規化しているので、
**残っていたのは表層だけ**だった。v2 はそこだけを直す。

```ubnf
token NUMBER     = NUMBER                        // 言語中立な組込み語彙
token STRING     = extern 'StringLiteralParser'  // host 実装は extern と明示
```

## 1. 表層の形 — なぜこの綴りか

| 案 | 採否 | 理由 |
|---|---|---|
| **裸の語（採用）** | ✅ | token 宣言の**値の位置**には利用者の token 名も規則名も現れない。つまり「利用者の名前と衝突し得ない」位置である。しかも `ANY` / `EOF` / `EMPTY` / `UNTIL(…)` / `CI(…)` が v1 から既にこの位置の裸の語であり、語彙を足すのはその一貫した延長にすぎない |
| 名前空間形（`builtin.NUMBER`） | ✗ | `dotted()` が読む形なので **host クラス名と同じ形**（`パッケージ.クラス`）になり、「クラス名を書かない」という目的に対して字面が逆行する |
| `builtin` キーワード（`= builtin NUMBER`） | ✗ | 1 語ぶん長くなるだけで、裸の語に無い曖昧さを 1 つも消さない。曖昧さは「host クラスの単純名と綴りが重なる」ケースだけで、それは下の 2 点で既に閉じている |

裸の語で起き得る唯一の衝突は「中立名と同じ綴りの host クラス単純名」である。これは

1. 中立名を **SCREAMING_SNAKE_CASE** に限ったので、`UpperCamelCase` が慣習の host クラス名とは字面が重ならない。
2. 重なっても **`extern '<id>'` が任意の名前を綴れる**（`token X = extern 'NUMBER'` は host 実装を指す）。

の 2 点で閉じている。つまり裸の語は「**最小で、かつ回復可能**」な形である。

## 2. 語彙 — 各名前が指す述語

右列は `docs/design/02-facts-tokens-bench-corpus.md` §A の受理範囲そのものである。
**中立名は §A の述語を 1 つも変えない。** これは解説ではなく実装の構造で保証している:
`crates/ubnfc-front/src/vocabulary.rs` は中立名を旧実装の FQN へ写すだけで、
その FQN を**既存の分類器 `resolve::builtin_class` に渡す**。したがって
「中立名で書いた IR」と「FQN で書いた IR」は構成上同じ組込み種別になる
（`crates/ubnfc-front/tests/ubnf_v2.rs` の
`every_neutral_name_produces_the_same_token_spec_as_its_host_class_spelling` が
36 名前すべてで `spec` / `diagnosticProfiles` / memo 安全性の一致を固定する）。

| 中立名 | v1 の綴り | 受理範囲（§A） |
|---|---|---|
| `NUMBER` | `elementary.NumberParser` | `[+-]? ( D+ '.' D+ \| D+ '.' \| D+ \| '.' D+ ) ( [eE] [+-]? D+ )?` |
| `IDENTIFIER` | `clang.IdentifierParser` | `[A-Za-z_][A-Za-z0-9_]*`（ASCII のみ） |
| `SINGLE_QUOTED` | `elementary.SingleQuotedParser` | `' ( \ <任意1cp> \| [^'] )* '` |
| `DOUBLE_QUOTED` | `elementary.QuotedParser` | `" ( \ <任意1cp> \| [^"] )* "` |
| `END_OF_INPUT` | `elementary.EndOfSourceParser` | 入力末尾でのみ成功、ゼロ幅 |
| `DIGIT` | `posix.DigitParser` | `[0-9]` 1 文字 |
| `ALPHA` | `posix.AlphabetParser` | `[A-Za-z]` |
| `ALNUM` | `posix.AlphabetNumericParser` | `[A-Za-z0-9]` |
| `ALPHA_UNDERSCORE` | `posix.AlphabetUnderScoreParser` | `[A-Za-z_]` |
| `ALNUM_UNDERSCORE` | `posix.AlphabetNumericUnderScoreParser` | `[A-Za-z0-9_]` |
| `WORD_CHAR` | `posix.WordParser` | `[A-Za-z0-9_]`（文字集合は `ALNUM_UNDERSCORE` と同じだが IR の `which` が違う別の組込み） |
| `ASCII` | `posix.AsciiParser` | `[\x00-\x7F]` |
| `BLANK` | `posix.BlankParser` | `[ \t]` |
| `CONTROL` | `posix.ControlParser` | ASCII 制御 `[\x00-\x1F\x7F]` |
| `GRAPH` | `posix.GraphParser` | `[\x21-\x7E]` |
| `PRINT` | `posix.PrintParser` | `[\x20-\x7E]` |
| `LOWER` | `posix.LowerParser` | `[a-z]` |
| `UPPER` | `posix.UpperParser` | `[A-Z]` |
| `PUNCT` | `posix.PunctuationParser` | ASCII 記号 |
| `SPACE` | `posix.SpaceParser` | `[\t\n\v\f\r ]` |
| `XDIGIT` | `posix.XDigitParser` | `[0-9A-Fa-f]` |
| `COLON` | `posix.ColonParser` | `:` |
| `COMMA` | `posix.CommaParser` | `,` |
| `DOT` | `posix.DotParser` | `.` |
| `HASH` | `posix.HashParser` | `#` |
| `SEMICOLON` | `posix.SemiColonParser` | `;` |
| `BACKSLASH` | `ascii.BackSlashParser` | `\` |
| `SLASH` | `ascii.SlashParser` | `/` |
| `DOUBLE_QUOTE` | `ascii.DoubleQuoteParser` | `"` |
| `EQUAL` | `ascii.EqualParser` | `=` |
| `GREATER_THAN` | `ascii.GreaterThanParser` | `>` |
| `LESS_THAN` | `ascii.LessThanParser` | `<` |
| `MINUS` | `ascii.MinusParser` | `-` |
| `PLUS` | `ascii.PlusParser` | `+` |
| `LEFT_PAREN` | `ascii.LeftParenthesisParser` | `(` |
| `RIGHT_PAREN` | `ascii.RightParenthesisParser` | `)` |

**語彙に入れなかったもの**（入れる意味が無いもの）:

| 旧綴り | 理由 |
|---|---|
| `ascii.PointParser` | `DOT`（`posix.DotParser`）と **IR が 1 バイトも変わらない**（どちらも `CodePointSet [46]`）。別名を足しても指す述語は 1 つ |
| `ascii.DivisionParser` | 同じ理由で `SLASH`（`ascii.SlashParser`）と同一（`CodePointSet [47]`） |
| `ANY` / `EOF` / `EMPTY` / `UNTIL(…)` / `CI(…)` / `NEGATION(…)` / `CHAR_RANGE(…)` / `LOOKAHEAD(…)` / `NEGATIVE_LOOKAHEAD(…)` / `REGEX(…)` | v1 から既に中立な綴り。直すものが無い |

> `EOF`（`TokenSpec::Eof`）と `END_OF_INPUT`（`Builtin{which:EndOfSource}`）は
> 「入力末尾でゼロ幅成功」という点は同じだが **IR の種別と診断プロファイルが違う別の token** で、
> v1 の観測がそれを固定している。だから 1 つに畳まず、`END_OF_INPUT` を
> `EndOfSourceParser` の中立な綴りとして足すだけにした。

## 3. host 実装の明示 — `extern '<id>'`

```ubnf
token CODE_START = extern 'CodeStartParser'
```

- **`'…'` は「クラス名」ではなく scanner registry の ID** である。registry
  （`scanners/first-chars.json` と `scanners/{java,rust}`）が ID を実装へ束ねる。
  FQN も書けるが、書く必要は無くなった。
- v1 では `extern` の直後に文字列が来る位置は**構文エラー**だったので、純粋な追加である。
- 単純名の `ClassRef` に出る `W-TOKEN-UNRESOLVED`（綴り間違いを拾うための警告）は
  extern には出さない。「host 実装である」ことは利用者が宣言しているからで、
  代わりに registry の有無を下の門で見る。
- 中立語彙と綴りが重なる host クラスの逃げ道でもある（`extern 'NUMBER'`）。

### registry が提供しない extern は**生成時に**落ちる

`ubnfc ir` / `ubnfc gen` に `--extern-first-chars`（＝ registry の宣言ファイル）を渡したとき、
**`@ubnf: 2` の文法**では宣言の無い extern を `E-TOKEN-EXTERN-UNREGISTERED` で落とし、
終了コード 1 を返す。v1 は凍結なので従来どおり stderr の注意だけで IR を出す。

```
$ ubnfc ir --grammar g.ubnf --extern-first-chars scanners/first-chars.json
ubnfc: E-TOKEN-EXTERN-UNREGISTERED: extern token implementation NoSuchScanner is not
provided by scanners/first-chars.json [hint: add NoSuchScanner to the scanner registry
(scanners/first-chars.json and scanners/{java,rust}), or fix the name in the extern declaration]
$ echo $?
1
```

門を**綴りではなく `languageVersion`** に掛けたのは、v2 が「host 実装は明示する」形式だと
宣言している以上、旧綴りの extern にも同じ責任を負わせるほうが一貫するからである。
registry を渡さない呼び出しは判定材料が無いので従来どおり（検査しない）。
固定は `crates/ubnfc-front/tests/ubnf_v2.rs`
`v2_rejects_an_extern_that_no_scanner_registry_provides`。

## 4. P4 の 3 つの extern は組込みで書けるか — 書けない

| token | 実装の述語 | 組込みで書けるか |
|---|---|---|
| `StringLiteralParser` | `Choice(DOUBLE_QUOTED, SINGLE_QUOTED)`（`scanners/java/Registry.java` の `case STRING`、tinyexpression `StringLiteralParser.java:19-25`） | **書けない**。§A に「引用符の選択」という組込みは無い。規則 `S ::= DOUBLE_QUOTED \| SINGLE_QUOTED` で近似はできるが、それは token を規則に替える変更で、記号が 1 つ増える（`rulePath` / `deepestRule` が変わり、`farthestExpected` の `StringLiteralParser` 表示ノードが Choice へ置き換わる）。綴りだけの移行ではない。加えて `Mode.matchOnly` 時に `resetMatchedWithConsumed` でカーソルを戻す token 層の約束（同 `run` 冒頭）は規則では表せない |
| `CodeStartParser` | **行頭**という位置述語 + `` ``` `` + `identifier ':' identifier ('.' identifier)*` + 行終端 | **書けない**。行頭判定に対応する組込みは §A に無い（`elementary.StartOfLineParser` は組込み対象外＝ Extern 扱い） |
| `CodeEndParser` | 行頭 + `` ``` `` + 行終端 | 同上 |

→ 3 つとも **明示 `extern` 宣言**にした。registry には短い ID の別名
（`StringLiteralParser` / `CodeStartParser` / `CodeEndParser`）を FQN と並べて足した
（`scanners/first-chars.json`・`scanners/java/Registry.java`・`scanners/rust/scanners.rs`、いずれも追加のみ）。

## 5. v1 / v2 の方針（D-042）

| 綴り | v1 | v2 |
|---|---|---|
| 中立語彙（`= NUMBER`） | **使える**（純粋な追加） | 使える |
| `extern '<id>'` | **使える**（純粋な追加） | 使える |
| host クラス名（`= org.unlaxer.…`） | 使える（無警告） | **使えるが `W-TOKEN-HOST-CLASS`** |
| registry に無い extern（`--extern-first-chars` 使用時） | 注意のみ・IR は出る | **`E-TOKEN-EXTERN-UNREGISTERED`・exit 1** |

### v2 は旧綴りを**拒否しない**。理由

1. **唯一の実利用者の文法をこちらが直せない。** tinyexpression の P4
   （`tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf`）は別 repo の
   読み取り専用ファイルで、token 宣言 7 本のうち 6 本が host クラス名である。v2 が旧綴りを
   拒否すると、この文法は「左再帰（D-039）を使うには token 宣言も同時に書き換える」ことを
   強いられる。v2 は**機能ごとに採れる**べきで、`@ubnf: 2` が token 宣言の破壊的変更を
   連れてくる設計にはしない。
2. **拒否しても得るものが無い。** 分類器（`builtin_class`）は v1 のために永久に残るので、
   拒否はコードを減らさない。強制したい利用者は D-037 と同じ `--strict` で
   `W-TOKEN-HOST-CLASS` をエラーにできる。既存の機械がそのまま使える。
3. **移行費用は低いが、強制する費用のほうが高い。** 実測（§6）では P4 の移行は token 宣言 6 行で
   済み、IR の差は 4 キー 20 箇所だけだった。これだけ安いなら、警告で十分に動機づけられる。

v1 で `W-TOKEN-HOST-CLASS` を**出さない**のは、v1 では旧綴りが唯一の綴りだからである
（凍結された 24 fixture・P4・corpus に意味の無い警告を足さない）。

## 6. P4-v2 の移行と証拠

`examples/p4-v2/tinyexpression-p4-v2.ubnf`（左再帰で書き直した P4、D-039）の token 宣言を
中立語彙と明示 extern へ移した。**token 宣言 7 行以外は 1 文字も変えていない。**

```ubnf
token NUMBER     = NUMBER
token IDENTIFIER = IDENTIFIER
token STRING     = extern 'StringLiteralParser'
token CODE_START = extern 'CodeStartParser'
token CODE_BODY  = UNTIL('```')
token CODE_END   = extern 'CodeEndParser'
token EOF        = END_OF_INPUT
```

### 6.1 IR は綴り以外 1 バイトも変わらない

移行前後の IR（`ubnfc ir --extern-first-chars scanners/first-chars.json`、
どちらも正規化すると 115,223 行の JSON）を、文法パスと byte offset だけ正規化して比較した差分は
**20 箇所・4 キーだけ**である（いずれも「綴りそのもの」か「人が読む説明文」）。

| 変わったフィールド | 何 | 件数 |
|---|---|---:|
| `memoSafety.originalDeclaration` | token 宣言の**原文そのもの**（IR は token を複数箇所に写す） | 11 |
| `memoSafety.reason` | 人が読むための説明文 | 3 |
| `TokenSpec.originalClassName` | extern の registry ID（FQN → 短い ID） | 3 |
| `TokenSpec.javaBinding` | 同上 | 3 |

`spec`（`kind` / `which` / `codePoints` / `firstChars`）、`diagnosticProfiles`、
`memoSafety` の `success` / `failure` / `provenance` / `purity`、`analysis`、`plan`、
`symbols`、AST 型、recipe は**すべて一致**した。`firstChars` が一致したのは、registry に
短い ID の別名を FQN と同じ値で足したためである。

### 6.2 parser の観測も変わらない

`examples/p4-v2/scripts/compare.sh` は、commit 済みの v1 P4 IR と P4-v2 から
Rust backend で parser を生成し、tinyexpression の fixture 16 本（`examples/p4-rust/tests/fixtures/*.tiny`、うち 7 本は
tinyexpression の `benchmarks/fixtures/*.tiny` と byte 一致）と診断契約を突く拒否入力 12 本、
計 28 入力の driver 応答（`accepted` / 二カーソル / AST / `diagnostics` / `scope` / …）を比較する。

```
case                       accepted  cursors    ast   diag   all
comparison-heavy-x16.tiny      True     True   True   True  True
comparison-heavy-x4.tiny       True     True   True   True  True
comparison-heavy-x64.tiny      True     True   True   True  True
comparison-heavy.tiny          True     True   True   True  True
complex-half.tiny              True     True   True   True  True
complex-tail.tiny              True     True   True   True  True
complex-x16.tiny               True     True   True   True  True
complex-x4.tiny                True     True   True   True  True
complex-x64.tiny               True     True   True   True  True
complex.tiny                   True     True   True   True  True
flat-arithmetic.tiny           True     True   True   True  True
invalid-syntax.tiny            True     True   True   True  True
large-match.tiny               True     True   True   True  True
valid-basic.tiny               True     True   True   True  True
valid-multiline.tiny           True     True   True   True  True
valid-unicode.tiny             True     True   True   True  True
reject:trailing-plus           True     True   True   True  True
reject:double-star             True     True   True   True  True
reject:unclosed-paren          True     True   True   True  True
reject:unclosed-paren-mul      True     True   True   True  True
reject:empty                   True     True   True   True  True
reject:bad-atom                True     True   True   True  True
reject:bad-bool                True     True   True   True  True
reject:bad-xor                 True     True   True   True  True
reject:bad-concat              True     True   True   True  True
reject:var-decl                True     True   True   True  True
prefix:number                  True     True   True   True  True
prefix:paren                   True     True   True   True  True
応答全体の一致（段名の読み替えのみ）: 28/28
AST の一致（読み替えなしの生比較）: 28/28
```

段の名前の読み替え（D-039 の脱糖が作る `NumberExpression__lr1` → `NumberTerm` など）以外の
正規化は無く、**token の綴りに由来する読み替えは 1 つも足していない**。
extern token の診断表示名が変わらないのは、両 backend が
`originalClassName` の**最後の `.` より後ろ**を表示名に使うためである
（`DiagnosticPlan.java:121`、`crates/ubnfc-rust/src/diagnostics.rs:317-324`）。
`org.unlaxer.tinyexpression.parser.javalang.CodeStartParser` も `CodeStartParser` も
表示名は `CodeStartParser` で、同じ観測になる。

この比較は **D-040 の語境界を取り込んだあとの木**で走らせている。P4-v2 は `@ubnf: 2` なので
リテラルに語境界が付き、比較相手の v1 P4 には付かない。それでも 28 入力の応答が完全に一致する
のは、語境界の実測（`docs/reports/2026-09-23-ubnf-v2-word-boundary.md`）が言うとおり
P4 の実文法では退行が 0 だからで、v2 の 3 機能（左再帰・語境界・token 語彙）は
この文法の上で互いに干渉しない。

## 7. 差分 fuzzing

`crates/ubnfc-fuzz/src/gen.rs` の文法生成器は、**v2 の seed 帯**（`V2_SEED_BASE` 以降）で
token を中立語彙／明示 extern で綴る。

- `NumberParser` → `NUMBER`、`clang.IdentifierParser` → `IDENTIFIER`（機能 `token:neutral`）
- `elementary.SingleQuotedParser` → 偶数 seed は `SINGLE_QUOTED`、**奇数 seed は
  `extern 'SingleQuotedParser'`**（機能 `token:extern`）。registry が両言語で提供している ID なので、
  extern の表層も両 backend で突き合わせられる。

**乱数は 1 つも追加で引かない**（綴りの差し替えだけ）ので、既存 seed の生成物は 1 バイトも変わらない
（`v1_seeds_keep_the_host_class_spelling` / `v2_seeds_spell_tokens_with_the_neutral_vocabulary_or_explicit_extern`）。

## 8. 検証

| 対象 | 結果 |
|---|---|
| corpus Java `run-all` | **Pass 1310 / Fail 0**、系統別表は `docs/reports/2026-09-23-java-corpus-contract.md` と完全一致 |
| corpus Rust `run-all` | **Pass 1292 / Fail 0**、系統別表は `docs/reports/2026-09-23-rust-corpus-contract.md` と完全一致 |
| `cargo test --workspace --offline` | 464 件 pass / 0 fail（self-host の parity・regenerate 固定点、IR golden 23 本、差分 fuzzing の固定 seed campaign 4 本を含む） |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | 0 |
| `cargo fmt --all -- --check` | 0 |
| `ubnfc-java` `mvn -o -q test` | 0 |
| `examples/p4-java` `mvn -o -q test` | 0 |
| `examples/p4-v2/scripts/compare.sh` | 28/28 一致（§6.2） |
| IR golden（`ir/fixtures/*.ir.json`） | 無変更（`crates/ubnfc-front/tests/pipeline_golden.rs`） |

すべて `master`（`f7c37b7`、D-040 の語境界を含む）を取り込んだあとの木で走らせた。

## 9. 残っている宿題

- **self-host の文法（`unlaxer-dsl/grammar/ubnf.ubnf`）は v2 の表層を読めない。** 別 repo の
  読み取り専用ファイルなので、UBNF を UBNF で書いた定義は当面 v1 の綴りのままである。
  self-host 固定点は v1 の文法を入力にしているので影響は無いが、「表層言語とその自己記述の乖離」は
  §4 パラメータ化ルールなどでも同じ形で増える。まとめて閉じる回が要る。
- **`WORD_CHAR` と `ALNUM_UNDERSCORE` は同じ文字集合で別の組込み**である（IR の `which` が違う）。
  v1 の観測がそれを固定しているので今は畳めない。受理範囲の統合は別の決定。
- **`extern` の純粋性・先頭文字は registry の申告のまま**（D-008 / D-033）。表層が明示になっても、
  「その scanner が本当に純粋か」を ubnfc が検査できるようにはなっていない。
