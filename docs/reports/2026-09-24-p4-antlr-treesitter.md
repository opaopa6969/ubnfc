# P4 を ANTLR 4 / tree-sitter へ移植して比べた（2026-09-24）

実用文法 **P4**（tinyexpression、UBNF v2 で 119 規則 / v1 の手書き形で 124 規則）を
ANTLR 4 と tree-sitter へ移植し、**同じ 16 fixture**で受理の一致と速度を比べた。
`docs/reports/2026-09-23-parser-speed-comparison.md` の family A と同じ文法・同じ入力だが
**別セッションの新規測定**なので、あちらの数字と横に並べて読んではいけない。

全数値は本日この機械で新しく採ったもので、過去の報告からの引用はない。
測れなかったものは §6 に列挙し、代わりの数値で埋めていない。

| | |
|---|---|
| 文法（移植元） | `examples/p4-v2/tinyexpression-p4-v2.ubnf`（md5 `2d3d8874cd56d3d8dd2c7d48747d7455`） |
| 移植先 | `examples/p4-antlr/TinyExpressionP4.g4` / `examples/p4-treesitter/grammar.js` |
| 対応表・逸脱の一覧 | [`examples/p4-antlr/README.md`](../../examples/p4-antlr/README.md)（119 規則 1:1、token 7、雛形 11） |
| 再現 | `benchmarks/scripts/p4-external.sh prepare\|accept\|negatives\|speed\|profile` |
| 生の出力 | [`2026-09-24-p4-antlr-treesitter/`](2026-09-24-p4-antlr-treesitter/)（受理・拒否入力・速度 2 pass・ANTLR profile・ANTLR 単発解析の 6 ファイル） |
| ubnfc commit | `79cf592b2ed34e5fe744ff751525550d3e3b9f8f`（`origin/master` `75dec71` を merge 済み） |
| 姉妹報告 | [`2026-09-24-antlr-treesitter-comparison.md`](2026-09-24-antlr-treesitter-comparison.md)（#26。JSON と小さな算術式で同じ 4 実装を比べたもの） |

## 0. 何を比べていないか

> **ANTLR4 は ALL(\*) の適応的予測と**エラー回復**を、tree-sitter は GLR の**増分解析**と
> エラー回復を持ち、どちらもエディタでの再解析を主目的に設計されている。ubnfc はそのどちらも
> 目的にしていない（増分解析は無く、回復は文法が `@recovery` を書いたときだけ）。
> **ここで測っているのは「1 回の完全な解析」だけ**で、各ツールが最も価値を出す用途は測っていない。
> ANTLR は `BailErrorStrategy` で回復を切り、tree-sitter は回復した結果を `has_error` で拒否扱いにしている。

P4 の注釈のうち **scope 解決**（`@scopeTree(mode=lexical)` / `@declares` / `@backref` /
`@catalog`）と **AST 射影**（`@mapping`）には ANTLR にも tree-sitter にも対応物が無い。
2 つの移植は**構文解析だけ**を行い、名前解決も型付き AST も作らない。
AST の**形**も比べていない（3 つのツールが作るものは別物）。

ubnfc 側は scope 解決を**常に**行う。`ParseOptions` に scope を切るフィールドは無いので、
「scope 無し」の数字は IR の `semantics.scope.effects` と `semantics.catalogs` を空にして
生成し直した parser を同じ構成で測って採った（§5 の «scope 無効化 IR» 行）。

## 1. 移植の逸脱（要約）

詳細と「影響する入力」は [`examples/p4-antlr/README.md` §スキャナレス意味論からの逸脱](../../examples/p4-antlr/README.md)。
受理結果に出るのは **逸脱 6（順序つき選択のコミット）だけ**である。

