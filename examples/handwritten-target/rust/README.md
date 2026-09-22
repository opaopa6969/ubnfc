# Rust 目標生成物: Operators / Power

UBNF から将来生成する保守的な Rust コードを人手で記述したもの。
設計の正は [v1 §3 / §5 / §6](../../../docs/design/03-design-v1.md)。
既存 parser / generator を実行・リンクせず、文法と既存テストの契約を読み、
静的な rule 関数、byte カーソル、memo、採用後の AST 構築として実装した。
この例は M1 の演算文法部分であり、scope annotation、longest、recovery 等の
文法に現れない機能を検証したものではない。

`src/operators.rs` と `src/power.rs` が生成物の本体。
`src/support.rs` は静的に共有できる scanner / recipe / memo 処理、
`src/api.rs` は結果型と AST の評価器。
`src/rt/{input,cursor,scope,memo,diag}.rs` は基点 `54519c6` の
`crates/ubnfc-rt-template/src/` から**無変更でコピー**した5ファイルで、
`include!` や runtime crate 依存を使わない。
`src/canonical.rs` の出力アダプターだけが serde / serde_json を使用する。
parser、API、support、rt は `rustc --edition=2021` 単独でもコンパイル・実行を確認した。

```rust
use ubnfc_target_rust::{canonical, operators, power, ParseOptions};

let result = operators::parse("1+2*3");
assert!(result.ok);
assert_eq!(result.ast.as_ref().unwrap().evaluate(), 7.0);
println!("{}", canonical::json(result.ast.as_ref().unwrap()));
assert_eq!(power::parse("2^3^2").ast.unwrap().evaluate(), 512.0);

let prefix = operators::parse_with_options("1+", ParseOptions {
    build_ast: true, require_eof: false,
});
assert!(prefix.ok);
assert_eq!(prefix.consumed_cp, 1);
```

## 結果・評価の契約

- `parse` は EOF を要求する。`require_eof: false` で PEG の prefix 成功を返す。
  EOF 不一致でも両カーソルは成功 prefix の終端を示し、root 自体の失敗なら `(0,0)`。
- `ok` は構文結果で、mapping 失敗なら `ok=true, ast=None` と `Mapping` 診断を返す。
  認識のみの場合も `ast=None` なので、区別には指定 options と診断の kind を使う。
- 内部 span は byte、公開 cursor / node span / capture / 診断位置は code point。
  `Input` は入力ごとに変換索引を一度構築し、ASCII は追加索引なし。
- canonical は `{type, span:[s,e], fields:{...}}`、text は JSON string、反復は array。
  `Number.value` は Rust の字句保持契約に従う文字列。評価用 `f64` と node ID は出力しない。
  map のキー順は harness `canonical.rs` と同じ serde_json の既定順。
- Operators は `left` から `op[]` / `right[]` を左 fold する。`/` は浮動小数点除算。
  `24/(3/2)=16`、`1e2/4-0.5=24.5`。Power は右再帰を評価し `2^3^2=512`。
- 構文診断は consumed 基準の最遠失敗位置、初出順・重複なしの expected、
  その位置で最深の rule path を保持する。同深度は先勝ち。
  成功した選択・反復内の失敗も残すため、`ok=true` でも構文診断があり得る。
  trivia 内の不一致は診断を出さない。rule 名には生成 helper と NUMBER も含む。
- `captures` は採用結果の完了順。`node_spans[node_id]` は AST の各 mapped node に対応する。
  mapping 失敗時も capture は残し、部分 AST / node_spans は返さない。
  lexical 出現は任意出力なので、この例では未実装。

## バックエンドへ渡す生成規則

1. **trivia は chain の挿入計画から出す。** 有効 policy は rule `@whitespace` >
   rule `@interleave` > grammar。呼出元からの動的継承ではない。
   chain `[e1,...,en]` は `SPACE e1 SPACE ... en SPACE` となり、
   一要素の rule / group helper でも同じ。複数候補の Choice 本体と一要素の候補には
   無条件の skip を足さず、複数要素の候補だけを chain にする。
   この文法の repeat helper も `SPACE Op@op SPACE Base@right SPACE`。
   optional の単一 literal は直接照合し、親 chain がその前後を処理する。
   javaStyle の集合は ASCII space / TAB / LF / VT / FF / CR、`//` 行コメント、
   非 nested `/*...*/`。閉じない block comment は一回分を巻き戻す。
