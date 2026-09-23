# MOTE SDK Specification

**Not implemented as a standalone SDK.** This is a design target, not a promise of stable extension/registration APIs. The actual public Rust modules are exported by `src/lib.rs`; see [docs/STATUS.md](docs/STATUS.md).

**Status:** Design specification
**Audience:** MOTE runtime engineers, SDK implementers, and third-party agent developers
**Purpose:** Define the stable extension surface that allows developers to build and ship agents on top of the MOTE runtime without requiring access to MOTE's internal implementation.

---

## 1. Design Goal

The MOTE SDK is the public contract between the MOTE runtime and agents built on top of it.

MOTE should remain a small, general-purpose agent substrate. Domain-specific behavior belongs in applications, agents, tools, adapters, and extensions built against the SDK.

The SDK MUST therefore expose enough capability to build serious agents while avoiding leakage of unnecessary runtime internals.

Primary goals:

- Build an agent without modifying the MOTE runtime.
- Keep the core runtime minimal.
- Make the public API stable and versioned.
- Permit third-party agents under independently chosen licenses while MOTE itself remains MIT-licensed.
- Support local-first and remote model/tool execution.
- Support deterministic tools and model-driven reasoning without forcing either into the core.
- Permit specialized agents such as coding, code review, research, benchmarking, security, data, and domain agents.
- Make extensions composable without turning MOTE into a monolithic framework.

Non-goals:

- MOTE is not a complete coding agent.
- MOTE is not a workflow/orchestration platform.
- MOTE is not a memory database.
- MOTE is not a model provider.
- MOTE does not prescribe a specific UI, model, vector database, or agent architecture.

---

## 2. Conceptual Architecture

```text
                    MOTE Runtime
                         |
                 Public SDK Boundary
                         |
       +-----------------+------------------+
       |                 |                  |
     Agents            Tools              Adapters
       |                 |                  |
       +-----------------+------------------+
                         |
                Application / Product
                         |
      +------------------+-------------------+
      |                  |                   |
 MOTE Review       MOTE Research       MOTE Benchmark
```

The runtime owns execution mechanics. The application owns domain behavior.

---

## 3. Public SDK Surface

The initial SDK SHOULD expose the following conceptual modules:

```text
mote
├── agent
├── runtime
├── model
├── tool
├── context
├── state
├── event
├── extension
├── capability
├── error
└── version
```

The exact language/package structure is implementation-dependent, but the semantic contract MUST remain equivalent.

---

## 4. Agent API

An Agent represents a task-oriented reasoning/execution unit.

Minimum conceptual interface:

```text
Agent
  id
  metadata
  capabilities
  instructions
  run(input, context)
  stop()
```

An agent MUST be able to:

- Receive structured and/or textual input.
- Read relevant context.
- Request model inference.
- Invoke permitted tools.
- Emit events.
- Maintain run-scoped state.
- Return a structured result.
- Stop or be cancelled.

The SDK SHOULD support both:

1. Simple agents implemented as a single loop.
2. Advanced agents implementing custom control loops while using MOTE services.

MOTE MUST NOT require every agent to use a predefined planning pattern.

---

## 5. Runtime API

The runtime executes an agent under a defined policy and capability set.

Conceptual interface:

```text
Runtime
  create_run(agent, input, options)
  execute(run)
  cancel(run)
  inspect(run)
```

Runtime responsibilities SHOULD include:

- Lifecycle management.
- Tool dispatch.
- Model dispatch.
- Context delivery.
- Cancellation/timeouts.
- Event emission.
- Resource/policy enforcement.
- Error propagation.
- Run identity and metadata.

Domain-specific logic MUST remain outside the runtime where practical.

---

## 6. Model Interface

Models are capabilities consumed by agents, not hard-coded dependencies.

Conceptual interface:

```text
Model
  id
  capabilities
  generate(request)
  stream(request)
```

The model abstraction SHOULD support:

- Local models.
- Remote APIs.
- Multiple providers.
- Streaming.
- Tool/function calling where supported.
- Structured output.
- Model-specific metadata.
- Usage/cost/latency telemetry.

