# はじめての ubnfc

小さな文法を 1 本書いて、そこから **Rust パーサと Java パーサを生成し、両方を走らせて
同じ AST と同じ診断が出るのを見る**ところまでをやる。所要 10 分。

前提:

- Rust 1.85 以上（本稿は 1.98.1）
- JDK 21 以上と Maven（`javac` / `java` / `mvn` が PATH にあること）
- ネットワークは不要（全コマンドが `--offline` / `-o`）

以下、`$UBNFC` を clone した ubnfc リポジトリのルートとする。
作業ディレクトリは `mktemp -d` で作る。

```sh
UBNFC=$(pwd)          # ubnfc リポジトリのルートで実行する
WORK=$(mktemp -d)
```

---

## 0. ジェネレータを建てる

Rust 側の CLI 2 本と、Java backend の jar を作る。

```sh
cd "$UBNFC"
cargo build --offline --bin ubnfc --bin ubnfc-rust
(cd ubnfc-java && mvn -o -q -DskipTests package)
```

```
   Compiling ubnfc-front v0.1.0 (.../crates/ubnfc-front)
   Compiling ubnfc-rust v0.1.0 (.../crates/ubnfc-rust)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 30.23s
```

`ubnfc-java/target/ubnfc-java-0.1.0-SNAPSHOT.jar` ができる。`ubnfc gen --target java` は
これを自動で探す（`UBNFC_JAVA_JAR` で明示指定も可能）。

> **`ubnfc-java/` を編集したら必ずこの `mvn package` をやり直すこと。**
> driver テンプレートと emitter は jar の resource なので、古い jar のまま走らせると
> 「Java backend だけが契約を破っている」ように見える偽陽性が出る（決定 D-033）。

以後は `ubnfc` を短く呼べるようにしておく。

```sh
export PATH="$UBNFC/target/debug:$PATH"
ubnfc --version
```

```
ubnfc 0.1.0
```

---

## 1. 文法を書く

`let` で変数を束縛して足し算するだけの言語を作る。
`@scopeTree` / `@declares` / `@backref` を入れて、**スコープと記号表まで文法に書ける**
ことを見る。

```sh
cat > "$WORK/tinylet.ubnf" <<'EOF'
grammar TinyLet {

  @whitespace: javaStyle

  token NUMBER     = org.unlaxer.parser.elementary.NumberParser
  token IDENTIFIER = org.unlaxer.parser.clang.IdentifierParser

  @root
  @scopeTree(mode=lexical)
  @mapping(Program, params=[bindings, result])
  Program ::= { Binding } @bindings Expression @result ;

  @declares(symbol=name)
  @mapping(Binding, params=[name, value])
  Binding ::= 'let' IDENTIFIER @name '=' Expression @value ';' ;

  @mapping(BinaryExpr, params=[left, op, right])
  @leftAssoc
  @precedence(level=10)
  Expression ::= Term @left { AddOp @op Term @right } ;

  AddOp ::= '+' | '-' ;

  Term ::= NUMBER | VarRef | '(' Expression ')' ;

  @backref(name=name)
  @mapping(VarRef, params=[name])
  VarRef ::= IDENTIFIER @name ;
}
EOF
```

読み方:

| 書いたもの | 意味 |
|---|---|
| `@whitespace: javaStyle` | 空白と `//` / `/* */` をどこで読み飛ばすか。挿入位置は front が計画を立てて IR に載せる |
| `token NUMBER = ...` | 組込み token。v1 は旧実装のクラス名で綴る（§7 に v2 の書き方） |
| `@root` | 入口の規則 |
| `@mapping(T, params=[...])` | 採用された結果から作る AST のノード型と field |
| `X @name` | capture。`params` の名前と対応する |
| `{ X }` | 0 回以上の反復（`X*` と同じ） |
| `@scopeTree(mode=lexical)` | この規則の開始でスコープに入り、成功で出る |
| `@declares(symbol=name)` | capture `@name` の文字列を、（leave 後に）親スコープへ宣言として登録する |
| `@backref(name=name)` | capture `@name` をスコープで引く。見つからなければ **WARNING を出して構文は成功させる** |
| `@leftAssoc` + `@precedence` | 左結合の演算子段。`left` / `op` / `right` の 3 つを取る正規形 |

