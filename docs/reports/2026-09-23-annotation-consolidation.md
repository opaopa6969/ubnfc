# UBNF アノテーション統廃合 — P7-1 / P7-3 の不具合と P1 / P2 / P3 / P5

> 2026-09-23。`docs/design/07-annotation-inventory.md` が推奨した束をまとめて入れた。
> 決定行は `03-design-v1.md` §8 の **D-043 〜 D-047**。
> IR は 1 バイトも変えていない（goldens 無変更）。v1 の受理・AST も変えていない。

## 0. 何をしたか

| 項目 | 結果 |
|---|---|
| P7-1 `@rightAssoc` span 補正の非対称 | **直した**（Java に trivia 門を追加、入れ子判定も Rust に合わせた）。ただし**観測差は作れなかった**（§1） |
| P7-2 Java の `languageVersion` 上限検査が無い | **棚卸しの誤り**。`ir/Validation.java:43-46` に既にある（§2） |
| P7-3 `@rightAssoc` + `@precedence` が `operator` を作らない | **前提が誤り**。本当の不具合は別（`@rightAssoc` + 規則参照演算子で文法が落ちる）。直した（§3） |
| P7-4 同一内容 TriviaPolicy の重複 | **見送り**。policy ID の採番が動いて IR が変わる（§4） |
| P1 `W-ANNOTATION-UNKNOWN` | 追加。24 fixture + `ubnf.ubnf` + P4 で **2 件**発火（§5） |
| P2 `@symbolRef` / `@sameText` | 追加。`@backref` は残し v2 で警告（§6） |
| P3 `@interleave` 非推奨（v2） | 追加（§7） |
| P5 `@predictiveChoice` 非推奨（v2）/ `@longestChoice` 維持 | 追加（§8） |

## 1. P7-1 — `@rightAssoc` の span 補正

### 根本原因

同じ「右結合ノードの span を significant end で切る」補正が、両 backend で**別の条件**に掛かっていた。

| | 門 | 入れ子判定 | 切り方 |
|---|---|---|---|
| Rust `emit.rs` | `rightAssoc` **かつ** 同じ規則に `interleave` / `whitespace` / `comment` 注釈 | `self.depth > 1` | `significant_end(events)` |
| Java `Session.java`（旧） | `rightAssoc` だけ | `activeCounts[id]++ > 0`（**同じ規則の再帰**呼出し） | field 値の終端の最大 |

### 正しい振る舞いの決め方

旧実装（`unlaxer-parser`、読み取り専用）の 2 つの oracle が両側を押さえていた。

1. **grammar 既定の trivia だけ**（規則に trivia 注釈なし）:
   `unlaxer-dsl/src/test/java/.../RightAssociativeMappingRuntimeTest.java:141-152`。
   `" 2 ^ 3 ^ 2 "` に対し root `[0,11]` / 右子 `[5,11]` / 葉 `[9,11]` ＝ **末尾 trivia を含む消費範囲のまま**。
   これは `generators.md` #138 の一般規則（「source span は各 mapped rule の消費範囲であり、
   javaStyle delimiter が消費した末尾空白・コメントを含むことがある」）そのもの。
2. **規則局所の trivia 注釈がある**:
   `unlaxer-dsl/src/test/resources/rule-trivia/corpus.json:14`（`RightScoped`、
   `@interleave(profile=commentsAndSpaces)`）。`" x /*r*/ ^ x "` に対し外側 `[0,13]` / 右子 `[11,12]`
   ＝ **significant end で切る**。Rust の round2 報告が「右結合の局所例外」「一般規則との差を限定」と
   書いているのはこれで、Java の round2 報告も同じ値を「#138 の一般規則と両立する例外」と記録している。

つまり**正しいのは Rust の門**（規則局所の trivia があるときだけ補正）。
入れ子判定は **`depth > 1`（＝外側の規則が末尾 trivia を消費する立場にある）**のほうが
理屈が通る（補正の理由は「この規則の末尾 trivia は外側の方針が消費したもの」だから、
自己再帰かどうかではなく入れ子かどうかで決まる）。

### 直したこと

- `ubnfc-java/src/main/java/org/ubnfc/java/emit/JavaEmitter.java`: `rightAssociativeSpan(rule)` を新設し、
  `rightAssoc` かつ `interleave` / `whitespace` / `comment` 注釈が同居するときだけ true。Rust と同一条件。