MOTE MUST NOT require a single model vendor.

A model router such as dmr-X MAY be implemented as a model provider/adapter from the SDK's perspective rather than being embedded into MOTE core.

---

## 7. Tool Interface

Tools provide deterministic or external capabilities to agents.

Conceptual interface:

```text
Tool
  id
  description
  input_schema
  output_schema
  capabilities
  execute(input, context)
```

Tools SHOULD be:

- Explicitly declared.
- Schema-described.
- Capability-scoped.
- Independently testable.
- Observable through events.
- Composable.

Examples:

```text
shell
filesystem
git
github
browser
database
http
static-analysis
test-runner
benchmark-runner
```

The tool system MUST support denying tools even when an agent requests them.

---

## 8. Context Interface

Context is the information made available to a run.

Conceptual structure:

```text
Context
  input
  instructions
  history
  artifacts
  tool_results
  references
  metadata
```

Context SHOULD be assembled incrementally rather than requiring an enormous static prompt.

The SDK MUST allow applications to provide their own context sources.

Examples:

- Repository context.
- Pull-request diff.
- Documents.
- Benchmark datasets.
- User-provided files.
- External API results.
- Memory retrieved from another system.

MOTE MUST NOT require a particular memory implementation.

---

## 9. State

State is run/application state distinct from model context.

The SDK SHOULD support:

- Ephemeral run state.
- Persisted application state through an application-defined backend.
- Checkpointing where supported.
- Metadata and provenance.

MOTE SHOULD define the state contract but SHOULD NOT force a database or storage technology.

External systems such as NOESIS/SMS MAY implement state or memory adapters.

---

## 10. Events

MOTE SHOULD expose a structured event stream for observability and integrations.

Conceptual events include:

```text
run.started
run.completed
run.failed
run.cancelled
model.requested
model.started
model.delta
model.completed
tool.requested
tool.started
tool.output
tool.completed
tool.failed
context.updated
state.changed
agent.message
agent.warning
```

Events SHOULD be streamable and serializable.

Applications MAY consume events for:

- UIs.
- Logging.
- Telemetry.
- Debugging.
- External orchestration.
- CI/CD integrations.
- Agent-to-agent communication.

---

## 11. Capabilities and Permissions

Capabilities define what an agent/run is allowed to access.

Examples:

```text
filesystem.read
filesystem.write
shell.execute
git.read
git.write
network.request
github.read
github.write
model.invoke
secret.read
```

Capabilities MUST be explicit enough to support least-privilege execution.

A tool SHOULD declare the capabilities it requires.

An agent SHOULD declare the capabilities it expects.

The runtime MUST enforce the effective permission set.

---

## 12. Extension API

Third-party developers MUST be able to add functionality without modifying MOTE internals.

An extension MAY provide:

- Agents.
- Tools.
- Models/providers.
- Context sources.
- State backends.
- Event sinks.
- Capability providers.
- Adapters.

Conceptual registration:

```text
Extension
  manifest
  register(runtime)
  unregister(runtime)
```

Extensions SHOULD be discoverable and versioned.

The runtime MUST treat extension code as untrusted unless explicitly granted execution privileges.

---

## 13. Application Boundary

A product built on MOTE SHOULD own its domain logic.

Example: a code-review agent:

```text
MOTE SDK
  + GitHub adapter
  + Git/diff tools
  + repository context provider
  + static-analysis tools
  + review policy
  + review agent
  + PR output adapter
        = MOTE Review
```

MOTE itself should not contain GitHub-specific review logic.

Similarly:

```text
MOTE + benchmark tools + model adapters + evaluator = MOTE Benchmark
MOTE + browser + research context + citation tools = MOTE Research
MOTE + shell + git + code tools = MOTE Coding Agent
```

---

## 14. Distribution and Licensing Boundary