### 1.1 まず検証だけする

```sh
ubnfc validate --grammar "$WORK/tinylet.ubnf"
echo "exit=$?"
```

```
exit=0
```

何も出なければ問題なし。わざと壊すと、たとえば `@precedence` を落とし、`@mapping` の
`params` と rule body の両方から `op` を落とした版はこう言われる（実際に試した出力）:

```
rule Expression uses @leftAssoc but @mapping(BinaryExpr) params does not contain 'op' [code: E-ASSOC-MAPPING-PARAM] [hint: Include left/op/right in @mapping params.]
rule Expression uses @leftAssoc but capture @op is missing [code: E-ASSOC-MISSING-CAPTURE] [hint: Add @op capture in the rule body.]
rule Expression uses @leftAssoc but has no @precedence [code: E-ASSOC-NO-PRECEDENCE] [hint: Add @precedence(level=...) to this operator rule.]
exit=1
```

### 1.2 Grammar IR を覗く

backend が読むのはこの JSON だけ。

```sh
ubnfc ir --grammar "$WORK/tinylet.ubnf" --out "$WORK/tinylet.ir.json"
python3 -c "
import json; d = json.load(open('$WORK/tinylet.ir.json'))
print(sorted(d.keys()))
print('languageVersion', d['languageVersion'])
print('entry', d['entry'])
print('rules', len(d['semantics']['rules']), 'expressions', len(d['semantics']['expressions']), 'captures', len(d['semantics']['captures']))
print('trivia insertion points', len(d['semantics']['trivia']['insertionPlan']))
print('requiredFeatures', d['requiredFeatures'])
"
```

```
['analysis', 'analysisVersion', 'entry', 'grammarUnits', 'importEdges', 'languageVersion', 'plan', 'requiredFeatures', 'semantics', 'serialization', 'sources', 'version']
languageVersion 1
entry {'grammarId': 'grammar:TinyLet', 'ruleId': 'TinyLet::Program'}
rules 6 expressions 28 captures 8
trivia insertion points 21
requiredFeatures ['backendDiagnostics', 'captures', 'cursorFrameFlag', 'deferredMapping', 'dualCursor', 'memoReplay', 'operators', 'orderedChoice', 'predictiveChoice', 'scopeUndo', 'structuredOccurrences', 'triviaInsertion']
```

- `semantics` が意味論（capture・trivia・scope 効果・構築レシピ）、`analysis` が解析結果
  （FIRST、ゼロ消費条件、memo 安全性）、`plan` が最適化計画。backend は `semantics` を
  再解釈してはいけない（決定 D-009）。
- `trivia.insertionPlan` に「どの式の境界でどの policy の trivia を読み飛ばすか」が 21 か所ぶん
  入っている。生成コードはこの計画をそのまま実行する。
- `requiredFeatures` を読めない backend は明示的に拒否する。

---

## 2. Rust パーサを生成して動かす

```sh
ubnfc gen --grammar "$WORK/tinylet.ubnf" --target rust --out "$WORK/rust-out"
find "$WORK/rust-out" -type f | sort
```

```
/tmp/.../rust-out/Cargo.toml
/tmp/.../rust-out/api.rs
/tmp/.../rust-out/ast.rs
/tmp/.../rust-out/evaluator.rs
/tmp/.../rust-out/grammar.ir.json
/tmp/.../rust-out/metadata.rs
/tmp/.../rust-out/mod.rs
/tmp/.../rust-out/parser.rs
/tmp/.../rust-out/rt/cursor.rs
/tmp/.../rust-out/rt/diag.rs
/tmp/.../rust-out/rt/input.rs
/tmp/.../rust-out/rt/lexcache.rs
/tmp/.../rust-out/rt/memo.rs
/tmp/.../rust-out/rt/mod.rs
/tmp/.../rust-out/rt/scope.rs
/tmp/.../rust-out/runtime.rs
/tmp/.../rust-out/src/bin/driver.rs
/tmp/.../rust-out/src/bin/driver_json.rs
```

