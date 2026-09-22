# 事実集: 組込み token の受理範囲 / ベンチ fixture / corpus 形式

> 2026-09-22 時点、unlaxer-parser と tinyexpression のソースから確認。「未確認」は要追試。

## A. 組込み token クラスの受理範囲（ubnfc が内蔵スキャナとして再実装する対象）

| UBNF で参照される名前 | 実装（unlaxer-common） | 受理範囲（ubnfc の scan 仕様） |
|---|---|---|
| `NumberParser` | `elementary/NumberParser` = Optional(Sign) Choice(digits '.' digits \| digits '.' \| digits \| '.' digits) Optional(Exponent) | `[+-]? ( D+ '.' D+ \| D+ '.' \| D+ \| '.' D+ ) ( [eE] [+-]? D+ )?`。**先頭符号・末尾ドット・先頭ドット・指数を受理**する（Java AST の `int` 変換は別契約で失敗し得る） |
| `IdentifierParser` | `clang/IdentifierParser` = AlphabetUnderScore ZeroOrMore(AlphabetNumericUnderScore) | `[A-Za-z_][A-Za-z0-9_]*`（Alphabet の Unicode 範囲は**未確認**: ASCII のみか `Character.isLetter` か） |
| `SingleQuotedParser` | `QuotedParser(SingleQuoteParser)` = quote ZeroOrMore(Choice(EscapeInQuoted, non-quote)) quote | `' ( \\ . \| [^'] )* '`。EscapeInQuoted = BackSlash + WildCardString（WildCardString の長さは**未確認**：1 文字か貪欲か） |
| `QuotedParser` | 同上（二重引用符） | `" ( \\ . \| [^"] )* "` |
| `EndOfSourceParser` | `LazyZero(WildCardCharacter)` | 入力末尾でのみ成功、ゼロ幅 |
| `DigitParser` | `MappedSingleCharacterParser("0123456789")` | `[0-9]` 1 文字 |
| `WordParser("x")` (リテラル `'x'`) | code point 列の完全一致。**単語境界チェックは無い** | 境界規則は生成器側（D-002、astra 確認中） |
| `CI('w')` | `IgnoreCaseWordParser` = `WordParser(w, ignoreCase=true)` | `equalsIgnoreCase` 相当 |
| `UNTIL('t')` | `WildCardStringTerminatorParser` = ZeroOrMore(WildCardString, terminator=**MatchOnly**(Word(t))) | 終端 `t` の直前まで消費し、**`t` 自体は消費しない**。終端未検出時の契約（EOF まで消費して成功か失敗か）は**未確認** |
| `NEGATION('cs')` | 生成クラス（`isMatch(int)` / `isMatch(char)`） | 集合外の 1 code point。空集合は任意 1 cp、EOF で失敗 |
| `CHAR_RANGE('a','b')` | 生成クラス | 両端含む、`char` 境界（BMP 非 surrogate） |
| `LOOKAHEAD('p')` / `NEGATIVE_LOOKAHEAD('p')` | `MatchOnly(Word)` / `Not(Word)` | ゼロ幅 |
| `ANY` | `WildCardCharacterParser` | 任意 1 cp、EOF で失敗 |
| `EOF` / `EMPTY` | | ゼロ幅 |
| `REGEX('p')` | `dsl/runtime/RegexTokenParser` = `matcher.lookingAt()` | Java `Pattern`、現在位置固定、ゼロ幅一致は失敗。ubnfc は正規部分集合を DFA 化、それ以外は Java では `Pattern` 委譲・Rust は生成時エラー（案） |
| `AlphabetParser` 等 posix | `MappedSingleCharacterParser` | 1 文字集合 |
| tinyexpression 独自（`StringLiteralParser`, `CodeStartParser`, `CodeEndParser`） | 別 repo の Java クラス | `Extern` token → ユーザー実装フック（D-008） |

## B. ベンチ fixture（tinyexpression/benchmarks/fixtures、JMH `P4ParserBenchmark` が使用）

| fixture | bytes | 行 | 括弧深さ | 備考 |
|---|---:|---:|---:|---|
| complex.tiny | 332 | 15 | 2 | var 宣言 2 + if/else + match + メソッド宣言、Unicode 含む |
| complex-x4 / x16 / x64 | 1,293 / 5,179 / 20,923 | 57 / 225 / 897 | 2 | スケーリング確認用 |
| comparison-heavy(.tiny/-x4/-x16/-x64) | 179 / 716 / 2,864 / 11,456 | 1 | 0 | 1 行の比較式 |
| flat-arithmetic | 1,426 | 1 | 0 | |
| large-match | 1,380 | 68 | 0 | |

- JMH: `tinyexpression/src/jmh/.../P4ParserBenchmark.java`（`Mode.AverageTime`、fixture を `@Param`）
- criterion: `tinyexpression/rust/tinyexpression-rs/benches/parser.rs`
- 既存記録（別 revision、直接比較不可）: Java facade complex 12.32 ms / complex-x64 958 ms、Rust complex 1.426 ms

→ ubnfc の性能目標は **complex / complex-x64 / comparison-heavy-x64** の 3 本で、
既存 Java facade（memo 込み）と既存 Rust に対する比を採る。x64 で超線形が出ないことも見る。

## C. 流用する corpus（unlaxer-dsl/src/test/resources）

形式が 5 系統ある。harness は系統ごとに loader を持つ。

| 系統 | ファイル | 件数 | 形 |
|---|---|---:|---|
| 文法内蔵・AST 完全比較 | nested-captures, rule-trivia, scope-annotations | 10 / 33 / 14 cases | `{grammar:str, cases:[{input, ast{type,span,fields}, captures, declarations}]}` |
| token 単体 | primitives, lexical | 109 / 78 | `{token, body, cases:[{input, value, prefix?}]}` |
| ルール単体 | semantic-cardinality (+pure-alias), tiny-code-fence, tiny-string-token, mixed-values(3) | 94+69 / 73 / 39 / 49 | `{name, body, cases:[{input, accepted, span, fields}]}` 等 |
| 文法ファイル別置き | evolution/{0..3}/Evolution.ubnf + conformance.json(37 入力) + diagnostics.json(8) 、associative(24)、right-associative(28)、cardinality(14)、numeric-capture(13) | | 入力列 + 期待値（value / javaOffset / javaInt） |
| 操作列 | scope-store（8 fixture の op 列）、catalog | | ScopeStore の操作と観測 |

合計 ≈ 700 ケース。`golden/*.java.txt` は既存生成物のスナップショットなので **使わない**
（ubnfc は生成物の形が違う）。`catalog/corpus.json` は非標準エスケープを含み Python json で
読めない → loader は寛容パーサが要る（要確認）。
