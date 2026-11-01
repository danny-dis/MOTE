# MOTE bridge protocol v1

SDK authors can use this stdio contract to implement a client. For application
integration, start with [Building on MOTE](BUILDING_ON_MOTE.md). Manifest fields are
in [Configuration](CONFIGURATION.md). CLI JSONL uses a different format.

Build the separate `mote-bridge` executable with `cargo build --release --bins`.
It embeds the Rust runtime and leaves the `mote` CLI unchanged. Each process runs
one task. It provides no HTTP service, dynamic plug-in loader, or OS sandbox.
Python, TypeScript and Go SDKs launch it with an argument array, never a shell.

## Transport

Send UTF-8 JSON objects over stdin/stdout, one per LF-terminated line. A frame may
contain at most 1,048,576 bytes, including its newline. Blank, malformed, oversized,
unterminated, and unexpected frames are errors. Reserve stdout for the protocol
and stderr for diagnostics. Python and TypeScript discard stderr through the OS.
The Go SDK continuously drains and discards it without retaining diagnostics.
No bundled SDK returns stderr to the caller.

The host starts the run with:

```json
{"type":"run","protocol":1,"manifest":{"name":"example","workspace":"/absolute/workspace","capabilities":["uppercase"],"models":[{"provider":"openai-compatible","model":"your-model","endpoint":"http://localhost:47113/v1/chat/completions"}]},"task":"Uppercase hello using uppercase with input {\"text\":\"hello\"}.","tools":[{"name":"uppercase","description":"Input object: text (string). Returns uppercase text."}]}
```

`tools` defaults to empty; the other run fields are required. `manifest` accepts
the normal manifest fields and defaults, and rejects unknown fields. Relative
workspaces resolve from the child process's working directory, not a manifest
file. SDKs expose `cwd` and do not implicitly change or grant capabilities.
Tool descriptions are appended to the model task as host-tool contracts.
Registering a tool does not grant it permission: its name must also appear in
`capabilities`. Keep credentials in the variables named by `auth_env`, not in tool metadata.

For a granted custom-tool call, the bridge sends:

```json
{"type":"tool_call","id":1,"name":"uppercase","input":{"text":"hello"}}
```

IDs are positive sequential integers, starting at 1 for each process. Only one
callback can be outstanding. The host must return the matching ID and exactly
one string-valued `output` or `error` field:

```json
{"type":"tool_result","id":1,"output":"HELLO"}
```

```json
{"type":"tool_result","id":1,"error":"handler failed"}
```

Unknown tools, mismatched IDs, both output/error, missing result fields, non-string
results, and non-object input fail closed. When a callback raises an exception,
SDKs send a generic error without exception text, stack traces, or sensitive inputs.
Neither the bridge nor the SDK retries callbacks.

After the runtime finishes, the bridge sends:

```json
{"type":"run_result","state":"completed","events":[]}
```

`state` is `completed`, `failed`, or `cancelled`. `events` holds the runtime event
objects and arrives at the end of the run, not as a live stream. The process exits
0 for completed and 1 for failed/cancelled. Protocol/setup failures emit
`{"type":"error","message":"..."}` and exit 2. Accept a result as successful only
when its exit status agrees. If the final events exceed the frame limit, the bridge
fails explicitly without truncating them. Side effects may already have occurred;
do not blindly retry a failed run.

## Timeouts and trust

The bridge waits up to 30 seconds for startup input. Callback waits use the
manifest run deadline. SDKs also enforce a configurable child-process wall-clock
timeout, defaulting to 150 seconds. On timeout or protocol error, they kill and
reap the child. Missing, truncated, and duplicate terminal results are rejected.

Python synchronous callbacks, Go handlers and JavaScript callbacks execute in the host
application. SDK process timeouts cannot forcibly stop arbitrary blocking host
code. Bound callbacks yourself or isolate them in separate workers. Killing the
bridge does not kill its entire process tree; tools that spawn children need their
own lifecycle and isolation policy. SDKs cannot promise a hard callback deadline
when a callback blocks the caller or event loop.

The model endpoint receives observations, and events may contain sensitive data.
Custom callbacks have host privileges. Neither the bridge nor capability names
provide an OS sandbox. Untrusted workloads need external isolation.
