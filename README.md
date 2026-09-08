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
                 (Markdown)
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

MOTE treats this as a specification, not as executable code. The runtime resolves requested capabilities against an external policy and capability registry.

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

## Status

MOTE is currently a specification and engineering project. The first implementation should validate the minimal runtime before adding higher-level features.

## License

License to be selected before the first public release.
