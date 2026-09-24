#!/usr/bin/env python3
"""4 実装の判定を 1 つの表にまとめる（issue #26）。

  usage: merge-verdicts.py <判定 TSV のあるディレクトリ> <JSONTestSuite/test_parsing>

入力: <dir>/{ubnfc-rust,ubnfc-java,antlr,tree-sitter}.tsv（path<TAB>verdict<TAB>reason）
出力: <dir>/verdicts.tsv と <dir>/summary.md
"""
import pathlib
import sys

IMPLS = ["ubnfc-rust", "ubnfc-java", "antlr", "tree-sitter"]
LABEL = {
    "ubnfc-rust": "ubnfc Rust",
    "ubnfc-java": "ubnfc Java",
    "antlr": "ANTLR4",
    "tree-sitter": "tree-sitter",
}


def load(path: pathlib.Path) -> dict[str, tuple[str, str]]:
    rows = {}
    # str.splitlines() は U+000B / U+000C / U+2028 等でも切るので使わない
    # （ANTLR の理由欄に問題の文字がそのまま入ることがある）。
    for line in path.read_text(encoding="utf-8").split("\n"):
        if not line.strip():
            continue
        parts = line.split("\t")
        name = pathlib.Path(parts[0]).name
        rows[name] = (parts[1], parts[2] if len(parts) > 2 else "")
    return rows


def main() -> int:
    out = pathlib.Path(sys.argv[1])
    suite = pathlib.Path(sys.argv[2])
    data = {impl: load(out / f"{impl}.tsv") for impl in IMPLS}
    names = sorted(data[IMPLS[0]])

    lines = ["file\texpected\t" + "\t".join(IMPLS) + "\treasons"]
    counts = {impl: {} for impl in IMPLS}
    suite_mismatch = {impl: [] for impl in IMPLS}
    impl_disagreement = []
    for name in names:
        kind = name[:2]
        expected = {"y_": "accept", "n_": "reject", "i_": "implementation-defined"}[kind]
        verdicts = {impl: data[impl][name][0] for impl in IMPLS}
        reasons = "; ".join(
            f"{impl}={data[impl][name][1]}" for impl in IMPLS if data[impl][name][1]
        )
        lines.append(
            f"{name}\t{expected}\t" + "\t".join(verdicts[i] for i in IMPLS) + f"\t{reasons}"
        )
        for impl in IMPLS:
            counts[impl][(kind, verdicts[impl])] = counts[impl].get((kind, verdicts[impl]), 0) + 1
            if kind in ("y_", "n_") and verdicts[impl] != expected:
                suite_mismatch[impl].append(name)
        if len(set(verdicts.values())) > 1:
            impl_disagreement.append((name, kind, dict(verdicts)))
    (out / "verdicts.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8")

    md = ["# JSONTestSuite 突き合わせ", ""]
    md.append(f"対象 {len(names)} 件（{suite}）", )
    md.append("")
    md.append("| 実装 | y_ 受理 / 全 | n_ 拒否 / 全 | i_ 受理 | i_ 拒否 | suite との不一致 |")
    md.append("|---|---:|---:|---:|---:|---|")
    total = {k: sum(1 for n in names if n[:2] == k) for k in ("y_", "n_", "i_")}
    for impl in IMPLS:
        c = counts[impl]
        md.append(
            f"| {LABEL[impl]} | {c.get(('y_','accept'),0)} / {total['y_']} "
            f"| {c.get(('n_','reject'),0)} / {total['n_']} "
            f"| {c.get(('i_','accept'),0)} | {c.get(('i_','reject'),0)} "
            f"| {', '.join(suite_mismatch[impl]) or 'なし'} |"
        )
    md.append("")
    md.append(f"## 実装間で判定が割れたファイル（{len(impl_disagreement)} 件）")
    md.append("")
    md.append("| file | 期待 | " + " | ".join(LABEL[i] for i in IMPLS) + " |")
    md.append("|---|---|" + "---|" * len(IMPLS))
    for name, kind, verdicts in impl_disagreement:
        expected = {"y_": "受理必須", "n_": "拒否必須", "i_": "実装依存"}[kind]
        md.append(f"| `{name}` | {expected} | " + " | ".join(verdicts[i] for i in IMPLS) + " |")
    (out / "summary.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    print("\n".join(md))
    return 0


if __name__ == "__main__":
    sys.exit(main())
