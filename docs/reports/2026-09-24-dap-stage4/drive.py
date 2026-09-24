import json, sys
from dapclient import Dap

cmd = [c for c in sys.argv[1:-3] if c != "--stepin"]
program, bp, out = sys.argv[-3], int(sys.argv[-2]), sys.argv[-1]
extra = {}
if '--ast' in cmd:
    cmd = [c for c in cmd if c != '--ast']
    extra = {"runtimeMode": "ast", "steppingMode": "ast"}

d = Dap(cmd)
record = {}
record['initialize'] = d.request("initialize", {"adapterID": "parity", "linesStartAt1": True, "columnsStartAt1": True}).get('body')
record['launch'] = d.request("launch", dict({"program": program, "stopOnEntry": True}, **extra)).get('success')
record['setBreakpoints'] = d.request("setBreakpoints", {
    "source": {"path": program, "name": program.split('/')[-1]},
    "breakpoints": [{"line": bp}]}).get('body')
d.request("configurationDone", {})
record['threads'] = d.request("threads").get('body')

def snap(label):
    st = d.request("stackTrace", {"threadId": 1}).get('body', {})
    frames = st.get('stackFrames', []) or []
    item = {"after": label, "frames": [
        {k: f.get(k) for k in ('id', 'name', 'line', 'column', 'endLine', 'endColumn')} for f in frames]}
    if frames:
        sc = d.request("scopes", {"frameId": frames[0]['id']}).get('body', {})
        item['scopes'] = []
        for scope in sc.get('scopes', []) or []:
            vs = d.request("variables", {"variablesReference": scope['variablesReference']}).get('body', {})
            item['scopes'].append({"name": scope.get('name'),
                                   "variables": [{k: v.get(k) for k in ('name', 'value', 'type')}
                                                 for v in (vs.get('variables') or [])]})
    return item

walk = "stepIn" if "--stepin" in sys.argv else "next"
cmdname = walk
steps = [snap("configurationDone")]
for i in range(4):
    reply = d.request(cmdname, {"threadId": 1})
    if not reply.get('success'):
        steps.append({"after": "%s%d" % (cmdname, i + 1), "error": reply.get('message')})
        break
    steps.append(snap("%s%d" % (cmdname, i + 1)))
record['steps'] = steps
record['evaluate'] = d.request("evaluate", {"expression": "1+1", "context": "repl"})
record['events'] = [e.get('event') + (":" + str(e.get('body', {}).get('reason'))
                    if e.get('event') == 'stopped' else '') for e in d.events]
open(out, 'w').write(json.dumps(record, ensure_ascii=False, indent=1))
d.close()
print("wrote", out)
