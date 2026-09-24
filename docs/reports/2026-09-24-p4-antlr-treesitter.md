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

---

# 追補: 流儀で書き直した文法で測り直した（2026-09-24、issue #39）

上の §1〜§8 は **P4 の規則を 1:1 で写した**移植の測定で、そのまま残してある（PR #38 の数字）。
この節は**同じ言語を ANTLR / tree-sitter の流儀で書き直した文法**を足して測り直したもので、
**別セッションの新規測定**である。機械が混んでいた時間帯（別作業の P4 ベンチが並走）に当たったので
**pass を 2 回**採り、負荷の低いほうを表にした（追補-5）。
**上の §5 の表と縦に並べて読んではいけない**（load average: §5 は 11.7 → 8.05、この節は pass 1 が 23.8 →（最大 66）→ 23.2、
pass 2 が 22.7 → 38.0 で測定中は 9.4〜27）。
1:1 移植と流儀版の比は**同じ回転（1 入力につき全実装を順に回す）の中で採っている**ので、そこだけは比較してよい。

| | |
|---|---|
| 文法（流儀版） | `examples/p4-antlr/TinyExpressionP4Idiomatic.g4` / `examples/p4-treesitter/grammar.idiomatic.js` |
| driver | `examples/p4-antlr/P4AntlrIdiomaticDriver.java` / `examples/p4-treesitter/driver.c`（共用） |
| 再現 | `benchmarks/scripts/p4-antlr-idiomatic.sh prepare\|accept\|negatives\|superset\|profile\|speed` |
| 生の出力 | [`2026-09-24-p4-antlr-treesitter/`](2026-09-24-p4-antlr-treesitter/) の `idiomatic-*`（受理・拒否入力・上位集合・profile・速度 2 pass の 6 ファイル） |
| 設計の対応表・上位集合の一覧 | [`examples/p4-antlr/README.md` §ANTLR / tree-sitter 流に書き直した版](../../examples/p4-antlr/README.md) |

## 追補-1. 書き直しの中身

P4 の 4 本の塔（`NumberExpression` / `StringExpression` / `BooleanExpression` /
`ObjectExpression`）と、それを選ぶ順序つき選択
`Expression ::= NumberExpression | BooleanExpression | …` を、**1 本の左再帰 `expression`**
（候補順＝優先順位の梯子）に統合した。型別に分かれていた宣言・メソッド宣言・external 呼出し・
match・組込み関数もそれぞれ 1 本に畳んだ（規則 120 本 → 37 本。ANTLR が生成する parser は 298,476 byte → 103,214 byte）。lexer 規則と token 定義は
1:1 移植と**バイト単位で同一**にしてあるので、変わったのは parser だけである。

優先順位の梯子（高いものから）: 後置 `[a:b:c]` / `.method()` → `*` `/` → `+` `-` →
`== != <= >= < >` → `^` → `&` → `|` → `?:`。
`<assoc=right>` / `prec.right` は `?:` の 1 箇所だけで、P4 に対応物は無い（P4 の三項は
引数位置か括弧付きに限られ、入れ子の裸三項を受理しない）。

## 追補-2. 受理: 16 fixture は 1:1 移植と完全一致、ただし言語は上位集合

**16 fixture の受理/拒否は 4 つの文法（1:1 の ANTLR・tree-sitter、流儀版の ANTLR・tree-sitter）で完全に一致する。**
`comparison-heavy` 系 4 件は、1:1 移植と同じく **`Formula` 入口で受理される**（§2 の ❌ 行と同じ）。
統合後は `1+2*3>=7&…` が「比較段 → `&` 段」という 1 本の導出になるので、
そもそも塔を選ぶ決定が存在しない。`examples/p4-v2/scripts/compare.py` の拒否入力 10 件も
流儀版 2 本とも 10/10 で拒否。

| 入力 | 1:1 ANTLR | 流儀 ANTLR | 1:1 tree-sitter | 流儀 tree-sitter |
|---|---|---|---|---|
| `complex` / `-x4` / `-x16` / `-x64` | 受理 | 受理 | 受理 | 受理 |
| `flat-arithmetic` / `large-match` | 受理 | 受理 | 受理 | 受理 |
| `valid-basic` / `-multiline` / `-unicode` | 受理 | 受理 | 受理 | 受理 |
| `comparison-heavy` 系 4 件（Formula 入口） | 受理 | 受理 | 受理 | 受理 |
| `complex-half` / `complex-tail` / `invalid-syntax` | 拒否 | 拒否 | 拒否 | 拒否 |

