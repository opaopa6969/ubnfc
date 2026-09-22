# Java runtime template

この独立 Maven モジュールは、生成 Java ソースへ埋め込む `ubnfc.rt` のテンプレートを
JDK 21 でコンパイル・テストする。生成物にこの jar の依存を追加しない。
本体の依存は JDK のみ。JUnit Jupiter 5.10.2 は test scope のみで使用する。

```sh
cd ubnfc-java/runtime-template
mvn -q -o test
```

## 契約と API の選択

設計の根拠は `docs/design/03-design-v1.md` §3.1、§3.3、§3.6〜3.9、§6。
既存リポジトリの `transactional-scopes.md`、`generated-scope-effects.md`、
`packrat-memoization.md` と `ScopeStore.java` は挙動の確認にだけ使用し、コードは転用していない。
`ParseContext`、旧 runtime、生成器の内部型には依存しない。

- `Input`: `String.toCharArray()` は構築時の一度だけ。サロゲートペアがなければ位置変換は
  配列不要の恒等 O(1)。ペアがあれば code point 境界表を一度作り、cp→UTF-16 は O(1)、
  UTF-16→cp は二分探索。末尾を含む境界だけを span として認め、ペア内部は明示的に拒否する。
  孤立サロゲートは JDK と同じ一文字扱い。CI 比較の完全互換を優先し、元の immutable な
  String も保持して `regionMatches(true, ...)` に委譲する。リテラルに単語境界は加えない。
- `Cursor`: 両位置は UTF-16。`advance` は両方に同じ差分を加え、`advanceMatchedOnly` は
  matched のみを進める。long checkpoint の上位 32 bit が consumed、下位が matched。
  先読み終了時のカーソル選択や反復の進捗判定は生成側が行う。
- `ScopeStore`: `Decl(name, offsetCp)`、`Reference(name, offset, len)`、
  `Diagnostic(msg, offset, len, severity)` を持つ。位置は呼出し側が変換した code point。
  `resolve` の未発見は null。`declaredInCurrentScope` は lookup の現在値だけの独立リスト。
  同名再宣言も全宣言履歴には残す。宣言・参照・診断の公開履歴は変更不可 live view。
  scope の出入り、再宣言時の旧 lookup 値、参照、診断、clear の逆操作を型付き journal に保存し、
  checkpoint は journal 長。子成功でも journal を保持し、親の restore で復元できる。
  checkpoint は同一 store の現在の履歴の祖先に限る。破棄した枝の checkpoint の再利用は不可。
- stateVersion は enter・有効な leave・非空名 declare・**非空の診断 clear** で新しい epoch を使う。
  空の clear、depth 0 の leave、空/null 名の宣言・参照は原典同様 no-op。
  addReference/addDiagnostic は version を進めない。restore は現在 version を戻し、epoch は戻さない
  （#269）。int epoch の枯渇は例外とし、別状態への version 再利用を防ぐ。
- `MemoTable`: 5 個の無制限 int は 128 bit に入らないため、1 個目の long を
  `ExprId:24 / mode:8 / stateVersion:32`、2 個目を `consumed:32 / matched:32` とする。
  ExprId は 0〜16,777,215、mode は 0〜255、位置は非負。範囲外は get/put とも例外。
  stateVersion は全 32 bit を保存する。キーは long 配列、値は int 配列のみ。
  payload は構築レシピ・scope 効果・局所診断等を持つ生成側 arena への int handle。
  線形探索、最大負荷率 0.5、拡張時に再配置する。`get` は未発見時 -1、発見時 slot。
  `ok/consumedEnd/matchedEnd/payload(slot)` で読む。slot の寿命は次の put/clear まで。
- `Diagnostics`: farthest 初期値は -1。expected ID は重複なしの挿入順。
  同位置では最も深い rule、同じ深さなら先着を保持し、より遠い失敗では置換する。
  `enterRule` ごとに局所 accumulator を作り、失敗と memo merge は global と全祖先へ伝播する。
  `snapshot` は rule 内では最内 rule の局所結果、rule 外では global。
  `leaveRule` は局所 Snapshot を返す。成功 rule の内部失敗も消さない。
  Snapshot の stack は保存した rule 自身からの suffix。memo hit では対象 rule を enter する前に
  `merge` し、現在の呼出元へ付け替える。suppression は入れ子可能で fail/merge の双方に効く。
  `checkpoint/restore` は global と実行中の全局所診断を復元し、Not の成否確定後の破棄に使う。
  `enterExpression` は rule stack に新しい rule を足さず匿名式の memo 用 accumulator を作る。
  `failAt(pos, expectedId)` の位置単位・consumed/matched の選択は生成側で統一する。
  D-011 / D-014 の主位置と最遠位置の分離は生成側が行い、このモジュールは最遠 accumulator を提供する。

## テストの範囲

`ScopeStoreTest` は transactional-scopes と #269 の復元・履歴・version 規則を検証する。
`GeneratedScopeEffectsTest` は generated-scope-effects の rule 境界、親への登録、capture の順序、
trim と code point 位置、宣言→参照、WARNING、choice/先読みを、明示的なイベント列で検証する。
capture の検出・binding、mode/description metadata、注釈検証、実際の生成 parser はこのモジュールの
対象外であり、emitter の検証済みとは扱わない。trim と未定義警告も生成側の責務で、store は自動で行わない。

`InputTest` は 😀 混在・孤立サロゲートを含む全境界の往復と JDK の文字列比較との一致を検証する。
`CursorTest` は二重位置・checkpoint・overflow、`MemoTableTest` は 10 万件、衝突、拡張、キーの境界を検証する。
`DiagnosticsTest` は最遠位置、抑止、局所 Snapshot、先行 sibling の混入防止、呼出元を変更した再生を検証する。