| # | 逸脱 | 受理への影響 |
|---|---|---|
| 1 | `NUMBER` の符号を token から構文側へ出した（両方） | `abs(- 3)` のように符号と数字の間に空白を挟んだ形を受理してしまう。16 fixture には無い |
| 2 | code fence の行頭条件（tree-sitter だけ表現できない） | 16 fixture は code fence を使わない |
| 3 | code fence の token 分割（ANTLR は 1 token に統合） | 同上 |
| 4 | 予約語と同じ綴りの識別子が使えない（両方） | 16 fixture には無い |
| 5 | `EOF` token（tree-sitter に無いが観測上は等価） | 無し |
| **6** | **順序つき選択のコミット**（PEG vs ALL(\*) / GLR） | **`comparison-heavy` 系 4 件**（§2） |
| 7 | 入口 2 本（tree-sitter は文法を 2 回 generate する） | 無し |
| 8 | GLR の曖昧性解決に `prec.dynamic` 27 箇所（tree-sitter だけ） | 木の形だけ |

**優先順位**は P4 v2 の候補順そのまま。`NumberExpression` は `MulOp` > `AddOp`、
`BooleanExpression` は `^` > `&` > `|`、`StringExpression` は `+` の 1 段。すべて左結合で、
**P4 には `@rightAssoc` が 1 つも無い**ので `<assoc=right>` も `prec.right` も使っていない。
**語境界**（D-040）は ANTLR では keyword 規則を `IDENTIFIER` より前に置くだけで無料、
tree-sitter は `word: $ => $.identifier` で得た。

**移植元の版について**: `.g4` と `grammar.js` は **v2**（`@ubnf: 2`、`literalBoundary: word`）から
起こした。一方、速度と受理を測った ubnfc Rust / Java の parser は **v1 の IR**
（`ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json`、
`literalBoundary: none`）から生成された commit 済みの生成物である。
v1 と v2 が同じ言語を表すことは `examples/p4-v2/README.md` と
`docs/reports/2026-09-23-ubnf-v2-left-recursion.md` が固定している。語境界の既定差は
「移植側のほうが厳しい」方向なので、16 fixture の受理には出ない（どれも予約語と同じ綴りの
識別子を使っていない）。

## 2. 受理の一致（16 fixture × 4 実装）

判定は「エラー無しで入力を全消費したか」。ubnfc は `ParseResult::ok` と `consumed_cp`、
ANTLR は `BailErrorStrategy`（エラー回復を一切しない）で例外が出ないこと、
tree-sitter は `ts_node_has_error` が偽で root が入力末尾まで覆っていること。
**AST の形は比べていない。**

### Formula 入口（P4 の `@root`）

| fixture | bytes | ubnfc Rust | ubnfc Java | ANTLR 4 | tree-sitter | 一致 |
|---|---:|---|---|---|---|---|
| `complex` | 332 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `complex-x4` | 1,293 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `complex-x16` | 5,179 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `complex-x64` | 20,923 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `flat-arithmetic` | 1,426 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `large-match` | 1,380 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `valid-basic` | 6 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `valid-multiline` | 40 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `valid-unicode` | 22 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `complex-half` | 169 | 拒否 | 拒否 | 拒否 | 拒否 | ✅ |
| `complex-tail` | 333 | 拒否 | 拒否 | 拒否 | 拒否 | ✅ |
| `invalid-syntax` | 4 | 拒否 | 拒否 | 拒否 | 拒否 | ✅ |
| **`comparison-heavy`** | 179 | **拒否** | **拒否** | **受理** | **受理** | ❌ |
| **`comparison-heavy-x4`** | 716 | **拒否** | **拒否** | **受理** | **受理** | ❌ |
| **`comparison-heavy-x16`** | 2,864 | **拒否** | **拒否** | **受理** | **受理** | ❌ |
| **`comparison-heavy-x64`** | 11,456 | **拒否** | **拒否** | **受理** | **受理** | ❌ |

**12/16 完全一致。残り 4 件の不一致は 1 つの原因に集約される。**