- `ubnfc-java/src/main/resources/templates/internal/Session.java`: `recursive`（`activeCounts`）を
  `nested`（`depth > 1`）に置き換え、使われなくなった `activeCounts` 配列（Session ごとの
  `int[ruleNames.length]` 確保と rule 呼出しごとの inc/dec）を削除した。

### 回帰テスト — と、見つからなかった観測差

`crates/ubnfc-fuzz/tests/right_assoc_span.rs`（Java / Rust の driver 応答を `observe::differences` で
突き合わせる的を絞った差分テスト）。4 文法 × 15 入力:

1. grammar 既定 trivia + 規則参照 `@left`（`Power.ubnf` と同形）
2. 同上 + **リテラル** `@left`（値 span が短くなる形）
3. 規則局所 `@interleave` + entry 規則（`RightScoped` と同形、`" x /*r*/ ^ x "` を含む）
4. 規則局所 `@whitespace` + **entry でない**右結合規則（`depth > 1` と「再帰」が食い違う形）

**修正前の Java（無条件 + `recursive`）でもこの 15 入力は全部一致した。**
理由は、Java の補正が実際に効くのは `f.c` が最後の field 値の終端より先にある場合だけで、
grammar 既定 trivia では capture の値 span 自体が末尾 trivia を含むところまで伸びるため
`max(field ends) == f.c` になり、補正が no-op になるから。`f.c` が値の終端より先に出るのは
規則局所 trivia があるとき、つまり **Rust も補正する場合**に限られる。
したがってこの非対称は「条件の食い違い」であって、現時点で到達可能な観測差ではない。
テストは 4 形をピン留めして、どちらかの backend が今後条件を動かしたら落ちるようにしてある。
差分 fuzzer の固定 seed campaign が見逃していたのは、演算子塔の規則に trivia 注釈を
付けないため（＝補正が no-op の側にしか当たらない）である。

## 2. P7-2 — Java の `languageVersion` 上限検査（棚卸しの誤り）

`ubnfc-java/src/main/java/org/ubnfc/java/ir/Validation.java`:

```java
public static final int MAX_LANGUAGE_VERSION = 2;
...
int language = ir.languageVersion() == null ? 1 : ir.languageVersion();
if (language < 1 || language > MAX_LANGUAGE_VERSION) unsupported("languageVersion:" + language);
```

D-038 が要求する「両 backend が未知バージョンを明示拒否する」は満たされている。修正不要。
棚卸し側（`07-annotation-inventory.md` §2.2 と P7-2）を訂正した。

## 3. P7-3 — `@rightAssoc` と `operator` metadata

### 棚卸しの前提は誤りだった

「`@leftAssoc` + `@precedence` は `operator` を作るが `@rightAssoc` + `@precedence` は作らない」
という読みは、`Power.ubnf`（右結合・**リテラル**演算子 `'^'`）1 件からの推測だった。
実際の分かれ目は結合性ではなく**演算子 atom の種類**である。
`lower/annotations.rs::operator` は左右どちらでも `operator_elements` を通したあと
「base と op が**規則参照**であること」を要求しており、リテラル演算子なら結合性に関係なく `None` になる。
実測（`@leftAssoc` + リテラル `'+'`）でも `operator=None` を確認した。

### 本当の不具合

`@rightAssoc` + **規則参照**演算子は、lowering が
`operator{associativity: right, canonicalForm: baseOpSelfOrBase}` を作るのに、
`analysis/operators.rs::canonical` が右結合の段として認めるのは**正規化後**の
`Base Op Self | Base` の形だけで、v1 の表層形 `Base @left { Op @op Self @right }` を認めない。
結果、`rule.operator.is_some() && tier.is_none()` になり
`AnalysisError::InvalidOperator { message: "operator metadata が指定する正規形ではありません" }` で
**文法ごと拒否される**。同じ形の `@leftAssoc` は通るので、これが本当の結合性の非対称だった。
D-039 が `E-LEFTREC-RIGHTASSOC-OP-REF` として「既存の穴」と名指しし、v2 の左再帰側だけ塞いでいたもの。

再現（修正前は exit 1）:

```ubnf
@mapping(BinR, params=[left, op, right]) @rightAssoc @precedence(level=20)
R ::= Atom @left { PowOp @op R @right };
PowOp ::= '^';
```

