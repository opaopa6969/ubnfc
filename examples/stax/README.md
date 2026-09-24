# stax — 型定義と構造定義のできる、自己拡張型の連接言語

UBNF v2 で書いた 3 つ目の実用文法（issue #66）。Forth の系譜の**連接スタック言語**で、
`type` / `struct` と**型付きスタック効果** `( Int Int -- Int )` を宣言できる。
制御構造は組込みではなく、引用 `[ ... ]` を受け取る**ただの語**として言語の中で定義する。

設計の正本は [`DESIGN.md`](DESIGN.md)。

```
: square ( Int -- Int ) dup * ;

struct Point { x: Int y: Int }

: main ( -- )
  7 square int_to_str print newline ;
```

| ファイル | 中身 |
|---|---|
| [`DESIGN.md`](DESIGN.md) | 設計文書。目的・構文・型システム・自己拡張の境界・実行モデル・名前 3 案 |
| [`stax.ubnf`](stax.ubnf) | UBNF v2 の文法（27 規則） |
| [`samples/`](samples/) | サンプル 20 本と [`samples/expected.json`](samples/expected.json)（期待判定） |
| [`scripts/crosscheck.sh`](scripts/crosscheck.sh) | 検証 → 両 backend 生成 → 20 本を突き合わせ |
| `ir/examples/stax.ir.json` | commit した Grammar IR の golden（repository ルート） |

生成物は commit しない（`examples/json` / `examples/p4-v2` と同じ方針）。
必要になったらその場で生成する。

## 走らせ方

すべて repository のルートから。ネットワークは要らない。

```sh
# 1. 文法を検証する（W-CHOICE-* ゼロ・エラーゼロが条件）
cargo run -q --offline --bin ubnfc -- validate --grammar examples/stax/stax.ubnf --strict

# 2. Grammar IR を見る（commit してある golden と byte 一致する）
cargo run -q --offline --bin ubnfc -- ir --grammar examples/stax/stax.ubnf --out - | head -40

# 3. 検証 → Rust / Java 両 backend を生成 → サンプル 20 本を突き合わせる
bash examples/stax/scripts/crosscheck.sh
```

`crosscheck.sh` は `mktemp -d` に生成して終了時に消す。
反復するときは `STAX_WORK=<dir> bash examples/stax/scripts/crosscheck.sh` で再利用できる。

手で 1 本だけ動かしたいときは、生成した Rust driver に JSON Lines を投げる
（[`docs/harness-driver-protocol.md`](../../docs/harness-driver-protocol.md)）:

```sh
printf '%s\n' '{"id":0,"grammar":"Stax","entry":null,"input":": main ( -- ) 1 2 + print ;","options":{"lexical":true,"scope":true}}' \
  | "$WORK/rust-out/target/debug/driver"
```

回帰は `cargo test --offline -p ubnfc-front --test pipeline_golden` の 3 本が見ている
（IR golden の byte 一致 / `--strict` 警告ゼロ / サンプルと `expected.json` の対応）。

## サンプル 20 本

`samples/expected.json` が期待判定を持つ。**parse** は stage 1 で検証でき、
**typecheck** は stage 3（Rust インタプリタのスタック効果検査）の期待値である。

| 本数 | parse | typecheck | 内容 |
|---:|---|---|---|
| 16 | accept（診断ゼロ） | ok | hello / 算術 / 語の定義 / prelude（制御構造）/ fizzbuzz / struct / ベクトル計算 / 型の別名 / 和・列挙 / 引用 / times / while / 局所束縛 / macro / ライブラリとその利用側 |
| 1 | **reject** | n/a | `17-bad-syntax.stax`（スタック効果の `)` が無い） |
| 1 | accept + **WARNING 2 件** | error | `18-undefined-word.stax`（未定義の語と前方参照） |
| 2 | accept（診断ゼロ） | **error** | `19-type-mismatch.stax`（`print` に Int）/ `20-effect-mismatch.stax`（宣言した効果と本体が不一致） |

`crosscheck.sh` は 20 本すべてについて、**受理（accepted / allConsumed）・canonical AST・
スコープ診断**が Rust backend と Java backend で一致することを検査する。
診断の `expected` ラベルと主 offset は言語別 oracle（D-011 / D-014）なので比較しない。

## この文法が使っている UBNF の機能

詳細と「使わなかったもの」は [`DESIGN.md`](DESIGN.md) §7。要点だけ:

| 機能 | なぜ要るか |
|---|---|
| **語境界**（D-040。v2 既定） | 組込み語 29 本と組込み型 4 本を**リテラルで**並べている。`dup` が `duplicate` を、`Int` が `Integer` を横取りしないことが設計の前提 |
| **到達不能候補の検査**（D-037） | 組込み語 29 本が 1 つの順序付き選択。順番を間違えると候補が黙って死ぬので `--strict` 警告ゼロを条件にしている |
| `@scopeTree` / `@declares`（8 箇所）/ `@symbolRef`（4 箇所） | 「語を定義する → 以後の参照が解決する → 未定義は WARNING」が**文法だけ**で出る。記号表のコードは 1 行も無い |
| `@mapping`（27 箇所） | typed AST。選択だけの規則にも付けて sealed interface + record の和型にしている |
| `@catalog`（6 箇所）/ `@doc`（17 箇所） | LSP の補完カタログと hover（stage 4） |

**左再帰（D-039）とパラメータ化ルール（D-043）は 1 度も使っていない。**
連接言語には演算子の優先順位が無いので、UBNF v2 の目玉である優先順位塔の除去が
そもそも要らない。P4（式言語）・JSON（データ形式）と性格の違う 3 本目の例になっている。

## 両 backend の差として見つかったこと

`NUMBER` token を `@mapping` の field へ**直接** capture すると、Java backend は
`int` へ変換し、Rust backend は字句の文字列を返して canonical AST が食い違っていた
（当初は `( NUMBER ) @text` と group を挟んで回避）。issue #75 / D-074 で
**token の直接 capture は両 backend とも原文の文字列**に揃えたので、いまは
`NumLit ::= NUMBER @text ;` とそのまま書いている。canonical AST の golden は
[`golden/`](golden/)（`scripts/crosscheck.py` が両 backend をこれと突き合わせる）。

`@recovery` を順序付き選択の末尾でない候補から到達する規則（`WordDef`）に置くと、
後続候補（`StructDef` など）のための入力でも回復が成功を返してしまう。
これは #71 / D-073 で `validate` の警告 `W-RECOVERY-IN-CHOICE` になった
（`--strict` で exit 5）。[`DESIGN.md`](DESIGN.md) §7.1。

## 次の段

stage 2 ジェネリクス（行変数）/ stage 3 Rust インタプリタ /
stage 4 LSP・VSIX / stage 5 自己拡張の境界レポート → UBNF v3 提案。
それぞれの受け入れ条件は issue #66 と [`DESIGN.md`](DESIGN.md) §9。
