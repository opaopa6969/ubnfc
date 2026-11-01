# 新 pin の corpus 残差の解消（issue #46 / #45）

4 系統は両 backend で比較 Fail 0。現在の全件実行には、未 merge の #44 が解消する tiny-code-fence の IR 失敗 13 件だけが残る。
#44 の取り込み後に最終の全件結果と CI / merge の結果を更新する。

## 対象と固定した条件

`opaopa6969/ubnfc` の issue #46 に #45 の受け入れ条件を含める。
`fix/issue-46-new-pin-corpus` の専用 worktree で実装し、共有の main 作業ツリーは編集していない。
変更前は `origin/master` の `cd0516f`。`scripts/external-pins.env` は変更していない。

| 外部 fixture | pin |
|---|---|
| unlaxer-parser | `d3ea73e8cfa440df8bdbb70abb29e06259435ba2` |
| tinyexpression | `c736c2c66e4569ebc5e22f1f0c5125a045679d35` |

変更前は Java / Rust とも 127 corpus file、1,279 source case、1,492 case × variant。
両 backend を独立に `run-all --jobs 4` で実行し、IR 失敗、生成・build 失敗、比較 Fail、期待拒否 Pass、Known、未観測を分けた。
変更前の全 case の outcome と差分は同梱の
[java-before.json](2026-11-01-new-pin-corpus-data/java-before.json) /
[rust-before.json](2026-11-01-new-pin-corpus-data/rust-before.json)に保存した。

## 原因と変更

### ゼロ上限の反復

Classic の `docs/zero-repetition.md` に従い、通常の `{0}` / `{0,0}` は子も terminator も実行せず成功する。
begin、内部 trivia、consumed / matched cursor、reset、子の capture と診断に触れない。
Rust の通常経路と純粋な走査は、上限 0 を 1 に読み替える処理を除いた。
LL(1) の証明と表はゼロ上限を子への辺を持たない `empty` 命令にし、runtime の Repeat でも子に入る前に上限を判定する。
Java の通常経路と一括走査も同じ契約に揃えた。

Java の EOF が使っていた歴史的な zero-match check（子を試して失敗だけを受理）は別の経路へ分離した。
これを通常の反復へ読み替えて EOF が常に成功する退行は、既存の EOF 診断テストで防ぐ。
正の上限と nullable な子の従来の進捗判定は維持する。

EMPTY / ANY / 非空 literal、matched が consumed より先にある状態、reset の両値、inversion、memo replay、
child / terminator の非実行、capture / 診断なし、scan / LL(1) の参照経路との同値を検証した。
ゼロ回の自己参照は leading edge から除き、`Root ::= Root{0} 'x'` を受理する。
後続の実行される自己参照と、ゼロ回の下にある不正な反復境界は引き続き拒否する。

### builtin none の実因

`@whitespace: none token none ::= '#';` の delimiter 集合は、修正前から空だった。
名前解決や同名 token の優先順位ではなく、Classic の `@ubnf: v2` に ubnfc 独自の暗黙の単語境界を課したことが実因だった。
`'a'` の `wordBoundary: true` が入力 `ab` の先頭で失敗していた。

Classic の綴り `v2` では Classic と同じ prefix 一致を保つ。
既存の数値版 `@ubnf: 2` の D-040 の既定と、明示 `@literalBoundary` の優先順位は維持する。
`none` / `javaStyle` は case-insensitive な予約 profile で、同名 token による shadowing は認めない。
受理・Root(value=`a`, span=[0,2]) と、`a#b` を拒否する診断位置を比較する。

### 名前付き空白

修正前の 12 件は `E-WHITESPACE-GLOBAL-STYLE` 7 件 / `E-WHITESPACE-RULE-STYLE` 5 件だった。
module 展開前に import で止まっていた 32 件にも、同じ対応が必要な文法を含む。

IR に additive な `Delimiter::Lexical { expression, label }` と必須 feature `lexicalTrivia` を追加した。
既存の atomic な宣言的 token program を再利用し、各照合の局所 capture を外へ漏らさない。
正の進捗がある間だけ繰り返し、ゼロ幅成功では止まる。Rust は通常 trivia / 診断なし skip、Java は通常 trivia と予測の走査に通す。
未知の FIRST は全 code point の上界とし、先頭 byte による除外を使わない。policy の同一性には program と label も含める。

D-083 の std/layout `SPACES_AND_COMMENTS` だけを javaStyle と同値にする扱いは維持した。
その他の token は javaStyle へ読み替えない。token 内で空白を挿入せず、rule の局所 policy と復帰、interleave に対する名前付き空白の優先順位を比較する。
この対応を期待拒否へ付け替えたケースはない。

### module / manifest / cache

harness の inline grammar について、既存 `imports` に加えて corpus の `modules` を展開する。
`manifest` から `ubnf.json`、明示依存と raw artifact の SHA-256 による lock、`.ubnf-cache/packages/<sha256>.json` を生成する。
Classic の fixture setup と同じく builtin / local の明示依存をオフラインで解決し、raw artifact の bytes は変更しない。
展開される全ファイルのパスと本文を build key の原文照合へ含める。
同じ main grammar で module が違う場合も、別の build と実際の配置になることを回帰テストで検証した。

