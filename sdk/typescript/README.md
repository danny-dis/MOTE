# MOTE TypeScript / Node.js SDK

The MOTE Node SDK lets a Node application host tools and run MOTE through the
versioned stdio `mote-bridge` executable. Its package metadata marks it
private, and it is not published to npm. It has no runtime npm dependencies.

Requirements: Node.js 20 or newer (`engines.node = ">=20"`). A Rust toolchain
is not required when you use a downloaded bridge executable. The bridge is
available in the [GitHub Releases](https://github.com/danny-dis/MOTE/releases)
archives; this repository is private, so GitHub repository access is required.
Release binaries are unsigned/not notarized. Verify archive checksums and
expect an OS warning where applicable. See [QUICKSTART](../../docs/QUICKSTART.md)
for release assets and extraction instructions.

## Quick start

Run these commands from the repository root. Use the downloaded
`mote-bridge` path in place of the source-built path; append `.exe` on Windows.

```bash
# Source checkout option; requires stable Rust.
cargo build --locked --release --bin mote-bridge

npm --prefix sdk/typescript ci
npm --prefix sdk/typescript run build
```

From an application directory, install the built local package:

```bash
npm install /absolute/path/to/MOTE/sdk/typescript
```

Set `MOTE_BRIDGE` to the downloaded or built executable,
`MOTE_MODEL_ENDPOINT` to a reachable OpenAI-compatible chat-completions
endpoint, and `MOTE_MODEL` to the model name. For authentication, set
`MODEL_API_KEY` and use `auth_env: "MODEL_API_KEY"` in the manifest. Do not
put a credential in YAML or source. The SDK does not install or start a
provider, gateway or model service.

From the repository root, after building the SDK:

```bash
node sdk/typescript/example.mjs
```

The example defaults to a local DMR-X-style endpoint and model `auto`; change
the environment for your provider. It requires a live, reachable model
endpoint and independently checks callback use and the observed `HELLO` value.

See [CONFIGURATION](../../docs/CONFIGURATION.md) for manifest fields, model
authentication and limits. See [BUILDING_ON_MOTE](../../docs/BUILDING_ON_MOTE.md)
and [QUICKSTART](../../docs/QUICKSTART.md) for the broader launch workflow.

## API

```typescript
import { Client } from "@mote-agent/sdk";

const result = await new Client({
  binary: "/absolute/path/to/mote-bridge",
  timeoutMs: 150_000,
}).run({
  manifest: {
    name: "demo",
    capabilities: ["uppercase"],
    models: [{
      provider: "openai-compatible",
      model: "your-model",
      endpoint: "https://your-provider.example/v1/chat/completions",
      auth_env: "MODEL_API_KEY",
    }],
  },
  task: 'Call uppercase with {"text":"hello"}, then complete.',
  tools: {
    uppercase: {
      description: "Input: text string. Returns uppercase text.",
      handler: async (input) => {
        if (typeof input.text !== "string") throw new Error("Expected text");
        return input.text.toUpperCase();
      },
    },
  },
});
if (!result.success) throw new Error("Agent run failed");
console.log(result.events);
```

`new Client({ binary, args, timeoutMs, cwd, env })` starts one trusted
executable per run without a shell. `args` is for an application-owned
launcher. `timeoutMs` must be positive, finite and no greater than the Node
timer limit; the default is 150,000 ms. `env` overrides inherited variables
and `cwd` controls the child process working directory.

`run({ manifest, task, tools })` resolves to `RunResult` with `state`,
`events`, `returncode` and `success`. Failed or cancelled runtime work returns
`success: false`. Setup, protocol, transport and deadline failures reject with
`MoteError`; inspect its `code` (`SETUP`, `PROTOCOL`, `TRANSPORT` or `TIMEOUT`).
Completion still requires application-level validation.

Tool handlers receive JSON objects and must return strings or promises of
strings. Validate their own input. Exceptions and non-string values become a
generic tool error without exception text or sensitive input. Registration does
not grant permission: each name also needs an explicit manifest capability.
Calls and complete runs are not automatically retried, and failures may have
side effects.

Frames are limited to 1 MiB including LF; retained stderr is limited to 64 KiB.
Events and observations arrive only in the final result; there is no live-event
or progress-stream API. Avoid logging sensitive events. The timeout kills and
reaps the direct bridge child even if a handler promise never resolves, but it
cannot stop blocking JavaScript work or forcibly cancel arbitrary descendants.
Neither the SDK nor bridge is an OS sandbox; callbacks run with host
application privileges.

## Tests and package check

From the repository root, with `MOTE_BRIDGE` set to a bridge executable:

```bash
npm --prefix sdk/typescript test
npm --prefix sdk/typescript run pack:check
```

These use real subprocesses and callbacks with a local scripted HTTP model,
not live inference. `pack:check` validates the built package, public import and
declarations in a temporary consumer. No npm publication is performed.