これで完結した crate になっている（`rt/` は runtime のコピーで、外部 crate へのリンクは無い。
通常依存ゼロ、`json` feature のときだけ serde / serde_json）。
`src/bin/driver.rs` はハーネス用の JSON Lines driver。

生成された AST 型はこうなっている（`ast.rs`）:

```rust
pub enum Ast { Text(String), Null,
  g_TinyLetAST_2e_Program(...), g_TinyLetAST_2e_Binding(...),
  g_TinyLetAST_2e_BinaryExpr(...), g_TinyLetAST_2e_VarRef(...) }

pub struct g_TinyLetAST_2e_Binding { pub span: Span, pub node_id: usize,
  pub g_name: String, pub g_value: Box<Ast> }
```

識別子が escape されているのは、生成名がホスト言語の型や package を隠さないようにするため
（canonical JSON の側は元の名前を保つ）。

### 2.1 呼び出し側を書く

```sh
mkdir -p "$WORK/rust-out/src/bin"
cat > "$WORK/rust-out/src/bin/demo.rs" <<'EOF'
use ubnfc_generated::{parse_with_options, ParseOptions};

fn main() {
    let source = std::env::args().nth(1).unwrap_or_default();
    let options = ParseOptions { diagnostics: true, ..Default::default() };
    let result = parse_with_options(&source, options);
    println!("ok={} consumed={} matched={}", result.ok, result.consumed_cp, result.matched_cp);
    if let Some(ast) = &result.ast {
        println!("{}", serde_json::to_string_pretty(&ast.canonical_value()).unwrap());
    }
    for d in &result.diagnostics {
        println!("diagnostic {:?} at {} expected {:?}", d.kind, d.offset_cp, d.expected);
    }
    for d in &result.scope_diagnostics {
        println!("scope {:?} at {}: {}", d.severity, d.offset, d.msg);
    }
}
EOF
printf '\n[[bin]]\nname = "demo"\npath = "src/bin/demo.rs"\n' >> "$WORK/rust-out/Cargo.toml"
(cd "$WORK/rust-out" && cargo build --offline --features json)
```

`ParseOptions::default()` は `require_eof: true` / `build_ast: true` / `diagnostics: false`。
診断は既定で**記録しない**（fast mode）。結果が診断を要するときだけ、入口が同じ入力を
`diagnostics: true` で決定的に解析し直す（決定 D-023）。ここでは成功時の投機的な
`hints` も見たいので明示的に `true` を渡している。

`canonical_value()` は `json` feature で生える。`{type, span, fields}` の正規形で、
Java 側と byte 単位で比較できる形。

### 2.2 走らせる

```sh
cd "$WORK/rust-out"
./target/debug/demo 'let x = 1 + 2; x + 3'
```

```
ok=true consumed=20 matched=20
{
  "fields": {
    "bindings": [
      {
        "fields": {
          "name": "x",
          "value": {
            "fields": { "left": "1", "op": [ "+" ], "right": [ "2" ] },
            "span": [ 8, 13 ],
            "type": "BinaryExpr"
          }
        },
        "span": [ 0, 15 ],
        "type": "Binding"
      }
    ],
    "result": {
      "fields": {
        "left": { "fields": { "name": "x" }, "span": [ 15, 17 ], "type": "VarRef" },
        "op": [ "+" ],
        "right": [ "3" ]
      },
      "span": [ 15, 20 ],
      "type": "BinaryExpr"
    }
  },
  "span": [ 0, 20 ],
  "type": "Program"
}
```

