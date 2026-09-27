# TypeScript / Node SDK (source distribution)

Node 20+, ESM, no runtime npm dependencies. From the repository root:

```bash
cargo build --locked --release --bins
npm --prefix sdk/typescript ci
npm --prefix sdk/typescript run build
```

From your application directory install the built source package:

```bash
npm install /absolute/path/to/MOTE/sdk/typescript
```

```typescript
import { Client } from "@mote-agent/sdk";

const result = await new Client({binary: "/absolute/path/mote-bridge"}).run({
  manifest: {name: "demo", capabilities: ["uppercase"], models: [{
    provider: "openai-compatible", model: "your-model",
    endpoint: "https://your-provider/v1/chat/completions", auth_env: "MODEL_API_KEY",
  }]},
  task: 'Call uppercase with {"text":"hello"}, then complete.',
  tools: {uppercase: {
    description: "Input: text string. Returns uppercase text.",
    handler: async input => {
      if (typeof input.text !== "string") throw new Error("Expected text");
      return input.text.toUpperCase();
    },
  }},
});
if (!result.success) throw new Error("Agent run failed");
console.log(result.events);
```

`Client({binary="mote-bridge", timeoutMs=150000, cwd, env, args})` starts one
process per `run({manifest,task,tools})`. `env` overrides inherited variables;
relative workspaces use the child's `cwd`. `args` is for a trusted executable
launcher, not a shell command. `timeoutMs` is a positive finite timer duration.

`RunResult` contains `state`, `events`, `returncode`, and `success`. Failed or
cancelled runtime runs return a false result. Protocol, setup, transport and
timeout failures reject with `MoteError` (`code` identifies the category).
Success requires completed state, matching exit status, and full stdout EOF
without trailing malformed/duplicate data. Validate application outputs yourself.

Registration does not grant capability: each tool name also needs an explicit
manifest grant. Handlers accept JSON objects, validate their own fields and return
strings or Promises of strings. Exceptions/non-string values are sent as generic
errors, without exception messages or sensitive inputs. Calls/runs are not retried.

Frames are limited to 1 MiB including LF; retained stderr to 64 KiB. Events arrive
at the end, not as live progress. Observations are sent to the configured model;
avoid logging sensitive events. No registry release or model service is installed.

The timeout kills/reaps the direct bridge child and rejects even if an async
handler's Promise never resolves. It cannot stop a blocking JavaScript event
loop or forcibly cancel arbitrary callback work. Bound/isolate callbacks and
spawned descendants yourself. This is not an OS sandbox; failure can follow
side effects.

## Runnable example and checks

Set `MOTE_BRIDGE` to the absolute built bridge path (`.exe` on Windows),
`MOTE_MODEL_ENDPOINT` and `MOTE_MODEL`; optionally set `MODEL_API_KEY`.

```bash
node sdk/typescript/example.mjs
# From repository root, MOTE_BRIDGE must be set for the E2E test:
npm --prefix sdk/typescript test
npm --prefix sdk/typescript run pack:check
```

The example defaults to a local DMR-X endpoint; this does not prove a gateway is
running. Tests use real subprocesses/callbacks and a local scripted HTTP model,
not live inference. `pack:check` builds, packs, installs into a temporary consumer,
then validates public imports and TypeScript declarations. CI sets `MOTE_BRIDGE`
and runs the E2E test rather than skipping it. No npm publication is performed.
