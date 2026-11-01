# tiny-code-fence の LONG 宣言と scanner（issue #33、2026-11-01）

対象は master `669c560`（PR #40 merge 後）。`scripts/external-pins.env` を変更せず、
専用 worktree と `mktemp -d` の外部 checkout で修正前後を比較した。
Java / Rust 生成器、front、fixture、期待値、乖離台帳は同一で、変更は corpus 合成と
harness 同梱 scanner、その契約テスト・文書だけである。

## 原因と変更

`TinyCodeFenceConformanceTest.grammar` は START / CLOSE / LONG / BODY / CHAR / END を
宣言するが、harness の `corpus/rule.rs::tiny` は LONG を省いていた。
`long-block` 12 件、`long-block-shifted` 1 件が `E-RULE-UNDEFINED` で停止する。
元テストと同じ `token LONG = org.unlaxer.tinyexpression.parser.javalang.LongCodeBlockParser`
を全 14 variant に追加した。

宣言だけでは同梱 registry に scanner がないため実行できない。
`scanners/java/Registry.java` と `scanners/rust/scanners.rs` に完全名のみを登録し、
`scanners/first-chars.json` に先頭 backtick を宣言した。
短い fence / BODY の仕様や生成 backend は変更していない。

元仕様は現 tinyexpression pin の `codeblock/LongCodeFence.java::scan` と
`parser/javalang/LongCodeBlockParser.java::getToken`、unlaxer-parser の
`docs/tiny-code-fence.md` と `TinyCodeFenceConformanceTest.java::grammar` で確認した。

- 開始は行頭、backtick は N >= 4、ASCII scheme と dotted class、必須改行。
- 本文は opaque。同幅 N 個だけの単独行で終了し、短い・長い・行中・末尾空白付きの fence は本文。
- 改行は LF / CRLF / CR、終端は EOF も可。非 BMP の公開 cursor/span は code point。
- 失敗は token 開始位置の `long code block`。消費と先読みの cursor を維持し、効果を返さない。
- 元 mapper の strip 契約に合わせ、value span は終端 backtick まで、cursor と capture 原文・span は終端改行を含む。
  これは既存 CodeStart / CodeEnd scanner と同じ値と原文の分離である。

最小の可逆な対応として、元の FQN と専用 scanner を維持した。LONG を UNTIL に置換すると
幅一致・行境界・opaque な本文の仕様を満たせない。fixture 削除、期待値の変更、Known 登録、
未観測への変更は行っていない。revert はこの PR の merge commit を `git revert -m 1` で戻す。

## 外部 pin と旧 pin との差

| repository | issue 本文の旧 pin | 今回の pin（変更なし） |
|---|---|---|
| unlaxer-parser | `08f2bc1c918a34b84c333a8178c990383fdffb8d` | `d3ea73e8cfa440df8bdbb70abb29e06259435ba2` |
| tinyexpression | `f86ce8a5ab0ab7d23fdba2cd477187df8aa7607c` | `c736c2c66e4569ebc5e22f1f0c5125a045679d35` |

旧 commit も専用外部 checkout に fetch して差分を確認した。tiny-code-fence/corpus.json は
旧 pin と現 pin で同一（14 文法、86 入力）。LONG の FQN 宣言も同一。
現 unlaxer-parser の元テストは native Rust の実行照合と Classic API の原文・AST 検査を追加している。
旧 tinyexpression pin には `LongCodeFence.java` / `LongCodeBlockParser.java` がなく、現 pin に存在する。
今回の scanner の根拠は現 pin の実装であり、旧 tinyexpression の実クラスでの実行互換性は主張しない。

## 再現と検証手順

Java は sdkman の JDK 21.0.9 / Maven 3.9.9、Rust は `RUSTUP_TOOLCHAIN=stable`。
変更前も後も以下の同じ環境を使い、before / after の出力先だけを変える。

