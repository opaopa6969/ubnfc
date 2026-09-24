import json, subprocess, sys, threading, queue

class Dap:
    def __init__(self, cmd):
        self.p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, bufsize=0)
        self.seq = 0
        self.events = []
        self.responses = queue.Queue()
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        f = self.p.stdout
        while True:
            headers = {}
            while True:
                line = f.readline()
                if not line: return
                line = line.decode('utf-8').strip()
                if line == '': break
                k, _, v = line.partition(':')
                headers[k.strip().lower()] = v.strip()
            n = int(headers.get('content-length', 0))
            body = json.loads(f.read(n).decode('utf-8'))
            if body.get('type') == 'event':
                self.events.append(body)
            else:
                self.responses.put(body)

    def request(self, command, arguments=None, timeout=60):
        self.seq += 1
        msg = {"seq": self.seq, "type": "request", "command": command}
        if arguments is not None: msg["arguments"] = arguments
        raw = json.dumps(msg).encode('utf-8')
        self.p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(raw) + raw)
        self.p.stdin.flush()
        return self.responses.get(timeout=timeout)

    def close(self):
        try: self.p.kill()
        except Exception: pass
