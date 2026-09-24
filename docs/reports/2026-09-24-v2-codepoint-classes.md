# UBNF v2 の code point クラス（D-063）

2026-09-24。issue [#34](https://github.com/opaopa6969/ubnfc/issues/34)。
[`2026-09-24-antlr-treesitter-comparison.md`](2026-09-24-antlr-treesitter-comparison.md) §1.1 A の申し送り。

## 0. 何が問題だったか

RFC 8259 §7 は JSON 文字列に生で置ける文字を「`"` と `\` と **U+0000〜U+001F** を除く全 Unicode 文字」と定める。
ANTLR は `~["\\\u0000-\u001F]`、tree-sitter は `/[^"\\\x00-\x1f]/` で 1 行。**UBNF v2 にはこれを書く手立てが無かった。**

| 書き方 | 何が足りないか |
|---|---|
| `NEGATION('…')` | 除外集合はリテラルで、リテラルのエスケープは `\n \t \r \\ \'` の 5 つ。綴れる制御文字は 3 つだけ |
| `CHAR_RANGE('min','max')` | 境界が Java `char` = BMP 限定（`E-TOKEN-CHAR-RANGE-SUPPLEMENTARY`）。正の範囲で書くと補助面の生文字を落とす |
| `REGEX('[^"\\\x00-\x1F]')` | front の正則部分集合パーサが `\x` を拒否 |

そのため `examples/json/json.ubnf` は `token SAFE = NEGATION('"\\')` を採り、**制御文字が通ることを受け入れていた**。
JSONTestSuite でちょうど 3 件（`n_string_unescaped_{ctrl_char,newline,tab}`）だけ ANTLR / tree-sitter と割れていた。

## 1. 採った拡張（D-063）

**新しい構成子を足さない。**既存の 2 つに、`@ubnf: 2` 限定（D-038）で 2 つのことを許す。

- **(a) エスケープ** — `CHAR_RANGE` の 2 つの境界と `NEGATION` の除外集合で、
  `\uXXXX`（16 進 4 桁ちょうど）と `\u{X…}`（1〜6 桁、U+10FFFF まで）を **code point 1 個**へ展開する。
- **(b) 補助面** — `CHAR_RANGE` の境界は U+0000..U+10FFFF の任意の code point 1 個。
  `E-TOKEN-CHAR-RANGE-SUPPLEMENTARY` は **v1 専用**になる。

D-050 の残り 3 条件（**展開後にちょうど 1 個**・`min <= max`・**surrogate を含まない**）は v2 でもそのまま。

### 候補 (c) を採らなかった理由

issue の候補 (c)（`REGEX` の部分集合に `\xHH` / `\uHHHH` / クラス否定を足す）は、
front の正則部分集合解析器と DFA 構築に触る。加えて **`REGEX` の文字列は解析器が生の `\` 列を読む**ので、
前段で `\u` を展開すると意味が二重になる（`\u0061` を `a` に畳んだ後の `[a-z]` は元の綴りと違う木になる）。
`CHAR_RANGE` の選択で同じ集合が書ける以上、先に安いほうを採った。別 issue で扱う。

### `NEGATION` に範囲を書けるようにしなかった理由

「32 個の制御文字を `NEGATION` に並べるのは冗長」は正しい。だが**正の範囲**（`CHAR_RANGE` を `|` で並べる）で
同じ集合が 4 本で書けるので、`NEGATION('…', CHAR_RANGE(…))` や `NOT_CHAR_RANGE(min,max)` を足すと
**構成子が 1 つ増えるぶんだけ損**になる。中立語彙（D-041）の表も動かさずに済む。

## 2. なぜ「原文から」展開するのか

字句（`syntax/lexer.rs` の `string()`）は `\\` を 1 個の `\` に畳む。したがって

| UBNF の綴り | 意味 | 字句値 |
|---|---|---|
| `'\\u0041'` | バックスラッシュ + `u0041` の 6 文字（原文は 7 文字） | `\u0041` |
| `'\u0041'` | （v1）未知のエスケープ = 6 文字 | `\u0041` |

**字句値では区別できない。**そこで AST の `TokenValue::CharRange` / `Negation` に
**引用符の内側の原文**を持たせ、`crates/ubnfc-front/src/escape.rs` が原文を 1 度だけ走査する。

走査規則は v1 と同じで `\u` の枝を足しただけなので、**`\u` を含まない原文では v1 の字句値と 1 文字も違わない**。
これを `crates/ubnfc-front/tests/escape.rs` が 11 個の綴りで固定している（`'\\u0041'` を含む）。

## 3. v1 は 1 bit も動かない

| 押さえたもの | 結果 |
|---|---|
| IR golden 22 本 + `ir/examples/tinycalc.ir.json` | **byte 一致**（`pipeline_golden.rs` の 4 テスト） |
| v1 / v2 で同じ token 宣言 → 同じ `semantics` / `analysis` / `plan` | 一致（`ubnf_v2.rs::v1_token_declarations_are_untouched_by_the_code_point_escapes`） |
| self-host 固定点（`ubnf.ubnf` は sibling 側の v1） | 不変。この拡張は `ubnf.ubnf` の解析に要らない |
| corpus | Java Pass 1310 / Rust Pass 1292 / Fail 0（不変） |
| IR schema | `charRange` の `maximum` を 65535 → 1114111 へ**広げた**だけ。既存 IR はすべて有効なまま |

**v1 での `\u` は明示的に拒否する**（`E-TOKEN-ESCAPE-V2-ONLY`）。仕様
（`unlaxer-parser/unlaxer-dsl/specs/ubnf-syntax.md`）は「表にないエスケープ列はエラーにしない。展開結果は実装依存」と
述べているので v1 の `'\u0041'` は 6 文字として受理されていた。狭めるにあたって、
**実在の v1 文法（本 repo・`unlaxer-parser`・`tinyexpression`・corpus・fixture・fuzz の findings）に
`CHAR_RANGE` / `NEGATION` の中の `\u` が 1 件も無い**ことを確かめた。D-050 と同じ「狭めるが実害ゼロ」の形である。

### 新しい診断

| code | いつ | hint |
|---|---|---|
| `E-TOKEN-ESCAPE-V2-ONLY` | v1 で `\u` を書いた | `@ubnf: 2` を宣言するか、文字をそのまま綴る |
| `E-TOKEN-ESCAPE-UNICODE` | 4 桁でない / `}` が無い / U+10FFFF 超 / 16 進でない | `\uXXXX` は 4 桁ちょうど、`\u{X…}` は 1〜6 桁 |
| `E-TOKEN-ESCAPE-SURROGATE` | 値が U+D800..U+DFFF | 単独 surrogate は code point ではない |

category は既存の `TOKEN`（D-050 で作ったもの）。

## 4. 生成器は 1 行も変えていない

両 backend の照合は**元から code point 単位**だった。

| backend | 1 文字の読み方 | `charRange` の判定 |
|---|---|---|
| Rust | `Input::cp_at(pos)` が UTF-8 を 1 scalar 復号して `(char, len)` を返す | `(min..=max).contains(&(c as u64))` |
| Java | `Character.codePointAt` + `Character.charCount`（`Session.range`） | `ch < min || ch > max` |

`TokenSpec::CharRange { min, max }` は front では `u64`、Java IR では `Integer`。
front / IR 参照検査（`analysis/checks.rs`、`ir/references.rs`）の **BMP 上限 `0xFFFF` を `0x10FFFF` へ広げた**のと、
JSON Schema の `maximum` を広げたのが変更のほぼすべてで、**生成器**（`crates/ubnfc-rust/src/tokens.rs`、
`JavaEmitter` の `case "charRange"`、両 backend の runtime template）は 1 文字も触っていない。

唯一の例外は Java 側の IR 検査 `ubnfc-java/.../ir/Validation.java` の 1 行で、
`charRange` の境界条件を「端点が surrogate でない」から「**surrogate を含む範囲でない**、かつ
`max <= 0x10FFFF`」へ直した（front と Rust 側は元からこの条件。Java だけが緩く、
v2 で `max > 0xFFFF` が書けるようになると差が観測できるようになるため）。

新しい回帰テストは backend 側にも置いた:

- Rust: `crates/ubnfc-rust/tests/semantics.rs` の `char_range_supplementary`（U+E000..U+10FFFF）と
  `support/semantic_assertions.rs`（U+1F600 は通る、U+D7FF は落ちる）。
- Java: `ubnfc-java/src/test/java/org/ubnfc/java/TokenTest.java` に 2 件
  （U+1F600..U+1F607 と U+E000..U+10FFFF。孤立 high surrogate `\ud83d` は落ちる）。

## 5. JSONTestSuite（318 件）

`examples/json/json.ubnf` の `token SAFE = NEGATION('"\\')` を 4 本の `CHAR_RANGE` へ置き換えた:

```ubnf
token SAFE_SP   = CHAR_RANGE('\u0020','\u0021')        // U+0020..U+0021
token SAFE_MID  = CHAR_RANGE('\u0023','\u005B')        // U+0023..U+005B
token SAFE_BMP  = CHAR_RANGE('\u005D','\u{D7FF}')      // U+005D..U+D7FF
token SAFE_SUPP = CHAR_RANGE('\u{E000}','\u{10FFFF}')  // U+E000..U+10FFFF
Char ::= Escape | SAFE_SP | SAFE_MID | SAFE_BMP | SAFE_SUPP ;
```

`benchmarks/scripts/check-jsontestsuite.sh` を **同じ手順で 2 回**（`origin/master` の文法と本 PR の文法）走らせた。

| 実装 | `y_` 受理 / 95 | `n_` 拒否 / 188 | `i_` 受理 | `i_` 拒否 | suite との不一致 |
|---|---:|---:|---:|---:|---|
| ubnfc Rust — **before** | 95 | 185 | 20 | 15 | 3 件（`n_string_unescaped_*`） |
| ubnfc Rust — **after** | **95** | **188** | 20 | 15 | **なし** |
| ubnfc Java — **before** | 95 | 185 | 20 | 15 | 3 件（同上） |
| ubnfc Java — **after** | **95** | **188** | 20 | 15 | **なし** |
| ANTLR4 4.13.2 | 95 | 188 | 21 | 14 | なし（不変） |
| tree-sitter 0.25.10 | 95 | 188 | 22 | 13 | なし（不変） |

**実装間で割れるのは 2 件だけ**になった（従来 5 件）。

| file | suite の分類 | ubnfc | ANTLR4 | tree-sitter | 理由 |
|---|---|---|---|---|---|
| `i_structure_500_nested_arrays` | 実装依存 | reject | accept | accept | 生成ランタイムのスタック番兵（issue #35） |
| `i_structure_UTF-8_BOM_empty_object` | 実装依存 | reject | reject | accept | tree-sitter だけ BOM を読み飛ばす |

どちらも `i_`（実装依存）なので**規格違反ではない**。受理必須・拒否必須の 283 件は 4 実装が完全一致する。

生データは [`examples/json/evidence/jsontestsuite-verdicts.tsv`](../../examples/json/evidence/jsontestsuite-verdicts.tsv)
（318 行 × 4 実装）と同ディレクトリの `jsontestsuite-summary.md`。差分は 3 行だけである。

> **速度は測り直していない。** `Char` の選択が 2 候補から 5 候補に増えるので、
> [`2026-09-24-antlr-treesitter-comparison.md`](2026-09-24-antlr-treesitter-comparison.md) §1.2 の
> JSON の数字は動き得る。同報告の §1.2 にその旨を追記した（共有機に 3 つの作業が並走していて、
> いま測っても比較できる値にならないため）。この PR は正しさの穴を塞ぐもので、速度の主張はしない。

### 仕様由来 corpus

`corpus/spec/json/*.json` の期待値は **JSONTestSuite の `y_` / `n_` 接頭辞**から機械的に作っており
（`benchmarks/scripts/make-json-spec-corpus.py`）、実装の観測値ではない。
`n_string_unescaped_*` の 3 件は**最初から `reject`** で、引用は RFC 8259 §7 の

> All Unicode characters may be placed within the quotation marks, except for the characters that MUST be escaped:
> quotation mark, reverse solidus, and the control characters (U+0000 through U+001F).

**この PR で期待値は 1 件も書き換えていない**（書き換える必要が無い＝実装が仕様へ寄った、という向きである）。
`cargo test -p ubnfc-harness --test spec_corpus` は緑のまま。

## 6. 差分ファジング

`crates/ubnfc-fuzz/src/gen.rs` の **v2 seed 帯だけ**に新しい token の綴りを足した。**乱数は 1 つも追加で引かない**
（D-043 と同じ作法）ので、v1 seed の生成物は完全に不変である。

| seed | 綴り | 述語 | 印 |
|---|---|---|---|
| `% 3 == 0` | `CHAR_RANGE('\u0061','\u{66}')` | `CHAR_RANGE('a','f')` と**同じ** | `token:charRangeEscape` |
| `% 3 == 1` | `CHAR_RANGE('\u{1F600}','\u{1F607}')` | 補助面（samples も差し替え） | `token:charRangeSupplementary` |
| `% 3 != 2` | `NEGATION('\u0061b\u{A}')` | `NEGATION('ab\n')` と**同じ** | `token:negationEscape` |

1 つ目と 3 つ目は「同じ集合を別の綴りで書く」形なので、front の展開が食い違えば
**両 backend が同時にモデルと食い違って赤くなる**。2 つ目は補助面の照合そのものを両 backend に踏ませる。

CI の固定 campaign に `e_v2_code_point_class_tokens_agree_across_backends` を足した
（`crates/ubnfc-fuzz/tests/campaign.rs`）。3 つの機能を 1 つずつ含む v2 seed
（`V2_SEED_BASE + 3` / `+ 24` / `+ 29`）を 12 入力ずつ両 backend へ流し、
既知 signature 以外の差分が 0 件であることを固定する。

> **副産物**: 帯を探す途中で v2 seed 100276 が `diff:/diagnostics/*/expected/*` と
> `diff:/recoveries/*/captureOccurrences/*` で両 backend 不一致になることが分かった。
> **D-063 とは無関係**（生成器の差し替えを一時的に無効化して同じ seed を `NEGATION('ab\n')` で
> 生成し直しても同じ 3 つの signature が出る）なので、この PR では直さず
> [ubnfc#41](https://github.com/opaopa6969/ubnfc/issues/41) に切り出した。

## 7. 仕様側への提案

ubnfc は先行実装であり、正本は `unlaxer-parser/unlaxer-dsl/specs/ubnf-syntax.md` の token 節にある。
仕様追加の提案を issue として出した: [opaopa6969/unlaxer-parser#288](https://github.com/opaopa6969/unlaxer-parser/issues/288)

## 8. 検証

| 検査 | 結果 |
|---|---|
| `cargo test --workspace --offline --no-fail-fast` | 緑 |
| `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --check` | 緑 |
| `mvn -o test`（`ubnfc-java` / `ubnfc-java/runtime-template`） | 緑 |
| self-host 固定点（`ubnfc-selfhost` の `parity` / `regenerate`） | 緑・byte 一致 |
| 差分ファジング campaign | 緑 |
| corpus | Java Pass 1310 / Rust Pass 1292 / Fail 0 |
| IR golden 22 本 + `ir/examples/tinycalc.ir.json` | byte 一致 |
| 新しい golden `ir/examples/json-v2.ir.json` | 追加（v2 の `charRange` を 4 本固定） |
