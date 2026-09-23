# MOTE Engineering Paper

**Design note, not an implementation inventory.** Conceptual AGENT.md loading, extension contracts, remote environments, and isolation described below are targets. The current library/CLI and its limitations are documented in [README.md](README.md) and [docs/STATUS.md](docs/STATUS.md).

## Minimal Agent Embodiment Through a Capability-Oriented Runtime

### Abstract

MOTE (Minimal Orchestration & Task Execution) is a Rust-first, model-agnostic runtime for turning declarative agent specifications into executable workers. Its primary engineering objective is to minimize the mandatory runtime while preserving a clean path to sophisticated capabilities.

Rather than competing directly with feature-rich coding agents, MOTE isolates the irreducible agent loop from optional capabilities such as repository intelligence, browser automation, memory, MCP, subagents, and remote execution.

The resulting architecture is intended to make agent execution cheap enough for large numbers of ephemeral workers while remaining extensible enough to serve as a substrate for software engineering, research, automation, and agent-citizen systems.

---

## 1. Problem

Modern coding agents commonly combine several layers:

```text
model + prompt system + planner + tools + memory + repository intelligence
+ sandbox + integrations + UI + orchestration + provider management
```

This is convenient for end users but creates unnecessary weight when the desired workload is small.

A GitHub repository may contain a useful agent specification in Markdown, yet instantiating that specification often requires a complete coding-agent application.

MOTE separates these concerns.

The engineering question becomes:

> How little software is required to turn a specification into a bounded autonomous worker?

---

## 2. Architectural thesis

MOTE follows five principles.

### 2.1 The agent loop should be small

The core needs only enough machinery to repeatedly transform model output into authorized actions and observations.

### 2.2 Capabilities should be external

A shell, Git client, browser, database, memory system, or GitHub integration should not become a permanent dependency of every worker.

### 2.3 Specifications should be declarative

Markdown is valuable for human-readable behavior and YAML/JSON manifests are useful for machine-readable metadata. Neither should be confused with authorization.

### 2.4 Execution should be isolated

The runtime must treat model-generated actions as untrusted requests that require capability and environment validation.

### 2.5 Scale should come from composition

Sophistication should be created by composing a small runtime with capabilities, not by continuously enlarging the kernel.

---

## 3. Reference architecture

```text
                    +--------------------+
                    | Agent Specification|
                    | AGENT.md + manifest |
                    +---------+----------+
                              |
                              v
                  +-----------------------+
                  |       MOTE CORE       |
                  |-----------------------|
                  | specification loader  |
                  | context assembly      |
                  | model adapter         |
                  | execution loop        |
                  | action validation     |
                  | lifecycle/event bus   |
                  +-----------+-----------+
                              |
               +--------------+--------------+
               |                             |
               v                             v
      +----------------+            +------------------+
      | Model Adapter  |            | Capability Host  |
      +----------------+            +---------+--------+
                                             |
                         +-------------------+------------------+
                         |          |          |       |        |
                        FS        Shell       Git     HTTP     MCP
                         |          |          |       |        |
                         +-------------------+------------------+
                                             |
                                             v
                                    +----------------+
                                    | Environment    |
                                    | process/container|
                                    | microVM/remote |
                                    +----------------+
```

The architecture deliberately avoids requiring a global orchestration layer.

---

## 4. Execution semantics

A MOTE run is a state machine driven by task progress.

```text
SPEC + TASK
    |
    v
CONTEXT
    |
    v
MODEL INFERENCE
    |
    v
PROPOSED ACTION
    |
    v
POLICY / CAPABILITY VALIDATION
    |
    +---- denied ----> observation/error
    |
    v
EXECUTION
    |
    v
OBSERVATION
    |
    v
CONTEXT UPDATE
    |
    +---- continue ---> MODEL
    |
    +---- complete --> RESULT
```

The runtime must distinguish four concepts:

1. **Intent** — what the model wants to do.
2. **Authorization** — whether the requested operation is permitted.
3. **Execution** — what the environment actually performed.
4. **Observation** — what the environment returned.

This separation is fundamental to the security model.

---

## 5. Rust implementation strategy

Rust is the preferred implementation language because MOTE is infrastructure rather than a user-facing scripting framework.

