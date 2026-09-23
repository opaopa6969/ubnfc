# UBNF v2 — バージョン門と左再帰ルール（D-038 / D-039）

> 2026-09-23。`docs/design/06-ubnf-v2.md` §0・§1 の実装。
> 対象: `crates/ubnfc-front/src/desugar.rs`、`examples/p4-v2/`、両 backend のバージョン門。

UBNF の最大の弱点は「左再帰の式文法を、人間が優先順位塔へ手で展開しなければならない」ことだった。
v2 では ANTLR4 と同じ「候補順＝優先順位」の左再帰ルールが書けて、**塔は compiler が書く**。

## 1. 表層構文と、そう決めた理由

```ubnf
grammar G {
  @ubnf: 2
  ...
  @mapping(Binary, params=[left, op, right])
  Expr ::=
      @rightAssoc Expr @left '^' @op Expr @right   // 先頭＝最高優先、右結合
    | Expr @left MulOp @op Expr @right
    | Expr @left AddOp @op Expr @right
    | '(' Expr ')'
    | NUMBER ;
}
```

| 決めたこと | 採った形 | 理由 |
|---|---|---|
| バージョン宣言 | grammar 設定 `@ubnf: 1|2`（既定 1、grammar ごとに 1 回） | 設定の値に数字を許すのは v1 では構文エラーだった位置なので純粋な追加 |
| 左再帰の引き金 | **`@leftRec` は要らない**。候補の**先頭要素が自分自身**であることだけ | v1 ではその形は `E-LOWER-D016` で落ちていたので曖昧にならない。注釈より小さい表層 |
| 優先順位 | **候補の並び順**（先頭が最高）。`@precedence` は書かない | ANTLR4 と同じ。塔の段を数字で並べ替える余地を作らない |
| 結合性 | 候補**先頭**の `@leftAssoc` / `@rightAssoc`（既定は左結合） | 設計案の「候補末尾に `@rightAssoc`」は**直前の要素の capture と区別が付かない**（`Expr @left '^' @op Expr @rightAssoc` は最後の `Expr` の capture 名に読める）。先頭は要素が来られない位置なので一意 |
| 段ごとの AST クラス | 候補**先頭**の `@mapping` が rule の `@mapping` を上書きする | P4 の真偽塔は段ごとに `BooleanOrExpr` / `BooleanAndExpr` / `BooleanXorExpr` と別クラス。これが無いと P4 を等価に書き直せない |
| 演算子候補の形 | `Self <演算子 1 要素> Self` の 3 要素だけ | 塔の正規形 `Base @left { Op @op Right @right }` へ 1 対 1 で写せる形。ほかは黙って別物を作らず拒否する |
| 段の名前 | 入口＝元の名前（最低優先）、中間＝`Name__lr{n}`、基底＝`Name__lrBase` | 入口の名前が変わらないので、ほかの規則からの参照も `@root` も診断の入口名もそのまま |

## 2. lowering と等価性

### 2.1 どこで展開するか

**resolve より前の構文木変換**（`crates/ubnfc-front/src/desugar.rs`、`pipeline::prepare_parsed` から呼ぶ）。
resolve / validate / lower / analysis / backend は **v1 の正規形しか見ない**。
`SourceInput` にも脱糖後の AST を入れるので `E-LOWER-SOURCE-STALE` の検査とも一致する。

候補が N 個の演算子候補と M 個（M ≥ 1）の基底候補からなるとき、深さ d = 0..N の規則を作る:

```
T_0  = 元の名前                       … 演算子候補 N-1（最低優先）
T_d  = Name__lr{d}          (1≤d<N)   … 演算子候補 N-1-d
T_N  = Name__lrBase                   … 基底候補をその順で並べた選択

左結合の段:  T_d ::= T_{d+1} @left { Op @op T_{d+1} @right }
右結合の段:  T_d ::= T_{d+1} @left { Op @op T_d     @right }
```

右結合の形は既存の `unlaxer-dsl/src/test/resources/right-associative/Power.ubnf` と同じ。
`@mapping` は演算子の段だけに付き（候補先頭の `@mapping` があればそちらが勝つ）、
基底の段には付かない。`@leftAssoc` / `@rightAssoc` と `@precedence(level=(d+1)*10)` は
capture が `left` / `op` / `right` のときだけ付ける（既存の operator metadata の正規形に一致する場合だけ）。