`Expression ::= NumberExpression | BooleanExpression | StringExpression | … ;` は PEG の
順序つき選択で、ubnfc は **先頭候補が成功したら後続が失敗しても他の候補へ戻らない**。
`1+2*3>=7&…` では `NumberExpression` が `1+2*3` まで食べて成功し、
`Formula` の末尾の `EOF` が届かないので全体が失敗する。診断も
`deepestRule=NumberFactor` / `ruleStack=[Formula, Expression, NumberExpression, …]` を指しており、
消費は 0（`consumed_cp=0`）。

- **ANTLR 4** の ALL(\*) は候補の選択に**後続文脈を含む先読み**を使う。`NumberExpression` では
  `EOF` に届かないことが予測段階で分かるので、2 番目の候補 `BooleanExpression` を選んで受理する。
  実質「無制限先読みの PEG」で、PEG より受理が広い。
- **tree-sitter** は GLR なので `NumberExpression` 経由と `BooleanExpression` 経由の解析を
  並行に保持し、生き残ったほうを採る。やはり受理が広い。

これは**文法の書き方の差ではなく解析戦略の差**なので、拒否させるように文法を歪めることはしなかった。

### BooleanExpression 入口

`comparison-heavy` 系は P4 の計測経路でもこちらの入口で解析される。**4/4 一致**。

| fixture | bytes | ubnfc Rust | ubnfc Java | ANTLR 4 | tree-sitter | 一致 |
|---|---:|---|---|---|---|---|
| `comparison-heavy` | 179 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `comparison-heavy-x4` | 716 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `comparison-heavy-x16` | 2,864 | 受理 | 受理 | 受理 | 受理 | ✅ |
| `comparison-heavy-x64` | 11,456 | 受理 | 受理 | 受理 | 受理 | ✅ |

ubnfc Java は Formula 入口の 12 件をこの入口では全て拒否する（当然）。
tree-sitter は開始規則が 1 本なので、この入口用に `P4_TS_ENTRY=BooleanExpression` で
もう一度 generate した別の parser を使っている（`STATE_COUNT` 654、Formula 入口は 882）。

## 3. 追加の拒否入力（10 件）

`examples/p4-v2/scripts/compare.py` の `REJECTS`（診断契約を突く入力）も同じ 4 実装に投げた。
**10/10 で 4 実装とも拒否**、不一致なし。

| 入力 | ubnfc Rust | ubnfc Java | ANTLR 4 | tree-sitter |
|---|---|---|---|---|
| `1 + ` | 拒否 | 拒否 | 拒否 | 拒否 |
| `1 * * 2` | 拒否 | 拒否 | 拒否 | 拒否 |
| `(1 + 2` | 拒否 | 拒否 | 拒否 | 拒否 |
| `(1 * 2` | 拒否 | 拒否 | 拒否 | 拒否 |
| （空入力） | 拒否 | 拒否 | 拒否 | 拒否 |
| `1 + @` | 拒否 | 拒否 | 拒否 | 拒否 |
| `$a & ` | 拒否 | 拒否 | 拒否 | 拒否 |
| `$a ^ ` | 拒否 | 拒否 | 拒否 | 拒否 |
| `'a' + ` | 拒否 | 拒否 | 拒否 | 拒否 |
| `var $a set 1 + ; $a` | 拒否 | 拒否 | 拒否 | 拒否 |

## 4. ANTLR の ALL(*) がこの文法で払う代金

**この文法では SLL が一度も足りない。** `-Dp4antlr.mode=sll`（SLL だけ）で 16 fixture を
かけると、`complex` 系 4 件・`valid-unicode`・`comparison-heavy` 系 4 件が
`Formula` 入口で bail する（＝SLL は受理できない）。P4 の型別の塔は文脈非依存な近似では
区別できないので、ANTLR 推奨の 2 段（SLL → LL）は毎回 SLL のぶんを捨てることになる。

