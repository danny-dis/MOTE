"""Small stdlib-only host SDK for the mote stdio bridge."""
import json
import math
import os
import queue
import subprocess
import threading
import time
from dataclasses import dataclass
from typing import Callable, Mapping

MAX_FRAME = 1048576
MAX_STDERR = 65536


class MoteError(Exception):
    pass


class TransportError(MoteError):
    pass


class ProtocolError(MoteError):
    pass


class SetupError(MoteError):
    pass


class TimeoutError(MoteError):
    pass


@dataclass(frozen=True)
class Tool:
    description: str
    handler: Callable[[dict], str]


@dataclass(frozen=True)
class RunResult:
    state: str
    events: list
    returncode: int

    @property
    def success(self):
        return self.state == "completed" and self.returncode == 0


class Client:
    def __init__(self, binary="mote-bridge", timeout=150, cwd=None, env=None):
        if isinstance(timeout, bool) or not isinstance(timeout, (int, float)) or not math.isfinite(timeout) or not 0 < timeout <= threading.TIMEOUT_MAX:
            raise SetupError("timeout must be positive, finite and supported by threading")
        self.binary, self.timeout, self.cwd, self.env = binary, timeout, cwd, env

    def run(self, manifest: dict, task: str, tools: Mapping[str, Tool] | None = None) -> RunResult:
        if not isinstance(manifest, dict) or not isinstance(task, str):
            raise SetupError("manifest must be an object and task must be a string")
        tools = {} if tools is None else tools
        if not isinstance(tools, Mapping):
            raise SetupError("tools must be a mapping")
        for name, tool in tools.items():
            if not isinstance(name, str) or not isinstance(tool, Tool) or not isinstance(tool.description, str) or not callable(tool.handler):
                raise SetupError("invalid tool")
        caps = manifest.get("capabilities", [])
        if not isinstance(caps, list) or any(not isinstance(x, str) for x in caps):
            raise SetupError("manifest capabilities must be strings")
        env = os.environ.copy()
        if self.env:
            env.update(self.env)
        argv = self.binary if isinstance(self.binary, (list, tuple)) else [self.binary]
        try:
            proc = subprocess.Popen(list(argv), stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, cwd=self.cwd, env=env, shell=False, bufsize=0)
        except (OSError, ValueError) as exc:
            raise TransportError("could not start mote bridge") from exc
        # Keep protocol buffering bounded if a child floods stdout while a host
        # callback is running; the reader then applies backpressure to the pipe.
        outq, errq = queue.Queue(maxsize=4), queue.Queue(maxsize=1)
        stop, expired = threading.Event(), threading.Event()
        readers = [
            threading.Thread(target=self._read_stdout, args=(proc.stdout, outq, stop), daemon=True),
            threading.Thread(target=self._read_stderr, args=(proc.stderr, errq), daemon=True),
        ]
        deadline = time.monotonic() + self.timeout
        def expire():
            expired.set()
            self._kill(proc)
        watchdog = threading.Timer(self.timeout, expire)
        watchdog.daemon = True
        watchdog.start()
        for reader in readers:
            reader.start()
        terminal = None
        try:
            self._send(proc, {"type": "run", "protocol": 1, "manifest": manifest, "task": task,
                              "tools": [{"name": n, "description": t.description} for n, t in tools.items()]})
            expected_id = 1
            while terminal is None:
                item = self._next(outq, deadline)
                if item is None:
                    raise ProtocolError("bridge closed before run result")
                if isinstance(item, Exception):
                    raise item
                msg = self._decode(item)
                typ = msg.get("type")
                if typ == "tool_call":
                    ident = msg.get("id")
                    if isinstance(ident, bool) or not isinstance(ident, int) or ident != expected_id:
                        raise ProtocolError("invalid tool call id")
                    expected_id += 1
                    name, inp = msg.get("name"), msg.get("input")
                    if not isinstance(name, str) or not isinstance(inp, dict):
                        raise ProtocolError("invalid tool call")
                    if name not in tools or name not in caps:
                        self._send(proc, {"type": "tool_result", "id": ident, "error": "tool unavailable"})
                        continue
                    try:
                        value = tools[name].handler(inp)
                        if not isinstance(value, str):
                            raise TypeError
                    except Exception:
                        self._send(proc, {"type": "tool_result", "id": ident, "error": "tool failed"})
                    else:
                        self._send(proc, {"type": "tool_result", "id": ident, "output": value})
                elif typ == "run_result":
                    if set(msg) - {"type", "state", "events"} or msg.get("state") not in {"completed", "failed", "cancelled"} or not isinstance(msg.get("events"), list) or any(not isinstance(x, dict) for x in msg["events"]):
                        raise ProtocolError("invalid run result")
                    terminal = msg
                elif typ == "error":
                    raise ProtocolError("bridge protocol error")
                else:
                    raise ProtocolError("unknown bridge message")
            self._close_input(proc)
            if self._drain(outq, deadline):
                raise ProtocolError("stdout after terminal result")
            rc = self._wait(proc, deadline)
            if expired.is_set():
                raise TimeoutError("bridge timed out")
            state = terminal["state"]
            if (state == "completed") != (rc == 0):
                raise ProtocolError("state and exit status disagree")
            if state != "completed" and rc != 1:
                raise ProtocolError("invalid failed exit status")
            return RunResult(state, terminal["events"], rc)
        except MoteError:
            if expired.is_set():
                raise TimeoutError("bridge timed out") from None
            raise
        except (BrokenPipeError, OSError, ValueError) as exc:
            if expired.is_set():
                raise TimeoutError("bridge timed out") from None
            raise TransportError("bridge transport failed") from exc
        finally:
            watchdog.cancel()
            stop.set()
            if proc.poll() is None:
                self._kill(proc)
            for reader in readers:
                reader.join(timeout=.2)
            for stream in (proc.stdin, proc.stdout, proc.stderr):
                try: stream.close()
                except Exception: pass

    @staticmethod
    def _read_stdout(stream, q, stop):
        def put(value):
            while not stop.is_set():
                try:
                    q.put(value, timeout=.05)
                    return True
                except queue.Full:
                    pass
            return False
        pending = bytearray()
        try:
            while not stop.is_set():
                chunk = stream.read(65536)
                if not chunk:
                    if pending:
                        put(ProtocolError("unterminated frame"))
                    break
                pending.extend(chunk)
                while b"\n" in pending:
                    end = pending.index(10) + 1
                    if end > MAX_FRAME:
                        put(ProtocolError("oversized frame"))
                        return
                    line = bytes(pending[:end])
                    del pending[:end]
                    if not put(line):
                        return
                if len(pending) >= MAX_FRAME:
                    put(ProtocolError("oversized frame"))
                    return
        except (OSError, ValueError):
            put(TransportError("stdout read failed"))
        finally:
            put(None)

    @staticmethod
    def _read_stderr(stream, q):
        retained = bytearray()
        try:
            while True:
                data = stream.read(8192)
                if not data: break
                if len(retained) < MAX_STDERR: retained.extend(data[:MAX_STDERR-len(retained)])
        except (OSError, ValueError):
            pass
        finally:
            q.put(bytes(retained))

    @staticmethod
    def _decode(line):
        try:
            obj = json.loads(line.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise ProtocolError("invalid UTF-8 or JSON frame") from exc
        if not isinstance(obj, dict): raise ProtocolError("frame must be an object")
        return obj

    @staticmethod
    def _send(proc, obj):
        raw = json.dumps(obj, ensure_ascii=False, separators=(",", ":")).encode("utf-8") + b"\n"
        if len(raw) > MAX_FRAME: raise ProtocolError("outgoing frame too large")
        remaining = memoryview(raw)
        while remaining:
            written = proc.stdin.write(remaining)
            if not written:
                raise TransportError("bridge stdin closed")
            remaining = remaining[written:]

    @staticmethod
    def _next(q, deadline):
        left = deadline - time.monotonic()
        if left <= 0: raise TimeoutError("bridge timed out")
        try: return q.get(timeout=left)
        except queue.Empty: raise TimeoutError("bridge timed out")

    @staticmethod
    def _wait(proc, deadline):
        while proc.poll() is None:
            if time.monotonic() >= deadline:
                Client._kill(proc); raise TimeoutError("bridge timed out")
            time.sleep(.01)
        return proc.returncode

    @staticmethod
    def _drain(q, deadline):
        # EOF is a queued sentinel, not an empty-queue snapshot: the child may
        # exit before its stdout reader has delivered all buffered frames.
        return Client._next(q, deadline) is not None

    @staticmethod
    def _close_input(proc):
        try: proc.stdin.close()
        except Exception: pass

    @staticmethod
    def _kill(proc):
        if proc.poll() is None:
            try: proc.kill()
            except OSError: pass
        try: proc.wait(timeout=2)
        except subprocess.TimeoutExpired: pass


__all__ = ["Client", "Tool", "RunResult", "MoteError", "TransportError", "ProtocolError", "SetupError", "TimeoutError"]