span は候補の span を使う（段ごとに違うので ExprId が衝突しない）。合成した規則の注釈は
候補の先頭の 0 幅 span に置く（lower が「最後の注釈の直後」を symbol の開始位置にするため）。

### 2.2 なぜ precedence climbing にしなかったか

D-004 の `climb(min)` / `lift` は**塔のノードを復元する**ための設計で、既存の AST 契約
（段ごとに 1 ノード、`left + op[] + right[]`）を保つのが目的だった。ところが
`crates/ubnfc-front/src/analysis/operators.rs` は P4 の 6 塔すべてを climbing **不適格**と判定する
（到達領域に trivia・scope・recovery・先読みがある）。`docs/reports/2026-09-22-p4-rust-perf-m5-round2.md`
§手順1 も、塔の形では climbing に移して得るものが無いと結論している。

つまり **climbing は「塔をどう速く実行するか」の話であって、v2 が消したい弱点ではない**。
消したい弱点は「塔を人間が書くこと」なので、**塔を機械が書く**のがいちばん小さい解になる。
副作用として、v1 の実行・AST・診断・memo・trivia の機械を 1 行も触らずに済み、
等価性が「同じ IR を作る」という形で構成的に保証される。

「演算子適用ごとの二項ノード」という AST も検討したが、それは両 backend に新しい AST 構築
recipe（リストを左畳み込みして入れ子の二項ノードを作る）を足すことになり、corpus 1,212 ケースの
リスクに見合わない。**第一段では採らない**（§8 の宿題）。

### 2.3 等価性の議論

脱糖は構文木から構文木への写像で、次を保つ:

1. **受理言語**: PEG の順序付き選択と `{ }` の意味論のもとで、候補順の塔は
   「高優先の演算子ほど内側で結合する」左再帰文法の標準的な展開そのもの。
   v1 で同じ塔を手で書いた場合と**同じ Expr 木**になる（規則名だけが違う）。
2. **AST**: 段ごとの `@mapping` と capture（`left` scalar / `op` list / `right` list）が
   手書きの塔と同一なので、`semantics.astTypes` と recipe（種別・結果型・field の基数と変換）が
   一致する。`crates/ubnfc-front/tests/ubnf_v2.rs::the_desugared_tower_matches_a_hand_written_v1_tower`
   がこれを固定している（`astTypes` は**完全一致**）。
3. **診断**: 同じ Expr 木を実行するので、D-011 / D-014 の契約側
   （`kind` / 主 `offset` / `farthestOffset` / `expected` / `farthestExpected`）は変わらない。
   規則名そのものを値に持つ `deepestRule` / `rulePath`（D-020 が「情報。Verdict に使わない」と
   決めている）と、合成 helper の表示名（`NumberTermRepeat0Parser` ↔
   `NumberExpression__lr1Repeat0Parser`、D-027 の「文法に現れない合成 parser の表示名」）だけが
   段の名前ぶん変わる。§3 で実測した。

## 3. 第一段で拒否するもの

「黙って別物を作る」より「はっきり落とす」を選んだ。すべて validate 段のエラーコード。

