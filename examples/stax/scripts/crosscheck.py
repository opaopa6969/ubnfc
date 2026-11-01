#!/usr/bin/env python3
"""stax のサンプルを Rust backend と Java backend の driver に流し、判定と canonical AST を突き合わせる。

使い方: crosscheck.py <rust-driver> <java-classes-dir> <samples-dir>

比較するのは共有契約の側だけ（決定 D-011 / D-014):
  - accepted / allConsumed
  - canonical AST（type / span / fields の完全一致）
  - スコープ診断（未定義シンボルの名前と位置）
  - golden/<sample>.ast.json がある sample は、両 backend の canonical AST をその golden と
    突き合わせる（D-074: token の直接 capture は原文の文字列。`NUMBER @text` が
    片方だけ数値になる退行をここで止める）。STAX_UPDATE_GOLDEN=1 で Rust の出力を書き直す。
診断の expected ラベルと主 offset は言語別 oracle なので比較しない。
"""
import json
import os
import pathlib
import subprocess
import sys


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False)


def run(cmd, requests):
    proc = subprocess.run(cmd, input=requests, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr[:4000])
        raise SystemExit(f"driver が異常終了: {cmd[0]} ({proc.returncode})")
    out = {}
    for line in proc.stdout.splitlines():
        if not line.strip():
            continue
        response = json.loads(line)
        if response["id"] in out:
            raise SystemExit(f"driver の応答 id が重複: {response['id']}")
        out[response["id"]] = response
    return out


def scope_warnings(response):
    return sorted(
        (d.get("offset"), d.get("message"))
        for d in response.get("diagnostics", [])
        if d.get("severity") == "WARNING"
    )


def main():
    rust_driver, java_classes, samples_dir = sys.argv[1:4]
    samples = pathlib.Path(samples_dir)
    cases = []
    for directory, verdict in ((samples, "accept"), (samples.parent / "rejects", "reject")):
        entries = json.loads((directory / "expected.json").read_text(encoding="utf-8"))["samples"]
        listed = [entry["file"] for entry in entries]
        found = sorted(p.name for p in directory.glob("*.stax"))
        if sorted(listed) != found or len(set(listed)) != len(listed):
            raise SystemExit(f"{directory}: expected.json と *.stax が 1 対 1 で対応しない")
        if not entries or any(entry["parse"] != verdict for entry in entries):
            raise SystemExit(f"{directory}: parse の期待値はすべて {verdict} にする")
        if verdict == "accept" and len(entries) != 20:
            raise SystemExit("受理サンプルは必ず 20 本にする")
        cases.extend((directory / entry["file"], entry) for entry in entries)
    files = [path.name for path, _ in cases]
    if len(set(files)) != len(files):
        raise SystemExit("受理・拒否ケースのファイル名が重複している")

    requests = "\n".join(
        json.dumps(
            {
                "id": index,
                "grammar": "Stax",
                "entry": None,
                "input": path.read_text(encoding="utf-8"),
                "options": {"lexical": True, "scope": True},
            },
            ensure_ascii=False,
        )
        for index, (path, _) in enumerate(cases)
    ) + "\n"

    rust = run([rust_driver], requests)
    java = run(["java", "-cp", java_classes, "target.driver.Driver"], requests)
    for backend, responses in (("Rust", rust), ("Java", java)):
        if set(responses) != set(range(len(cases))):
            raise SystemExit(f"{backend}: driver の応答 id が要求と一致しない")

    failures = []
    golden_dir = samples.parent / "golden"
    goldens = sorted(golden_dir.glob("*.ast.json")) if golden_dir.is_dir() else []
    for path in goldens:
        name = path.name[: -len(".ast.json")]
        if name not in files:
            raise SystemExit(f"golden {path.name} に対応する sample が無い")
        index = files.index(name)
        r, j = rust.get(index, {}), java.get(index, {})
        if os.environ.get("STAX_UPDATE_GOLDEN") == "1":
            path.write_text(
                json.dumps(r.get("ast"), sort_keys=True, ensure_ascii=False, indent=1) + "\n",
                encoding="utf-8",
            )
        want = json.loads(path.read_text(encoding="utf-8"))
        for backend, response in (("Rust", r), ("Java", j)):
            if canonical(response.get("ast")) != canonical(want):
                failures.append(f"{name}: {backend} の canonical AST が golden/{path.name} と違う")
    print(f"canonical AST golden: {len(goldens)} 本（両 backend と突き合わせ）")
    print(f"{'サンプル':<26} {'受理':<6} {'AST':<5} {'スコープ':<8} 期待")
    for index, name in enumerate(files):
        want = cases[index][1]
        r, j = rust.get(index), java.get(index)
        if r is None or j is None:
            failures.append(f"{name}: 応答が無い")
            continue

        accept_ok = (
            r.get("accepted") == j.get("accepted")
            and r.get("allConsumed") == j.get("allConsumed")
        )
        ast_ok = canonical(r.get("ast")) == canonical(j.get("ast"))
        scope_ok = scope_warnings(r) == scope_warnings(j)

        want_accept = want["parse"] == "accept"
        verdict_ok = all(
            response.get("accepted") is want_accept
            and response.get("allConsumed") is want_accept
            and response.get("mappingError") is None
            for response in (r, j)
        )
        # 両 backend が AST を作らず null を返しても「一致」として通さない。
        if want_accept:
            for backend, response in (("Rust", r), ("Java", j)):
                ast = response.get("ast")
                if not (
                    isinstance(ast, dict)
                    and set(ast) == {"type", "span", "fields"}
                    and ast["type"] == "Program"
                    and isinstance(ast["fields"].get("items"), list)
                    and ast["fields"]["items"]
                ):
                    failures.append(f"{name}: {backend} の Program AST が無いか空")
        else:
            for backend, response in (("Rust", r), ("Java", j)):
                if not any(d.get("kind") == want["diagnostic"] for d in response.get("diagnostics", [])):
                    failures.append(f"{name}: {backend} の構文診断が無い")
        names = [m.split("'")[1] for _, m in scope_warnings(r)]
        want_ok = names == want.get("scopeWarnings", [])

        ok = accept_ok and ast_ok and scope_ok and verdict_ok and want_ok
        mark = "OK" if ok else "NG"
        print(
            f"{name:<26} {'一致' if accept_ok else '不一致':<6} "
            f"{'一致' if ast_ok else '不一致':<5} {'一致' if scope_ok else '不一致':<8} "
            f"{want['parse']}/{want.get('typecheck')} {mark}"
        )
        if not ok:
            if not verdict_ok:
                failures.append(f"{name}: parse の判定が期待と違う（期待 {want['parse']}）")
            if not want_ok:
                failures.append(f"{name}: スコープ警告が期待と違う {names} != {want.get('scopeWarnings')}")
            if not accept_ok:
                failures.append(f"{name}: 受理が backend 間で違う")
            if not ast_ok:
                failures.append(f"{name}: canonical AST が backend 間で違う")
            if not scope_ok:
                failures.append(f"{name}: スコープ診断が backend 間で違う")

    accepted = sum(1 for _, entry in cases if entry["parse"] == "accept")
    print(
        f"\n{len(files)} 本中 parse 受理 {accepted} 本 / 拒否 {len(files) - accepted} 本、"
        f"型検査で落ちる期待 {sum(1 for _, entry in cases if entry.get('typecheck') == 'error')} 本（stage 3）"
    )
    if failures:
        print("\n不一致:")
        for line in failures:
            print(" -", line)
        raise SystemExit(1)
    print("両 backend の受理・canonical AST・スコープ診断はすべて一致した。")


if __name__ == "__main__":
    main()
