# stax の自己拡張の境界と UBNF v3 最小提案（#16）

段階1の仮説を、段階3の実行系と段階4の生成 LSP がある状態で再評価した。
**現行 stax は固定構文と quotation / macro を維持し、今回 front を追加しない。**
外部語彙の宣言効果・scope の版付き memo は既に存在する。
「固定 PEG では Forth を原理的に書けない」「外から来た名前は登録できない」という
段階1の広い断定は採らない。固定した語列の認識、言語側の辞書の実行、生成 parser の
構文・宣言・typed AST 観測を別々に評価する。

UBNF v3 については、不変な初期語彙 snapshot と外部宣言の出所を扱う共通契約を
最小提案として残す。**提案・未実装・未承認**であり、今回の検証成功には数えない。
解析中に可変な辞書へ依存する認識や、動的な IR / AST 型の生成は採用しない。

## 1. 検証対象と再現コマンド

基準は段階4の merge `bdd87bd`。本番の `examples/stax/stax.ubnf`、front、両 backend、
`scripts/external-pins.env` は変更していない。追加は例・再現・テスト・この報告。

```sh
RUSTUP_TOOLCHAIN=stable cargo test --offline -p ubnfc-stax --test boundaries
RUSTUP_TOOLCHAIN=stable cargo test --offline -p ubnfc-harness --test stax_boundaries
```

前者は [cases.json](../../examples/stax/boundaries/cases.json) の11ケースについて
生成 Rust parser の構文判定と、CLI の終了コード・stdout / エラー理由を検証する。
後者は [check-boundaries.sh](../../examples/stax/scripts/check-boundaries.sh) から
2つの評価文法を strict 検証し、Java / Rust の生成物を一時ディレクトリへ組み、
[generated.json](../../examples/stax/boundaries/generated.json) と観測を照合する。
実測には stable Rust、JDK 21.0.9 / Maven 3.9.9 を使った。

`which gforth` は終了コード1・出力無し。このマシンでは Forth 実装を実行していない。
以下の Forth 側の期待は標準に基づくもので、stax と生成 parser の実測とは区別する。
manifest の `forthReferences[].executed` は false のままにする。

## 2. Forth の3例と意味保存の範囲