| コード | 何を拒否するか | なぜ |
|---|---|---|
| `E-LEFTREC-ALT-SHAPE` | `Self <演算子> Self` の 3 要素でない演算子候補（前置 `'-' Expr` を演算子として扱う形、後置、三項 `Expr '?' Expr ':' Expr`、単独 `Expr`） | 塔の正規形 `Base @left { Op @op Right @right }` に写せない。capture の投影と AST の形が未定義 |
| `E-LEFTREC-NO-BASE` | 基底候補が 1 つも無い | 何も受理しない規則になる |
| `E-LEFTREC-NULLABLE-OP` | 演算子要素に `?` / `*` / `{0,n}` | 反復が空一致して止まらない |
| `E-LEFTREC-RIGHTASSOC-OP-REF` | **規則参照**を演算子にした右結合候補 | v1 の右結合正規形が `lower/annotations.rs::operator_elements`（`Base @left { Op @op Self @right }`）と `analysis/operators.rs::canonical`（`Base Op Self \| Base`）で食い違っており、演算子が規則参照のときだけ operator metadata が作られてこの穴が表に出る（同じ文法は v1 でも落ちる）。演算子をリテラルか括弧つき選択で書けば通る |
| `E-LEFTREC-ANNOTATION` | 左再帰ルールに付けた `@scopeTree` / `@declares` / `@backref` / `@recovery` / `@catalog` / `@skip` / `@enum` / `@eval` / `@leftAssoc` / `@precedence` / `@longestChoice` / `@predictiveChoice` など | どの段に投影するかが未定義。`@root` / `@mapping` / `@doc` / `@whitespace` / `@interleave` は定義済みなので通す（`@root` は入口の段、`@mapping` は演算子の段、残りは全段） |
| `E-LEFTREC-NAME-COLLISION` | `Name__lr{n}` / `Name__lrBase` が既存の規則・token と衝突 | 脱糖が既存の定義を上書きしない |
| `E-LEFTREC-ALT-ANNOTATION` | 候補先頭に `@leftAssoc` / `@rightAssoc` / `@mapping` 以外を置いた、または演算子候補でない候補に候補先頭注釈を置いた | 候補に意味を持つ注釈はこの 3 つだけ |
| `E-ALT-ASSOC-UNSUPPORTED` | v1 の文法に置いた候補先頭注釈 | v1 は凍結。v2 の左再帰ルール以外では意味を持たない |
| `E-UBNF-VERSION-UNSUPPORTED` / `E-UBNF-VERSION-DUPLICATE` | `@ubnf` が 1/2 以外、または 2 回 | バージョン門（D-038） |
| `E-LOWER-INDIRECT-LEFT-RECURSION` | 間接左再帰（既存） | ANTLR4 も拒否する。脱糖後の段は左再帰でないので既存の検査がそのまま効く |

`@typeof` を演算子候補の要素に付けた場合も `E-LEFTREC-ALT-SHAPE` で落ちる（`plain()` 検査）。

**警告**: 演算子候補でない候補の末尾に自分自身が来る形（`'-' Expr`）は、その `Expr` が
**式全体＝最低優先の段**を指す。v1 で塔の基底に同じものを書いたときと同じ意味なので受理するが、
ANTLR 風の「前置演算子にも優先順位がある」意味ではないので `W-LEFTREC-TRAILING-SELF-REF` で知らせる。

## 4. P4-v2 — 実際の文法での効果

`examples/p4-v2/tinyexpression-p4-v2.ubnf`。式の塔以外は tinyexpression の P4 と 1 文字も同じ。

| | v1（手書きの塔） | v2 |
|---|---:|---:|
| **規則数** | **124** | **119**（−5、−4.0%） |
| 式の塔にあたる規則 | 8 | **3** |
| IR の規則数（脱糖後） | 124 | 125 |

書き換えたのは 3 つの塔:

- 数値: `NumberExpression` / `NumberTerm` / `NumberFactor` → `NumberExpression` 1 本（演算子候補 2 + 基底候補 13）
- 文字列: `StringExpression` / `StringTerm` → `StringExpression` 1 本（演算子候補 1 + 基底候補 16）
- 真偽: `BooleanExpression` / `BooleanAndExpression` / `BooleanXorExpression` → `BooleanExpression` 1 本（演算子候補 3 + 基底候補 1）
  （段ごとに候補先頭 `@mapping`。`BooleanFactor` は自前の `@mapping(BooleanFactorExpr)` を持つので残す）

**正直に書くと、行数の削減は劇的ではない**（−5 規則）。設計文書の「同じ塔を 4 回書いている」は
実際より大きく言っていて、P4 の塔は 3 つ・合わせて 8 規則である。効いているのは量ではなく
**優先順位が宣言的になったこと**で、段を足す・順番を変える操作が「候補を 1 行足す / 動かす」になった。
v1 では新しい段を入れるたびに規則を 1 本作り、上下の段の参照を両方書き換える必要があった。

IR の規則数が 1 増えるのは、真偽塔の基底候補が `BooleanFactor` という規則参照 1 個なので
`BooleanExpression__lrBase ::= BooleanFactor` という**透過的な 1 段**ができるため。
AST も値も作らないので観測には出ない（`rulePath` にだけ現れる）。

