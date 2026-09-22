# D-004 precedence climbing と既存 AST 契約の両立

決定: 条件付きで採用（M5、差分比較で確認できた塔だけ）。保守的生成（段ごとの関数）を先に作る。

既存契約: 左結合は `left + op[] + right[]`、右結合は各段で空または 1 要素、span は各 mapped rule の消費範囲。
優先順位数値で解析順を組み替えない（`specs/generators.md:141`）。

astra 提案の正規形（段 0 = 最低、H = 最高。trivia・副作用・recovery のない塔）:

```text
climb(min):
    x = atom()
    if x == FAIL: return FAIL
    tier = H + 1
    ceiling = H
    while (op, k) = peekOperator(min, ceiling):
        mark = checkpoint()
        consume(op)
        y = climb(assoc[k] == RIGHT ? k : k + 1)
        if y == FAIL:
            restore(mark)
            break
        x = lift(x, tier, k)
        x.op.append(op)
        x.right.append(y)
        x.end = cursor
        tier = k
        ceiling = assoc[k] == RIGHT ? k - 1 : k
    return lift(x, tier, min)

lift(x, from, to):
    for k = from - 1 downto to:
        x = NodeOfRule(k, left=x, op=[], right=[], span=[x.start, x.end])
    return x
```

- `lift` が演算子のない段も復元し、同じ左結合段ではリストへ追記。括弧内の同じ mapping class を外側へ混ぜない。
- `ceiling` は内側で右辺解析に失敗した高優先度演算子を外側で再解釈することを防ぐ。
- 独立 Python 試作: 2 段 × 左右結合 4 組合せ × 括弧 × 不完全入力 = 223,948 ケースで段別パーサと AST/span/消費位置一致
  （UBNF 実生成器での検証ではない）。
- trivia がある場合 `lift` の子 span コピーは使えない。各段の入口/出口と delimiter 消費位置の境界計画が要る
  （既存 chain は先頭と各要素後にも trivia を置く。`ParserGenerator.java:495`）。
- 対象外: 異なる trivia、先読み、recovery、非正規形、曖昧な演算子選択 → 通常経路。