（実際の出力は 1 要素 1 行の pretty JSON。ここでは紙面のため配列とオブジェクトを畳んである。
値・span・順序は実出力そのまま。）

見どころ:

- `span` はすべて **Unicode code point の半開区間**。内部は byte（Rust）/ UTF-16（Java）
  で持っていて、入力ごとに変換表を一度だけ作っている（決定 D-007）。
- `VarRef` の span が `[15, 17]` で、`x` 自体は `[15, 16]` なのに 1 広い。
  **capture と span の境界を token の終端へ縮めない**契約で、後続 trivia を含む。
- `consumed` と `matched` が別々に出ている。これは二重カーソルで、
  `matched` は「照合が到達した位置」（`LOOKAHEAD` などゼロ消費の構文が進める）であって
  最遠到達位置ではない。詳細は `docs/design/04-cursor-and-diagnostics-contract.md`。

### 2.3 スコープの診断を見る

`y` を宣言せずに使う。

```sh
./target/debug/demo 'let x = 1; y + 1'
```

```
ok=true consumed=16 matched=16
{
  "fields": {
    ...
    "result": {
      "fields": {
        "left": { "fields": { "name": "y" }, "span": [ 11, 13 ], "type": "VarRef" },
        "op": [ "+" ],
        "right": [ "1" ]
      },
      "span": [ 11, 16 ],
      "type": "BinaryExpr"
    }
  },
  "span": [ 0, 16 ],
  "type": "Program"
}
scope Warning at 11: 未定義のシンボル: 'y'
```

**構文としては成功したまま、未定義参照が WARNING として別に出る。**
これが `@scopeTree` + `@declares` + `@backref` の効果で、パーサを書いた側は
シンボル表のコードを 1 行も書いていない。`offset` は code point。

### 2.4 構文エラーを見る

```sh
./target/debug/demo '(1 + 2'
```

```
ok=false consumed=0 matched=0
diagnostic Syntax at 6 expected ["+", "-", ")"]
```

`expected` は文法に書かれた終端の表示文字列。位置 6（入力の末尾）で
「`+` か `-` で式を続けるか `)` で閉じるか」を要求している。

---

## 3. Java パーサを生成して動かす

**同じ IR から**、今度は Java を出す。

```sh
ubnfc gen --grammar "$WORK/tinylet.ubnf" --target java --out "$WORK/java-out"
find "$WORK/java-out" -type f | sort
```

```
/tmp/.../java-out/grammar.ir.json
/tmp/.../java-out/src/main/java/target/TinyLetAST.java
/tmp/.../java-out/src/main/java/target/TinyLetEvaluator.java
/tmp/.../java-out/src/main/java/target/TinyLetParser.java
/tmp/.../java-out/src/main/java/target/TinyLetResult.java
/tmp/.../java-out/src/main/java/target/api/Capture.java
/tmp/.../java-out/src/main/java/target/api/Diagnostic.java
/tmp/.../java-out/src/main/java/target/api/Json.java
/tmp/.../java-out/src/main/java/target/api/MappingException.java
/tmp/.../java-out/src/main/java/target/api/ParseOptions.java
/tmp/.../java-out/src/main/java/target/api/ParseResult.java
/tmp/.../java-out/src/main/java/target/api/SourceName.java
/tmp/.../java-out/src/main/java/target/api/Span.java
/tmp/.../java-out/src/main/java/target/api/TokenScanner.java
/tmp/.../java-out/src/main/java/target/driver/Driver.java
/tmp/.../java-out/src/main/java/target/internal/Recipe.java
/tmp/.../java-out/src/main/java/target/internal/Session.java
/tmp/.../java-out/src/main/java/target/rt/Cursor.java
/tmp/.../java-out/src/main/java/target/rt/Diagnostics.java
/tmp/.../java-out/src/main/java/target/rt/Input.java
/tmp/.../java-out/src/main/java/target/rt/MemoTable.java
/tmp/.../java-out/src/main/java/target/rt/ScopeStore.java
/tmp/.../java-out/src/main/java/target/rt/Severity.java
```