### 4.1 IR の照合（codegen 不要、`cargo test` に載っている）

`ubnfc ir` で両方の IR を作って突き合わせた結果:

- `semantics.astTypes`: **87 型すべてが byte 一致**（v1 にだけ / v2 にだけある型は 0）
- recipe: v1 の `NumberExpression` / `NumberTerm` / `StringExpression` /
  `BooleanExpression` / `BooleanAndExpression` / `BooleanXorExpression` の
  (種別, 結果型, field の基数・変換) が、v2 の
  `NumberExpression` / `NumberExpression__lr1` / `StringExpression` /
  `BooleanExpression` / `BooleanExpression__lr1` / `BooleanExpression__lr2` と 1 対 1 で一致
  （すべて `assocLists`、`left` scalar / `op` list / `right` list）
- `languageVersion`: v1 = 1、v2 = 2
- `ubnfc validate` の警告: **v1 と v2 で完全に同じ 9 件**
  （`W-CHOICE-SHADOWED-PATH`。D-037 の到達不能候補検査が `BooleanComparable` /
  `BranchExpression` / `Expression` / `ObjectExpression` の候補順について出すもの）。
  左再帰ルールへの書き換えで新しい死んだ候補が生まれていないことの裏付けになる。

### 4.2 生成した parser の観測比較

`examples/p4-v2/scripts/compare.sh`。両方の IR から Rust backend で parser を生成し、
`--release` で build して、同じ入力を両方の driver（`docs/harness-driver-protocol.md`）に投げる。

入力は 16 個の fixture（`examples/p4-rust/tests/fixtures/*.tiny`。tinyexpression の
`benchmarks/fixtures/*.tiny` と md5 一致を確認した）＋ 診断契約を突く 12 個の拒否入力
（`1 + `、`1 * * 2`、`(1 + 2`、`(1 * 2`、空文字、`1 + @`、`$a & `、`$a ^ `、`'a' + `、
`var $a set 1 + ; $a`、`1 + 2 garbage`、`(1 + 2) * `）。

**正規化は段の名前の読み替え 1 つだけ**（`compare.py` の `TIER_NAMES`）:

```
NumberExpression__lr1    -> NumberTerm
NumberExpression__lrBase -> NumberFactor
StringExpression__lrBase -> StringTerm
BooleanExpression__lr1   -> BooleanAndExpression
BooleanExpression__lr2   -> BooleanXorExpression
BooleanExpression__lrBase -> （rulePath から除去。v1 に対応規則の無い透過的な 1 段）
```

対応は脱糖の定義（候補順＝優先順位、最深が基底）から一意に決まる 1 対 1 で、
文字列として現れるところ（`deepestRule` / `rulePath` / `farthestExpected` の合成 parser 名）を
置換する。`farthestExpected` は昇順の集合なので置換後に並べ直す。

結果:

| 比較 | 結果 |
|---|---|
| **AST（正規化なしの生比較）** | **28 / 28 一致**（AST JSON に規則名が出ないので読み替え不要） |
| accepted / consumed / matched / allConsumed / fullConsumed / fullMatched | 28 / 28 一致 |
| diagnostics（段名の読み替えのみ） | 28 / 28 一致 |
| **応答全体**（`captures` / `value` / `mappingError` / `scope` / `recoveries` / `catalogs` を含む） | **28 / 28 一致** |

読み替え前に残っていた差は次の 3 種類だけで、すべて規則名そのものだった:

- `diagnostics[].rulePath`: 18 ケース（段の名前 + 透過的な 1 段）
- `diagnostics[].deepestRule`: 1 ケース（`complex-half.tiny`: `NumberFactor` → `NumberExpression__lrBase`）
- `diagnostics[].farthestExpected` の合成 helper 名: 5 ケース
  （`NumberTermRepeat0Parser` → `NumberExpression__lr1Repeat0Parser`、
  `NumberTermParser` → `NumberExpression__lr1Parser`）。D-027 が「文法に現れない合成 parser の
  表示名は言語別 hints」と決めている類。

