# MOTE Design Principles

## 1. The kernel is a constraint, not a feature marketplace

Every capability added to the kernel increases complexity, testing burden, attack surface, and maintenance cost. MOTE should therefore keep a small number of primitives exceptionally stable.

## 2. YAGNI

YAGNI means **You Aren't Gonna Need It**. It is an engineering heuristic associated with Extreme Programming's simple-design practice: avoid implementing functionality merely because a future requirement is imaginable.

For MOTE:

> **Do not implement speculative capabilities. Do implement the smallest abstractions already required to preserve the current architecture.**

YAGNI is not a ban on architecture. It is a warning against paying complexity costs for unvalidated future benefits.

## 3. Extensibility is a present requirement

Because MOTE's purpose is to be extensible, a small extension contract is not YAGNI. What would violate YAGNI is creating dozens of future-specific extension systems before a real consumer exists.

The target is one coherent capability contract that can support the first real extensions. New extension categories should appear when actual requirements justify them.

## 4. Prefer protocols over knowledge

The kernel should know what a capability can do through a contract, not how a particular integration works. Git, LSP, browser, MCP, dmr-X, NOESIS, and external services should remain outside the kernel's business logic whenever possible.

## 5. Deterministic mechanics around probabilistic reasoning

The model can propose. The runtime validates named grants and paths for its built-in file actions, executes, records, and enforces budgets. This is not a policy engine or OS sandbox; permitted processes retain the user's host permissions.

## 6. Add architecture at the last responsible moment

When a requirement becomes concrete, introduce the smallest general structure that addresses it. Do not confuse "we might need X" with "we need an abstraction for X today."

## 7. Easy to remove

A MOTE component is healthier when it can be removed or replaced without restructuring the kernel. This is a practical test for unnecessary coupling.

## 8. Extension-pressure test

Before adding a kernel feature, ask:

- What current task requires it?
- Why cannot an extension provide it?
- What concrete complexity does it remove?
- What dependency, attack surface, and maintenance burden does it add?
- Can the decision be deferred until a real consumer exists?

If the primary justification is "a future user may want it," the default is to defer it.

## 9. Exceptions to strict YAGNI

Do not use YAGNI to postpone foundations that are intrinsically expensive or unsafe to retrofit, including security boundaries, data-loss protection, execution isolation required by the threat model, and stable interfaces already consumed by external systems.

The principle is not "never prepare." It is "do not build unvalidated product functionality in advance."

## 10. YAGNI must not become a runtime operation

A runtime operation such as `yagni()` would require the system to infer whether a proposed feature is genuinely needed. That is a strategic planning judgment rather than an objective execution property.

A dedicated YAGNI runtime feature would also introduce its own policy, historical evidence, cost modeling, and decision logic, increasing the very complexity MOTE is designed to avoid.

Therefore MOTE treats YAGNI as:

1. a project-level engineering rule;
2. an architecture/admission criterion for the kernel;
3. an optional agent planning heuristic;
4. a code-review criterion.

## 11. MOTE rule

**Core:** minimum mechanisms with demonstrated or structurally necessary value.

**Extensions:** specialized, optional, provider-specific, or deployment-specific behavior.

**Policy:** YAGNI governs admission to the core; it is not itself a core runtime capability.
