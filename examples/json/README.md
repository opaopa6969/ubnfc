# JSON（RFC 8259）を 3 記法で書いたもの

同じ言語を UBNF v2 / ANTLR 4 / tree-sitter の 3 通りで書いて、
[JSONTestSuite](https://github.com/nst/JSONTestSuite) で受理・拒否を突き合わせ、
同一入力で速度を測るための一式（issue #26）。

| ファイル | 記法 | 生成物 |
|---|---|---|
| [`json.ubnf`](json.ubnf) | UBNF v2（`@ubnf: 2`） | ubnfc Rust backend / Java backend |
| [`antlr/JSON.g4`](antlr/JSON.g4) | ANTLR 4.13.2 | Java parser（ALL(*)） |
| [`treesitter/grammar.js`](treesitter/grammar.js) | tree-sitter 0.25.10 | C parser（GLR） |

結果は [`docs/reports/2026-09-24-antlr-treesitter-comparison.md`](../../docs/reports/2026-09-24-antlr-treesitter-comparison.md)。
走らせ方は同報告の「再現手順」を見ること。

## UBNF 版が使っている v2 の機能

| 機能 | 書いた場所 | なぜ必要か |
|---|---|---|
| `@ubnf: 2` | 冒頭 | 以下の既定を有効にする |
| **語境界（`@literalBoundary` の v2 既定 `word`）** | `'true'` / `'false'` / `'null'` | `truex` を `true` + 余りではなく**リテラル不一致**として落とす。v1 既定の `none` では語境界が無い |
| `@whitespace: none`（暗黙 trivia を切る） | 冒頭 | RFC 8259 の ws はちょうど 4 文字。`javaStyle` は `//` `/* */` を必ず区切りに入れてしまい、JSON でコメントが通る（`crates/ubnfc-front/src/lower/trivia.rs:85-99`）。さらに `javaStyle` は 0x0B / 0x0C も空白扱いする |
| `CHAR_RANGE(...)` token | `ZERO` `ONENINE` `E_LOWER` `ESC_U` ほか | **語境界は rule 本体のリテラルにだけ効き、token には効かない**。`'0'` `'e'` `'u'` をリテラルで書くと `0e1` や `é` が語境界で失敗する。英数 1 文字は token にして避けている |
| **code point クラス（D-062）** — `CHAR_RANGE` の境界の `\uXXXX` / `\u{X…}` と BMP 外 | `SAFE_SP` `SAFE_MID` `SAFE_BMP` `SAFE_SUPP` | 文字列の中に生で置ける 1 文字（`"` `\` と U+0000..U+001F を除く全 code point）。v1 では書けない |
| `XDIGIT` / `DIGIT`（v2 の言語中立な組込み語彙、D-041） | `HEX` / `DIGIT` | host クラス名（`org.unlaxer...`）を書かずに済む |
| 有界反復 `HEX{4}` | `Escape` | `\uXXXX` の 4 桁 |
| `@mapping(...)` と、**全候補が mapped rule への参照だけの選択**（`Value`） | 各規則 | `JsonAST` が sealed interface + record の和型として自動で出る |

左再帰は使っていない（JSON に左再帰は無い）。`@scopeTree` / `@declares` / `@backref` /
`@precedence` も使っていない。

## 3 記法で揃えられなかった点

### 1. 文字列の中の制御文字（U+0000..U+001F）— **D-062 で解消**

RFC 8259 §7 は「引用符・逆ソリダス・制御文字 (U+0000〜U+001F) 以外の Unicode 文字は
そのまま置ける」と定める。ANTLR は `~["\\\u0000-\u001F]`、tree-sitter は
`/[^"\\\x00-\x1f]/` で書く。UBNF はかつてこの集合を書けず（issue #34）、
`token SAFE = NEGATION('"\\')` で**制御文字が通ることを受け入れていた**——
JSONTestSuite の `n_string_unescaped_{ctrl_char,newline,tab}` 3 件で他の 2 記法と判定が割れていた。

**v2 の code point クラス（D-062）で書けるようになった。**正の範囲を 4 本並べる:

```ubnf
token SAFE_SP   = CHAR_RANGE('\u0020','\u0021')        // U+0020..U+0021
token SAFE_MID  = CHAR_RANGE('\u0023','\u005B')        // U+0023..U+005B
token SAFE_BMP  = CHAR_RANGE('\u005D','\u{D7FF}')      // U+005D..U+D7FF
token SAFE_SUPP = CHAR_RANGE('\u{E000}','\u{10FFFF}')  // U+E000..U+10FFFF
```

- `\uXXXX` / `\u{X…}` は `CHAR_RANGE` の境界と `NEGATION` の除外集合でだけ使える（v2 限定）。
- `CHAR_RANGE` の境界の BMP 制限は v2 で外れたので、補助面（絵文字など）も範囲に入る。
- surrogate 区画（U+D800..U+DFFF）を含む範囲は引き続き拒否されるので、上側が 2 本に割れる。

**両 backend とも `n_` 188/188 拒否・`y_` 95/95 受理**になり、この項目の不一致は 0 件。
負の書き方（`NEGATION` にエスケープを書いて 34 文字を除外する）でも同じ集合を綴れるが、
正の範囲のほうが短いのでこちらを採った。

### 2. サロゲート対の妥当性 — 3 記法とも検査しない

RFC 8259 §7 は BMP 外の文字を `"𝄞"` のように UTF-16 サロゲート対で表すと述べるが、
`string` の ABNF は `\u` の後ろに 4 桁の 16 進を許すだけで、対になっているかは縛らない。
3 記法とも ABNF どおり「4 桁の 16 進」までしか書いていない。JSONTestSuite が
`i_string_lone_second_surrogate` 等を**実装依存 (`i_`)** に分類しているのはこのためで、
3 記法の判定は一致する（すべて受理）。

### 3. 入れ子の深さ — 実装依存

`i_structure_500_nested_arrays`（`[` 500 重）は ANTLR / tree-sitter が受理、ubnfc は拒否する。
ubnfc の生成ランタイムは**ネイティブスタックを 256 KiB 以上使う再帰**を
`maximum parse depth exceeded` で止める（生成物の `runtime.rs` のスタック番兵。
`ParseOptions.max_depth` とは別で、上げても変わらない）。この文法では 200 重あたりが境目になる。

### 4. 先頭 BOM — tree-sitter だけ受理

`i_structure_UTF-8_BOM_empty_object`（`EF BB BF 7B 7D`）は tree-sitter だけが受理する。
RFC 8259 §8.1 は「JSON text を解析する実装は BOM を誤りとせず無視して**よい**」と述べており、
どちらも規格内。3 つの文法はいずれも BOM を明示的には書いていない。

### 5. `\u` 以外の細かい対応

| | UBNF v2 | ANTLR 4 | tree-sitter |
|---|---|---|---|
| 空白 | `Ws ::= { ' ' \| '\t' \| '\n' \| '\r' }` を構造文字の前後に明示 | `WS : [ \t\n\r]+ -> skip ;` | `extras: $ => [/[ \t\n\r]/]`（既定の `/\s/` は広すぎるので置換） |
| 語境界 | v2 既定の語境界（リテラル） | 字句が最長一致 + 未知文字 token `ERRCHAR` | 字句が構文状態依存 + 未知文字が ERROR |
| 拒否の判定 | `ok == false`（`require_eof` 既定 true） | `BailErrorStrategy` + 例外を投げる `ErrorListener`（既定の回復に任せると受理になる） | `root.hasError()` と「ルートが入力全体を覆っているか」 |
| 「木」 | `@mapping` の record 木（typed AST） | parse tree（`ParserRuleContext`） | CST（必ず作られる） |

## 符号化の扱い（byte か code point か）

| 実装 | 入力の見え方 |
|---|---|
| ubnfc Rust | 生成物の入口が `&str`。不正な UTF-8 は**渡せない**ので、ドライバが厳密復号で弾く |
| ubnfc Java | `CharSequence`。ドライバが `CodingErrorAction.REPORT` で厳密復号し、失敗を拒否とする |
| ANTLR | 同上。ANTLR 既定の `CharStreams` は不正 byte を U+FFFD へ**置換**するので、置換に任せると不正 UTF-8 を受理してしまう |
| tree-sitter | `ts_parser_parse_string` に**byte 列と長さ**を渡す（NUL も通る）。不正な UTF-8 は tree-sitter 自身が字句で落とす |

UTF-16 で書かれた `n_`/`i_` ファイル、不正 UTF-8、overlong 符号化はいずれも
「UTF-8 として不正」なので 4 実装すべてが拒否する（理由の出所が違うだけ）。