### 直したこと（D-045）

`lower/annotations.rs::operator` は右結合では `operator` を作らない。
これで `@rightAssoc` はリテラル演算子でも規則参照演算子でも同じ扱い（`noncanonical` →
`tierFunctions` 計画）になり、上の文法が通る。**既存 IR は 1 バイトも変わらない**
（右結合 + 規則参照演算子の文法はこれまで受理されていなかったので、変わる IR が存在しない）。

**やっていないこと**: `@rightAssoc` にも `@leftAssoc` と同じ `operator` を作る完全対称化。
それには `canonical` に v1 の表層右結合形を教える必要があり、`Power.ubnf` の IR に `operator` が
生え、`plan.operators` の該当行が `noncanonical` の tierFunctions 計画から段ベースの計画に変わる。
今回の束の前提（IR 不変）を出るので別作業にする。

## 4. P7-4 — TriviaPolicy の重複（見送り）

`lower/trivia.rs` の重複排除キーは `(raw_settings, effective_delimiter_set)` で、
`rawSettings` の来歴（`interleave` 由来か `whitespace` 由来か）が違うだけで policy が分裂する。
キーを `effective_delimiter_set` だけにすれば直るが、**policy ID の採番が動いて IR が変わる**
（`trivia:javaStyle` / `trivia:javaStyle:<ruleId>` の割り当てが変わる）。
self-host の byte 一致に出るため、IR 不変を前提にしたこの束では入れない。
P3 の移行が進んで `@interleave` が消えれば自然に解消する（棚卸しの指摘どおり）。

## 5. P1 — `W-ANNOTATION-UNKNOWN`（D-043）

UBNF の `@<任意の識別子>` catch-all（`Annotation::Simple`）が綴り間違いを黙って metadata にしていた。

**警告文**（規則注釈）:

```
rule <Rule> has an unknown annotation: @<name>
  [code: W-ANNOTATION-UNKNOWN]
  [hint: Remove @<name> or fix the spelling; unknown annotations are carried as metadata and never reach code generation.]
```

**版で門を掛けない**。未知注釈は「v1 で唯一の綴り」ではなく常に間違いなので、
D-042（旧綴りは v2 だけ警告）ではなく **D-037（`W-CHOICE-*` は v1 でも警告）と同じ扱い**にした。
受理・AST・生成コードは変わらない純粋な診断の追加である。`--strict` でエラー化できる。

### 発火箇所（24 fixture + `ubnf.ubnf` + P4）

| 文法 | 注釈 | 備考 |
|---|---|---|
| `rust/unlaxer-ubnf/tests/fixtures/known/keyword-boundary.ubnf` | `@rooted` | `@root` の綴り間違い。`@root` が無いので entry が `lower/mod.rs` の「最初の規則」fallback に落ち、**偶然正しく動いていた**（`W-GENERAL-NO-ROOT` は元から出ていた） |
| `rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf` | `@custom` | 合成カバレッジ fixture。この文法は元から 12 件の ERROR を出す |

それ以外の 22 fixture・`ubnf.ubnf`・P4・P4-v2 は **0 件**。未知の grammar 設定も 0 件。

**未知の grammar 設定**（`@foo: bar`）には同じ警告を **`@ubnf: 2` でだけ**出す。
設定側には旧 Java 検証器と揃えた明示の前方互換契約があり
（`crates/ubnfc-front/tests/fixtures/validation-java.json` の
`testUnknownGlobalSettingCompatibilityIsUnchanged`: `@futureSetting: value` に診断なし）、
これは corpus 由来の oracle なので v1 では崩さない。

### 変わった ubnfc 自身の期待値

| テスト | 旧 | 新 |
|---|---|---|
| `crates/ubnfc-front/tests/resolve_validate_fixtures.rs` `keyword-boundary.ubnf` | `{W-GENERAL-NO-ROOT: 1}` | `{W-GENERAL-NO-ROOT: 1, W-ANNOTATION-UNKNOWN: 1}` |
| 同 `positive/annotations.ubnf` | `{}` | `{W-ANNOTATION-UNKNOWN: 1}` |
| 同 合計 | `{W-LEFT-RECURSION: 2, W-GENERAL-NO-ROOT: 3}` | `+ {W-ANNOTATION-UNKNOWN: 2}` |
| `crates/ubnfc-front/tests/pipeline_golden.rs` `annotations.ubnf` の `severityCounts.WARNING` | 0 | 1（ERROR は 12 のまま） |
| `ir/fixtures/errors/rust__…__annotations.ubnf.txt` | 12 行 | 13 行（`W-ANNOTATION-UNKNOWN` を 1 行追加） |