受理する**言語**は P4 の上位集合になる。型の誤りが構文の誤りでなくなるためで、
次の 14 件は **1:1 移植が拒否し、流儀版 2 本がともに受理する**（14/14 一致）。

`1 + true` / `'a' * 2` / `true & 1 * 2` / `1 == 2 == 3` / `$c ? 1 : 2` / `pow(1)` / `min()` /
`toNum(1, 2)` / `$a.startsWith(1)` / `not(1)` / `(1 + 2).trim()` / `$a[0:1:]` /
`match { $a > 1 -> 1, default -> "x" }` / `var $a as number set "text"; $a`

理由は `examples/p4-antlr/README.md` の表に 1 件ずつ書いた。

## 追補-3. ANTLR の予測: 全文脈への転落が 0 になる

`Parser.setProfile(true)` → `ParseInfo`（`p4-antlr-idiomatic.sh profile`）。
**12 件すべてで `LL_Fallback` 0・曖昧 0・文脈依存 0・`LL_MaxLook` 0**。

| fixture / 入口 | 1:1 移植（§4 より） | 流儀版 |
|---|---|---|
| `complex` / Formula | `expression` 決定 98 が 10 回中 **10 回**全文脈へ転落、`branchExpression` 決定 92 が 2 回中 **2 回**。予測が時間の **99%**、うち 97.5% がこの 2 決定 | 全決定で転落 **0**。予測は時間の **22%**、最大は `expression` 決定 12 の 9.7 ms（呼出し 39 回） |
| `comparison-heavy` / BooleanExpression（流儀版は Expression） | 転落 0、予測 13% | 転落 0、予測 **1%** |

流儀版の profiler の上位（転落・曖昧・文脈依存はすべて 0 なので列を省く）:

| fixture | 予測が占める割合 | 最大の決定 | 呼出し |
|---|---:|---|---:|
| `complex` | 22% | `expression`（決定 12） | 39 |
| `complex-x4` | 8% | `expression`（決定 12） | 156 |
| `complex-x16` | 6% | `expression`（決定 11） | 143 |
| `complex-x64` | 7% | `matchExpression`（決定 18） | 128 |
| `flat-arithmetic` | 15% | `expression`（決定 11） | 255 |
| `large-match` | 7% | `matchExpression`（決定 18） | 64 |
| `comparison-heavy` / Formula | 23% | `expression`（決定 12） | 125 |
| `comparison-heavy` / Expression | 1% | `expression`（決定 12） | 125 |
| `comparison-heavy-x64` / Formula | 10% | `expression`（決定 11） | 4,031 |

（profiling 自体が 1 桁ぶん高いので、絶対値ではなく**配分**だけが読める。）

**`PredictionMode.SLL` + `BailErrorStrategy` だけで 16 fixture すべてが 2 段と同じ判定になる。**
1:1 移植は `complex` 系 4 件・`valid-unicode`・`comparison-heavy` 系 4 件の計 9 件で bail していた（§4）。
つまり ANTLR 推奨の 2 段（SLL → LL）の 1 段目が、この文法で初めて仕事をするようになった。

## 追補-4. tree-sitter 側

同じ統合を `grammar.idiomatic.js` に入れた（速い書き換えで済んだので測定まで行った）。

| | `grammar.js`（1:1） | `grammar.idiomatic.js`（流儀） |
|---|---:|---:|
| `conflicts` | 23 件 | **1 件**（`Formula` 入口のみ。`Expression` 入口は 0） |
| `prec.dynamic` | 27 箇所 / 7 規則 | **0 箇所** |
| `STATE_COUNT`（Formula / 第 2 入口） | 882 / 654 | **307 / 183** |
| `LARGE_STATE_COUNT` | 103 / 84 | 3 / 2 |
| `src/parser.c` | 1,018,051 / 748,372 byte | **356,765 / 223,062 byte** |
| `complex` の CST node 数 | 235 | 217 |

残る 1 件（`[$.variable_ref]`）は型の話ではなく、`Formula` 入口で `$a string` が
「型キーワード付きの `variable_ref`」か「`variable_ref` の次に返り値型 `string` の
`method_declaration`」かが 1 token で決まらないというもの。`grammar.js` も `Formula` 入口でだけ
同じ 1 件を要求する。つまり **23 件のうち 22 件は型別の塔が重なることだけが原因**だった。

## 追補-5. 速度

計測規約は §5・§7 と同じ（`taskset -c 26` で 1 コア固定、単一 thread、**1 入力につき全実装を順に回す**、
**7 run** の中央値と最小）。**この機械は共有で、この日は別作業の P4 ベンチが並走していた**ため、
**pass を 2 回**採って負荷の低いほう（pass 2）を表にした。各行の `load1` は driver が実測したものである。
ANTLR / tree-sitter の行は driver が 1 行ごとに load1 を出すので併記した。

