#!/usr/bin/env python3
"""生成 crate の Cargo.toml に scanner feature と timing bin を足す（harness の runner と同じ形）。"""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()
assert text.count("default = []") == 1, "scanner feature hook がありません"
text = text.replace("default = []", 'default = ["scanners"]', 1)
text += '\n[[bin]]\nname = "timing"\npath = "src/bin/timing.rs"\n'
open(path, "w", encoding="utf-8").write(text)
