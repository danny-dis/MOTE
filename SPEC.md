# MOTE Specification

**Minimal Orchestration & Task Execution**  
Version: 0.1 — Initial Architecture Specification

## 1. Purpose

MOTE defines a minimal, reusable runtime for embodying an agent specification and executing tasks through external models and capabilities.

The central design question is:

> What is the smallest runtime that can turn a declarative agent specification into a useful autonomous worker without forcing a large agent framework on every workload?

MOTE answers with a deliberately small kernel and a capability-oriented extension model.

## 2. Scope

MOTE MUST provide:

1. Agent specification loading.
2. Model-provider abstraction.
3. A task execution loop.
4. Structured action and observation handling.
5. Capability discovery and invocation.
6. Environment abstraction.
7. Structured lifecycle events.
8. Cancellation and bounded execution.
9. Clear separation between policy and model output.

MOTE SHOULD provide these without requiring a database, vector store, browser, MCP server, subagent system, or heavyweight UI.

## 3. Conceptual architecture

```text
+-----------------------+
| Agent Specification   |
| AGENT.md / manifest   |
+-----------+-----------+
            |
            v
+-----------------------+
|       MOTE CORE       |
|-----------------------|
| Spec Loader            |
| Context Builder        |
| Model Adapter          |
| Agent Loop             |
| Action Validator       |
| Event Emitter          |
+-----------+-----------+
            |
     +------+------+
     |             |
     v             v
+---------+   +----------------+
| Model   |   | Capabilities   |
| Adapter |   | FS / Shell /   |
+---------+   | Git / HTTP ... |
              +-------+--------+
                      |
                      v
              +---------------+
              | Environment   |
              | local/container|
              | microVM/remote |
              +---------------+
```

## 4. Agent specification

An agent specification is declarative. It defines what an agent is supposed to do; it does not grant permissions.

Recommended files:

```text
agent/
├── AGENT.md          # human-readable identity and behavior
├── manifest.yaml     # machine-readable metadata
├── policy.md         # optional human-readable constraints
└── capabilities/     # optional capability declarations
```

A minimal manifest may define:

```yaml
name: repository-maintainer
version: 0.1
model: auto
capabilities:
  - filesystem
  - shell
  - git
constraints:
  max_iterations: 40
```

## 5. Core execution loop

The normative minimal loop is:

```text
1. Load specification.
2. Resolve permitted capabilities.
3. Build initial context.
4. Ask the model for the next action.
5. Validate the proposed action.
6. Execute it in the environment.
7. Capture the observation.
8. Emit an event.
9. Continue until completion, failure, cancellation, or a configured limit.
```

The model MUST NOT directly bypass capability validation.

## 6. Actions

MOTE should use a small universal action protocol rather than hard-coding dozens of tools.

Conceptually:

```json
{
  "action": "execute",
  "capability": "shell",
  "input": {
    "command": "cargo test"
  }
}
```

The exact wire representation is implementation-defined in v0.1, but the semantic distinction between **intent**, **authorization**, **execution**, and **observation** MUST remain.

## 7. Capabilities

Capabilities are independently implemented units that expose operations to MOTE.

A capability SHOULD declare:

- name
- version
- operations
- input schema
- output schema
- required permissions
- resource limits
- side effects

Capabilities MUST NOT automatically receive permissions merely because an agent specification requests them.

## 8. Model abstraction

MOTE MUST NOT depend on a specific LLM vendor or model family.

The model adapter should support, where practical:

- request/response inference
- streaming output
- tool/action calls
- cancellation
- token/context metadata
- provider-specific configuration without leaking provider details into the core

Model selection can be delegated to an external router such as dmr-X.

## 9. Environment abstraction

Execution environments MUST be replaceable.

Initial targets:

- local process execution
- isolated container
- microVM
- remote execution endpoint

The core should interact with an environment through a narrow interface rather than assuming a local shell.

## 10. Context

MOTE should prefer deterministic, bounded context assembly over an always-on memory subsystem.

Possible context sources:

- agent specification
- task
- selected files
- repository map
- previous observations
- capability schemas
- externally supplied memory

Memory is an extension, not a kernel dependency.

## 11. Lifecycle

A MOTE run has explicit states:

```text
CREATED
  -> INITIALIZING
  -> RUNNING
  -> COMPLETING
  -> COMPLETED

RUNNING -> FAILED
RUNNING -> CANCELLED
RUNNING -> TIMED_OUT
```

State transitions SHOULD be observable through structured events.

## 12. Security model

Security MUST be enforced outside the model's natural-language instructions.

Required controls include:

- capability allowlists
- filesystem boundaries
- process/resource limits
- network policy
- secret isolation
- execution timeouts
- cancellation
- audit events

MOTE should support least-privilege execution as a first-class design principle.

## 13. Extensibility tiers

MOTE should grow through tiers rather than increasing the minimum runtime.

### Tier 0 — Kernel

Spec loader, model adapter, loop, action protocol, environment, events.

### Tier 1 — Basic worker

Filesystem, shell, git.

### Tier 2 — Software worker

Repository mapping, tests, patch management, GitHub.

### Tier 3 — Connected worker

HTTP, MCP, databases, browsers, external APIs.

### Tier 4 — Advanced agent

Memory, planning, subagents, long-running workflows, computer use.

The existence of Tier 4 MUST NOT force Tier 0 users to install Tier 4 components.

## 14. Interoperability

MOTE SHOULD expose multiple interfaces without making any one of them mandatory:

- CLI
- local IPC
- HTTP API
- streaming events
- library API
- MCP adapter
- agent-to-agent protocol adapters

## 15. Design invariants

The implementation MUST preserve these invariants:

1. The model is replaceable.
2. Capabilities are replaceable.
3. The environment is replaceable.
4. Policy is separate from model instructions.
5. The kernel remains usable without optional extensions.
6. An agent specification does not equal authorization.
7. A task can be executed without requiring a heavyweight coding-agent application.
8. MOTE remains independently deployable from GLUE and other ecosystem projects.

## 16. Relationship to GLUE

GLUE may use MOTE as an embodiment runtime for citizens, including agents discovered from GitHub repositories.

The dependency direction is:

```text
GLUE -> MOTE
```

not:

```text
MOTE -> GLUE
```

MOTE MUST remain useful outside GLUE.

## 17. Relationship to the wider stack

MOTE is intended to be a reusable low-level execution primitive.

```text
ATHENA        orchestration/governance
GLUE          agent citizenship/coordination
NOESIS        memory substrate
DANNY         digital self
DMR-X         model and worker routing
Ghost Factory software transformation
MOTE          lightweight agent embodiment/execution
```

These systems may integrate with MOTE through explicit interfaces rather than making MOTE responsible for their domains.

## 18. Success criteria

The first implementation is successful if it can:

- load an agent specification;
- accept a task;
- invoke a model;
- execute a permitted action;
- observe the result;
- iterate to completion;
- run inside an isolated environment;
- operate with only the capabilities actually needed;
- expose structured lifecycle events;
- remain useful without any heavyweight coding-agent framework.

## 19. Future work

Future specifications may define:

- a stable capability ABI
- capability packaging and discovery
- signed agent specifications
- deterministic replay
- checkpointing
- remote workers
- WASM capabilities
- microVM orchestration
- multi-agent protocols
- resource-aware scheduling
- policy engines
- repository intelligence

These should be added only when demonstrated necessary; minimality is a feature, not a temporary limitation.