- pass 1: load average 23.8 →（途中最大 66）→ 23.2。x16 / x64 の行が最大 2〜3 倍悪化している。
- **pass 2（下の表）: load average 22.7 → 38.0、実際の測定中は 9.4〜27**。

**§5 の表（PR #38、load 11.7 → 8.05）とは別セッションなので縦に並べてはいけない。**
比べてよいのは**同じ回転の中の行同士**である。

### Formula 入口

| 入力 | bytes | 実装 | 認識 中央値 / 最小 ms | 木 中央値 / 最小 ms | 認識 µs/byte | load1（認識） |
|---|---:|---|---:|---:|---:|---:|
| `complex` | 332 | ubnfc Rust（生成, scope あり） | **0.1068** / 0.0952 | 0.1404 / 0.1189 | 0.322 | — |
| `complex` | 332 | ubnfc Java（生成） | **0.2440** / 0.1811 | 0.4748 / 0.3610 | 0.735 | — |
| `complex` | 332 | **ANTLR 4 1:1 移植**（two-stage） | **993.1540** / 624.0755 | 953.9276 / 613.6842 | 2991.4 | 21.97 |
| `complex` | 332 | **ANTLR 4 流儀**（two-stage） | **1.8338** / 1.6234 | 2.4346 / 1.6253 | 5.52 | 20.41 |
| `complex` | 332 | **ANTLR 4 流儀**（SLL のみ） | **1.5236** / 1.4421 | 1.5317 / 1.3839 | 4.59 | 18.76 |
| `complex` | 332 | tree-sitter 1:1 移植 | **0.0446** / 0.0408 | 0.0515 / 0.0484 | 0.134 | 18.02 |
| `complex` | 332 | **tree-sitter 流儀** | **0.0352** / 0.0301 | 0.0363 / 0.0336 | 0.106 | 17.02 |
| `complex-x4` | 1,293 | ubnfc Rust（生成, scope あり） | **0.3308** / 0.2973 | 0.4201 / 0.4018 | 0.256 | — |
| `complex-x4` | 1,293 | ubnfc Java（生成） | **0.4153** / 0.3430 | 1.2165 / 0.8838 | 0.321 | — |
| `complex-x4` | 1,293 | ANTLR 4 1:1 移植 | 測っていない（§5 で 4,119 ms。1 parse が数秒で回転に載らない） | | | |
| `complex-x4` | 1,293 | **ANTLR 4 流儀**（two-stage） | **5.8668** / 4.8983 | 5.6078 / 5.5108 | 4.54 | 16.08 |
| `complex-x4` | 1,293 | **ANTLR 4 流儀**（SLL のみ） | **7.3185** / 6.6085 | 5.0122 / 4.9398 | 5.66 | 15.04 |
| `complex-x4` | 1,293 | tree-sitter 1:1 移植 | **0.1586** / 0.1503 | 0.1848 / 0.1740 | 0.123 | 14.15 |
| `complex-x4` | 1,293 | **tree-sitter 流儀** | **0.1153** / 0.1085 | 0.1659 / 0.1396 | 0.089 | 18.56 |
| `complex-x16` | 5,179 | ubnfc Rust（生成, scope あり） | **1.3180** / 1.2838 | 1.8964 / 1.5300 | 0.254 | — |
| `complex-x16` | 5,179 | ubnfc Java（生成） | **4.5129** / 1.3982 | 9.3808 / 4.2580 | 0.871 | — |
| `complex-x16` | 5,179 | **ANTLR 4 流儀**（two-stage） | **23.7571** / 20.2038 | 21.0721 / 19.4539 | 4.59 | 22.44 |
| `complex-x16` | 5,179 | **ANTLR 4 流儀**（SLL のみ） | **22.7916** / 20.0628 | 26.9105 / 22.3188 | 4.40 | 18.67 |
| `complex-x16` | 5,179 | tree-sitter 1:1 移植 | **0.6935** / 0.6048 | 0.8089 / 0.7460 | 0.134 | 16.29 |
| `complex-x16` | 5,179 | **tree-sitter 流儀** | **0.4825** / 0.4619 | 0.6167 / 0.5212 | 0.093 | 15.87 |
| `complex-x64` | 20,923 | ubnfc Rust（生成, scope あり） | **10.9862** / 8.3077 | 22.7109 / 16.5449 | 0.525 | — |
| `complex-x64` | 20,923 | ubnfc Java（生成） | **6.5057** / 5.4309 | 16.9134 / 16.3784 | 0.311 | — |
| `complex-x64` | 20,923 | **ANTLR 4 流儀**（two-stage） | **81.9592** / 80.0409 | 80.2346 / 79.3340 | 3.92 | 10.54 |
| `complex-x64` | 20,923 | **ANTLR 4 流儀**（SLL のみ） | **87.0831** / 79.8898 | 83.5367 / 80.4823 | 4.16 | 10.86 |
| `complex-x64` | 20,923 | tree-sitter 1:1 移植 | **2.3893** / 2.3144 | 3.0029 / 2.7106 | 0.114 | 10.09 |
| `complex-x64` | 20,923 | **tree-sitter 流儀** | **1.8202** / 1.7406 | 2.4169 / 2.0839 | 0.087 | 9.38 |

