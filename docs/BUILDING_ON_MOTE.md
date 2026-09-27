# Build an application on MOTE

MOTE supplies the execution loop. Your application owns authentication, UI,
persistence, scheduling, business tools, approvals, and output validation.
Choose the language boundary you need; an HTTP service is not required.

## Get the engine

For Python/TypeScript, download and extract the archive for your platform from
[GitHub Releases](https://github.com/danny-dis/MOTE/releases/latest). Follow the
[binary quick start](QUICKSTART.md), verify the checksum, and use the absolute path
to `mote-bridge` (`mote-bridge.exe` on Windows). No Rust installation is needed.
Repository access is required while the project is private; binaries are unsigned.

The SDKs are installed from this source checkout separately. They are not published
to npm/PyPI and do not bundle or download the executable. If you prefer to compile,
run this from a checkout with stable Rust and its native build tools installed:

```bash
cargo build --locked --release --bins
```

This produces `target/release/mote` and `target/release/mote-bridge` (append `.exe`
on Windows). `mote` is the existing YAML CLI. `mote-bridge` lets Python/TypeScript
applications supply callbacks over a versioned stdio protocol. The SDK does not
download binaries or start a server. Point it at a downloaded or locally built bridge you trust.

## Python

Requires Python 3.10 or newer; runtime uses the standard library only.

```bash
python -m pip install ./sdk/python
```

```python
from mote_sdk import Client, Tool

client = Client(binary="/absolute/path/to/mote-bridge")
result = client.run(
    manifest={
        "name": "support-worker",
        "workspace": ".",
        "capabilities": ["uppercase"],
        "models": [{
            "provider": "openai-compatible",
            "model": "your-model-name",
            "endpoint": "http://127.0.0.1:8080/v1/chat/completions",
        }],
    },
    task='Call uppercase with {"text":"hello"}, then complete.',
    tools={"uppercase": Tool(
        description="Input: text (string). Returns uppercase text.",
        handler=lambda data: data["text"].upper(),
    )},
)
if not result.success:
    raise RuntimeError("Agent run failed; inspect its events")
print(result.events)
```

The endpoint must be reachable; change it/model for your provider. For API keys,
set `auth_env` in the model configuration to an environment-variable **name**.
See [Configuration](CONFIGURATION.md) for adapter names, fields, and defaults.
The SDK inherits environment variables and accepts explicit overrides; it does
not embed secrets in requests to the bridge. See [Python SDK](../sdk/python/README.md)
for errors, limits, testing and the runnable example.

## TypeScript / Node.js

Requires Node 20 or newer. Build the local package before installing it into an
application; it is not published to npm by this change.

```bash
npm --prefix sdk/typescript ci
npm --prefix sdk/typescript run build
# From your application directory:
npm install /absolute/path/to/MOTE/sdk/typescript
```

```typescript
import { Client } from "@mote-agent/sdk";

const client = new Client({ binary: "/absolute/path/to/mote-bridge" });
const result = await client.run({
  manifest: {
    name: "support-worker",
    workspace: ".",
    capabilities: ["uppercase"],
    models: [{
      provider: "openai-compatible",
      model: "your-model-name",
      endpoint: "http://127.0.0.1:8080/v1/chat/completions",
    }],
  },
  task: 'Call uppercase with {"text":"hello"}, then complete.',
  tools: {
    uppercase: {
      description: "Input: text (string). Returns uppercase text.",
      handler: async (input) => {
        if (typeof input.text !== "string") throw new Error("Invalid input");
        return input.text.toUpperCase();
      },
    },
  },
});
if (!result.success) throw new Error("Agent run failed");
console.log(result.events);
```

See [TypeScript SDK](../sdk/typescript/README.md) for the actual API and examples.
Both SDKs support built-in capabilities without registering any custom tools.
They launch processes without a command shell and support binary paths with spaces.

## Rust

Use [the application starter](../starters/rust-agent/README.md). It loads a real
HTTP model configuration, registers a host tool, and validates its output file.
The Rust crate is `mote-agent`, imported as `mote`. Use a local path or pin a Git
revision when depending on this source. This change does not publish the crate
to a registry. The Rust application can embed the library directly;
it does not need `mote-bridge` or either language SDK.

## Contracts and responsibilities

- **Explicit grants:** registering a callback does not grant it. Its name must
  also appear in manifest capabilities. Models cannot invent executable tools.
- **Input validation:** callbacks accept JSON objects and return strings. Each
  callback must validate its own domain-specific fields. Descriptions guide the
  model; they are not JSON Schema enforcement.
- **Completion is not correctness:** inspect events and independently validate
  outputs before publishing changes, charging a user, or accepting a report.
- **No automatic retry:** callbacks, writes and entire failed runs are not
  silently repeated. A failed/timeout run may already have side effects.
- **Boundaries:** callbacks run with the application's privileges. Neither the
  SDK nor bridge is an OS sandbox. Use isolated workers for untrusted workloads.
- **Timeouts:** SDKs bound the child process, but cannot forcibly interrupt a
  blocking host callback. Bound or isolate callbacks yourself.
- **Events:** tool callbacks are interactive; the runtime event list arrives at
  completion, not as a live progress stream. Logs/observations may contain data
  sent to the model; avoid exposing sensitive workspaces.
- **Versions:** the new SDKs are source-installable 0.1.0 packages. No PyPI/npm
  release or hosted API is implied. The wire contract is [protocol v1](BRIDGE_PROTOCOL.md).

## Verification

CI tests the Rust runtime, starter and both SDKs on Linux, Windows and macOS.
SDK integration tests launch the actual bridge against a local scripted HTTP
model server and execute actual language callbacks. These deterministic tests
require no model account and are not misrepresented as live-provider tests.
Opt-in live tests still require a reachable, funded/within-quota model endpoint.
