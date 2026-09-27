# MOTE Specification

> **Historical design proposal — not current product documentation.** This specification preserves a proposed runtime design and does not establish live health, current behavior, or shipped features. For current MOTE documentation, see [STATUS](../STATUS.md), [BUILDING_ON_MOTE](../BUILDING_ON_MOTE.md), and [QUICKSTART](../QUICKSTART.md).

**Conceptual design, not the implemented manifest or an OS security boundary.** The runnable YAML schema is `src/config.rs`; [README.md](../../README.md) and [docs/STATUS.md](../STATUS.md) describe shipped behavior.

**Minimal Orchestration & Task Execution**
Version: 0.2 — Minimal Agent Execution Kernel

## 1. Purpose

MOTE defines a minimal, reusable **agent execution kernel** for embodying an agent specification and executing tasks through external models and composable capabilities.

The central design question is:

> What is the smallest runtime that can turn a declarative agent specification into a useful autonomous worker without forcing a large agent framework on every workload?

MOTE answers with a deliberately small kernel and a capability-oriented extension model.

MOTE is **not a coding-agent application**. It is a low-level execution primitive that can be embedded into many kinds of workers: coding agents, benchmark agents, research workers, automation workers, CI workers, ATHENA workers, Ghost Factory workers, and others.

The core thesis is:

> **Do not make the agent smarter by making the core bigger. Make the core smaller and compose exactly the capabilities the workload needs.**

MOTE supplies the minimum execution machinery around a model; it does not attempt to provide every feature that a complete agent application might eventually need.

## 2. Scope

MOTE MUST provide:

1. Agent specification loading.
2. Model-provider abstraction.
3. A deterministic task execution loop.
4. Structured action and observation handling.
5. Capability discovery and invocation.
6. Environment abstraction.
7. Structured lifecycle events.
8. Cancellation and bounded execution.
9. Clear separation between policy and model output.
10. Resource/budget enforcement sufficient to bound a run.

MOTE SHOULD provide these without requiring a database, vector store, browser, MCP server, subagent system, persistent memory system, or heavyweight UI.

MOTE MUST NOT require coding-specific functionality for non-coding workloads.

## 3. Conceptual architecture

```text
+-----------------------+
| Agent Specification   |
| AGENT.md / manifest   |
+-----------+-----------+
            |
            v
+-------------------------------+
|          MOTE CORE            |
|-------------------------------|
| Spec Loader                   |
| Context Builder               |
| Model Interface               |
| Agent Execution Loop          |
| Action / Policy Boundary      |
| State + Budget Enforcement    |
| Event Stream                  |
| Cancellation                  |
+---------------+---------------+
                |
        +-------+-------+
        |               |
        v               v
+---------------+  +----------------+
| Model / dmr-X |  | Capabilities   |
| Adapter       |  | FS / Shell /   |
+---------------+  | Git / HTTP ... |
                   +-------+--------+
                           |
                           v
                   +---------------+
                   | Environment   |
                   | local/container|
                   | microVM/remote |
                   +---------------+
```

The model supplies reasoning or proposed intent. MOTE supplies deterministic execution mechanics: authorization, capability dispatch, environment execution, observation, state transitions, limits, and events.

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

A runnable manifest with the current CLI uses explicit capability grants and a model chain (see `specs/`):

```yaml
name: repository-maintainer
workspace: .
capabilities: [read_file, list_dir]
max_iterations: 20
max_per_tool: 5
models:
  - provider: dmr-x
    model: auto
    endpoint: http://127.0.0.1:47113/v1/chat/completions
```

This permits only built-in workspace reads/listings; the configured endpoint must be running. Shell/Git processes need an explicitly allowlisted executable and an externally isolated environment for untrusted work. Declarative goals do not grant permissions or OS isolation.

## 5. Core execution model

The normative minimal loop is:

```text
OBSERVE
   ↓
PLAN / REASON
   ↓
PROPOSE ACTION
   ↓
AUTHORIZE / VALIDATE
   ↓
ACT
   ↓
OBSERVE RESULT
   ↓
VERIFY / TERMINATE?
   └──────────────→ repeat
```