**`complex` で 1:1 移植 993.15 ms → 流儀 1.83 ms、同じ回転の中で 542 倍**（SLL のみなら 652 倍）。
これが issue #39 の答えである。**「ANTLR が遅い」のではなく「PEG 前提の型別の塔を ALL(\*) に
そのまま持ち込むと遅い」**。ただし流儀版でも **ubnfc Rust の 17 倍、tree-sitter 流儀の 52 倍**
かかっており（`complex`）、桁は縮まったが順序は変わらない。

**ANTLR 流儀版の x16 / x64 は 7 run の中央値が採れた**（1 parse が 24〜82 ms なので、
1:1 移植のように「単発の壁時計」で代用する必要がない。§6 の制約が 1 つ消えた）。

### スケーリング（認識の µs/byte、pass 2）

| 実装 | 332 B | 1,293 B | 5,179 B | 20,923 B | 332 B → 20,923 B |
|---|---:|---:|---:|---:|---:|
| tree-sitter 流儀 | 0.106 | 0.089 | 0.093 | 0.087 | ×0.82 |
| tree-sitter 1:1 | 0.134 | 0.123 | 0.134 | 0.114 | ×0.85 |
| ubnfc Rust（scope あり） | 0.322 | 0.256 | 0.254 | 0.525 | ×1.63 |
| ubnfc Java | 0.735 | 0.321 | 0.871 | 0.311 | ×0.42 |
| **ANTLR 4 流儀**（two-stage） | **5.52** | **4.54** | **4.59** | **3.92** | **×0.71** |
| ANTLR 4 1:1 移植（§5 より） | 1,944 | 3,186 | 3,396 ※ | 2,991 ※ | ×1.54 |

流儀版は **µs/byte が入力長でほぼ一定**（むしろ下がる）で、1:1 移植の ×1.54 という増加が
消えた。全文脈予測は DFA に載らないので入力が長いほど効いていたが、その予測自体が無くなったからである。
ubnfc Rust の ×1.63 は §5 の ×1.22 より悪いが、これは x64 の行を採ったときの負荷
（ばらつきは最小値 8.31 ms〜中央値 10.99 ms）であって、この節で新しい主張はしない。

### BooleanExpression（1:1）/ Expression（流儀）入口

| 入力 | bytes | 実装 | 認識 中央値 / 最小 ms | 木 中央値 / 最小 ms | 認識 µs/byte | load1（認識） |
|---|---:|---|---:|---:|---:|---:|
| `comparison-heavy` | 179 | ubnfc Rust（生成, scope あり） | **0.0444** / 0.0435 | 0.0782 / 0.0737 | 0.248 | — |
| `comparison-heavy` | 179 | ubnfc Java（生成） | **0.0415** / 0.0359 | 0.1310 / 0.1209 | 0.232 | — |
| `comparison-heavy` | 179 | ANTLR 4 1:1 移植 | **2.9675** / 2.0901 | 1.1774 / 1.0023 | 16.58 | 10.99 |
| `comparison-heavy` | 179 | **ANTLR 4 流儀**（two-stage） | **1.1545** / 1.0850 | 1.0922 / 0.9804 | 6.45 | 10.87 |
| `comparison-heavy` | 179 | **ANTLR 4 流儀**（SLL のみ） | **1.0523** / 1.0011 | 0.9718 / 0.9461 | 5.88 | 10.49 |
| `comparison-heavy` | 179 | tree-sitter 1:1 移植 | **0.0404** / 0.0337 | 0.0562 / 0.0510 | 0.226 | 9.87 |
| `comparison-heavy` | 179 | **tree-sitter 流儀** | **0.0618** / 0.0575 | 0.0737 / 0.0713 | 0.345 | 11.45 |
| `comparison-heavy-x64` | 11,456 | ubnfc Rust（生成, scope あり） | **7.1725** / 6.0742 | 24.2616 / 23.2980 | 0.626 | — |
| `comparison-heavy-x64` | 11,456 | ubnfc Java（生成） | **10.0951** / 7.7880 | 34.7906 / 28.2389 | 0.881 | — |
| `comparison-heavy-x64` | 11,456 | ANTLR 4 1:1 移植 | **101.9177** / 88.0574 | 106.8374 / 95.5775 | 8.90 | 26.95 |
| `comparison-heavy-x64` | 11,456 | **ANTLR 4 流儀**（two-stage） | **89.8110** / 82.5520 | 105.5065 / 85.1440 | 7.84 | 24.97 |
| `comparison-heavy-x64` | 11,456 | **ANTLR 4 流儀**（SLL のみ） | **110.8702** / 88.5291 | 108.6924 / 92.7572 | 9.68 | 30.10 |
| `comparison-heavy-x64` | 11,456 | tree-sitter 1:1 移植 | **3.1886** / 2.3960 | 3.1207 / 2.9325 | 0.278 | 27.54 |
| `comparison-heavy-x64` | 11,456 | **tree-sitter 流儀** | **12.3821** / 2.7080 | 13.1623 / 10.1611 | 1.081 | 36.01 |

