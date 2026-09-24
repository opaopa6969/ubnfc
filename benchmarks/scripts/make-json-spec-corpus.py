#!/usr/bin/env python3
"""JSONTestSuite の y_* / n_* を `corpus/spec/json/` の仕様由来ケースへ落とす（issue #26）。

  usage: make-json-spec-corpus.py <JSONTestSuite/test_parsing> <corpus/spec/json>

期待値の出所:
  - **受理/拒否の別**は JSONTestSuite の `y_` / `n_` 接頭辞（この suite 自身が
    RFC 8259 から導いた分類で、どの実装の観測値でもない）。`i_`（実装依存）は
    取り込まない——仕様が決めていないものを corpus に入れない、という
    `corpus/spec/README.md` の絶対規則 3 に合わせる。
  - **引用**は RFC 8259 の該当節の逐語の一文。ファイル名の第 1 成分（array /
    object / number / string / structure）と符号化関連の語（UTF / BOM）で節を選ぶ。
  引用が実在することは `crates/ubnfc-harness/tests/spec_corpus.rs` が
  `benchmarks/external/rfc8259.txt`（scripts/fetch-external-tools.sh が取得）と照合する。
"""
import json
import pathlib
import re
import sys

URL = "https://www.rfc-editor.org/rfc/rfc8259.txt"
DOC = "benchmarks/external/rfc8259.txt"

# (見出し行そのもの, 逐語の一文)
SECTIONS = {
    "grammar": (
        "2.  JSON Grammar",
        "A JSON text is a serialized value.",
    ),
    "ws": (
        "2.  JSON Grammar",
        "Insignificant whitespace is allowed before or after any of the six structural characters.",
    ),
    "object": (
        "4.  Objects",
        "An object structure is represented as a pair of curly brackets surrounding zero or"
        " more name/value pairs (or members).",
    ),
    "array": (
        "5.  Arrays",
        "An array structure is represented as square brackets surrounding zero or more values"
        " (or elements).",
    ),
    "number": (
        "6.  Numbers",
        "Numeric values that cannot be represented in the grammar below (such as Infinity and"
        " NaN) are not permitted.",
    ),
    "string": (
        "7.  Strings",
        "All Unicode characters may be placed within the quotation marks, except for the"
        " characters that MUST be escaped: quotation mark, reverse solidus, and the control"
        " characters (U+0000 through U+001F).",
    ),
    "encoding": (
        "8.1.  Character Encoding",
        "JSON text exchanged between systems that are not part of a closed ecosystem MUST be"
        " encoded using UTF-8 [RFC3629].",
    ),
}

SUMMARY = {
    "grammar": "JSON text の全体構造（値 1 個と ws）。",
    "object": "オブジェクト（波括弧・メンバ・区切り）。",
    "array": "配列（角括弧・要素・区切り）。",
    "number": "数値（符号・整数部・小数部・指数部）。",
    "string": "文字列（引用符・エスケープ・生で置けない文字）。",
    "encoding": "文字符号化（UTF-8 必須、BOM、不正な byte 列）。",
}


def section_of(name: str) -> str:
    body = name[2:-5]
    if re.search(r"utf|bom", body, re.IGNORECASE):
        return "encoding"
    head = body.split("_")[0]
    if head in ("object", "array", "number", "string"):
        return head
    if head == "multidigit":
        return "number"
    if head == "incomplete":
        return "string"
    return "grammar"


def main() -> int:
    suite = pathlib.Path(sys.argv[1])
    out = pathlib.Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    for stale in out.glob("*.json"):
        stale.unlink()

    buckets: dict[str, list[dict]] = {key: [] for key in SUMMARY}
    for path in sorted(suite.glob("*.json")):
        name = path.name
        if name[:2] not in ("y_", "n_"):
            continue
        section = section_of(name)
        heading, sentence = SECTIONS[section]
        raw = path.read_bytes()
        case = {
            "id": f"json/{name[:-5]}",
            "scope": "parse",
            "citation": {"file": DOC, "heading": heading, "sentence": sentence, "url": URL},
        }
        try:
            text = raw.decode("utf-8")
            if "\x00" in text:
                raise UnicodeDecodeError("utf-8", raw, 0, 1, "NUL")
            case["input"] = text
        except UnicodeDecodeError:
            # UTF-8 として不正な byte 列と NUL は JSON 文字列に載せられない。
            case["inputHex"] = raw.hex()
        case["expectation"] = {"verdict": "accept" if name.startswith("y_") else "reject"}
        case["suiteFile"] = name
        buckets[section].append(case)

    written = 0
    for section, cases in buckets.items():
        if not cases:
            continue
        payload = {
            "family": "json",
            "section": SECTIONS[section][0],
            "summary": SUMMARY[section],
            "cases": cases,
        }
        target = out / f"{section}.json"
        target.write_text(
            json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        written += len(cases)
        print(f"{target.name}: {len(cases)} cases")
    print(f"total {written}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