corpus / oracle の期待値は一切変えていない。

## 6. P2 — `@symbolRef` / `@sameText`（D-044）

`@backref` の意味は「文法中のどこか 1 つの規則に `@scopeTree` があるか」という**非局所**な判定で
決まっていた（`validate/mod.rs`、`lower/annotations.rs`）。どこか 1 規則に `@scopeTree` を足すと
文法中の全 `@backref` の意味が黙って変わる。

- `@symbolRef(name=X)`: scope store 検索。未定義は WARNING で構文成功。
- `@sameText(name=X)`: 同一 rule 内テキスト一致。不一致は ERROR。
- 明示綴りは非局所判定より**優先**する。旧 `@backref` だけが従来規則で決まる。

**警告文**（`@ubnf: 2` の文法だけ）:

```
rule <Rule> uses @backref, whose meaning depends on whether any rule in the grammar has @scopeTree
  [code: W-ANNOTATION-BACKREF-AMBIGUOUS]
  [hint: Use @symbolRef(name=...) for a scope lookup, or @sameText(name=...) for a same-rule text match.]
```

**IR は変わらない**。`scope.effects[].mode` はすでに `lexical` / `dynamic` / `sameRuleText` に
解決済みで、front が 2 つの表層名を同じ field へ落とすだけ。backend は 1 行も変わらない。
`rules[].annotations[].name` には書かれた綴り（`backref` / `symbolRef` / `sameText`）をそのまま残す。

副次効果として `E-ANNOTATION-BACKREF-CAPTURE`（capture 名の存在検査）を、明示綴りでは
テキストモードでも効かせる。旧 `@backref` の v1 診断は従来どおり scope モードのときだけ。

`ubnf.ubnf`（UBNF 自身のメタ文法、`unlaxer-parser` の読み取り専用ファイル）は v1 のままなので
旧綴りしか持たない。self-host 経路（`selfhost/mod.rs` の `BackrefAnnotation`）は `mode=None` を作る。

### 発火箇所

| 文法 | 件数 |
|---|---|
| `examples/p4-v2/tinyexpression-p4-v2.ubnf`（`@ubnf: 2`） | **2**（`MethodInvocation`、`VariableRef`） |
| v1 の 24 fixture・`ubnf.ubnf`・P4 | 0（v2 門） |

実在の `@backref` は全件が `@scopeTree` を持つ文法にあるので、全件 `@symbolRef` へ機械置換できる。
`@sameText` の実使用は世界にゼロ。

## 7. P3 — `@interleave` の非推奨（D-046、v2 のみ）

**警告文**:

```
rule <Rule> uses @interleave(profile=<p>), which resolves to the same delimiter set as @whitespace
  [code: W-ANNOTATION-INTERLEAVE-DEPRECATED]
  [hint: Use @whitespace (or @whitespace(javaStyle)) instead; @interleave is removed in a later version.]
```

**IR は変えない**。`trivia.policies[].rawSettings.interleave` と policy ID の綴りが self-host の
byte 一致に効くので、採番規則も重複排除キーも触っていない（§4）。

### 発火箇所

| 文法 | 件数 |
|---|---|
| `examples/p4-v2/tinyexpression-p4-v2.ubnf` | **3**（`Formula`、`ImportDeclaration`、`VariableDeclaration`） |
| v1 の 24 fixture・`ubnf.ubnf`・P4 | 0（v2 門。P4 の 3 件は v1 なので黙る） |

`@interleave` の corpus oracle は rule-trivia の 2 件（Tt[5,6] と Tt[14]）で、いずれも v1 のまま無影響。

## 8. P5 — `@predictiveChoice` の非推奨 / `@longestChoice` の維持（D-047、v2 のみ）

**警告文**:

```
rule <Rule> uses @predictiveChoice, which the choice analysis already decides on its own
  [code: W-ANNOTATION-PREDICTIVE-REDUNDANT]
  [hint: Remove @predictiveChoice; the front picks strategy=predict with proof=exclusiveCandidates where it is provable. @longestChoice is a different (semantic) choice and stays.]
```