さらに重いのは **LL の全文脈予測が DFA に載らない**ことで、`complex` を何回解析しても
1 parse の所要は下がらない（下の実測、30 回目でも同じ桁）。ANTLR の profiler
（`Parser.setProfile(true)` → `ParseInfo`。`p4-external.sh profile` で再現）で配分を採ると、
時間はほぼ 2 つの決定に集中する。

| fixture / 入口 | decision | 規則 | 呼出し | LL 全文脈へ落ちた回数 | 予測時間 ms | LL 最大先読み | 曖昧 | 文脈依存 |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `complex` / Formula | 98 | `expression` | 10 | **10** | 3,498.8 | 70 | 10 | 1 |
| `complex` / Formula | 92 | `branchExpression` | 2 | **2** | 1,598.8 | 24 | 2 | 2 |
| `complex` / Formula | 64 | `argumentExpression` | 8 | 0 | 44.2 | 0 | 0 | 0 |
| `complex` / Formula | 13 | `stringVariableDeclaration` | 1 | 0 | 26.2 | 0 | 0 | 0 |
| `complex` / Formula | 85 | `stringExpression` | 5 | 0 | 17.9 | 0 | 0 | 0 |
| `comparison-heavy` / BooleanExpression | 68 | `numberExpression` | 94 | 0 | 0.7 | 0 | 0 | 0 |
| `comparison-heavy` / BooleanExpression | 88 | `booleanExpression` | 31 | 0 | 0.1 | 0 | 0 | 0 |

- `complex` を `Formula` 入口で解析すると、**時間の 99% が予測**で、そのうち 97.5% が
  `expression`（P4 の `Expression ::= NumberExpression | BooleanExpression | …`）と
  `branchExpression` の 2 決定に集中する。呼出しはそれぞれ 10 回と 2 回しかないのに、
  **全部が LL 全文脈へ落ち、全部が曖昧**（ANTLR は曖昧なら最小番号の候補を採る＝P4 の候補順と同じ）。
- 同じ parser で `comparison-heavy` を `BooleanExpression` 入口で解析すると、
  **予測は 13%、LL 全文脈への転落は 0 回**。`Expression` / `BranchExpression` がこの入口の
  下に現れないからである。**同じ文法・同じ実装でこれだけ違う**ので、遅さは移植の書き方ではなく
  P4 の型別の順序つき選択という形そのものに起因する（§5 の実測でも Formula 入口 645 ms に対し
  BooleanExpression 入口 0.99 ms、**650 倍**）。

## 5. 速度

- 同一ホスト・同一セッション、`taskset -c 26` で 1 コアに固定、単一 thread。
- **1 入力につき全実装を順に回す**（`p4-external.sh speed`）。**7 run**の中央値と最小。
- **列の意味はツールごとに違う**:

| 実装 | 「認識」列 | 「木」列 |
|---|---|---|
| ubnfc Rust / Java（生成） | `build_ast=false`。木も capture も作らない | `build_ast=true`。**型付き AST**（`@mapping` のクラス） |
| ANTLR 4 | `setBuildParseTree(false)`。木を作らない | ANTLR の **parse tree**（`ParserRuleContext` の木） |
| tree-sitter | **「認識のみ」は存在しない**。`ts_parser_parse`（CST 構築込み）+ `has_error` | 同じ parse に **CST 全 node の深さ優先走査**を足したもの |

`ParseOptions`: ubnfc Rust は既定（`memo=true, predict=true, factor=true, reuse_buffers=true,`
`lexical_cache=false, diagnostics=false, occurrences=false, value_spans=false`）に
`limits.{diagnostics,arena_entries,memo_entries}` だけ広げたもの。ubnfc Java は対応する
`recognize-fast` / `ast-fast`（観測を何も要求しない）。**scope 解決は「scope あり」行では常に有効**で、
切る option は無い（§0）。

### Formula 入口（P4 の `@root`）

