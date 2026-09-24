# JSON: Jackson / Gson / serde_json との比較

2026-09-24。issue [#68](https://github.com/opaopa6969/ubnfc/issues/68)。ubnfc は `2217625`（PR #64 = D-072 の一括走査を含む master）。

#26 / #48 の JSON 比較の相手は、文法から parser を生成する ANTLR4 と tree-sitter だけだった。
この報告は、**JSON 専用に手で最適化されたライブラリ**（Java の Jackson と Gson、Rust の serde_json）を
同じ入力・同じホスト・同じセッションで並べる。**負けるのは最初から分かっている**。
知りたかったのは、どれだけ負けるかと、**何を作っていて負けるのか**だった。

> **先に書いておく注意。** Jackson・Gson・serde_json は JSON 専用の手書きスキャナで、数値と文字列には
> 速い経路があり、**位置（span）も診断も作らない**。ubnfc は汎用の文法（`examples/json/json.ubnf`）から
> 生成した parser で、typed AST の各ノードに span があり、失敗したときは期待集合つきの診断を返す。
> Jackson の `JsonNode` / Gson の `JsonElement` / `serde_json::Value` は「値の木」で、
> typed AST ではない（規則名もノード型も span も無い）。`readValue(Map)` と `fromJson(Map)` は
> `LinkedHashMap` / `ArrayList` / `Double` などを確保する。各表の「作るもの」列はこの差を書いたもの。

---

## 1. 速度

### 1.1 読み方（必ず先に読むこと）

- **主に見るのは最小値**。今回の走行中、ホストでは他のエージェントの `rustc -O3`・maven・
  JUnit が並行していて、load average は **11〜21** だった（§3.3）。計測は `taskset -c 30` で
  1 コアに固定したが、固定していないプロセスも core 30 に割り付けられ、計測中の JVM が
  CPU 35% しか取れていない瞬間を実際に見ている。**そのせいで中央値は最小値の多くは 2〜4 倍、ひどいセルでは 30 倍に膨らんでいる**。
  最小値のほうは静かなセッションで測った過去の値とよく合う（1 MB の最小値で ANTLR4 0.0115 対
  #48 報告の 0.0113、tree-sitter 0.0501 対 0.0503、ubnfc Rust 0.0133 対 0.0153 µs/byte）。
  中央値の列は参考として残すが、**実装間の比は最小値から計算している**。
- 各セルは 7 run × 2 周。**入力ごとに 15 の (実装, モード) を順番に回し**、それを 2 周した。
  中央値は 2 周それぞれの中央値の平均、最小値は 14 run 全体の最小値。
- MB/s は最小値から出した値（`bytes / min`）。µs/byte も同じく最小値から出した。
- 割当は 1 parse あたりの byte 数。Java は `ThreadMXBean.getThreadAllocatedBytes`、Rust は
  GlobalAlloc を数える allocator で、どちらも計時とは別の 1 回の parse で数えた。tree-sitter は C なので測っていない。

### 1.2 1 MB（`json-1m.json`、1,000,112 byte）

| 実装 / モード | 作るもの | µs/byte 最小 | MB/s（最小から） | µs/byte 中央値 | 割当 B/parse |
|---|---|---:|---:|---:|---:|
| serde_json 1.0.151 / `from_str::<IgnoredAny>` | **何も作らない** | **0.0008** | **1,197** | 0.0011 | 8 |
| Jackson 2.22.3 / streaming（`nextToken` だけ回す） | **何も作らない** | 0.0015 | 678 | 0.0067 | 760 |
| Jackson / `readValue(…, Object.class)` | Map / List / 数値の箱 | 0.0032 | 317 | 0.0060 | 9,331,608 |
| Jackson / `readTree` | 値の木（`JsonNode`・span なし） | 0.0035 | 286 | 0.0112 | 9,289,400 |
| Gson 2.14.0 / `JsonParser.parseString` | 値の木（`JsonElement`・span なし） | 0.0039 | 258 | 0.0093 | 10,654,616 |
| Gson / `fromJson(…, Object.class)` | Map / List / 数値の箱 | 0.0046 | 216 | 0.0051 | 9,967,672 |
| serde_json / `from_str::<Value>` | 値の木（`Value`・span なし） | 0.0055 | 183 | 0.0150 | 13,159,868 |
| ANTLR4 4.13.2 / 認識のみ | 何も作らない（token 列は作る） | 0.0115 | 87.3 | 0.0339 | 27,067,376 |
| **ubnfc Rust / 認識のみ** | **何も作らない**（成否と診断） | **0.0133** | **75.1** | 0.0377 | 76,557,137 |
| ANTLR4 / parse tree + walk | 規則の木（span 相当の token 位置つき） | 0.0307 | 32.5 | 0.1020 | 43,763,576 |
| tree-sitter 0.25.10 / CST 構築 | CST（必ず作られる。byte 位置つき） | 0.0501 | 20.0 | 0.3674 | — |
| **ubnfc Java / 認識のみ** | **何も作らない**（成否と診断） | **0.0524** | **19.1** | 0.0741 | 163,343,912 |
| tree-sitter / CST + 全走査 | 同上 + `TSTreeCursor` 全走査 | 0.0717 | 13.9 | 0.2837 | — |
| **ubnfc Rust / typed AST** | **typed AST（span・診断つき）** | **0.1813** | **5.5** | 0.5895 | 523,358,141 |
| **ubnfc Java / typed AST** | **typed AST（span・診断つき）** | **0.2305** | **4.3** | 0.5745 | 350,851,880 |

**1 MB の比（ms の最小値から計算）**

| ubnfc | 相手 | 倍率 |
|---|---|---:|
| Rust 認識 0.0133 | serde_json `IgnoredAny` 0.0008 | **15.9 倍遅い** |
| Rust 認識 0.0133 | Jackson streaming 0.0015 | **9.0 倍遅い** |
| Rust typed AST 0.1813 | serde_json `Value` 0.0055 | **33 倍遅い** |
| Java 認識 0.0524 | Jackson streaming 0.0015 | **36 倍遅い** |
| Java typed AST 0.2305 | Jackson `readTree` 0.0035 | **66 倍遅い** |
| Java typed AST 0.2305 | Gson `parseString` 0.0039 | **60 倍遅い** |

### 1.3 10 KB（`json-10k.json`、10,066 byte）

| 実装 / モード | 作るもの | µs/byte 最小 | MB/s（最小から） | µs/byte 中央値 | 割当 B/parse |
|---|---|---:|---:|---:|---:|
| serde_json / `IgnoredAny` | 何も作らない | **0.0009** | **1,071** | 0.0017 | 8 |
| Jackson / streaming | 何も作らない | 0.0033 | 303 | 0.0065 | 720 |
| Jackson / `readTree` | 値の木（span なし） | 0.0048 | 207 | 0.1622 ※ | 92,336 |
| Gson / `parseString` | 値の木（span なし） | 0.0064 | 155 | 0.0078 | 107,800 |
| serde_json / `Value` | 値の木（span なし） | 0.0067 | 148 | 0.0115 | 143,769 |
| Jackson / `readValue(Map)` | Map | 0.0070 | 144 | 0.0170 | 92,704 |
| Gson / `fromJson(Map)` | Map | 0.0087 | 114 | 0.0116 | 100,912 |
| ANTLR4 / 認識のみ | 何も作らない | 0.0150 | 66.8 | 0.1314 | 273,152 |
| **ubnfc Rust / 認識のみ** | 何も作らない | **0.0207** | **48.3** | 0.0407 | 20,705 |
| **ubnfc Rust / typed AST** | typed AST（span・診断） | **0.0498** | **20.1** | 0.1488 | 674,155 |
| tree-sitter / CST + 走査 | CST | 0.0712 | 14.0 | 0.1818 | — |
| **ubnfc Java / 認識のみ** | 何も作らない | **0.0768** | **13.0** | 0.1391 | 1,642,368 |
| tree-sitter / CST 構築 | CST | 0.0800 | 12.5 | 0.1532 | — |
| ANTLR4 / parse tree + walk | 規則の木 | 0.1163 | 8.6 | 0.2031 | 442,616 |
| **ubnfc Java / typed AST** | typed AST（span・診断） | **0.2062** | **4.9** | 0.5217 | 3,476,120 |

※ Jackson `readTree` の 10 KB は 1 周目の中央値が 0.316 µs/byte と外れていて（最小値は 0.0048）、
負荷の揺れで読めない。10 KB は ANTLR4 parse tree の最小値（0.1163）も 1 MB（0.0307）より悪く、
この入力サイズは全体に揺れが大きい。**10 KB の比は 1 MB より信用度が低い**。

10 KB の比（最小値）: ubnfc Rust 認識は serde_json `IgnoredAny` の **22 倍**・Jackson streaming の
**6.3 倍**、Rust typed AST は serde_json `Value` の **7.4 倍**・Jackson `readTree` の **10 倍**、
Java 認識は Jackson streaming の **23 倍**、Java typed AST は `readTree` の **43 倍**。

### 1.4 1 KB と 100 KB（最小値の µs/byte）

| 実装 / モード | 1 KB | 100 KB |
|---|---:|---:|
| serde_json / `IgnoredAny` | 0.0015 | 0.0009 |
| Jackson / streaming | 0.0040 | 0.0014 |
| Jackson / `readTree` | 0.0086 | 0.0031 |
| Jackson / `readValue(Map)` | 0.0053 | 0.0030 |
| Gson / `parseString` | 0.0075 | 0.0037 |
| Gson / `fromJson(Map)` | 0.0106 | 0.0060 |
| serde_json / `Value` | 0.0053 | 0.0070 |
| ANTLR4 / 認識のみ | 0.0338 | 0.0138 |
| ANTLR4 / parse tree + walk | 0.0689 | 0.0304 |
| tree-sitter / CST 構築 | 0.0777 | 0.0411 |
| ubnfc Rust / 認識のみ | 0.0360 | 0.0175 |
| ubnfc Rust / typed AST | 0.0533 | 0.2114 |
| ubnfc Java / 認識のみ | 0.1381 | 0.0729 |
| ubnfc Java / typed AST | 0.1589 | 0.2045 |

全 4 サイズ × 15 モード × 2 周の生の値は
[`examples/json/evidence/speed-json-libraries.csv`](../../examples/json/evidence/speed-json-libraries.csv) にある。

### 1.5 数字の横に置く注意

- **「何も作らない」同士でも中身は違う。** serde_json の `IgnoredAny` は字句を読み飛ばすだけで、
  数値は桁を検査するが値にしない。文字列も escape を検査するだけで String を作らない
  （割当 8 byte は `from_str` の内部 1 回分）。Jackson streaming も同じく、token 型を返すだけで、
  `getText()` / `getNumberValue()` を呼ばないかぎり数値も文字列も実体化しない。ubnfc の「認識のみ」は
  PEG の評価をすべて行い、失敗したら期待集合つきの診断を組み立てる（成功時もそのための mark を持つ）。
- **ubnfc の typed AST は span と診断を持つ**。値の木（`JsonNode` / `JsonElement` / `Value`）は
  位置を持たないので、「どの byte から来た値か」を後から問えない。ubnfc Rust typed AST の
  1 parse 523 MB は事象 arena と mapping の確保（#48 報告で既出。別 issue）で、
  「値の木を作ること」そのものの値段ではない。
- **Map モードの割当は値の木とほぼ同じ**（1 MB で 9.3〜10.7 MB）。Jackson の `readValue(Map)` が
  `readTree` より速いのは、`JsonNode` の型階層より `LinkedHashMap`/`ArrayList` のほうが軽いから。
- **Java の JIT**: Jackson / Gson は「500 反復以上かつ 5 秒以上」で暖機した。ANTLR4 / ubnfc Java は既存の
  driver のまま「500 反復、最長 30 秒」。ubnfc Java の 1 MB typed AST は 30 秒の打ち切りで
  **40 / 78 反復しか暖機できていない**（CSV の warmup 列）。暖機不足は ubnfc Java に不利な方向。
- **JMH は使っていない**。ANTLR / ubnfc の行を同じ走行で並べ直すため、既存の手書きループ
  （`AntlrRunner` / `UbnfcRunner` と同じ計時）に揃えた。代わりに暖機回数を全行で記録している。
- **simd-json は未計測**。`~/.cargo/registry` に無く、crates.io はこの環境から遮断されている。

---

## 2. 正しさ（JSONTestSuite 318 件）

同じ commit（`1ef36fa`）の `test_parsing` 318 件（`y_` 95 / `n_` 188 / `i_` 35）を全実装に流した。
Java の実装はすべて同じ厳密 UTF-8 復号（`CodingErrorAction.REPORT`）を通してから String を渡す。
受理は「先頭から **1 つの JSON 値**を読み切り、後ろに空白しか残らない」こと。
Jackson は streaming の token を数えて top-level の値が 1 つだけであることを見ている
（`ObjectMapper.readValue` の既定は**末尾の余りを黙って捨てる**ので、判定には使っていない）。

| 実装 / 設定 | `y_` 受理 / 95 | `n_` 拒否 / 188 | `n_` を受理した数 | `i_` 受理 | `i_` 拒否 |
|---|---:|---:|---:|---:|---:|
| ubnfc Rust（生成） | 95 | **188** | 0 | 21 | 14 |
| ubnfc Java（生成） | 95 | **188** | 0 | 21 | 14 |
| ANTLR4 4.13.2 | 95 | **188** | 0 | 21 | 14 |
| tree-sitter 0.25.10 | 95 | **188** | 0 | 22 | 13 |
| Jackson 2.22.3 既定（`new JsonFactory()`） | 95 | **188** | 0 | 21 | 14 |
| Jackson 厳格（`JsonReadFeature` の 14 個を全部 off） | 95 | **188** | 0 | 21 | 14 |
| Jackson 寛容（同じ 14 個を全部 on） | 95 | 141 | **47** | 21 | 14 |
| Gson 2.14.0 既定（`JsonParser.parseString`＝LENIENT） | 95 | 102 | **86** | 21 | 14 |
| Gson `Strictness.STRICT` | 95 | **188** | 0 | 21 | 14 |
| serde_json 1.0.151 既定（`IgnoredAny`） | 95 | **188** | 0 | 21 | 14 |

- **Jackson 2.x の既定はすでに厳格**で、既定と「全部 off」の判定は 318 件すべて同じだった。
  off にした 14 個: `ALLOW_JAVA_COMMENTS` `ALLOW_YAML_COMMENTS` `ALLOW_BACKSLASH_ESCAPING_ANY_CHARACTER`
  `ALLOW_SINGLE_QUOTES` `ALLOW_UNESCAPED_CONTROL_CHARS` `ALLOW_UNQUOTED_FIELD_NAMES` `ALLOW_RS_CONTROL_CHAR`
  `ALLOW_LEADING_DECIMAL_POINT_FOR_NUMBERS` `ALLOW_LEADING_PLUS_SIGN_FOR_NUMBERS`
  `ALLOW_LEADING_ZEROS_FOR_NUMBERS` `ALLOW_NON_NUMERIC_NUMBERS` `ALLOW_TRAILING_DECIMAL_POINT_FOR_NUMBERS`
  `ALLOW_MISSING_VALUES` `ALLOW_TRAILING_COMMA`。
- **Gson の既定は寛容**。`JsonParser.parseString` は `LEGACY_STRICT` の reader を `LENIENT` へ上げてから読むので、
  `n_` の 46%（86 / 188）を受理する。`Strictness.STRICT`（Gson 2.11+）にすると 188 件すべてを拒否する。
- `i_` で実装が割れるのは 2 件だけ:
  - `i_structure_500_nested_arrays` — **Gson は 2 設定とも拒否**（`Nesting limit 255 reached`）。他は全部受理
    （Jackson の既定の入れ子上限は 1,000。serde_json の `IgnoredAny` は `ignore_value` が明示スタックの反復で読み飛ばすので
    深さの上限に触れない。`Value` へ読むと既定の再帰上限 128 で拒否されるはずで、**serde_json の `i_` 判定は `IgnoredAny` のもの**）。
  - `i_structure_UTF-8_BOM_empty_object` — **Gson（2 設定とも）と tree-sitter が受理**、他は拒否。
- 318 行 × 10 設定の判定は [`examples/json/evidence/jsontestsuite-verdicts-jsonlibs.tsv`](../../examples/json/evidence/jsontestsuite-verdicts-jsonlibs.tsv)。

### 2.1 寛容モードが受理する `n_` ファイル

**Jackson（寛容 = 14 個全部 on）の 47 件。** 各 feature を**1 つだけ**既定へ足して流し直し、
どの feature で通ったかを切り分けた（[`jsontestsuite-jackson-features.tsv`](../../examples/json/evidence/jsontestsuite-jackson-features.tsv)）。
46 件は単独の feature で説明でき、残る 1 件は 2 つの組み合わせ。

| feature | 件数 | ファイル（`n_` と `.json` を省略） |
|---|---:|---|
| `ALLOW_MISSING_VALUES` | 8 | `array_comma_and_number` `array_double_comma` `array_double_extra_comma` `array_extra_comma` `array_just_comma` `array_missing_value` `array_number_and_comma` `array_number_and_several_commas` |
| `ALLOW_BACKSLASH_ESCAPING_ANY_CHARACTER` | 7 | `string_backslash_00` `string_escape_x` `string_escaped_ctrl_char_tab` `string_escaped_emoji` `string_incomplete_surrogate_escape_invalid` `string_invalid_backslash_esc` `string_unicode_CapitalU` |
| `ALLOW_TRAILING_DECIMAL_POINT_FOR_NUMBERS` | 6 | `number_-2.` `number_0.e1` `number_2.e+3` `number_2.e-3` `number_2.e3` `number_real_without_fractional_part` |
| `ALLOW_UNQUOTED_FIELD_NAMES` | 4 | `object_non_string_key` `object_non_string_key_but_huge_number_instead` `object_repeated_null_null` `object_unquoted_key` |
| `ALLOW_TRAILING_COMMA` | 3 | `object_trailing_comma` `array_extra_comma`* `array_number_and_comma`* |
| `ALLOW_JAVA_COMMENTS` | 3 | `object_trailing_comment` `object_trailing_comment_slash_open` `structure_object_with_comment` |
| `ALLOW_LEADING_DECIMAL_POINT_FOR_NUMBERS` | 3 | `number_.2e-3` `number_neg_real_without_int_part` `number_starting_with_dot` |
| `ALLOW_LEADING_ZEROS_FOR_NUMBERS` | 3 | `number_-01` `number_neg_int_starting_with_zero` `number_with_leading_zero` |
| `ALLOW_NON_NUMERIC_NUMBERS` | 3 | `number_NaN` `number_infinity` `number_minus_infinity` |
| `ALLOW_UNESCAPED_CONTROL_CHARS` | 3 | `string_unescaped_ctrl_char` `string_unescaped_newline` `string_unescaped_tab` |
| `ALLOW_SINGLE_QUOTES` | 2 | `object_single_quote` `string_single_quote` |
| `ALLOW_YAML_COMMENTS` | 2 | `object_with_trailing_garbage` `structure_trailing_#` |
| `ALLOW_LEADING_PLUS_SIGN_FOR_NUMBERS` | 1 | `number_+1` |
| `ALLOW_UNQUOTED_FIELD_NAMES` + `ALLOW_SINGLE_QUOTES` | 1 | `object_key_with_single_quotes`（`{key: 'value'}`。どちらか片方では通らない） |
| `ALLOW_RS_CONTROL_CHAR` | 0 | — |

\* 2 件は `ALLOW_MISSING_VALUES` でも `ALLOW_TRAILING_COMMA` でも通る（件数は両方に数えた。重複を除くと 47 件）。

**Gson 既定（LENIENT）の 86 件。** Gson には feature 単位のスイッチが無いので、ファイルの中身を見て
どの寛容規則で通ったかを分類した（分類は目視）。

| 寛容規則 | 件数 | ファイル（`n_` と `.json` を省略） |
|---|---:|---|
| 引用符の無い語を文字列（または数値）として読む | 64 | 不正な数値 45 件（`number_++` `number_+1` `number_+Inf` `number_-01` `number_-1.0.` `number_-2.` `number_-NaN` `number_.-1` `number_.2e-3` `number_0.1.2` `number_0.3e+` `number_0.3e` `number_0.e1` `number_0_capital_E+` `number_0_capital_E` `number_0e+` `number_0e` `number_1.0e+` `number_1.0e-` `number_1.0e` `number_1eE2` `number_2.e+3` `number_2.e-3` `number_2.e3` `number_9.e+` `number_Inf` `number_NaN` `number_U+FF11_fullwidth_digit_one` `number_expression` `number_hex_1_digit` `number_hex_2_digits` `number_infinity` `number_invalid+-` `number_invalid-negative-real` `number_minus_infinity` `number_minus_sign_with_trailing_garbage` `number_neg_int_starting_with_zero` `number_neg_real_without_int_part` `number_neg_with_garbage_at_end` `number_real_garbage_after_e` `number_real_without_fractional_part` `number_starting_with_dot` `number_with_alpha` `number_with_alpha_char` `number_with_leading_zero`）、ほか 19 件（`array_just_minus` `array_star_inside` `incomplete_false` `incomplete_null` `incomplete_true` `multidigit_number_then_00` `object_bad_value` `string_accentuated_char_no_quotes` `string_single_string_no_double_quotes` `structure_U+2060_word_joined` `structure_angle_bracket_.` `structure_angle_bracket_null` `structure_ascii-unicode-identifier` `structure_capitalized_True` `structure_null-byte-outside-string` `structure_number_with_trailing_garbage` `structure_single_star` `structure_unicode-identifier` `structure_whitespace_U+2060_word_joiner`） |
| 配列の欠けた値・余分なカンマ（`null` で埋める） | 8 | `array_comma_and_number` `array_double_comma` `array_double_extra_comma` `array_extra_comma` `array_just_comma` `array_missing_value` `array_number_and_comma` `array_number_and_several_commas` |
| 単引用符・引用符の無いキー | 7 | `object_key_with_single_quotes` `object_non_string_key` `object_non_string_key_but_huge_number_instead` `object_repeated_null_null` `object_single_quote` `object_unquoted_key` `string_single_quote` |
| 文字列の中の生の制御文字 | 3 | `string_unescaped_ctrl_char` `string_unescaped_newline` `string_unescaped_tab` |
| 空の文書を `JsonNull` として返す | 3 | `single_space` `structure_no_data` `structure_UTF8_BOM_no_data` |
| コメント | 1 | `structure_object_with_comment` |

Gson の寛容モードは「引用符の無い語」を**区切り文字が来るまで**文字列として読むので、壊れた数値・
途中で切れた `tru` / `nul`・`<null>`・NUL byte まで値として通る。
**Gson の既定を JSON の検証に使ってはいけない**、という結論になる（`Strictness.STRICT` なら 188 / 188）。

---

## 3. 条件

### 3.1 版と取得元

| もの | 版 | 取得元 / sha256 |
|---|---|---|
| Jackson core / databind | 2.22.3 | Maven Central。`8a501126…b094c` / `556db543…e0e0a9` |
| Jackson annotations | 2.22（2.x 系の最新。2.22.3 は core/databind だけのパッチ） | `21ddb598…a127a2d0` |
| Gson | 2.14.0 | `2cbd119b…faa2c36f` |
| error_prone_annotations（Gson の pom 依存） | 2.48.0 | `b49c5c95…a49b9c855` |
| serde_json / serde | 1.0.151 / 1.0.229 | `~/.cargo/registry`（`--offline`。crates.io は遮断） |
| simd-json | — | registry に無い → **未計測** |
| ANTLR4 / tree-sitter / JSONTestSuite | 4.13.2 / 0.25.10 / `1ef36fa` | `scripts/fetch-external-tools.sh`（#26 の pin のまま） |

jar の完全な sha256 は [`scripts/fetch-json-libraries.sh`](../../scripts/fetch-json-libraries.sh) に pin してある。
取得先は `.gitignore` 済みの `benchmarks/external/`。

### 3.2 入力

#26 と同じ 4 本（`benchmarks/scripts/make-json-inputs.py` が JSONTestSuite の `y_*` から決定的に作る）。
この走行で作り直したものは [`inputs.sha256`](../../examples/json/evidence/inputs.sha256) と byte 一致した。

| 入力 | byte | sha256 |
|---|---:|---|
| `json-1k.json` | 1,067 | `d447f5b338fad2f4418b6b762c6cb6bd451ef33bd36551010252af6c300ddc90` |
| `json-10k.json` | 10,066 | `7ab96e563a716f06876abdead3c7125bc945e730831dc5790f99a60ef01f02c1` |
| `json-100k.json` | 100,039 | `d7a4d32f25212e517f1a4e48cf684326c3874fb49e566f2f541911e9798d2ee1` |
| `json-1m.json` | 1,000,112 | `ae482dce4b9b5f5ed2dad2f988e1d0f675663faf61f22c31d52dccbefaf9849c` |

### 3.3 ホストと計測

- Linux 6.18.33.2-microsoft-standard-WSL2 x86_64、32 論理 CPU、`taskset -c 30`、単一 thread。
- JDK 21.0.9（Oracle HotSpot）、全 Java 実装 `-Xms2g -Xmx2g`。Rust は `cargo 1.98.1`、`--release`。
- 1 run = 0.5 秒ぶんの反復、7 run × 2 周。ubnfc Rust は `UBNFC_ARENA_ENTRIES=200000000`（#26 と同じ）。
- 暖機: Jackson / Gson は 500 反復以上かつ 5 秒以上（最長 30 秒）。ANTLR4 / ubnfc Java は 500 反復・最長 30 秒。
  Rust は反復数の半分（bench_util.rs の既定）。
- load average（1 分 / 5 分 / 15 分）: 1 周目開始 6.25 / 4.40 / 7.22 → 1 周目終了 14.21 / 15.50 / 15.09、
  2 周目途中の最大 21.10 → 終了 11.91 / 14.42 / 14.93
  （[`speed-json-libraries-load.txt`](../../examples/json/evidence/speed-json-libraries-load.txt)）。

### 3.4 再現

```sh
WORK=$(mktemp -d)
scripts/fetch-external-tools.sh && scripts/fetch-json-libraries.sh
WORK=$WORK benchmarks/scripts/prepare-external.sh            # ANTLR4 / tree-sitter / ubnfc / 入力
WORK=$WORK benchmarks/scripts/run-json-libraries.sh prepare  # Jackson / Gson / serde_json の driver
WORK=$WORK benchmarks/scripts/run-json-libraries.sh bench    # 1 周（2 周目はもう一度）
WORK=$WORK benchmarks/scripts/check-jsontestsuite.sh         # ubnfc / ANTLR4 / tree-sitter の判定
WORK=$WORK benchmarks/scripts/run-json-libraries.sh check    # Jackson 3 設定 / Gson 2 設定 / serde_json
WORK=$WORK benchmarks/scripts/run-json-libraries.sh check-features  # Jackson の feature 別の切り分け
```

driver は `benchmarks/java/JsonLibRunner.java`（Jackson / Gson）と `benchmarks/src/family_jsonlib.rs`（serde_json）。
どちらも ubnfc の cargo workspace / maven build には入っていない（`$WORK` 配下でだけ build する）。

---

## 4. 結論

- **速度の負けは 1 桁から 2 桁**。1 MB の最小値で、ubnfc Rust の認識は serde_json `IgnoredAny` の 16 倍・
  Jackson streaming の 9 倍遅い（ANTLR4 認識とは 1.2 倍差で、生成系の中では並ぶ）。値の木を作る側では、
  ubnfc Rust の typed AST が serde_json `Value` の 33 倍、ubnfc Java の typed AST が Jackson `readTree` の 66 倍遅い。
  差のほとんどは認識ではなく **typed AST の構築（事象 arena と mapping、1 MB で 350〜520 MB の確保）** にある。
- **正しさでは並ぶ**。ubnfc の 2 backend・Jackson（既定・厳格）・Gson STRICT・serde_json はどれも
  `y_` 95/95・`n_` 188/188。**Gson の既定（LENIENT）は `n_` を 86 件受理する**ので検証には使えない。
  Jackson の寛容 feature を全部 on にすると 47 件受理する（feature 別の内訳は §2.1）。
- 次に効くのは認識ではなく typed AST 側（#58 の Java 直接構築、#59 の Rust arena/SoA）。専用ライブラリとの
  差を 1 桁に縮めるには、値の木と同程度の確保（1 MB で約 10 MB）まで typed AST の確保を落とす必要がある。

メモリ用 3 行:
1. JSON 1 MB 最小値: ubnfc Rust 認識 0.0133 µs/byte は serde_json IgnoredAny 0.0008 の 16 倍・Jackson streaming 0.0015 の 9 倍遅い。typed AST は serde_json Value の 33 倍（Rust）/ Jackson readTree の 66 倍（Java）遅い。
2. JSONTestSuite: ubnfc・Jackson 既定/厳格・Gson STRICT・serde_json は全部 y 95/95・n 188/188。Gson 既定（LENIENT）は n_ を 86 件、Jackson 寛容全 on は 47 件受理。Jackson 2.x の既定はすでに厳格。
3. 計測は load 11〜21 の共有機で中央値が最小値の 2〜4 倍（最大 30 倍）に膨らんだ。最小値が過去の静かなセッションと一致するので最小値で読む。差の主因は typed AST の確保（1 MB で 350〜520 MB 対 値の木 10 MB）。
