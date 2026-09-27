# Python SDK (source distribution)

Python 3.10+, standard-library-only runtime. From the MOTE repository root:

```bash
cargo build --locked --release --bins
python -m pip install ./sdk/python
```

Set `MOTE_BRIDGE` to the absolute `target/release/mote-bridge` path (`.exe` on
Windows), `MOTE_MODEL_ENDPOINT` to an OpenAI-compatible chat-completions URL,
and `MOTE_MODEL` to a supported model. If authentication is needed, set
`MODEL_API_KEY` in the environment. Then run:

```bash
python sdk/python/example.py
```

The example defaults to local DMR-X and model `auto`; these are examples, not
proof that a gateway is running. No service or binary is automatically installed.

## API

```python
from mote_sdk import Client, Tool

result = Client(binary="/absolute/path/mote-bridge", timeout=150).run(
    manifest={
        "name": "demo", "capabilities": ["uppercase"],
        "models": [{"provider": "openai-compatible", "model": "your-model",
                    "endpoint": "https://your-provider/v1/chat/completions",
                    "auth_env": "MODEL_API_KEY"}],
    },
    task='Call uppercase with {"text":"hello"}, then complete.',
    tools={"uppercase": Tool("Input: text string. Returns uppercase text.",
                             lambda data: data["text"].upper())},
)
if not result.success:
    raise RuntimeError("Agent run failed")
print(result.events)
```

- `Client(binary="mote-bridge", timeout=150, cwd=None, env=None)` creates a
  synchronous client. `timeout` is positive finite seconds. `env` overrides
  inherited variables; `cwd` determines relative workspace paths. `binary` can
  also be an executable/argument list for an application-owned launcher.
- `run(manifest, task, tools=None)` starts one process, uses protocol v1 and
  returns `RunResult(state, events, returncode)`. `success` requires completed
  state and exit code zero. Failed/cancelled runtime runs return a false result;
  setup, protocol, transport and timeout problems raise a `MoteError` subclass
  (`SetupError`, `ProtocolError`, `TransportError`, `TimeoutError`).
- `Tool(description, handler)` accepts a JSON object and returns a string.
  Validate domain-specific fields inside the handler. Exceptions/non-string
  results produce a generic failure, never exception text in the model prompt.
- Registration is not permission: tool names also need manifest capabilities.
  No tool calls or runs are automatically retried. Failed runs may have side effects.
- stdout frames are capped at 1 MiB including newline; retained stderr at 64 KiB.
  Runtime events arrive at the end. Treat events/model observations as potentially
  sensitive. Successful completion is not an application-level acceptance test.
- A watchdog kills/reaps the direct bridge child on deadline, including blocked
  pipe writes. Python callbacks run synchronously in your process: the SDK cannot
  forcibly interrupt them. Bound/isolate blocking callbacks and child processes.
  This is not an OS sandbox.

## Tests

From the repository root, with `MOTE_BRIDGE` pointing at the built executable:

```bash
python -m pip install "./sdk/python[test]"
python -m pytest sdk/python/tests -q
```

The E2E test uses a real bridge and callback with a local scripted HTTP model.
It is deterministic, not live inference, and is skipped only if `MOTE_BRIDGE`
is absent. CI supplies it. No PyPI publication is implied by this source package.
