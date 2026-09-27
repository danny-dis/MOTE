# MOTE

**Minimal Orchestration & Task Execution**

MOTE is a lightweight, model-agnostic Rust library and CLI for building trusted-workspace agents. It keeps the execution loop small and leaves scheduling, memory, user interfaces, and workload isolation to the application that embeds it. It is intentionally independent of GLUE, ATHENA, Ghost Factory, DANNY, and any particular model provider.

## Why MOTE?

Most modern coding agents bundle a large amount of functionality into one application: planning, memory, browser automation, MCP, repository indexing, subagents, UI, provider integrations, and orchestration.

MOTE takes the opposite approach:

> **Keep the agent kernel tiny. Make capabilities composable.**

A MOTE instance can start as a minimal worker and acquire additional capabilities only when a task requires them.

## Core model

```text
             Agent Specification
                 (YAML)
                      |
                      v
              +---------------+
              |  MOTE Runtime |
              |---------------|
              | spec loader   |
              | agent loop    |
              | action parser |
              | observation   |
              | event stream  |
              +-------+-------+
                      |
          +-----------+-----------+
          |           |           |
       Model       Runtime     Capability
       Adapter    Environment    Layer
          |           |           |
        LLM/API   shell/VM     git/fs/http/...
```

The essential loop is:

```text
LOAD -> THINK -> ACT -> OBSERVE -> REPEAT -> COMPLETE
```

## Design principles

- **Tiny core** — minimize mandatory code and dependencies.
- **Rust-first** — low overhead, fast startup, strong process and concurrency primitives, single-binary distribution.
- **Model agnostic** — MOTE does not own the model.
- **Capability based** — tools are dynamically attached rather than permanently embedded.
- **Specification driven** — Markdown/YAML can define an agent's identity, goals, constraints, and capabilities.
- **Environment agnostic** — run locally, in containers, microVMs, remote workers, or other execution environments.
- **Composable** — use MOTE alone or as a worker primitive inside larger systems.
- **Observable** — every important action should be representable as structured events.
- **Scoped by default** — no capability is granted implicitly; process isolation and host authorization remain the embedding application's responsibility.

## Agent specification

A repository can contain an `AGENT.md` describing the citizen that MOTE should embody.

Example:

```markdown
# Repository Maintainer

## Identity
Maintain this repository and keep changes aligned with its engineering standards.

## Goals
- Fix requested defects.
- Keep tests passing.
- Minimize unnecessary changes.

## Capabilities
- filesystem
- shell
- git

## Constraints
- Do not modify protected configuration.
- Do not publish credentials.
```

The current CLI reads YAML manifests. `AGENT.md` and external authorization/policy integration remain design goals, not implemented security controls.

## Downloads