AST は sealed interface + record（`TinyLetAST.java`）:

```java
public sealed interface TinyLetAST permits TinyLetAST.Program, TinyLetAST.Binding,
                                           TinyLetAST.BinaryExpr, TinyLetAST.VarRef {
    record Program(java.util.List<TinyLetAST.Binding> bindings, TinyLetAST.BinaryExpr result)
        implements TinyLetAST { ... }
    record Binding(java.lang.String name, TinyLetAST.BinaryExpr value) implements TinyLetAST {}
    record BinaryExpr(java.lang.Object left, java.util.List<java.lang.String> op,
                      java.util.List<java.lang.Object> right) implements TinyLetAST {}
    record VarRef(java.lang.String name) implements TinyLetAST {}
}
```

### 3.1 呼び出し側

```sh
cat > "$WORK/java-out/src/main/java/target/Demo.java" <<'EOF'
package target;

import target.api.ParseOptions;
import target.api.ParseResult;

public final class Demo {
    public static void main(String[] args) {
        ParseResult<TinyLetAST> r = TinyLetParser.parse(args[0], ParseOptions.DEFAULT);
        System.out.println("ok=" + r.ok() + " consumed=" + r.consumedCp() + " matched=" + r.matchedCp());
        r.ast().ifPresent(System.out::println);
        var d = r.diagnostics();
        if (d != null && !d.kind().equals("none")) {
            System.out.println("diagnostic " + d.kind() + " at " + d.offset() + " expected " + d.expected());
        }
        r.scope().diagnostics().forEach(s ->
            System.out.println("scope " + s.severity() + " at " + s.offset() + ": " + s.msg()));
    }
}
EOF
cd "$WORK/java-out"
mkdir -p classes
javac -d classes $(find src/main/java -name '*.java' -not -path '*/driver/*')
```

`driver/` を外しているのは、ハーネス用 driver がここでは要らないだけ。一緒に compile しても通る。
Maven も外部依存も不要で、`javac` だけで建つ。

### 3.2 走らせる（Rust と同じ 3 本）

```sh
java -cp classes target.Demo 'let x = 1 + 2; x + 3'
```

```
ok=true consumed=20 matched=20
Program[bindings=[Binding[name=x, value=BinaryExpr[left=1, op=[+], right=[2]]]], result=BinaryExpr[left=VarRef[name=x], op=[+], right=[3]]]
```

```sh
java -cp classes target.Demo 'let x = 1; y + 1'
```

```
ok=true consumed=16 matched=16
Program[bindings=[Binding[name=x, value=BinaryExpr[left=1, op=[], right=[]]]], result=BinaryExpr[left=VarRef[name=y], op=[+], right=[1]]]
scope WARNING at 11: 未定義のシンボル: 'y'
```

```sh
java -cp classes target.Demo '(1 + 2'
```

```
ok=false consumed=0 matched=0
diagnostic syntax at 6 expected [' ', ')', '*/', '+', '-', '.', '/*', '//', DigitParser, EParser, ExponentParser, ExpressionRepeat0Parser]
```

### 3.3 2 つの出力を突き合わせる

| 観測 | Rust | Java | 契約 |
|---|---|---|---|
| `ok` / `consumed` / `matched` | `true` / 20 / 20 | 同じ | 共有契約。両 backend で一致すること |
| AST の型・field・値 | `Program{bindings:[Binding{name:"x", …}], result:BinaryExpr{…}}` | 同じ | 共有契約 |
| scope の WARNING 位置 | 11 | 11 | 共有契約（code point） |
| 構文診断の `kind` と主 `offset` | `Syntax` / 6 | `syntax` / 6 | **主 offset は言語別 oracle**（決定 D-011）。ここでは一致しているが、一般には Java が「失敗 frame 内の `max(consumed, matched)` の最遠」、Rust が「`fail_at` の最遠」で違い得る |
| `expected` の中身 | `["+", "-", ")"]` | `[' ', ')', '*/', '+', '-', '.', '/*', '//', DigitParser, …]` | **言語別 hints**（決定 D-014）。corpus が検証するのは「非空であること」と、末尾入力時の一致だけ |

