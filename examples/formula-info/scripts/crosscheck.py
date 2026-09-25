#!/usr/bin/env python3
"""formula-info のサンプルを Rust backend と Java backend の driver に流し、判定と canonical AST を
突き合わせる（issue #83）。

使い方: crosscheck.py <rust-driver> <java-classes-dir> <samples-dir>

比較するのは共有契約の側だけ（stax の scripts/crosscheck.py と同じ枠組み）:
  - accepted / allConsumed
  - canonical AST（type / span / fields の完全一致）
この文法に scope annotation は無いのでスコープ診断は比較しない。
expected.json の parse: accept/reject（構文層の判定）と一致するかも見る。
loaderReject: true の 9 本は構文層では accept のまま（旧 Java loader の意味検査はこの
文法の外側なので、ここでは比較対象にしない）。
"""
import json
import pathlib
import subprocess
import sys


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False)


def run(cmd, requests):
    proc = subprocess.run(cmd, input=requests, capture_output=True, text=True, encoding="utf-8")
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr[:4000])
        raise SystemExit(f"driver が異常終了: {cmd[0]} ({proc.returncode})")
    out = {}
    for line in proc.stdout.splitlines():
        if not line.strip():
            continue
        response = json.loads(line)
        out[response["id"]] = response
    return out


def main():
    rust_driver, java_classes, samples_dir = sys.argv[1:4]
    samples = pathlib.Path(samples_dir)
    expected = json.loads((samples / "expected.json").read_text(encoding="utf-8"))
    files = [entry["file"] for entry in expected["samples"]]
    for name in sorted(p.name for p in samples.glob("*.fi")):
        if name not in files:
            raise SystemExit(f"expected.json に {name} の期待値がない")

    requests = "\n".join(
        json.dumps(
            {
                "id": index,
                "grammar": "FormulaInfo",
                "entry": None,
                "input": (samples / name).read_text(encoding="utf-8"),
                "options": {"lexical": True, "scope": False},
            },
            ensure_ascii=False,
        )
        for index, name in enumerate(files)
    ) + "\n"

    rust = run([rust_driver], requests)
    java = run(["java", "-cp", java_classes, "target.driver.Driver"], requests)

    failures = []
    print(f"{'サンプル':<52} {'受理':<6} {'AST':<5} 期待")
    for index, name in enumerate(files):
        want = expected["samples"][index]
        r, j = rust.get(index), java.get(index)
        if r is None or j is None:
            failures.append(f"{name}: 応答が無い")
            continue

        accept_ok = (
            r.get("accepted") == j.get("accepted")
            and r.get("allConsumed") == j.get("allConsumed")
        )
        ast_ok = canonical(r.get("ast")) == canonical(j.get("ast"))

        want_accept = want["parse"] == "accept"
        verdict_ok = r.get("allConsumed") is want_accept

        ok = accept_ok and ast_ok and verdict_ok
        mark = "OK" if ok else "NG"
        print(
            f"{name:<52} {'一致' if accept_ok else '不一致':<6} "
            f"{'一致' if ast_ok else '不一致':<5} {want['parse']} {mark}"
        )
        if not ok:
            if not verdict_ok:
                failures.append(f"{name}: parse の判定が期待と違う（期待 {want['parse']}）")
            if not accept_ok:
                failures.append(f"{name}: 受理が backend 間で違う")
            if not ast_ok:
                failures.append(f"{name}: canonical AST が backend 間で違う")

    accepted = sum(1 for e in expected["samples"] if e["parse"] == "accept")
    loader_reject = sum(1 for e in expected["samples"] if e.get("loaderReject"))
    print(
        f"\n{len(files)} 本中 構文層で受理 {accepted} 本 / 構文拒否 "
        f"{len(files) - accepted} 本（うち構文は受理だが loader 拒否 {loader_reject} 本を含む）"
    )
    if failures:
        print("\n不一致:")
        for line in failures:
            print(" -", line)
        raise SystemExit(1)
    print("両 backend の受理・canonical AST はすべて一致した。")


if __name__ == "__main__":
    main()