標準の [`(`](https://forth-standard.org/standard/core/p) は `)` までの入力を読む immediate word。
[`."`](https://forth-standard.org/standard/core/Dotq) は引用符までの文字列を読み、
その表示を実行時へ組み込む。ここでは colon 定義内で使い、標準で未定義とされる
interpretation 時の振る舞いには依存しない。
[`CREATE`](https://forth-standard.org/standard/core/CREATE) は入力から名前を読み、
データ領域に結び付いた定義を作る。[`DOES>`](https://forth-standard.org/standard/core/DOES) は
CREATE で作った直近の定義の実行動作を置き換える。

### 2.1 `( ... )` — 固定構文で用途を代替

Forth の入力と標準上の期待:

```forth
: value ( 注釈) 2 3 + ; value
\ スタックに5。注釈は最初の ) まで。
```

stax の実測:

```stax
: main ( -- ) /* 注釈 */ 2 3 + int_to_str print ;
// exit 0、stdout = "5"
```

`( Int -- Int )` は効果の固定構文として使い、`identity` を通した7の表示も検証した。
一方、本体に `( note )` を置いたケースは構文エラー・exit 1。
保持したのは注釈を読み飛ばす用途と、効果を明示する用途である。
Forth と同じ括弧コメントの全 byte 列、入力領域の操作、`(` の語としての差し替えは保持しない。

### 2.2 `."` — quotation + macro で代替

Forth の入力と標準上の期待:

```forth
: greet ." hello" ; greet
\ greet の実行時にhelloを表示。
```

stax の実測:

```stax
macro say ( [ -- Str ] -- [ -- ] ) let text [ text call print ] ;
: main ( -- ) [ "hello" ] say call ;
// exit 0、stdout = "hello"
```

macro は引用を変換し、表示は返された引用を call した時に走る。
コンパイル時に print する macro は「展開時の I/O は禁止」で exit 1、stdout は空。
`." hello"` をそのまま stax 本体に書いた場合も構文エラー・exit 1。
表示の内容と遅延した実行時点を保持し、入力ストリームを読む力や parsing word の再定義は保持しない。
文字列は stax の固定 STRING token で先に解析する。Forth の文字列 escape / parse area の
すべてを再現したとは扱わない。

### 2.3 `CREATE ... DOES>` — 現契約では不可能（現行 stax の macro / Loader の範囲）

Forth の入力と標準上の期待:

```forth
: constant CREATE , DOES> @ ;
42 constant answer answer
\ answer という語を作り、その実行で42を積む。
```

この定義形を stax へ持ち込んだケースは構文エラー・exit 1。
項の並びとして `42 constant answer` だけを本体に置けば構文は受理するが、
checker が `constant` を未定義として拒否する。構文受理は動的宣言を作った証拠にならない。
macro が返す `[ ... ]` の内側に `: answer ... ;` を置く形も、TopLevel は Term ではないため構文拒否。
文字列 `"made"` を捕捉して AST の名前にする試みも「捕捉値は引用に限定」で拒否する。

固定した代替は動く:

```stax
: answer ( -- Int ) 42 ;
: main ( -- ) answer int_to_str print ;
// exit 0、stdout = "42"
```

保持したのは、既知の名前に定数を与えて後で使う用途だけ。
利用者定義の defining word が次の入力名を読み、新しい辞書項目を作る力は保持しない。
この「不可能」の対象は **現行 stax の Term 引用を返す macro・固定した Loader / 辞書 API**。
UBNF の extern に言語固有の処理系を実装することや、解析後の意味処理で辞書を作ることまで
原理的に禁止する意味ではない。既存の WordDef AST を後処理で構築する設計も別途あり得るが、
現行 stax の macro 契約には含めず、実装済みとも数えない。

## 3. 語列の認識と語彙依存の意味を分ける

Forth のテキスト処理は、名前を読み、辞書を調べ、解釈 / コンパイル状態に従って動作する。
これは標準の [3.4 Text interpreter / 3.4.1 Parsing](https://forth-standard.org/standard/usage#usage:parsing)
に記載された手順であり、単なる文字列の分割と同一ではない。

[word-stream.ubnf](../../examples/stax/boundaries/word-stream.ubnf) は1行・ASCII 空白区切りの
非空語を宣言的 token で読む固定文法である。上記の Forth 入力3例と `日本語 >> 2dup!` の
計4例を両 backend が全文受理し、語列 AST の型・値・span が一致した。
`answer` も普通の文字列として現れるが、**全4例とも declarations=[]**。
コメント、文字列のまとまり、数値変換、語の実行、CREATE の宣言効果は実装していない。
Forth 全体の字句規則や意味論の実装成功には数えない。

stax の `WordRef ::= IDENT` も、未定義の一般名を構文としては受け取る。
それを生成 scope が Warning とし、段階3の checker が拒否する。
「どの byte 列を受け取れるか」と「どの語を知っているか」を分ければ、固定 PEG と
動的な言語側 VM を併用できる。VM で新しく作った宣言を生成 parser の scope / LSP に
自動的に反映できるかは、さらに別の契約である。

## 4. 既存の語彙機能が固定される時点

| 機能 | 固定されるもの | 解析時に変わるもの・限界 |
|---|---|---|
| literal / builtin token | front で IR 化した綴り・認識種別、生成したコード | 入力位置。実行中の辞書変更で literal 自体は変わらない |
| 宣言的 token | 閉じた LexicalExpression、import 解決、参照展開を IR / 生成時に固定 | 入力、token 内 CAPTURE / SAME_AS の局所値。host の可変辞書は読まない |
| extern token | 論理 ID、cursor / effect 契約、memoSafety、任意の FIRST 上界を IR に固定 | 呼出し時に登録した scanner が認識と replay 可能な効果を返す |
| @catalog | 文法由来の候補と context の静的 metadata | 外部語彙を初期 scope へ読み込む指示ではない。現行 LSP の補完の厳密な context 絞り込みでもない |
| @declares / @symbolRef | capture site と効果順を IR に固定 | 今回の入力から得た名前と位置を scope に登録・解決する。新しい綴りそのものは生成時に固定しない |
| stax の WordDef / macro | 文法と typed AST の型、語の辞書 snapshot の規則 | 語名や値、引用の本体は入力による。macro は Term 引用を返し、TopLevel の動的追加 API は無い |

根拠: [宣言的 token の契約](../design/08-declarative-tokens.md)、
[IR の型](../../crates/ubnfc-front/src/ir/types.rs)、
[extern の lowering](../../crates/ubnfc-front/src/lower/tokens.rs)、
[memo の計画](../../crates/ubnfc-front/src/analysis/planning.rs)。
extern の `firstChars` は必要条件の上界で、外部辞書が変わるときに不変とは限らない。
文脈依存にする場合は全 snapshot を覆う上界か unknown が必要であり、誤った上界を
渡して候補を除外してよい契約ではない。

## 5. X1 の再評価 — 既存 extern でも語彙は供給できる

[vocabulary.ubnf](../../examples/stax/boundaries/vocabulary.ubnf) は
`Root ::= ( Seeded | Word ) EOF`、`Seeded ::= SEED '+' Word` とする。
SEED は非消費で、不変 snapshot の名前を Declare 効果として返す。
`@memoSafeToken` は付けない。IR 上で extern とその祖先の成功・失敗 memo が
`unsafeExternClosure` により無効であることも検査する。

[Rust probe](../../examples/stax/boundaries/vocabulary_probe.rs) と
[Java probe](../../examples/stax/boundaries/VocabularyProbe.java) は host 側の辞書を変更しない。
Java は memo on/off、Rust は memo on/off × lexical cache on/off で次の観測に一致した。

| snapshot | 入力 | 全文受理 | 宣言 | 未定義警告数 |
|---|---|---|---|---:|
| external | `+ external` | true | external | 0 |
| external | `+ missing` | true | external | 1 |
| external | `external` | true | 空 | 1 |
| other | `+ external` | true | other | 1 |
| other | `+ missing` | true | other | 1 |
| other | `external` | true | 空 | 1 |

3/6行目は、Seeded の scanner が返した宣言が `'+'` の失敗で巻き戻され、
Word の fallback に漏れないことを検証する。scope の最終 depth は全て0。
6入力条件を backend / 設定の計36回で照合した。これは既存機能の成功であり、v3 の成功ではない。

宣言位置0は**この再現だけの仮の出所**で、外部ファイルの正しい定義位置ではない。
種別・URI・外部 span を持つ初期語彙入力の共通 API は、この例では提供していない。
この差が X1 の追加を検討する理由になるが、単純な名前供給が現契約で不可能という理由にはならない。

## 6. X1 / X2 / X3 の契約への影響

### 現行の土台

- [Rust TokenScanner](../../crates/ubnfc-rust/templates/api.rs) は読取用の ScopeStore を受け取る。
  [Java TokenScanner](../../ubnfc-java/src/main/resources/templates/api/TokenScanner.java) は
  Input と ScanState を受け取り、ScopeStore は受け取らない。API が同形だとは扱わない。
  両方とも宣言・参照等の効果を返せる。
- [Rust memo key](../../crates/ubnfc-rust/templates/memo.rs) は cursor / flag / scope 版を持つ。
  [生成式](../../crates/ubnfc-rust/src/expressions.rs) は効果の無い式の版を0へ省略する。
  [Java Session](../../ubnfc-java/src/main/resources/templates/internal/Session.java) の rule / expression memo も
  scope.stateVersion を使い、効果を再生する。新しい外部依存まで既にキー化されているわけではない。
- scope の変更は journal に入り、候補失敗時に効果・参照・診断とともに戻す。
  Rust / Java とも払い出し epoch は巻き戻さず、別の枝へ同じ新規版を再利用しない。
  根拠は [Rust scope](../../crates/ubnfc-rust/templates/scope.rs) と
  [Java ScopeStore](../../ubnfc-java/runtime-template/src/main/java/ubnfc/rt/ScopeStore.java)。
- [Rust LexCache](../../crates/ubnfc-rust/templates/lexcache.rs) は固定 literal の一致集合と trivia 用。
  extern の辞書を無条件で位置 cache する仕組みではない。parse ごとの epoch は、語彙の改訂版とは別物。
- 保証の無い extern と祖先は memo 不可。安全な token だという alias assertion は利用者の保証であり、
  可変な host 状態の自動 rollback を提供しない。parser の journal が戻すのは返された効果であり、
  scanner の内部変数や I/O ではない。診断用の再解析でも同じ答えを再現できる必要がある。
- 現在の buffer reuse は増分解析ではない。Rust memo の reset、Java MemoTable の世代更新で
  前回内容を再利用しない。生成 LSP も didChange ごとの全文再解析である。

UBNF の言語版（この例は2）、IR の schema 版、scope の状態版、buffer の世代、
将来の外部語彙 snapshot の改訂識別子は別物である。宣言的 token の固定プログラムが
IR に含まれることを、可変辞書の版が cache key に含まれることと読み替えない。

| 論点 | IR / 公開 API | rollback / memo / 字句 cache | 増分・両 backend |
|---|---|---|---|
| **X1: 不変の初期語彙** | 実際の語彙は parse 呼出しの入力。ネイティブ対応なら capability と外部宣言の出所を定義する。@catalog の出力 metadata と分離 | baseline を branch journal より外へ置き、parse 中は不変。stax の warning だけを変える範囲でも診断・出所・memo 観測は変わる。「契約影響無し」とは言わない | URI / kind / span と snapshot 同一性を両言語で揃える。将来の再利用は snapshot 依存も照合。現在の seed 代替は両方で実測済み |
| **X2: 解析中に変わる認識語彙** | 依存 context、純粋性、効果、語彙版を宣言する契約が必要。現行 IR に `volatile` という token option は無い | 外部状態も rollback / replay 対象にするか、不変 snapshot の明示遷移にする。依存する式・token・FIRST 上界・LL(1) / factor / predict 証明まで考慮。単に memo を切るだけでは外部状態と診断再解析を整合させられない | Java scanner への context の渡し方も設計が要る。編集範囲外でも辞書変更の依存範囲を無効化。未実装・この報告では未実測 |
| **X3: 語彙依存の構文形** | 生成時に固定した IR / AST 型と、動的に選ぶ意味規則を分ける。語列 AST + 言語側の意味処理なら既存方式 | native な意味述語を足すなら同じ入力位置の形・capture・宣言・診断が context に依存する。巻戻し、効果再生、最適化の除外証明も再設計 | 固定 shape の上の VM と、未知の型を持つ AST を動的生成することは別。前者を選び、後者は提案に採らない |

X1 の例のキャッシュ切替一致は、不変 snapshot の結果である。
任意に変わる host 辞書や、新しい X2 / X3 プロトコルを検証した成功件数には入れない。

## 7. 最小 v3 提案と採らない案

**最小提案は X1 の共通入力契約だけ**とする。名前の集合に加え、外部宣言の識別子・種別・
URI / 範囲を持つ不変 snapshot を parse 開始前に渡し、初期の外側 scope として扱う。
schema / API 名や `@vocabulary` の綴りは未決定で、未実装の構文例を受理済みと見せない。

ネイティブ化する場合の設計条件:

1. 古い IR / ParseOptions は従来の空 snapshot と等価。新しい capability が要求され、
   対応しない generator は明示的に拒否する。IR の変更はスキーマ検証と両 generator の対応を揃えて行う。
2. 実際の名前を grammar IR に焼き込まず、同じ生成物に複数 snapshot を渡せる。
   IR は必要な入力契約・依存を表し、名前・出所は parse 引数とする。
3. 解析中の外部変更を禁止する。初期 baseline は別の候補の rollback で消えない。
   成功・失敗・診断再解析に同じ snapshot を使い、scope のローカルな版と snapshot 同一性を区別する。
4. 外部宣言を現在文書の offset 0 に偽装せず、生成 ScopeResult / LSP が出所を区別する。
   別 URI の range はその文書の座標で表し、UTF-8 byte / code point / UTF-16 を混同しない。
5. 初期語彙を読める全経路の依存を解析し、scope 版の省略や候補除外が安全か証明する。
   現行 stax の symbolRef は Warning であり、受理範囲を変えないが、任意の scanner に一般化しない。
6. 同名の初期語彙とソース宣言の shadow、未定義診断、外部位置への definition、
   memo on/off・診断再解析・両 backend の観測一致を受け入れ条件にする。
   将来増分再利用するなら snapshot 改訂時の無効化も検証する。

**今回の判断は front 追加不要**。stax は明示 use 名、固定した定義形、quotation / macro で
現在の用途を満たす。X1 の簡単な名前供給は既存 extern 効果でもできた。
外部ファイルの型付き語彙を parser / LSP に統合する要求が出た時点で、上記条件を持つ
別の実装 issue にする。今回この実装は必要としておらず、新たな実装 issue は起票しない。
段階4で切り出した scope 精度 #49 と回復 #50、実機確認 #48 はそれぞれ別の残条件である。

採らない案と理由:

- **@catalog をそのまま入力にする**: 静的な出力候補と解析時の依存を混ぜる。
  現行 LSP が context を厳密に消費しているという前提も成り立たない。
- **mutable な scanner 辞書を持てば十分とする**: parser の効果 rollback と host 内部状態は別であり、
  再試行・memo hit・診断再解析で振る舞いが変わる。今回の不変 snapshot とは同じ契約ではない。
- **`volatile` を1語足して X2 を完了扱いする**: 現行の extern memo 除外に加え、語彙版と外部状態の
  rollback、最適化の依存、編集後の無効化を設計する必要がある。
- **native な任意意味述語 / 実行中の IR 書換えを X3 として導入する**: 生成時の型・解析計画・
  診断と side effect の保証を広く変える。一般語列の固定 AST と明示的な言語処理で分離する方が小さい。
- **完全な Forth を stax の必須範囲にする**: 入力領域・データ領域・辞書・compile 状態の実装が必要。
  コメント用途・文字列出力・固定定数の代替より大きく、本報告の意味保存範囲を越える。

## 8. 成功と未検証を分けた検証結果

| 対象 | 実測結果 |
|---|---|
| stax 11ケース | 構文受理7 / 拒否4。実行成功4、未定義・macro の拒否3、構文拒否4。stdout / exit / エラー理由を CI 照合 |
| 固定した一般語列4ケース | Java / Rust が全文受理、AST 型・値・span 一致、宣言ゼロ。意味実行ではない |
| 外部語彙6条件 | 両 backend と memo / Rust 字句 cache 切替の36回で受理・宣言・警告・depth 一致。失敗枝から宣言が漏れない |
| Forth の実行 | **未実施**。gforth 不在。標準上の期待との比較に留め、実行成功件数は0 |
| native X1 の出所付き初期語彙 API、X2、X3 | **提案・未実装**。成功件数0 |

CI には2つの通常テストを接続した。原文 stax の strict、20サンプル、両 backend AST 比較、
LSP / 回復の golden は既存テストのまま維持する。現在の実装で必要な再現は検証し、
未実装の拡張を成功へ読み替えない。
