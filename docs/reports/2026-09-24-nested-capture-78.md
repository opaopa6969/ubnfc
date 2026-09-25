# mixed な選択の text 候補と入れ子の同名 capture（#78 → D-078）

直接 capture の帯（#75、`DIRECT_CAPTURE_SEED_BASE` = 1,000,000）の seed **1000002** で、`Direct` 規則と関係のない
乱択部分が `diff:/ast/fields/tail/*` を出した。原因は入れ子の同名 capture の順序ではなく、
**意味値を持つ選択の中にある text だけの候補**の値の作り方だった。Rust が直接の逸脱で、掃き出しの途中で
Java にも別方向の逸脱が見つかった。両方を D-078 にそろえた。

## 1. 再現

`cargo run -p ubnfc-fuzz --offline -- --start-seed 1000002 --seeds 1 --inputs 12` の縮小結果
（`crates/ubnfc-fuzz/findings/diff-ast-fields-tail-bf11b609-seed1000002/`）:

```ubnf
@root @mapping(Top, params=[head, tail, v14, v15, direct])
Root ::= Op0 @head 'あ' @head ( ',' @tail ( M1+ @v14 | '=' @v15 )? )* @tail Direct @direct;
```

入力 `iあ,=0:1''` での capture の出現（`captures`）は両 backend で同じ
（`tail` `,` [2,3] → `v15` `=` [3,4] → `tail` `,=` [2,4]）。違うのは外側 `@tail` の出現の**値**だけだった。

| | `tail` | `v15` |
|---|---|---|
| Java | `[",", "="]` | `["="]` |
| Rust（修正前） | `[","]` | `["="]` |

## 2. 契約の読み

- 外側 `@tail` は `( … )*` への capture なので、**反復の各要素**が 1 出現になる（`nested-capture-migration.md` の表、D-032）。
  出現順は内側 `','` の `@tail` が先で、外側が後（完了順、§3.6）。ここは両 backend で一致していた。
- 出現の値は要素の**意味値**。連接の中の text（`','`）は区切りなので値にならない（D-032。Java の `SemanticShape` も
  Rust の `semantic_shape` も同じ読み方をする）。
- 残る `( M1+ @v14 | '=' @v15 )?` は、**node の候補と text だけの候補を持つ mixed な選択**。
  旧実装はこの形の text 候補に `#semantic:text` を束縛し、候補が採用されると一致範囲の字句を 1 つの値にしていた
  （`unlaxer-dsl` の `CaptureBindingPlan.markSemanticValues`、`MapperRuleEmitter.semanticValues`。
  `SemanticCardinality.shape` は選択の text 候補を `VALUE` に合流させる）。
- したがって外側の値は `"="` になる。Java は `v15` の値を `tail` へ畳んでいるのではない。
  **同じ字句が別の site の意味値として 2 回現れている**だけで、`('a' @value) @value` → `["a", "a"]` と同じ形。

→ issue の形では **Java が正しく、Rust が逸脱**していた。決定は D-078（`03-design-v1.md` §8）。
corpus で最も近い観測は `conformance-nested-captures-text-node-and-mixed-values`
（`(Leaf @value | 'b' @value) @value` → `["b", "b"]`）だが、これは **scalar 形**の選択だけを扱う。
list 形の mixed 選択の観測は corpus に無かった。

## 3. 原因

- **Rust**: `semantic_shape` は選択の text 候補を数えない（`filter_map` で落とす）。scalar / optional 形の選択は、
  `Values{wrap:true}` の「値が無ければ一致範囲の字句を使う」fallback のおかげで偶然 Java と一致していた。
  一方 `M1+` を含む選択は list 形（`wrap:false`）なので、`'='` が採用されても値が作られない。さらに外側の
  `node_values` は「値の群を含む」出現に字句 fallback を掛けないので、出現の値ごと消えていた。
- **Java**（初回の掃き出しで判明）: 0 回の反復や省略は「値なし」（空の `values`）を返し、`combine` はそれを
  候補全体へ伝える。このため `( '-' '='? )*` の `-` や `[ '-' ] [ 'b'+ ')' ] T1*` のような text 候補が値を失っていた
  （旧実装は字句を返す）。当初の Rust だけの修正はこの形で Rust 側を Java と食い違わせ、標準帯の seed 14
  （空一致の token 候補。Java は `""`、Rust は値なし）が新しい差として出た。そこで両側を直した。

## 4. 修正（D-078）

両 backend で「意味値を持つ選択の、意味値を持たない候補（`@skip` 参照を除く）」を生成時に数え、
候補が採用されたら**一致範囲の字句 1 つ**を値にする。空の一致なら `""`。