**この入口では差が小さい**（`comparison-heavy` で 2.57 倍、`-x64` では 1.13 倍で、最小値どうしなら
ほぼ同じ）。理由は §4 のとおりで、**1:1 移植も `BooleanExpression` 入口では全文脈へ落ちていない**からである。
統合で消えたのは `Expression` / `BranchExpression` の型別選択だけで、そこを通らない入口では
得られるのは ATN が小さくなったぶんだけになる。**塔の統合が効くのは `Formula` 入口である**という
§4 の読みが、そのまま速度で確かめられた。

`comparison-heavy-x64` の tree-sitter 流儀の行（12.38 ms）は **load1 36〜38 で採った 1 行**で、
最小値は 2.71 ms、pass 1 では中央値 4.66 ms である。**この 1 行だけは負荷の影響が大きく、
1:1 移植（3.19 ms）より遅いという主張はしない。**

## 追補-6. 何を構文で決めるか（P4/ubnfc と ANTLR 流のトレードオフ）

この 2 本の文法の違いは速度ではなく、**型をどの層で決めるか**である。P4/ubnfc は型を
**構文で**決める。`NumberExpression` / `StringExpression` / `BooleanExpression` /
`ObjectExpression` の 4 本の塔があり、`$x` がどの塔に入れるかは scope 宣言
（`@declares` / `@backref` / `@scopeTree`）が解決する。だから `1 + true` は構文エラーで、
parser が返す診断はそのまま「その位置に数値式が来るはず」という型の話になり、
AST は `@mapping` で型ごとのクラスへ落ちる（後段に型推論の層が要らない）。
代わりに、文法は型の数だけ塔が増え、PEG の順序つき選択が「どの塔から試すか」を担うので、
`Expression ::= NumberExpression | BooleanExpression | …` の先頭候補にコミットして
`comparison-heavy` を拒否する（§2）——受理が狭いことと速いことは同じ性質の裏表である。
ANTLR 流は逆に、**構文は 1 本の優先順位の梯子だけ**を決め、型は後段（listener / 型検査）に
委ねる。規則は 120 本から 37 本に減り、決定は局所的になって ALL(\*) の全文脈予測が消える
（`complex` で 542 倍、追補-5）。代わりに `1 + true` は構文としては正しい木になり、
**型の誤りを見つける層を別に書く必要がある**（P4 なら文法が無料でやっていた仕事）。
どちらが正しいという話ではなく、**型検査器を書くか、文法に型を書くか**の選択で、
その選択が予測のコストと診断の質の両方を同時に決めている。

## 追補-7. この節で測れなかったもの

| 対象 | 理由 |
|---|---|
| ANTLR 1:1 移植の `complex-x4` / `-x16` / `-x64` を `Formula` 入口で | §6 と同じ。1 parse が数秒〜数十秒で、1 入力につき全実装を回す規約に載らない。§5 の ※ 付きの値を参照する |
| ubnfc Rust の「scope 無効化 IR」 | この節の主題（文法の書き方）と直交するので回転から外した。§5 にある |
| ANTLR / tree-sitter の割当 byte・回数 | §6 と同じ。計測機構を入れていない |
| 負荷の低い環境での再測定 | 共有機で別作業が並走していた。pass を 2 回採って低いほうを載せ、各行に `load1` を付けた。絶対値を §5 と並べて読まないこと |