```sh
export RUSTUP_TOOLCHAIN=stable
export JAVA_HOME="$HOME/.sdkman/candidates/java/current"
export PATH="$JAVA_HOME/bin:$HOME/.sdkman/candidates/maven/current/bin:$PATH"
d=$(mktemp -d)
eval "$(scripts/fetch-external-fixtures.sh "$d" | sed 's/^/export /')"
cargo build --bin ubnfc --bin ubnfc-rust --bin ubnfc-harness
(cd ubnfc-java && mvn -q -o -DskipTests package)
target/debug/ubnfc-harness run-all --backend java --jobs 4 \
  --report "$d/java.md" --json "$d/java.json"
target/debug/ubnfc-harness run-all --backend rust --families tiny-code-fence --jobs 4 \
  --report "$d/rust-fence.md" --json "$d/rust-fence.json"
cargo test -p ubnfc-harness
```

`tests/corpus.rs` は全 14 文法の LONG 宣言、全 86 件、長い fence 13 件（受理 6 / 拒否 7）を固定する。
`tests/runall.rs` は fence 全件を Java / Rust で実行し、受理・全入力消費・両 cursor・AST span・全 fields を
元期待値と比較する。長い fence では raw capture の text/span、正常時の空診断、拒否時の
syntax / offset / farthestOffset / expected / farthestExpected と cursor 巻き戻しも照合する。
Java primary expected は `RootParser, long code block`、Rust は `long code block`、両者の
farthestExpected は `RootParser, long code block` で、既存公開診断の差も明示固定した。

`scanners/java/RegistryTest.java` と `scanners/rust/scanners_tests.rs` は UTF-16 / UTF-8 の
内部位置、先読み・反転・reset、幅違い・行境界・必須改行・不正 identifier、失敗原子性を確認する。
これらは既存 `tests/scanners.rs` から実 API と生成物で実行される。

## Java 全 corpus の系統別比較

表の単位は case × variant。`P` は通常 Pass、`拒否P` は期待拒否 Pass、`K` は Known、`U` は未観測。

| 系統 | 変更前 IR / P / 拒否P / K / Fail / U | 変更後 IR / P / 拒否P / K / Fail / U |
|---|---|---|
| associative | 0 / 24 / 24 / 0 / 0 / 0 | 0 / 24 / 24 / 0 / 0 / 0 |
| cardinality | 0 / 70 / 0 / 0 / 0 / 0 | 0 / 70 / 0 / 0 / 0 / 0 |
| catalog | 0 / 2 / 0 / 0 / 0 / 0 | 0 / 2 / 0 / 0 / 0 / 0 |
| evolution | 0 / 140 / 0 / 0 / 0 / 16 | 0 / 140 / 0 / 0 / 0 / 16 |
| lexical | 0 / 78 / 0 / 0 / 0 / 0 | 0 / 78 / 0 / 0 / 0 / 0 |
| mixed-values | 0 / 98 / 0 / 0 / 0 / 0 | 0 / 98 / 0 / 0 / 0 / 0 |
| nested-captures | 0 / 10 / 0 / 0 / 0 / 0 | 0 / 10 / 0 / 0 / 0 / 0 |
| numeric-capture | 0 / 13 / 0 / 0 / 0 / 0 | 0 / 13 / 0 / 0 / 0 / 0 |
| oracle | 0 / 332 / 3 / 29 / 0 / 34 | 0 / 332 / 3 / 29 / 0 / 34 |
| primitives | 0 / 113 / 0 / 0 / 2 / 0 | 0 / 113 / 0 / 0 / 2 / 0 |
| right-associative | 0 / 112 / 0 / 0 / 0 / 0 | 0 / 112 / 0 / 0 / 0 / 0 |
| rule-trivia | 44 / 34 / 0 / 0 / 1 / 2 | 44 / 34 / 0 / 0 / 1 / 2 |
| scope-annotations | 0 / 14 / 0 / 0 / 0 / 0 | 0 / 14 / 0 / 0 / 0 / 0 |
| scope-store | 0 / 0 / 0 / 0 / 0 / 9 | 0 / 0 / 0 / 0 / 0 / 9 |
| semantic-cardinality | 0 / 163 / 0 / 0 / 0 / 0 | 0 / 163 / 0 / 0 / 0 / 0 |
| tiny-code-fence | 13 / 73 / 0 / 0 / 0 / 0 | 0 / 86 / 0 / 0 / 0 / 0 |
| tiny-string-token | 0 / 39 / 0 / 0 / 0 / 0 | 0 / 39 / 0 / 0 / 0 / 0 |
| 合計 | 57 / 1315 / 27 / 29 / 3 / 61 | 44 / 1328 / 27 / 29 / 3 / 61 |