| 入力 | bytes | 実装 | 認識 中央値 / 最小 ms | 木 中央値 / 最小 ms | 認識 µs/byte | 割当/parse（ubnfc のみ） |
|---|---:|---|---:|---:|---:|---|
| `complex` | 332 | ubnfc Rust（生成, scope あり） | **0.0696** / 0.0690 | 0.0959 / 0.0934 | 0.210 | 324 回 / 31 KB |
| `complex` | 332 | ubnfc Rust（生成, scope 無効化 IR） | **0.0571** / 0.0553 | 0.0815 / 0.0790 | 0.172 | 31 回 / 19 KB |
| `complex` | 332 | ubnfc Java（生成） | **0.2574** / 0.1993 | 0.3506 / 0.3110 | 0.775 | 71 KB |
| `complex` | 332 | ANTLR 4（two-stage） | **645.5256** / 497.0404 | 693.1208 / 553.1889 | 1944.354 |  |
| `complex` | 332 | tree-sitter | **0.0415** / 0.0381 | 0.0441 / 0.0438 | 0.125 | node 235 |
| `complex-x4` | 1,293 | ubnfc Rust（生成, scope あり） | **0.2811** / 0.2784 | 0.4181 / 0.3883 | 0.217 | 1211 回 / 62 KB |
| `complex-x4` | 1,293 | ubnfc Rust（生成, scope 無効化 IR） | **0.2243** / 0.2212 | 0.3046 / 0.3008 | 0.173 | 102 回 / 21 KB |
| `complex-x4` | 1,293 | ubnfc Java（生成） | **0.7549** / 0.6959 | 1.2182 / 1.0352 | 0.584 | 257 KB |
| `complex-x4` | 1,293 | ANTLR 4（two-stage） | **4119.7117** / 3683.5251 | 4779.0640 / 4031.6215 | 3186.165 |  |
| `complex-x4` | 1,293 | tree-sitter | **0.2004** / 0.1730 | 0.2324 / 0.1814 | 0.155 | node 904 |
| `complex-x16` | 5,179 | ubnfc Rust（生成, scope あり） | **1.5941** / 1.2873 | 2.1610 / 1.6174 | 0.308 | 4741 回 / 192 KB |
| `complex-x16` | 5,179 | ubnfc Rust（生成, scope 無効化 IR） | **1.1358** / 1.0941 | 1.6454 / 1.4564 | 0.219 | 390 回 / 27 KB |
| `complex-x16` | 5,179 | ubnfc Java（生成） | **3.3127** / 3.0563 | 5.7502 / 4.7036 | 0.640 | 1029 KB |
| `complex-x16` | 5,179 | tree-sitter | **0.6136** / 0.5734 | 0.6760 / 0.6628 | 0.118 | node 3580 |
| `complex-x64` | 20,923 | ubnfc Rust（生成, scope あり） | **5.3848** / 5.3660 | 12.4795 / 9.4673 | 0.257 | 18836 回 / 716 KB |
| `complex-x64` | 20,923 | ubnfc Rust（生成, scope 無効化 IR） | **4.4446** / 4.0029 | 9.4366 / 8.5899 | 0.212 | 1542 回 / 54 KB |
| `complex-x64` | 20,923 | ubnfc Java（生成） | **15.2713** / 14.2151 | 24.1741 / 22.5601 | 0.730 | 4099 KB |
| `complex-x64` | 20,923 | tree-sitter | **2.7650** / 2.3725 | 2.7772 / 2.6989 | 0.132 | node 14284 |

**ANTLR の `complex-x16` / `complex-x64` の行が無い**のは、1 parse が 14〜69 秒で
7 run の中央値を採るのに 20 分以上かかるため（§6）。単発の壁時計だけを下のスケーリング表に ※ 付きで置いた。
`割当/parse` 欄は ubnfc だけの実測（Rust は GlobalAlloc の回数と byte、Java は `ThreadMXBean` の byte）。
tree-sitter の欄は割当ではなく **CST の node 数**、ANTLR の欄は空（§6）。

### BooleanExpression 入口