Relevant properties include:

- low runtime overhead;
- fast process startup;
- strong memory and thread safety;
- efficient async I/O;
- good subprocess control;
- straightforward static/single-binary distribution;
- suitable primitives for concurrent workers;
- natural integration with containers, Linux processes, and systems infrastructure.

The implementation should resist dependency inflation. A small number of carefully selected crates should be preferred over a large framework dependency tree.

### Proposed workspace

```text
mote/
├── crates/
│   ├── mote-core/          # execution state machine
│   ├── mote-spec/          # specification parsing
│   ├── mote-model/         # model adapter traits
│   ├── mote-capability/    # capability interfaces
│   ├── mote-env/           # execution environments
│   ├── mote-events/        # structured event types
│   └── mote-cli/           # optional CLI
├── examples/
├── specs/
├── tests/
└── docs/
```

For the earliest prototype, these can remain a single crate. Splitting should happen only when boundaries stabilize.

---

## 6. Core traits

Conceptually, the runtime needs interfaces similar to:

```rust
trait Model {
    async fn infer(&self, request: ModelRequest) -> Result<ModelResponse>;
}

trait Capability {
    fn descriptor(&self) -> CapabilityDescriptor;
    async fn invoke(&self, request: CapabilityRequest) -> Result<CapabilityResponse>;
}

trait Environment {
    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult>;
}
```

The actual API should remain minimal until implementation experiments identify stable requirements.

---

## 7. Capability architecture

Capabilities are the principal extension mechanism.

A capability is a bounded interface to an external operation. It should expose machine-readable schemas and declared side effects.

Example:

```text
Capability: git
Operations:
  - status
  - diff
  - checkout
  - commit
Permissions:
  repository.read
  repository.write
```

A future registry may support local, dynamically loaded, remote, or WASM-backed capabilities.

The core should not care which implementation provides the capability.

---

## 8. Agent specifications as portable behavior

The specification layer allows repositories to carry their own agent definitions.

Example:

```text
repository/
├── AGENT.md
├── mote.yaml
└── src/
```

`AGENT.md` communicates identity, goals, constraints, and operational guidance.

`mote.yaml` can provide strict machine-readable configuration.

This enables a useful distinction:

```text
Specification = what the worker is
Task          = what the worker must do now
Capability    = what the worker may request
Policy        = what the worker is allowed to do
Environment   = where the action occurs
Model         = how the worker reasons
```

No individual layer needs to become the entire agent framework.

---

## 9. Context efficiency

MOTE should minimize context overhead as aggressively as it minimizes binary/runtime overhead.

The default context should contain only:

- relevant specification;
- current task;
- capability schemas required by the task;
- recent observations;
- explicitly selected repository/context information.

Repository mapping and semantic indexing should be optional extensions.

This allows a trivial task to remain trivial rather than paying the cost of a full codebase analysis pipeline.

---

## 10. Coding-agent profile

MOTE can implement a coding profile without turning the core into a coding-specific application.

A minimal coding worker might use:

```text
MOTE
 + filesystem
 + shell
 + git
 + repository context
```

A more capable worker might add:

```text
 + GitHub
 + tests
 + browser
 + MCP
 + memory
 + planning
```

The runtime remains identical.

This is the principal extensibility property.

---

## 11. Comparison with heavyweight coding agents

MOTE is not designed to win by reproducing every feature of mature coding environments.

The intended distinction is:

| System type | Primary objective |
|---|---|
| Full coding agent | Rich interactive software engineering |
| Agent framework | Building configurable agents |
| Orchestrator | Coordinating many workers |
| MOTE | Cheapest useful agent embodiment |

MOTE can use mature coding-agent ideas where they are valuable, particularly minimal agent loops, repository-aware context, extension systems, and capability abstraction. It should not inherit their entire application surface.

---

## 12. Scaling model

A key target is cheap ephemeral execution.

```text
                 Control system
                       |
          +------------+------------+
          |            |            |
        MOTE         MOTE         MOTE
       worker       worker       worker
          |            |            |
       shell         git          HTTP
```

Thousands of small workers should not require thousands of heavyweight application stacks.

A future scheduler can place workers on local processes, containers, microVMs, or remote execution nodes according to cost, risk, and latency.

