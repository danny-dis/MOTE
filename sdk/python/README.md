# MOTE Python SDK

The MOTE Python SDK lets a Python application host tools and run MOTE through
the versioned stdio `mote-bridge` executable. It is source-installable and is
not published to PyPI. The SDK uses only the Python standard library at
runtime.

Requirements: Python 3.10 or newer (`requires-python = ">=3.10"`). A Rust
toolchain is not required when you use a downloaded bridge executable. The
bridge is available in the [GitHub Releases](https://github.com/danny-dis/MOTE/releases)
archives; this repository is private, so you need repository access. Release
binaries are unsigned/not notarized. Verify the archive checksum, expect an OS
warning where applicable, and do not treat a checksum as publisher identity.
See the repository [QUICKSTART](../../docs/QUICKSTART.md) for platform assets
and extraction instructions.

## Quick start

Run the following from the repository root after extracting a release archive
or building the bridge. On Windows, append `.exe` to the executable path.

```bash
# Source checkout option; requires stable Rust.
cargo build --locked --release --bin mote-bridge

python -m pip install ./sdk/python
```

Point `MOTE_BRIDGE` at the downloaded or built `mote-bridge`, set
`MOTE_MODEL_ENDPOINT` to a reachable OpenAI-compatible chat-completions
endpoint, and set `MOTE_MODEL`. If the endpoint requires authentication, set
`MODEL_API_KEY`; the manifest must refer to the variable by name with
`auth_env: "MODEL_API_KEY"`. No provider, gateway, model, or API key is
bundled, and the SDK does not install or start a service.

From the repository root:

```bash
python sdk/python/example.py
```

The checked-in example defaults to a local DMR-X-style endpoint and model
`auto`; change the environment for your provider. Its success checks include
the callback and the observed `HELLO` result, but running it requires a live,
reachable model endpoint.

For configuration fields, defaults, model providers, workspace boundaries and
timeouts, see [CONFIGURATION](../../docs/CONFIGURATION.md). For the complete
integration context, see [BUILDING_ON_MOTE](../../docs/BUILDING_ON_MOTE.md) and
[QUICKSTART](../../docs/QUICKSTART.md).

## API

```python
from mote_sdk import Client, Tool

client = Client(binary="/absolute/path/to/mote-bridge", timeout=150)
result = client.run(
    manifest={
        "name": "demo",
        "capabilities": ["uppercase"],
        "models": [{
            "provider": "openai-compatible",
            "model": "your-model",
            "endpoint": "https://your-provider.example/v1/chat/completions",
            "auth_env": "MODEL_API_KEY",
        }],
    },
    task='Call uppercase with {"text":"hello"}, then complete.',
    tools={"uppercase": Tool(
        "Input: text string. Returns uppercase text.",
        lambda data: data["text"].upper(),
    )},
)
if not result.success:
    raise RuntimeError("Agent run failed")
print(result.events)
```

`Client(binary="mote-bridge", timeout=150, cwd=None, env=None)` starts one
trusted executable per `run`. `binary` may be an executable/argument list for
an application-owned launcher; it is never passed through a shell. `timeout`
is a positive finite number of seconds. `env` overrides inherited environment
variables and `cwd` controls the bridge process working directory.

`run(manifest, task, tools=None)` returns `RunResult(state, events, returncode)`.
`success` requires state `completed` and exit code `0`; failed or cancelled
runtime work returns a false result. Setup, protocol, transport and deadline
failures raise `SetupError`, `ProtocolError`, `TransportError` or
`TimeoutError` (all subclass `MoteError`).

`Tool(description, handler)` accepts a JSON object and must return a string.
Validate domain-specific fields in the handler. Handler exceptions and
non-string results become a generic tool failure; exception text and sensitive
inputs are not sent to the model. Registering a tool is not permission: its
name must also be in `manifest["capabilities"]`. Runs and callbacks are not
automatically retried, and a failed or timed-out run may already have side
effects.

The SDK limits each bridge stdout frame to 1 MiB including its newline and retains
at most 64 KiB of stderr. `events` and model observations arrive after the run;
there is no live-event or progress-stream API. Treat them as potentially
sensitive. The SDK watchdog kills and reaps the direct bridge child on a
deadline, but Python callbacks run in the host process and cannot be forcibly
interrupted. Bound or isolate blocking callbacks and descendants yourself.
Neither the SDK nor the bridge is an OS sandbox; host code runs with the
application's privileges.

## Tests

From the repository root:

```bash
python -m pip install "./sdk/python[test]"
python -m pytest sdk/python/tests -q
```

Set `MOTE_BRIDGE` to a bridge executable before running the tests. The E2E
coverage uses a real subprocess and callback with a local scripted HTTP model;
it is deterministic and does not test live inference. No PyPI publication is
implied by this source package.