front は package 内の相対 import と明示的な推移依存を artifact の files / dependencies と lock で解決する。
生成時に外部取得はしない。ID・版・SHA-256・依存の不一致、未宣言依存、artifact の外へのパスは拒否する。
入れ子の entry と `../` による artifact 内参照、外への参照拒否も検証した。
失敗は元の import の位置付き診断へ返す。

専用 worktree には既定の sibling fixture がないため、CLI の既定パステストはその場合の明示エラーも検証するようにした。
実 corpus の成功は `--root` と環境変数の両経路で検証し、fixture が存在する CI の既定パス成功検査も維持する。

### 新たに実測した Rust の不完全指数

変更前の Rust は issue の 3 件に加え、`evolution/diagnostics.json` の `neg(1e+)` が主位置 5 対期待 7 で失敗した。
現 pin の native `unlaxer-runtime::parse_detailed` を直接呼んで独立に実測したところ、`neg(1e+)` は位置 7、`n1e+;` は位置 4 だった。
Classic #717 / #752 と同じく、消費は仮数まで戻しても、指数内の失敗位置を診断へ残すようにした。
旧 local Rust oracle の `numeric/7` も、この実走値へ更新した。
[観測値](2026-11-01-new-pin-corpus-data/native-number.json)と
[独立 probe](2026-11-01-new-pin-corpus-data/native-number.rs)を同梱する。probe は ubnfc に依存しない。

### singular diagnostic の観測

rule-trivia の `diagnostic: [kind, offset]` は driver の公開 `diagnostics[0]` から種類と code point 位置を比較する。
Classic の `RuleTriviaConformanceTest` と同じ射影である。
空の診断列・位置違い・種類違いは Fail、キー自体の欠落だけを未観測とする回帰テストを追加した。
名前付き空白 7 件を新たな未観測へ付け替えず、従来から未観測だった builtin の 2 件も照合できるようにした。

## IR / golden / 独立 oracle

schema の追加は delimiter の新種、必須 feature の新値、raw whitespace の名前付き文字列で、既存の形・キーは削除していない。
D-083 の既存 std/layout raw settings と delimiter 集合は維持した。
既存 golden の値には、schema 拡張とは別に必要な互換性修正がある。
自己記述文法 `unlaxer-dsl/grammar/ubnf.ubnf` の literal 64 個と対応 LL(1) 命令 64 個の `wordBoundary` を true から false にした。
Classic `v2` の prefix 一致を正すための差で、その他の既存 IR golden は byte 一致する。
「既存 golden の値も additive な差だけ」とする条件については、この明示した 128 箇所が例外になる。
値を隠して旧 golden に合わせることはしない。自己ホスト frontend は生成時にキーワード境界を明示する
`with_keyword_boundaries` を通す。`@rooted` を `@root` と `ed` に分割せず、従来の frontend 受理・AST を維持する。
通常の Classic grammar の既定とは分け、既存の再生成 example と固定点テストにも同じ手順を使う。
この明示化により自己ホストの parser source は従来どおりで、共通 runtime の修正だけが再生成される。

ゼロ上限の local oracle 2 ファイルは、既存 `corpus/oracle/tools/collect.sh` で現 pin の Classic を実走して再採取した。
入力・文法は無変更で、ubnfc の実測から期待値を作っていない。
`empty-zero` / `repeat-zero` の新しい prefix 成功と trailing-input 診断を保存した。
他の採取物は旧 revision を維持する。`verify.py` はこの 2 ファイルだけの記録済みの revision 差を認め、未記録の混在は拒否する。
設計判断は D-092 に記録した。

## 再現

```sh
export RUSTUP_TOOLCHAIN=stable
export JAVA_HOME="$HOME/.sdkman/candidates/java/current"
export PATH="$JAVA_HOME/bin:$HOME/.sdkman/candidates/maven/current/bin:$PATH"
d=$(mktemp -d)
scripts/fetch-external-fixtures.sh "$d/external" > "$d/fixtures.env"
set -a
. "$d/fixtures.env"
set +a
cargo build --bin ubnfc --bin ubnfc-rust --bin ubnfc-harness
(cd ubnfc-java && mvn -q -o -DskipTests package)
target/debug/ubnfc-harness run-all --backend java --jobs 4 --json "$d/java.json" --report "$d/java.md"
target/debug/ubnfc-harness run-all --backend rust --jobs 4 --json "$d/rust.json" --report "$d/rust.md"
cargo test --workspace --no-fail-fast
(cd ubnfc-java && mvn -q -o test)
python3 corpus/oracle/tools/verify.py
```

Classic Java oracle は次のコマンドで別ディレクトリへ再採取し、`empty-zero.json` / `repeat-zero.json` を比較できる。

```sh
corpus/oracle/tools/collect.sh "$UNLAXER_PARSER_DIR" "$d/oracle-current"
```