全 1,492 件。Java 全 corpus は変更前 **IR 失敗 57 / Fail 3 / exit 1**、変更後 **IR 失敗 44 / Fail 3 / exit 1**。
生成拒否・javac/build 失敗・実行エラーは全て 0。全体の IR 失敗 0 / Fail 0 / exit 0 という受け入れ条件は未達。
LONG の 13 件のみが IR 失敗から Pass へ変化した。その他の outcome / differences / missing は全件一致。
既存 fence 73 件の actual は実行内の応答対応用 wire id と、文法原文の位置から作る
lexical exprId 以外は全項目一致した（宣言追加による原文位置の移動）。fence 以外は分類・差分・欠落項目を比較し、actual 全体の一致は主張しない。
Java / Rust の fence 全 86 件が各期待値と一致し、両方とも 86 Pass / Fail 0 / 未観測 0。LONG 13 件の受理、両 cursor、AST、capture 原文・span は両 backend 間でも一致。
長い fence は 6 受理・7 拒否のままで、拒否を Pass とするのは拒否期待との一致による。

[集計と長い fence 全 13 件の両言語実測 JSON](2026-11-01-tiny-code-fence-long.json) に
期待値・入力・実測・差分・欠落項目、front / 生成器 SHA-256 と系統別件数を保存した。

| 長い fence の受理ケース | cursor（consumed = matched） | capture span（code point） |
|---|---:|---|
| long-block-shifted/unicode-prefix | 30 | [2, 30] |
| long-block/cr | 30 | [0, 30] |
| long-block/crlf | 45 | [0, 45] |
| long-block/empty | 20 | [0, 20] |
| long-block/eof | 19 | [0, 19] |
| long-block/lf | 34 | [0, 34] |

## 現 pin で残る全体検証の阻害要因

旧 pin での issue 本文は全 1,444 件、今回の現 pin は全 1,492 件。
rule-trivia は 33 → 81 件へ 48 件増加し、primitives の `{0,0}` 2 件は拒否期待から
受理期待（value = `ab`）へ更新されている。これらを旧期待値へ戻してはいない。

| 現 pin で変更前からある失敗 | 件数 | 根拠と担当層 |
|---|---:|---|
| rule-trivia / named whitespace | IR 12 | `E-WHITESPACE-GLOBAL-STYLE` 7、`E-WHITESPACE-RULE-STYLE` 5。front は javaStyle / none と特定 package profile のみ対応 |
| rule-trivia / import | IR 32 | `E-IMPORT-UNRESOLVED`。現 fixture の `modules` / `manifest` が harness で展開されない。展開後も独自 named whitespace は front 対応が必要 |
| primitives / `35:EMPTY/0`・`36:ANY/0` | Fail 2 | `T{0,0} 'ab' @value` の全入力受理と AST が期待と異なる。合成文法は元 `RustConformanceTest.tokenCorpus` と同じ |
| rule-trivia / BuiltinNoneReserved | Fail 1 | `@whitespace: none` と同名 token `none` の併存で入力 `ab` を拒否する。文法原文を変更せず検出 |

この PR の LONG 13 件は解消し、他系統の Pass / Known / 未観測・既存失敗は悪化していないが、
全体条件を満たしたとは記録しない。LONG の失敗を隠すための除外・免除も追加していない。
元の作業範囲では front / backend の変更は禁じられているため、残る全体条件と作業範囲の
扱いについて確認を求めた。判断が確定するまで全体完了・issue close の根拠にはしない。

## ローカルテスト

- loader と両言語 scanner 契約テスト: 成功。
- 両 backend の fence 全 86 件の E2E（診断の候補配列も厳密比較）: 成功。
- `cargo test -p ubnfc-harness -- --skip list_uses_explicit_root_and_environment_and_rejects_missing_corpus`: 90 成功、exit 0。
- 除外した既存 CLI テストは、環境変数を外して固定の隣接 checkout を検証するため、専用 worktree では
  隣接 repository がなく失敗する。テスト自体は変更していない。CI は固定 pin の sibling link を用意し、
  このテストも含む `cargo test --workspace --no-fail-fast` を実行する。
- 変更 Rust ファイルの `rustfmt --check` と `git diff --check`: 成功。
