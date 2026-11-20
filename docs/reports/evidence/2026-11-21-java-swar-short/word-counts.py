import re
import subprocess
import sys
from pathlib import Path

work = Path(sys.argv[1]).resolve()
scripts = Path(__file__).resolve().parent
print("variant,method,lane0,lane1,lane2,lane3,lane4,skip_nonascii,skip_ascii")
for variant in ("before", "candidate"):
    generated = work / "generated" / variant
    profile = work / "counts" / variant
    profile.mkdir(parents=True, exist_ok=True)
    source = (generated / "src/gen/json/JsonParser.java").read_text()
    methods = []
    sections = re.split(r"(?=    private static long scan\d+\(char\[\] t, int p\) \{)", source)
    for index, section in enumerate(sections):
        if "long word = scanSwarWord" not in section:
            continue
        row = len(methods)
        methods.append(re.match(r"    private static long (scan\d+)\(", section)[1])
        marker = "if (mismatch == 0) { p += 4;"
        assert section.count(marker) == 1
        section = section.replace(marker, "WORD_COUNTS[%d][mismatch == 0 ? 4 : Long.numberOfTrailingZeros(mismatch) >>> 4]++;\n" % row + marker)
        if variant == "candidate":
            marker = "if (first >= 128) break;"
            assert section.count(marker) == 1 and section.count("break asciiScan;") == 1
            section = section.replace(marker, "if (first >= 128) { WORD_COUNTS[%d][5]++; break; }" % row)
            section = section.replace("break asciiScan;", "{ WORD_COUNTS[%d][6]++; break asciiScan; }" % row)
        sections[index] = section
    assert len(methods) == 5
    source = "".join(sections)
    marker = "private static final long SCAN_LANE_HIGH"
    assert source.count(marker) == 1
    source = source.replace(marker,
        "static final long[][] WORD_COUNTS = new long[%d][7];\n" % len(methods)
        + "static final String[] WORD_NAMES = {%s};\n" % ", ".join('"%s"' % method for method in methods)
        + marker)
    parser = profile / "JsonParser.java"
    parser.write_text(source)
    classes = generated / "classes"
    subprocess.run(["javac", "--release", "17", "-cp", str(classes), "-d", str(profile),
                    str(parser), str(scripts / "WordCounts.java")], check=True)
    output = subprocess.check_output(["java", "-cp", f"{profile}:{classes}", "gen.json.WordCounts",
                                      str(work / "inputs/json-1m.json")], text=True)
    for line in output.splitlines():
        print(f"{variant},{line}")
