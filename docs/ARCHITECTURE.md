# Architecture

MOTE is a library first, with small command-line and cross-language entry points.
It does not own your application's users, storage, UI, scheduling, or deployment.

```text
Rust application ───────────────────────────────┐
                                               │
CLI → YAML manifest → model chain → runtime ←───┤
                                   │           │
Python / TypeScript → stdio bridge ─┘           │
                                   │           │
                          permitted tool calls │
                                   ↓           │
                     observation → next action ┘
```

## The execution loop

1. Load a trusted manifest and establish a workspace.
2. Ask the configured model for one structured action.
3. Check the named capability and applicable limits.
4. Execute the built-in action or application-owned handler.
5. Record the observation and include it in the next model prompt.
6. Stop on completion, cancellation, error, or an exhausted limit.

The model proposes actions; it does not register code or grant itself permissions.
Output validation remains the application's job. See the [Rust starter](../starters/rust-agent/README.md)
for an example that checks both callback execution and the resulting file.

## Source map

| Module | Responsibility |
|---|---|
| [`config.rs`](../src/config.rs) | Typed manifest and defaults |
| [`action.rs`](../src/action.rs) | Action data types |
| [`model.rs`](../src/model.rs) | Provider adapters, action parsing, fallback chain |
| [`runtime.rs`](../src/runtime.rs) | Execution loop, tools, shell/Git execution, budgets and state |
| [`capability.rs`](../src/capability.rs) | Workspace file/directory operations and path validation |
| [`decision.rs`](../src/decision.rs) | Optional typed decision providers and fallback |
| [`event.rs`](../src/event.rs) | Lifecycle event representation |
| [`main.rs`](../src/main.rs) | YAML CLI and post-run JSONL output |
| [`mote-bridge.rs`](../src/bin/mote-bridge.rs) | One-task, bidirectional stdio bridge |

## Language boundaries

**Rust** embeds the public library directly. `Model` supplies actions and `CustomTool`
registers application handlers. A Rust application does not need the bridge.

**Python and TypeScript** launch one bridge process for one run. The bridge asks the
host for custom-tool results and reuses the Rust runtime's checks. The SDKs validate
frames, enforce child-process deadlines, and reconcile terminal results with exit
status. There is no HTTP worker service or duplicated agent loop in the SDKs.

Registration and permission are separate: a tool must be registered by the host and
granted by the trusted manifest. The bridge carries data, not dynamically loaded code.

## Boundaries that matter

- Workspaces constrain built-in file operations, not every possible host operation.
- Shell, Git, and custom handlers retain host-user privileges.
- Timeouts cannot safely interrupt arbitrary synchronous application callbacks or
  guarantee that all descendant processes exit.
- CLI events and the SDK event list are delivered after the run; callbacks are the
  interactive part of the bridge protocol.
- Deterministic tests check mechanics. Live model responses and business outcomes
  require separate checks in the intended deployment.

Read [Security](../SECURITY.md) before adding capabilities or using untrusted content.

## Design principles

Keep a small, stable execution kernel. **YAGNI** (You Aren't Gonna Need It) means
not adding functionality for hypothetical future consumers. It does not mean
avoiding architecture that current users already need.

- Add the smallest extension contract justified by a real application. Prefer
  narrow protocols over provider-specific knowledge in the kernel.
- Keep specialized integrations and deployment behavior in application code.
  Components should be replaceable without restructuring the execution loop.
- Put deterministic checks around probabilistic model proposals: named grants,
  workspace validation, observations, and budgets are mechanics, not an OS sandbox.
- Do not postpone security boundaries required by the threat model, data-loss
  protection, or stable interfaces already consumed by other systems.
- Treat YAGNI as an engineering and review criterion, not a runtime capability.

Before adding a core feature, identify the current task it enables, why an
application tool cannot supply it, what complexity it removes, and its dependency,
security, and maintenance costs. If there is no concrete consumer, defer it.