| 入力 | bytes | 実装 | 認識 中央値 / 最小 ms | 木 中央値 / 最小 ms | 認識 µs/byte | 割当/parse（ubnfc のみ） |
|---|---:|---|---:|---:|---:|---|
| `comparison-heavy` | 179 | ubnfc Rust（生成, scope あり） | **0.0425** / 0.0414 | 0.0705 / 0.0690 | 0.238 | 22 回 / 19 KB |
| `comparison-heavy` | 179 | ubnfc Rust（生成, scope 無効化 IR） | **0.0402** / 0.0397 | 0.0701 / 0.0686 | 0.225 | 21 回 / 19 KB |
| `comparison-heavy` | 179 | ubnfc Java（生成） | **0.0830** / 0.0817 | 0.1629 / 0.1567 | 0.464 | 42 KB |
| `comparison-heavy` | 179 | ANTLR 4（two-stage） | **0.9949** / 0.9620 | 0.9843 / 0.9714 | 5.558 |  |
| `comparison-heavy` | 179 | tree-sitter | **0.0331** / 0.0325 | 0.0415 / 0.0405 | 0.185 | node 330 |
| `comparison-heavy-x64` | 11,456 | ubnfc Rust（生成, scope あり） | **2.9041** / 2.8428 | 11.1105 / 8.0962 | 0.253 | 1030 回 / 42 KB |
| `comparison-heavy-x64` | 11,456 | ubnfc Rust（生成, scope 無効化 IR） | **2.9021** / 2.7289 | 17.1801 / 9.2481 | 0.253 | 1029 回 / 42 KB |
| `comparison-heavy-x64` | 11,456 | ubnfc Java（生成） | **6.6504** / 5.9189 | 20.0231 / 16.0862 | 0.581 | 2534 KB |
| `comparison-heavy-x64` | 11,456 | ANTLR 4（two-stage） | **67.0364** / 63.0722 | 69.5712 / 64.8543 | 5.852 |  |
| `comparison-heavy-x64` | 11,456 | tree-sitter | **3.3289** / 2.3121 | 2.9176 / 2.7232 | 0.291 | node 21183 |

### スケーリング（認識の µs/byte）

| 実装 | complex 332 B | x4 1,293 B | x16 5,179 B | x64 20,923 B | 332 B → 20,923 B |
|---|---:|---:|---:|---:|---:|
| tree-sitter | 0.125 | 0.155 | 0.118 | 0.132 | ×1.06 |
| ubnfc Rust（scope 無効化 IR） | 0.172 | 0.173 | 0.219 | 0.212 | ×1.23 |
| ubnfc Rust（scope あり） | 0.210 | 0.217 | 0.308 | 0.257 | ×1.22 |
| ubnfc Java | 0.775 | 0.584 | 0.640 | 0.730 | ×0.94 |
| **ANTLR 4（Formula 入口）** | **1,944** | **3,186** | **3,396** ※ | **2,991** ※ | **×1.54** |

※ ANTLR の x16 / x64 は 7 run の中央値ではなく**単発の解析 3 回の中央値**（§6）。
x16 は 25.4 / 13.7 / 17.6 秒、x64 は 69.4 / 62.6 / 62.7 秒（1 parse あたり）。
ubnfc Rust（scope あり・認識）との比は complex で **9,280 倍**、complex-x64 で **11,600 倍**。


## 6. 測れなかったもの（数値で埋めていない）