つまり **受理・カーソル・AST・スコープは厳密に一致させ、診断の見せ方は言語ごとに違ってよい**、
という線の引き方になっている。差分ファジング（`crates/ubnfc-fuzz`）はこの線の内側だけを
厳密比較する。

---

## 4. front に間違いを見つけてもらう

PEG の順序付き選択は、候補の順番を間違えると後ろの候補が**黙って死ぬ**。
`AddOp` に `'+='` を足してみる。

```sh
sed "s/AddOp ::= '+' | '-' ;/AddOp ::= '+' | '+=' | '-' ;/" "$WORK/tinylet.ubnf" > "$WORK/dead.ubnf"
ubnfc validate --grammar "$WORK/dead.ubnf"; echo "exit=$?"
```

```
alternative 1 of rule AddOp ('+') is a prefix of every string alternative 2 can match (it must start with '+='), so alternative 2 is never reached [code: W-CHOICE-PREFIX] [hint: PEG ordered choice commits to the first success; put the longer literal first.]
exit=0
```

警告なので既定では exit 0（受理・AST・診断・IR は 1 byte も変わらない）。
CI で落としたければ `--strict`:

```sh
ubnfc validate --grammar "$WORK/dead.ubnf" --strict; echo "exit=$?"
```

```
alternative 1 of rule AddOp ('+') is a prefix of every string alternative 2 can match (it must start with '+='), so alternative 2 is never reached [code: W-CHOICE-PREFIX] [hint: PEG ordered choice commits to the first success; put the longer literal first.]
exit=5
```

この検査は**証明できたときだけ**警告する（決定 D-037、4 コード）。
`W-CHOICE-NULLABLE-NOT-LAST` / `W-CHOICE-PREFIX` / `W-CHOICE-UNREACHABLE` /
`W-CHOICE-SHADOWED-PATH`。証明の中身は `docs/reports/2026-09-23-choice-reachability.md`。

---

## 5. リテラルは接頭辞一致である（v1 の重要な落とし穴）

UBNF v1 のリテラルは **code point 列の接頭辞一致で、単語境界を見ない**（決定 D-002）。
旧実装がそうだったので、互換のためにそうしてある。つまり `'let'` は `letx` の頭にも一致する。

```sh
cd "$WORK/rust-out" && ./target/debug/demo 'letx = 1; letx'
```

```
ok=true consumed=14 matched=14
{
  "fields": {
    "bindings": [
      {
        "fields": { "name": "x", "value": { ... "span": [ 7, 8 ], "type": "BinaryExpr" } },
        "span": [ 0, 10 ],
        "type": "Binding"
      }
    ],
    ...
}
scope Warning at 10: 未定義のシンボル: 'letx'
```

`letx = 1;` が **`let` + 識別子 `x` の束縛として通ってしまう**
（`name` が `letx` ではなく `x` になっている）。

これは UBNF v2 で直っている（§6）。

---

## 6. UBNF v2 を試す

`@ubnf: 2` を書くと、v2 の挙動が有効になる。v1 は凍結されているので、既存の文法は
1 文字も変えずに通り続ける。

```sh
sed -e 's/^  @whitespace: javaStyle/  @ubnf: 2\n  @whitespace: javaStyle/' \
    -e 's#token NUMBER     = org.unlaxer.parser.elementary.NumberParser#token NUMBER     = NUMBER#' \
    -e 's#token IDENTIFIER = org.unlaxer.parser.clang.IdentifierParser#token IDENTIFIER = IDENTIFIER#' \
    "$WORK/tinylet.ubnf" > "$WORK/tinylet-v2.ubnf"
head -8 "$WORK/tinylet-v2.ubnf"
ubnfc validate --grammar "$WORK/tinylet-v2.ubnf"; echo "exit=$?"
ubnfc gen --grammar "$WORK/tinylet-v2.ubnf" --target rust --out "$WORK/rust-v2"
```