根拠は棚卸しのとおり: front は注釈と無関係に `plan.choices[].strategy=predict` を 16 箇所選んでおり
（`analysis/choices.rs` の `proof.kind=exclusiveCandidates`）、一方どの文法もこの注釈を使っていない。
意味論は `ordered` と同値なので、消しても受理・AST・診断は変わらない。

`@longestChoice` は**残す**。最長一致は受理する言語そのものが変わる選択で解析から導出できない。
`03-design-v1.md` §3.4 に「最適化ではなく意味論の選択」と明記した。

### 発火箇所

実在する文法は 24 fixture・`ubnf.ubnf`・P4・P4-v2 のいずれも `@predictiveChoice` を使っていないので
**0 件**（`ubnf.ubnf` の 1 件はメタ文法中の文字列リテラルであって使用ではない）。

## 9. 差分 fuzzer の generator（`@ubnf: 2` 帯）

`crates/ubnfc-fuzz/src/gen.rs` の v2 分岐を足した。**乱数の引き方は v1 と同じ**にしてあるので、
v1 帯の既存 seed の生成物は 1 バイトも変わらない（D-041 / D-042 と同じ作法）。

| 旧綴り | v2 帯での生成 |
|---|---|
| `@interleave(profile=javaStyle\|commentsAndSpaces)` | `@whitespace(javaStyle)`（同じ規則に `@whitespace` が既にあるときは書かない＝重複エラーを避ける） |
| `@backref(name=X)` | `@symbolRef(name=X)`（生成側は必ず `@scopeTree` と同居させているので scope 側） |
| `@predictiveChoice` | 生成しない（`@longestChoice` は両帯で生成する） |

## 10. 検証

| 項目 | 結果 |
|---|---|
| `cargo test --workspace --offline` | **緑**（79 target、exit 0）。`ubnf_v2` は 18 → **22** テスト（D-043 〜 D-045 の 4 本を追加） |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | **緑** |
| `cargo fmt --all -- --check` | **緑** |
| `cd ubnfc-java && mvn -o -q test` | **緑** |
| `cd examples/p4-java && mvn -o -q test` | **緑**（`scripts/generate.sh --write` で `internal/Session.java` だけ再生成。P4 は `@rightAssoc` を使わないので `rightAssociative` 引数は全 124 規則で `false` のまま） |
| `cargo test -p ubnfc-fuzz` | **緑**（固定 seed campaign `a_fixed_seed_campaign_...` と新規 `right_associative_node_spans_agree_between_backends`） |
| self-host 固定点 | **緑**（`generated_frontend_reaches_the_ir_and_rust_fixed_point`、`unmodified_golden_regenerates_every_committed_byte`、`p4_ir_regenerates_every_committed_byte`） |
| corpus Java | **Pass 1310 / Fail 0**（期待拒否 Pass 28、Known 28、未観測 59）。系統表 305 行が `docs/reports/2026-09-23-java-corpus-contract.md` と**完全一致** |
| corpus Rust | **Pass 1292 / Fail 0**（期待拒否 Pass 30、Known 28、未観測 75）。系統表 305 行が `docs/reports/2026-09-23-rust-corpus-contract.md` と**完全一致** |
| IR goldens | **無変更**（`ir/fixtures/*.ir.json` の diff はゼロ。変わったのは `ir/fixtures/errors/…annotations.ubnf.txt` の 1 行だけ） |

corpus / oracle の期待値は 1 件も変えていない。変わった ubnfc 自身の期待値は §5 の表のとおり。

## 11. 残件

- **P4**（`@precedence(level=N)` を省略可能に）と **P6**（`@enum` / `@commonField` を `@mapping` の
  下位形へ）は未着手。P4 は `analysisVersion` を上げる必要があり corpus の再走が要る。
- **P7-4**（TriviaPolicy の重複排除）は policy ID の採番が動くので見送り（§4）。
- **`@rightAssoc` の `operator` 完全対称化**（§3 の「やっていないこと」）。
  `analysis::operators::canonical` に v1 の表層右結合形を教える作業で、`plan.operators` が変わる。
- **`E-ANNOTATION-DUPLICATE-BACKREF` の文言**は新綴りでも `@backref` と言う。
  文言を綴りごとに変えると旧 Java 検証器の parity corpus（`validation-java.json`）と食い違うので据え置いた。
