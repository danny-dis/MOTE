# Jev Integration Specification

## Summary
Extends MOTE with an optional Decision Provider while keeping the substrate lightweight.

## Implementation
- Add minimal DecisionProvider interface for typed choice/score outputs.
- Do not make Jev a mandatory dependency.
- Allow templates to declare bounded decision requirements such as classification, tool selection, safety gate, or escalation.
- Use DMR-X as the preferred runtime integration.
- Provide deterministic fallback behavior.
- Add a small benchmark against a generative classifier.
- Keep model-specific types out of the core API.

## Acceptance criteria
- MOTE works with zero decision-model dependencies.
- Agents can opt into bounded decisions through a stable interface.
- Jev remains an adapter, not a core dependency.