- Rust: `Generator.text_alternatives`（`expressions::text_alternatives`）。その候補の式関数と、因数分解された候補 Seq
  （`factor.rs` の `success`）に `Event::Values{text:true}` を置く。`mapping.rs` は `text` のとき子の値を字句 1 つに置き換える。
  #85 の `PackedEvent` では flags の bit 1 に載せる。
- Java: `JavaEmitter.textAlternatives`。その式の包みには意味形 `textAlternative` を渡し、`Session.textAlternative`
  （bypass 経路でも同じ）が `values` を plain text（null）に戻す。すると選択の射影が字句 1 つを作る。
- scalar 形の選択は、候補が作った字句を外側の `Values{wrap}` が同じ範囲の字句で置き換えるので、観測は変わらない。
- 生成物（P4 Rust / P4 Java / self-host）を再生成した。P4 には該当する候補が 9 か所ある。canonical AST golden は変わっていない。

## 5. 回帰試験

- `crates/ubnfc-fuzz/tests/mixed_choice_text_value.rs`（新規）では、両 backend を突き合わせたうえで値も固定する。
  対象は issue の形（`,=` → `tail: [",", "="]`、`,=,x,,=` の混在）、list 形の選択の直接 capture（literal 1 つ / 連接）、
  scalar 形、因数分解される連接候補、空一致の token 候補（`""`）、空反復（`""` / `"=="`）、内側の 0 回省略（`-` / `-=-`）。
  Rust の修正を外すと issue の形で落ちることを確認した。
- `tests/campaign.rs` の g に seed 1000002 を戻した（固定 seed は 5 本）。
- 所見 `diff-ast-fields-tail-bf11b609-seed1000002` を `crates/ubnfc-fuzz/findings/` に置いた。

## 6. 掃き出し

条件は各帯 201 seed × 12 入力、`--shrink-builds 0`。修正前（B = `origin/master` cefdbd7）と修正後（F = この PR）を比べた。
fuzz は signature ごとに最初に見つかった seed だけを記録する。

| 帯 | 比較 / 全受理 | 消えた signature（B の seed） | 増えた signature |
|---|---|---|---|
| 直接 capture v1（1000000–） | 2412 / 513 | なし（`tail/*` の代表が 1000002 → 1000189 に変わった） | なし |
| 直接 capture v2（1100000–） | 2412 / 246 | `head/*/fields/head/*`（1100156）、`head/*/fields/head/fields/left/fields/left/*`（1100039） | なし |
| 標準 v1（1–） | 2412 / 646 | `head/*/fields/right/*`（168）、`head/fields/left/fields/left/*`（38）。`head/*` の代表は 9 → 84（84 は B でも出る） | なし |
| 標準 v2（100001–） | 2412 / 467 | `head/fields/head/fields/left/*`（100181） | なし |

- 同じ原因のものとして、1000002 / 1000004（`tail/*`）、1100039（Rust が空反復の候補に `""` を作っていた）、
  9（Java が内側の 0 回省略で `-SELECTSELECT` を落としていた）は、個別に再実行して差が消えたことを確かめた。
- **別原因で残るもの**: 1000189 の `tail/*` は B でも出る既存の差。`L1? imp.Piece+` の候補が `L1` なしで一致すると、
  Rust だけが Optional 形の `Values{wrap}` fallback で字句 `"i"` を作る（Java と旧実装は値なし）。これは issue #86 に切り出した。
  ほかの残りの signature（`diff:/ast` + `mappingError`、`captures/*`、`diagnostics/*`、`head/fields/left` の引用符など）は
  B と F で代表 seed が同じで、この修正とは関係がない。

## 7. 検証

`cd ubnfc-java && mvn -o -q -DskipTests package` を先に実行した。

| 項目 | 結果 |
|---|---|
| corpus Java | **Pass 1,310** / 期待拒否 Pass 27 / Known 29 / **Fail 0** / 未観測 59（不変） |
| corpus Rust | **Pass 1,292** / 期待拒否 Pass 29 / Known 29 / **Fail 0** / 未観測 75（不変） |
| canonical JSON golden 13 件（`java_and_rust_goldens_are_identical_without_normalization`） | 不変・緑 |
| 生成物の byte 一致（`p4_ir_regenerates_every_committed_byte`、self-host `unmodified_golden_regenerates_every_committed_byte`、P4 Java `generate.sh --check`） | 再生成後に緑 |
| self-host 固定点（`generated_frontend_reaches_the_ir_and_rust_fixed_point`） | 緑 |
| `cargo test --workspace --offline --no-fail-fast`（固定 seed campaign a〜g と新規の targeted を含む） | 緑 |
| `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --check` | 緑 |
| `mvn -o test` / `mvn -o -Plsp test` | 緑 |
