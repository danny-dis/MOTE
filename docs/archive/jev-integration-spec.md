# Jev decision integration status

> **Historical integration/proposal snapshot — not current product documentation.** This file preserves a point-in-time integration note and does not establish live health, current behavior, or shipped features. For current MOTE documentation, see [STATUS](../STATUS.md), [BUILDING_ON_MOTE](../BUILDING_ON_MOTE.md), and [QUICKSTART](../QUICKSTART.md).

MOTE has an optional typed `DecisionProvider` action (`src/decision.rs`, `src/runtime.rs`). Jev and DMR-X adapters can answer choice/score/Noul requests; no Jev dependency is required for ordinary tasks. A configured denying choice (`deny`, `reject`, `block`, or `escalate`) may serve as a fail-closed fallback **only when it is a key in the request criteria**. Numeric score/Noul responses fail if the provider fails; MOTE does not invent numeric answers.

The historical proposal also asked for a benchmark against a generative classifier. No such benchmark exists in this repo. Provider availability and real-world decision quality have not been verified by the local unit tests. See `specs/decision-gate.yaml` for the current manifest shape and `docs/STATUS.md` for security limitations.
