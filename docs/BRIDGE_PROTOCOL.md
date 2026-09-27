# MOTE bridge protocol v1

This is the implemented stdio contract for SDK authors. Application developers
should start with [Building on MOTE](BUILDING_ON_MOTE.md); manifest fields are
documented in [Configuration](CONFIGURATION.md). CLI JSONL is a different format.

`mote-bridge` is a separate executable built with `cargo build --release --bins`.
It embeds the existing runtime without changing the `mote` CLI. One process runs
one task. No HTTP service, dynamic plug-in loader or OS sandbox is introduced.
Python and TypeScript SDKs launch this binary using an argument array, never a shell.

## Transport

UTF-8 JSON objects, one per LF-terminated line on stdin/stdout. Each frame,
including its newline, is at most 1,048,576 bytes. Blank, malformed, oversized,
unterminated or unexpected frames are errors. stdout is exclusively protocol;
stderr is diagnostics. SDKs bound retained stderr to 65,536 bytes.

The host first sends:

```json
{"type":"run","protocol":1,"manifest":{"name":"example","workspace":"/absolute/workspace","capabilities":["uppercase"],"models":[{"provider":"openai-compatible","model":"your-model","endpoint":"http://localhost:47113/v1/chat/completions"}]},"task":"Uppercase hello using uppercase with input {\"text\":\"hello\"}.","tools":[{"name":"uppercase","description":"Input object: text (string). Returns uppercase text."}]}
```

`tools` defaults to empty. Other run fields are required. `manifest` uses the
normal manifest fields/defaults and rejects unknown fields. A relative workspace
is relative to the child process working directory (not a manifest file).
SDKs expose `cwd` and never implicitly change or grant capabilities.
Tool descriptions are appended to the model task as available host-tool contracts.
Registering a tool does NOT grant it: its name must also be in `capabilities`.
Credentials remain in named environment variables (`auth_env`), not tool metadata.

When the runtime invokes a granted custom tool, the bridge sends:

```json
{"type":"tool_call","id":1,"name":"uppercase","input":{"text":"hello"}}
```

IDs are positive, sequential integers starting at 1, scoped to this process.
Only one callback is outstanding. The host replies with the matching ID and
exactly one string-valued `output` or `error` field:

```json
{"type":"tool_result","id":1,"output":"HELLO"}
```

```json
{"type":"tool_result","id":1,"error":"handler failed"}
```

Unknown tools, mismatched IDs, both output/error, neither field, non-string
results and non-object input fail closed. SDKs send a generic error for callback
exceptions, without sending exception text/stack traces or sensitive inputs.
No callback is retried by the bridge or SDK.

After the runtime finishes, the bridge emits:

```json
{"type":"run_result","state":"completed","events":[]}
```

`state` is `completed`, `failed`, or `cancelled`. `events` contains the existing
runtime event objects. These events are returned at the end, not streamed live.
Exit code is 0 for completed and 1 for failed/cancelled. Protocol/setup failures
emit `{"type":"error","message":"..."}` and exit 2. A result without a matching
exit status is not success. If the final events cannot fit a frame, the bridge
fails explicitly instead of silently truncating them. Side effects may already
have occurred; callers must not blindly retry a failed run.

## Timeouts and trust

Startup input has a 30-second deadline. Callback waits are bounded by the
manifest run deadline. SDKs additionally enforce a configurable child-process
wall-clock timeout (default 150 seconds), kill and reap the child on timeout or
protocol errors, and reject missing/truncated/duplicate terminal results.
Python synchronous callbacks and JavaScript callbacks run in the host application;
SDK process timeouts cannot forcibly stop arbitrary blocking host code. Bound
callbacks yourself or isolate them in separate workers. The SDK kills the bridge
process, not an entire process tree; tools that spawn children need their own
lifecycle/isolation policy. SDKs must not claim a
hard deadline for a callback that blocks their caller/event loop.

The model endpoint receives observations. Events may contain sensitive data.
Custom callbacks run with host privileges. Neither the bridge nor capability
names are an OS sandbox; untrusted workloads require external isolation.