Download prebuilt Windows x64, Linux x64, and macOS Apple Silicon binaries from
[GitHub Releases](https://github.com/danny-dis/MOTE/releases/latest). Each archive
contains the CLI, SDK bridge, example configuration, quick-start, and licenses.
See [binary quick-start](docs/QUICKSTART.md) for requirements and checksum checks.
Repository access is required while MOTE is private.

Releases are automatic on matching version tags; see [release procedure](docs/RELEASING.md).

## Building on MOTE

Start with the [application-building guide](docs/BUILDING_ON_MOTE.md):

- **Rust:** [application starter](starters/rust-agent/) with a real model adapter, custom tool, and independent output validation.
- **Python:** [source-installable SDK](sdk/python/) with Python callbacks.
- **TypeScript/Node:** [source-installable SDK](sdk/typescript/) with synchronous or asynchronous callbacks.
- **Cross-language contract:** [`mote-bridge` protocol v1](docs/BRIDGE_PROTOCOL.md), a separate, one-run stdio binary. No server required; registration never grants permission.

Build both binaries with `cargo build --locked --release --bins`. SDKs do not bundle or download the executable; these packages are not automatically published to PyPI/npm. Existing `mote` CLI usage remains unchanged.

MOTE exposes a Rust library: implement the `Model` trait to supply actions, configure a `Manifest`, then run `Runtime` in a `WorkspaceFs`. The Cargo package is named `mote-agent` (the unrelated `mote` package on crates.io is **not** this project); its Rust library import remains `mote`. The runnable examples work without a model account:

```bash
cargo run --locked --example basic_agent
cargo run --locked --example custom_tool
```

The [custom-tool example](examples/custom_tool.rs) registers a host-owned Rust handler with `Runtime::with_tools` and grants it by name in the manifest. It accepts a JSON-object input and returns a bounded text observation; unknown and ungranted names are rejected. For HTTP models, the host must describe registered tools in the task; action-line syntax is `custom: name | {"key":"value"}`. Registration does not itself grant permission. A YAML file alone cannot load executable Rust handlers. There is no dynamic plug-in ABI or tool sandbox; handlers run in the embedding process and must bound their own blocking work.

## Extensibility

MOTE should remain small even when the surrounding system becomes large.

Optional capabilities may include:

- filesystem
- shell
- git
- GitHub
- HTTP
- browser
- databases
- MCP
- memory
- repository maps/indexing
- planning
- subagents
- computer use
- remote execution
- GLUE citizen integration

None are required by the core runtime.

## Ecosystem role

MOTE is an independent primitive.

```text
ATHENA       -> can schedule MOTE workers
GLUE         -> can embody GitHub citizens through MOTE
Ghost Factory -> can use MOTE for lightweight software tasks
DANNY        -> can request MOTE for inexpensive delegated work
DMR-X        -> can select the model used by MOTE
MOTE         -> remains independently deployable
```

This separation prevents the lightweight runtime from becoming another monolithic orchestration platform.

## Non-goals

MOTE is not intended to initially be:

- a full IDE
- a replacement for Claude Code, OpenCode, OpenHands, or other heavyweight coding environments
- a mandatory multi-agent framework
- a memory database
- a model provider
- an enterprise control plane

Those capabilities may be integrated externally when useful.

## Status and quick start

MOTE 0.13.0 (`mote-agent` Cargo package) contains a Rust library (`src/lib.rs`, imported as `mote`) and CLI (`src/main.rs`). A YAML manifest selects the model chain, workspace, limits, and explicit capabilities. The built-in actions are `shell`, `read_file`, `write_file`, `list_dir`, `git`, `decision`, and `complete`; Rust embedders can additionally register host-owned custom handlers with explicit manifest grants. A task must use at least one permitted capability before completion. `max_per_tool` defaults to 5 physical calls per capability (including retries); transient read/list failures may retry up to three times with backoff. Shell, Git, and writes are never auto-retried. A successful explicit write to the configured `output_file` ends the run; a summary cannot claim success by reusing an old report.

```bash
cargo build --release
# Configure a reachable OpenAI-compatible endpoint in specs/dmr-x-local.yaml first.
./target/release/mote --jsonl specs/dmr-x-local.yaml "Describe the workspace"
```

`--jsonl` prints structured lifecycle events after the run, not an incremental live stream. Model credentials are read from the environment variable named by a manifest's `auth_env`, never stored in example YAML. See `specs/` for examples and `docs/STATUS.md` for implementation status. For a local endpoint without credentials, `specs/keyless-local.yaml` is a parser-tested example; a working server is still required. The Pollinations example sends workspace observations to a third-party endpoint and has not been live-verified. On decision-provider failure, runtime fallback is available only for a choice keyed `deny`, `reject`, `block`, or `escalate` that appears in the request criteria; score and Noul requests fail rather than receiving an invented zero. The caller must treat those keys as denying actions, not aliases for approval.

**Security boundary:** capabilities are deny-by-default and built-in file actions check workspace paths, but shell and Git launch normal OS processes with the invoking user's permissions. An executable allowlist and timeout are **not an OS sandbox**: permitted programs and their arguments may access files, network, subprocesses, and secrets outside the workspace. Use an externally isolated account/container/VM for untrusted model output or repositories; do not treat the manifest as authorization on its own. Cancellation and time limits are best-effort rather than a guarantee against child processes or blocked network calls. JSONL events include observations and most action arguments (write-file content is omitted), and observations are sent to the configured model endpoint; do not point MOTE at sensitive workspaces or share its event logs without review. See the [deployment security checklist](SECURITY.md).

## Production deployment boundary

MOTE is a **library/CLI kernel**, not a hosted agent service or security sandbox. A supported deployment uses a trusted operator-controlled manifest and runs each agent under a dedicated low-privilege identity or external container/VM with constrained filesystem, environment, and network access. The host must decide which manifests, capabilities, model endpoints, and Rust tool handlers are allowed; loading a user-submitted manifest does **not** authorize its requests. Use the [security checklist](SECURITY.md) before unattended runs. Multi-tenant execution of arbitrary user agents on a shared host is outside this release scope.

The embedding application owns scheduling, persistence, user authentication, credential storage, monitoring, and restart/retry policy. MOTE provides bounded individual runs and events, not a durable job queue. Exercise the real model endpoint and the target OS/environment before promoting an agent to production; the keyless example and CI do not test a provider's availability. This pre-1.0 Rust API may change in a minor release; pin a compatible crate version and test upgrades.

## Verification and action protocol

Run `cargo test --locked --all-targets`, `cargo test --locked --release --all-targets`,
`cargo fmt --check`, and `cargo clippy --locked --all-targets -- -D warnings` for deterministic checks.
For opt-in **real model** verification, build the release binary and run:

```bash
python scripts/live_smoke.py --binary target/release/mote --output smoke-results
# Windows: --binary target/release/mote.exe
```

The stdlib-only smoke runner uses synthetic temporary workspaces through a local
DMR-X endpoint by default. It checks CSV aggregation, exact whitespace/Unicode
copying, and configuration edits, three times each. It independently validates
all output files, retains every attempt in a new evidence directory, and exits
nonzero if any task fails. `--endpoint`, `--model`, and `--auth-env` select another
provider; the endpoint receives the synthetic inputs. It is not run by CI and
makes no claim that all providers or workloads are reliable.

HTTP models are prompted for one JSON action, for example
`{"action":"write_file","path":"report.txt","content":"    indented\n"}`
(with the newline escaped in the JSON string). This uses the existing `Action`
serialization format and preserves content exactly, including empty files.
Legacy `write_file: path | content` remains accepted: at most one ASCII space
immediately after `|` is a separator; all subsequent whitespace is file content.
Other legacy action lines remain supported. Responses explicitly marked
truncated, filtered, or otherwise unfinished are rejected before execution;
compatible endpoints that omit completion metadata remain supported.

Each model turn receives granted capability names, the per-tool budget, the
output path, and labeled results of completed actions. A completion summary
written to `output_file` uses the same permission, budget and event path as an
explicit `write_file`. `Completed` means execution finished, not that an arbitrary
artifact is semantically correct; the embedding application must still validate
its own acceptance criteria.

## License

MIT. See [LICENSE](LICENSE). You may build and distribute agents on top of MOTE under the terms of that license. Dependencies retain their own licenses; see [third-party notices](THIRD_PARTY_NOTICES.md) when distributing a bundled binary.
