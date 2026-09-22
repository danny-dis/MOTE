# MOTE

**Minimal Orchestration & Task Execution**

MOTE is a lightweight, model-agnostic agent runtime designed to embody an agent specification and execute tasks with the smallest practical control loop. It is intentionally independent of GLUE, ATHENA, Ghost Factory, DANNY, and any particular model provider.

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
- **Safe by default** — permissions and execution boundaries belong to the environment/capability layer, not to model instructions alone.

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

MOTE 0.12.0 contains a Rust library (`src/lib.rs`) and CLI (`src/main.rs`). A YAML manifest selects the model provider, workspace, and allowed capabilities. The current actions are `shell`, `read_file`, `write_file`, `list_dir`, `git`, `decision`, and `complete`. A task must use at least one permitted capability before completion.

```bash
cargo build --release
# Configure a reachable OpenAI-compatible endpoint in specs/dmr-x-local.yaml first.
./target/release/mote --jsonl specs/dmr-x-local.yaml "Describe the workspace"
```

`--jsonl` emits structured lifecycle events. Model credentials are read from the environment variable named by a manifest's `auth_env`, never stored in example YAML. See `specs/` for examples and `docs/STATUS.md` for implementation status. On decision-provider failure, runtime fallback is available only for a choice keyed `deny`, `reject`, `block`, or `escalate` that appears in the request criteria; score and Noul requests fail rather than receiving an invented zero. The caller must treat those keys as denying actions, not aliases for approval.

**Security boundary:** capabilities are deny-by-default and built-in file actions check workspace paths, but shell and Git launch normal OS processes with the invoking user's permissions. An executable allowlist and timeout are **not an OS sandbox**: permitted programs and their arguments may access files, network, subprocesses, and secrets outside the workspace. Use an externally isolated account/container/VM for untrusted model output or repositories; do not treat the manifest as authorization on its own. Cancellation and time limits are best-effort rather than a guarantee against child processes or blocked network calls. JSONL events include raw actions and observations, and observations are sent to the configured model endpoint; do not point MOTE at sensitive workspaces or share its event logs without review.

## License

No license has been selected. All rights are reserved unless the owner grants permission separately.
