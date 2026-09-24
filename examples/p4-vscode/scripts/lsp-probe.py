#!/usr/bin/env python3
"""LSP クライアントの台本。initialize → didOpen → publishDiagnostics を集めて JSON で出す。

新旧どちらのサーバにも同じ手順で話しかけられる（差は jar と languageId だけ）。
使い方:
  lsp-probe.py --jar <server.jar> --language-id <id> --fixtures <dir> [--jvm-arg X]...
"""
import argparse
import json
import os
import pathlib
import subprocess
import sys
import threading
import time


class Client:
    def __init__(self, command, cwd=None):
        self.process = subprocess.Popen(
            command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, cwd=cwd)
        self.next_id = 0
        self.inbox = []
        self.lock = threading.Condition()
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()

    def _read(self):
        stream = self.process.stdout
        while True:
            length = None
            while True:
                line = stream.readline()
                if not line:
                    return
                if line in (b"\r\n", b"\n"):
                    break
                name, _, value = line.decode("ascii", "replace").partition(":")
                if name.strip().lower() == "content-length":
                    length = int(value.strip())
            if length is None:
                continue
            body = stream.read(length)
            with self.lock:
                self.inbox.append(json.loads(body.decode("utf-8")))
                self.lock.notify_all()

    def send(self, message):
        body = json.dumps(message, ensure_ascii=False).encode("utf-8")
        self.process.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        self.process.stdin.flush()

    def request(self, method, params, timeout=60.0):
        self.next_id += 1
        ident = self.next_id
        self.send({"jsonrpc": "2.0", "id": ident, "method": method, "params": params})
        return self.wait(lambda m: m.get("id") == ident, timeout)

    def notify(self, method, params):
        self.send({"jsonrpc": "2.0", "method": method, "params": params})

    def wait(self, predicate, timeout):
        deadline = time.monotonic() + timeout
        with self.lock:
            while True:
                for index, message in enumerate(self.inbox):
                    if predicate(message):
                        return self.inbox.pop(index)
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    return None
                self.lock.wait(remaining)

    def close(self):
        try:
            self.request("shutdown", None, timeout=10.0)
            self.notify("exit", None)
        except Exception:
            pass
        try:
            self.process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.process.kill()


def simplify(diagnostic):
    r = diagnostic["range"]
    return {
        "range": [r["start"]["line"], r["start"]["character"], r["end"]["line"], r["end"]["character"]],
        "severity": diagnostic.get("severity"),
        "code": diagnostic.get("code") if isinstance(diagnostic.get("code"), (str, int)) else None,
        "source": diagnostic.get("source"),
        "message": diagnostic.get("message"),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--jar", required=True)
    parser.add_argument("--language-id", required=True)
    parser.add_argument("--fixtures", required=True)
    parser.add_argument("--jvm-arg", action="append", default=[])
    parser.add_argument("--settle", type=float, default=1.5,
                        help="publishDiagnostics を待ってから追加分を拾う秒数")
    parser.add_argument("--out", default="-")
    args = parser.parse_args()

    command = ["java", *args.jvm_arg, "-jar", args.jar]
    client = Client(command)
    client.request("initialize", {
        "processId": os.getpid(),
        "rootUri": None,
        "capabilities": {"textDocument": {"publishDiagnostics": {"relatedInformation": True}}},
    })
    client.notify("initialized", {})

    report = {}
    for path in sorted(pathlib.Path(args.fixtures).glob("*.tiny")):
        text = path.read_text(encoding="utf-8")
        uri = path.resolve().as_uri()
        client.notify("textDocument/didOpen", {"textDocument": {
            "uri": uri, "languageId": args.language_id, "version": 1, "text": text}})
        message = client.wait(
            lambda m: m.get("method") == "textDocument/publishDiagnostics"
            and m["params"]["uri"] == uri, timeout=120.0)
        diagnostics = [] if message is None else message["params"]["diagnostics"]
        timed_out = message is None
        # 追いかけて来る publish があれば最後のものを採る。
        deadline = time.monotonic() + args.settle
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            more = client.wait(
                lambda m: m.get("method") == "textDocument/publishDiagnostics"
                and m["params"]["uri"] == uri, timeout=remaining)
            if more is None:
                break
            diagnostics = more["params"]["diagnostics"]
        report[path.name] = {
            "timedOut": timed_out,
            "diagnostics": [simplify(d) for d in diagnostics],
        }
    client.close()
    text = json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True)
    if args.out == "-":
        print(text)
    else:
        pathlib.Path(args.out).write_text(text + "\n", encoding="utf-8")


if __name__ == "__main__":
    sys.exit(main())