native probe は上記 `.rs` を一時 crate の `src/main.rs` へ保存し、依存を
`unlaxer-runtime = { path = "<UNLAXER_PARSER_DIR>/rust/unlaxer-runtime" }` として
`cargo run --offline --manifest-path <一時 crate>/Cargo.toml` で実行する。

## 系統別表（#44 取り込み前の中間結果）

全 1,492 件。両 backend とも生成・build・実行エラーは 0、exit は残る LONG 13 件のため 1。
新しい期待拒否は 0 件。rule-trivia の未観測 2 件は診断の照合を実装して Pass へ移った。
その他の Known / 未観測は不変。


### Java

| 系統 | Pass 前→後 | 期待拒否 Pass 前→後 | Known 前→後 | 未観測 前→後 | IR 失敗 前→後 | Fail 前→後 |
|---|---:|---:|---:|---:|---:|---:|
| associative | 24→24 | 24→24 | 0→0 | 0→0 | 0→0 | 0→0 |
| cardinality | 70→70 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| catalog | 2→2 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| evolution | 140→140 | 0→0 | 0→0 | 16→16 | 0→0 | 0→0 |
| lexical | 78→78 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| mixed-values | 98→98 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| nested-captures | 10→10 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| numeric-capture | 13→13 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| oracle | 332→332 | 3→3 | 29→29 | 34→34 | 0→0 | 0→0 |
| primitives | 113→115 | 0→0 | 0→0 | 0→0 | 0→0 | 2→0 |
| right-associative | 112→112 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| rule-trivia | 34→81 | 0→0 | 0→0 | 2→0 | 44→0 | 1→0 |
| scope-annotations | 14→14 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| scope-store | 0→0 | 0→0 | 0→0 | 9→9 | 0→0 | 0→0 |
| semantic-cardinality | 163→163 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| tiny-code-fence | 73→73 | 0→0 | 0→0 | 0→0 | 13→13 | 0→0 |
| tiny-string-token | 39→39 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| 合計 | 1315→1364 | 27→27 | 29→29 | 61→59 | 57→13 | 3→0 |

変更前 front SHA-256: `d4c6cebb6aa634a79d53338f80c6389b7a4f70608ac4d96bcb2d70fc74cbe92a`。生成器 SHA-256: `74b7c7bbb9d8e4404f1dcf1e1c614c2a1714a402496f285426a5a2fe29ea3555`。

#44 前の修正後 front SHA-256: `1baf975cb62150bc366db7508dda892d76c9918ed318055e20065bfe82e01cd2`。生成器 SHA-256: `2acb43a1310b3dfddd0ccb9869627db9b0087cbbff6925e1b5def7041e55f7fd`。

[全 case の比較結果](2026-11-01-new-pin-corpus-data/java-pre44.json)。

### Rust

| 系統 | Pass 前→後 | 期待拒否 Pass 前→後 | Known 前→後 | 未観測 前→後 | IR 失敗 前→後 | Fail 前→後 |
|---|---:|---:|---:|---:|---:|---:|
| associative | 24→24 | 24→24 | 0→0 | 0→0 | 0→0 | 0→0 |
| cardinality | 70→70 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| catalog | 2→2 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| evolution | 139→140 | 0→0 | 0→0 | 16→16 | 0→0 | 1→0 |
| lexical | 78→78 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| mixed-values | 98→98 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| nested-captures | 10→10 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| numeric-capture | 13→13 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| oracle | 328→328 | 5→5 | 29→29 | 36→36 | 0→0 | 0→0 |
| primitives | 113→115 | 0→0 | 0→0 | 0→0 | 0→0 | 2→0 |
| right-associative | 112→112 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| rule-trivia | 34→81 | 0→0 | 0→0 | 2→0 | 44→0 | 1→0 |
| scope-annotations | 0→0 | 0→0 | 0→0 | 14→14 | 0→0 | 0→0 |
| scope-store | 0→0 | 0→0 | 0→0 | 9→9 | 0→0 | 0→0 |
| semantic-cardinality | 163→163 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| tiny-code-fence | 73→73 | 0→0 | 0→0 | 0→0 | 13→13 | 0→0 |
| tiny-string-token | 39→39 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 |
| 合計 | 1296→1346 | 29→29 | 29→29 | 77→75 | 57→13 | 4→0 |

変更前 front SHA-256: `d4c6cebb6aa634a79d53338f80c6389b7a4f70608ac4d96bcb2d70fc74cbe92a`。生成器 SHA-256: `a2fa844f771fca3b2bbcfa4d51fe6d3b36774ff911c3ca8686998244c0eae505`。

#44 前の修正後 front SHA-256: `1baf975cb62150bc366db7508dda892d76c9918ed318055e20065bfe82e01cd2`。生成器 SHA-256: `60f7c42d493ff826a0fde1f773d1db127f10509f02ce0dfe686efc4aee5ace06`。

[全 case の比較結果](2026-11-01-new-pin-corpus-data/rust-pre44.json)。

最終の全件実行・CI・merge は確認待ち。完了として扱わない。
