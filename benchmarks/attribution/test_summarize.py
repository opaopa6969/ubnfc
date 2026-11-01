"""包含コストの二重計上・別 thread の混入を防ぐ集計の検査。"""
import contextlib
import io
import json
import pathlib
import tempfile
import unittest

import summarize


class SummaryTest(unittest.TestCase):
    def capture(self, data, function, *args):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "input"
            path.write_text(data)
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                function(path, *args)
            return out.getvalue()

    def test_callgrind_excludes_edge_cost_and_resolves_compressed_names(self):
        data = "positions: line\nevents: Ir Bcm\nfn=(1) caller\n1 40 2\ncfn=(2) child\ncalls=6 1\n1 900 80\nfn=(2)\n1 60 4\n"
        output = self.capture(data, summarize.callgrind, 2)
        self.assertIn("合計\t—\t未取得\t100\t50.000\t3.000", output)
        self.assertIn("child\t3.000\t未取得\t60.000\t30.000\t2.000", output)

    def test_jfr_only_main_top_frame(self):
        def event(thread, name):
            return {"values": {"sampledThread": {"javaName": thread}, "stackTrace": {"frames": [{"method": {"type": {"name": "C"}, "name": name, "descriptor": "()V"}}]}}}
        data = json.dumps({"recording": {"events": [event("main", "hot"), event("GC", "other")]}})
        output = self.capture(data, summarize.jfr)
        self.assertIn("C.hot()V\t1\t100.000", output)
        self.assertNotIn("other", output)

    def test_perf_period_and_empty_failure(self):
        output = self.capture("  25.00%  4000  [.] hot  -  -\n", summarize.perf, 2)
        self.assertIn("[.] hot\t2000.000\t25.00", output)
        with self.assertRaises(ValueError):
            self.capture("# Samples: 0\n", summarize.perf, 2)


if __name__ == "__main__":
    unittest.main()