```
grammar TinyLet {

  @ubnf: 2
  @whitespace: javaStyle

  token NUMBER     = NUMBER
  token IDENTIFIER = IDENTIFIER

rule VarRef uses @backref, whose meaning depends on whether any rule in the grammar has @scopeTree [code: W-ANNOTATION-BACKREF-AMBIGUOUS] [hint: Use @symbolRef(name=...) for a scope lookup, or @sameText(name=...) for a same-rule text match.]
exit=0
```

`@ubnf: 2` にすると、この文法が使っている旧綴りの `@backref` に新しい警告が付く
（決定 D-046。`docs/design/07-annotation-inventory.md` §4「P2」）。`@backref` の意味は
「文法中のどこかに `@scopeTree` がある規則が 1 つでもあるか」という非局所判定で決まるため、
v2 では明示的な `@symbolRef(name=...)` / `@sameText(name=...)` を勧める、というだけの
WARNING で、受理・AST・診断は 1 byte も変えない（exit は 0 のまま）。以下は 2 つ変わっている。

1. **token 語彙が言語中立になる**（決定 D-041）。`org.unlaxer.parser.clang.IdentifierParser`
   ではなく `IDENTIFIER` と書ける。裸の SCREAMING_SNAKE_CASE で 36 名前。
   受理範囲は 1 つも変えていない（中立名を旧 FQN へ写して既存の分類器に渡すだけ）。
   ホスト実装が要るものは `token STRING = extern 'StringLiteralParser'` と明示する。
2. **リテラルが単語境界を持つ**（決定 D-040）。末尾が識別子構成文字（ASCII `[A-Za-z0-9_]`）の
   リテラルは、一致の直後も識別子構成文字なら一致しない。

同じ demo を v2 の生成物で建てて §5 の入力を入れると:

```sh
cp "$WORK/rust-out/src/bin/demo.rs" "$WORK/rust-v2/src/bin/demo.rs"
printf '\n[[bin]]\nname = "demo"\npath = "src/bin/demo.rs"\n' >> "$WORK/rust-v2/Cargo.toml"
(cd "$WORK/rust-v2" && cargo build --offline --features json && ./target/debug/demo 'letx = 1; letx')
```

```
ok=false consumed=5 matched=5
diagnostic TrailingInput at 5 expected ["end of input"]
scope Warning at 0: 未定義のシンボル: 'letx'
```

`'let'` が `letx` に食い込まなくなったので、`letx = 1; letx` は
「式 `letx` のあとにゴミが続いている」として正しく拒否される。

v2 では他に**左再帰ルール**が書ける（候補の並び順が優先順位。front が v1 の優先順位塔へ
脱糖する。決定 D-039）。計画と実装状況は [`docs/design/06-ubnf-v2.md`](design/06-ubnf-v2.md)。

---

## 7. 次に読むもの

| 知りたいこと | 文書 |
|---|---|
| 全体像・現状・できていないこと | [`../README.md`](../README.md) |
| 設計の正本と決定表 | [`design/03-design-v1.md`](design/03-design-v1.md) |
| カーソル・診断・memo キーの厳密な契約 | [`design/04-cursor-and-diagnostics-contract.md`](design/04-cursor-and-diagnostics-contract.md) |
| 組込み token の受理範囲 | [`design/02-facts-tokens-bench-corpus.md`](design/02-facts-tokens-bench-corpus.md) §A |
| どの機能にどんな oracle があるか | [`compat-matrix.md`](compat-matrix.md) |
| 生成物とハーネスの通信形式 | [`harness-driver-protocol.md`](harness-driver-protocol.md) |
| 実在の大きい文法での実例 | `examples/p4-rust` / `examples/p4-java`（tinyexpression P4、124 規則） |

## 8. 片付け

```sh
rm -rf "$WORK"
```