MOTE itself is licensed under MIT; see [LICENSE](LICENSE). Third-party applications and agents may be distributed under licenses of their authors' choice, subject to MOTE's MIT notice and their own dependency terms. An SDK boundary is still a design target, not an implemented stable plug-in API.

```text
MIT-licensed MOTE runtime and Rust library
        |
   Public Rust API (pre-1.0, evolving)
        |
  Third-party agents with independent licenses
```

An application should be able to depend on the public Rust library without relying on MOTE's private implementation details. This is an API design goal, not a separate commercial licensing scheme. Consult qualified legal counsel for a particular distribution's license obligations.

---

## 15. Versioning

The SDK MUST use semantic versioning or an equivalent explicit compatibility policy.

Breaking changes MUST be identified clearly.

The SDK SHOULD provide:

- Runtime version discovery.
- SDK version discovery.
- Capability negotiation.
- API compatibility information.

Agents SHOULD be able to declare the SDK/runtime versions they support.

---

## 16. Error Model

Errors SHOULD be structured and machine-readable.

Conceptual categories:

```text
InvalidInput
PermissionDenied
ToolError
ModelError
ContextError
StateError
Timeout
Cancelled
ResourceExhausted
ExtensionError
RuntimeError
```

Errors SHOULD contain stable codes and useful metadata without exposing secrets or internal implementation details.

---

## 17. Streaming

The SDK SHOULD support streaming for:

- Model output.
- Tool output.
- Agent messages.
- Runtime events.

Streaming MUST preserve event ordering within a run where ordering is semantically required.

Applications MUST be able to consume a run incrementally without waiting for completion.

---

## 18. Testing Contract

The SDK SHOULD provide test utilities allowing developers to test agents independently from production models and external services.

Required concepts:

- Mock models.
- Mock tools.
- Deterministic event capture.
- Synthetic contexts.
- Run replay fixtures where feasible.

This allows an agent such as MOTE Review to test its review policy without requiring a live GitHub repository or expensive model call for every test.

---

## 19. Reference Agent: MOTE Review

The first serious proof-of-concept SHOULD be a CodeRabbit-class code-review agent built externally on the SDK.

It should demonstrate that MOTE can support:

1. GitHub PR ingestion.
2. Diff acquisition.
3. Repository exploration.
4. Context selection.
5. Static analysis.
6. Test execution.
7. Model reasoning.
8. Finding prioritization.
9. Structured review output.
10. Inline GitHub comments/reviews.
11. Streaming progress.
12. Configurable review policy.
13. Multiple model providers.
14. Sandboxed tool execution.

The implementation should live outside MOTE core.

Success criterion:

> A useful autonomous code-review product can be implemented entirely through the public MOTE SDK and extension surface, with no changes to MOTE core for domain-specific features.

---

## 20. Minimal First SDK

The first implementation SHOULD resist premature abstraction.

Minimum viable SDK:

```text
Agent
Runtime
Model
Tool
Context
Event
Capability
Error
Extension
```

Everything else should be added only when a real agent requires it.

This preserves the central MOTE principle:

> **Provide the minimum substrate necessary to build capable agents; put everything else above the substrate.**

---

## 21. Architectural Invariants

These MUST remain true as MOTE evolves:

1. MOTE core stays small.
2. Agents own domain behavior.
3. Tools are explicit capabilities.
4. Models are replaceable.
5. Context is composable.
6. State is backend-independent.
7. Extensions do not require core modification.
8. Runtime enforces permissions.
9. Events provide observability.
10. The SDK is the compatibility boundary.
11. Internal implementation details are not part of the public contract.
12. A serious agent must be possible without turning MOTE into a specialized platform.

---

## 22. Acceptance Test

MOTE SDK is considered viable when an independent developer can implement the following without modifying MOTE core:

```text
MOTE Review
  -> receive GitHub PR
  -> inspect repository and diff
  -> invoke analysis/test tools
  -> request model reasoning
  -> produce structured findings
  -> stream progress
  -> publish GitHub review
```

If this requires adding GitHub, code-review, or domain-specific logic to MOTE itself, the SDK boundary is insufficient and must be redesigned.
