import json
import os
import stat
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

import pytest

from mote_sdk import Client, MoteError, ProtocolError, Tool, TransportError


ROOT = Path(__file__).parent


def child(tmp_path, script):
    p = tmp_path / "child.py"
    p.write_text(script, encoding="utf-8")
    return str(p)


def client(tmp_path, script, **kw):
    return Client(binary=sys.executable, cwd=str(tmp_path), env={"PYTHONPATH": str(ROOT.parent)}, **kw), [sys.executable, child(tmp_path, script)]


def test_success_and_callback(tmp_path):
    script = """import sys,json
run=json.loads(sys.stdin.readline()); print(json.dumps({'type':'tool_call','id':1,'name':'up','input':{'text':'hello'}}),flush=True)
r=json.loads(sys.stdin.readline()); assert r['output']=='HELLO'; print(json.dumps({'type':'run_result','state':'completed','events':[{'kind':'observation','text':'HELLO'}]}),flush=True)
"""
    c, _ = client(tmp_path, script)
    c.binary = [sys.executable, child(tmp_path, script)]
    got = c.run({"capabilities": ["up"]}, "go", {"up": Tool("upper", lambda x: x["text"].upper())})
    assert got.success and got.events[0]["text"] == "HELLO" and got.returncode == 0


def test_denied_tool_does_not_run(tmp_path):
    script = """import sys,json
json.loads(sys.stdin.readline()); print(json.dumps({'type':'tool_call','id':1,'name':'up','input':{}}),flush=True); print(json.dumps({'type':'run_result','state':'failed','events':[]}),flush=True); sys.exit(1)
"""
    c, _ = client(tmp_path, script); c.binary = [sys.executable, child(tmp_path, script)]
    called = []
    result = c.run({"capabilities": []}, "go", {"up": Tool("x", lambda x: called.append(1) or "x")})
    assert not called and not result.success


@pytest.mark.parametrize("kind", ["json", "oversized", "truncated"])
def test_bad_frames(tmp_path, kind):
    payload = {"json": "b'{bad}\\n'", "oversized": "b'x' * 1048577 + b'\\n'", "truncated": "b'{\\\"type\\\":\\\"run_result\\\"'"}[kind]
    script = "import sys; sys.stdin.buffer.readline(); sys.stdout.buffer.write(" + payload + "); sys.stdout.buffer.flush()"
    c, _ = client(tmp_path, script); c.binary = [sys.executable, child(tmp_path, script)]
    with pytest.raises((ProtocolError, TransportError)):
        c.run({}, "x")


def test_callback_errors_redacted_and_non_string(tmp_path):
    script = """import sys,json
json.loads(sys.stdin.readline()); print(json.dumps({'type':'tool_call','id':1,'name':'x','input':{'secret':'dont-log'}}),flush=True); print(json.dumps({'type':'run_result','state':'failed','events':[]}),flush=True); sys.exit(1)
"""
    c, _ = client(tmp_path, script); c.binary = [sys.executable, child(tmp_path, script)]
    seen = []
    c.run({"capabilities": ["x"]}, "x", {"x": Tool("x", lambda x: seen.append(x) or 4)})
    assert seen == [{"secret": "dont-log"}]


@pytest.mark.parametrize("messages", [
    [{"type":"tool_call","id":2,"name":"x","input":{}}],
    [{"type":"tool_call","id":1,"name":"unknown","input":{}}],
    [{"type":"run_result","state":"completed","events":[]},{"type":"run_result","state":"completed","events":[]}],
    [{"type":"run_result","state":"completed","events":[]}],
])
def test_protocol_validation(tmp_path, messages):
    script = "import sys,json; sys.stdin.readline(); [print(json.dumps(x), flush=True) for x in " + repr(messages) + "]"
    c, _ = client(tmp_path, script); c.binary = [sys.executable, child(tmp_path, script)]
    if len(messages) == 1 and messages[0].get("type") == "run_result":
        assert c.run({"capabilities": ["x"]}, "x", {"x": Tool("x", lambda x: "ok")}).success
    else:
        with pytest.raises(MoteError): c.run({"capabilities": ["x"]}, "x", {"x": Tool("x", lambda x: "ok")})


def test_timeout_kills_child(tmp_path):
    marker = tmp_path / "alive"
    script = "import sys,time; sys.stdin.readline(); open(" + repr(str(marker)) + ",'w').close(); time.sleep(30)"
    c, _ = client(tmp_path, script, timeout=1); c.binary = [sys.executable, child(tmp_path, script)]
    with pytest.raises(MoteError): c.run({}, "x")
    assert marker.exists()


def test_e2e_bridge():
    binary = os.environ.get("MOTE_BRIDGE")
    if not binary: pytest.skip("MOTE_BRIDGE not set")
    class Model(BaseHTTPRequestHandler):
        calls = 0
        def do_POST(self):
            length = int(self.headers.get("Content-Length", "0"))
            body = json.loads(self.rfile.read(length))
            Model.calls += 1
            # The deterministic fixture presents the runtime's small action language,
            # then returns the observed tool output as the assistant completion.
            content = '{"action":"custom","name":"uppercase","input":{"text":"hello"},"summary":"uppercase hello"}' if Model.calls == 1 else '{"action":"complete","summary":"HELLO"}'
            payload = json.dumps({"choices": [{"message": {"role": "assistant", "content": content}}]}).encode()
            self.send_response(200); self.send_header("Content-Type", "application/json"); self.send_header("Content-Length", str(len(payload))); self.end_headers(); self.wfile.write(payload)
        def log_message(self, *_): pass
    server = HTTPServer(("127.0.0.1", 0), Model)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        endpoint = "http://127.0.0.1:%d/v1/chat/completions" % server.server_port
        manifest = {"name": "sdk-e2e", "workspace": ".", "capabilities": ["uppercase"],
                    "models": [{"provider": "openai-compatible", "model": "fixture", "endpoint": endpoint}]}
        result = Client(binary=binary, timeout=15).run(
            manifest, "Ask uppercase for hello and finish after observing HELLO.",
            {"uppercase": Tool("Input object has text. Returns uppercase text.", lambda x: x["text"].upper())})
        assert result.success and any("HELLO" in json.dumps(event) for event in result.events)
    finally:
        server.shutdown()