| 対象 | 理由 |
|---|---|
| ANTLR の `complex-x16` / `complex-x64` を `Formula` 入口で 7 run 中央値 | 1 parse が 14〜69 秒。7 run × 3 反復でも 20〜25 分／モードかかる。**単発 3 回の壁時計**だけを採り、そう明記した（§5 の ※。x16 は load1 11〜13、x64 は 13〜18 で測った）。それでも「秒の桁」以上の主張はしていない |
| ANTLR / tree-sitter の割当 byte・回数 | 計測機構を入れていない。ubnfc Rust は GlobalAlloc、ubnfc Java は `ThreadMXBean` で採っている。ANTLR 側の数字は空欄のままにした |
| ubnfc の「scope 出力だけを切る」`ParseOptions` | **存在しない。** scope 効果は文法の注釈から生成された parser 本体に埋め込まれていて、実行時に切れない。代わりに IR の `semantics.scope.effects` / `semantics.catalogs` を空にして生成し直した parser を測った（受理と消費は 16 fixture すべてで scope ありと完全一致することを確認済み） |
| AST / CST の形の比較 | 3 つのツールが作るものは別物（型付き AST / `ParserRuleContext` の木 / tree-sitter の CST）。issue の指示どおり比べていない |
| ANTLR の `sll` 単独モードの速度 | この文法では受理できない（16 fixture 中 9 件で bail）ので速度を出す意味がない。受理の可否だけ確認した（§4） |
| tree-sitter の「認識のみ」 | 存在しない。`ts_parser_parse` は必ず CST を作る |
| `comparison-heavy-x64` の AST モードの run 間ばらつき | ubnfc Rust は 1 parse で 5.5 MB・26,715 回の確保があり、中央値が run ごとに 8〜24 ms へ振れる。scope あり 11.11 ms と scope 無効化 IR 17.18 ms の大小は**このばらつきの中**で、差の主張はしない（最小値は 8.10 / 9.25 ms） |

## 7. 測定条件

- host: AMD Ryzen 9 7950X（32 論理 CPU）、WSL2 Linux 6.18.33.2。**共有機（別作業が並走）**。
  `taskset -c 26` で 1 コアに固定、単一 thread。
  load average（1 分値）: speed 開始 11.7 → 終了 8.05（各行の `load1` 列と `# load average` 行に実測）。
- rustc 1.98.1 / cargo 1.98.1、既定の release profile。
  1 run は `UBNFC_CMP_RUN_SECONDS=1.0` ぶんの反復、warmup は反復数の半分（最大 2,000）、**7 run** の中央値・最小。
  確保回数と byte は GlobalAlloc を数える別 parse（計時外）。
- Java: Oracle JDK 21.0.9 / HotSpot 64-Bit。**ANTLR・ubnfc Java とも `-Xms2g -Xmx2g`**
  （ubnfc Java だけ `-Xss16m` を追加。2026-09-23 の報告は `-Xms512m` だったので、そちらの数字とは
  JVM 設定が違う）。ubnfc Java の warmup は 2,000 parse、**7 run**。
  ANTLR の warmup は「2,000 parse または 10 秒のどちらか早いほう」（`-Dp4antlr.warmupSeconds`）で、
  実際に回った回数は出力の `warmup` 列にある（`complex` は 12〜13 回、`comparison-heavy` は 2,000 回）。
  反復数は**warmup の後**の 1 parse から `0.5 秒 / parse` で決める。
- ANTLR 4.13.2（Maven Central の complete jar）。`BailErrorStrategy`（エラー回復をしない）、
  既定は **2 段（SLL → LL）**。`removeErrorListeners()` 済み。生成は警告 0
  （`TinyExpressionP4Parser.java` 298,471 byte / `TinyExpressionP4Lexer.java` 50,657 byte）。
- tree-sitter CLI 0.25.10（`da6fe9beb4f7f67beb75914ca8e0d48ae48d6406`）、runtime も v0.25.10 のソースを
  `cc -O2 -std=gnu11` で driver ごとリンク（gcc 11.4.0）。`LANGUAGE_VERSION 15`、外部 C scanner なし。
- 入力は `examples/p4-rust/tests/fixtures/*.tiny`。ubnfc Java 用の複製と md5 一致を確認
  （`complex` `762028b0a40214f4702b1ad4f471e514`、`complex-x64` `f37bad4cde36a0556a1a3a905c156d2d`）。