---

## 13. Relationship to the ecosystem

MOTE is deliberately independent.

```text
                    +---------+
                    | ATHENA  |
                    +----+----+
                         |
                    schedules
                         |
                         v
+---------+        +----------+        +-----------+
|  GLUE   |------->|   MOTE   |<-------| Ghost     |
+---------+        +----------+        | Factory   |
     |                  ^              +-----------+
 citizens               |
                    +---+---+
                    | DMR-X |
                    +-------+
```

GLUE can use MOTE to embody discovered citizens. ATHENA can use MOTE as a worker substrate. Ghost Factory can use it for lightweight engineering operations. dmr-X can supply model selection/routing.

MOTE does not need to understand the higher-level semantics of those systems.

---

## 14. Security architecture

Security should be layered.

### Layer 1 — Specification

Declares desired behavior and requested capabilities.

### Layer 2 — Policy

Determines which requested operations are actually allowed.

### Layer 3 — Capability

Constrains what an operation can access.

### Layer 4 — Environment

Provides OS/container/microVM-level isolation and resource limits.

### Layer 5 — Audit

Records actions, authorization decisions, observations, and lifecycle transitions.

This prevents the common mistake of treating a system prompt as a security boundary.

---

## 15. Observability

Every run should expose structured events such as:

```text
run.created
spec.loaded
model.requested
model.response
action.proposed
action.authorized
action.denied
action.started
action.completed
observation.received
run.completed
run.failed
run.cancelled
```

Events should be usable locally or forwarded to an external observability/control system.

---

## 16. Performance objectives

The project should optimize for:

- minimal startup latency;
- low idle memory;
- low dependency count;
- efficient streaming;
- bounded context construction;
- low per-action overhead;
- high worker density;
- deterministic cancellation;
- efficient process reuse when desired.

Exact numerical targets should be established by benchmarks rather than guessed in the specification.

---

## 17. Development phases

### Phase 0 — Kernel prototype

Implement:

- specification loading;
- one model adapter;
- one environment;
- one generic action protocol;
- execution loop;
- event stream.

### Phase 1 — Useful worker

Add:

- filesystem;
- shell;
- Git;
- cancellation;
- resource limits;
- basic coding profile.

### Phase 2 — Extensibility

Add:

- capability registry;
- plugin packaging;
- provider adapters;
- remote environment interface.

### Phase 3 — Ecosystem integration

Add adapters for:

- GLUE;
- ATHENA;
- dmr-X;
- Ghost Factory;
- MCP;
- agent-to-agent protocols.

### Phase 4 — Advanced execution

Only if justified by benchmarks and real workloads:

- microVM environments;
- WASM capabilities;
- persistent workers;
- checkpointing;
- memory adapters;
- multi-agent execution.

---

## 18. Engineering rules

1. Do not add a feature to the kernel when it can be a capability.
2. Do not add persistent state when the caller can provide it.
3. Do not add an integration when an adapter can provide it.
4. Do not require a heavyweight framework to run a simple worker.
5. Benchmark before optimizing and benchmark before increasing complexity.
6. Keep security enforcement outside model-generated instructions.
7. Preserve model/environment/capability interchangeability.
8. Prefer stable protocols over framework-specific APIs.
9. Make failure and cancellation first-class.
10. Keep the minimal installation minimal.

---

## 19. Research foundation

MOTE's architecture is informed by recurring patterns in modern coding-agent systems:

- minimal agent-loop approaches demonstrate that useful coding behavior does not require a huge control framework;
- repository-aware context systems demonstrate the value of selective codebase representation;
- extension-oriented runtimes demonstrate that tools and integrations can remain outside the agent kernel;
- capability/toolset architectures demonstrate that functionality can be composed dynamically.

MOTE deliberately combines these lessons without attempting to reproduce any single existing agent.

---

## 20. Final thesis

The future agent stack does not need every worker to be a full coding agent.

Many workloads need only:

```text
specification
     +
small runtime
     +
model
     +
one or two capabilities
     +
bounded environment
```

MOTE exists to make that configuration a first-class engineering primitive.

**The goal is not the smallest agent with the fewest features. The goal is the smallest agent that can acquire exactly the features required by its task.**