2. **span と capture の境界を token 終端へ縮めない。**
   `NumericLiteral ::= (NUMBER) @value` は外側 chain と group helper の二重境界を持つ。
   `@value` は helper の後続 trivia まで含む。文字列化時の trim は U+0000〜U+0020。
   `1 /*inner*/ +2` の value は `"1 /*inner*/"` となる。
   ` 1 + 2*3 ` の root は `[0,9]`、左 Term / Number は `[1,3]`、右 Term は `[5,9]`。
3. **rightAssoc の recipe は右部分木を保持する。** 正規化は `Base Op Self | Base`。
   第1枝は chain、fallback は Base への直接参照。各段の recipe は
   `Power { left, tail: Option<(operator_span, right_recipe)> }` とし、採用後に
   `op[]` / `right[]` を各0または1要素へ展開する。反復を flat な左 fold へ変えない。
   内側 `right` capture を外側より先に完了させる。
   ` 2 ` の mapped left は `[0,3]`、` 2 ^ 3 ` の mapped left は `[1,3]`。
4. **数値変換は採用後の mapper に置く。** 認識中は group の `value_span` と
   NUMBER の `token_span` を recipe に記録するだけ。採用 root から辿ったときだけ
   value を文字列化し、token の slice を `NumberMapper: fn(&str)->Option<f64>` へ渡す。
   既定は f64。コメント付き group の評価は既存 Rust probe の
   `split('/').next().trim().parse()` と同じ数値になるが、ここでは記録済み token 範囲を使う。
   i32 変換 callback による overflow / 小数 / 指数の失敗で、構文成功との分離を検証する。
5. **memo キーは全項目を保持する。** `(expr_id, consumed, matched, mode, state_version)`。
   全 grammar rule、NUMBER、実行する group / repeat / recursive helper を memo 化する。
   mode は AST 有無と Power の文法 variant を含む。この文法群に scope 操作はなく
   `ScopeStore.state_version()` は通常0だが、キーから削除しない。表の寿命は一 parse。
   u32 offset を超える入力は assert で明示拒否し、キーを切り捨てない。
   payload は終了カーソル、recipe ID、capture event DAG、局所診断を保持する。
   memo hit は対象 rule に enter する前に診断を現在の caller stack へ再生する。
   scope 効果を持つ文法へ展開する場合は、enter / leave / declare / reference の
   順序付き再生を payload へ追加する必要がある。
6. **繰り返し出現するコードをテンプレート化できる。** `key→replay→enter→body→leave`、
   chain の trivia 境界、宣言順 Choice と checkpoint、反復 helper、左右結合 recipe、
   Number scanner、採用 recipe の mapping / 出現再生、code point 変換が共通部。
   左結合は `Vec<(op_span,right_recipe)>` をループで積む。
   反復は子が成功した周回を count してから進捗・max を判定し、最後に min/max を検査する。
   ゼロ進捗の成功も一回数えるので `EMPTY{1,2}` と `EMPTY{2,2}` の結果は異なる。
   rollback は採用 event の参照だけを戻し、memo が参照する arena の index を再利用しない。

この段階では FIRST 予測、memo 省略、skip cache、SIMD、precedence climbing を加えていない。
Power の右再帰と AST mapping は Rust の呼出し stack を使う。
深い右結合・括弧への明示 stack 化は、rule / trivia / span 境界を保持する次の最適化候補。

## corpus と検証

読み取り元は sibling `unlaxer-parser`（確認時 `ed4178e`）の
`unlaxer-dsl/src/test/resources/{associative,right-associative}/`。
別配置の場合は `UNLAXER_PARSER_ROOT` で指定する。実行時に serde_json で直接読み、
harness crate や旧 parser には依存しない。