- 生成物（ANTLR の `*.java`、tree-sitter の `src/parser.c`）は commit していない。`$WORK` 配下に作る。
  外部ツールの取得は #26 の `scripts/fetch-external-tools.sh`（版と sha256 が pin されている。
  置き場は `.gitignore` 済みの `benchmarks/external/`）。

**再現の確認**: 同じセッションで `p4-external.sh prepare|accept|negatives|speed` を
**もう一度**（`scripts/fetch-external-tools.sh` 経由で取り直した jar / CLI で）通した。
`accept` と `negatives` の出力は 1 byte も違わなかった。速度は**そのとき機械の負荷が高く**
（load average 開始 12.98 → 終了 24.13、上の表は 11.66 → 8.05）、
`complex` の認識で ubnfc Rust 0.0696 → 0.0811 ms、tree-sitter 0.0415 → 0.0444 ms、
ANTLR 645.5 → 672.1 ms と全体に 5〜17% 悪化した。**実装間の順序は同じ**
（tree-sitter < ubnfc Rust < ubnfc Java ≪ ANTLR の Formula 入口）。
表には負荷の低いほうの pass を載せている（そちらの ubnfc Rust `complex-x4` 認識 0.281 ms は、
2026-09-23 の報告 family A の 0.281 ms と一致する）。

## 8. 目についたこと

1. **tree-sitter が全入力で最速**（認識 0.118〜0.291 µs/byte）。しかも tree-sitter の「認識」は
   CST 構築込みで、木の全走査を足しても `complex` で 0.0415 → 0.0441 ms とほぼ無料。
   ubnfc Rust の認識（木を作らない）が 0.0696 ms なので、**CST を作りながらなお 1.7 倍速い**。
   ただし tree-sitter は scope 解決をしないし、型付き AST も作らない（§0）。
2. **ANTLR は `Formula` 入口だけが壊滅的**。`complex` で 645 ms、ubnfc Rust の **9,280 倍**。
   同じ parser・同じ文法でも `BooleanExpression` 入口なら 0.99 ms（**650 倍速い**）。
   profiler の配分（§4）と合わせて、原因は P4 の
   `Expression ::= NumberExpression | BooleanExpression | StringExpression | …` という
   **型別の順序つき選択**にあり、ALL(\*) がそこで毎回 LL 全文脈へ落ちる。
   全文脈の予測結果は DFA に載らないので、**何回解析しても速くならない**（30 回目も同じ桁）。
   PEG は先頭候補にコミットするのでこの代金を払わない — 受理が狭いこと（§2 の 4 件）と
   速いことは同じ性質の裏表である。
3. **scope 解決の代金は認識時間の 17〜29%、確保回数では 10 倍前後**。
   `complex` の認識で 0.0696 → 0.0571 ms（−18%）、確保 324 回 → 31 回。`complex-x16` は
   1.594 → 1.136 ms（−29%）、`complex-x64` は 5.38 → 4.44 ms（−17%）で 18,836 回 → 1,542 回。ANTLR と tree-sitter はこの仕事を**まったくしていない**ので、
   両者と比べるときはこの差を引いて読む（引いてもなお tree-sitter のほうが速い）。
4. **ubnfc Java は Rust の 2.1〜3.7 倍**（認識）。割当は `complex-x64` の AST で 12.5 MB と、
   Rust の 3.26 MB の 3.8 倍。2026-09-23 の報告 §7-1 と同じ方向（Java backend の最適化が未着手）。
5. **超線形なものは無い**（ANTLR を除く）。tree-sitter ×1.06、ubnfc Rust ×1.22、ubnfc Java ×0.94。
   ANTLR は 332 B → 20,923 B で ×1.54。
6. **受理は 16/16 中 12 件で 4 実装が完全一致**し、残り 4 件も原因は 1 つ（PEG のコミット）。
   追加の拒否入力 10 件も 4 実装すべてで拒否。**文法の移植そのものは合っている**と言ってよい。