**主 `offset` / `farthestOffset` / `kind` / `expected` / `severity` / `message` は読み替え前から
1 件も違わなかった。** 診断契約でいちばん壊れやすい部分（どこまで到達したか）が保たれている。

## 5. 性能

同じ harness（`examples/p4-rust/examples/bench.rs` と同じ方式: warmup 後 5 run の中央値、
1 run は `UBNFC_BENCH_RUN_SECONDS=1.0` ぶんの反復）。v1 / v2 を 3 ラウンド交互に回した中央値。
`scripts/compare.sh --timing`。

| 入力 | mode | v1 (ms/parse) | v2 (ms/parse) | v2/v1 |
|---|---|---:|---:|---:|
| complex.tiny | 認識 | 0.0698 | 0.0703 | 1.007 |
| complex.tiny | AST | 0.0920 | 0.0962 | 1.046 |
| complex-x4.tiny | 認識 | 0.2783 | 0.2826 | 1.015 |
| complex-x4.tiny | AST | 0.3610 | 0.3610 | 1.000 |
| complex-x16.tiny | 認識 | 1.1328 | 1.1421 | 1.008 |
| complex-x16.tiny | AST | 1.4440 | 1.4635 | 1.014 |
| complex-x64.tiny | 認識 | 4.7770 | 4.9499 | 1.036 |
| complex-x64.tiny | AST | 6.5313 | 7.5588 | 1.157 |
| flat-arithmetic.tiny | 認識 | 0.1052 | 0.1100 | 1.046 |
| flat-arithmetic.tiny | AST | 0.1835 | 0.1942 | 1.058 |
| large-match.tiny | 認識 | 0.3825 | 0.3989 | 1.043 |
| large-match.tiny | AST | 0.5179 | 0.5496 | 1.061 |

**v2 は 0〜6% 遅い**（`complex-x64` の AST だけ +15.7%）。原因は算法ではなく
**透過的な 1 段**（`BooleanExpression__lrBase ::= BooleanFactor`）の規則呼出しが 1 段増えることで、
v2 が実行するのは v1 と同じ塔なので、それ以外に差が出る理由が無い。
`complex-x64` の AST は別ラウンドで v2 が速いことも v1 が速いこともあり
（測定した個々のラウンドで 7.27 ms / 9.01 ms など）、共有マシンでの雑音の幅が大きい。
**この段は「基底候補が規則参照 1 個のときは段を作らない」で消せる**（§8 の宿題）。

## 6. v1 が変わっていないことの確認

| 検査 | 結果 |
|---|---|
| `cargo test --workspace --offline` | 成功（FAILED 0）。`ubnf_v2.rs` 13 本・`language_version.rs` 2 本を含む |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | 警告なし |
| `cargo fmt --all -- --check` | 差分なし |
| `cd ubnfc-java && mvn -o -q test` | 成功 |
| `cd examples/p4-java && mvn -o -q test` | 成功 |
| self-host 固定点（`crates/ubnfc-selfhost`） | 成功（`regenerate` / `parity`） |
| `cargo test -p ubnfc-fuzz`（jar を先に package） | 成功（campaign 3 本） |
| corpus Java（`run-all --backend java`） | **Pass 1310 / Fail 0**（期待拒否 Pass 28、Known 28、未観測 59）。系統別の表は `docs/reports/2026-09-23-java-corpus-contract.md` と**完全一致**。報告 `docs/reports/2026-09-23-java-corpus-leftrec.{md,json}` |
| corpus Rust（`run-all --backend rust`） | **Pass 1292 / Fail 0**（期待拒否 Pass 30、Known 28、未観測 75）。系統別の表は `docs/reports/2026-09-23-rust-corpus-contract.md` と**完全一致**。報告 `docs/reports/2026-09-23-rust-corpus-leftrec.{md,json}` |

v1 の凍結は構造で保証している:

- 表層に足した構文はすべて「v1 では構文エラーだった位置」（設定値の数字、候補**先頭**の注釈）。
- 脱糖は `@ubnf: 2` を宣言した grammar でだけ走る。v1 の grammar に対しては
  「候補先頭注釈が無いこと」を検査するだけで AST を変えない。
- `@ubnf: 2` を宣言しても v2 の機能を使わなければ `semantics` / `analysis` / `plan` は
  1 バイトも変わらない（`ubnf_v2.rs::language_version_defaults_to_one_and_is_carried_in_the_ir`）。