Power の `modes` は受理する派生文法を限定する。既存 `RustConformanceTest` と同じく
`text`、`mapped`、`unicode`、`grouped` の4モードで全28行を評価する。
`mapped` は Atom を `(NUMBER) @value` の Number にし、`unicode` はその前へ `[ '😀' ]`、
`grouped` は `Base ::= Atom | '(' Expr ')'` を加えて Expr の左を Base にする。

- **68テスト**: corpus 52（Operators 24 + Power 28）、追加意味論11、内部規則5。
- corpus の文法・入力組は **136**（24 + 28×4）。それぞれ認識のみ / AST 付きで照合する。
- [golden/operators.json](tests/golden/operators.json) と [golden/power.json](tests/golden/power.json)
  に全136組の canonical AST とカーソルを固定。受理・全消費・value は元 corpus を独立 oracle とする。
- 全 golden を型・span・field の省略なしの木表示で目視確認した。
  左の flat list、右の各段 singleton、括弧による内側 span、コメント保持、
  Unicode code point、mode 不一致と不正入力の null を確認済み。
  Java runtime test の具体的 span と value を `tests/semantics.rs` に独立固定している。
- 通常テストは golden を更新しない。個別のレビューには
  `cargo run -p ubnfc-target-rust --example inspect -- operators ' 1 + 2*3 '` または
  `-- power mapped '2^3^2'` を使う。

```sh
cargo test -p ubnfc-target-rust
cargo clippy -p ubnfc-target-rust -- -D warnings
cargo clippy -p ubnfc-target-rust --all-targets -- -D warnings
cargo fmt -p ubnfc-target-rust
cargo fmt --all -- --check
```

参照した契約は [NumberParser 受理範囲](../../../docs/design/02-facts-tokens-bench-corpus.md)、
旧 repo の `docs/rule-trivia.md`、`ParserGenerator.java:495` 付近と
`ParserRuleEmitter.java` の helper / choice / rightAssoc 展開、
`AssociativeMappingRuntimeTest`、`RightAssociativeMappingRuntimeTest`、
`RustConformanceTest`、両 corpus の `probe.rs.txt`。

## ベンチ

```sh
cargo run --release -p ubnfc-target-rust --example bench
```

`std::time::Instant`、各入力・モードで warmup 100回、100,000回 × 5 run の中央値。
新規 Session / Input / memo の作成、認識、診断、結果の破棄を含む。
AST 付きは capture / recipe / typed AST / node_spans / 数値 mapping も含む。
canonical JSON 直列化と評価器の fold は計測外。`black_box` で入力・options・結果を保持する。
入力は `1+2*3`（5 bytes）、その括弧深さ8版（21 bytes）、`1` を `+` で連ねた512 atom（1023 bytes）。

測定: 2026-09-22、AMD Ryzen 9 7950X、WSL2 Linux 6.18.33.2、
rustc 1.98.1、Cargo の既定 release profile、単一 thread、CPU pinning なし。
既存 Java / Rust parser との速度比較ではなく、この全 memo 経路の初期値。

単位: µs/parse。中央値は次のとおり。

| 入力 | 認識のみ | AST 付き |
|---|---:|---:|
| `1+2*3` | 2.318 | 3.315 |
| 括弧深さ8 | 20.030 | 26.110 |
| 512 atom | 1015.251 | 1061.853 |

全 run（同じく µs/parse、実行順）:

| 入力 | モード | run 1 | run 2 | run 3 | run 4 | run 5 |
|---|---|---:|---:|---:|---:|---:|
| `1+2*3` | 認識 | 2.318 | 2.350 | 2.307 | 2.324 | 2.297 |
| `1+2*3` | AST | 3.315 | 3.302 | 3.290 | 3.422 | 3.420 |
| 括弧深さ8 | 認識 | 20.083 | 20.266 | 19.918 | 19.720 | 20.030 |
| 括弧深さ8 | AST | 30.577 | 27.749 | 25.492 | 25.733 | 26.110 |
| 512 atom | 認識 | 1015.251 | 1011.253 | 1022.528 | 1040.002 | 1005.741 |
| 512 atom | AST | 1085.730 | 1061.853 | 1068.916 | 1055.394 | 1042.946 |