A concrete run performs:

1. Load specification.
2. Resolve permitted capabilities.
3. Build initial context.
4. Ask the model for the next intent/action.
5. Validate the proposed action against policy, capability schema, and budgets.
6. Execute the action in the environment.
7. Capture the observation.
8. Emit a structured event.
9. Continue until completion, failure, cancellation, timeout, or a configured limit.

The model MUST NOT directly bypass capability validation or environment controls.

MOTE does not prescribe a particular planning algorithm. A simple model/tool loop is valid; more advanced planning can be supplied as an extension when the workload actually requires it.

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

The exact wire representation is implementation-defined in v0.2, but the semantic distinction between **intent**, **authorization**, **execution**, and **observation** MUST remain.

An action should carry enough structured information for deterministic validation and auditing without forcing provider-specific model formats into the kernel.

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

Examples include:

```text
filesystem
shell
http
model
benchmark-dataset
evaluator
results-store
git
github
browser
lsp
mcp
```

The list is illustrative, not a mandatory built-in catalog. A workload should receive only the capabilities it needs.

## 8. Model abstraction

MOTE MUST NOT depend on a specific LLM vendor, model family, or inference runtime.

The model interface should support, where practical:

- request/response inference
- streaming output
- structured action/tool calls
- cancellation
- token/context metadata
- provider-specific configuration without leaking provider details into the core

Model selection can be delegated to an external router such as dmr-X.

MOTE therefore sits **around** model reasoning rather than attempting to become another model framework. The model provides intelligence; MOTE provides the minimum machinery required to turn that intelligence into bounded, observable execution.

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
- benchmark dataset items

Memory is an extension, not a kernel dependency.

Context construction is part of execution, but heavyweight retrieval, semantic memory, long-term persistence, and knowledge graphs belong outside the minimum kernel unless a workload explicitly needs them.

## 11. Lifecycle and state

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

The execution state should be sufficient to support bounded deterministic runtime mechanics without requiring a persistent database.

## 12. Budgets and bounded execution

MOTE MUST support explicit limits for autonomous execution. At minimum, implementations should be able to bound:

- iterations/steps
- wall-clock execution time
- model usage where measurable
- capability/resource usage where measurable

A workload may define stricter budgets. Exceeding a budget is a normal terminal condition, not an exceptional framework failure.

Budget enforcement belongs in the runtime because it is part of safe, predictable execution rather than an optional application feature.

## 13. Security model

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

An agent specification is descriptive; authorization is an independent runtime concern.

## 14. Extensibility tiers

MOTE should grow through tiers rather than increasing the minimum runtime.

### Tier 0 — Kernel

Spec loader, model interface, execution loop, action protocol, policy boundary, environment interface, state, budgets, events, cancellation.

### Tier 1 — Basic worker

Filesystem, shell, simple process execution.

### Tier 2 — Domain worker

Capabilities appropriate to a particular workload, such as git/tests for software work or datasets/evaluators for model benchmarking.

### Tier 3 — Connected worker

HTTP, MCP, databases, browsers, external APIs, GitHub, remote workers.

### Tier 4 — Advanced agent

Memory, sophisticated planning, subagents, long-running workflows, computer use, persistent sessions, autonomous scheduling.

The existence of Tier 4 MUST NOT force Tier 0 users to install Tier 4 components.

## 15. Benchmark agents as a first-class validation target

MOTE is intentionally well suited to agents whose job is to evaluate models rather than write software.

For example, an LLM benchmarking worker can be composed as:

```text
MOTE
├── model adapter / dmr-X
├── task loop
├── benchmark dataset capability
├── evaluator / metrics capability
└── results capability
```

It does not need to carry:

```text
browser
LSP
GitHub
repository map
coding-specific session system
TUI
subagents
persistent memory
```

unless the benchmark actually requires them.

This demonstrates the core MOTE property: **the runtime remains constant while the capability set changes with the workload**.

