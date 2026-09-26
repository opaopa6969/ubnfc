# formula-info — tinyexpression の FormulaInfo 文書

UBNF v2 で書いた 4 つ目の実用文法（issue #83）。tinyexpression の Java loader
（`org.unlaxer.tinyexpression.loader.FormulaInfo*Parser`）が読む、行指向のブロック形式。

```
tags:NORMAL
formula:
1 + 2
---END_OF_PART---
```

**出典と更新の向き**: この文法は tinyexpression 側で先に書かれたもの
（[opaopa6969/tinyexpression](https://github.com/opaopa6969/tinyexpression) issue #180、
PR [#191](https://github.com/opaopa6969/tinyexpression/pull/191)、
直近では issue [#211](https://github.com/opaopa6969/tinyexpression/issues/211) / PR
[#213](https://github.com/opaopa6969/tinyexpression/pull/213)（END_OF_PART 行の行末空白・
後続文字・calculatorName 重複）、commit `63945e02bdb9a87bb0832259f5e40a3789f61c03` の
`grammar/formula-info.ubnf`）をそのまま取り込んだもの。
**tinyexpression → ubnfc** の一方向で、ubnfc 側からは変更しない
（差分が要るときは tinyexpression 側で直してから再取込みする）。tinyexpression では
これを Rust backend で生成して vendoring し、薄い変換層（`formula_info::load`）で
値の意味検査・実行 backend の解決・評価まで通している。ubnfc 側のこのディレクトリは
**構文層だけ**を文法サンプルとして持つ。

| ファイル | 中身 |
|---|---|
| [`formula-info.ubnf`](formula-info.ubnf) | UBNF v2 の文法（tinyexpression からの取込み、無改変） |
| [`samples/`](samples/) | サンプル 53 本と [`samples/expected.json`](samples/expected.json)（期待判定） |
| [`scripts/crosscheck.sh`](scripts/crosscheck.sh) | 検証 → 両 backend 生成 → 53 本を突き合わせ |
| `ir/examples/formula-info.ir.json` | commit した Grammar IR の golden（repository ルート） |

生成物は commit しない（`examples/json` / `examples/stax` と同じ方針）。
必要になったらその場で生成する。

## 走らせ方

すべて repository のルートから。ネットワークは要らない。

```sh
# 1. 文法を検証する（警告ゼロが条件）
cargo run -q --offline --bin ubnfc -- validate --grammar examples/formula-info/formula-info.ubnf --strict

# 2. Grammar IR を見る（commit してある golden と byte 一致する）
cargo run -q --offline --bin ubnfc -- ir --grammar examples/formula-info/formula-info.ubnf --out - | head -40

# 3. 検証 → Rust / Java 両 backend を生成 → サンプル 53 本を突き合わせる
bash examples/formula-info/scripts/crosscheck.sh
```

`crosscheck.sh` は `mktemp -d` に生成して終了時に消す。
反復するときは `FORMULA_INFO_WORK=<dir> bash examples/formula-info/scripts/crosscheck.sh` で再利用できる。

回帰は `cargo test --offline -p ubnfc-front --test pipeline_golden` の 3 本が見ている
（IR golden の byte 一致 / `--strict` 警告ゼロ / サンプルと `expected.json` の対応）。

## サンプル 53 本

`samples/expected.json` が期待判定を持つ。判定はこの文法（**構文層**だけ）の期待値で、
`FormulaInfoBlocksParser` を全文消費で実行した旧 Java 実装
（`FormulaInfoSourceDocument.parse`）の判定と一致する。

| 本数 | 内容 |
|---:|---|
| 38 | **構文受理**。最小形 / 末尾改行なし / 終端マーク無し（EOF で閉じる）/ CRLF / CR のみ / 先頭コメント・空白行 / 値の先頭空白保持 / 終端マーク類似行 / ヘッダ類似行（`a.b`、`a b:c`、`1a:b`、`$x:y`、日本語）/ dotted key / コメントのみのブロック / 空文書 / 空白行のみ / VT・FF / Unicode / 既知キー・dependsOn・backend 別名・型名 / 空の値 / 既存 fixture 3 本（実データ）/ 終端マークの後ろの空白・タブ（issue #211、LF・CRLF） |
| — うち 11 | 構文は受理するが、旧 Java **loader**（意味検査: `formula` 必須・`byteCode` の hex 検査・`resultType`/`dependsOn` の解決・issue #211 で加わった「1 ブロック内の calculatorName 重複」検査）はここで拒否する（`expected.json` の `loaderReject: true`）。この文法はそこまで見ないので、構文層では accept のまま |
| 15 | **構文拒否**。終端マークで始まる / 終端マーク 2 連続 / 先頭のゴミ行 / 字下げされた key・コメント / `key :` / ブロック後のゴミ / 非 ASCII・数字・`$` 始まりの key / `---END_OF_PART--- `（末尾空白）/ 終端マークの後ろに空白以外の文字・VT（issue #211、4 本） |

`crosscheck.sh` は 53 本すべてについて、**受理（accepted / allConsumed）・canonical AST**
が Rust backend と Java backend で一致することを検査する。この文法に scope 注釈は無い。

## この文法が使っている UBNF の機能

- **`@whitespace: none`（行指向）**。json や stax はトークン区切りだが、この文法は行が
  意味の単位で、暗黙のスキップが一切無い。改行の種類（`\r\n` / `\r` / `\n`）・行末の
  扱い（EOF で閉じる）を規則で明示している。
- **規則レベルの否定先読みが無い UBNF での補集合の書き方**。「`KEY:` で始まらない行」
  という否定条件を、`NOT_ID_HEAD`（`NEGATION`）で始まる行 | `IDENT { '.' IDENT }` の
  直後が `:` でない行 | 空行、という**正の場合分け**で書き、`NEGATIVE_LOOKAHEAD('---END_OF_PART---')`
  と `LOOKAHEAD('\r'|'\n')` を組み合わせて終端行・行末条件を弾く。他の 3 文法（P4・JSON・
  stax）にはこのパターンが無い。
- `CHAR_RANGE('\u000B','\u000C')`（VT/FF）のような v2 の `\u` code point 境界。
- 全規則の `@doc` と、typed AST 用の `@mapping`
  （`FormulaInfoDocument{blocks}` / `FormulaInfoBlock{leading, entries, end}` /
  `FormulaInfoEntry{key, value}` / `FormulaInfoValue{text}` / `CommentLine` / `BlankLine` /
  `EndOfPart`）。
- **左再帰・パラメータ化ルール・scope 注釈は使っていない**。演算子の優先順位も無い
  （値はテキストの塊で、式としての構造は host 側 `formula_info::load` が見る）。

## 気付いた点（tinyexpression issue #180 から持ち越し）

text capture は前後の空白が trim されるため、値の**生の**文字列（先頭空白・末尾改行込み）
が要る利用者は `FormulaInfoValue` node の span から原文を切り出す。tinyexpression の
Rust loader はこのパターンを使っている。これが推奨パターンでよいか、raw capture の
綴りを別に用意するかは未解決のまま（tinyexpression 側の議論）。

## 次の段

この文法は取込みのみで、ubnfc 側のコード変更は無い。仕様側の変更は
tinyexpression（`grammar/formula-info.ubnf`）で先に行い、ここへ再取込みする。