- backend は `languageVersion` を**読んで拒否するだけ**で、生成物は変わらない
  （`crates/ubnfc-rust/tests/language_version.rs` が v1 と v2 の生成物の byte 一致を固定）。

IR の変更は `languageVersion`（必須キー）1 つだけ。既存フィールドは 1 つも変えていないが、
必須なので IR golden 22 本・`ir/examples/tinycalc.ir.json`・両 backend のテスト用 IR 23 本を
再生成（挿入 1 行のみ）した。

## 7. 差分ファジングの拡張

`crates/ubnfc-fuzz` の文法生成器に v2 の左再帰ルールを足した。

- 既存 seed の生成物を 1 バイトも変えないため、**v2 は乱数を引かず seed の値だけで決める**:
  `seed >= gen::V2_SEED_BASE`（100,000）の帯でだけ `@ubnf: 2` を出す。
  seed 269（A-1 の回帰試験）と 4001..4004（CI の固定 campaign）は無傷。
- v2 の帯では、演算子塔を「候補順＝優先順位」の左再帰ルール 1 本で書く。
  段ごとの AST クラスは**候補先頭の `@mapping`** で分ける（v1 の塔の `Bin{step}` と同じ形。
  1 クラスを共有すると `E-LOWER-SHARED-SCHEMA` で落ちる文法が出る）。
- `model::Element` に `lead`（候補先頭の注釈）を足した。縮小は `lead` を落とす候補を出す。
- 新しい固定 seed campaign `c_v2_left_recursive_grammars_agree_across_backends`
  （seed 100,001..100,003、入力 10 個）。**discard 0、driver error 0、未登録の差分 0**。
- 帯 100,000..100,120 の左再帰文法 78 件すべてが front を通る（discard 0）ことも確認した。

**未着手**: v2 の帯は左結合の二項演算子しか出さない。右結合（`@rightAssoc`）と
「基底候補が複数ある左再帰ルール」の組合せは fuzzer が作らない。前者は
`crates/ubnfc-fuzz/src/gen.rs` の v2 ブロックで `lead` に `@rightAssoc` を足すだけで出せるが、
`E-LEFTREC-RIGHTASSOC-OP-REF` と衝突しないよう演算子をリテラルに限る必要がある（今もリテラル）。

## 8. 残っている宿題

1. **透過的な 1 段を作らない**: 基底候補が「量指定子・capture の無い規則参照 1 個」のときは
   `Name__lrBase` を作らずその規則を直接参照する。§5 の 0〜6% と IR の規則数 +1 が消える。
2. **前置・後置・三項の候補**（`E-LEFTREC-ALT-SHAPE`）。ANTLR4 風の前置演算子の優先順位を
   入れるなら、段の形が `T_d ::= <prefix> T_d | T_{d+1}` になり、`@mapping` の投影（片方の候補に
   capture が無い）を決める必要がある。
3. **右結合の正規形の食い違い**（`E-LEFTREC-RIGHTASSOC-OP-REF`）。`analysis/operators.rs::canonical`
   に `Base @left { Op @op Self @right }` を右結合として足せば消えるが、`Power.ubnf` の
   OperatorPlan（現在 `TierFunctions` の noncanonical）が変わるので v1 の観測影響を測ってから。
4. **演算子適用ごとの二項ノード**という AST 形。両 backend に「`left + op[] + right[]` を
   左畳み込みして入れ子の二項ノードにする」recipe を足す話で、v1 の塔とは別の AST になるので
   v2 の中でも opt-in（`@mapping` の形で選ぶ）にするのが筋。
5. **`@scopeTree` / `@recovery` を持つ左再帰ルール**。段ごとに scope を張るのか入口だけかを
   決める必要がある（現状は `E-LEFTREC-ANNOTATION`）。
6. **self-host**: `unlaxer-dsl/grammar/ubnf.ubnf` は v2 の表層構文（設定値の数字、候補先頭注釈）を
   まだ受理しない。self-host 固定点は v1 の文法だけを対象にしているので今は無傷だが、
   v2 の文法を self-host 経路で読むには ubnf.ubnf 側の更新が要る。