MOTE should therefore be evaluated not only as a coding-agent foundation but also as a general worker runtime for benchmark, research, automation, CI, and other narrowly scoped agents.

## 16. Interoperability

MOTE SHOULD expose multiple interfaces without making any one of them mandatory:

- CLI
- local IPC
- HTTP API
- streaming events
- library API
- MCP adapter
- agent-to-agent protocol adapters

Interfaces should be layered so a minimal embedded use does not require a network server or heavyweight protocol stack.

## 17. Design invariants

The implementation MUST preserve these invariants:

1. The model is replaceable.
2. Capabilities are replaceable.
3. The environment is replaceable.
4. Policy is separate from model instructions.
5. The kernel remains usable without optional extensions.
6. An agent specification does not equal authorization.
7. A task can be executed without requiring a heavyweight coding-agent application.
8. MOTE remains independently deployable from GLUE and other ecosystem projects.
9. Non-coding workloads MUST NOT pay for coding-specific infrastructure.
10. Optional functionality MUST remain composable rather than becoming mandatory kernel surface area.
11. MOTE's execution mechanics MUST remain deterministic and bounded even when model reasoning is probabilistic.

## 18. YAGNI and minimality

**YAGNI (You Aren't Gonna Need It) is a core MOTE engineering principle, not a runtime operation.**

MOTE should build the smallest general mechanism justified by current requirements. Features presumed to be useful someday should not be added to the kernel merely because they may become useful later.

Therefore:

- speculative capabilities belong outside the kernel;
- new abstractions require a demonstrated current use case;
- optional functionality should be implemented as an extension where practical;
- interfaces should be kept narrow;
- complexity should be paid for only when the workload needs it.

MOTE MUST NOT implement a `yagni()`-style runtime heuristic. Whether a feature is necessary is an engineering/design judgment, not something the execution kernel can reliably infer.

YAGNI does **not** justify omitting requirements that are already necessary for security, isolation, correctness, data integrity, or the stated MOTE execution contract.

Minimality is therefore an architectural property, not merely a temporary lack of features.

## 19. Relationship to GLUE

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

## 20. Relationship to the wider stack

MOTE is intended to be a reusable low-level execution primitive.

```text
ATHENA        orchestration/governance
GLUE          agent citizenship/coordination
NOESIS        memory substrate
DANNY         digital self
DMR-X         model and worker routing
Ghost Factory software transformation
MOTE          lightweight agent embodiment/execution kernel
```

These systems may integrate with MOTE through explicit interfaces rather than making MOTE responsible for their domains.

A typical software worker may therefore look like:

```text
Ghost Factory / ATHENA
          |
         MOTE
       /  |  \
    dmr-X  Git  Sandbox
              \
             Tests
```

while an LLM benchmark worker can be:

```text
Benchmark Agent
      |
     MOTE
   /   |    \
Model Dataset Evaluator
```

The same kernel serves both without requiring the benchmark worker to become a coding agent.

## 21. Success criteria

The first implementation is successful if it can:

- load an agent specification;
- accept a task;
- invoke a replaceable model;
- execute a permitted action;
- observe the result;
- iterate to completion;
- enforce budgets and cancellation;
- run inside an isolated environment;
- operate with only the capabilities actually needed;
- expose structured lifecycle events;
- remain useful without any heavyweight coding-agent framework;
- support at least one non-coding workload, preferably an LLM/model benchmarking worker, using no coding-specific dependencies.

MOTE should also be measured on the properties that motivate its existence:

- startup latency
- idle memory/RSS
- binary size
- dependency footprint
- execution overhead
- capability invocation overhead
- reproducibility of runtime behavior

These metrics should be tracked without turning benchmarking infrastructure into a mandatory kernel dependency.

## 22. Future work

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
- richer benchmark/evaluation protocols

These should be added only when demonstrated necessary. The existence of a plausible future use case is not sufficient justification for increasing the kernel's minimum surface area.

**MOTE's goal is not to compete with complete coding-agent applications by accumulating features. Its goal is to make the smallest useful execution kernel from which those applications and many other specialized workers can be composed.**
