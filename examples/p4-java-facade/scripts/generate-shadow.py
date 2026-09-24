#!/usr/bin/env python3
"""shadow 用の P4PreferredAstMapper を UbnfcP4PreferredAstMapper から機械生成する。

tinyexpression には parser facade の SPI が無い（ServiceLoader も META-INF/services も無い）。
差し込める唯一の口は同 FQN クラスによる classpath shadow なので、公開面が食い違わないよう
shadow 側は手で書かず、本体から生成する。

使い方: generate-shadow.py [--write|--check]
"""
import os, subprocess, sys, tempfile

HEADER = """// scripts/generate-shadow.py が UbnfcP4PreferredAstMapper から生成する。手で編集しない。
// classpath 上でこのクラスを tinyexpression の jar より前に置くと、
// FormulaInfo -> DslGeneratedAstJavaEmitter -> JavaCodeCalculatorV3 -> javac の経路が
// そのまま ubnfc 生成パーサで動く（tinyexpression は無変更）。
"""


def transform(source):
    out = source
    out = out.replace("package org.unlaxer.tinyexpression.p4.ubnfc;",
                      "package org.unlaxer.tinyexpression.p4;")
    out = out.replace("import org.unlaxer.tinyexpression.p4.P4SourceText;\n", "")
    out = out.replace("import org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4AST;",
                      "import org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4AST;\n"
                      "import org.unlaxer.tinyexpression.p4.ubnfc.UbnfcP4Parse;")
    out = out.replace("UbnfcP4PreferredAstMapper", "P4PreferredAstMapper")
    return HEADER + out


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "--write"
    module = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    src = os.path.join(module,
        "src/main/java/org/unlaxer/tinyexpression/p4/ubnfc/UbnfcP4PreferredAstMapper.java")
    out = os.path.join(module, "src/shadow/java/org/unlaxer/tinyexpression/p4/P4PreferredAstMapper.java")
    text = transform(open(src, encoding="utf-8").read())
    if mode == "--write":
        open(out, "w", encoding="utf-8").write(text)
        print("書き出し: " + out)
        return
    current = open(out, encoding="utf-8").read() if os.path.exists(out) else ""
    if current != text:
        with tempfile.NamedTemporaryFile("w", suffix=".java", delete=False) as f:
            f.write(text)
            tmp = f.name
        subprocess.run(["diff", "-u", out, tmp])
        raise SystemExit("shadow が最新ではない: " + out)
    print("一致: " + out)


main()
