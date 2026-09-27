# Configuration reference

The CLI reads a YAML manifest. Field names and defaults come from
[`src/config.rs`](../src/config.rs); unknown fields are rejected. The application
must control the manifest: accepting arbitrary caller-supplied grants is not authorization.

## Minimal read-only manifest

```yaml
name: document-worker
workspace: ./workspace
capabilities: [list_dir, read_file]
max_iterations: 8
models:
  - provider: openai-compatible
    model: your-model-name
    endpoint: http://127.0.0.1:8080/v1/chat/completions
    auth_env: MOTE_API_KEY
```

Replace the endpoint and model with your provider's actual values. `auth_env` is the
**name of an environment variable**, never the key itself. If your local endpoint
does not require authentication, omit `auth_env`. MOTE does not expand `${VARIABLE}`
inside YAML or start a model server. The example endpoint is not a hosted MOTE service.

For the CLI, relative `workspace` paths are resolved relative to the manifest's
location. For SDK manifests, they are resolved relative to the bridge process's
working directory (`cwd`). An absolute workspace removes that ambiguity.

## Top-level fields

| Field | Default | Meaning |
|---|---|---|
| `name` | Required | Agent/run configuration name |
| `workspace` | `.` | Root for built-in file operations |
| `capabilities` | `[]` | Explicitly granted tool names; empty means no tools |
| `max_iterations` | `20` | Maximum execution-loop iterations |
| `max_per_tool` | `5` | Physical invocation budget per capability/tool name |
| `max_runtime_seconds` | `120` | Elapsed runtime budget; not hard OS isolation |
| `command_timeout_seconds` | `30` | Timeout for direct shell/Git commands |
| `max_output_bytes` | `65536` | Observation bound; permitted range `1`–`1048576` |
| `shell` | Empty allowlist, `unsafe_shell: false` | Shell/Git execution settings |
| `output_file` | None | Optional workspace-relative final report path |
| `models` | `[]` | Ordered HTTP model chain; CLI/bridge require at least one |
| `decision` | None | Optional typed decision provider configuration |

A programmatically embedded Rust application can supply its own `Model`, rather
than use the HTTP model chain. Optional fields do not imply that every combination
is valid: runtime setup checks grants, paths, and observation limits.

## Models and authentication

Each model entry requires `provider`, `model`, and `endpoint`. Optional fields:

- `auth_env`: environment-variable name; if present but unset, inference fails.
- `timeout_seconds`: HTTP timeout, default `30`.

Supported adapter names:

| `provider` | Request format |
|---|---|
| `openai` or `openai-compatible` | OpenAI-style chat completions; Bearer authentication |
| `dmr-x` or `dmrx` | Same chat-completions format, for a configured DMR-X gateway |
| `google` or `gemini` | Gemini generate-content format; `x-goog-api-key` |
| `anthropic` | Anthropic messages format; `x-api-key` |

Use the full endpoint route expected by that adapter. For Gemini, the generate-content
endpoint includes the selected model in its URL; merely changing the `model` label
does not rewrite the URL. Native adapter support is not a promise that every provider,
model, or endpoint version is compatible. Credentials, billing, quotas, and network
access are managed outside MOTE.

Models are tried in order if inference fails. Empty responses receive one retry
before fallback. Explicitly unfinished provider responses are rejected instead of
executing potentially truncated actions. Tool side effects and whole runs are not
automatically retried because they could duplicate writes or external operations.

## Capabilities

Built-in names are `list_dir`, `read_file`, `write_file`, `shell`, `git`, and optional
`decision`. `complete` ends a run; if it writes `output_file`, that write requires
`write_file` and consumes the normal write budget. A successful explicit write to
`output_file` can also complete the run. Existing report content is not proof of a
fresh successful output.

Custom tool names must be both registered by the host and listed in `capabilities`.
YAML cannot import Python/JavaScript functions or load Rust plug-ins. See the
[application guide](BUILDING_ON_MOTE.md).

Built-in file paths cannot be absolute, traverse through `..`, or resolve outside
the workspace at validation time. This does not protect against all filesystem races,
child processes, or trusted handlers accessing the host. See [Security](../SECURITY.md).

## Shell and Git

```yaml
capabilities: [shell]
shell:
  allowed_programs: [whoami]
  unsafe_shell: false
command_timeout_seconds: 10
```

This is an explicit opt-in, not part of the read-only quick start. Default shell
execution parses an executable/arguments and rejects shell operators. Allowlisting
an interpreter gives it substantial host access. `unsafe_shell: true` deliberately
uses a shell and should not be enabled for untrusted workloads.

The built-in Git capability uses a restricted command set by default and also
requires the shell authorization expected by runtime validation. Use narrowly scoped
trusted configurations rather than copying broad grants from historical examples.

## Optional decisions

The `decision` object has required `provider` and `endpoint`, optional `model`
(default `jev-latest`), `auth_env`, `fallback`, and `timeout_seconds` (default `30`).
It is a specialized typed-choice adapter, not a general approval service. Refer to
[`src/decision.rs`](../src/decision.rs) and [the example](../specs/decision-gate.yaml)
for its request contract. The host must choose a valid fail-closed fallback; MOTE
must not invent an allowed action when a decision provider fails.

## CLI and events

```sh
mote --version
mote --help
mote agent.yaml "Your task"
mote --jsonl agent.yaml "Your task"
```

Use `./mote` or `.\mote.exe` if the executable is not on `PATH`.

- Exit `0`: runtime completed.
- Exit `1`: runtime failed or was cancelled.
- Exit `2`: CLI/setup error, such as an invalid manifest.

Normal output summarizes the state and event count. `--jsonl` prints one lifecycle
event per line **after** the run. Event types include `RunStarted`, `ActionProposed`,
`ActionExecuted`, `ObservationReceived`, `RunCompleted`, and `Error`. Observations may
include sensitive file contents or command output and may be sent to the model.

A zero exit code means execution completed, not that an answer is true or a generated
file meets your business requirements. Validate outputs independently.
