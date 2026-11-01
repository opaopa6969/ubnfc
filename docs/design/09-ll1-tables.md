# LL(1) で選択を確定できる領域の表駆動認識

issue [#2](https://github.com/opaopa6969/ubnfc/issues/2)、D-091。
D-023 の診断用再解析、D-058 / D-064 の memo 判定、D-072 の一括走査を前提とする。
実測と適用率は [計測報告](../reports/2026-11-01-ll1-tables.md) に記録する。

## 判定する領域

`analysis/ll1.rs` が、通過する全節点を同じ認識機械で実行できる閉領域を求める。
まず局所条件を判定し、未対応の依存先を持つ式を落とす最大不動点を計算する。
右再帰は許す。左再帰の検査は既存の解析で済ませ、左再帰を正規化した規則と
operator 計画を持つ規則は除く。候補順・式 ID・意味論の本体は変更しない。

| 構造 | 条件と実行 |
|---|---|
| Seq / Group | 全要素が領域内。列の続行位置は明示 stack に置く |
| Choice | longest 以外。全候補が非 nullable、EOF を開始条件に持たず、FIRST の必要条件の上界が互いに素 |
| Optional | 子が領域内。子が途中で失敗しても入口位置へ巻き戻して成功する |
| Repeat | 終端子なし、本体が非 nullable かつ最小消費量 > 0、max ≠ 0。min/max と各試行の入口位置を保持する |
| RuleRef | 参照先も領域内。再帰は Call と明示 stack で処理する |
| Literal | case-sensitive。語境界は従来と同じ `identifier_byte_at` で判定する |
| 単一 code point token / Any | CharRange / Negation / 列挙した builtin。既存 Facts の正確な集合を使う |
| EOF | byte 位置が入力長と一致することを検査する |

trivia 挿入境界、scope 効果、recovery の所有規則・適用位置、Extern、regex、UNTIL、
code fence、lookahead、反転、MatchOnly / CalleeDefault の子 mode、Empty は含めない。
Empty は「消費しないが matched を進める」ため、通常の空文字列と同一視しない。

mapping と capture の構造は認識表に入れられる。ただし今回の Rust 実行は、
**文法全体に scope / recovery がなく、AST・arena AST・出現表・lexical を要求しない場合**に限る。
これらの条件では元の `event()` も事象を記録しない。scope のある P4 は静的解析の結果を載せるが、
実行しない。効果を持つ親が子孫の事象を読む可能性まで局所的に証明する変更は今回に含めない。

判定する LL(1) 性は Choice の候補決定に関するものである。Optional / Repeat の
FIRST/FOLLOW 排他を新しい意味論として要求せず、PEG の局所巻戻しを維持する。
例えば JSON の `{ Ws ',' Ws Member }` は末尾空白を読んでから comma 不在で戻り得る。
この巻戻しまで消すと末尾空白の受理が変わるため、表の明示 frame で再現する。

## IR の表

任意キー `plan.ll1` を追加する。空なら省略する。既存キー、semantics、analysis、
requiredFeatures はそのまま保持する。各行は `exprId`、`root`、`operation` を持ち、
参照は同じ配列の 0 起点 index。配列順は既存の式配列順で決定的に生成する。

Choice は **状態 × code point の区間クラス → 次状態**の疎な表を持つ。
各行の区間集合はソート済みで、候補間で交わらない。例えば JSON の Value は
`{` / `[` / `"` / `-` と数字 / `t` / `f` / `n` でそれぞれ 1 候補へ遷移する。
Literal、Class、EOF は葉の命令、Sequence は続行列、Call は規則本体の状態を指す。
Optional と Repeat は局所失敗の受け皿を明示 stack に置く。
再帰を含むため機械全体は有限状態 DFA ではなく、表を持つ pushdown 型の認識機械となる。

`root` は規則本体、または領域外の親に接する式を示す。Rust はその閉包に memo 対象が
1 つでもあれば使わない。D-058 の計画と memo 容量制限の発火位置を変えないためである。
単発の葉を VM へ渡す費用を避けるため、3 状態以上で Choice / Repeat を含む根だけを実行する。
backend は表を使う前に semantics から証明を再生成し、式 ID で命令と参照先を照合する。
保守的な部分表や状態配列の並べ替えも許し、既存の左再帰診断を優先する。破損した参照、クラス、
不適切な証明があれば表を生成・実行せず参照経路を使う。意味論を加工して古い計画が残った
既存の IR も受け付けるためであり、意味論そのものの検証と診断は従来どおりである。
旧 IR はこの再検証を必要としない。

## Rust の実行と切替

`ParseOptions::ll1` は既定 true。false は既存の式評価へ戻す。
既存 `scan` と独立に切り替えられ、scan A/B の driver は LL(1) を外して比較する。
表の実行は `Session::<false>` の認識専用経路だけに置く。さらに invert がなく、
入口の consumed と matched が等しく、既存の資源失敗がないことを要求する。

表の実行は Session を変更せず、ローカルな byte 位置と明示 stack だけを使う。
成功した場合だけ Step を作り、consumed / matched をまとめて更新する。
Seq の entry flag と、Group / RuleRef が委譲する出口 flag も再現する。
同じ byte 位置に到達するだけでなく、後続の dual cursor の動作まで維持する。

D-072 ですでに証明した一括走査の根に到達した場合は、既存の純粋な走査関数を表の辺として呼ぶ。
`options.scan = false` なら同じ表の通常命令で進む。scan の生成器自体は変更しない。
これにより文字列・数値・空白を式の VM 命令へ戻さず、外側の再帰・列・選択の機構だけを減らす。

入力不一致、論理 `max_depth` への接近、仮想 Call 深さ 32 または明示 frame 数 256 への到達では、
入口から通常の式評価を実行する。Optional / Repeat の子の通常の失敗は VM 内で巻き戻す。
上限到達そのものを成功や資源エラーには変換しない。資源診断は既存経路が確定する。
通常経路の解析結果が失敗・部分消費・診断を要する場合は、D-023 の診断 session で再解析する。

## 観測同値の根拠

1. 葉は既存と同じ文字集合・UTF-8 幅・語境界・EOF 判定を使う。Empty など dual cursor を
   別々に動かす葉は除くため、領域内の成功状態は consumed = matched を維持する。
2. Choice の FIRST は必要条件の上界である。現在文字を含む候補が 1 つなら他候補は
   成功できず、0 個なら全候補が失敗する。nullable / EOF 候補を除くので隠れた空成功もない。
3. 列は順序を保つ。Optional / Repeat は失敗した試行の開始位置を復元する。
   Repeat は greedy な周回数と min/max を保つ。Call は同じ本体に入り、深さ上限は省略しない。
4. この実行条件で事象・scope・recovery・診断は記録されない。memo に作用する領域は除外する。
   途中失敗・資源上限では Session を一切変更せず参照経路へ戻す。
5. AST と出現表を要求した解析、診断 session は常に従来経路である。
   canonical JSON、AST、診断、corpus の観測はこの参照経路に一致する。
   操作計数 `statistics` は最適化により変わるため、同値比較では除外する。

同値の実行検証は生成 Rust のテストと差分 fuzz campaign の LL(1) on/off 比較が担う。
Java は後続 issue で同じ計画を移植する。
