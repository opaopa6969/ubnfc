#!/usr/bin/env python3
"""P4(v1) と P4-v2 の driver 応答を同じ入力で比較する。

usage: compare.py <v1-driver> <v2-driver> <fixture-dir>

driver protocol（docs/harness-driver-protocol.md）は 1 行 1 リクエスト / 1 行 1 応答。

比較するのは「観測の同値」であり、唯一の正規化は **規則名の読み替え**である。
v1 は優先順位塔の各段に人間が名前を付けている（NumberTerm …）のに対し、v2 は
同じ塔を脱糖が作るので段の名前が機械生成になる（NumberExpression__lr1 …）。
名前が値に出る観測（diagnostics[].deepestRule / rulePath / farthestExpected の
合成 parser 表示名）は、この対応表で v2 側を v1 側の名前へ置き換えてから比べる。
対応は 1 対 1 で、脱糖の定義（候補順＝優先順位、最深が基底）から一意に決まる。

例外が 1 つある: v2 の `BooleanExpression__lrBase` は基底候補が
`BooleanFactor` 1 個の規則参照なので、v1 には対応する規則が無い「透過的な
1 段の間接」である。rulePath からはこの 1 要素を取り除く（AST も値も作らない）。

これ以外は AST を含めて**厳密比較**する。
"""
import json
import os
import re
import subprocess
import sys

# 脱糖が作る段 -> v1 の塔の段。候補順（高優先が先）から一意。
TIER_NAMES = {
    "NumberExpression__lr1": "NumberTerm",
    "NumberExpression__lrBase": "NumberFactor",
    "StringExpression__lrBase": "StringTerm",
    "BooleanExpression__lr1": "BooleanAndExpression",
    "BooleanExpression__lr2": "BooleanXorExpression",
}
# v1 に対応規則の無い透過的な 1 段。
TRANSPARENT = "BooleanExpression__lrBase"

# 診断契約（主 offset / farthest / farthestExpected、D-011 / D-014）を突く拒否入力。
# 塔の段と左再帰の段で「どこまで到達したか」がずれると、ここが最初に割れる。
REJECTS = [
    ("reject:trailing-plus", "1 + "),
    ("reject:double-star", "1 * * 2"),
    ("reject:unclosed-paren", "(1 + 2"),
    ("reject:unclosed-paren-mul", "(1 * 2"),
    ("reject:empty", ""),
    ("reject:bad-atom", "1 + @"),
    ("reject:bad-bool", "$a & "),
    ("reject:bad-xor", "$a ^ "),
    ("reject:bad-concat", "'a' + "),
    ("reject:var-decl", "var $a set 1 + ; $a"),
    ("prefix:number", "1 + 2 garbage"),
    ("prefix:paren", "(1 + 2) * "),
]
RENAME = re.compile("|".join(sorted(TIER_NAMES, key=len, reverse=True)))


def rename(value, key=None):
    """v2 の応答に現れる段名を v1 の塔の段名へ読み替える。

    `farthestExpected` は昇順に整列した集合なので、読み替えたあとに並べ直す
    （`NumberExpression__lr1Parser` と `NumberTermParser` では整列位置が違う）。
    """
    if isinstance(value, dict):
        return {k: rename(v, k) for k, v in value.items()}
    if isinstance(value, list):
        out = []
        for item in value:
            if isinstance(item, str) and TRANSPARENT in item:
                continue  # rulePath の透過的な 1 段
            out.append(rename(item, key))
        return sorted(out) if key == "farthestExpected" else out
    if isinstance(value, str):
        return RENAME.sub(lambda m: TIER_NAMES[m.group(0)], value)
    return value


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False)


def requests(fixtures):
    cases = []
    for name in sorted(os.listdir(fixtures)):
        if name.endswith(".tiny"):
            text = open(os.path.join(fixtures, name), encoding="utf-8").read()
            cases.append((name, text))
    cases.extend(REJECTS)
    return cases


def run(driver, cases):
    lines = [json.dumps({
        "id": index,
        "grammar": "TinyExpressionP4",
        "entry": None,
        "input": text,
        "options": {"lexical": False, "scope": True},
        "observations": {"family": "p4v2", "variant": "rust"},
    }, ensure_ascii=False) for index, (_, text) in enumerate(cases)]
    out = subprocess.run([driver], input="\n".join(lines) + "\n",
                         capture_output=True, text=True, check=True)
    responses = {}
    for line in out.stdout.splitlines():
        if line.strip():
            row = json.loads(line)
            responses[row["id"]] = row
    return responses


def main():
    v1_driver, v2_driver, fixtures = sys.argv[1:4]
    cases = requests(fixtures)
    a, b = run(v1_driver, cases), run(v2_driver, cases)
    failures = []
    raw_ast_equal = 0
    print(f"{'case':<26} {'accepted':>8} {'cursors':>8} {'ast':>6} {'diag':>6} {'all':>5}")
    for index, (name, _) in enumerate(cases):
        left, right = a[index], rename(b[index])
        accepted = left.get("accepted") == right.get("accepted")
        keys = ("consumed", "matched", "allConsumed", "fullConsumed", "fullMatched")
        cursors = [left.get(k) for k in keys] == [right.get(k) for k in keys]
        ast = canonical(left.get("ast")) == canonical(right.get("ast"))
        # AST には規則名が出ないので、読み替え前でも一致するはず。
        raw_ast_equal += canonical(left.get("ast")) == canonical(b[index].get("ast"))
        diag = canonical(left.get("diagnostics")) == canonical(right.get("diagnostics"))
        every = canonical(left) == canonical(right)
        ok = accepted and cursors and ast and diag and every
        if not ok:
            failures.append(name)
        print(f"{name:<26} {str(accepted):>8} {str(cursors):>8} {str(ast):>6} "
              f"{str(diag):>6} {str(every):>5}")
        if not ok:
            for key in sorted(set(left) | set(right)):
                if canonical(left.get(key)) != canonical(right.get(key)):
                    print(f"    差分 {key}:")
                    print(f"      v1: {canonical(left.get(key))[:600]}")
                    print(f"      v2: {canonical(right.get(key))[:600]}")
    print(f"\n応答全体の一致（段名の読み替えのみ）: {len(cases) - len(failures)}/{len(cases)}")
    print(f"AST の一致（読み替えなしの生比較）: {raw_ast_equal}/{len(cases)}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
